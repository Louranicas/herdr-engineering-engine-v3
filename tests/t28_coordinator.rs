//! T28 coordinator cases (a module of `t28_actions`): the active-generation manifest, startup
//! reconciliation composed into `serve`, and the `health` it serves (review D-C3 step 2).
//!
//! Every state root here is a scratch directory; nothing touches `$HOME/.local/state`.
use habitat_engine::app::coordinator::{
    self, ACTIVE_MANIFEST, ACTIVE_SCHEMA, Unselected, health_of, leaves_work_outstanding,
};
use habitat_engine::app::startup::{Cursor, CursorEntry, LedgerAccess, Pass};
use habitat_engine::contracts::UuidV4;
use habitat_engine::contracts::control::{Database, Recovery, Socket};
use habitat_engine::recovery::{
    CursorRefusal, Decision, Mode, ProcessCustody, Reconciliation, Rule, Unknown,
};
use habitat_engine::store::Store;
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

#[test]
fn a_generation_is_selected_only_from_the_operators_private_manifest() -> Outcome {
    let scratch = Scratch::new()?;
    assert_eq!(
        coordinator::active(&scratch.0.join("absent")),
        Err(Unselected::Absent)
    );
    let root = scratch.0.join("root");
    DirBuilder::new().mode(0o700).create(&root)?;
    assert_eq!(coordinator::active(&root), Err(Unselected::Absent));
    manifest(&root, &selecting(GENERATION, EPOCH), 0o600)?;
    let selected = coordinator::active(&root).map_err(|error| format!("{error:?}"))?;
    assert_eq!(
        (selected.generation.as_str(), selected.epoch.as_str()),
        (GENERATION, EPOCH)
    );
    fs::set_permissions(
        root.join(ACTIVE_MANIFEST),
        fs::Permissions::from_mode(0o644),
    )?;
    assert_eq!(coordinator::active(&root), Err(Unselected::Custody));
    fs::set_permissions(
        root.join(ACTIVE_MANIFEST),
        fs::Permissions::from_mode(0o600),
    )?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755))?;
    assert_eq!(coordinator::active(&root), Err(Unselected::Custody));
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
        assert_eq!(
            coordinator::active(&root),
            Err(Unselected::Malformed),
            "{case}"
        );
    }
    // A manifest reached through a link is not the operator's manifest.
    let elsewhere = scratch.0.join("elsewhere");
    DirBuilder::new().mode(0o700).create(&elsewhere)?;
    manifest(&elsewhere, &selecting(GENERATION, EPOCH), 0o600)?;
    fs::remove_file(root.join(ACTIVE_MANIFEST))?;
    symlink(elsewhere.join(ACTIVE_MANIFEST), root.join(ACTIVE_MANIFEST))?;
    assert_eq!(coordinator::active(&root), Err(Unselected::Custody));
    Ok(())
}

#[test]
fn health_is_blocked_until_a_generation_is_commissioned_and_ready_after() -> Outcome {
    let scratch = Scratch::new()?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let (health, why) = coordinator::observe_at_start(&scratch.0.join("absent"), CHECKED, deadline);
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
    let (health, line) = coordinator::observe_at_start(&root, CHECKED, deadline);
    assert_eq!(
        (health.recovery, health.database, health.ready()),
        (Recovery::Complete, Database::Ready, true),
        "{line}"
    );
    assert_eq!(
        health.checked_unix_ms, CHECKED,
        "the observation instant passes through"
    );
    // The selected epoch must be the ledger's own: a manifest naming another refuses startup.
    fs::remove_file(root.join(ACTIVE_MANIFEST))?;
    manifest(
        &root,
        &selecting(GENERATION, "28c00000-0000-4000-8000-0000000000ee"),
        0o600,
    )?;
    let (health, why) = coordinator::observe_at_start(&root, CHECKED, deadline);
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
