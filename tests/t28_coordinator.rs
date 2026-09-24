//! T28 coordinator cases (a module of `t28_actions`): the active-generation manifest, startup
//! reconciliation composed into `serve`, and the `health` it serves (review D-C3 step 2).
//!
//! Every state root here is a scratch directory; nothing touches `$HOME/.local/state`.
use habitat_engine::app::coordinator::{
    self, ACTIVE_MANIFEST, ACTIVE_SCHEMA, Unselected, health_of, leaves_work_outstanding,
    startup_line,
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

/// A state root holding a real, empty ledger for the selected generation.
fn commissioned(scratch: &Scratch) -> Result<PathBuf, Box<dyn Error>> {
    let root = scratch.0.join("state");
    DirBuilder::new().mode(0o700).create(&root)?;
    drop(
        Store::open(
            &root,
            UuidV4::parse(GENERATION)?,
            UuidV4::parse(EPOCH)?,
            true,
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?,
    );
    manifest(&root, &selecting(GENERATION, EPOCH), 0o600)?;
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
