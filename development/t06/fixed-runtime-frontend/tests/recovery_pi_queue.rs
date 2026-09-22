//! Actual Pi Session/local pipe parent loss with live peer and read-only inspector.
use habitat_engine::worker::process::{self, ProcessSpec};
use serde_json::Value;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Area {
    path: PathBuf,
    inode: (u64, u64),
}
impl Area {
    fn new(mode: &str) -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "pi-{}-{}-{mode}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        let m = fs::metadata(&path).unwrap();
        Self {
            path,
            inode: (m.dev(), m.ino()),
        }
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        let m = fs::symlink_metadata(&self.path).unwrap();
        assert_eq!((m.dev(), m.ino()), self.inode);
        fs::remove_dir_all(&self.path).unwrap();
    }
}
fn case(mode: &str) {
    let a = Area::new(mode);
    let evidence =
        PathBuf::from(std::env::var_os("T07_CANCEL_EVIDENCE").expect("owned evidence root"))
            .join(a.path.file_name().unwrap());
    fs::create_dir_all(evidence.parent().unwrap()).unwrap();
    let package = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let spec = ProcessSpec {
        executable: "/usr/bin/python3".into(),
        arguments: vec![
            "-B".into(),
            package
                .join("tests/fixtures/recovery_pi_queue.py")
                .into_os_string(),
            env!("CARGO_BIN_EXE_hee3-recovery-pi-queue-fixture").into(),
            env!("CARGO_BIN_EXE_hee3-fixed-runtime-frontend").into(),
            mode.into(),
            a.path.clone().into_os_string(),
            evidence.clone().into_os_string(),
        ],
        directory: package,
        environment: vec![
            ("LC_ALL".into(), "C".into()),
            ("TMPDIR".into(), std::env::temp_dir().into_os_string()),
        ],
        input: vec![],
        stream_limit: 1_048_576,
    };
    let report = process::run(
        &spec,
        Instant::now() + Duration::from_secs(40),
        &AtomicBool::new(false),
    )
    .unwrap();
    fs::write(
        evidence.with_extension("process.txt"),
        format!("{spec:#?}\n{report:#?}"),
    )
    .unwrap();
    assert!(
        report.exit_code == Some(0)
            && report.signal.is_none()
            && report.interruption.is_none()
            && report.pending.is_none()
            && report.leader_reaped
            && report.process_group_settled
            && report.stdout.eof
            && report.stderr.eof
            && !report.stdout.failed
            && !report.stderr.failed
            && !report.stdout.truncated
            && !report.stderr.truncated
            && report.stderr.bytes.is_empty(),
        "{report:#?}"
    );
    let result: Value =
        serde_json::from_slice(&fs::read(evidence.join("result.json")).unwrap()).unwrap();
    assert_eq!(result["accepted_control"], true);
    assert_eq!(result["input_drift"], serde_json::json!([]));
    let observation: Value =
        serde_json::from_slice(&fs::read(evidence.join("observation.json")).unwrap()).unwrap();
    assert_eq!(observation["mode"], mode);
    assert_eq!(observation["peer_exit"], 0);
    // The inspector reports the engine policy's decision for the one running
    // attempt: its literal adapter identity is not a process identity, so the
    // policy retains it unknown as unobserved; the Pi queue stays unreconciled.
    let inspections = observation["inspections"].as_array().unwrap();
    assert_eq!(inspections.len(), 2);
    for r in inspections {
        assert_eq!(r["execution_resumed"], false);
        assert_eq!(r["policy_permits_execution"], false);
        assert_eq!(r["policy_permits_execution"], r["execution_resumed"]);
        assert_eq!(r["pi_queue_custody"], "unreconciled");
        assert_eq!(r["reconciliation"]["inputs"]["pi_queue"], "unreconciled");
        let decisions = r["reconciliation"]["attempts"].as_array().unwrap();
        assert_eq!(decisions.len(), 1, "{decisions:?}");
        let d = &decisions[0];
        assert_eq!(d["rule"], "R07", "{d}");
        assert_eq!(d["name"], "retain_unknown", "{d}");
        assert_eq!(
            d["decision"]["reason"]["reason"], "process_unobserved",
            "{d}"
        );
        assert_eq!(d["decision"]["process"]["custody"], "unobserved");
        assert_eq!(r["inventory"]["attempts"][0]["state"], "running");
    }
}
#[test]
fn steering_queue_survives_parent_loss_without_replay() {
    case("steering");
}
#[test]
fn follow_up_queue_survives_parent_loss_without_replay() {
    case("follow-up");
}
#[test]
fn clear_written_before_effect_remains_unknown_after_loss() {
    case("clear-before");
}
#[test]
fn clear_effected_with_lost_reply_remains_unknown_after_loss() {
    case("clear-after");
}
#[test]
fn owned_clear_abort_idle_has_correlated_benign_neighbor() {
    case("benign");
}
#[test]
fn changed_vendor_session_poisoned_without_stale_commands() {
    case("wrong-session");
}
#[test]
fn wrong_clear_id_cannot_authorize_abort() {
    case("wrong-id");
}
#[test]
fn old_generation_reply_cannot_authorize_abort() {
    case("late-generation");
}
#[test]
fn clear_reply_eof_retains_pending_identity_and_unknown_custody() {
    case("eof");
}
