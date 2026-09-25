//! Exact runtime, schema and migration readback. No optimistic compatibility.

use super::{CutPoint, Error, Result, digest, digest_text};
use rusqlite::types::ValueRef;
use rusqlite::{Connection, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

/// One migration: the whole file, which executes, and the digest of its body below the anchor
/// block, pinned as a literal (A01, per file). A step that rebuilds a table names it in
/// `preserves`, and the upgrade compares that table's rows before and after the step.
pub(super) struct Migration {
    pub(super) sql: &'static str,
    pub(super) body: &'static str,
    pub(super) preserves: &'static [Preserved],
}

/// A table a migration rebuilds or extends, with the order its rows are digested in (its primary
/// key) and the columns compared: `*` for a rebuild, which must keep every column, or the pre-step
/// columns for a step that adds one (B14-P2c: `SELECT *` would count the new column and refuse).
pub(super) struct Preserved {
    pub(super) table: &'static str,
    pub(super) order: &'static str,
    pub(super) columns: &'static str,
}

/// THE ordered migration chain (A25; RC06/T04): the one door for migration identity. Version `k`
/// is `MIGRATIONS[k - 1]`; a ledger records `k` rows linked by their predecessor columns and
/// `user_version = k`. Only an appended entry may follow a released one.
const MIGRATIONS: [Migration; 4] = [
    Migration {
        sql: include_str!("../../migrations/001.sql"),
        body: "sha256:ac5916feaee05749404dd7d87d98cde7e2ae93048e8b07e133ba7868fc1ee9f2",
        preserves: &[],
    },
    Migration {
        sql: include_str!("../../migrations/002.sql"),
        body: "sha256:bcd3de842dddb090ba6ef208b825d7764ca7d7b8326e0a18394fcc784ce24d4a",
        preserves: &[Preserved {
            table: "operations",
            order: "principal_uid,principal_role,action,version,request_key",
            columns: "*",
        }],
    },
    Migration {
        sql: include_str!("../../migrations/003.sql"),
        body: "sha256:4b4e9a7e06fa50f26bc8e74af58cd444fe8a6cfca47b7b85e4a598743bbf2d7d",
        preserves: &[
            Preserved {
                table: "operations",
                order: "principal_uid,principal_role,action,version,request_key",
                columns: "*",
            },
            Preserved {
                table: "task_stops",
                order: "task_id",
                columns: "*",
            },
        ],
    },
    Migration {
        sql: include_str!("../../migrations/004.sql"),
        body: "sha256:8476c3229448ef8da34f62e8fe75391bf4246e2a6cb73f04c81f0aba3f13382f",
        preserves: &[Preserved {
            table: "tasks",
            order: "id",
            columns: "id,principal_uid,principal_role,spec,criteria_digest,generation,state,\
                      cancellation,accepted_event,limit_ms,spent_ms,reserved_work_ms,reserved_verify_ms",
        }],
    },
];

/// The version a current ledger records: the chain's length.
pub(super) const CURRENT: u32 = 4;
const _: () = assert!(MIGRATIONS.len() == CURRENT as usize);

/// Which clause of the migration chain a ledger (or this binary) fails, at which version (A25).
/// Every clause refuses by its own name, so a refusal says what differs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Chain {
    /// This binary's migration file no longer has its pinned body digest.
    File { version: u32 },
    /// The ledger records a later version than this binary knows: no downgrade.
    Newer { recorded: u32, current: u32 },
    /// `user_version` and the number of history rows disagree.
    History { recorded: u32, rows: u32 },
    /// The history row at `position` is not version `position`: a gap or a reordering.
    Sequence { position: u32 },
    /// History row `version` names another body than migration `version`'s.
    Checksum { version: u32 },
    /// History row `version` does not link to version `version - 1` and its body.
    Predecessor { version: u32 },
    /// History row `version` was applied by another package.
    Package { version: u32 },
    /// The ledger's schema is not what applying migrations `1..=version` produces.
    Schema { version: u32 },
    /// Upgrade step `version` changed the rows of a table it must preserve.
    Preserved { version: u32 },
}

fn chain(fault: Chain) -> Error {
    Error::Chain(fault)
}

/// The chain's entries with their versions, `1..=CURRENT`.
fn migrations() -> impl Iterator<Item = (u32, &'static Migration)> {
    (1..=CURRENT).zip(MIGRATIONS.iter())
}

/// Migration `version`, or the named refusal a version outside the chain earns.
fn migration(version: u32) -> Result<&'static Migration> {
    migrations()
        .find(|(known, _)| *known == version)
        .map(|(_, migration)| migration)
        .ok_or(chain(Chain::Newer {
            recorded: version,
            current: CURRENT,
        }))
}

/// Every migration file still has its pinned body: the binary's own chain is intact.
fn files_intact() -> Result<()> {
    intact(&MIGRATIONS)
}

/// Each of `chain`'s files has its pinned body; version `k` is `chain[k - 1]`.
pub(super) fn intact(list: &[Migration]) -> Result<()> {
    for (version, migration) in (1..).zip(list) {
        if identity(migration.sql)? != migration.body {
            return Err(chain(Chain::File { version }));
        }
    }
    Ok(())
}

/// Record `version` in the history, linked to its predecessor, as applied by this package.
fn record(connection: &Connection, version: u32) -> Result<()> {
    let body = migration(version)?.body;
    let predecessor = match version.checked_sub(1) {
        Some(previous) if previous > 0 => Some(migration(previous)?.body),
        _ => None,
    };
    connection.execute(
        "INSERT INTO migration_history VALUES(?,?,?,?,?)",
        params![version, body, version - 1, predecessor, PACKAGE],
    )?;
    Ok(())
}

/// A table's row count and the digest of every value of every row, in primary-key order, each
/// value tagged with its storage type so no two different rows can hash alike.
fn preserved_state(connection: &Connection, table: &Preserved) -> Result<(u64, String)> {
    let mut statement = connection.prepare(&format!(
        "SELECT {} FROM {} ORDER BY {}",
        table.columns, table.table, table.order
    ))?;
    let width = statement.column_count();
    let mut rows = statement.query([])?;
    let mut hash = Sha256::new();
    let mut count = 0_u64;
    while let Some(row) = rows.next()? {
        count += 1;
        for index in 0..width {
            match row.get_ref(index)? {
                ValueRef::Null => hash.update(b"n"),
                ValueRef::Integer(value) => {
                    hash.update(b"i");
                    hash.update(value.to_be_bytes());
                }
                ValueRef::Real(value) => {
                    hash.update(b"r");
                    hash.update(value.to_be_bytes());
                }
                ValueRef::Text(bytes) => {
                    hash.update(b"t");
                    hash.update((bytes.len() as u64).to_be_bytes());
                    hash.update(bytes);
                }
                ValueRef::Blob(bytes) => {
                    hash.update(b"b");
                    hash.update((bytes.len() as u64).to_be_bytes());
                    hash.update(bytes);
                }
            }
        }
    }
    Ok((count, digest_text(&hash.finalize())))
}

fn preserved_states(connection: &Connection, migration: &Migration) -> Result<Vec<(u64, String)>> {
    migration
        .preserves
        .iter()
        .map(|table| preserved_state(connection, table))
        .collect()
}

/// Apply upgrade step `version` in its own transaction: the migration, a check that every table
/// it rebuilds kept exactly its rows, the history row and `user_version`. Any failure rolls the
/// whole step back, leaving the ledger as it was (A25; RC06).
pub(super) fn step(
    connection: &mut Connection,
    version: u32,
    fault: super::Fault,
    deadline: Instant,
) -> Result<()> {
    step_with(connection, version, migration(version)?, fault, deadline)
}

/// [`step`] with the migration handed in: the chain's own for every production step.
pub(super) fn step_with(
    connection: &mut Connection,
    version: u32,
    migration: &Migration,
    fault: super::Fault,
    deadline: Instant,
) -> Result<()> {
    bound(connection, deadline)?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let before = preserved_states(&tx, migration)?;
    tx.execute_batch(migration.sql)?;
    cut_point!(fault, CutPoint::MigrationStep);
    if preserved_states(&tx, migration)? != before {
        return Err(chain(Chain::Preserved { version }));
    }
    record(&tx, version)?;
    tx.pragma_update(None, "user_version", version)?;
    tx.commit()?;
    Ok(())
}
/// The corpus publisher owns every byte up to and including this line and rewrites it without
/// changing any DDL, so a migration's identity is the digest of what follows it.
const ANCHOR_END: &str = "-- HEE3-ANCHORS-END\n";
const APPLICATION_ID: i64 = 0x4845_4533;
const PACKAGE: &str = concat!("hee3-draft-schema1/", env!("CARGO_PKG_VERSION"));
const SOURCE_ID: &str =
    "2026-07-24 19:02:57 bf7c7f30031888f4e796e429ab3978879485813aaca6f641c7b33e4e09459bcc";

/// Digest of the migration body below the single publisher-owned anchor block. The whole file is
/// executed, so the excluded block must be unable to execute anything: every line of it is a `--`
/// comment or blank. A missing or repeated end marker is refused: a second block could otherwise
/// hide a DDL change.
pub(super) fn identity(sql: &str) -> Result<String> {
    match sql.split_once(ANCHOR_END) {
        Some((block, body))
            if !body.contains(ANCHOR_END)
                && block
                    .lines()
                    .all(|line| line.is_empty() || line.starts_with("--")) =>
        {
            Ok(digest(body.as_bytes()))
        }
        _ => Err(Error::UnsupportedSchema),
    }
}

pub(super) fn runtime(deadline: Instant) -> Result<()> {
    super::remaining(deadline)?;
    if rusqlite::version() != "3.53.4" {
        return Err(Error::Runtime);
    }
    let connection = Connection::open_in_memory()?;
    protect(&connection, deadline)?;
    let source: String = connection.query_row("SELECT sqlite_source_id()", [], |row| row.get(0))?;
    if source != SOURCE_ID {
        return Err(Error::Runtime);
    }
    for option in [
        "THREADSAFE=1",
        "DEFAULT_FOREIGN_KEYS",
        "DQS=0",
        "OMIT_LOAD_EXTENSION",
    ] {
        let enabled: bool =
            connection.query_row("SELECT sqlite_compileoption_used(?)", [option], |row| {
                row.get(0)
            })?;
        if !enabled {
            return Err(Error::Runtime);
        }
    }
    Ok(())
}

pub(super) fn protect(connection: &Connection, deadline: Instant) -> Result<()> {
    bound(connection, deadline)?;
    safe_settings(connection)
}

pub(super) fn bound(connection: &Connection, deadline: Instant) -> Result<()> {
    let left = super::remaining(deadline)?;
    connection.busy_timeout(left.min(Duration::from_secs(5)))?;
    connection.progress_handler(1000, Some(move || Instant::now() >= deadline))?;
    Ok(())
}

fn safe_settings(connection: &Connection) -> Result<()> {
    use rusqlite::config::DbConfig;
    for (setting, enabled) in [
        (DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true),
        (DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA, false),
        (DbConfig::SQLITE_DBCONFIG_DQS_DDL, false),
        (DbConfig::SQLITE_DBCONFIG_DQS_DML, false),
    ] {
        if connection.set_db_config(setting, enabled)? != enabled {
            return Err(Error::Runtime);
        }
    }
    connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA read_uncommitted=OFF; PRAGMA locking_mode=NORMAL; PRAGMA wal_autocheckpoint=1000;")?;
    Ok(())
}

pub(super) fn profile(connection: &Connection, deadline: Instant) -> Result<()> {
    bound(connection, deadline)?;
    safe_settings(connection)?;
    let mode: String = connection.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    if mode != "wal" {
        return Err(Error::Runtime);
    }
    connection.execute_batch("PRAGMA synchronous=FULL;")?;
    for (query, expected) in [
        ("PRAGMA synchronous", 2),
        ("PRAGMA foreign_keys", 1),
        ("PRAGMA read_uncommitted", 0),
        ("PRAGMA wal_autocheckpoint", 1000),
    ] {
        let value: i64 = connection.query_row(query, [], |row| row.get(0))?;
        if value != expected {
            return Err(Error::Runtime);
        }
    }
    let locking: String = connection.query_row("PRAGMA locking_mode", [], |row| row.get(0))?;
    if locking != "normal" {
        return Err(Error::Runtime);
    }
    Ok(())
}

/// Create a fresh ledger at the last migration, or validate an existing one's recorded chain.
/// Returns the version the ledger records.
pub(super) fn initialize(
    connection: &mut Connection,
    created: bool,
    generation: &str,
    epoch: &str,
    fault: super::Fault,
    deadline: Instant,
) -> Result<u32> {
    protect(connection, deadline)?;
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let app: i64 = connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    if !created {
        return validate(connection, generation, deadline);
    }
    let objects: i64 =
        connection.query_row("SELECT count(*) FROM sqlite_schema", [], |row| row.get(0))?;
    if version != 0 || app != 0 || objects != 0 {
        return Err(Error::UnsupportedSchema);
    }
    safe_settings(connection)?;
    files_intact()?;
    // A fresh ledger goes straight to the last migration, applying each in order in one
    // transaction, so its schema text is exactly what an upgrade through every step produces.
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    for (version, migration) in migrations() {
        tx.execute_batch(migration.sql)?;
        if version == 1 {
            cut_point!(fault, CutPoint::MigrationWrite);
        }
        record(&tx, version)?;
    }
    tx.execute(
        "INSERT INTO ledger_meta VALUES(1,?,?,'normal')",
        params![epoch, generation],
    )?;
    tx.pragma_update(None, "application_id", APPLICATION_ID)?;
    tx.pragma_update(None, "user_version", CURRENT)?;
    tx.commit()?;
    validate(connection, generation, deadline)
}

type SchemaRow = (String, String, String, Option<String>);
fn schema_rows(connection: &Connection) -> Result<Vec<SchemaRow>> {
    let mut statement = connection
        .prepare("SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name LIMIT 65")?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if rows.len() >= 65 {
        return Err(Error::UnsupportedSchema);
    }
    Ok(rows)
}

type HistoryRow = (i64, String, i64, Option<String>, String);

/// The recorded chain, clause by clause: `user_version` within this binary's chain, one history
/// row per version, each row version `k` naming migration `k`'s body, linked to `k - 1`, applied
/// by this package, and a schema equal to applying migrations `1..=k`. Returns `k`.
fn recorded_chain(connection: &Connection, deadline: Instant) -> Result<u32> {
    files_intact()?;
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let recorded = u32::try_from(version).map_err(|_| Error::UnsupportedSchema)?;
    if recorded == 0 {
        return Err(Error::UnsupportedSchema);
    }
    if recorded > CURRENT {
        return Err(chain(Chain::Newer {
            recorded,
            current: CURRENT,
        }));
    }
    let rows: i64 = connection.query_row("SELECT count(*) FROM migration_history", [], |row| {
        row.get(0)
    })?;
    if rows != i64::from(recorded) {
        return Err(chain(Chain::History {
            recorded,
            rows: u32::try_from(rows).unwrap_or(u32::MAX),
        }));
    }
    let mut statement = connection.prepare(
        "SELECT version,checksum,predecessor_version,predecessor_checksum,package_identity \
         FROM migration_history ORDER BY version LIMIT ?",
    )?;
    let history = statement
        .query_map([recorded], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?
        .collect::<std::result::Result<Vec<HistoryRow>, _>>()?;
    let mut predecessor: Option<&str> = None;
    for ((position, migration), (version, checksum, previous, previous_checksum, package)) in
        migrations().zip(history)
    {
        if version != i64::from(position) {
            return Err(chain(Chain::Sequence { position }));
        }
        if checksum != migration.body {
            return Err(chain(Chain::Checksum { version: position }));
        }
        if previous != i64::from(position - 1) || previous_checksum.as_deref() != predecessor {
            return Err(chain(Chain::Predecessor { version: position }));
        }
        if package != PACKAGE {
            return Err(chain(Chain::Package { version: position }));
        }
        predecessor = Some(migration.body);
    }
    let expected = Connection::open_in_memory()?;
    protect(&expected, deadline)?;
    for (_, migration) in migrations().take_while(|(version, _)| *version <= recorded) {
        expected.execute_batch(migration.sql)?;
    }
    if schema_rows(connection)? != schema_rows(&expected)? {
        return Err(chain(Chain::Schema { version: recorded }));
    }
    Ok(recorded)
}

/// Validate a ledger: its recorded migration chain (a valid prefix of this binary's), its
/// generation, integrity and logical invariants. Returns the version it records; the caller
/// decides which version it may use (`Store::open` requires the last one).
pub(super) fn validate(
    connection: &Connection,
    generation: &str,
    deadline: Instant,
) -> Result<u32> {
    protect(connection, deadline)?;
    let app: i64 = connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    if app != APPLICATION_ID {
        return Err(Error::UnsupportedSchema);
    }
    let recorded = recorded_chain(connection, deadline)?;
    let actual: String = connection.query_row(
        "SELECT generation FROM ledger_meta WHERE singleton=1",
        [],
        |row| row.get(0),
    )?;
    if actual != generation {
        return Err(Error::Conflict);
    }
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(Error::Corrupt);
    }
    if connection
        .prepare("PRAGMA foreign_key_check")?
        .query([])?
        .next()?
        .is_some()
    {
        return Err(Error::Corrupt);
    }
    let invalid:bool=connection.query_row("SELECT EXISTS(SELECT 1 FROM tasks t WHERE (accepted_event IS NOT NULL)!=(state='accepted') OR (accepted_event IS NOT NULL AND NOT EXISTS(SELECT 1 FROM acceptances a JOIN outbox o ON o.event_id=a.event_id WHERE a.event_id=t.accepted_event AND a.task_id=t.id)))",[],|row|row.get(0))?;
    if invalid {
        return Err(Error::Corrupt);
    }
    let roster_invalid:bool=connection.query_row("SELECT EXISTS(SELECT 1 FROM roster_records r LEFT JOIN roster_revisions v ON v.record_id=r.id AND v.revision=r.revision WHERE v.record_id IS NULL OR r.definition!=v.definition OR r.disabled!=v.disabled) OR EXISTS(SELECT 1 FROM roster_records r JOIN roster_observations o ON o.id=r.observation_id WHERE o.record_id!=r.id OR o.revision!=r.revision OR o.instance_id IS NOT NULL) OR EXISTS(SELECT 1 FROM roster_instances i JOIN attempts a ON a.id=i.attempt_id WHERE i.task_id!=a.task_id) OR EXISTS(SELECT 1 FROM roster_cancel_causes c JOIN attempts a ON a.id=c.attempt_id JOIN events e ON e.id=c.event_id WHERE c.task_id!=a.task_id OR e.roster_id!=c.record_id)",[],|row|row.get(0))?;
    if roster_invalid {
        return Err(Error::Corrupt);
    }
    Ok(recorded)
}
