//! Actual same-namespace PID reuse held across two real inspector calls and
//! reported by the engine as `pid_reused`; the original child alive during
//! inspection reported `live_same_identity`; the reaped original reported
//! `absent`; with the namespace guard and the reuse-oracle sensitivity as
//! distinct controls. Every arm leaves resumption false.
use habitat_engine::worker::process::{self, ProcessSpec};
use serde_json::{Value, json};
use std::collections::BTreeMap;
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
            "pidreuse-{}-{}-{mode}",
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
/// Run one control through the bounded Python owner and return its retained
/// observation record. The Rust test asserts the literal outcome itself.
fn case(mode: &str) -> Value {
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
                .join("tests/fixtures/recovery_pid_reuse.py")
                .into_os_string(),
            env!("CARGO_BIN_EXE_hee3-recovery-pid-reuse-fixture").into(),
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
    assert_eq!(result["accepted_control"], true, "{result}");
    assert_eq!(result["input_drift"], json!([]));
    assert_eq!(result["unshare"], mode != "guard");
    assert_eq!(result["descendants_detected"], false);
    let observation: Value =
        serde_json::from_slice(&fs::read(evidence.join("observation.json")).unwrap()).unwrap();
    assert_eq!(observation["mode"], mode);
    assert_eq!(observation["unreaped"], json!([]));
    assert_eq!(observation["late_reaped"], json!([]));
    assert_eq!(observation["diagnostics"], json!([]));
    observation
}
#[test]
fn same_namespace_pid_reuse_is_held_across_two_inspections_without_resume() {
    let o = case("actual");
    assert_eq!(o["verdict"], "actual-reuse", "{o}");
    assert_eq!(o["actual_reuse"], true);
    assert_eq!(o["counter_written"], true);
    assert_eq!(o["counter_reset"], true);
    let (old, new) = (&o["old"], &o["new"]);
    assert_eq!(old["pid"], 2, "old worker at the private namespace PID2");
    assert_eq!(new["pid"], old["pid"], "replacement received the same PID");
    assert_eq!(new["namespace"], old["namespace"], "same PID namespace");
    assert!(
        new["start_ticks"].as_u64().unwrap() > old["start_ticks"].as_u64().unwrap(),
        "replacement started later: {old} {new}"
    );
    assert_eq!(o["self"]["namespace"], old["namespace"]);
    assert_ne!(o["self"]["namespace"], o["parent"]["pid"]);
    assert_eq!(o["old_reaped"], json!({"pid": 2, "exit": 0}));
    assert_eq!(o["replacement_reaped"], json!({"pid": 2, "exit": 0}));
    assert_eq!(o["identity_before_inspection"], *new);
    assert_eq!(o["identity_after_inspection"], *new);
    assert_eq!(o["replacement_release"]["trailing_bytes"], 0);
    let bound: Value = serde_json::from_str(
        o["binding"]["observation"]["input"]["actual_identity"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        bound, *old,
        "old identity bound into the roster observation"
    );
    assert_eq!(o["binding"]["attempts"].as_array().unwrap().len(), 1);
    assert_eq!(o["binding"]["attempts"][0][3], "running");
    let entries = inspections(&o, "pid_reused", "process_identity_reused", "R07");
    for entry in &entries {
        assert_eq!(
            entry["observed"], *old,
            "the ledger's identity is the old one"
        );
        assert_eq!(entry["differs"], json!(["start_ticks"]));
        let live = live_without_state(entry);
        assert_eq!(live, *new, "the live read is the replacement: {entry}");
    }
}
/// Both inspections of one record: resumption false, the engine's custody arm
/// as given, exactly one classified attempt entry naming the fixture attempt,
/// the engine policy's `retain_unknown` decision by `reason` and `rule` with the
/// custody carried as its evidence, an unresolved running attempt and nothing
/// dispatched or settled.
fn inspections(o: &Value, custody: &str, reason: &str, rule: &str) -> Vec<Value> {
    let inspections = o["inspections"].as_array().unwrap();
    assert_eq!(inspections.len(), 2);
    let mut entries = Vec::new();
    for r in inspections {
        assert_eq!(r["kind"], "hee3-development-recovery-inspection/1");
        assert_eq!(r["execution_resumed"], false);
        assert_eq!(r["recovery_complete"], false);
        assert_eq!(r["replay_authorized"], false);
        assert_eq!(r["policy_permits_execution"], false);
        assert_eq!(r["policy_permits_execution"], r["execution_resumed"]);
        assert_eq!(r["reconciliation"]["kind"], "hee3-recovery-policy/1");
        assert_eq!(r["reconciliation"]["mode_parsed"], true);
        assert_eq!(r["reconciliation"]["cursor"], Value::Null);
        let decisions = r["reconciliation"]["attempts"].as_array().unwrap();
        assert_eq!(decisions.len(), 1, "{decisions:?}");
        let d = &decisions[0];
        assert_eq!(d["attempt"], "07000000-0000-4000-8000-000000000006");
        assert_eq!(d["rule"], rule, "{d}");
        assert_eq!(d["name"], "retain_unknown", "{d}");
        assert_eq!(d["decision"]["decision"], "retain_unknown");
        assert_eq!(d["decision"]["reason"]["reason"], reason, "{d}");
        assert_eq!(d["decision"]["process"]["custody"], custody, "{d}");
        assert_eq!(d["decision"]["cancellation_pending"], false);
        if custody == "pid_reused" {
            assert_eq!(d["decision"]["reason"]["differs"], json!(["start_ticks"]));
            assert_eq!(d["decision"]["process"]["differs"], json!(["start_ticks"]));
        }
        assert_eq!(r["physical_process_custody"], custody, "{r}");
        assert_eq!(r["pi_queue_custody"], "unreconciled");
        let list = r["physical_process_identities"].as_array().unwrap();
        assert_eq!(list.len(), 1, "{list:?}");
        let entry = &list[0];
        assert_eq!(entry["attempt"], "07000000-0000-4000-8000-000000000006");
        assert_eq!(
            entry["record_id"],
            o["binding"]["observation"]["input"]["record_id"]
        );
        assert_eq!(entry["custody"], custody);
        assert_eq!(entry["error"], Value::Null);
        assert_eq!(entry["observed"], o["old"]);
        assert_eq!(r["inventory"]["attempts"].as_array().unwrap().len(), 1);
        assert_eq!(r["inventory"]["attempts"][0]["state"], "running");
        assert_eq!(r["inventory"]["acceptances"], json!([]));
        assert_eq!(r["inventory"]["verifications"], json!([]));
        assert_eq!(r["inventory"]["stops"], json!([]));
        assert_eq!(r["inventory"]["pending_delivery"], json!([]));
        entries.push(entry.clone());
    }
    assert_eq!(o["snapshot_before"], o["snapshot_after"]);
    assert_eq!(
        o["snapshot_before"]["attempts"].as_array().unwrap().len(),
        1
    );
    assert_eq!(o["expected_custody"], custody);
    entries
}
/// The entry's live identity as `{pid,start_ticks,namespace}`, with the stat
/// state retained by the report checked to be a live one.
fn live_without_state(entry: &Value) -> Value {
    let mut live: BTreeMap<String, Value> = serde_json::from_value(entry["live"].clone()).unwrap();
    let state = live.remove("state").unwrap();
    assert!(
        matches!(state.as_str(), Some("S" | "R" | "D")),
        "live state {state}"
    );
    serde_json::to_value(live).unwrap()
}
#[test]
fn original_child_alive_during_inspection_is_reported_live_same_identity() {
    let o = case("alive");
    assert_eq!(o["verdict"], "live-same-identity", "{o}");
    assert_eq!(o["actual_reuse"], false);
    assert_eq!(o["counter_written"], true);
    assert_eq!(o["counter_reset"], false, "no replacement is started");
    assert!(o.get("new").is_none() && o.get("replacement_reaped").is_none());
    let old = &o["old"];
    assert_eq!(old["pid"], 2);
    assert_eq!(o["identity_before_inspection"], *old);
    assert_eq!(o["identity_after_inspection"], *old);
    assert_eq!(o["old_reaped"], json!({"pid": 2, "exit": 0}));
    assert_eq!(o["old_release"]["trailing_bytes"], 0);
    assert_eq!(o["old_release"]["identity"], *old);
    let bound: Value = serde_json::from_str(
        o["binding"]["observation"]["input"]["actual_identity"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(bound, *old);
    for entry in inspections(&o, "live_same_identity", "pi_queue_unreconciled", "R06") {
        assert_eq!(entry["differs"], json!([]));
        assert_eq!(live_without_state(&entry), *old, "{entry}");
    }
}
#[test]
fn reaped_original_child_is_reported_absent() {
    let o = case("absent");
    assert_eq!(o["verdict"], "absent-after-reap", "{o}");
    assert_eq!(o["actual_reuse"], false);
    assert_eq!(o["counter_reset"], false);
    assert!(o.get("new").is_none());
    assert_eq!(o["old"]["pid"], 2);
    assert_eq!(o["old_reaped"], json!({"pid": 2, "exit": 0}));
    assert_eq!(
        o["identity_before_inspection"],
        json!({"pid": 2, "present": false})
    );
    assert_eq!(
        o["identity_after_inspection"],
        json!({"pid": 2, "present": false})
    );
    for entry in inspections(&o, "absent", "acknowledgement_unrecorded", "R08") {
        assert_eq!(entry["live"], Value::Null);
        assert_eq!(entry["differs"], json!([]));
    }
}
#[test]
fn namespace_guard_refuses_before_counter_access_outside_private_namespace() {
    let o = case("guard");
    assert_eq!(o["verdict"], "refused", "{o}");
    assert_eq!(o["refusal"], "private user/pid/mount namespace guard");
    assert_eq!(o["counter_opened"], false, "counter never opened");
    assert_eq!(o["counter_written"], false);
    assert_eq!(o["children_started"], 0, "no child before the guard");
    assert_ne!(o["self"]["pid"], 1);
    assert_eq!(o["self"]["namespace"], o["parent"]["pid"]);
    assert!(o.get("old").is_none() && o.get("new").is_none());
}
#[test]
fn differing_replacement_identity_is_refused_as_no_actual_reuse() {
    let o = case("mismatch");
    assert_eq!(o["verdict"], "refused", "{o}");
    assert_eq!(o["refusal"], "no actual reuse: pid");
    assert_eq!(
        o["actual_reuse"], false,
        "a fake mismatch is not actual reuse"
    );
    assert_eq!(o["counter_reset"], false);
    assert_eq!(o["observed_mismatch"], json!(["pid"]));
    assert_eq!(o["old"]["pid"], 2);
    assert_ne!(o["new"]["pid"], o["old"]["pid"]);
    assert_eq!(o["new"]["namespace"], o["old"]["namespace"]);
    assert_eq!(o["inspections"], json!([]), "no inspector on a refused run");
    assert_eq!(o["old_reaped"]["exit"], 0);
    assert_eq!(o["replacement_reaped"]["exit"], 0);
    assert_eq!(
        o["sensitivity"],
        json!({"pid": ["pid"], "namespace": ["namespace"], "start": ["start"]})
    );
}
