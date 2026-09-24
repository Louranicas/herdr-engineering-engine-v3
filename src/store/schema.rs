//! Exact runtime, schema and migration readback. No optimistic compatibility.

use super::{CutPoint, Error, Result, check_point, digest};
use rusqlite::{Connection, TransactionBehavior, params};
use std::time::{Duration, Instant};

const SQL: &str = include_str!("../../migrations/001.sql");
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

pub(super) fn initialize(
    connection: &mut Connection,
    created: bool,
    generation: &str,
    epoch: &str,
    fault: Option<CutPoint>,
    deadline: Instant,
) -> Result<()> {
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
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute_batch(SQL)?;
    check_point(fault, CutPoint::MigrationWrite)?;
    tx.execute(
        "INSERT INTO migration_history VALUES(1,?,0,NULL,?)",
        params![identity(SQL)?, PACKAGE],
    )?;
    tx.execute(
        "INSERT INTO ledger_meta VALUES(1,?,?,'normal')",
        params![epoch, generation],
    )?;
    tx.pragma_update(None, "application_id", APPLICATION_ID)?;
    tx.pragma_update(None, "user_version", 1)?;
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

pub(super) fn validate(connection: &Connection, generation: &str, deadline: Instant) -> Result<()> {
    protect(connection, deadline)?;
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let app: i64 = connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    if version != 1 || app != APPLICATION_ID {
        return Err(Error::UnsupportedSchema);
    }
    let expected = Connection::open_in_memory()?;
    protect(&expected, deadline)?;
    expected.execute_batch(SQL)?;
    if schema_rows(connection)? != schema_rows(&expected)? {
        return Err(Error::UnsupportedSchema);
    }
    let count: i64 = connection.query_row("SELECT count(*) FROM migration_history", [], |row| {
        row.get(0)
    })?;
    let history:(i64,String,i64,Option<String>,String)=connection.query_row("SELECT version,checksum,predecessor_version,predecessor_checksum,package_identity FROM migration_history",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?)))?;
    if count != 1 || history != (1, identity(SQL)?, 0, None, PACKAGE.to_owned()) {
        return Err(Error::UnsupportedSchema);
    }
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
    Ok(())
}
