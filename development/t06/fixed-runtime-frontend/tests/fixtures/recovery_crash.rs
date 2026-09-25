//! Test-only literal local adapter and abrupt coordinator-loss fixture.
//! No production recovery, dispatch grant, Pi or native isolation claim.
use habitat_engine::{
    contracts::{Sha256Digest, UuidV4},
    store::{
        Allocation, Effect, Expected, Principal, RecoveryLimits, Settlement, Store, Submission,
        Verification, VerificationVerdict,
    },
    worker::{
        Binding, Capabilities, Contract, Envelope, Event, Feature, Invocation, Request, Selection,
        process::{self, Observer, ObserverDecision, ProcessObservation, ProcessSpec},
    },
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
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
const STAGE: &str = "07000000-0000-4000-8000-00000000000d";
const OTHER: &str = "07000000-0000-4000-8000-00000000000e";
const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn id(v: &str) -> UuidV4<'_> {
    UuidV4::parse(v).unwrap()
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn principal() -> Principal {
    Principal::new(rustix::process::geteuid().as_raw(), "operator").unwrap()
}
fn invocation() -> Invocation<'static> {
    Invocation {
        binding: Binding {
            task: id(TASK),
            attempt: id(ATTEMPT),
            generation: "1".parse().unwrap(),
        },
        id: id(OTHER),
    }
}
fn ack() -> String {
    format!("ACK {TASK} {ATTEMPT} 1 {OTHER}\n")
}
fn retained(path: &Path, value: &serde_json::Value) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(&serde_json::to_vec(value).unwrap()).unwrap();
    file.sync_all().unwrap();
    File::open(path.parent().unwrap())
        .unwrap()
        .sync_all()
        .unwrap();
}
fn crash(path: &Path, phase: &str) {
    retained(
        &path.join("cut.json"),
        &json!({"phase":phase,"signal":9,"kind":"test-only returned-boundary cut"}),
    );
    rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::KILL)
        .unwrap();
    std::process::exit(90)
}
fn rostered(path: &Path) -> Store {
    use habitat_engine::contracts::roster::{
        Availability, Kind, Locality, ObservationInput, ObservationSource, RosterDefinitionV1,
        Selection, Update,
    };
    use habitat_engine::store::{RequestSource, RosterStart};
    fs::DirBuilder::new()
        .mode(0o700)
        .create(path.join("store"))
        .unwrap();
    let mut store = Store::open(&path.join("store"), id(GEN), id(EPOCH), true, end()).unwrap();
    store
        .submit(
            Submission {
                principal: &principal(),
                key: id(KEY),
                task: id(TASK),
                event: id(ADMIT),
                request_bytes: b"literal crash fixture",
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
    let input = Update {
        idempotency_key: OTHER.into(),
        record_id: None,
        expected_revision: None,
        definition: RosterDefinitionV1 {
            kind: Kind::Agent,
            display_name: "recovery-crash-fixture".into(),
            owner_id: "fixture-worker".into(),
            version: "v1".into(),
            capabilities: vec!["text".into()],
            locality: Locality::Local,
            endpoint_ref: Some(OTHER.into()),
            limitations: "test-only literal local adapter; no runtime grant".into(),
        },
        audit_reason: "bounded abrupt-loss fixture".into(),
    };
    let raw=serde_json::to_vec(&json!({"protocol":"hee3.control","version":1,"kind":"request","request_id":OTHER,"action":"roster.update","action_version":1,"idempotency_key":OTHER,"deadline_unix_ms":"1030000","authority":{"grant_id":OTHER,"scope_sha256":DIGEST},"precondition":null,"body":{"record_id":null,"definition":input.definition,"audit_reason":input.audit_reason}})).unwrap();
    let head = store
        .roster_apply(&principal(), &[input], RequestSource::Native(&raw), end())
        .unwrap()
        .remove(0)
        .head;
    store
        .roster_observe_worker(
            &principal(),
            &ObservationInput {
                record_id: head.record_id.clone(),
                record_version: head.record_version.clone(),
                owner_id: head.definition.owner_id.clone(),
                endpoint_ref: head.definition.endpoint_ref.clone(),
                instance_id: None,
                instance_generation: None,
                source: ObservationSource::Worker,
                observed_unix_ms: None,
                availability: Availability::Available,
                actual_identity: Some("literal/fixture".into()),
                immutable_revision: None,
                capabilities: vec!["text".into()],
                evidence_ref: OTHER.into(),
            },
            end(),
        )
        .unwrap();
    let selections = [Selection {
        record_id: head.record_id.clone(),
        expected_revision: head.record_version,
        capabilities: vec!["text".into()],
        local_only: true,
        version: Some("v1".into()),
        ttl_ms: 60_000,
    }];
    store
        .begin_rostered_attempt(
            RosterStart {
                principal: &principal(),
                task: id(TASK),
                expected: "1".parse().unwrap(),
                attempt: id(ATTEMPT),
                event: id(START),
                agent_record_id: &head.record_id,
                session: id(KEY),
                workspace: id(STAGE),
                selections: &selections,
                lease_ms: 1,
            },
            end(),
        )
        .unwrap();
    store
}
struct Watch {
    store: Store,
    path: PathBuf,
    mode: String,
    contract: Contract<'static>,
    origin: Instant,
    acknowledged: bool,
    reuse_checked: bool,
}
impl Observer for Watch {
    fn observe(&mut self, o: &ProcessObservation<'_>) -> ObserverDecision {
        if self.mode == "pre-ack" && o.stdout.bytes.is_empty() {
            crash(&self.path, "pre-ack-held-stdin");
        }
        if !self.acknowledged && o.stdout.bytes.contains(&b'\n') {
            assert_eq!(
                o.stdout.bytes,
                ack().as_bytes(),
                "exact literal adapter correlation"
            );
            self.contract
                .observe(
                    Envelope {
                        invocation: invocation(),
                        sequence: 1,
                        event: Event::Acknowledged,
                    },
                    u64::try_from(self.origin.elapsed().as_millis()).unwrap(),
                )
                .unwrap();
            self.acknowledged = true;
            retained(
                &self.path.join("ack.json"),
                &json!({"raw":String::from_utf8(o.stdout.bytes.clone()).unwrap(),"task":TASK,"attempt":ATTEMPT,"generation":"1","invocation":OTHER,"contract_accepted":true}),
            );
        }
        if self.acknowledged && !self.reuse_checked && !o.leader_terminal {
            let snapshot = self
                .store
                .recovery_inventory(
                    id(EPOCH),
                    RecoveryLimits {
                        rows: 1024,
                        bytes: 1_048_576,
                    },
                    end(),
                )
                .unwrap();
            let now = self.store.roster_snapshot(&principal(), end()).unwrap().now;
            let instance = &snapshot.instances[0];
            let bytes = fs::read(self.path.join("workspace/output")).unwrap();
            if now.epoch == instance.started.epoch
                && now.monotonic_ms > instance.lease_expires_monotonic_ms
                && bytes.len() >= 2
            {
                let denied = self.store.begin_attempt(
                    id(TASK),
                    "2".parse().unwrap(),
                    id(ACCEPT),
                    id(CHECK),
                    end(),
                );
                assert!(denied.is_err(), "live old workspace must not be reused");
                retained(
                    &self.path.join("live-writer.json"),
                    &json!({"leader_pid":o.leader_pid,"leader_terminal":false,"workspace_bytes":bytes.len(),"lease_epoch":now.epoch,"now_ms":now.monotonic_ms,"expires_ms":instance.lease_expires_monotonic_ms,"begin_attempt_error":format!("{:?}",denied.unwrap_err()),"reserved_work_ms":snapshot.tasks[0].head.reserved_work_ms,"attempts":snapshot.attempts.len()}),
                );
                self.reuse_checked = true;
                if self.mode == "post-ack" {
                    crash(&self.path, "post-correlated-ack-live-writer");
                }
                fs::write(self.path.join("workspace/release"), b"stop").unwrap();
            }
        }
        ObserverDecision {
            hold_stdin: self.mode == "pre-ack",
            ready_to_finish: self.acknowledged && self.reuse_checked,
            stop: None,
        }
    }
}
fn adapter(path: &Path) {
    let mut input = Vec::new();
    std::io::stdin().take(4).read_to_end(&mut input).unwrap();
    if input.is_empty() {
        return;
    }
    assert_eq!(input, b"go\n");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path.join("output"))
        .unwrap();
    file.write_all(b"x").unwrap();
    file.sync_all().unwrap();
    print!("{}", ack());
    std::io::stdout().flush().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut written = 1;
    while !path.join("release").exists() {
        assert!(
            Instant::now() < deadline && written < 4096,
            "fixture writer release bound"
        );
        written += 1;
        file.write_all(b"x").unwrap();
        file.sync_all().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn expected(store: &mut Store) -> Expected<'static> {
    Expected {
        task: id(TASK),
        task_generation: store
            .get(&principal(), id(TASK), end())
            .unwrap()
            .generation
            .parse()
            .unwrap(),
        attempt: id(ATTEMPT),
        attempt_generation: "1".parse().unwrap(),
    }
}
fn driver(path: &Path, mode: &str) {
    assert!(matches!(
        mode,
        "pre-ack"
            | "post-ack"
            | "benign"
            | "worker-return"
            | "evidence-published"
            | "verification-return"
            | "acceptance-return"
    ));
    fs::DirBuilder::new()
        .mode(0o700)
        .create(path.join("workspace"))
        .unwrap();
    let store = rostered(path);
    let required = Capabilities::new(&[Feature::FinalOutput]);
    let mut contract = Contract::new(Request {
        invocation: invocation(),
        recipe: Sha256Digest::parse(DIGEST).unwrap(),
        workspace: Sha256Digest::parse(DIGEST).unwrap(),
        adapter_profile: "literal-crash-fixture".into(),
        selection: Selection {
            provider: "local".into(),
            model: "literal-writer".into(),
            effort: None,
        },
        required,
        prompt: "literal fixture; not a grant".into(),
    })
    .unwrap();
    contract.launch(required, 0).unwrap();
    let spec = ProcessSpec {
        executable: std::env::current_exe().unwrap(),
        arguments: vec!["adapter".into(), path.join("workspace").into_os_string()],
        directory: path.to_owned(),
        environment: vec![("LC_ALL".into(), "C".into())],
        input: b"go\n".to_vec(),
        stream_limit: 4096,
    };
    let mut watch = Watch {
        store,
        path: path.to_owned(),
        mode: mode.into(),
        contract,
        origin: Instant::now(),
        acknowledged: false,
        reuse_checked: false,
    };
    let report = process::run_observed(
        &spec,
        Instant::now() + Duration::from_secs(8),
        &AtomicBool::new(false),
        &mut watch,
    )
    .unwrap();
    assert_eq!(report.exit_code, Some(0));
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
            && !report.stderr.truncated
            && report.stderr.bytes.is_empty()
    );
    let used = u64::try_from(report.elapsed.as_nanos().div_ceil(1_000_000)).unwrap();
    retained(
        &path.join("process.json"),
        &json!({"raw":format!("{report:#?}"),"used_ms":used,"spec":format!("{spec:#?}")}),
    );
    let ex = expected(&mut watch.store);
    watch
        .store
        .settle_attempt(
            &ex,
            Settlement {
                effect: Effect::Committed,
                used_ms: Some(used),
                cleanup_settled: true,
                ready_to_verify: true,
            },
            id(SETTLE),
            end(),
        )
        .unwrap();
    verify_and_accept(&mut watch.store, path, mode);
}
fn verify_and_accept(store: &mut Store, path: &Path, mode: &str) {
    if mode == "worker-return" {
        crash(path, mode);
    }
    let verifier_origin = Instant::now();
    let data = fs::read(path.join("workspace/output")).unwrap();
    assert!(!data.is_empty() && data.iter().all(|v| *v == b'x'));
    let evidence=store.publish(&serde_json::to_vec(&json!({"kind":"literal local writer oracle","sha256":digest(&data),"bytes":data.len(),"valid":true})).unwrap(),id(STAGE),end()).unwrap();
    if mode == "evidence-published" {
        crash(path, mode);
    }
    let ex = expected(store);
    store
        .record_verification(
            &ex,
            &Verification {
                verdict: VerificationVerdict::Passed,
                subject: Sha256Digest::parse(DIGEST).unwrap(),
                evidence: evidence.clone(),
                used_ms: Some(
                    u64::try_from(verifier_origin.elapsed().as_nanos().div_ceil(1_000_000))
                        .unwrap(),
                ),
                cleanup_settled: true,
            },
            id(CHECK),
            end(),
        )
        .unwrap();
    if mode == "verification-return" {
        crash(path, mode);
    }
    let ex = expected(store);
    let prepared = store
        .prepare_verified_acceptance(
            &ex,
            id(ACCEPT),
            Sha256Digest::parse(DIGEST).unwrap(),
            &evidence,
            std::slice::from_ref(&evidence),
            end(),
        )
        .unwrap();
    store.accept(&prepared, 0, end()).unwrap();
    if mode == "acceptance-return" {
        crash(path, mode);
    }
    retained(
        &path.join("benign.json"),
        &json!({"accepted":true,"fixture_only":true}),
    );
}
fn digest(data: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut result = "sha256:".to_owned();
    for byte in Sha256::digest(data) {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    assert_eq!(
        args.len(),
        if args.get(1).is_some_and(|v| v == "adapter") {
            3
        } else {
            4
        }
    );
    let path = PathBuf::from(&args[2]);
    if args[1] == "adapter" {
        adapter(&path);
    } else {
        assert_eq!(args[1], "driver");
        driver(&path, args[3].to_str().unwrap());
    }
}
