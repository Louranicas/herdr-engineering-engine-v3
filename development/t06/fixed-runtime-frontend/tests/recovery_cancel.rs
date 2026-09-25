//! Actual cancellation-finalizer commands; closed-ledger independent SQL oracle.
use habitat_engine::contracts::{Generation, Sha256Digest, UuidV4};
use habitat_engine::store::{
    Allocation, Effect, Expected, Object, Principal, Settlement, Store, Submission, Verification,
    VerificationVerdict,
};
use habitat_engine::worker::process::{self, ProcessSpec};
use rusqlite::{Connection, OpenFlags, types::Value as SqlValue};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::PathBuf,
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
    Principal::new(rustix::process::geteuid().as_raw(), "operator").unwrap()
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
        self.admit_as(&principal());
    }
    fn admit_as(&mut self, owner: &Principal) {
        self.store()
            .submit(
                Submission {
                    principal: owner,
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
    fn revision(&mut self) -> String {
        self.store()
            .get(&principal(), id(TASK), end())
            .unwrap()
            .generation
    }
    fn cancelled(&mut self) {
        self.ready();
        self.verify(VerificationVerdict::Passed, Some(20), true);
        self.cancel();
    }
    fn dispatch(&self, expected: &str, event: &str, label: &str) -> (i32, PathBuf) {
        self.dispatch_epoch(expected, event, label, EPOCH)
    }
    fn dispatch_epoch(
        &self,
        expected: &str,
        event: &str,
        label: &str,
        epoch: &str,
    ) -> (i32, PathBuf) {
        let output = self.path.join(label);
        let arguments = vec![
            "finish-cancelled".into(),
            self.path.join("store").into_os_string(),
            GEN.into(),
            epoch.into(),
            TASK.into(),
            expected.into(),
            event.into(),
            output.clone().into_os_string(),
        ];
        let spec = ProcessSpec {
            executable: PathBuf::from(env!("CARGO_BIN_EXE_hee3-fixed-runtime-frontend")),
            arguments,
            directory: self.path.clone(),
            environment: vec![("LC_ALL".into(), "C".into())],
            input: vec![],
            stream_limit: 65_536,
        };
        let r = process::run(&spec, end(), &AtomicBool::new(false)).unwrap();
        if let Ok(evidence) = std::env::var("T07_CANCEL_EVIDENCE") {
            let name = std::thread::current()
                .name()
                .unwrap_or("unnamed")
                .replace("::", "-");
            let directory = PathBuf::from(evidence).join(format!("{name}-{label}"));
            fs::create_dir_all(&directory).unwrap();
            fs::write(
                directory.join("process.txt"),
                format!("argv={:?}\n{r:#?}", spec.arguments),
            )
            .unwrap();
            for name in ["report.json", "complete.json"] {
                if output.join(name).is_file() {
                    fs::copy(output.join(name), directory.join(name)).unwrap();
                }
            }
        }
        assert!(
            r.pending.is_none() && r.leader_reaped && r.process_group_settled,
            "{r:?}"
        );
        assert_eq!(r.signal, None);
        assert_eq!(r.interruption, None);
        assert!(
            r.stdout.eof
                && r.stderr.eof
                && !r.stdout.failed
                && !r.stderr.failed
                && !r.stdout.truncated
                && !r.stderr.truncated
        );
        assert!(r.stdout.bytes.is_empty());
        let status = r.exit_code.unwrap();
        assert_eq!(r.stderr.bytes.is_empty(), status == 0, "{r:?}");
        (status, output)
    }
    fn invoke(&self, expected: &str, event: &str, label: &str) -> (i32, Value) {
        let (status, output) = self.dispatch(expected, event, label);
        let bytes = fs::read(output.join("report.json")).unwrap();
        let report: Value = serde_json::from_slice(&bytes).unwrap();
        let completion: Value =
            serde_json::from_slice(&fs::read(output.join("complete.json")).unwrap()).unwrap();
        assert_eq!(completion["report_bytes"], bytes.len());
        assert_eq!(completion["report_sha256"], expected_digest(&bytes));
        assert_eq!(report["recovery_complete"], false);
        assert_eq!(report["execution_resumed"], false);
        assert_eq!(report["delivery_performed"], false);
        (status, report)
    }
    fn successful(&mut self) -> Value {
        let generation = self.revision();
        self.close();
        let before = self.ledger();
        let (status, report) = self.invoke(&generation, FINAL, "finalized");
        assert_eq!(status, 0, "{report}");
        assert_eq!(report["operation"]["status"], "ok");
        assert_eq!(report["readback"]["status"], "ok");
        self.check_delta(&before, &report);
        report
    }
    fn check_delta(&self, before: &BTreeMap<String, Vec<Vec<SqlValue>>>, report: &Value) {
        let after = self.ledger();
        for (table, rows) in before {
            if ![
                "artifacts",
                "events",
                "outbox",
                "task_stops",
                "tasks",
                "sqlite_sequence",
            ]
            .contains(&table.as_str())
            {
                assert_eq!(rows, &after[table], "{table}");
            }
        }
        for table in ["artifacts", "events", "outbox", "task_stops"] {
            assert_eq!(after[table].len(), before[table].len() + 1, "{table}");
        }
        assert_eq!(after["acceptances"].len(), 0);
        let db = Connection::open_with_flags(self.db(), OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let row: (String, String, String, String) = db
            .query_row(
                "SELECT task_id,event_id,reason,state FROM task_stops",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            row,
            (
                TASK.to_owned(),
                FINAL.to_owned(),
                "recovery_cancelled".to_owned(),
                "cancelled".to_owned()
            )
        );
        let row: (String, i64, i64) = db
            .query_row(
                "SELECT state,reserved_work_ms,reserved_verify_ms FROM tasks WHERE id=?",
                [TASK],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(row, ("cancelled".to_owned(), 0, 0));
        let outbox: (String, String, i64) = db
            .query_row("SELECT event_id,recipient,delivered FROM outbox", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .unwrap();
        assert_eq!(
            outbox,
            (
                FINAL.to_owned(),
                format!("{}:operator", rustix::process::geteuid().as_raw()),
                0
            )
        );
        let spent: i64 = db
            .query_row("SELECT spent_ms FROM tasks WHERE id=?", [TASK], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(report["readback"]["value"]["tasks"][0]["spent_ms"], spent);
        assert_eq!(after["tasks"][0][10], before["tasks"][0][10]); // actual spent column remains exact
        db.close().unwrap();
    }
    fn refused(&mut self) {
        let generation = self.revision();
        self.close();
        let before = self.ledger();
        let (status, report) = self.invoke(&generation, FINAL, "refused");
        assert_eq!(status, 1, "{report}");
        assert_eq!(report["operation"]["status"], "error");
        assert_eq!(report["readback"]["status"], "ok");
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
const FINAL: &str = "07000000-0000-4000-8000-00000000000f";
fn expected_digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(s, "{byte:02x}").unwrap();
    }
    s
}
#[test]
fn settled_cancel_completes_once_with_exact_spend() {
    let mut r = Rig::new();
    r.cancelled();
    let v = r.successful();
    assert_eq!(v["operation"]["value"]["generation"], "6");
    assert_eq!(v["operation"]["value"]["replayed"], false);
    assert_eq!(
        v["operation"]["value"]["terminal_operation_committed"],
        true
    );
    assert_eq!(
        v["operation"]["value"]["current_invocation_wrote_terminal"],
        true
    );
    assert!(v["operation"]["value"].get("effect").is_none());
    assert_eq!(v["readback"]["value"]["tasks"][0]["spent_ms"], 47_997);
}
#[test]
fn lost_reply_exact_retry_reads_same_stop_without_writes() {
    let mut r = Rig::new();
    r.cancelled();
    let original = r.revision();
    r.close();
    let before_first = r.ledger();
    let (_unconsumed_status, _unconsumed_report) = r.dispatch(&original, FINAL, "unconsumed-first");
    let before = r.ledger();
    assert_eq!(
        before["task_stops"].len(),
        before_first["task_stops"].len() + 1
    );
    let (status, v) = r.invoke(&original, FINAL, "unconsumed-reply-retry");
    assert_eq!(status, 0, "{v}");
    assert_eq!(v["operation"]["value"]["replayed"], true);
    assert_eq!(
        v["operation"]["value"]["terminal_operation_committed"],
        true
    );
    assert_eq!(
        v["operation"]["value"]["current_invocation_wrote_terminal"],
        false
    );
    assert!(v["operation"]["value"].get("effect").is_none());
    assert_eq!(v["operation"]["value"]["event"], FINAL);
    assert_eq!(before, r.ledger());
}
#[test]
fn changed_retry_event_is_not_normalized_to_previous_operation() {
    let mut r = Rig::new();
    r.cancelled();
    let g = r.revision();
    r.successful();
    let before = r.ledger();
    let (status, _) = r.invoke(&g, OTHER, "foreign-event");
    assert_eq!(status, 1);
    assert_eq!(before, r.ledger());
    assert_eq!(r.invoke(&g, FINAL, "benign-retry").0, 0);
    assert_eq!(before, r.ledger());
}
#[test]
fn changed_retry_generation_refuses() {
    let mut r = Rig::new();
    r.cancelled();
    r.successful();
    let before = r.ledger();
    assert_eq!(r.invoke("6", FINAL, "foreign-generation").0, 1);
    assert_eq!(before, r.ledger());
}
#[test]
fn uncancelled_exhaustion_is_not_cancellation() {
    let mut r = Rig::new();
    r.ready();
    r.verify(VerificationVerdict::Failed, Some(20), true);
    r.refused();
}
#[test]
fn accepted_history_is_not_rewritten() {
    let mut r = Rig::new();
    r.accept();
    r.refused();
}
#[test]
fn cancellation_without_attempts_keeps_preparation_obligation() {
    let mut r = Rig::new();
    r.admit();
    r.cancel();
    r.refused();
}
#[test]
fn running_worker_cannot_be_settled_by_reopen() {
    let mut r = Rig::new();
    r.start();
    r.cancel();
    r.refused();
}
#[test]
fn missing_verifier_is_not_invented_zero() {
    let mut r = Rig::new();
    r.ready();
    r.cancel();
    r.refused();
}
#[test]
fn unknown_work_usage_keeps_reservation() {
    let mut r = Rig::new();
    r.start();
    r.settle(Effect::None, None, true);
    r.cancel();
    r.refused();
}
#[test]
fn pending_effect_keeps_reservation() {
    let mut r = Rig::new();
    r.start();
    r.settle(Effect::Pending, Some(20), true);
    r.cancel();
    r.refused();
}
#[test]
fn unknown_effect_keeps_reservation() {
    let mut r = Rig::new();
    r.start();
    r.settle(Effect::Unknown, Some(20), true);
    r.cancel();
    r.refused();
}
#[test]
fn partial_cleanup_keeps_reservation() {
    let mut r = Rig::new();
    r.start();
    r.settle(Effect::None, Some(20), false);
    r.cancel();
    r.refused();
}
#[test]
fn unknown_verifier_usage_keeps_reservation() {
    let mut r = Rig::new();
    r.ready();
    r.verify(VerificationVerdict::Error, None, true);
    r.cancel();
    r.refused();
}
#[test]
fn partial_verifier_cleanup_keeps_reservation() {
    let mut r = Rig::new();
    r.ready();
    r.verify(VerificationVerdict::Error, Some(20), false);
    r.cancel();
    r.refused();
}
#[test]
fn stale_generation_refuses_then_current_neighbor_finishes() {
    let mut r = Rig::new();
    r.cancelled();
    let g = r.revision();
    r.close();
    let before = r.ledger();
    assert_eq!(r.invoke("4", FINAL, "stale").0, 1);
    assert_eq!(before, r.ledger());
    let (status, v) = r.invoke(&g, FINAL, "current");
    assert_eq!(status, 0);
    r.check_delta(&before, &v);
}
#[test]
fn foreign_role_refuses_without_impersonating_stored_owner() {
    let mut r = Rig::new();
    let owner = Principal::new(rustix::process::geteuid().as_raw(), "other").unwrap();
    r.admit_as(&owner);
    r.store()
        .cancel(id(TASK), "1".parse().unwrap(), id(CANCEL), end())
        .unwrap();
    r.close();
    let before = r.ledger();
    let (status, v) = r.invoke("2", FINAL, "foreign-role");
    assert_eq!(status, 1);
    assert!(
        v["operation"]["error"]
            .as_str()
            .unwrap()
            .contains("NotFound")
    );
    assert_eq!(v["readback"]["status"], "error");
    assert!(v["readback"]["value"].is_null());
    assert_eq!(before, r.ledger());
}
#[test]
fn foreign_uid_refuses_without_impersonating_stored_owner() {
    let mut r = Rig::new();
    let owner = Principal::new(rustix::process::geteuid().as_raw() + 1, "operator").unwrap();
    r.admit_as(&owner);
    r.store()
        .cancel(id(TASK), "1".parse().unwrap(), id(CANCEL), end())
        .unwrap();
    r.close();
    let before = r.ledger();
    assert_eq!(r.invoke("2", FINAL, "foreign-uid").0, 1);
    assert_eq!(before, r.ledger());
}
#[test]
fn event_collision_rolls_back_all_ledger_changes() {
    let mut r = Rig::new();
    r.cancelled();
    let g = r.revision();
    r.close();
    let before = r.ledger();
    assert_eq!(r.invoke(&g, ADMIT, "collision").0, 1);
    assert_eq!(before, r.ledger());
}
#[test]
fn replay_missing_evidence_does_not_rewrite_history() {
    let mut r = Rig::new();
    r.cancelled();
    let g = r.revision();
    let v = r.successful();
    let before = r.ledger();
    let digest = v["operation"]["value"]["evidence"]["digest"]
        .as_str()
        .unwrap()
        .strip_prefix("sha256:")
        .unwrap();
    let path = r
        .path
        .join("store/generations")
        .join(GEN)
        .join("objects/sha256")
        .join(&digest[..2])
        .join(digest);
    let retained = fs::read(&path).unwrap();
    let mode = fs::metadata(&path).unwrap().permissions();
    fs::remove_file(&path).unwrap();
    assert_eq!(r.invoke(&g, FINAL, "missing-proof").0, 1);
    assert_eq!(before, r.ledger());
    fs::write(&path, retained).unwrap();
    fs::set_permissions(path, mode).unwrap();
    assert_eq!(r.invoke(&g, FINAL, "restored-proof").0, 0);
    assert_eq!(before, r.ledger());
}

#[test]
fn actual_owned_child_cancel_cleanup_then_restart_finalization() {
    struct CancelOwner<'a> {
        store: &'a mut Store,
        token: &'a AtomicBool,
        result: Option<Result<String, habitat_engine::store::Error>>,
        deadline: Instant,
    }
    impl process::Observer for CancelOwner<'_> {
        fn observe(&mut self, _: &process::ProcessObservation<'_>) -> process::ObserverDecision {
            if self.result.is_none() {
                self.result = Some(self.store.cancel(
                    id(TASK),
                    "2".parse().unwrap(),
                    id(CANCEL),
                    self.deadline,
                ));
                self.token.store(true, Ordering::Release);
            }
            process::ObserverDecision {
                hold_stdin: false,
                ready_to_finish: true,
                stop: None,
            }
        }
    }
    let mut r = Rig::new();
    r.start();
    let token = AtomicBool::new(false);
    let deadline = end();
    let spec = ProcessSpec {
        executable: PathBuf::from("/usr/bin/sleep"),
        arguments: vec!["5".into()],
        directory: r.path.clone(),
        environment: vec![("LC_ALL".into(), "C".into())],
        input: vec![],
        stream_limit: 4096,
    };
    let mut owner = CancelOwner {
        store: r.store(),
        token: &token,
        result: None,
        deadline,
    };
    let raw = process::run_observed(&spec, deadline, &token, &mut owner).unwrap();
    assert_eq!(owner.result.unwrap().unwrap(), "3");
    if let Ok(path) = std::env::var("T07_CANCEL_EVIDENCE") {
        fs::create_dir_all(&path).unwrap();
        fs::write(
            PathBuf::from(path).join("actual-owned-child.txt"),
            format!("{spec:#?}\n{raw:#?}"),
        )
        .unwrap();
    }
    assert_eq!(raw.interruption, Some(process::Interruption::Cancelled));
    assert!(
        raw.pending.is_none() && raw.leader_reaped && raw.process_group_settled,
        "{raw:#?}"
    );
    assert!(
        raw.stdout.eof
            && raw.stderr.eof
            && !raw.stdout.failed
            && !raw.stderr.failed
            && !raw.stdout.truncated
            && !raw.stderr.truncated
    );
    let used = u64::try_from(raw.elapsed.as_nanos().div_ceil(1_000_000)).unwrap();
    r.settle(Effect::None, Some(used), true);
    // This fixture never invokes a checker: the trusted owner records that fact,
    // rather than deriving verifier idleness from an absent durable row.
    r.evidence = r
        .store()
        .publish(
            b"local fixture: cancelled before verifier dispatch; no checker invoked",
            id(OTHER),
            end(),
        )
        .unwrap();
    r.verify(VerificationVerdict::Cancelled, Some(0), true);
    let report = r.successful();
    assert_eq!(report["readback"]["value"]["tasks"][0]["spent_ms"], used);
    assert_eq!(
        report["readback"]["value"]["verifications"][0]["verdict"],
        "cancelled"
    );
}
#[test]
fn expired_administrative_deadline_cannot_commit_or_reset_task_budget() {
    let mut r = Rig::new();
    r.cancelled();
    let generation = r.revision();
    r.close();
    let before = r.ledger();
    let root = r.path.join("store");
    let output = r.path.join("expired");
    let request = hee3_fixed_runtime_frontend::recovery_cancel::Request {
        root: &root,
        store_generation: GEN,
        epoch: EPOCH,
        task: TASK,
        expected_generation: &generation,
        event: FINAL,
    };
    assert!(
        hee3_fixed_runtime_frontend::recovery_cancel::finish_until(
            request,
            &output,
            Instant::now()
        )
        .is_err()
    );
    assert!(!output.exists());
    assert_eq!(before, r.ledger());
}

#[test]
fn wrong_epoch_refuses_before_terminal_write_then_correct_neighbor_succeeds() {
    let mut r = Rig::new();
    r.cancelled();
    let g = r.revision();
    r.close();
    let before = r.ledger();
    assert_eq!(r.dispatch_epoch(&g, FINAL, "wrong-epoch", OTHER).0, 1);
    assert_eq!(before, r.ledger());
    let (status, v) = r.invoke(&g, FINAL, "correct-epoch");
    assert_eq!(status, 0);
    r.check_delta(&before, &v);
}
#[test]
fn duplicate_store_owner_blocks_finalization() {
    let mut r = Rig::new();
    r.cancelled();
    let g = r.revision();
    let before = r.ledger();
    let (status, v) = r.invoke(&g, FINAL, "locked");
    assert_eq!(status, 1);
    assert!(v["operation"]["error"].as_str().unwrap().contains("Locked"));
    assert_eq!(before, r.ledger());
}
#[test]
fn known_committed_effect_is_retained_in_cancelled_history() {
    let mut r = Rig::new();
    r.start();
    r.settle(Effect::Committed, Some(47_977), true);
    r.verify(VerificationVerdict::Passed, Some(20), true);
    r.cancel();
    let v = r.successful();
    assert_eq!(v["readback"]["value"]["attempts"][0]["effect"], "committed");
}

#[test]
fn authorized_readback_does_not_export_another_principals_task() {
    let mut r = Rig::new();
    r.cancelled();
    let foreign = Principal::new(rustix::process::geteuid().as_raw() + 1, "other").unwrap();
    r.store()
        .submit(
            Submission {
                principal: &foreign,
                key: id("07000000-0000-4000-8000-000000000021"),
                task: id(OTHER),
                event: id("07000000-0000-4000-8000-000000000022"),
                request_bytes: b"foreign task privacy fixture",
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
    let before = r.ledger();
    let v = r.successful();
    let tasks = v["readback"]["value"]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"], TASK);
    let after = r.ledger();
    let foreign = |rows: &BTreeMap<String, Vec<Vec<SqlValue>>>| {
        rows["tasks"]
            .iter()
            .find(|row| row[0] == SqlValue::Text(OTHER.to_owned()))
            .unwrap()
            .clone()
    };
    assert_eq!(foreign(&before), foreign(&after));
}
