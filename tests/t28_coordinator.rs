//! T28 coordinator cases (a module of `t28_actions`): the active-generation manifest, startup
//! reconciliation composed into `serve`, and the `health` it serves (review D-C3 step 2).
//!
//! Every state root here is a scratch directory; nothing touches `$HOME/.local/state`.
use habitat_engine::app::coordinator::{
    self, ACTIVE_MANIFEST, ACTIVE_SCHEMA, Active, CommissionError, Commissioned, Unselected,
    commission, health_of, leaves_work_outstanding, startup_line,
};
use habitat_engine::app::startup::{Counts, Cursor, CursorEntry, LedgerAccess, Pass};
use habitat_engine::contracts::UuidV4;
use habitat_engine::contracts::control::{Database, Health, Recovery, Socket};
use habitat_engine::recovery::{
    CursorRefusal, Decision, Mode, ProcessCustody, Reconciliation, Rule, Unknown,
};
use habitat_engine::store::{Error as StoreError, Principal, Store};
use serde_json::json;
use std::error::Error;
use std::fs::{self, DirBuilder};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

type Outcome = Result<(), Box<dyn Error>>;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const GENERATION: &str = "28c00000-0000-4000-8000-000000000001";
const EPOCH: &str = "28c00000-0000-4000-8000-000000000002";
const CHECKED: u64 = 1_790_000_123_456;

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Result<Self, Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!(
            "hee3-t28c-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path)?;
        Ok(Self(path))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn manifest(root: &Path, body: &serde_json::Value, mode: u32) -> Outcome {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(root.join(ACTIVE_MANIFEST))?;
    file.write_all(serde_json::to_string(body)?.as_bytes())?;
    fs::set_permissions(root.join(ACTIVE_MANIFEST), fs::Permissions::from_mode(mode))?;
    Ok(())
}

fn selecting(generation: &str, epoch: &str) -> serde_json::Value {
    json!({"schema": ACTIVE_SCHEMA, "generation": generation, "epoch": epoch})
}

/// A state root holding a real, empty ledger for the selected generation, made by the one
/// commissioning door (OPS-1): the manifest every case below reads is the one writer's.
fn commissioned(scratch: &Scratch) -> Result<PathBuf, Box<dyn Error>> {
    let root = scratch.0.join("state");
    commission(
        &root,
        Active {
            generation: UuidV4::parse(GENERATION)?,
            epoch: UuidV4::parse(EPOCH)?,
        },
        Instant::now() + Duration::from_secs(10),
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(root)
}

/// What the manifest under `root` selects, as owned text for comparison.
fn select(root: &Path) -> Result<(String, String), Unselected> {
    let manifest = coordinator::read_manifest(root)?;
    let active = manifest.active()?;
    Ok((
        active.generation.as_str().to_owned(),
        active.epoch.as_str().to_owned(),
    ))
}

/// The health and line a start under `root` leaves, reading its manifest as `serve` does.
fn observe(root: &Path, deadline: Instant) -> (Health, String) {
    let manifest = coordinator::read_manifest(root);
    let started = coordinator::observe_at_start(root, &manifest, CHECKED, deadline);
    (started.health, started.line)
}

#[test]
fn a_generation_is_selected_only_from_the_operators_private_manifest() -> Outcome {
    let scratch = Scratch::new()?;
    assert_eq!(select(&scratch.0.join("absent")), Err(Unselected::Absent));
    let root = scratch.0.join("root");
    DirBuilder::new().mode(0o700).create(&root)?;
    assert_eq!(select(&root), Err(Unselected::Absent));
    manifest(&root, &selecting(GENERATION, EPOCH), 0o600)?;
    assert_eq!(select(&root), Ok((GENERATION.to_owned(), EPOCH.to_owned())));
    fs::set_permissions(
        root.join(ACTIVE_MANIFEST),
        fs::Permissions::from_mode(0o644),
    )?;
    assert_eq!(select(&root), Err(Unselected::Custody));
    fs::set_permissions(
        root.join(ACTIVE_MANIFEST),
        fs::Permissions::from_mode(0o600),
    )?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755))?;
    assert_eq!(select(&root), Err(Unselected::Custody));
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    for (case, body) in [
        (
            "unknown member",
            json!({"schema": ACTIVE_SCHEMA, "generation": GENERATION, "epoch": EPOCH, "x": 1}),
        ),
        (
            "another schema",
            json!({"schema": "hee3.active-generation/2", "generation": GENERATION, "epoch": EPOCH}),
        ),
        ("not a uuid", selecting("../other", EPOCH)),
    ] {
        fs::remove_file(root.join(ACTIVE_MANIFEST))?;
        manifest(&root, &body, 0o600)?;
        assert_eq!(select(&root), Err(Unselected::Malformed), "{case}");
    }
    // The manifest's strings are read as they are spelled: an escape that decodes to canonical
    // text is still not canonical text, in any member.
    let escaped = GENERATION.replacen('-', "\\u002d", 1);
    for (case, text) in [
        (
            "escaped generation",
            format!(r#"{{"schema":"{ACTIVE_SCHEMA}","generation":"{escaped}","epoch":"{EPOCH}"}}"#),
        ),
        (
            "escaped schema",
            format!(
                r#"{{"schema":"{}","generation":"{GENERATION}","epoch":"{EPOCH}"}}"#,
                ACTIVE_SCHEMA.replacen('/', r"\/", 1)
            ),
        ),
    ] {
        fs::remove_file(root.join(ACTIVE_MANIFEST))?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join(ACTIVE_MANIFEST))?
            .write_all(text.as_bytes())?;
        assert!(text.contains('\\'), "{case}: {text} carries an escape");
        let decoded: serde_json::Value = serde_json::from_str(&text)?;
        assert_eq!(
            decoded,
            selecting(GENERATION, EPOCH),
            "{case}: the escape decodes to the canonical record"
        );
        assert_eq!(select(&root), Err(Unselected::Malformed), "{case}");
    }
    // A manifest reached through a link is not the operator's manifest.
    let elsewhere = scratch.0.join("elsewhere");
    DirBuilder::new().mode(0o700).create(&elsewhere)?;
    manifest(&elsewhere, &selecting(GENERATION, EPOCH), 0o600)?;
    fs::remove_file(root.join(ACTIVE_MANIFEST))?;
    symlink(elsewhere.join(ACTIVE_MANIFEST), root.join(ACTIVE_MANIFEST))?;
    assert_eq!(select(&root), Err(Unselected::Custody));
    Ok(())
}

#[test]
fn health_is_blocked_until_a_generation_is_commissioned_and_ready_after() -> Outcome {
    let scratch = Scratch::new()?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let (health, why) = observe(&scratch.0.join("absent"), deadline);
    assert_eq!(
        (
            health.recovery,
            health.database,
            health.socket,
            health.ready()
        ),
        (
            Recovery::Blocked,
            Database::Unavailable,
            Socket::Owned,
            false
        )
    );
    assert!(why.contains("Absent"), "{why}");
    let root = commissioned(&scratch)?;
    let (health, line) = observe(&root, deadline);
    assert_eq!(
        (health.recovery, health.database, health.ready()),
        (Recovery::Complete, Database::Ready, true),
        "{line}"
    );
    assert_eq!(
        health.checked_unix_ms, CHECKED,
        "the observation instant passes through"
    );
    // B03c: the startup line names the cleanup tail, so a backlog is never silent at the one line
    // an operator sees at start. Here every count is its identity element; the two-fixture case
    // below pins each field off the origin (F129).
    assert_eq!(
        line,
        format!(
            "generation {GENERATION} reconciled: attempts=0 writes=0 cleanup=0 cleanup_backlog=0 \
             recovery=complete database=ready"
        )
    );
    // The selected epoch must be the ledger's own: a manifest naming another refuses startup.
    fs::remove_file(root.join(ACTIVE_MANIFEST))?;
    manifest(
        &root,
        &selecting(GENERATION, "28c00000-0000-4000-8000-0000000000ee"),
        0o600,
    )?;
    let (health, why) = observe(&root, deadline);
    assert_eq!(
        (health.recovery, health.database),
        (Recovery::Blocked, Database::Unavailable),
        "{why}"
    );
    Ok(())
}

#[test]
fn each_reconciliation_is_classified_and_the_pass_reports_what_it_left() {
    let unknown = Reconciliation::RetainUnknown {
        reason: Unknown::HistoryContradictory,
        process: ProcessCustody::Absent,
        cancellation_pending: false,
        workspace: None,
    };
    let settled = [
        Reconciliation::StaleObservationRefused {
            observed_epoch: "a".into(),
            ledger_epoch: "b".into(),
        },
        Reconciliation::RefuseStaleCursor {
            reason: CursorRefusal::EpochChanged,
            cursor_epoch: "a".into(),
            cursor_sequence: 3,
            ledger_epoch: "b".into(),
            event_high_water: 4,
        },
        Reconciliation::CursorSnapshotOnly {
            epoch: "a".into(),
            sequence: 3,
            event_high_water: 4,
            mode: Mode::Normal,
            replay: false,
        },
    ];
    assert!(leaves_work_outstanding(&unknown));
    for reconciliation in &settled {
        assert!(
            !leaves_work_outstanding(reconciliation),
            "{}",
            reconciliation.name()
        );
    }
    let pass = |mode, ledger, reconciliation: Option<Reconciliation>| Pass {
        epoch: EPOCH.into(),
        generation: GENERATION.into(),
        mode,
        event_high_water: 4,
        ledger,
        attempts: Vec::new(),
        cursors: reconciliation
            .into_iter()
            .map(|reconciliation| CursorEntry {
                cursor: Cursor {
                    epoch: EPOCH.into(),
                    sequence: 3,
                },
                decision: Decision {
                    rule: Rule::R01StaleObservationEpoch,
                    reconciliation,
                },
                rule: "R01",
            })
            .collect(),
        writes: 0,
        permits_execution: false,
        cleanup: Vec::new(),
        cleanup_backlog: 0,
    };
    let health = |p: &Pass| {
        let h = health_of(Ok(p), CHECKED);
        (h.recovery, h.database, h.ready())
    };
    assert_eq!(
        health(&pass(Mode::Normal, LedgerAccess::Writable, None)),
        (Recovery::Complete, Database::Ready, true)
    );
    assert_eq!(
        health(&pass(Mode::Normal, LedgerAccess::Writable, Some(unknown))),
        (Recovery::Pending, Database::Ready, false)
    );
    assert_eq!(
        health(&pass(
            Mode::Normal,
            LedgerAccess::Writable,
            Some(settled[1].clone())
        )),
        (Recovery::Complete, Database::Ready, true)
    );
    assert_eq!(
        health(&pass(
            Mode::Reconciliation,
            LedgerAccess::InspectionOnly,
            None
        )),
        (Recovery::Blocked, Database::Degraded, false)
    );
    assert_eq!(
        health_of(Err("refused"), CHECKED).body(),
        json!({"protocol_version": 1, "engine_version": env!("CARGO_PKG_VERSION"), "ready": false,
               "recovery": "blocked", "database": "unavailable", "socket": "owned",
               "checked_unix_ms": "1790000123456"})
    );
}

/// B03c: the startup line, whole, over two fixtures that differ in every field -- a renderer
/// that ignored any field, swapped two, or hard-coded one example fails one of them (F124).
#[test]
fn the_startup_line_names_every_count_and_the_health_it_left() {
    let health = |recovery, database| Health {
        recovery,
        database,
        socket: Socket::Owned,
        checked_unix_ms: CHECKED,
    };
    assert_eq!(
        startup_line(
            GENERATION,
            Counts {
                attempts: 3,
                writes: 7,
                cleanup: 32,
                cleanup_backlog: 8,
            },
            &health(Recovery::Pending, Database::Ready),
        ),
        "generation 28c00000-0000-4000-8000-000000000001 reconciled: attempts=3 writes=7 cleanup=32 \
         cleanup_backlog=8 recovery=pending database=ready"
    );
    assert_eq!(
        startup_line(
            OTHER_GENERATION,
            Counts {
                attempts: 11,
                writes: 2,
                cleanup: 5,
                cleanup_backlog: 1_025,
            },
            &health(Recovery::Blocked, Database::Degraded),
        ),
        "generation 28c00000-0000-4000-8000-000000000003 reconciled: attempts=11 writes=2 cleanup=5 \
         cleanup_backlog=1025 recovery=blocked database=degraded"
    );
}

const OTHER_GENERATION: &str = "28c00000-0000-4000-8000-000000000003";
const OTHER_EPOCH: &str = "28c00000-0000-4000-8000-000000000004";

/// Create a second, empty ledger generation beside the commissioned one.
fn ledger_at(root: &Path, generation: &str, epoch: &str) -> Outcome {
    drop(
        Store::open(
            root,
            UuidV4::parse(generation)?,
            UuidV4::parse(epoch)?,
            true,
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?,
    );
    Ok(())
}

/// How many tasks one generation's ledger holds, read through the store's inspection door.
fn tasks_in(root: &Path, generation: &str, epoch: &str) -> Result<usize, Box<dyn Error>> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut store = Store::open_inspection(
        root,
        UuidV4::parse(generation)?,
        UuidV4::parse(epoch)?,
        deadline,
    )
    .map_err(|error| format!("{error:?}"))?;
    let inventory = store
        .recovery_inventory(UuidV4::parse(epoch)?, coordinator::START_LIMITS, deadline)
        .map_err(|error| format!("{error:?}"))?;
    Ok(inventory.tasks.len())
}

fn replace_manifest(root: &Path, generation: &str, epoch: &str) -> Outcome {
    fs::remove_file(root.join(ACTIVE_MANIFEST))?;
    manifest(root, &selecting(generation, epoch), 0o600)
}

#[test]
fn a_manifest_swapped_after_reconciliation_cannot_change_the_generation_served() -> Outcome {
    let scratch = Scratch::new()?;
    let root = commissioned(&scratch)?;
    ledger_at(&root, OTHER_GENERATION, OTHER_EPOCH)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let manifest = coordinator::read_manifest(&root);
    let started = coordinator::observe_at_start(&root, &manifest, CHECKED, deadline);
    assert!(started.health.ready(), "{}", started.line);
    // After reconciliation the manifest names another commissioned generation.
    replace_manifest(&root, OTHER_GENERATION, OTHER_EPOCH)?;
    let reconciled = started.reconciled.ok_or("nothing was reconciled")?;
    assert_eq!(
        (
            reconciled.active.generation.as_str(),
            reconciled.active.epoch.as_str(),
            reconciled.pass.generation.as_str()
        ),
        (GENERATION, EPOCH, GENERATION)
    );
    let tasks = coordinator::compose_tasks(reconciled)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let submit = super::tasks::request(
        "task.submit",
        1,
        Some(super::tasks::KEY),
        &json!({"spec": super::tasks::spec()}),
    );
    let reply = super::tasks::serve(&tasks, &operator, &submit)?;
    assert_eq!(
        reply["body"]["engine_cursor"]["epoch"],
        json!(EPOCH),
        "{reply}"
    );
    drop(tasks);
    assert_eq!(
        (
            tasks_in(&root, GENERATION, EPOCH)?,
            tasks_in(&root, OTHER_GENERATION, OTHER_EPOCH)?
        ),
        (1, 0),
        "the admission landed in the reconciled generation and only there"
    );
    Ok(())
}

#[test]
fn the_writer_lock_is_held_from_reconciliation_to_the_task_owner() -> Outcome {
    let scratch = Scratch::new()?;
    let root = commissioned(&scratch)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let manifest = coordinator::read_manifest(&root);
    let started = coordinator::observe_at_start(&root, &manifest, CHECKED, deadline);
    assert!(started.reconciled.is_some(), "{}", started.line);
    // Between reconciliation and composing the task owner, no one else can open the ledger writable.
    let second = Store::open(
        &root,
        UuidV4::parse(GENERATION)?,
        UuidV4::parse(EPOCH)?,
        false,
        deadline,
    );
    assert!(
        matches!(second, Err(StoreError::Locked)),
        "{:?}",
        second.map(drop)
    );
    drop(started);
    Ok(())
}

// ---- OPS-1: `commission`, the operator's act that creates the state root -------------------------

/// The migration chain's length from the repository's own `migrations/*.sql` files: the independent
/// source a commissioned ledger's `user_version` is compared against (every file is a T04 subject,
/// so the gate stages them).
pub(super) fn migration_count() -> Result<u32, Box<dyn Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let mut count = 0_u32;
    for entry in fs::read_dir(&directory)? {
        if entry?
            .path()
            .extension()
            .is_some_and(|extension| extension == "sql")
        {
            count += 1;
        }
    }
    Ok(count)
}

/// `PRAGMA user_version` of the ledger at `ledger`, read through a raw read-only connection: not the
/// store's reading of it.
pub(super) fn raw_user_version(ledger: &Path) -> Result<u32, Box<dyn Error>> {
    let connection = rusqlite::Connection::open_with_flags(
        ledger,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    Ok(connection.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// The permission bits at `path`, read without following a link.
fn mode_of(path: &Path) -> Result<u32, Box<dyn Error>> {
    use std::os::unix::fs::MetadataExt;
    Ok(fs::symlink_metadata(path)?.mode() & 0o777)
}

/// The names in `directory` that are commissioning stages (`.<name>.<id>.staged`).
fn staged_in(directory: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut staged = Vec::new();
    for entry in fs::read_dir(directory)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if name.ends_with(".staged") {
            staged.push(name);
        }
    }
    Ok(staged)
}

/// Two fixtures that differ in every id (F129: never only the identity element).
const COMMISSION_FIXTURES: [(&str, &str); 2] = [
    (
        "28c00000-0000-4000-8000-0000000000c1",
        "28c00000-0000-4000-8000-0000000000c2",
    ),
    (
        "7a1b2c3d-4e5f-4a6b-9c8d-7e6f5a4b3c2d",
        "0f9e8d7c-6b5a-4f3e-a2d1-c0b9a8f7e6d5",
    ),
];

/// OPS-1 case 1 · commissioning creates a private 0700 root, a private 0600 manifest selecting the
/// generation, and a ledger at the migration chain's length; what it returns is read back from the
/// placed root, compared as a whole value and a whole line over two fixtures that differ in every
/// id. The known answers come from outside the code under test: the modes from `symlink_metadata`,
/// the manifest's bytes from a literal, `user_version` from a raw `PRAGMA` and from the count of
/// `migrations/*.sql`. The root's missing ancestors are created on the way; no stage is left.
#[test]
fn commissioning_creates_a_private_root_a_private_manifest_and_a_current_ledger() -> Outcome {
    let scratch = Scratch::new()?;
    let migrations = migration_count()?;
    for (index, (generation, epoch)) in COMMISSION_FIXTURES.into_iter().enumerate() {
        let parent = scratch.0.join(format!("home-{index}/.local/state"));
        let root = parent.join("herdr-engineering-engine-v3");
        let commissioned = commission(
            &root,
            Active {
                generation: UuidV4::parse(generation)?,
                epoch: UuidV4::parse(epoch)?,
            },
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?;
        assert_eq!(
            commissioned,
            Commissioned {
                root: root.clone(),
                generation: generation.to_owned(),
                epoch: epoch.to_owned(),
                user_version: migrations,
                root_mode: 0o700,
                manifest_mode: 0o600,
            }
        );
        assert_eq!(
            commissioned.line(),
            format!(
                "commissioned {} generation={generation} epoch={epoch} user_version={migrations} \
                 root_mode=0700 manifest_mode=0600",
                root.display()
            )
        );
        assert!(fs::symlink_metadata(&root)?.is_dir());
        assert_eq!(mode_of(&root)?, 0o700);
        let manifest_path = root.join(ACTIVE_MANIFEST);
        assert!(fs::symlink_metadata(&manifest_path)?.is_file());
        assert_eq!(mode_of(&manifest_path)?, 0o600);
        assert_eq!(
            fs::read_to_string(&manifest_path)?,
            format!(
                r#"{{"schema":"hee3.active-generation/1","generation":"{generation}","epoch":"{epoch}"}}"#
            )
        );
        let ledger = root
            .join("generations")
            .join(generation)
            .join("ledger.sqlite3");
        assert_eq!(raw_user_version(&ledger)?, migrations);
        assert_eq!(staged_in(&parent)?, Vec::<String>::new());
    }
    Ok(())
}

/// OPS-1 case 2 · anything standing at the root is refused by its path and left exactly as it was:
/// an empty 0700 directory (an operator's `mkdir`, R1.4), a regular file, and a link to an empty
/// 0700 directory (the link and its target both unchanged). No stage is left beside it.
#[test]
fn commissioning_refuses_an_existing_root_by_name_and_changes_nothing() -> Outcome {
    let scratch = Scratch::new()?;
    let target = scratch.0.join("target");
    DirBuilder::new().mode(0o700).create(&target)?;
    for shape in ["directory", "file", "link"] {
        let parent = scratch.0.join(shape);
        DirBuilder::new().mode(0o700).create(&parent)?;
        let root = parent.join("herdr-engineering-engine-v3");
        match shape {
            "directory" => DirBuilder::new().mode(0o700).create(&root)?,
            "file" => fs::write(&root, b"an operator's file")?,
            _ => symlink(&target, &root)?,
        }
        let before = (
            fs::symlink_metadata(&root)?.file_type(),
            fs::read_link(&root).ok(),
            fs::read(&root).ok(),
            fs::read_dir(&parent)?.count(),
            fs::read_dir(&target)?.count(),
        );
        let refused = commission(
            &root,
            Active {
                generation: UuidV4::parse(GENERATION)?,
                epoch: UuidV4::parse(EPOCH)?,
            },
            Instant::now() + Duration::from_secs(10),
        );
        assert_eq!(
            refused,
            Err(CommissionError::Exists(root.clone())),
            "{shape}"
        );
        let after = (
            fs::symlink_metadata(&root)?.file_type(),
            fs::read_link(&root).ok(),
            fs::read(&root).ok(),
            fs::read_dir(&parent)?.count(),
            fs::read_dir(&target)?.count(),
        );
        assert_eq!(before, after, "{shape}");
        assert_eq!(staged_in(&parent)?, Vec::<String>::new(), "{shape}");
    }
    assert_eq!(
        fs::read_dir(&target)?.count(),
        0,
        "the link's target was never written"
    );
    Ok(())
}

/// OPS-1 case 3 · the producer/consumer seam at the library: what `commission` placed is what
/// startup reconciles, read as `serve` reads it, and the start line is compared whole.
#[test]
fn a_commissioned_root_is_what_startup_reconciles() -> Outcome {
    let scratch = Scratch::new()?;
    let (generation, epoch) = COMMISSION_FIXTURES[1];
    let root = scratch.0.join("herdr-engineering-engine-v3");
    let deadline = Instant::now() + Duration::from_secs(10);
    commission(
        &root,
        Active {
            generation: UuidV4::parse(generation)?,
            epoch: UuidV4::parse(epoch)?,
        },
        deadline,
    )
    .map_err(|error| format!("{error:?}"))?;
    let manifest = coordinator::read_manifest(&root);
    let started = coordinator::observe_at_start(&root, &manifest, CHECKED, deadline);
    assert_eq!(
        started.line,
        format!(
            "generation {generation} reconciled: attempts=0 writes=0 cleanup=0 cleanup_backlog=0 \
             recovery=complete database=ready"
        )
    );
    Ok(())
}

/// OPS-1 case 4 · a deadline already spent commissions nothing: the stage is made, the store's
/// create door refuses the spent deadline, and the refusal is `Deadline` by its own name (not a
/// store refusal); no root stands and the stage is removed — the only case that reaches the
/// cleanup, since every other refusal comes before the stage.
#[test]
fn a_spent_deadline_commissions_nothing() -> Outcome {
    let scratch = Scratch::new()?;
    let parent = scratch.0.join("home/.local/state");
    let root = parent.join("herdr-engineering-engine-v3");
    let refused = commission(
        &root,
        Active {
            generation: UuidV4::parse(GENERATION)?,
            epoch: UuidV4::parse(EPOCH)?,
        },
        Instant::now(),
    );
    assert_eq!(refused, Err(CommissionError::Deadline));
    assert!(fs::symlink_metadata(&root).is_err(), "no root stands");
    assert_eq!(
        fs::read_dir(&parent)?.count(),
        0,
        "the stage was removed and nothing else was made beside the root"
    );
    Ok(())
}
