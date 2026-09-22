//! Actual test-only local coordinator death followed by production recovery readback.
use habitat_engine::{
    contracts::UuidV4,
    store::Store,
    worker::process::{self, ProcessSpec},
};
use rusqlite::{Connection, OpenFlags, types::Value as SqlValue};
use serde_json::Value;
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
const NEW_ATTEMPT: &str = "07000000-0000-4000-8000-000000000020";
const NEW_EVENT: &str = "07000000-0000-4000-8000-000000000021";
static NEXT: AtomicU64 = AtomicU64::new(0);
fn id(v: &str) -> UuidV4<'_> {
    UuidV4::parse(v).unwrap()
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn package() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn digest(p: &Path) -> String {
    use std::fmt::Write as _;
    let mut result = "sha256:".to_owned();
    for byte in Sha256::digest(fs::read(p).unwrap()) {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}

struct Area {
    path: PathBuf,
    inode: (u64, u64),
    evidence: PathBuf,
}
impl Area {
    fn new(mode: &str) -> Self {
        let key = format!(
            "crash-{}-{}-{mode}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().canonicalize().unwrap().join(&key);
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        let m = fs::metadata(&path).unwrap();
        let evidence =
            PathBuf::from(std::env::var_os("T07_CANCEL_EVIDENCE").expect("owned evidence root"))
                .join(key);
        fs::create_dir_all(&evidence).unwrap();
        Self {
            path,
            inode: (m.dev(), m.ino()),
            evidence,
        }
    }
    fn run(&self, exe: &Path, args: Vec<std::ffi::OsString>, label: &str) -> i32 {
        let spec = ProcessSpec {
            executable: exe.to_owned(),
            arguments: args,
            directory: package(),
            environment: vec![
                ("LC_ALL".into(), "C".into()),
                ("TMPDIR".into(), std::env::temp_dir().into_os_string()),
                ("PYTHONDONTWRITEBYTECODE".into(), "1".into()),
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
            self.evidence.join(format!("{label}-process.txt")),
            format!("{spec:#?}\n{report:#?}"),
        )
        .unwrap();
        assert!(
            report.signal.is_none()
                && report.interruption.is_none()
                && report.pending.is_none()
                && report.leader_reaped
                && report.process_group_settled
                && report.stdout.eof
                && report.stderr.eof
                && !report.stdout.failed
                && !report.stderr.failed
                && !report.stdout.truncated
                && !report.stderr.truncated,
            "outer owner: {report:#?}"
        );
        fs::write(
            self.evidence.join(format!("{label}-stdout")),
            &report.stdout.bytes,
        )
        .unwrap();
        fs::write(
            self.evidence.join(format!("{label}-stderr")),
            &report.stderr.bytes,
        )
        .unwrap();
        report.exit_code.unwrap()
    }
    fn fixture(&self, mode: &str) {
        let exe = PathBuf::from(env!("CARGO_BIN_EXE_hee3-recovery-crash-fixture"));
        assert_eq!(
            self.run(
                Path::new("/usr/bin/python3"),
                vec![
                    "-B".into(),
                    package()
                        .join("tests/fixtures/recovery_crash.py")
                        .into_os_string(),
                    exe.clone().into_os_string(),
                    digest(&exe).into(),
                    mode.into(),
                    self.path.clone().into_os_string(),
                    self.evidence.join("inner").into_os_string()
                ],
                "fixture"
            ),
            0
        );
        let raw: Value =
            serde_json::from_slice(&fs::read(self.evidence.join("inner/result.json")).unwrap())
                .unwrap();
        assert_eq!(raw["accepted_control"], true);
        assert_eq!(
            raw["descendants_detected"],
            matches!(mode, "pre-ack" | "post-ack")
        );
        for name in [
            "cut.json",
            "ack.json",
            "live-writer.json",
            "process.json",
            "benign.json",
        ] {
            let p = self.path.join(name);
            if p.exists() {
                fs::copy(p, self.evidence.join(name)).unwrap();
            }
        }
    }
    fn db(&self) -> Connection {
        let d = Connection::open_with_flags(
            self.path
                .join("store/generations")
                .join(GEN)
                .join("ledger.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        d.execute_batch("PRAGMA query_only=ON").unwrap();
        d
    }
    fn snapshot(&self) -> BTreeMap<String, Vec<Vec<SqlValue>>> {
        let db = self.db();
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
            let count = q.column_count();
            let mut rows: Vec<Vec<SqlValue>> = q
                .query_map([], |r| (0..count).map(|i| r.get(i)).collect())
                .unwrap()
                .map(Result::unwrap)
                .collect();
            rows.sort_by_key(|r| format!("{r:?}"));
            out.insert(name, rows);
        }
        db.close().unwrap();
        out
    }
    fn inspect(&self, label: &str) -> Value {
        let path = self.path.join(label);
        assert_eq!(
            self.run(
                Path::new(env!("CARGO_BIN_EXE_hee3-fixed-runtime-frontend")),
                vec![
                    "inspect-recovery".into(),
                    self.path.join("store").into_os_string(),
                    GEN.into(),
                    EPOCH.into(),
                    path.clone().into_os_string()
                ],
                label
            ),
            0
        );
        for n in ["report.json", "complete.json"] {
            fs::copy(path.join(n), self.evidence.join(format!("{label}-{n}"))).unwrap();
        }
        serde_json::from_slice(&fs::read(path.join("report.json")).unwrap()).unwrap()
    }
    fn check(&self, mode: &str) {
        let before = self.snapshot();
        let db = self.db();
        let (state, generation, work, verify): (String, String, i64, i64) = db
            .query_row(
                "SELECT state,generation,reserved_work_ms,reserved_verify_ms FROM tasks WHERE id=?",
                [TASK],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        let accepted = matches!(mode, "acceptance-return" | "benign");
        let verifying = matches!(
            mode,
            "worker-return" | "evidence-published" | "verification-return"
        );
        assert_eq!(
            state,
            if accepted {
                "accepted"
            } else if verifying {
                "verifying"
            } else {
                "running"
            }
        );
        assert_eq!(
            generation,
            if accepted {
                "5"
            } else if mode == "verification-return" {
                "4"
            } else if verifying {
                "3"
            } else {
                "2"
            }
        );
        assert_eq!(before["tasks"].len(), 1);
        assert_eq!(before["attempts"].len(), 1);
        assert_eq!(before["acceptances"].len(), usize::from(accepted));
        assert!(before["task_stops"].is_empty());
        if matches!(mode, "pre-ack" | "post-ack") {
            assert_eq!((work, verify), (900_000, 300_000));
            assert!(before["verifications"].is_empty());
        }
        if accepted {
            assert_eq!((work, verify), (0, 0));
        }
        let task_outbox:i64=db.query_row("SELECT count(*) FROM outbox JOIN events ON outbox.event_id=events.id WHERE events.task_id=?",[TASK],|r|r.get(0)).unwrap();
        assert_eq!(task_outbox, i64::from(accepted));
        db.close().unwrap();
        // The engine policy's decision the inspector must report for this boundary:
        // a running attempt whose worker identity is not a process identity is
        // unobserved; a settled attempt without a cleanup readback is unverified;
        // a committed acceptance stands once with its event.
        let (name, rule, reason) = if accepted {
            ("acceptance_stands", "R04", None)
        } else if verifying {
            ("retain_unknown", "R11", Some("cleanup_unverified"))
        } else {
            ("retain_unknown", "R07", Some("process_unobserved"))
        };
        for label in ["inspect-first", "inspect-retry"] {
            let report = self.inspect(label);
            assert_eq!(report["recovery_complete"], false);
            assert_eq!(report["execution_resumed"], false);
            assert_eq!(report["policy_permits_execution"], false);
            assert_eq!(
                report["policy_permits_execution"],
                report["execution_resumed"]
            );
            assert_eq!(report["physical_process_custody"], "unreconciled");
            assert_eq!(report["pi_queue_custody"], "unreconciled");
            let decisions = report["reconciliation"]["attempts"].as_array().unwrap();
            assert_eq!(decisions.len(), 1, "{decisions:?}");
            let d = &decisions[0];
            assert_eq!(d["task"], TASK);
            assert_eq!(d["rule"], rule, "{mode}: {d}");
            assert_eq!(d["name"], name, "{mode}: {d}");
            assert_eq!(d["decision"]["decision"], name);
            if let Some(reason) = reason {
                assert_eq!(d["decision"]["reason"]["reason"], reason, "{mode}: {d}");
                assert_eq!(d["decision"]["process"]["custody"], "unobserved");
                assert_eq!(d["decision"]["cancellation_pending"], false);
            } else {
                assert_eq!(
                    d["decision"]["event"],
                    report["inventory"]["tasks"][0]["accepted_event"]
                );
                assert_eq!(d["decision"]["later_cancellation"], Value::Null);
                assert_eq!(d["decision"]["cleanup"], "settled");
            }
            assert_eq!(self.snapshot(), before);
        }
        fs::write(
            self.evidence.join("logical-snapshot.txt"),
            format!("{before:#?}"),
        )
        .unwrap();
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
    a.fixture(mode);
    a.check(mode);
}
#[test]
fn crash_before_correlated_ack_retains_unresolved_attempt() {
    case("pre-ack");
}
#[test]
fn crash_after_correlated_ack_retains_unresolved_attempt() {
    case("post-ack");
}
#[test]
fn matched_benign_local_adapter_can_complete() {
    case("benign");
}
#[test]
fn crash_after_worker_return_preserves_verification_obligation() {
    case("worker-return");
}
#[test]
fn crash_after_cas_publication_never_infers_verification() {
    case("evidence-published");
}
#[test]
fn crash_after_verification_return_is_not_acceptance() {
    case("verification-return");
}
#[test]
fn crash_after_acceptance_return_preserves_completion_once() {
    case("acceptance-return");
}
#[test]
fn stale_generation_after_actual_crash_cannot_dispatch() {
    let a = Area::new("stale");
    a.fixture("post-ack");
    let before = a.snapshot();
    let mut store = Store::open(&a.path.join("store"), id(GEN), id(EPOCH), false, end()).unwrap();
    for generation in ["1", "2"] {
        assert!(
            store
                .begin_attempt(
                    id(TASK),
                    generation.parse().unwrap(),
                    id(NEW_ATTEMPT),
                    id(NEW_EVENT),
                    end()
                )
                .is_err()
        );
    }
    drop(store);
    assert_eq!(a.snapshot(), before);
    a.check("post-ack");
}
#[test]
fn partial_cleanup_cannot_pass_actual_crash_oracle() {
    let a = Area::new("cleanup-oracle");
    a.fixture("post-ack");
    let script = "import importlib.util,json,sys; from pathlib import Path; s=importlib.util.spec_from_file_location('oracle',sys.argv[1]); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); r=json.loads(Path(sys.argv[2]).read_text()); r['cleanup_complete']=False; m.validate(r,'post-ack',Path(sys.argv[3]))";
    assert_eq!(
        a.run(
            Path::new("/usr/bin/python3"),
            vec![
                "-B".into(),
                "-c".into(),
                script.into(),
                package()
                    .join("tests/fixtures/recovery_crash.py")
                    .into_os_string(),
                a.evidence.join("inner/result.json").into_os_string(),
                a.path.clone().into_os_string()
            ],
            "cleanup-refusal"
        ),
        1
    );
    assert!(
        fs::read_to_string(a.evidence.join("cleanup-refusal-stderr"))
            .unwrap()
            .contains("Actual loss/custody result refused")
    );
    a.check("post-ack");
}
