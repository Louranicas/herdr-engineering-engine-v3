//! Test-only Pi protocol driver; no model call or production resume API.
use habitat_engine::{
    contracts::{Sha256Digest, UuidV4},
    store::{Allocation, Principal, Store, Submission},
    worker::pi::{Binding, Command, Frame, Observation, Refusal, RunState, Session},
};
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const GEN: &str = "07000000-0000-4000-8000-000000000001";
const EPOCH: &str = "07000000-0000-4000-8000-000000000002";
const TASK: &str = "07000000-0000-4000-8000-000000000003";
const KEY: &str = "07000000-0000-4000-8000-000000000004";
const ADMIT: &str = "07000000-0000-4000-8000-000000000005";
const ATTEMPT: &str = "07000000-0000-4000-8000-000000000006";
const START: &str = "07000000-0000-4000-8000-000000000007";
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
fn retained(path: &Path, value: &serde_json::Value) {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    f.write_all(&serde_json::to_vec(value).unwrap()).unwrap();
    f.sync_all().unwrap();
    File::open(path.parent().unwrap())
        .unwrap()
        .sync_all()
        .unwrap();
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
                request_bytes: b"literal Pi restart fixture",
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
            display_name: "recovery-pi-fixture".into(),
            owner_id: "fixture-worker".into(),
            version: "v1".into(),
            capabilities: vec!["text".into()],
            locality: Locality::Local,
            endpoint_ref: Some(OTHER.into()),
            limitations: "test-only literal local adapter; no runtime grant".into(),
        },
        audit_reason: "bounded Pi parent-loss fixture".into(),
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

struct Driver {
    session: Session<'static>,
    origin: Instant,
    path: PathBuf,
    sequence: u32,
}
impl Driver {
    fn elapsed(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_millis()).unwrap()
    }
    fn issue(&mut self, command: Command) {
        let bytes = self.session.issue(command, self.elapsed()).unwrap();
        retained(
            &self.path.join(format!("request-{}.json", self.sequence)),
            &json!({"bytes":bytes,"pending":self.session.pending_id(),"elapsed_ms":self.elapsed()}),
        );
        let mut out = std::io::stdout().lock();
        out.write_all(&bytes).unwrap();
        out.flush().unwrap();
        self.sequence += 1;
    }
    fn receive(&mut self) -> Result<Observation, Refusal> {
        let mut bytes = Vec::new();
        let n = std::io::Read::take(std::io::stdin().lock(), 4097)
            .read_until(b'\n', &mut bytes)
            .unwrap();
        assert!(bytes.len() <= 4096);
        retained(
            &self.path.join(format!("response-{}.json", self.sequence)),
            &json!({"bytes":bytes,"elapsed_ms":self.elapsed()}),
        );
        if n == 0 {
            self.session.transport_failed();
            return Err(Refusal::Poisoned);
        }
        assert_eq!(bytes.pop(), Some(b'\n'));
        let frame = Frame::parse(&bytes).unwrap();
        self.session.observe(&frame, self.elapsed())
    }
    fn refused(&mut self, expected: Refusal) {
        assert_eq!(self.receive(), Err(expected));
        assert_eq!(self.session.run_state(), RunState::Refused);
        let pending = self.session.pending_id().map(str::to_owned);
        assert_eq!(
            self.session.issue(Command::Abort, self.elapsed()),
            Err(Refusal::Poisoned)
        );
        retained(
            &self.path.join("driver-result.json"),
            &json!({"refusal":format!("{expected:?}"),"poisoned":true,"pending":pending,"generation":"2","session":self.session.session_id()}),
        );
    }
    fn crash(&self, phase: &str) {
        retained(
            &self.path.join("cut.json"),
            &json!({"phase":phase,"pending":self.session.pending_id(),"generation":"2","task":TASK,"attempt":ATTEMPT,"session":self.session.session_id()}),
        );
        rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::KILL)
            .unwrap();
        std::process::exit(90)
    }
    fn wait_peer(&self) {
        let deadline = self.origin + Duration::from_secs(8);
        while !self.path.join("clear-observed.json").exists() {
            assert!(Instant::now() < deadline, "peer cut timeout");
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let path = PathBuf::from(&args[1]);
    let mode = &args[2];
    let _store = rostered(&path);
    let mut d = Driver {
        session: Session::new(Binding {
            task: id(TASK),
            attempt: id(ATTEMPT),
            generation: "2".parse().unwrap(),
        }),
        origin: Instant::now(),
        path,
        sequence: 0,
    };
    d.issue(Command::State);
    assert!(matches!(d.receive(), Ok(Observation::State(_))));
    assert_eq!(d.session.session_id(), Some("owned-pi-fixture"));
    match mode.as_str() {
        "steering" | "follow-up" => {
            d.sequence += 1;
            d.refused(Refusal::Ordering);
            d.crash(mode);
        }
        "clear-before" | "clear-after" => {
            d.issue(Command::ClearQueue);
            d.wait_peer();
            d.crash(mode);
        }
        "wrong-session" => {
            d.issue(Command::State);
            d.refused(Refusal::Session);
        }
        "wrong-id" | "late-generation" => {
            d.issue(Command::ClearQueue);
            d.refused(Refusal::Correlation);
        }
        "eof" => {
            d.issue(Command::ClearQueue);
            d.refused(Refusal::Poisoned);
        }
        "benign" => {
            d.issue(Command::ClearQueue);
            assert_eq!(d.receive(), Ok(Observation::Acknowledged));
            d.issue(Command::Abort);
            assert_eq!(d.receive(), Ok(Observation::Acknowledged));
            d.issue(Command::State);
            assert_eq!(d.receive(), Ok(Observation::CancelledPiIdle));
            retained(
                &d.path.join("driver-result.json"),
                &json!({"pi_idle":true,"engine_acceptance":false,"generation":"2","session":d.session.session_id()}),
            );
        }
        _ => panic!("unknown mode"),
    }
}
