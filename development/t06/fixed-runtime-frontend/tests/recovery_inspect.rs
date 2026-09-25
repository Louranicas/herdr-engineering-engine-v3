//! Actual executable readback with independently enumerated durable expectations.
use habitat_engine::contracts::{Generation, Sha256Digest, UuidV4};
use habitat_engine::store::{
    Allocation, Effect, Expected, Object, Principal, Settlement, Store, Submission, Verification,
    VerificationVerdict,
};
use habitat_engine::worker::process::{self, ProcessSpec};
use rusqlite::{Connection, OpenFlags, types::Value as SqlValue};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, Instant},
};
const GEN: &str = "07000000-0000-4000-8000-000000000001";
const EPOCH: &str = "07000000-0000-4000-8000-000000000002";
const TASK: &str = "07000000-0000-4000-8000-000000000003";
const KEY: &str = "07000000-0000-4000-8000-000000000004";
const ADMIT: &str = "07000000-0000-4000-8000-000000000005";
const ATTEMPT: &str = "07000000-0000-4000-8000-000000000006";
const START: &str = "07000000-0000-4000-8000-000000000007";
const SETTLE: &str = "07000000-0000-4000-8000-000000000008";
const CHECK: &str = "07000000-0000-4000-8000-000000000009";
const ACCEPT: &str = "07000000-0000-4000-8000-00000000000a";
const CANCEL: &str = "07000000-0000-4000-8000-00000000000b";
const STAGE: &str = "07000000-0000-4000-8000-00000000000d";
const OTHER: &str = "07000000-0000-4000-8000-00000000000e";
const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
static NEXT: AtomicU64 = AtomicU64::new(0);
fn id(v: &str) -> UuidV4<'_> {
    UuidV4::parse(v).unwrap()
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn principal() -> Principal {
    Principal::new(1000, "operator").unwrap()
}
struct Rig {
    path: PathBuf,
    inode: (u64, u64),
    store: Option<Store>,
    evidence: Object,
}
impl Rig {
    fn new() -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "inspect-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        let inode = fs::metadata(&path).unwrap();
        let inode = (inode.dev(), inode.ino());
        fs::DirBuilder::new()
            .mode(0o700)
            .create(path.join("store"))
            .unwrap();
        let store = Store::open(&path.join("store"), id(GEN), id(EPOCH), true, end()).unwrap();
        let evidence = store
            .publish(b"retained fixture evidence", id(STAGE), end())
            .unwrap();
        Self {
            path,
            inode,
            store: Some(store),
            evidence,
        }
    }
    fn store(&mut self) -> &mut Store {
        self.store.as_mut().unwrap()
    }
    fn admit(&mut self) {
        self.store()
            .submit(
                Submission {
                    principal: &principal(),
                    key: id(KEY),
                    task: id(TASK),
                    event: id(ADMIT),
                    request_bytes: b"independent restart fixture",
                    workspace_id: id("28f00000-0000-4000-8000-00000000000a"),
                    criteria: Sha256Digest::parse(DIGEST).unwrap(),
                    allocation: Allocation {
                        limit_ms: 1_200_000,
                        work_ms: 900_000,
                        verify_ms: 300_000,
                    },
                },
                end(),
            )
            .unwrap();
    }
    fn start(&mut self) {
        self.admit();
        self.store()
            .begin_attempt(
                id(TASK),
                "1".parse().unwrap(),
                id(ATTEMPT),
                id(START),
                end(),
            )
            .unwrap();
    }
    fn expected(&mut self) -> Expected<'static> {
        let g: Generation = self
            .store()
            .get(&principal(), id(TASK), end())
            .unwrap()
            .generation
            .parse()
            .unwrap();
        Expected {
            task: id(TASK),
            task_generation: g,
            attempt: id(ATTEMPT),
            attempt_generation: "1".parse().unwrap(),
        }
    }
    fn settle(&mut self, effect: Effect, used: Option<u64>, cleanup: bool) {
        let x = self.expected();
        self.store()
            .settle_attempt(
                &x,
                Settlement {
                    effect,
                    used_ms: used,
                    cleanup_settled: cleanup,
                    ready_to_verify: true,
                },
                id(SETTLE),
                end(),
            )
            .unwrap();
    }
    fn ready(&mut self) {
        self.start();
        self.settle(Effect::None, Some(47_977), true);
    }
    fn verify(&mut self, verdict: VerificationVerdict, used: Option<u64>, cleanup: bool) {
        let x = self.expected();
        let evidence = self.evidence.clone();
        self.store()
            .record_verification(
                &x,
                &Verification {
                    verdict,
                    subject: Sha256Digest::parse(DIGEST).unwrap(),
                    evidence,
                    used_ms: used,
                    cleanup_settled: cleanup,
                },
                id(CHECK),
                end(),
            )
            .unwrap();
    }
    fn accept(&mut self) {
        self.ready();
        self.verify(VerificationVerdict::Passed, Some(20), true);
        let x = self.expected();
        let e = self.evidence.clone();
        let p = self
            .store()
            .prepare_verified_acceptance(
                &x,
                id(ACCEPT),
                Sha256Digest::parse(DIGEST).unwrap(),
                &e,
                std::slice::from_ref(&e),
                end(),
            )
            .unwrap();
        self.store().accept(&p, 0, end()).unwrap();
    }
    fn cancel(&mut self) {
        let g = self
            .store()
            .get(&principal(), id(TASK), end())
            .unwrap()
            .generation
            .parse()
            .unwrap();
        self.store().cancel(id(TASK), g, id(CANCEL), end()).unwrap();
    }
    fn close(&mut self) {
        drop(self.store.take());
    }
    fn db(&self) -> PathBuf {
        self.path
            .join("store/generations")
            .join(GEN)
            .join("ledger.sqlite3")
    }
    fn ledger(&self) -> BTreeMap<String, Vec<Vec<SqlValue>>> {
        let db = Connection::open_with_flags(
            self.db(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        db.execute_batch("PRAGMA query_only=ON;").unwrap();
        let names: Vec<String> = db
            .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let mut out = BTreeMap::new();
        for name in names {
            let mut q = db
                .prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))
                .unwrap();
            let n = q.column_count();
            let mut rows: Vec<Vec<SqlValue>> = q
                .query_map([], |r| (0..n).map(|i| r.get(i)).collect())
                .unwrap()
                .map(Result::unwrap)
                .collect();
            rows.sort_by_key(|r| format!("{r:?}"));
            out.insert(name, rows);
        }
        db.close().unwrap();
        out
    }
    fn invoke(&self, generation: &str, epoch: &str, output: &Path) -> (i32, String) {
        self.invoke_cursor(generation, epoch, output, None)
    }
    fn invoke_cursor(
        &self,
        generation: &str,
        epoch: &str,
        output: &Path,
        cursor: Option<&str>,
    ) -> (i32, String) {
        let mut args = vec![
            "inspect-recovery".into(),
            self.path.join("store").into_os_string(),
            generation.into(),
            epoch.into(),
            output.as_os_str().to_owned(),
        ];
        if let Some(cursor) = cursor {
            args.push(cursor.into());
        }
        let spec = ProcessSpec {
            executable: PathBuf::from(env!("CARGO_BIN_EXE_hee3-fixed-runtime-frontend")),
            arguments: args,
            directory: self.path.clone(),
            environment: vec![("LC_ALL".into(), "C".into())],
            input: vec![],
            stream_limit: 65_536,
        };
        let r = process::run(&spec, end(), &AtomicBool::new(false)).unwrap();
        let label = output.file_name().unwrap().to_string_lossy();
        if let Ok(evidence) = std::env::var("T07_INSPECT_EVIDENCE") {
            let directory = PathBuf::from(evidence).join(format!(
                "{}-{label}",
                self.path.file_name().unwrap().to_string_lossy()
            ));
            fs::create_dir_all(&directory).unwrap();
            fs::write(
                directory.join("process.txt"),
                format!("argv={:?}\n{r:#?}", spec.arguments),
            )
            .unwrap();
            for name in ["report.json", "complete.json", "error.json"] {
                if output.join(name).is_file() {
                    fs::copy(output.join(name), directory.join(name)).unwrap();
                }
            }
        }
        assert!(r.pending.is_none(), "{r:?}");
        assert!(r.leader_reaped && r.process_group_settled, "{r:?}");
        assert_eq!(r.interruption, None);
        assert_eq!(r.signal, None);
        assert!(
            r.stdout.eof
                && r.stderr.eof
                && !r.stdout.failed
                && !r.stderr.failed
                && !r.stdout.truncated
                && !r.stderr.truncated
        );
        assert!(r.stdout.bytes.is_empty());
        (
            r.exit_code.unwrap(),
            String::from_utf8(r.stderr.bytes).unwrap(),
        )
    }
    fn inspect(&mut self) -> Value {
        self.close();
        let before = self.ledger();
        let output = self.path.join("report");
        let (exit, stderr) = self.invoke(GEN, EPOCH, &output);
        assert_eq!((exit, stderr), (0, String::new()));
        assert_eq!(before, self.ledger());
        let bytes = fs::read(output.join("report.json")).unwrap();
        let report: Value = serde_json::from_slice(&bytes).unwrap();
        let complete: Value =
            serde_json::from_slice(&fs::read(output.join("complete.json")).unwrap()).unwrap();
        assert_eq!(complete["report_bytes"], bytes.len());
        assert_eq!(complete["report_sha256"], expected_digest(&bytes));
        assert_eq!(report["recovery_complete"], false);
        assert_eq!(report["execution_resumed"], false);
        assert_eq!(report["physical_process_custody"], "unreconciled");
        assert_eq!(report["evidence_availability"], "unassessed");
        report["inventory"].clone()
    }
    fn refuse(&mut self, g: &str, e: &str) {
        self.close();
        let before = self.ledger();
        let out = self.path.join("refusal");
        let (status, error) = self.invoke(g, e, &out);
        assert_eq!(status, 1);
        assert!(!error.is_empty());
        assert!(!out.join("complete.json").exists());
        assert_eq!(before, self.ledger());
    }
}
impl Drop for Rig {
    fn drop(&mut self) {
        self.close();
        let m = fs::symlink_metadata(&self.path).unwrap();
        assert_eq!((m.dev(), m.ino()), self.inode);
        assert!(!m.file_type().is_symlink());
        fs::remove_dir_all(&self.path).unwrap();
    }
}
#[test]
fn empty_store() {
    let mut r = Rig::new();
    let v = r.inspect();
    assert_eq!(v["event_high_water"], "0");
    assert_eq!(v["tasks"], json!([]));
}
#[test]
fn accepted_completion_retained() {
    let mut r = Rig::new();
    r.accept();
    let v = r.inspect();
    assert_eq!(v["tasks"][0]["accepted_event"], ACCEPT);
    assert_eq!(v["tasks"][0]["spent_ms"], 47_997);
    assert_eq!(v["tasks"][0]["reserved_work_ms"], 0);
    assert_eq!(v["acceptances"][0]["event"], ACCEPT);
    assert_eq!(
        v["pending_delivery"],
        json!([{"event":ACCEPT,"recipient":"1000:operator","sequence":"5","task":TASK,"roster":null}])
    );
}
#[test]
fn accepted_history_after_ack() {
    let mut r = Rig::new();
    r.accept();
    r.store()
        .acknowledge_delivery(id(ACCEPT), "1000:operator", end())
        .unwrap();
    let v = r.inspect();
    assert_eq!(v["acceptances"][0]["event"], ACCEPT);
    assert_eq!(v["pending_delivery"], json!([]));
}
#[test]
fn verifying_without_verifier() {
    let mut r = Rig::new();
    r.ready();
    let v = r.inspect();
    assert_eq!(v["tasks"][0]["state"], "verifying");
    assert_eq!(v["tasks"][0]["reserved_verify_ms"], 300_000);
    assert_eq!(v["verifications"], json!([]));
    assert_eq!(v["acceptances"], json!([]));
}
#[test]
fn committed_cancel_still_pending_cleanup() {
    let mut r = Rig::new();
    r.start();
    r.cancel();
    let v = r.inspect();
    assert_eq!(v["tasks"][0]["cancellation"], true);
    assert_eq!(v["tasks"][0]["state"], "cancellation_requested");
    assert_eq!(v["attempts"][0]["cleanup"], "pending");
    assert_eq!(v["tasks"][0]["reserved_work_ms"], 900_000);
}
#[test]
fn cancel_after_acceptance_is_history() {
    let mut r = Rig::new();
    r.accept();
    r.cancel();
    let v = r.inspect();
    assert_eq!(v["tasks"][0]["state"], "accepted");
    assert_eq!(v["tasks"][0]["cancellation"], false);
    assert_eq!(v["event_high_water"], "5");
}
#[test]
fn unknown_usage_not_zero() {
    let mut r = Rig::new();
    r.start();
    r.settle(Effect::Unknown, None, false);
    let v = r.inspect();
    assert_eq!(v["attempts"][0]["effect"], "unknown");
    assert_eq!(v["attempts"][0]["used_ms"], Value::Null);
    assert_eq!(v["tasks"][0]["reserved_work_ms"], 900_000);
}
#[test]
fn pending_effect_not_settled() {
    let mut r = Rig::new();
    r.start();
    r.settle(Effect::Pending, Some(9), false);
    let v = r.inspect();
    assert_eq!(v["attempts"][0]["effect"], "pending");
    assert_eq!(v["tasks"][0]["state"], "effect_unknown");
}
#[test]
fn verifier_error_retains_unknown() {
    let mut r = Rig::new();
    r.ready();
    r.verify(VerificationVerdict::Error, None, false);
    let v = r.inspect();
    assert_eq!(v["verifications"][0]["verdict"], "error");
    assert_eq!(v["verifications"][0]["used_ms"], Value::Null);
    assert_eq!(v["verifications"][0]["cleanup_settled"], false);
    assert_eq!(v["tasks"][0]["reserved_verify_ms"], 300_000);
}
#[test]
fn admitted_is_not_dispatched() {
    let mut r = Rig::new();
    r.admit();
    let v = r.inspect();
    assert_eq!(v["tasks"][0]["state"], "admitted");
    assert_eq!(v["attempts"], json!([]));
    assert_eq!(v["tasks"][0]["reserved_work_ms"], 900_000);
}
#[test]
fn wrong_epoch() {
    let mut r = Rig::new();
    r.admit();
    r.refuse(GEN, OTHER);
    assert_eq!(r.inspect()["tasks"][0]["id"], TASK);
}
#[test]
fn wrong_generation() {
    let mut r = Rig::new();
    r.admit();
    r.refuse(OTHER, EPOCH);
    assert_eq!(r.inspect()["tasks"][0]["id"], TASK);
}
#[test]
fn held_store_lock() {
    let r = Rig::new();
    let out = r.path.join("locked");
    let (code, error) = r.invoke(GEN, EPOCH, &out);
    assert_eq!(code, 1);
    assert!(error.contains("Locked"));
    assert!(!out.join("complete.json").exists());
}
#[test]
fn existing_report_is_not_replaced() {
    let mut r = Rig::new();
    r.inspect();
    let out = r.path.join("report");
    let before = fs::read(out.join("complete.json")).unwrap();
    assert_eq!(r.invoke(GEN, EPOCH, &out).0, 1);
    assert_eq!(before, fs::read(out.join("complete.json")).unwrap());
}
#[test]
fn output_inside_store_refuses() {
    let mut r = Rig::new();
    r.close();
    let out = r.path.join("store/report");
    assert_eq!(r.invoke(GEN, EPOCH, &out).0, 1);
    assert!(!out.exists());
}
#[test]
fn malformed_epoch_refuses() {
    let mut r = Rig::new();
    r.refuse(GEN, "not-a-uuid");
}
#[test]
fn expired_inspection_has_no_effect() {
    let mut r = Rig::new();
    r.close();
    let before = r.ledger();
    let out = r.path.join("expired");
    assert!(
        hee3_fixed_runtime_frontend::recovery::inspect_until(
            &r.path.join("store"),
            GEN,
            EPOCH,
            &out,
            Instant::now()
        )
        .is_err()
    );
    assert!(!out.exists());
    assert_eq!(before, r.ledger());
}
#[test]
fn inventory_bound_never_successfully_truncates() {
    let mut r = Rig::new();
    r.close();
    let db = Connection::open(r.db()).unwrap();
    db.execute_batch("BEGIN IMMEDIATE;").unwrap();
    for n in 1..=1025 {
        let task = format!("07000000-0000-4000-8001-{n:012x}");
        db.execute(
            "INSERT INTO tasks VALUES(?,1000,'operator',x'00',?,'1','admitted',0,NULL,1,0,0,0)",
            rusqlite::params![task, DIGEST],
        )
        .unwrap();
    }
    db.execute_batch("COMMIT;").unwrap();
    db.close().unwrap();
    r.refuse(GEN, EPOCH);
    let error = fs::read_to_string(r.path.join("refusal/error.json")).unwrap();
    assert!(error.contains("Bound"));
}

#[test]
fn fresh_repeated_report_does_not_duplicate_effects() {
    let mut r = Rig::new();
    r.accept();
    let first = r.inspect();
    let before = r.ledger();
    let output = r.path.join("again");
    assert_eq!(r.invoke(GEN, EPOCH, &output), (0, String::new()));
    let second: Value =
        serde_json::from_slice(&fs::read(output.join("report.json")).unwrap()).unwrap();
    assert_eq!(first, second["inventory"]);
    assert_eq!(before, r.ledger());
}
#[test]
fn symlink_report_parent_refuses() {
    let mut r = Rig::new();
    r.close();
    let link = r.path.join("alias");
    std::os::unix::fs::symlink(&r.path, &link).unwrap();
    let output = link.join("report");
    assert_eq!(r.invoke(GEN, EPOCH, &output).0, 1);
    assert!(!r.path.join("report").exists());
}

fn expected_digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut text = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(text, "{byte:02x}").unwrap();
    }
    text
}

#[test]
fn non_utf8_root_is_typed_refusal_not_reporting_panic() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let mut r = Rig::new();
    r.close();
    let root = r.path.join(OsString::from_vec(b"store-\xff".to_vec()));
    fs::rename(r.path.join("store"), &root).unwrap();
    let output = r.path.join("bad-encoding");
    let observed = std::panic::catch_unwind(|| {
        hee3_fixed_runtime_frontend::recovery::inspect(&root, GEN, EPOCH, &output)
    });
    assert!(observed.is_ok(), "malformed path must not panic");
    assert!(observed.unwrap().is_err());
    assert!(!output.exists());
}

fn cursor_json(epoch: &str, sequence: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    json!({"epoch":epoch,"sequence":sequence,"filter_sha256":DIGEST,
        "visibility_revision":"0","issued_unix_ms":(now-1000).to_string(),
        "expires_unix_ms":(now+60_000).to_string()})
    .to_string()
}

fn copy_snapshot_tree(source: &Path, destination: &Path) {
    fs::DirBuilder::new()
        .mode(0o700)
        .create(destination)
        .unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let meta = entry.file_type().unwrap();
        assert!(!meta.is_symlink());
        if meta.is_dir() {
            copy_snapshot_tree(&entry.path(), &destination.join(entry.file_name()));
        } else {
            assert!(meta.is_file());
            fs::copy(entry.path(), destination.join(entry.file_name())).unwrap();
        }
    }
}

fn restored_fixture() -> Rig {
    // Disposable verified snapshot, then explicitly test-only epoch/mode transaction.
    let mut source = Rig::new();
    source.accept();
    let snapshot = source.path.join("snapshot");
    fs::DirBuilder::new().mode(0o700).create(&snapshot).unwrap();
    let backup = source.store().backup(&snapshot, end()).unwrap();
    assert_eq!(
        backup.operational_status,
        habitat_engine::store::RestoreStatus::Unqualified
    );
    let manifest = fs::read(snapshot.join("store-backup.json")).unwrap();
    let hash = expected_digest(&manifest);
    assert_eq!(
        Store::inspect_backup(&snapshot, Sha256Digest::parse(&hash).unwrap(), end()).unwrap(),
        backup
    );
    let database_before = fs::read(snapshot.join("ledger.sqlite3")).unwrap();
    assert_eq!(expected_digest(&database_before), backup.database_sha256);
    source.close();
    let mut target = Rig::new();
    target.close();
    let target_generation = target.path.join("store/generations").join(GEN);
    // Only this freshly owned throwaway generation is replaced, never source/backup.
    fs::remove_dir_all(&target_generation).unwrap();
    copy_snapshot_tree(&snapshot, &target_generation);
    assert_eq!(backup.cutoff, 5);
    assert_eq!(
        Store::inspect_backup(
            &target_generation,
            Sha256Digest::parse(&hash).unwrap(),
            end()
        )
        .unwrap(),
        backup
    );
    let mut db = Connection::open(target.db()).unwrap();
    let before: Vec<(String, i64)> = db
        .prepare("SELECT id,sequence FROM events ORDER BY sequence")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let tx = db.transaction().unwrap();
    assert_eq!(
        tx.execute(
            "UPDATE ledger_meta SET epoch=?,mode='reconciliation' WHERE singleton=1 AND epoch=?",
            [OTHER, EPOCH]
        )
        .unwrap(),
        1
    );
    tx.commit().unwrap();
    let after: Vec<(String, i64)> = db
        .prepare("SELECT id,sequence FROM events ORDER BY sequence")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(before, after);
    db.close().unwrap();
    assert_eq!(
        database_before,
        fs::read(snapshot.join("ledger.sqlite3")).unwrap()
    );
    if let Ok(evidence) = std::env::var("T07_INSPECT_EVIDENCE") {
        let directory = PathBuf::from(evidence).join(format!(
            "{}-snapshot-provenance",
            target.path.file_name().unwrap().to_string_lossy()
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("fixture.json"),serde_json::to_vec_pretty(&json!({
            "kind":"test-only-verified-snapshot-epoch-rotation","operational_restore":false,
            "backup":backup,"source_manifest_sha256":hash,"old_epoch":EPOCH,"new_epoch":OTHER,
            "sql":"UPDATE ledger_meta SET epoch=?,mode='reconciliation' WHERE singleton=1 AND epoch=?",
            "historical_events_before":before,"historical_events_after":after,
            "database_after_sha256":expected_digest(&fs::read(target.db()).unwrap()),
            "logical_after":format!("{:?}",target.ledger())})).unwrap()).unwrap();
    }
    target
}

#[test]
fn restored_snapshot_old_cursor_requires_resync_without_any_logical_write() {
    let r = restored_fixture();
    let before = r.ledger();
    assert!(matches!(
        Store::open(&r.path.join("store"), id(GEN), id(OTHER), false, end()),
        Err(habitat_engine::store::Error::RecoveryRequired)
    ));
    for label in ["old-first", "old-retry"] {
        let out = r.path.join(label);
        assert_eq!(
            r.invoke_cursor(GEN, OTHER, &out, Some(&cursor_json(EPOCH, "5"))),
            (0, String::new())
        );
        let report: Value =
            serde_json::from_slice(&fs::read(out.join("report.json")).unwrap()).unwrap();
        assert_eq!(report["cursor_disposition"], "resync_required");
        assert_eq!(report["replay_authorized"], false);
        assert_eq!(report["inventory"]["epoch"], OTHER);
        assert_eq!(report["inventory"]["mode"], "reconciliation");
        assert_eq!(
            report["inventory"]["acceptances"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            report["inventory"]["pending_delivery"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(before, r.ledger());
    }
}

#[test]
fn restored_snapshot_current_cursor_is_snapshot_only_never_replay() {
    let r = restored_fixture();
    let before = r.ledger();
    let out = r.path.join("current");
    assert_eq!(
        r.invoke_cursor(GEN, OTHER, &out, Some(&cursor_json(OTHER, "0"))),
        (0, String::new())
    );
    let report: Value =
        serde_json::from_slice(&fs::read(out.join("report.json")).unwrap()).unwrap();
    assert_eq!(report["cursor_disposition"], "snapshot_only");
    assert_eq!(report["replay_authorized"], false);
    assert_eq!(report["filter_visibility_retention"], "unassessed");
    assert_eq!(before, r.ledger());
}

#[test]
fn actual_inspector_rejects_future_sequence_and_expired_cursor() {
    let mut r = Rig::new();
    r.admit();
    r.close();
    let before = r.ledger();
    let future = cursor_json(EPOCH, "2");
    let mut expired: Value = serde_json::from_str(&cursor_json(EPOCH, "1")).unwrap();
    expired["issued_unix_ms"] = json!("0");
    expired["expires_unix_ms"] = json!("1");
    for (label, cursor) in [("future", future), ("expired", expired.to_string())] {
        let out = r.path.join(label);
        assert_eq!(r.invoke_cursor(GEN, EPOCH, &out, Some(&cursor)).0, 1);
        assert!(!out.join("complete.json").exists());
        let error: Value =
            serde_json::from_slice(&fs::read(out.join("error.json")).unwrap()).unwrap();
        assert_eq!(error["inspection_completed"], false);
        assert_eq!(before, r.ledger());
    }
}

#[test]
fn actual_inspector_rejects_duplicate_cursor_fields_without_creating_report() {
    let mut r = Rig::new();
    r.admit();
    r.close();
    let before = r.ledger();
    let out = r.path.join("malformed");
    let cursor = cursor_json(EPOCH, "1").replacen('{', "{\"epoch\":\"invalid\",", 1);
    assert_eq!(r.invoke_cursor(GEN, EPOCH, &out, Some(&cursor)).0, 1);
    assert!(!out.exists());
    assert_eq!(before, r.ledger());
}

#[test]
fn actual_inspector_rejects_positional_cursor_record() {
    let mut r = Rig::new();
    r.admit();
    r.close();
    let before = r.ledger();
    let out = r.path.join("positional");
    let map: Value = serde_json::from_str(&cursor_json(EPOCH, "1")).unwrap();
    let raw = json!([
        map["epoch"],
        map["sequence"],
        map["filter_sha256"],
        map["visibility_revision"],
        map["issued_unix_ms"],
        map["expires_unix_ms"]
    ])
    .to_string();
    assert_eq!(r.invoke_cursor(GEN, EPOCH, &out, Some(&raw)).0, 1);
    assert!(!out.exists());
    assert_eq!(before, r.ledger());
}
