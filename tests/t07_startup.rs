//! T07 application slice: the startup pass feeds `recovery::reconcile` real
//! readbacks through the `Physical` seam and acts on its decision. Every case
//! asserts (a) what the shell handed the seam and the policy, from a recording
//! double that answers from a model keyed by what it was handed, (b) the decision
//! the pass received, (c) the ledger effect it performed or refused, and (d) the
//! readback after the effect. Expected values are fixed literals; the `Host`
//! cases read a real child process, a real Pi peer over pipes, and a real
//! directory.
use habitat_engine::app::startup::{
    self, Action, Claim, Cursor, Entry, Host, LedgerAccess, LiveIdentity, LiveRead, Pass, Physical,
    PiLink, Presence, ProcessIdentity, Startup, Subject, SubjectValue, classify, gone, intended,
    live_read, parse_stat, presence, proc_read_failure,
};
use habitat_engine::contracts::roster::{
    Availability, Kind, Locality, ObservationInput, ObservationSource, ReceiptTime,
    RosterDefinitionV1, Selection, Update,
};
use habitat_engine::contracts::{Generation, Sha256Digest, UuidV4, receipt::Name};
use habitat_engine::recovery::{
    Acknowledgement, AttemptState, Cleanup, CleanupReadback, CleanupTarget, CursorRefusal,
    Dimension, Evidence, GenerationSubject, Mode, PiQueueCustody, ProcessCustody, Reconciliation,
    ReuseRefusal, Rule, TaskState, Unknown, Verdict, Verification, WorkspaceReadback,
};
use habitat_engine::store::{
    Allocation, Effect, Error as StoreError, Expected, LOCK_SETTLE, Object, Principal,
    RECORD_BODY_LIMIT, RECORD_SCAN_LIMIT, ReconciliationRecord, RecordKind, RecordRow,
    RecoveryInventory, RecoveryLimits, RequestSource, RosterStart, Settlement, Stop, Store,
    Submission, VerificationVerdict, record_id,
};
use habitat_engine::worker::workspace::{Error as WorkspaceError, remove_owned};
use rusqlite::{Connection, OpenFlags};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, DirBuilder};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

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
const STOP: &str = "07000000-0000-4000-8000-00000000000c";
const STAGE: &str = "07000000-0000-4000-8000-00000000000d";
const OTHER: &str = "07000000-0000-4000-8000-00000000000e";
const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
/// The identity the pinned observation retains for the fixture worker; the
/// world model answers `/proc` reads for PID 4242 from its process table.
const IDENTITY: &str = r#"{"namespace":"pid:[4026534249]","pid":4242,"start_ticks":65353749}"#;
const PID: u32 = 4242;
const NAMESPACE: &str = "pid:[4026534249]";
const START_TICKS: u64 = 65_353_749;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn id(s: &str) -> UuidV4<'_> {
    UuidV4::parse(s).unwrap()
}
fn generation(s: &str) -> Generation {
    s.parse().unwrap()
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn limits() -> RecoveryLimits {
    RecoveryLimits {
        rows: 1024,
        bytes: 1_048_576,
    }
}
fn principal() -> Principal {
    Principal::new(rustix::process::geteuid().as_raw(), "operator").unwrap()
}
fn identity() -> ProcessIdentity {
    ProcessIdentity {
        pid: PID,
        start_ticks: START_TICKS,
        namespace: NAMESPACE.into(),
    }
}
fn present(start_ticks: u64, namespace: &str) -> LiveRead {
    LiveRead::Present(LiveIdentity {
        start_ticks,
        namespace: namespace.into(),
        state: 'S',
    })
}

// ---- the world: a recording double that answers from state keyed by what it is handed ----

#[derive(Clone, Debug, Eq, PartialEq)]
enum Call {
    Process {
        subject: SubjectValue,
        identity: ProcessIdentity,
    },
    PiQueue(SubjectValue),
    Cleanup(SubjectValue),
    Workspace(SubjectValue),
    Acknowledgement(SubjectValue),
    Clock,
    Attach {
        subject: SubjectValue,
        identity: ProcessIdentity,
    },
    Clean {
        subject: SubjectValue,
        target: String,
    },
}

#[derive(Default)]
struct World {
    /// PID → what `/proc` says. A PID the pass never observed reads as absent.
    processes: BTreeMap<u32, LiveRead>,
    /// Attempt → queue custody; an attempt with no session is unreconciled.
    queues: BTreeMap<String, PiQueueCustody>,
    /// Attempt → the obligations still remaining; no entry means the world
    /// cannot read the attempt's cleanup at all.
    obligations: BTreeMap<String, Vec<String>>,
    workspaces: BTreeMap<String, WorkspaceReadback>,
    acknowledgements: BTreeMap<String, Acknowledgement>,
    clock: Option<ReceiptTime>,
    attach_refusal: Option<String>,
    clean_refusals: BTreeSet<String>,
    calls: Vec<Call>,
    /// A second writer: when set, the readback AFTER the effect (the second `cleanup`
    /// read) first settles the attempt's cleanup column through its own connection -- the
    /// ledger changing under the pass, which only a concurrent writer can do.
    concurrent_settle: Option<PathBuf>,
    /// Block inside the cleanup effect after announcing it on stdout: the kill test's child
    /// uses it to be killed with `SIGKILL` between the durable intent row and the effect.
    block_in_clean: bool,
}

impl World {
    fn new() -> Self {
        Self::default()
    }
    fn with_process(mut self, pid: u32, live: LiveRead) -> Self {
        self.processes.insert(pid, live);
        self
    }
    fn with_queue(mut self, custody: PiQueueCustody) -> Self {
        self.queues.insert(ATTEMPT.into(), custody);
        self
    }
    fn with_obligations(mut self, remaining: &[&str]) -> Self {
        self.obligations.insert(
            ATTEMPT.into(),
            remaining.iter().map(|s| (*s).to_owned()).collect(),
        );
        self
    }
    fn with_workspace(mut self, readback: WorkspaceReadback) -> Self {
        self.workspaces.insert(ATTEMPT.into(), readback);
        self
    }
    fn with_acknowledgement(mut self, acknowledgement: Acknowledgement) -> Self {
        self.acknowledgements
            .insert(ATTEMPT.into(), acknowledgement);
        self
    }
    fn with_clock(mut self, epoch: &str, monotonic_ms: u64) -> Self {
        self.clock = Some(ReceiptTime {
            epoch: epoch.into(),
            monotonic_ms,
            unix_ms: 1_700_000_000_000 + monotonic_ms,
        });
        self
    }
    fn refusing_attach(mut self, error: &str) -> Self {
        self.attach_refusal = Some(error.into());
        self
    }
    fn refusing_clean(mut self, target: &str) -> Self {
        self.clean_refusals.insert(target.into());
        self
    }
    fn calls_of(&self, name: &str) -> Vec<&Call> {
        self.calls
            .iter()
            .filter(|call| match call {
                Call::Process { .. } => name == "process",
                Call::PiQueue(_) => name == "pi_queue",
                Call::Cleanup(_) => name == "cleanup",
                Call::Workspace(_) => name == "workspace",
                Call::Acknowledgement(_) => name == "acknowledgement",
                Call::Clock => name == "clock",
                Call::Attach { .. } => name == "attach",
                Call::Clean { .. } => name == "clean",
            })
            .collect()
    }
}

impl Physical for World {
    fn process(&mut self, subject: &Subject<'_>, identity: &ProcessIdentity) -> LiveRead {
        self.calls.push(Call::Process {
            subject: subject.to_value(),
            identity: identity.clone(),
        });
        self.processes
            .get(&identity.pid)
            .cloned()
            .unwrap_or(LiveRead::Absent)
    }
    fn pi_queue(&mut self, subject: &Subject<'_>) -> PiQueueCustody {
        self.calls.push(Call::PiQueue(subject.to_value()));
        self.queues
            .get(subject.attempt)
            .cloned()
            .unwrap_or(PiQueueCustody::Unreconciled)
    }
    fn cleanup(&mut self, subject: &Subject<'_>) -> CleanupReadback {
        self.calls.push(Call::Cleanup(subject.to_value()));
        if let Some(db) = &self.concurrent_settle
            && self.calls_of("cleanup").len() == 2
        {
            let db = Connection::open_with_flags(
                db,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )
            .unwrap();
            assert_eq!(
                db.execute(
                    "UPDATE attempts SET cleanup='settled' WHERE id=?",
                    [subject.attempt]
                )
                .unwrap(),
                1
            );
            db.close().unwrap();
        }
        match self.obligations.get(subject.attempt) {
            None => CleanupReadback::NotRead,
            Some(remaining) if remaining.is_empty() => CleanupReadback::Complete,
            Some(remaining) => CleanupReadback::Partial {
                remaining: remaining.clone(),
            },
        }
    }
    fn workspace(&mut self, subject: &Subject<'_>) -> WorkspaceReadback {
        self.calls.push(Call::Workspace(subject.to_value()));
        self.workspaces
            .get(subject.attempt)
            .copied()
            .unwrap_or(WorkspaceReadback::NotRead)
    }
    fn acknowledgement(&mut self, subject: &Subject<'_>) -> Acknowledgement {
        self.calls.push(Call::Acknowledgement(subject.to_value()));
        self.acknowledgements
            .get(subject.attempt)
            .copied()
            .unwrap_or(Acknowledgement::Unrecorded)
    }
    fn clock(&mut self) -> Option<ReceiptTime> {
        self.calls.push(Call::Clock);
        self.clock.clone()
    }
    fn attach(&mut self, subject: &Subject<'_>, identity: &ProcessIdentity) -> Result<(), String> {
        self.calls.push(Call::Attach {
            subject: subject.to_value(),
            identity: identity.clone(),
        });
        match &self.attach_refusal {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
    fn clean(&mut self, subject: &Subject<'_>, target: &str) -> Result<(), String> {
        self.calls.push(Call::Clean {
            subject: subject.to_value(),
            target: target.into(),
        });
        if self.block_in_clean {
            use std::io::Write as _;
            let mut out = std::io::stdout().lock();
            let _ = writeln!(out, "{KILL_MARKER}");
            let _ = out.flush();
            drop(out);
            // Bounded: the parent kills within its own budget; if it never does, give up.
            std::thread::sleep(std::time::Duration::from_secs(60));
            return Err("the kill never came".into());
        }
        if self.clean_refusals.contains(target) {
            return Err(format!("refused: {target}"));
        }
        let remaining = self
            .obligations
            .get_mut(subject.attempt)
            .ok_or_else(|| format!("no obligations readable for {}", subject.attempt))?;
        let index = remaining
            .iter()
            .position(|name| name == target)
            .ok_or_else(|| format!("nothing remaining named {target}"))?;
        remaining.remove(index);
        Ok(())
    }
}

// ---- the ledger fixture --------------------------------------------------------------------

struct Area {
    path: PathBuf,
}
impl Area {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "t07-startup-{}-{}-{label}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self { path }
    }
    fn store(&self) -> PathBuf {
        self.path.join("store")
    }
    fn db(&self) -> PathBuf {
        self.store()
            .join("generations")
            .join(GEN)
            .join("ledger.sqlite3")
    }
    fn connection(&self) -> Connection {
        let db = Connection::open_with_flags(
            self.db(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        db.execute_batch("PRAGMA query_only=ON").unwrap();
        db
    }
    /// Independent count of journal rows, read with a separate read-only connection.
    fn events(&self) -> u64 {
        let db = self.connection();
        let count: i64 = db
            .query_row("SELECT count(*) FROM events", [], |r| r.get(0))
            .unwrap();
        db.close().unwrap();
        u64::try_from(count).unwrap()
    }
    fn attempt_row(&self) -> (String, String, String) {
        let db = self.connection();
        let row = db
            .query_row(
                "SELECT state,effect,cleanup FROM attempts WHERE id=?",
                [ATTEMPT],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        db.close().unwrap();
        row
    }
    fn task_row(&self) -> (String, bool, Option<String>) {
        let db = self.connection();
        let row = db
            .query_row(
                "SELECT state,cancellation,accepted_event FROM tasks WHERE id=?",
                [TASK],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        db.close().unwrap();
        row
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).unwrap();
    }
}

struct Rig {
    area: Area,
    store: Option<Store>,
    evidence: Object,
}
impl Rig {
    fn admitted() -> Self {
        let area = Area::new("rig");
        DirBuilder::new().mode(0o700).create(area.store()).unwrap();
        let mut store = Store::open(&area.store(), id(GEN), id(EPOCH), true, deadline()).unwrap();
        store
            .submit(
                Submission {
                    principal: &principal(),
                    key: id(KEY),
                    task: id(TASK),
                    event: id(ADMIT),
                    request_bytes: b"startup reconciliation fixture",
                    criteria: Sha256Digest::parse(DIGEST).unwrap(),
                    allocation: Allocation {
                        limit_ms: 1_200_000,
                        work_ms: 900_000,
                        verify_ms: 300_000,
                    },
                },
                deadline(),
            )
            .unwrap();
        let evidence = store
            .publish(b"retained fixture evidence", id(STAGE), deadline())
            .unwrap();
        Self {
            area,
            store: Some(store),
            evidence,
        }
    }
    fn store(&mut self) -> &mut Store {
        self.store.as_mut().unwrap()
    }
    fn revision(&mut self) -> String {
        self.store()
            .get(&principal(), id(TASK), deadline())
            .unwrap()
            .generation
    }
    /// A plain running attempt: no roster pin, so no process identity.
    fn running() -> Self {
        let mut r = Self::admitted();
        r.store()
            .begin_attempt(
                id(TASK),
                generation("1"),
                id(ATTEMPT),
                id(START),
                deadline(),
            )
            .unwrap();
        r
    }
    /// A rostered running attempt whose pinned observation carries `identity`
    /// as its `actual_identity`, leased for `lease_ms`.
    fn rostered(identity: &str, lease_ms: u64) -> Self {
        let mut r = Self::admitted();
        let input = Update {
            idempotency_key: OTHER.into(),
            record_id: None,
            expected_revision: None,
            definition: RosterDefinitionV1 {
                kind: Kind::Agent,
                display_name: "startup-fixture".into(),
                owner_id: "fixture-worker".into(),
                version: "v1".into(),
                capabilities: vec!["text".into()],
                locality: Locality::Local,
                endpoint_ref: Some(OTHER.into()),
                limitations: "test-only literal local adapter; no runtime grant".into(),
            },
            audit_reason: "startup reconciliation fixture".into(),
        };
        let raw = serde_json::to_vec(&json!({"protocol":"hee3.control","version":1,"kind":"request","request_id":OTHER,"action":"roster.update","action_version":1,"idempotency_key":OTHER,"deadline_unix_ms":"1030000","authority":{"grant_id":OTHER,"scope_sha256":DIGEST},"precondition":null,"body":{"record_id":null,"definition":input.definition,"audit_reason":input.audit_reason}})).unwrap();
        let head = r
            .store()
            .roster_apply(
                &principal(),
                &[input],
                RequestSource::Native(&raw),
                deadline(),
            )
            .unwrap()
            .remove(0)
            .head;
        r.store()
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
                    actual_identity: Some(identity.into()),
                    immutable_revision: None,
                    capabilities: vec!["text".into()],
                    evidence_ref: OTHER.into(),
                },
                deadline(),
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
        r.store()
            .begin_rostered_attempt(
                RosterStart {
                    principal: &principal(),
                    task: id(TASK),
                    expected: generation("1"),
                    attempt: id(ATTEMPT),
                    event: id(START),
                    agent_record_id: &head.record_id,
                    session: id(KEY),
                    workspace: id(STAGE),
                    selections: &selections,
                    lease_ms,
                },
                deadline(),
            )
            .unwrap();
        r
    }
    fn expected(&mut self) -> Expected<'static> {
        Expected {
            task: id(TASK),
            task_generation: generation(&self.revision()),
            attempt: id(ATTEMPT),
            attempt_generation: generation("1"),
        }
    }
    fn settle(&mut self, effect: Effect, used_ms: Option<u64>, cleanup_settled: bool) {
        let expected = self.expected();
        self.store()
            .settle_attempt(
                &expected,
                Settlement {
                    effect,
                    used_ms,
                    cleanup_settled,
                    ready_to_verify: true,
                },
                id(SETTLE),
                deadline(),
            )
            .unwrap();
    }
    /// Worker returned cleanly: attempt settled, task verifying.
    fn ready() -> Self {
        let mut r = Self::running();
        r.settle(Effect::None, Some(47_977), true);
        r
    }
    fn verify(&mut self, verdict: VerificationVerdict, cleanup_settled: bool) {
        let e = self.expected();
        let evidence = self.evidence.clone();
        self.store()
            .record_verification(
                &e,
                &habitat_engine::store::Verification {
                    verdict,
                    subject: Sha256Digest::parse(DIGEST).unwrap(),
                    evidence,
                    used_ms: Some(20),
                    cleanup_settled,
                },
                id(CHECK),
                deadline(),
            )
            .unwrap();
    }
    fn accept(&mut self) {
        let e = self.expected();
        let evidence = self.evidence.clone();
        let publication = self
            .store()
            .prepare_verified_acceptance(
                &e,
                id(ACCEPT),
                Sha256Digest::parse(DIGEST).unwrap(),
                &evidence,
                std::slice::from_ref(&evidence),
                deadline(),
            )
            .unwrap();
        self.store().accept(&publication, 0, deadline()).unwrap();
    }
    fn cancel(&mut self) {
        let revision = self.revision();
        self.store()
            .cancel(id(TASK), generation(&revision), id(CANCEL), deadline())
            .unwrap();
    }
    fn stop(&mut self) {
        let revision = self.revision();
        let evidence = self.evidence.clone();
        let reason = Name::new("fixture-stop").unwrap();
        self.store()
            .finish_unaccepted(
                &principal(),
                Stop {
                    task: id(TASK),
                    generation: generation(&revision),
                    reason: &reason,
                    evidence: &evidence,
                    event: id(STOP),
                },
                deadline(),
            )
            .unwrap();
    }
    fn close(&mut self) {
        drop(self.store.take());
    }
    fn inventory(&mut self) -> RecoveryInventory {
        self.close();
        let mut store =
            Store::open_inspection(&self.area.store(), id(GEN), id(EPOCH), deadline()).unwrap();
        store
            .recovery_inventory(id(EPOCH), limits(), deadline())
            .unwrap()
    }
    fn ordinals(&mut self) -> habitat_engine::store::TerminalOrdinals {
        self.close();
        let store =
            Store::open_inspection(&self.area.store(), id(GEN), id(EPOCH), deadline()).unwrap();
        store.terminal_ordinals(id(TASK), deadline()).unwrap()
    }
    fn records(&mut self) -> Vec<RecordRow> {
        self.close();
        let store =
            Store::open_inspection(&self.area.store(), id(GEN), id(EPOCH), deadline()).unwrap();
        store
            .reconciliation_records(id(ATTEMPT), deadline())
            .unwrap()
    }
    fn pass_with(
        &mut self,
        world: &mut dyn Physical,
        restored_from: Option<&str>,
        cursors: &[Cursor],
        claims: &[Claim],
    ) -> Pass {
        self.close();
        startup::run(
            &Startup {
                root: &self.area.store(),
                generation: id(GEN),
                epoch: id(EPOCH),
                limits: limits(),
                restored_from,
                cursors,
                claims,
                deadline: deadline(),
            },
            world,
        )
        .unwrap()
    }
    fn pass(&mut self, world: &mut dyn Physical) -> Pass {
        self.pass_with(world, None, &[], &[])
    }
    /// Delete the published evidence object's bytes: the test-only way to make an
    /// evidence readback report `Absent` for a verification that names it.
    fn remove_evidence_object(&mut self) {
        self.close();
        let hex = &self.evidence.digest()["sha256:".len()..];
        let path = self
            .area
            .store()
            .join("generations")
            .join(GEN)
            .join("objects")
            .join("sha256")
            .join(&hex[..2])
            .join(hex);
        fs::remove_file(path).unwrap();
    }
}

fn subject_value(generation: u64, rostered: bool) -> SubjectValue {
    SubjectValue {
        task: TASK.into(),
        attempt: ATTEMPT.into(),
        generation,
        workspace_ref: rostered.then(|| STAGE.to_owned()),
        session: rostered.then(|| KEY.to_owned()),
    }
}

fn only(pass: &Pass) -> &Entry {
    assert_eq!(pass.attempts.len(), 1, "{pass:#?}");
    assert_eq!(pass.ledger, LedgerAccess::Writable);
    assert_eq!(pass.mode, Mode::Normal);
    assert!(!pass.permits_execution);
    &pass.attempts[0]
}

fn body(row: &RecordRow) -> Value {
    serde_json::from_slice(&row.body).unwrap()
}

/// The four readbacks were asked about exactly this subject, once each.
fn readbacks_asked(world: &World, subject: &SubjectValue) {
    for (name, call) in [
        ("pi_queue", Call::PiQueue(subject.clone())),
        ("cleanup", Call::Cleanup(subject.clone())),
        ("workspace", Call::Workspace(subject.clone())),
        ("acknowledgement", Call::Acknowledgement(subject.clone())),
    ] {
        assert_eq!(
            world.calls_of(name),
            vec![&call],
            "{name}: {:?}",
            world.calls
        );
    }
    assert_eq!(world.calls_of("clock").len(), 1);
}

fn retained(entry: &Entry, rule: Rule, reason: &Unknown, custody: &ProcessCustody) {
    assert_eq!(entry.decision.rule, rule, "{entry:#?}");
    assert_eq!(entry.rule, rule.id());
    match &entry.decision.reconciliation {
        Reconciliation::RetainUnknown {
            reason: actual,
            process,
            ..
        } => {
            assert_eq!(actual, reason);
            assert_eq!(process, custody);
        }
        other => panic!("not retained: {other:?}"),
    }
    assert_eq!(entry.action, Some(Action::RetainedUnknown));
    assert_eq!(entry.records.len(), 1);
    assert!(entry.records[0].fresh);
}

// ---- enumeration, records and idempotency ---------------------------------------------------

/// `T07-AP-01` · an admitted ledger with no attempt: the pass opens both ways,
/// reports the ledger writable and decides nothing.
#[test]
fn empty_ledger_yields_no_entries_and_no_writes() {
    let mut r = Rig::admitted();
    let before = r.area.events();
    let mut world = World::new();
    let pass = r.pass(&mut world);
    assert_eq!(
        (pass.epoch.as_str(), pass.generation.as_str()),
        (EPOCH, GEN)
    );
    assert_eq!(pass.mode, Mode::Normal);
    assert_eq!(pass.ledger, LedgerAccess::Writable);
    assert_eq!(pass.event_high_water, 1);
    assert!(pass.attempts.is_empty() && pass.cursors.is_empty());
    assert_eq!(pass.writes, 0);
    assert!(!pass.permits_execution);
    assert_eq!(r.area.events(), before);
    assert_eq!(world.calls, vec![Call::Clock]);
}

/// `T07-AP-02` · a running attempt whose observation is not a process identity:
/// the seam is never asked about a process, the other four questions name the
/// attempt, and R07 retains it unknown as unobserved with one journal row.
#[test]
fn running_attempt_without_identity_is_retained_unobserved() {
    let mut r = Rig::running();
    let before = r.area.events();
    let mut world = World::new();
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(world.calls_of("process"), Vec::<&Call>::new());
    readbacks_asked(&world, &subject_value(1, false));
    assert_eq!(entry.handed.identity, None);
    assert_eq!(entry.handed.process, ProcessCustody::Unobserved);
    assert_eq!(entry.handed.pi_queue, PiQueueCustody::Unreconciled);
    assert_eq!(entry.handed.cleanup, CleanupReadback::NotRead);
    assert_eq!(entry.handed.workspace, WorkspaceReadback::NotRead);
    assert_eq!(entry.handed.acknowledgement, Acknowledgement::Unrecorded);
    assert_eq!(entry.handed.attempt_state, AttemptState::Running);
    assert_eq!(entry.handed.task_state, TaskState::Running);
    retained(
        entry,
        Rule::R07ProcessNotOurs,
        &Unknown::ProcessUnobserved,
        &ProcessCustody::Unobserved,
    );
    assert_eq!(pass.writes, 1);
    assert_eq!(r.area.events(), before + 1);
    let rows = r.records();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind, RecordKind::Decided);
    assert_eq!(rows[0].sequence, entry.records[0].sequence);
    assert_eq!(rows[0].event, entry.records[0].event);
    assert_eq!(rows[0].generation, "2");
    let b = body(&rows[0]);
    assert_eq!(b["kind"], "hee3-reconciliation-decided/1");
    assert_eq!(b["attempt"], ATTEMPT);
    assert_eq!(b["task"], TASK);
    assert_eq!(b["rule"], "R07");
    assert_eq!(b["decision"]["decision"], "retain_unknown");
    assert_eq!(b["decision"]["reason"]["reason"], "process_unobserved");
    assert_eq!(b["intent"], "retained_unknown");
    assert_eq!(b["handed"]["process"]["custody"], "unobserved");
    assert_eq!(r.area.attempt_row().0, "running");
}

/// `T07-AP-03` · duplicate-write proof: the same world twice yields one set of
/// rows, by an independent row count and by the receipt's ordinals.
#[test]
fn second_pass_over_the_same_world_writes_nothing() {
    let mut r = Rig::running();
    let mut world = World::new();
    let first = r.pass(&mut world);
    let rows = r.area.events();
    let second = r.pass(&mut world);
    assert_eq!(first.writes, 1);
    assert_eq!(second.writes, 0);
    assert_eq!(r.area.events(), rows);
    assert_eq!(second.attempts[0].records.len(), 1);
    assert!(!second.attempts[0].records[0].fresh);
    assert_eq!(
        second.attempts[0].records[0].sequence,
        first.attempts[0].records[0].sequence
    );
    assert_eq!(
        second.attempts[0].records[0].event,
        first.attempts[0].records[0].event
    );
    assert_eq!(second.attempts[0].decision, first.attempts[0].decision);
    assert_eq!(r.records().len(), 1);
}

// ---- live worker: reattach, never redispatch -------------------------------------------------

/// `T07-AP-04` · same identity alive, Pi not applicable: R06 reattaches
/// observation only; the seam is handed the pinned identity, attach is called
/// with it, the readback after attach is live, two rows are written.
#[test]
fn live_same_identity_reattaches_observation_without_redispatch() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let before = r.area.events();
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::NotApplicable);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    let subject = subject_value(1, true);
    assert_eq!(
        world.calls_of("process"),
        vec![
            &Call::Process {
                subject: subject.clone(),
                identity: identity()
            },
            &Call::Process {
                subject: subject.clone(),
                identity: identity()
            }
        ],
        "one read before the decision, one readback after the effect"
    );
    assert_eq!(
        world.calls_of("attach"),
        vec![&Call::Attach {
            subject: subject.clone(),
            identity: identity()
        }]
    );
    readbacks_asked(&world, &subject);
    assert_eq!(entry.handed.identity, Some(identity()));
    assert_eq!(entry.handed.process, ProcessCustody::LiveSameIdentity);
    assert_eq!(entry.decision.rule, Rule::R06LiveOwnedChild);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::ReattachObservationOnly {
            generation: 1,
            process: ProcessCustody::LiveSameIdentity,
            pi_queue: PiQueueCustody::NotApplicable,
            cancellation_pending: false,
            redispatch: false,
        }
    );
    assert_eq!(
        entry.action,
        Some(Action::ObservationAttached {
            generation: 1,
            readback: ProcessCustody::LiveSameIdentity
        })
    );
    assert_eq!(entry.records.len(), 2);
    assert!(entry.records.iter().all(|record| record.fresh));
    assert_eq!(entry.records[1].sequence, entry.records[0].sequence + 1);
    assert_eq!(pass.writes, 2);
    assert_eq!(r.area.events(), before + 2);
    let rows = r.records();
    assert_eq!(
        rows.iter().map(|row| row.kind).collect::<Vec<_>>(),
        vec![RecordKind::Decided, RecordKind::Readback]
    );
    let readback = body(&rows[1]);
    assert_eq!(readback["effect"], "observation_attached");
    assert_eq!(readback["readback"]["custody"], "live_same_identity");
    assert_eq!(body(&rows[0])["decision"]["redispatch"], false);
    assert_eq!(
        r.area.attempt_row(),
        ("running".into(), "pending".into(), "pending".into())
    );
}

/// `T07-AP-05` · reattach is idempotent: the second pass attaches again (an
/// observation, not a dispatch) and writes nothing new.
#[test]
fn second_reattach_pass_reattaches_and_writes_nothing() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let first = r.pass(&mut world);
    let rows = r.area.events();
    let second = r.pass(&mut world);
    assert_eq!(first.writes, 2);
    assert_eq!(second.writes, 0);
    assert_eq!(r.area.events(), rows);
    assert_eq!(world.calls_of("attach").len(), 2);
    assert_eq!(
        second.attempts[0]
            .records
            .iter()
            .map(|record| (record.fresh, record.sequence))
            .collect::<Vec<_>>(),
        first.attempts[0]
            .records
            .iter()
            .map(|record| (false, record.sequence))
            .collect::<Vec<_>>()
    );
}

/// `T07-AP-06` · a live child with queued Pi messages is not positively
/// reconciled: R06 retains it with the occupied queue; nothing is attached.
#[test]
fn queued_pi_messages_block_reattach() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Queued {
            steering: 2,
            follow_up: 1,
        });
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    retained(
        entry,
        Rule::R06LiveOwnedChild,
        &Unknown::PiQueueOccupied {
            steering: 2,
            follow_up: 1,
        },
        &ProcessCustody::LiveSameIdentity,
    );
    assert!(world.calls_of("attach").is_empty());
    assert_eq!(world.calls_of("process").len(), 1);
    assert_eq!(pass.writes, 1);
}

/// `T07-AP-07` · an unreconciled Pi queue keeps a live child unknown.
#[test]
fn unreconciled_pi_queue_blocks_reattach() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new().with_process(PID, present(START_TICKS, NAMESPACE));
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R06LiveOwnedChild,
        &Unknown::PiQueueUnreconciled,
        &ProcessCustody::LiveSameIdentity,
    );
    assert!(world.calls_of("attach").is_empty());
}

/// `T07-AP-08` · a pending clear keeps a live child unknown, naming the command.
#[test]
fn pending_pi_clear_blocks_reattach_and_names_the_command() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::ClearPending {
            command: "clear-7".into(),
        });
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R06LiveOwnedChild,
        &Unknown::PiClearPending {
            command: "clear-7".into(),
        },
        &ProcessCustody::LiveSameIdentity,
    );
    assert!(world.calls_of("attach").is_empty());
    assert_eq!(
        body(&r.records()[0])["decision"]["reason"]["command"],
        "clear-7"
    );
}

/// `T07-AP-09` · an idle Pi queue lets the live child be reattached.
#[test]
fn idle_pi_queue_permits_reattach() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert!(matches!(
        entry.decision.reconciliation,
        Reconciliation::ReattachObservationOnly {
            pi_queue: PiQueueCustody::Idle,
            redispatch: false,
            ..
        }
    ));
    assert_eq!(world.calls_of("attach").len(), 1);
}

/// `T07-AP-10` · the attach effect refused by the world: the decision stands
/// recorded, no readback row is written, the refusal is carried verbatim.
#[test]
fn refused_attach_records_the_decision_and_the_refusal_without_a_readback() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle)
        .refusing_attach("pidfd_open: EPERM");
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R06LiveOwnedChild);
    assert_eq!(
        entry.action,
        Some(Action::AttachRefused {
            generation: 1,
            error: "pidfd_open: EPERM".into()
        })
    );
    assert_eq!(entry.records.len(), 1);
    assert_eq!(
        world.calls_of("process").len(),
        1,
        "no readback without an effect"
    );
    assert_eq!(pass.writes, 1);
    assert_eq!(r.records().len(), 1);
}

// ---- PID reuse and lost workers ---------------------------------------------------------------

/// `T07-AP-11` · the PID lives on with later start ticks: R07 retains it as
/// identity reused on `start_ticks`; the double was handed the retained PID.
#[test]
fn pid_reused_on_start_ticks_is_retained_and_never_attached() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS + 22, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(
        world.calls_of("process"),
        vec![&Call::Process {
            subject: subject_value(1, true),
            identity: identity()
        }]
    );
    retained(
        entry,
        Rule::R07ProcessNotOurs,
        &Unknown::ProcessIdentityReused {
            differs: vec![Dimension::StartTicks],
        },
        &ProcessCustody::PidReused {
            differs: vec![Dimension::StartTicks],
        },
    );
    assert!(world.calls_of("attach").is_empty());
    let b = body(&r.records()[0]);
    assert_eq!(b["decision"]["reason"]["differs"], json!(["start_ticks"]));
    assert_eq!(b["handed"]["identity"]["pid"], PID);
}

/// `T07-AP-12` · the same PID in another namespace is reused on `namespace`.
#[test]
fn pid_reused_on_namespace_is_retained() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new().with_process(PID, present(START_TICKS, "pid:[4026531836]"));
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R07ProcessNotOurs,
        &Unknown::ProcessIdentityReused {
            differs: vec![Dimension::Namespace],
        },
        &ProcessCustody::PidReused {
            differs: vec![Dimension::Namespace],
        },
    );
}

/// `T07-AP-13` · an unreadable `/proc` entry keeps its text and is not ours.
#[test]
fn unreadable_process_is_retained_with_the_error_text() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new().with_process(
        PID,
        LiveRead::Unreadable("stat: Permission denied (os error 13)".into()),
    );
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R07ProcessNotOurs,
        &Unknown::ProcessUnreadable {
            error: "stat: Permission denied (os error 13)".into(),
        },
        &ProcessCustody::Unreadable {
            error: "stat: Permission denied (os error 13)".into(),
        },
    );
    assert_eq!(
        body(&r.records()[0])["handed"]["process"]["error"],
        "stat: Permission denied (os error 13)"
    );
}

/// `T07-AP-14` · crash before the correlated acknowledgement: worker absent,
/// acknowledgement not seen → R08 dispatch unacknowledged; no effect.
#[test]
fn absent_worker_before_acknowledgement_is_dispatch_unacknowledged() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new().with_acknowledgement(Acknowledgement::NotSeen);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.handed.acknowledgement, Acknowledgement::NotSeen);
    retained(
        entry,
        Rule::R08WorkerAbsent,
        &Unknown::DispatchUnacknowledged,
        &ProcessCustody::Absent,
    );
    assert!(world.calls_of("attach").is_empty() && world.calls_of("clean").is_empty());
}

/// `T07-AP-15` · crash after the correlated acknowledgement → acknowledged worker lost.
#[test]
fn absent_worker_after_acknowledgement_is_acknowledged_worker_lost() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world =
        World::new().with_acknowledgement(Acknowledgement::Correlated { generation: 1 });
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R08WorkerAbsent,
        &Unknown::AcknowledgedWorkerLost { generation: 1 },
        &ProcessCustody::Absent,
    );
    assert_eq!(
        body(&r.records()[0])["handed"]["acknowledgement"],
        json!({"correlated": {"generation": 1}})
    );
}

/// `T07-AP-16` · no acknowledgement record at all → unrecorded, still unknown.
#[test]
fn absent_worker_without_acknowledgement_record_is_unrecorded() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new();
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R08WorkerAbsent,
        &Unknown::AcknowledgementUnrecorded,
        &ProcessCustody::Absent,
    );
}

// ---- workspace reuse: lease expiry alone never licenses it -----------------------------------

fn reuse_refused(entry: &Entry, reason: &ReuseRefusal) {
    assert_eq!(entry.decision.rule, Rule::R09WorkspaceReuse, "{entry:#?}");
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::WorkspaceReuseRefused {
            reason: reason.clone(),
            process: ProcessCustody::Absent,
        }
    );
    assert_eq!(
        entry.action,
        Some(Action::ReuseRefused {
            reason: reason.clone()
        })
    );
    assert_eq!(entry.records.len(), 1);
}

/// `T07-AP-17` · a literal (non-process) identity is unobserved, and R07 decides
/// before any workspace question: the writable workspace is read and carried but
/// never becomes a reuse decision.
#[test]
fn literal_identity_is_unobserved_before_the_workspace_is_considered() {
    let mut r = Rig::rostered("literal/fixture", 60_000);
    let mut world = World::new().with_workspace(WorkspaceReadback::Writable { bytes: 512 });
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert!(world.calls_of("process").is_empty());
    retained(
        entry,
        Rule::R07ProcessNotOurs,
        &Unknown::ProcessUnobserved,
        &ProcessCustody::Unobserved,
    );
    assert_eq!(
        entry.handed.workspace,
        WorkspaceReadback::Writable { bytes: 512 }
    );
    assert_eq!(world.calls_of("workspace").len(), 1);
}

/// `T07-AP-18` · absent worker, writable workspace, lease held by the roster,
/// no clock at startup → refused as clock unavailable, by rule R09.
#[test]
fn leased_writable_workspace_without_clock_is_refused_clock_unavailable() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new().with_workspace(WorkspaceReadback::Writable { bytes: 4096 });
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    reuse_refused(entry, &ReuseRefusal::ClockUnavailable);
    assert_eq!(entry.handed.clock_epoch, None);
    assert_eq!(
        body(&r.records()[0])["decision"]["reason"]["reason"],
        "clock_unavailable"
    );
}

/// `T07-AP-19` · a clock from another receiver epoch is not comparable to the lease.
#[test]
fn lease_from_another_receiver_epoch_is_not_comparable() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let lease_epoch = r.inventory().instances[0].started.epoch.clone();
    let mut world = World::new()
        .with_workspace(WorkspaceReadback::Writable { bytes: 1 })
        .with_clock(OTHER, 5);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    reuse_refused(
        entry,
        &ReuseRefusal::LeaseClockNotComparable {
            lease_epoch: lease_epoch.clone(),
            clock_epoch: OTHER.into(),
        },
    );
    assert_eq!(entry.handed.clock_epoch.as_deref(), Some(OTHER));
    assert_ne!(lease_epoch, OTHER);
}

/// `T07-AP-20` · a comparable clock before expiry: the lease is held.
#[test]
fn lease_held_in_its_own_epoch_is_refused_with_the_remaining_time() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let instance = r.inventory().instances[0].clone();
    let mut world = World::new()
        .with_workspace(WorkspaceReadback::Writable { bytes: 1 })
        .with_clock(
            &instance.started.epoch,
            instance.lease_expires_monotonic_ms - 250,
        );
    let pass = r.pass(&mut world);
    reuse_refused(only(&pass), &ReuseRefusal::LeaseHeld { remaining_ms: 250 });
}

/// `T07-AP-21` · lease expiry alone cannot authorize reuse of a still-writable workspace.
#[test]
fn expired_lease_over_writable_workspace_is_still_refused() {
    let mut r = Rig::rostered(IDENTITY, 1);
    let instance = r.inventory().instances[0].clone();
    let mut world = World::new()
        .with_workspace(WorkspaceReadback::Writable { bytes: 77 })
        .with_clock(
            &instance.started.epoch,
            instance.lease_expires_monotonic_ms + 900,
        );
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    reuse_refused(
        entry,
        &ReuseRefusal::LeaseExpiredWritable {
            expired_by_ms: 900,
            bytes: 77,
        },
    );
    assert!(
        world.calls_of("clean").is_empty(),
        "no cleanup effect on a refused reuse"
    );
    assert_eq!(r.area.attempt_row().0, "running");
}

/// `T07-AP-22` · the same absent worker with a released workspace is not a reuse
/// question at all: R08 decides on the acknowledgement class.
#[test]
fn released_workspace_of_absent_worker_falls_to_the_acknowledgement_rule() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_workspace(WorkspaceReadback::Released)
        .with_acknowledgement(Acknowledgement::NotSeen);
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R08WorkerAbsent,
        &Unknown::DispatchUnacknowledged,
        &ProcessCustody::Absent,
    );
}

// ---- retained claims: stale epochs and generations ------------------------------------------

fn claim(epoch: &str, task_generation: u64, attempt_generation: u64) -> Claim {
    Claim {
        attempt: ATTEMPT.into(),
        epoch: epoch.into(),
        task_generation,
        attempt_generation,
    }
}

/// `T07-AP-23` · a retained claim from another ledger epoch is refused before any
/// physical rule, even for a live child; nothing is attached.
#[test]
fn claim_from_another_epoch_is_refused_before_the_live_child_is_considered() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let pass = r.pass_with(&mut world, None, &[], &[claim(OTHER, 2, 1)]);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R01StaleObservationEpoch);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::StaleObservationRefused {
            observed_epoch: OTHER.into(),
            ledger_epoch: EPOCH.into(),
        }
    );
    assert_eq!(entry.action, Some(Action::StaleRefused));
    assert_eq!(entry.handed.claim, Some(claim(OTHER, 2, 1)));
    assert!(world.calls_of("attach").is_empty());
    assert_eq!(pass.writes, 1);
    assert_eq!(body(&r.records()[0])["rule"], "R01");
}

/// `T07-AP-24` · a claim naming an older task generation is stale.
#[test]
fn claim_with_stale_task_generation_is_refused() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new().with_process(PID, present(START_TICKS, NAMESPACE));
    let pass = r.pass_with(&mut world, None, &[], &[claim(EPOCH, 1, 1)]);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R02StaleGeneration);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::StaleGenerationRefused {
            subject: GenerationSubject::Task,
            claimed: 1,
            current: 2,
        }
    );
    assert_eq!(entry.action, Some(Action::StaleRefused));
}

/// `T07-AP-25` · the attempt generation is checked separately from the task's.
#[test]
fn claim_with_stale_attempt_generation_is_refused() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new();
    let pass = r.pass_with(&mut world, None, &[], &[claim(EPOCH, 2, 3)]);
    assert_eq!(
        only(&pass).decision.reconciliation,
        Reconciliation::StaleGenerationRefused {
            subject: GenerationSubject::Attempt,
            claimed: 3,
            current: 1,
        }
    );
}

/// `T07-AP-26` · a current claim passes through to the physical rules; a claim
/// for another attempt is not this attempt's.
#[test]
fn current_claim_passes_through_and_foreign_claims_are_ignored() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let foreign = Claim {
        attempt: OTHER.into(),
        epoch: OTHER.into(),
        task_generation: 9,
        attempt_generation: 9,
    };
    let pass = r.pass_with(&mut world, None, &[], &[foreign, claim(EPOCH, 2, 1)]);
    let entry = only(&pass);
    assert_eq!(entry.handed.claim, Some(claim(EPOCH, 2, 1)));
    assert_eq!(entry.decision.rule, Rule::R06LiveOwnedChild);
    assert_eq!(world.calls_of("attach").len(), 1);
}

// ---- settled workers: cleanup by readback, intent before effect -------------------------------

/// `T07-AP-27` · a settled worker whose cleanup cannot be read back stays unverified.
#[test]
fn settled_worker_with_unread_cleanup_is_retained_unverified() {
    let mut r = Rig::ready();
    let mut world = World::new();
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.handed.attempt_state, AttemptState::Settled);
    assert_eq!(entry.handed.task_state, TaskState::Verifying);
    retained(
        entry,
        Rule::R11CleanupReadback,
        &Unknown::CleanupUnverified,
        &ProcessCustody::Unobserved,
    );
    assert!(world.calls_of("clean").is_empty());
}

/// `T07-AP-28` · partial cleanup: the decision (the intent, naming the target)
/// is journaled before the effect; the effect is performed on exactly that
/// target; the readback after it is complete; both rows carry their ordinals.
#[test]
fn partial_cleanup_records_intent_performs_the_target_and_reads_back_complete() {
    let mut r = Rig::ready();
    let before = r.area.events();
    let mut world = World::new().with_obligations(&["workspace"]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    let subject = subject_value(1, false);
    assert_eq!(
        entry.handed.cleanup,
        CleanupReadback::Partial {
            remaining: vec!["workspace".into()]
        }
    );
    assert_eq!(entry.decision.rule, Rule::R11CleanupReadback);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::CleanupCandidate {
            what: vec![CleanupTarget::Remaining {
                name: "workspace".into()
            }],
            process: ProcessCustody::Unobserved,
        }
    );
    assert_eq!(
        world.calls_of("clean"),
        vec![&Call::Clean {
            subject: subject.clone(),
            target: "workspace".into()
        }]
    );
    assert_eq!(
        world.calls_of("cleanup").len(),
        2,
        "one read before the decision, one readback after the effect"
    );
    assert_eq!(world.obligations[ATTEMPT], Vec::<String>::new());
    assert_eq!(
        entry.action,
        Some(Action::CleanupPerformed {
            targets: vec![CleanupTarget::Remaining {
                name: "workspace".into()
            }],
            effects: vec![startup::CleanupEffect {
                target: "workspace".into(),
                performed: true,
                error: None
            }],
            readback: CleanupReadback::Complete,
            ledger_settled: false,
            ledger_refusal: None,
        })
    );
    assert_eq!(entry.records.len(), 2);
    assert!(
        entry.records[0].sequence < entry.records[1].sequence,
        "intent precedes readback"
    );
    assert_eq!(r.area.events(), before + 2);
    let rows = r.records();
    assert_eq!(body(&rows[0])["intent"], "cleanup_performed");
    let readback = body(&rows[1]);
    assert_eq!(readback["effect"], "cleanup");
    assert_eq!(readback["readback"]["cleanup_readback"], "complete");
    assert_eq!(readback["settle_cleanup"], false);
    assert_eq!(
        r.area.attempt_row(),
        ("settled".into(), "none".into(), "settled".into())
    );
}

/// `T07-AP-29` · the world refuses the effect: the readback stays partial and the
/// refusal is recorded with the target it named; nothing is settled.
#[test]
fn refused_cleanup_effect_is_recorded_with_a_partial_readback() {
    let mut r = Rig::ready();
    let mut world = World::new()
        .with_obligations(&["workspace"])
        .refusing_clean("workspace");
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(
        entry.action,
        Some(Action::CleanupPerformed {
            targets: vec![CleanupTarget::Remaining {
                name: "workspace".into()
            }],
            effects: vec![startup::CleanupEffect {
                target: "workspace".into(),
                performed: false,
                error: Some("refused: workspace".into())
            }],
            readback: CleanupReadback::Partial {
                remaining: vec!["workspace".into()]
            },
            ledger_settled: false,
            ledger_refusal: None,
        })
    );
    assert_eq!(world.obligations[ATTEMPT], vec!["workspace".to_owned()]);
    let readback = body(&r.records()[1]);
    assert_eq!(readback["effects"][0]["error"], "refused: workspace");
    assert_eq!(readback["readback"]["remaining"], json!(["workspace"]));
}

/// `T07-AP-30` · an `unknown` attempt whose cleanup reads back complete: the
/// ledger settlement target is performed in the readback row's transaction and
/// the column reads back `settled`.
#[test]
fn complete_readback_settles_the_ledger_cleanup_column_with_the_readback_row() {
    let mut r = Rig::running();
    r.settle(Effect::Committed, Some(100), false);
    assert_eq!(
        r.area.attempt_row(),
        ("unknown".into(), "committed".into(), "unknown".into())
    );
    let mut world = World::new().with_obligations(&[]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::CleanupCandidate {
            what: vec![CleanupTarget::LedgerSettlement {
                cleanup: Cleanup::Unknown
            }],
            process: ProcessCustody::Unobserved,
        }
    );
    assert!(
        world.calls_of("clean").is_empty(),
        "a ledger target is not a world effect"
    );
    assert_eq!(
        entry.action,
        Some(Action::CleanupPerformed {
            targets: vec![CleanupTarget::LedgerSettlement {
                cleanup: Cleanup::Unknown
            }],
            effects: vec![],
            readback: CleanupReadback::Complete,
            ledger_settled: true,
            ledger_refusal: None,
        })
    );
    assert!(entry.records[1].cleanup_settled);
    assert_eq!(
        r.area.attempt_row(),
        ("unknown".into(), "committed".into(), "settled".into())
    );
    assert_eq!(body(&r.records()[1])["settle_cleanup"], true);
}

/// `T07-AP-30b` · the settlement races a second writer: between the readback and its row,
/// the attempt's cleanup column is settled elsewhere. The store refuses the settlement as a
/// conflict, and the pass records the readback WITHOUT it and names the refusal, rather
/// than failing the pass or claiming a settlement it did not make. This arm was reachable
/// only by a concurrent writer, so no case had reached it; mutation shard C found its
/// guard free to flip either way.
#[test]
fn a_settlement_lost_to_a_concurrent_writer_is_recorded_as_refused() {
    let mut r = Rig::running();
    r.settle(Effect::Committed, Some(100), false);
    let mut world = World::new().with_obligations(&[]);
    world.concurrent_settle = Some(r.area.db());
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(
        entry.action,
        Some(Action::CleanupPerformed {
            targets: vec![CleanupTarget::LedgerSettlement {
                cleanup: Cleanup::Unknown
            }],
            effects: vec![],
            readback: CleanupReadback::Complete,
            ledger_settled: false,
            ledger_refusal: Some("conflict".into()),
        })
    );
    assert!(!entry.records[1].cleanup_settled);
    let readback = body(&r.records()[1]);
    assert_eq!(readback["settle_cleanup"], false);
    assert_eq!(readback["settle_refused"], "conflict");
    assert_eq!(
        r.area.attempt_row(),
        ("unknown".into(), "committed".into(), "settled".into()),
        "the column is the other writer's, not this pass's"
    );
}

/// `T07-AP-31` · after the settlement the world is different, so the next pass
/// decides differently once (the task stays `effect_unknown`, explicit) and the
/// pass after that writes nothing.
#[test]
fn passes_after_a_ledger_effect_converge_to_no_writes() {
    let mut r = Rig::running();
    r.settle(Effect::Committed, Some(100), false);
    let mut world = World::new().with_obligations(&[]);
    let first = r.pass(&mut world);
    let second = r.pass(&mut world);
    let rows = r.area.events();
    let third = r.pass(&mut world);
    assert_eq!(first.writes, 2);
    assert_eq!(second.writes, 1);
    assert_eq!(third.writes, 0);
    assert_eq!(r.area.events(), rows);
    retained(
        &second.attempts[0],
        Rule::R14UnexpectedState,
        &Unknown::TaskStateUnexpected {
            state: TaskState::EffectUnknown,
        },
        &ProcessCustody::Unobserved,
    );
    assert_eq!(third.attempts[0].decision, second.attempts[0].decision);
    assert_eq!(r.records().len(), 3);
}

/// `T07-AP-32` · two remaining obligations are each performed once and read
/// back complete; the ledger's own settlement is a target only when the
/// readback before the decision was complete, so it follows on the next pass.
#[test]
fn remaining_obligations_are_performed_then_the_ledger_settles_on_the_next_pass() {
    let mut r = Rig::running();
    r.settle(Effect::None, Some(100), false);
    let mut world = World::new().with_obligations(&["workspace", "pi-session"]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(
        world.calls_of("clean").len(),
        2,
        "each remaining obligation is performed once: {:?}",
        world.calls
    );
    let Some(Action::CleanupPerformed {
        targets,
        effects,
        readback,
        ledger_settled,
        ..
    }) = &entry.action
    else {
        panic!("{entry:#?}");
    };
    assert_eq!(
        *targets,
        vec![
            CleanupTarget::Remaining {
                name: "workspace".into()
            },
            CleanupTarget::Remaining {
                name: "pi-session".into()
            }
        ]
    );
    assert!(effects.iter().all(|effect| effect.performed));
    assert_eq!(*readback, CleanupReadback::Complete);
    assert!(!ledger_settled);
    assert_eq!(r.area.attempt_row().2, "unknown");
    let next = r.pass(&mut world);
    assert!(matches!(
        next.attempts[0].action,
        Some(Action::CleanupPerformed {
            ledger_settled: true,
            ..
        })
    ));
    assert_eq!(r.area.attempt_row().2, "settled");
    assert_eq!(next.writes, 2);
}

/// `T07-AP-33` · with the remaining obligation refused, the readback stays
/// partial and nothing in the ledger is settled.
#[test]
fn ledger_is_not_settled_while_a_remaining_obligation_persists() {
    let mut r = Rig::running();
    r.settle(Effect::None, Some(100), false);
    let mut world = World::new()
        .with_obligations(&["workspace"])
        .refusing_clean("workspace");
    let pass = r.pass(&mut world);
    let Some(Action::CleanupPerformed {
        readback,
        ledger_settled,
        ..
    }) = &only(&pass).action
    else {
        panic!("{pass:#?}");
    };
    assert_eq!(
        *readback,
        CleanupReadback::Partial {
            remaining: vec!["workspace".into()]
        }
    );
    assert!(!ledger_settled);
    assert_eq!(r.area.attempt_row().2, "unknown");
}

/// `T07-AP-34` · an unknown external effect stays explicit: no cleanup is read for it.
#[test]
fn unknown_effect_is_retained_explicitly() {
    let mut r = Rig::running();
    r.settle(Effect::Unknown, Some(100), true);
    let mut world = World::new().with_obligations(&[]);
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R10EffectAmbiguity,
        &Unknown::EffectUnknown,
        &ProcessCustody::Unobserved,
    );
    assert!(world.calls_of("clean").is_empty());
}

/// `T07-AP-35` · a pending external effect is distinct from unknown.
#[test]
fn pending_effect_is_retained_as_pending() {
    let mut r = Rig::running();
    r.settle(Effect::Pending, None, false);
    let mut world = World::new();
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R10EffectAmbiguity,
        &Unknown::EffectPending,
        &ProcessCustody::Unobserved,
    );
}

/// `T07-AP-36` · a settled attempt whose worker is still alive holds its
/// workspace: R09 refuses reuse as live holder and no cleanup is attempted.
#[test]
fn live_holder_of_a_settled_attempt_blocks_cleanup() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    r.settle(Effect::None, Some(10), true);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_obligations(&["workspace"]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R09WorkspaceReuse);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::WorkspaceReuseRefused {
            reason: ReuseRefusal::LiveHolder,
            process: ProcessCustody::LiveSameIdentity,
        }
    );
    assert_eq!(
        entry.action,
        Some(Action::ReuseRefused {
            reason: ReuseRefusal::LiveHolder
        })
    );
    assert!(world.calls_of("clean").is_empty() && world.calls_of("attach").is_empty());
}

// ---- verification, evidence and acceptance boundaries ------------------------------------------

/// `T07-AP-37` · restart at the verification boundary: worker settled, cleanup
/// complete, no verdict yet → R12 verification outstanding, evidence unassessed.
#[test]
fn restart_at_verification_boundary_reports_verification_outstanding() {
    let mut r = Rig::ready();
    let mut world = World::new().with_obligations(&[]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R12VerificationBoundary);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::VerificationOutstanding {
            task_state: TaskState::Verifying,
            verification: Verification::None,
            evidence: Evidence::Unassessed,
            acceptance_prepared: false,
        }
    );
    assert_eq!(
        entry.action,
        Some(Action::VerificationOutstanding {
            task_state: TaskState::Verifying
        })
    );
    assert_eq!(entry.records.len(), 1);
    assert_eq!(r.area.task_row().0, "verifying");
}

/// `T07-AP-38` · restart at the evidence boundary: a passed verification with
/// its evidence object readable → evidence published, acceptance prepared, and
/// still not acceptance.
#[test]
fn restart_after_passed_verification_reads_evidence_back_as_published() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, true);
    let mut world = World::new().with_obligations(&[]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.handed.evidence, Evidence::Published);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::VerificationOutstanding {
            task_state: TaskState::Verifying,
            verification: Verification::Recorded {
                verdict: Verdict::Passed,
                cleanup_settled: true,
            },
            evidence: Evidence::Published,
            acceptance_prepared: true,
        }
    );
    assert_eq!(r.area.task_row(), ("verifying".into(), false, None));
}

/// `T07-AP-39` · the evidence object's bytes gone: the real readback says absent.
#[test]
fn missing_evidence_object_reads_back_absent() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, true);
    r.remove_evidence_object();
    let mut world = World::new().with_obligations(&[]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.handed.evidence, Evidence::Absent);
    assert!(matches!(
        entry.decision.reconciliation,
        Reconciliation::VerificationOutstanding {
            evidence: Evidence::Absent,
            acceptance_prepared: true,
            ..
        }
    ));
    assert_eq!(body(&r.records()[0])["handed"]["evidence"], "absent");
}

/// `T07-AP-40` · a failed verification leaves the task repair-pending: still R12.
#[test]
fn failed_verification_is_repair_pending_and_outstanding() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Failed, true);
    let mut world = World::new().with_obligations(&[]);
    let pass = r.pass(&mut world);
    assert!(matches!(
        only(&pass).decision.reconciliation,
        Reconciliation::VerificationOutstanding {
            task_state: TaskState::RepairPending,
            verification: Verification::Recorded {
                verdict: Verdict::Failed,
                ..
            },
            acceptance_prepared: false,
            ..
        }
    ));
}

/// `T07-AP-41` · a verifier error fails the task; its attempt's workspace is
/// releasable, and releasing is recorded, not performed.
#[test]
fn failed_task_workspace_is_recorded_releasable_without_effect() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Error, true);
    let mut world = World::new().with_obligations(&[]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R11CleanupReadback);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::WorkspaceReleasable {
            cleanup_readback: CleanupReadback::Complete,
            process: ProcessCustody::Unobserved,
            task_state: TaskState::Failed,
        }
    );
    assert_eq!(
        entry.action,
        Some(Action::ReleasableRecorded {
            task_state: TaskState::Failed
        })
    );
    assert!(world.calls_of("clean").is_empty());
    assert_eq!(entry.records.len(), 1);
}

/// `T07-AP-42` · a stopped (failed) task with a remaining obligation still gets
/// its cleanup performed: R11 precedes the terminal state.
#[test]
fn stopped_task_with_remaining_obligation_is_cleaned() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Failed, true);
    r.stop();
    assert_eq!(r.area.task_row().0, "failed");
    let mut world = World::new().with_obligations(&["workspace"]);
    let pass = r.pass(&mut world);
    assert!(matches!(
        only(&pass).action,
        Some(Action::CleanupPerformed {
            readback: CleanupReadback::Complete,
            ..
        })
    ));
    assert_eq!(world.calls_of("clean").len(), 1);
}

/// `T07-AP-43` · restart at the acceptance boundary: a committed acceptance
/// stands once with its journal ordinal; the ledger head agrees; nothing is
/// rewritten and the second pass writes nothing.
#[test]
fn committed_acceptance_stands_with_its_ordinal_and_is_not_rewritten() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, true);
    r.accept();
    let before = r.area.task_row();
    assert_eq!(before, ("accepted".into(), false, Some(ACCEPT.into())));
    let mut world = World::new().with_obligations(&["workspace"]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R04AcceptanceStands);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::AcceptanceStands {
            event: ACCEPT.into(),
            ordinal: Some(5),
            later_cancellation: None,
            cleanup: Cleanup::Settled,
        }
    );
    assert_eq!(
        entry.handed.history,
        startup::HistoryValue::Accepted {
            event: ACCEPT.into(),
            ordinal: Some(5)
        }
    );
    assert_eq!(
        entry.action,
        Some(Action::AcceptanceStands {
            event: ACCEPT.into(),
            head_event: Some(ACCEPT.into()),
            agrees: true,
        })
    );
    assert!(
        world.calls_of("clean").is_empty(),
        "history precedes physical evidence"
    );
    assert_eq!(r.area.task_row(), before);
    let second = r.pass(&mut world);
    assert_eq!(second.writes, 0);
    assert_eq!(r.area.task_row(), before);
}

/// `T07-AP-44` · a cancellation offered after acceptance changes nothing in the
/// ledger (the store keeps it), and the pass still reports acceptance standing.
#[test]
fn later_cancellation_cannot_rewrite_a_committed_acceptance() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, true);
    r.accept();
    r.cancel();
    assert_eq!(
        r.area.task_row(),
        ("accepted".into(), false, Some(ACCEPT.into()))
    );
    let mut world = World::new();
    let pass = r.pass(&mut world);
    assert!(matches!(
        only(&pass).decision.reconciliation,
        Reconciliation::AcceptanceStands {
            later_cancellation: None,
            ..
        }
    ));
    assert_eq!(
        r.area.task_row(),
        ("accepted".into(), false, Some(ACCEPT.into()))
    );
}

/// `T07-AP-44b` · a ledger holding BOTH commits for one task -- the row flagged cancelled
/// and accepted, and both events journalled -- is handed to the policy as `Both` and ordered
/// by R03: acceptance first, so it stands and names the later cancellation. The store never
/// writes such a ledger (AP-44: a cancellation after acceptance is neither flagged nor
/// journalled; acceptance after cancellation is refused), so it is reached through the
/// test-only row edit the inventory admits, as AP-70 reaches its state. R03 exists for a
/// ledger this store did not write; without this case the `(Some, Some)` arm of
/// `task_history` could be deleted and every such ledger would read as contradictory --
/// mutation shard C found exactly that.
#[test]
fn a_ledger_holding_both_commits_is_ordered_by_its_journal() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, true);
    r.accept();
    r.close();
    {
        let db = Connection::open_with_flags(
            r.area.db(),
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        assert_eq!(
            db.execute("UPDATE tasks SET cancellation=1 WHERE id=?", [TASK])
                .unwrap(),
            1
        );
        assert_eq!(
            db.execute(
                "INSERT INTO events(id,task_id,generation,kind,body) \
                 SELECT ?1,task_id,generation,'cancellation_requested',x'7b7d' \
                 FROM events WHERE id=?2",
                [CANCEL, ACCEPT],
            )
            .unwrap(),
            1
        );
        db.close().unwrap();
    }
    let ordinals = r.ordinals();
    let (_, acceptance) = ordinals.accepted.clone().unwrap();
    let cancellation = ordinals.cancellation.unwrap();
    assert!(
        cancellation > acceptance,
        "{cancellation} after {acceptance}"
    );
    let mut world = World::new();
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R03CommitOrdering);
    assert_eq!(
        entry.handed.history,
        startup::HistoryValue::Both {
            cancellation,
            acceptance_event: ACCEPT.into(),
            acceptance,
        }
    );
    assert!(matches!(
        entry.decision.reconciliation,
        Reconciliation::AcceptanceStands {
            later_cancellation: Some(later),
            ..
        } if later == cancellation
    ));
}

/// Journal `count` reconciliation records for `attempt` on TASK in one statement, through the
/// test-only row edit: writing 4097 through `record_reconciliation` would measure the store's
/// commit rate, not the read bound under test.
fn journal_records(r: &mut Rig, attempt: &str, count: usize, tag: &str) {
    r.close();
    let db = Connection::open_with_flags(
        r.area.db(),
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .unwrap();
    let written = db
        .execute(
            "INSERT INTO events(id,task_id,generation,kind,body) \
             WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i+1 FROM n WHERE i<?1) \
             SELECT ?2||'-'||i, ?3, (SELECT generation FROM tasks WHERE id=?3), \
                    'reconciliation_decided', CAST('{\"attempt\":\"'||?4||'\"}' AS BLOB) FROM n",
            rusqlite::params![i64::try_from(count).unwrap(), tag, TASK, attempt],
        )
        .unwrap();
    assert_eq!(written, count);
    db.close().unwrap();
}

fn read_records(r: &mut Rig) -> Result<usize, StoreError> {
    r.close();
    let store = Store::open_inspection(&r.area.store(), id(GEN), id(EPOCH), deadline()).unwrap();
    store
        .reconciliation_records(id(ATTEMPT), deadline())
        .map(|rows| rows.len())
}

/// `T07-AP-71` · the reconciliation read bound, from both sides: exactly the bound is read
/// whole, one more is refused by name. Mutation shard C found the bound free to move to
/// `>= 4096` or `== 4096` -- nothing read exactly 4096 records.
#[test]
fn reconciliation_records_are_read_whole_at_the_bound_and_refused_past_it() {
    assert_eq!(RECORD_SCAN_LIMIT, 4096);
    let mut r = Rig::ready();
    journal_records(&mut r, ATTEMPT, RECORD_SCAN_LIMIT, "at");
    assert!(matches!(read_records(&mut r), Ok(n) if n == RECORD_SCAN_LIMIT));
    journal_records(&mut r, ATTEMPT, 1, "past");
    assert!(matches!(read_records(&mut r), Err(StoreError::Bound)));
}

/// `T07-AP-72` · the bound is on what the read ACQUIRED, not on what survived the attempt
/// filter. A sibling attempt's 4097 records ahead of this attempt's one used to fill the
/// query's LIMIT and return this attempt's history as an empty Ok -- a silent truncation.
#[test]
fn a_sibling_attempts_records_cannot_truncate_this_attempts_history() {
    let mut r = Rig::ready();
    journal_records(&mut r, OTHER, RECORD_SCAN_LIMIT + 1, "sibling");
    journal_records(&mut r, ATTEMPT, 1, "mine");
    assert!(matches!(read_records(&mut r), Err(StoreError::Bound)));
}

/// `T07-AP-45` · a committed cancellation stands for a settled worker with its
/// journal ordinal; the ledger head agrees.
#[test]
fn committed_cancellation_stands_for_settled_work_with_its_ordinal() {
    let mut r = Rig::running();
    r.cancel();
    r.settle(Effect::None, Some(10), true);
    let mut world = World::new().with_obligations(&[]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R05CancellationStands);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::CancellationStands {
            ordinal: Some(3),
            rejected_acceptance: None,
            attempt_state: AttemptState::Settled,
            cleanup: Cleanup::Settled,
        }
    );
    assert_eq!(
        entry.action,
        Some(Action::CancellationStands {
            rejected_acceptance: None,
            head_cancellation: true,
            agrees: true,
        })
    );
    assert_eq!(r.area.task_row().0, "cancellation_requested");
}

/// `T07-AP-46` · acceptance explicitly rejects an earlier committed cancellation:
/// a prepared acceptance (passed verification) after cancellation is named as
/// rejected, even before terminal cleanup.
#[test]
fn prepared_acceptance_after_cancellation_is_rejected_before_cleanup() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, true);
    r.cancel();
    let mut world = World::new().with_obligations(&["workspace"]);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(
        entry.decision.reconciliation,
        Reconciliation::CancellationStands {
            ordinal: Some(5),
            rejected_acceptance: Some(CHECK.into()),
            attempt_state: AttemptState::Settled,
            cleanup: Cleanup::Settled,
        }
    );
    assert_eq!(
        entry.action,
        Some(Action::CancellationStands {
            rejected_acceptance: Some(CHECK.into()),
            head_cancellation: true,
            agrees: true,
        })
    );
    assert!(
        world.calls_of("clean").is_empty(),
        "the rejection is decided before cleanup"
    );
    assert_eq!(
        r.area.task_row(),
        ("cancellation_requested".into(), true, None)
    );
    assert_eq!(
        body(&r.records()[0])["decision"]["rejected_acceptance"],
        CHECK
    );
}

/// `T07-AP-47` · cancellation over a still-running absent worker is not settled
/// work: the physical rules decide, carrying `cancellation_pending`.
#[test]
fn cancellation_over_running_absent_worker_is_pending_not_stood() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    r.cancel();
    let mut world = World::new().with_acknowledgement(Acknowledgement::NotSeen);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.decision.rule, Rule::R08WorkerAbsent);
    assert!(matches!(
        entry.decision.reconciliation,
        Reconciliation::RetainUnknown {
            reason: Unknown::DispatchUnacknowledged,
            cancellation_pending: true,
            ..
        }
    ));
    let ordinal = r.ordinals().cancellation;
    assert!(ordinal.is_some());
    assert_eq!(
        entry.handed.history,
        startup::HistoryValue::Cancelled { ordinal }
    );
}

/// `T07-AP-48` · a live child under a pending cancellation is reattached with the
/// flag carried, never redispatched.
#[test]
fn cancellation_pending_is_carried_into_reattach() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    r.cancel();
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let pass = r.pass(&mut world);
    assert!(matches!(
        only(&pass).decision.reconciliation,
        Reconciliation::ReattachObservationOnly {
            cancellation_pending: true,
            redispatch: false,
            ..
        }
    ));
    assert_eq!(world.calls_of("attach").len(), 1);
}

/// `T07-AP-49` · a cancelled-then-stopped task: cancellation stands with the
/// terminal state committed, cleanup settled.
#[test]
fn cancelled_and_stopped_task_reports_cancellation_standing() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Failed, true);
    r.cancel();
    r.stop();
    assert_eq!(r.area.task_row(), ("cancelled".into(), true, None));
    let mut world = World::new().with_obligations(&[]);
    let pass = r.pass(&mut world);
    assert!(matches!(
        only(&pass).decision.reconciliation,
        Reconciliation::CancellationStands {
            ordinal: Some(5),
            rejected_acceptance: None,
            ..
        }
    ));
}

// ---- cursors and restored epochs -------------------------------------------------------------

fn cursor(epoch: &str, sequence: u64) -> Cursor {
    Cursor {
        epoch: epoch.into(),
        sequence,
    }
}

/// `T07-AP-50` · a cursor of this epoch at or below the high-water mark is a
/// snapshot only; the receipt carries `replay: false` read from the arm.
#[test]
fn current_epoch_cursor_is_snapshot_only() {
    let mut r = Rig::running();
    let mut world = World::new();
    let pass = r.pass_with(&mut world, None, &[cursor(EPOCH, 2)], &[]);
    assert_eq!(pass.cursors.len(), 1);
    assert_eq!(pass.cursors[0].rule, "R13");
    assert_eq!(
        pass.cursors[0].decision.reconciliation,
        Reconciliation::CursorSnapshotOnly {
            epoch: EPOCH.into(),
            sequence: 2,
            event_high_water: 2,
            mode: Mode::Normal,
            replay: false,
        }
    );
    assert!(!pass.permits_execution);
}

/// `T07-AP-51` · a cursor ahead of the journal is refused as a future sequence.
#[test]
fn future_sequence_cursor_is_refused() {
    let mut r = Rig::running();
    let mut world = World::new();
    let pass = r.pass_with(&mut world, None, &[cursor(EPOCH, 3)], &[]);
    assert_eq!(
        pass.cursors[0].decision.reconciliation,
        Reconciliation::RefuseStaleCursor {
            reason: CursorRefusal::FutureSequence,
            cursor_epoch: EPOCH.into(),
            cursor_sequence: 3,
            ledger_epoch: EPOCH.into(),
            event_high_water: 2,
        }
    );
}

/// `T07-AP-52` · a cursor from another epoch is refused as epoch changed.
#[test]
fn other_epoch_cursor_is_refused() {
    let mut r = Rig::admitted();
    let mut world = World::new();
    let pass = r.pass_with(&mut world, None, &[cursor(OTHER, 1)], &[]);
    assert!(matches!(
        pass.cursors[0].decision.reconciliation,
        Reconciliation::RefuseStaleCursor {
            reason: CursorRefusal::EpochChanged,
            ..
        }
    ));
}

/// `T07-AP-53` · a restored ledger cannot silently reuse a cursor of the epoch it
/// was restored from, even when that cursor's sequence would otherwise fit.
#[test]
fn prior_epoch_of_a_restore_is_refused_by_name() {
    let mut r = Rig::admitted();
    let mut world = World::new();
    let pass = r.pass_with(
        &mut world,
        Some(OTHER),
        &[cursor(OTHER, 1), cursor(EPOCH, 1)],
        &[],
    );
    assert_eq!(pass.cursors.len(), 2);
    assert!(matches!(
        pass.cursors[0].decision.reconciliation,
        Reconciliation::RefuseStaleCursor {
            reason: CursorRefusal::PriorEpochOfRestore,
            ..
        }
    ));
    assert!(matches!(
        pass.cursors[1].decision.reconciliation,
        Reconciliation::CursorSnapshotOnly { replay: false, .. }
    ));
}

/// `T07-AP-54` · a ledger in reconciliation mode (test-only epoch rotation on a
/// closed ledger, as `t07_inventory` performs it) can only be inspected: the pass
/// reports every decision, records none, and performs no effect.
#[test]
fn reconciliation_mode_ledger_is_decided_but_not_acted_on() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    r.close();
    {
        let db = Connection::open_with_flags(
            r.area.db(),
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        db.execute(
            "UPDATE ledger_meta SET mode='reconciliation' WHERE singleton=1 AND epoch=?",
            [EPOCH],
        )
        .unwrap();
        db.close().unwrap();
    }
    assert!(matches!(
        Store::open(&r.area.store(), id(GEN), id(EPOCH), false, deadline()),
        Err(StoreError::RecoveryRequired)
    ));
    let before = r.area.events();
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let pass = r.pass(&mut world);
    assert_eq!(pass.mode, Mode::Reconciliation);
    assert_eq!(pass.ledger, LedgerAccess::InspectionOnly);
    assert_eq!(pass.attempts.len(), 1);
    assert_eq!(pass.attempts[0].decision.rule, Rule::R06LiveOwnedChild);
    assert_eq!(pass.attempts[0].action, None);
    assert!(pass.attempts[0].records.is_empty());
    assert_eq!(pass.writes, 0);
    assert!(
        world.calls_of("attach").is_empty(),
        "no effect without a recordable intent"
    );
    assert_eq!(r.area.events(), before);
    assert!(r.records().is_empty());
}

// ---- the store's record surface -------------------------------------------------------------

/// `T07-AP-55` · a record body must be a JSON object within the bound; the
/// journal is untouched by a refusal.
#[test]
fn record_bodies_outside_the_bound_are_refused() {
    let mut r = Rig::running();
    let before = r.area.events();
    for (body, expected) in [
        (Vec::new(), "Bound"),
        (b"[]".to_vec(), "Bound"),
        (b"\"text\"".to_vec(), "Bound"),
        (
            format!("{{\"pad\":\"{}\"}}", "x".repeat(RECORD_BODY_LIMIT)).into_bytes(),
            "Bound",
        ),
    ] {
        let error = r
            .store()
            .record_reconciliation(
                &ReconciliationRecord {
                    attempt: id(ATTEMPT),
                    kind: RecordKind::Decided,
                    body: &body,
                    settle_cleanup: false,
                },
                deadline(),
            )
            .unwrap_err();
        assert_eq!(format!("{error:?}"), expected, "{}", body.len());
    }
    let largest = format!("{{\"pad\":\"{}\"}}", "x".repeat(RECORD_BODY_LIMIT - 10));
    assert_eq!(largest.len(), RECORD_BODY_LIMIT);
    let recorded = r
        .store()
        .record_reconciliation(
            &ReconciliationRecord {
                attempt: id(ATTEMPT),
                kind: RecordKind::Decided,
                body: largest.as_bytes(),
                settle_cleanup: false,
            },
            deadline(),
        )
        .unwrap();
    assert!(recorded.fresh);
    assert_eq!(r.area.events(), before + 1);
}

/// `T07-AP-56` · an inspection-only store cannot record; an unknown attempt,
/// a settlement on a decided record and a settlement of a settled row are refused.
#[test]
fn record_refusals_name_their_reasons() {
    let mut r = Rig::ready();
    let body = br#"{"attempt":"x"}"#;
    let settle = ReconciliationRecord {
        attempt: id(ATTEMPT),
        kind: RecordKind::Decided,
        body,
        settle_cleanup: true,
    };
    assert!(matches!(
        r.store().record_reconciliation(&settle, deadline()),
        Err(StoreError::Invalid)
    ));
    let settled_row = ReconciliationRecord {
        kind: RecordKind::Readback,
        ..settle
    };
    assert!(matches!(
        r.store().record_reconciliation(&settled_row, deadline()),
        Err(StoreError::Conflict)
    ));
    let unknown = ReconciliationRecord {
        attempt: id(OTHER),
        kind: RecordKind::Decided,
        body,
        settle_cleanup: false,
    };
    assert!(matches!(
        r.store().record_reconciliation(&unknown, deadline()),
        Err(StoreError::NotFound)
    ));
    r.close();
    let mut inspection =
        Store::open_inspection(&r.area.store(), id(GEN), id(EPOCH), deadline()).unwrap();
    assert!(matches!(
        inspection.record_reconciliation(
            &ReconciliationRecord {
                attempt: id(ATTEMPT),
                kind: RecordKind::Decided,
                body,
                settle_cleanup: false,
            },
            deadline()
        ),
        Err(StoreError::InspectionOnly)
    ));
    assert_eq!(r.area.events(), 3);
}

/// `T07-AP-57` · the record id is content-derived and spelled as the ledger's
/// UUID grammar: equal inputs give equal ids, any changed coordinate a new one.
#[test]
fn record_id_is_deterministic_and_parses_as_a_ledger_uuid() {
    let a = record_id(EPOCH, ATTEMPT, RecordKind::Decided, b"{}");
    assert_eq!(a, record_id(EPOCH, ATTEMPT, RecordKind::Decided, b"{}"));
    UuidV4::parse(&a).unwrap();
    let variants = [
        record_id(OTHER, ATTEMPT, RecordKind::Decided, b"{}"),
        record_id(EPOCH, OTHER, RecordKind::Decided, b"{}"),
        record_id(EPOCH, ATTEMPT, RecordKind::Readback, b"{}"),
        record_id(EPOCH, ATTEMPT, RecordKind::Decided, b"{ }"),
    ];
    for other in &variants {
        assert_ne!(&a, other);
        UuidV4::parse(other).unwrap();
    }
    assert_eq!(variants.iter().collect::<BTreeSet<_>>().len(), 4);
    assert_eq!(
        RecordKind::parse("reconciliation_decided"),
        Some(RecordKind::Decided)
    );
    assert_eq!(
        RecordKind::parse("reconciliation_readback"),
        Some(RecordKind::Readback)
    );
    assert_eq!(RecordKind::parse("accepted"), None);
}

/// `T07-AP-58` · the same record written twice through the store is one row, and
/// the second call says so.
#[test]
fn identical_record_is_found_not_rewritten() {
    let mut r = Rig::running();
    let record = ReconciliationRecord {
        attempt: id(ATTEMPT),
        kind: RecordKind::Decided,
        body: br#"{"attempt":"07000000-0000-4000-8000-000000000006","k":1}"#,
        settle_cleanup: false,
    };
    let first = r
        .store()
        .record_reconciliation(&record, deadline())
        .unwrap();
    let second = r
        .store()
        .record_reconciliation(&record, deadline())
        .unwrap();
    assert!(first.fresh && !second.fresh);
    assert_eq!(
        (first.event.as_str(), first.sequence),
        (second.event.as_str(), second.sequence)
    );
    assert_eq!(first.sequence, 3);
    let rows = r.records();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].generation, "2");
    assert_eq!(r.area.events(), 3);
}

/// `T07-AP-59` · the journal ordinals of a task's terminal commits are read
/// from the events themselves, and absent when uncommitted.
#[test]
fn terminal_ordinals_come_from_the_journal() {
    let mut r = Rig::ready();
    assert_eq!(
        r.store().terminal_ordinals(id(TASK), deadline()).unwrap(),
        habitat_engine::store::TerminalOrdinals {
            accepted: None,
            cancellation: None
        }
    );
    r.verify(VerificationVerdict::Passed, true);
    r.accept();
    assert_eq!(
        r.store().terminal_ordinals(id(TASK), deadline()).unwrap(),
        habitat_engine::store::TerminalOrdinals {
            accepted: Some((ACCEPT.into(), 5)),
            cancellation: None
        }
    );
    assert!(matches!(
        r.store().terminal_ordinals(id(OTHER), deadline()),
        Err(StoreError::NotFound)
    ));
}

/// `T07-AP-60` · evidence availability is a two-stage readback, and each stage is
/// load-bearing: the digest must be REGISTERED by a ledger row (`publish` writes the
/// CAS object before any ledger reference and registers nothing), and its bytes must
/// then verify against the registered size. The three outcomes below are reached by
/// three different causes — registered+readable, never registered, registered but the
/// bytes gone — so neither stage can be deleted without reddening this case.
#[test]
fn evidence_availability_reads_the_object_back() {
    let mut r = Rig::ready();
    let digest = r.evidence.digest().to_owned();
    // Before any ledger row names it, a published object is NOT available: stage 1
    // is the `artifacts` registration, and `publish` deliberately does not write it.
    assert_eq!(
        r.store().evidence_available(&digest, deadline()).unwrap(),
        habitat_engine::store::EvidenceAvailability::Absent,
        "published but unregistered evidence is absent"
    );
    r.verify(VerificationVerdict::Passed, true);
    // Stage 1 now passes and stage 2 reads the bytes back.
    assert_eq!(
        r.store().evidence_available(&digest, deadline()).unwrap(),
        habitat_engine::store::EvidenceAvailability::Published
    );
    // `record_verification` registers only the evidence digest, never the subject,
    // so the subject digest is still unregistered: stage 1 refuses it.
    assert_eq!(
        r.store().evidence_available(DIGEST, deadline()).unwrap(),
        habitat_engine::store::EvidenceAvailability::Absent
    );
    // Registered, but the bytes are gone: only stage 2 can produce this one.
    r.remove_evidence_object();
    let store = Store::open_inspection(&r.area.store(), id(GEN), id(EPOCH), deadline()).unwrap();
    assert_eq!(
        store.evidence_available(&digest, deadline()).unwrap(),
        habitat_engine::store::EvidenceAvailability::Absent
    );
}

/// `T07-AP-61` · `intended` names the action every attempt arm will take, and the
/// two cursor arms handed to the attempt actor are named as not attempt decisions.
#[test]
fn intended_action_names_are_closed_over_the_decision_arms() {
    let cursor_arm = Reconciliation::CursorSnapshotOnly {
        epoch: EPOCH.into(),
        sequence: 1,
        event_high_water: 1,
        mode: Mode::Normal,
        replay: false,
    };
    let refused = Reconciliation::RefuseStaleCursor {
        reason: CursorRefusal::EpochChanged,
        cursor_epoch: OTHER.into(),
        cursor_sequence: 1,
        ledger_epoch: EPOCH.into(),
        event_high_water: 1,
    };
    assert_eq!(intended(&cursor_arm), "not_an_attempt_decision");
    assert_eq!(intended(&refused), "not_an_attempt_decision");
    assert_eq!(
        intended(&Reconciliation::CleanupCandidate {
            what: vec![],
            process: ProcessCustody::Absent
        }),
        "cleanup_performed"
    );
    assert_eq!(
        intended(&Reconciliation::WorkspaceReleasable {
            cleanup_readback: CleanupReadback::Complete,
            process: ProcessCustody::Absent,
            task_state: TaskState::Failed
        }),
        "releasable_recorded"
    );
    assert_eq!(
        Action::NotAnAttemptDecision.name(),
        "not_an_attempt_decision"
    );
    assert_eq!(Action::StaleRefused.name(), "stale_refused");
    assert_eq!(Action::RetainedUnknown.name(), "retained_unknown");
}

// ---- the runtime world: real readbacks ---------------------------------------------------------

/// A real child that answers the Pi `get_state` command over its pipes with the
/// given state and otherwise waits; its identity is the world's, not a fixture's.
struct Peer {
    child: Child,
    reader: Option<BufReader<std::process::ChildStdout>>,
    pid: u32,
}
/// How a `Peer` shapes its one reply line.
#[derive(Clone, Copy)]
enum Reply {
    /// The compact reply, newline-terminated.
    Line,
    /// Newline-terminated, padded (inside `sessionId`) to exactly this many bytes before
    /// the newline.
    Sized(usize),
    /// The compact reply, one stray byte, then end-of-file with the peer still alive: an
    /// unterminated frame whose parse would succeed if the stray byte were dropped.
    Unterminated,
}

impl Peer {
    fn spawn(idle: bool) -> Self {
        Self::spawn_with(idle, Reply::Line)
    }
    fn spawn_with(idle: bool, reply: Reply) -> Self {
        let (target, stray) = match reply {
            Reply::Line => (0, false),
            Reply::Sized(bytes) => (bytes, false),
            Reply::Unterminated => (0, true),
        };
        let state = if idle {
            r#"{"thinkingLevel":"off","isStreaming":false,"isCompacting":false,"steeringMode":"all","followUpMode":"one-at-a-time","sessionId":"owned-pi-peer","autoCompactionEnabled":false,"messageCount":0,"pendingMessageCount":0}"#
        } else {
            r#"{"thinkingLevel":"off","isStreaming":true,"isCompacting":false,"steeringMode":"all","followUpMode":"one-at-a-time","sessionId":"owned-pi-peer","autoCompactionEnabled":false,"messageCount":3,"pendingMessageCount":2}"#
        };
        let stray = if stray { "True" } else { "False" };
        let script = format!(
            "import json,os,sys\nsys.stdout.write('ready\\n');sys.stdout.flush()\nfor line in sys.stdin:\n    q=json.loads(line)\n    if q.get('type')=='get_state':\n        d=json.loads('{state}')\n        r=lambda: json.dumps({{'type':'response','id':q['id'],'command':'get_state','success':True,'data':d}},separators=(',',':'))\n        if {target}:\n            d['sessionId']=''\n            d['sessionId']='p'*({target}-len(r().encode()))\n            assert len(r().encode())=={target}, len(r().encode())\n        if {stray}:\n            sys.stdout.write(r()+'Z');sys.stdout.flush();os.close(1)\n        else:\n            sys.stdout.write(r()+'\\n');sys.stdout.flush()\n"
        );
        let mut child = Command::new("/usr/bin/python3")
            .args(["-B", "-c", &script])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut reader = BufReader::new(child.stdout.take().unwrap());
        let mut ready = String::new();
        reader.read_line(&mut ready).unwrap();
        assert_eq!(ready, "ready\n");
        let pid = child.id();
        Self {
            child,
            reader: Some(reader),
            pid,
        }
    }
    /// The identity text the roster would retain, from the same `/proc` read the
    /// classifier makes.
    fn identity_text(&self) -> String {
        let LiveRead::Present(live) = live_read(self.pid, deadline()) else {
            panic!("peer {} not present", self.pid)
        };
        serde_json::to_string(&ProcessIdentity {
            pid: self.pid,
            start_ticks: live.start_ticks,
            namespace: live.namespace,
        })
        .unwrap()
    }
    fn link(&mut self) -> PiLink {
        PiLink {
            reader: Box::new(self.reader.take().unwrap()),
            writer: Box::new(self.child.stdin.take().unwrap()),
        }
    }
    /// Kill by handle and reap; never by pattern.
    fn kill(&mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// `T07-AP-62` · real `/proc` and a real Pi peer over pipes: the live child of the
/// retained identity with an idle session is reattached through `Host`, which
/// then holds a PID handle; the readback after attach is the live identity.
#[test]
fn host_reattaches_a_real_live_child_with_an_idle_pi_session() {
    let mut peer = Peer::spawn(true);
    let text = peer.identity_text();
    let mut r = Rig::rostered(&text, 60_000);
    let mut host = Host::new(deadline());
    host.pi.insert(ATTEMPT.into(), peer.link());
    let pass = r.pass(&mut host);
    let entry = only(&pass);
    assert_eq!(
        entry.handed.identity.as_ref().map(|i| i.pid),
        Some(peer.pid)
    );
    assert_eq!(entry.handed.process, ProcessCustody::LiveSameIdentity);
    assert_eq!(entry.handed.pi_queue, PiQueueCustody::Idle);
    assert_eq!(entry.decision.rule, Rule::R06LiveOwnedChild);
    assert_eq!(
        entry.action,
        Some(Action::ObservationAttached {
            generation: 1,
            readback: ProcessCustody::LiveSameIdentity
        })
    );
    assert_eq!(host.attached(), vec![ATTEMPT]);
    assert_eq!(entry.records.len(), 2);
    assert_eq!(r.records().len(), 2);
}

/// `T07-AP-63` · the same real child with a busy Pi session: the codec refuses to
/// adopt it, the queue stays unreconciled, the live child is retained, nothing attached.
#[test]
fn host_keeps_a_real_live_child_unknown_when_its_pi_session_is_busy() {
    let mut peer = Peer::spawn(false);
    let text = peer.identity_text();
    let mut r = Rig::rostered(&text, 60_000);
    let mut host = Host::new(deadline());
    host.pi.insert(ATTEMPT.into(), peer.link());
    let pass = r.pass(&mut host);
    retained(
        only(&pass),
        Rule::R06LiveOwnedChild,
        &Unknown::PiQueueUnreconciled,
        &ProcessCustody::LiveSameIdentity,
    );
    assert!(host.attached().is_empty());
}

/// The Pi queue custody `Host` reads from a real peer whose reply has the given shape.
fn pi_queue_through(reply: Reply) -> PiQueueCustody {
    let mut peer = Peer::spawn_with(true, reply);
    let text = peer.identity_text();
    let mut r = Rig::rostered(&text, 60_000);
    let mut host = Host::new(deadline());
    host.pi.insert(ATTEMPT.into(), peer.link());
    let pass = r.pass(&mut host);
    only(&pass).handed.pi_queue.clone()
}

/// `T07-AP-63b` · the Pi frame bound from both sides, through a real peer: a frame of
/// exactly 4096 bytes is read, one of 4097 is refused. Mutation shard C found the bound
/// free to move to `== 4096` -- no case had ever sent a frame at the bound.
#[test]
fn a_pi_frame_at_the_bound_is_read_and_one_past_it_is_refused() {
    assert_eq!(pi_queue_through(Reply::Sized(4096)), PiQueueCustody::Idle);
    assert_eq!(
        pi_queue_through(Reply::Sized(4097)),
        PiQueueCustody::Unreconciled
    );
}

/// `T07-AP-63c` · a frame that ends without its newline is refused, even when dropping the
/// last byte would leave a valid reply. Shard C found `||` free to become `&&`, which reads
/// the unterminated reply as an idle queue.
#[test]
fn an_unterminated_pi_frame_is_refused() {
    assert_eq!(pi_queue_through(Reply::Line), PiQueueCustody::Idle);
    assert_eq!(
        pi_queue_through(Reply::Unterminated),
        PiQueueCustody::Unreconciled
    );
}

/// `T07-AP-64` · the real child killed by its handle and reaped: `/proc` says
/// absent; with the caller's correlated acknowledgement the worker is lost.
#[test]
fn host_reads_a_reaped_real_child_as_absent() {
    let mut peer = Peer::spawn(true);
    let text = peer.identity_text();
    let mut r = Rig::rostered(&text, 60_000);
    peer.kill();
    let mut host = Host::new(deadline());
    host.acknowledgements.insert(
        ATTEMPT.into(),
        Acknowledgement::Correlated { generation: 1 },
    );
    let pass = r.pass(&mut host);
    retained(
        only(&pass),
        Rule::R08WorkerAbsent,
        &Unknown::AcknowledgedWorkerLost { generation: 1 },
        &ProcessCustody::Absent,
    );
    let plain = r.pass(&mut Host::new(deadline()));
    retained(
        only(&plain),
        Rule::R08WorkerAbsent,
        &Unknown::AcknowledgementUnrecorded,
        &ProcessCustody::Absent,
    );
}

/// `T07-AP-65` · a real workspace directory: cleanup reads back partial and the
/// workspace writable with its bytes; the effect removes it under the owner
/// guard; the readback after is complete and the next pass finds R12.
#[test]
fn host_cleans_a_real_workspace_directory_and_reads_it_back() {
    let mut r = Rig::ready();
    let area = Area::new("workspace");
    let workspace = area.path.join("ws");
    DirBuilder::new().mode(0o700).create(&workspace).unwrap();
    fs::create_dir(workspace.join("nested")).unwrap();
    fs::write(workspace.join("output"), b"12345").unwrap();
    fs::write(workspace.join("nested").join("more"), b"67").unwrap();
    let mut host = Host::new(deadline());
    host.workspaces.insert(ATTEMPT.into(), workspace.clone());
    let pass = r.pass(&mut host);
    let entry = only(&pass);
    assert_eq!(
        entry.handed.cleanup,
        CleanupReadback::Partial {
            remaining: vec!["workspace".into()]
        }
    );
    assert_eq!(
        entry.handed.workspace,
        WorkspaceReadback::Writable { bytes: 7 }
    );
    assert!(matches!(
        entry.action,
        Some(Action::CleanupPerformed {
            readback: CleanupReadback::Complete,
            ledger_settled: false,
            ..
        })
    ));
    assert!(!workspace.exists());
    let next = r.pass(&mut host);
    assert_eq!(next.attempts[0].handed.cleanup, CleanupReadback::Complete);
    assert_eq!(
        next.attempts[0].handed.workspace,
        WorkspaceReadback::Released
    );
    assert_eq!(
        next.attempts[0].decision.rule,
        Rule::R12VerificationBoundary
    );
}

/// `T07-AP-66` · nothing bound for the attempt: the host infers no convention and
/// answers not-read, unreconciled and unrecorded.
#[test]
fn host_without_bindings_reads_nothing_and_infers_nothing() {
    let mut r = Rig::ready();
    let mut host = Host::new(deadline());
    let pass = r.pass(&mut host);
    let entry = only(&pass);
    assert_eq!(entry.handed.cleanup, CleanupReadback::NotRead);
    assert_eq!(entry.handed.workspace, WorkspaceReadback::NotRead);
    assert_eq!(entry.handed.pi_queue, PiQueueCustody::Unreconciled);
    assert_eq!(entry.handed.acknowledgement, Acknowledgement::Unrecorded);
    assert_eq!(entry.handed.clock_epoch, None);
    retained(
        entry,
        Rule::R11CleanupReadback,
        &Unknown::CleanupUnverified,
        &ProcessCustody::Unobserved,
    );
}

/// `T07-AP-67` · the host's workspace guards: a symlinked or wrongly-moded path
/// is not read, and cleaning refuses it and any target other than the workspace.
#[test]
fn host_workspace_guards_refuse_links_modes_and_unknown_targets() {
    let area = Area::new("guards");
    let real = area.path.join("real");
    DirBuilder::new().mode(0o700).create(&real).unwrap();
    let link = area.path.join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let open = area.path.join("open");
    DirBuilder::new().mode(0o755).create(&open).unwrap();
    let mut host = Host::new(deadline());
    host.workspaces.insert("link".into(), link.clone());
    host.workspaces.insert("open".into(), open.clone());
    host.workspaces.insert("real".into(), real.clone());
    let subject = |attempt: &'static str| Subject {
        task: TASK,
        attempt,
        generation: 1,
        workspace_ref: None,
        session: None,
    };
    assert_eq!(host.workspace(&subject("link")), WorkspaceReadback::NotRead);
    assert_eq!(host.workspace(&subject("open")), WorkspaceReadback::NotRead);
    assert_eq!(
        host.workspace(&subject("real")),
        WorkspaceReadback::Writable { bytes: 0 }
    );
    assert_eq!(
        host.cleanup(&subject("link")),
        CleanupReadback::Partial {
            remaining: vec!["workspace".into()]
        }
    );
    assert!(host.clean(&subject("link"), "workspace").is_err());
    assert!(link.exists() && real.exists());
    assert!(host.clean(&subject("open"), "workspace").is_err());
    assert!(open.exists());
    assert!(host.clean(&subject("real"), "pi-session").is_err());
    assert!(real.exists());
    assert!(host.clean(&subject("none"), "workspace").is_err());
    assert_eq!(host.clean(&subject("real"), "workspace"), Ok(()));
    assert!(!real.exists());
    assert_eq!(host.cleanup(&subject("real")), CleanupReadback::Complete);
    assert_eq!(
        host.workspace(&subject("real")),
        WorkspaceReadback::Released
    );
    fs::set_permissions(&open, fs::Permissions::from_mode(0o700)).unwrap();
}

/// `T07-AP-68` · the shared classifier and `/proc` reader: this process is live
/// with its own namespace, an impossible PID is absent, a passed deadline is
/// unreadable, and the stat parser counts fields after the last parenthesis.
#[test]
fn shared_process_reader_and_classifier_agree_with_the_kernel() {
    let LiveRead::Present(me) = live_read(std::process::id(), deadline()) else {
        panic!("own process is present")
    };
    let namespace = fs::read_link("/proc/self/ns/pid").unwrap();
    assert_eq!(me.namespace, namespace.to_str().unwrap());
    assert!(me.start_ticks > 0);
    assert_eq!(live_read(u32::MAX, deadline()), LiveRead::Absent);
    assert_eq!(
        live_read(std::process::id(), Instant::now()),
        LiveRead::Unreadable("inspection deadline".into())
    );
    let observed = ProcessIdentity {
        pid: std::process::id(),
        start_ticks: me.start_ticks,
        namespace: me.namespace.clone(),
    };
    assert_eq!(
        classify(&observed, &live_read(std::process::id(), deadline())),
        ProcessCustody::LiveSameIdentity
    );
    assert_eq!(
        classify(&observed, &present(me.start_ticks + 1, "pid:[1]")),
        ProcessCustody::PidReused {
            differs: vec![Dimension::StartTicks, Dimension::Namespace]
        }
    );
    let stat = "4127178 (fix) ture) S 4127176 4127178 4127176 0 -1 4194560 213 0 0 0 0 0 0 0 20 0 1 0 65524797 237809664 941 0 0 0 0 0 0 0 65536 4 65538 1 0 0 17 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0";
    assert_eq!(parse_stat(stat), Some(('S', 65_524_797)));
    assert_eq!(parse_stat("no parenthesis"), None);
    assert_eq!(ProcessIdentity::parse(IDENTITY), Some(identity()));
    assert_eq!(ProcessIdentity::parse("literal/fixture"), None);
}

/// `T07-AP-69` · the receipt's `permits_execution` is read from every decision:
/// over a pass whose entries span reattach, retain and a cursor snapshot, it is
/// false because no arm carries a true flag, not because it was never read.
#[test]
fn receipt_reads_permission_from_every_decision() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let pass = r.pass_with(&mut world, None, &[cursor(EPOCH, 1)], &[]);
    assert!(matches!(
        pass.attempts[0].decision.reconciliation,
        Reconciliation::ReattachObservationOnly {
            redispatch: false,
            ..
        }
    ));
    assert!(matches!(
        pass.cursors[0].decision.reconciliation,
        Reconciliation::CursorSnapshotOnly { replay: false, .. }
    ));
    assert!(!pass.permits_execution);
    assert!(!pass.attempts[0].decision.reconciliation.permits_execution());
    let planted = Reconciliation::ReattachObservationOnly {
        generation: 1,
        process: ProcessCustody::LiveSameIdentity,
        pi_queue: PiQueueCustody::Idle,
        cancellation_pending: false,
        redispatch: true,
    };
    assert!(planted.permits_execution(), "the flag is what is read");
}

/// `T07-AP-70` · a queued attempt state the store never produces is not
/// interpreted: R14 retains it (reached through the test-only row edit the
/// inventory admits, on a closed ledger).
#[test]
fn queued_attempt_state_is_retained_uninterpreted() {
    let mut r = Rig::running();
    r.close();
    {
        let db = Connection::open_with_flags(
            r.area.db(),
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        assert_eq!(
            db.execute("UPDATE attempts SET state='queued' WHERE id=?", [ATTEMPT])
                .unwrap(),
            1
        );
        db.close().unwrap();
    }
    let mut world = World::new();
    let pass = r.pass(&mut world);
    retained(
        only(&pass),
        Rule::R14UnexpectedState,
        &Unknown::UnexpectedAttemptState {
            state: AttemptState::Queued,
        },
        &ProcessCustody::Unobserved,
    );
}

/// `T07-AP-71` · the world's answers are keyed by what it is handed: a pass over
/// a pin naming another PID reads that PID, not the one the world prepared.
#[test]
fn seam_answers_follow_the_identity_the_pass_hands_it() {
    let other = r#"{"namespace":"pid:[4026534249]","pid":4243,"start_ticks":65353749}"#;
    let mut r = Rig::rostered(other, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let pass = r.pass(&mut world);
    let entry = only(&pass);
    assert_eq!(entry.handed.identity.as_ref().map(|i| i.pid), Some(4243));
    assert_eq!(
        world.calls_of("process"),
        vec![&Call::Process {
            subject: subject_value(1, true),
            identity: ProcessIdentity {
                pid: 4243,
                start_ticks: START_TICKS,
                namespace: NAMESPACE.into()
            }
        }]
    );
    retained(
        entry,
        Rule::R08WorkerAbsent,
        &Unknown::AcknowledgementUnrecorded,
        &ProcessCustody::Absent,
    );
}

/// `T07-AP-72` · every row the pass writes names its attempt, and the journal
/// readback returns them in ordinal order across both kinds.
#[test]
fn journal_readback_returns_the_pass_rows_in_ordinal_order() {
    let mut r = Rig::rostered(IDENTITY, 60_000);
    let mut world = World::new()
        .with_process(PID, present(START_TICKS, NAMESPACE))
        .with_queue(PiQueueCustody::Idle);
    let pass = r.pass(&mut world);
    let rows = r.records();
    assert_eq!(
        rows.iter().map(|row| row.sequence).collect::<Vec<_>>(),
        pass.attempts[0]
            .records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>()
    );
    assert!(
        rows.windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence)
    );
    for row in &rows {
        assert_eq!(body(row)["attempt"], ATTEMPT);
    }
}

// ---- the process-identity surface moved out of the frontend inspector -------------------------

/// `T07-AP-73` · the 256-byte cap is a bound on the identity TEXT, not on what it
/// spells: the two neighbours either side of it are both well-formed identities
/// differing only in JSON whitespace, so nothing but the bound can separate them.
#[test]
fn identity_text_bound_separates_two_hundred_fifty_six_from_two_hundred_fifty_seven() {
    let head = r#"{"pid":4242,"start_ticks":65353749,"namespace":"pid:[4026534249]""#;
    let padded = |len: usize| {
        let pad = len - head.len() - 1;
        let text = format!("{head}{}}}", " ".repeat(pad));
        assert_eq!(text.len(), len, "fixture builds an exact-length text");
        text
    };
    let expected = ProcessIdentity {
        pid: 4242,
        start_ticks: 65_353_749,
        namespace: "pid:[4026534249]".into(),
    };
    assert_eq!(ProcessIdentity::parse(&padded(256)), Some(expected));
    assert_eq!(ProcessIdentity::parse(&padded(257)), None);
}

/// `T07-AP-74` · each conjunct of the identity predicate refuses on its own: a
/// zero PID with a well-formed namespace link, and a well-formed PID with each
/// shape of malformed link — no prefix, no suffix, no digits, a non-digit, and
/// one digit past the twenty-digit cap. The twenty-digit link and the fixture
/// identity are the two positives, so none of the refusals is vacuous.
#[test]
fn identity_refuses_a_zero_pid_and_every_malformed_namespace_link() {
    let text =
        |pid: u32, ns: &str| format!(r#"{{"pid":{pid},"start_ticks":7,"namespace":"{ns}"}}"#);
    assert_eq!(
        ProcessIdentity::parse(&text(0, "pid:[4026534249]")),
        None,
        "pid 0 is not a process, however good the namespace link"
    );
    for namespace in [
        "",
        "pid:[]",
        "pid:[4026534249",
        "4026534249]",
        "pid:[4026x34249]",
        "pid:[123456789012345678901]",
    ] {
        assert_eq!(
            ProcessIdentity::parse(&text(1, namespace)),
            None,
            "malformed namespace link accepted: {namespace}"
        );
    }
    assert_eq!(
        ProcessIdentity::parse(&text(1, "pid:[12345678901234567890]")),
        Some(ProcessIdentity {
            pid: 1,
            start_ticks: 7,
            namespace: "pid:[12345678901234567890]".into()
        }),
        "exactly twenty digits is inside the cap"
    );
    assert_eq!(ProcessIdentity::parse(IDENTITY), Some(identity()));
}

/// `T07-AP-75` · "gone" is exactly `ENOENT` and `ESRCH`. Every other errno is an
/// unreadable process, never an absent one: a read that was refused says nothing
/// about whether the process is there, and reporting it as absence would let a
/// custody decision be made from a read that never happened.
#[test]
fn only_enoent_and_esrch_mean_the_process_is_gone() {
    use std::io::{Error as IoError, ErrorKind};
    assert!(gone(&IoError::from(ErrorKind::NotFound)));
    assert!(gone(&IoError::from_raw_os_error(
        rustix::io::Errno::NOENT.raw_os_error()
    )));
    assert!(gone(&IoError::from_raw_os_error(
        rustix::io::Errno::SRCH.raw_os_error()
    )));
    for errno in [
        rustix::io::Errno::ACCESS,
        rustix::io::Errno::PERM,
        rustix::io::Errno::IO,
        rustix::io::Errno::NOTDIR,
        rustix::io::Errno::INVAL,
    ] {
        let error = IoError::from_raw_os_error(errno.raw_os_error());
        assert!(
            !gone(&error),
            "{errno:?} was read as absence: {error} kind={:?}",
            error.kind()
        );
    }
}

/// `T07-AP-75b` · the whole `/proc` failure decision, reached by argument. As match guards
/// inside `live_read` it survived mutation shard C both ways (`gone` replaced by `true` and
/// by `false`): only an arranged `/proc` entry reached the non-gone branch. Two unreadable
/// fixtures differ in both the read named and the errno, and each is asserted whole.
#[test]
fn a_failed_proc_read_is_absence_only_when_the_process_is_gone() {
    use std::io::Error as IoError;
    let errno = |e: rustix::io::Errno| IoError::from_raw_os_error(e.raw_os_error());
    assert_eq!(
        proc_read_failure(&errno(rustix::io::Errno::NOENT), "stat"),
        LiveRead::Absent
    );
    assert_eq!(
        proc_read_failure(&errno(rustix::io::Errno::SRCH), "ns"),
        LiveRead::Absent
    );
    let access = errno(rustix::io::Errno::ACCESS);
    assert_eq!(
        proc_read_failure(&access, "stat"),
        LiveRead::Unreadable(format!("stat: {access}"))
    );
    let io = errno(rustix::io::Errno::IO);
    assert_eq!(
        proc_read_failure(&io, "ns"),
        LiveRead::Unreadable(format!("ns: {io}"))
    );
    assert_ne!(
        format!("{access}"),
        format!("{io}"),
        "the fixtures must differ"
    );
}

/// `T07-AP-75c` · a path read is absent only on `NotFound`. The `Host` cleanup and
/// workspace readbacks map these three outcomes; their guards survived shard C as `true`,
/// which would have settled a cleanup from a permission failure.
#[test]
fn a_path_is_absent_only_when_it_is_not_found() {
    use std::io::{Error as IoError, ErrorKind};
    assert_eq!(presence(&Ok(())), Presence::Present);
    assert_eq!(
        presence(&Err(IoError::from(ErrorKind::NotFound))),
        Presence::Absent
    );
    for kind in [
        ErrorKind::PermissionDenied,
        ErrorKind::NotADirectory,
        ErrorKind::Other,
    ] {
        assert_eq!(
            presence(&Err(IoError::from(kind))),
            Presence::Unreadable,
            "{kind:?} was read as absence"
        );
    }
}

/// `T07-AP-76` · `Host::attach` holds a handle only on a process whose live
/// identity equals the pinned one in BOTH dimensions. A real child is pinned
/// with its own start ticks and namespace; the same PID with either coordinate
/// altered is refused, and only the exact pair attaches.
#[test]
fn host_attach_refuses_a_pid_whose_identity_moved_in_either_dimension() {
    let peer = Peer::spawn(true);
    let observed: ProcessIdentity = serde_json::from_str(&peer.identity_text()).unwrap();
    assert_eq!(observed.pid, peer.pid);
    let subject = Subject {
        task: TASK,
        attempt: ATTEMPT,
        generation: 1,
        workspace_ref: None,
        session: None,
    };
    let mut host = Host::new(deadline());
    let ticks_moved = ProcessIdentity {
        start_ticks: observed.start_ticks + 1,
        ..observed.clone()
    };
    let namespace_moved = ProcessIdentity {
        namespace: "pid:[1]".into(),
        ..observed.clone()
    };
    for moved in [&ticks_moved, &namespace_moved] {
        let refusal = host.attach(&subject, moved).unwrap_err();
        assert!(
            refusal.starts_with("identity changed under the handle"),
            "{refusal}"
        );
        assert!(host.attached().is_empty(), "a refused attach kept a handle");
    }
    host.attach(&subject, &observed).unwrap();
    assert_eq!(host.attached(), vec![ATTEMPT]);
    drop(peer);
}

/// `T07-AP-77` · the workspace walk's DEPTH bound is a bound on the recursion,
/// and it is reached by descending: sixteen levels — the bound exactly — are
/// read back as writable, eighteen are refused. The two fixtures differ only in
/// depth, and the shallow one sits ON the bound so that the comparison itself
/// is what separates them.
#[test]
fn workspace_walk_refuses_past_its_depth_bound() {
    let area = Area::new("depth");
    let build = |name: &str, levels: usize| {
        let root = area.path.join(name);
        DirBuilder::new().mode(0o700).create(&root).unwrap();
        let mut path = root.clone();
        for level in 0..levels {
            path = path.join(format!("d{level}"));
            fs::create_dir(&path).unwrap();
        }
        fs::write(path.join("leaf"), b"1234").unwrap();
        root
    };
    let subject = Subject {
        task: TASK,
        attempt: ATTEMPT,
        generation: 1,
        workspace_ref: None,
        session: None,
    };
    let mut host = Host::new(deadline());
    // Sixteen nested levels put the deepest read at depth 16 — exactly ON the
    // bound, which is the only place a `>` and a `>=` differ. A shallower
    // fixture passes under both and pins nothing (found by planting `>=`).
    host.workspaces.insert(ATTEMPT.into(), build("shallow", 16));
    assert_eq!(
        host.workspace(&subject),
        WorkspaceReadback::Writable { bytes: 4 },
        "sixteen nested levels are exactly on the bound and must still read back"
    );
    let mut host = Host::new(deadline());
    host.workspaces.insert(ATTEMPT.into(), build("deep", 18));
    assert_eq!(
        host.workspace(&subject),
        WorkspaceReadback::NotRead,
        "eighteen nested levels must refuse, not report a size"
    );
}

/// `T07-AP-78` · the walk's ENTRY bound is a bound on entries actually counted,
/// and it is reached by counting them: 4096 entries are read back as writable,
/// 4097 are refused. The two fixtures differ by one file.
#[test]
fn workspace_walk_refuses_past_its_entry_bound() {
    let area = Area::new("entries");
    let build = |name: &str, files: usize| {
        let root = area.path.join(name);
        DirBuilder::new().mode(0o700).create(&root).unwrap();
        for index in 0..files {
            fs::write(root.join(format!("f{index}")), b"1").unwrap();
        }
        root
    };
    let subject = Subject {
        task: TASK,
        attempt: ATTEMPT,
        generation: 1,
        workspace_ref: None,
        session: None,
    };
    let mut host = Host::new(deadline());
    host.workspaces.insert(ATTEMPT.into(), build("at", 4096));
    assert_eq!(
        host.workspace(&subject),
        WorkspaceReadback::Writable { bytes: 4096 },
        "exactly 4096 entries is inside the bound"
    );
    let mut host = Host::new(deadline());
    host.workspaces.insert(ATTEMPT.into(), build("over", 4097));
    assert_eq!(
        host.workspace(&subject),
        WorkspaceReadback::NotRead,
        "4097 entries must refuse, not report a size"
    );
}

/// T07-AP-79 · a ledger that changed between the inspection read and the writable read is
/// refused as `Changed`, never acted on. The branch in `run` is reachable only through a
/// concurrent writer, so its rule was extracted (review §3: "add the test reaching
/// startup.rs:817-818"); this reaches it with values, from both sides.
#[test]
fn a_ledger_that_changed_between_reads_is_refused() {
    assert!(startup::confirm_unchanged(&["task-a", "gen-1"], &["task-a", "gen-1"]).is_ok());
    for reread in [["task-a", "gen-2"], ["task-b", "gen-1"]] {
        assert!(
            matches!(
                startup::confirm_unchanged(&["task-a", "gen-1"], &reread),
                Err(startup::Error::Changed)
            ),
            "{reread:?} was accepted as unchanged"
        );
    }
}

const KILL_MARKER: &str = "HEE3-T07-AP80-IN-EFFECT";
const KILL_CHILD_ENV: &str = "HEE3_T07_KILL_CHILD_STORE";

/// The child half of T07-AP-80. Inert unless the parent names a store through the
/// environment; then it runs one real startup pass whose cleanup effect blocks after the
/// intent row is durable, and waits to be killed.
#[test]
fn kill_child_entrypoint() {
    let Ok(root) = std::env::var(KILL_CHILD_ENV) else {
        return;
    };
    let root = PathBuf::from(root);
    let mut world = World::new().with_obligations(&["workspace"]);
    world.block_in_clean = true;
    let _ = startup::run(
        &Startup {
            root: &root,
            generation: id(GEN),
            epoch: id(EPOCH),
            limits: limits(),
            restored_from: None,
            cursors: &[],
            claims: &[],
            deadline: deadline(),
        },
        &mut world,
    );
}

/// T07-AP-80 · obligation 6 under a REAL kill (F132: only a real writer under a real kill can
/// show "recorded before"). A child process runs a startup pass and is killed with `SIGKILL` inside the
/// cleanup effect, after the intent row is journaled. The intent survives, alone, with no
/// readback; the next pass performs the effect once and journals its readback, and the intent is
/// not written twice.
#[test]
fn a_pass_killed_inside_its_effect_recovers_without_a_duplicate_intent()
-> Result<(), Box<dyn std::error::Error>> {
    let mut r = Rig::ready();
    r.close();
    let before = r.area.events();
    let mut child = std::process::Command::new(std::env::current_exe()?)
        .args([
            "kill_child_entrypoint",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(KILL_CHILD_ENV, r.area.store())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().ok_or("the child has no stdout")?;
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        use std::io::BufRead as _;
        for line in std::io::BufReader::new(stdout)
            .lines()
            .map_while(Result::ok)
        {
            if line.contains(KILL_MARKER) {
                let _ = tx.send(());
                return;
            }
        }
    });
    let budget = std::time::Duration::from_secs(30);
    let reached = rx.recv_timeout(budget);
    child.kill()?;
    let status = child.wait()?;
    reader.join().map_err(|_| "the reader thread panicked")?;
    assert!(
        reached.is_ok(),
        "the child never reached the effect within {budget:?}"
    );
    assert!(
        !status.success(),
        "the child was killed, not finished: {status:?}"
    );

    assert_eq!(
        r.area.events(),
        before + 1,
        "the intent was journaled before the kill"
    );
    let rows = r.records();
    assert_eq!(rows.len(), 1, "the intent alone survives the kill");
    assert_eq!(body(&rows[0])["intent"], "cleanup_performed");

    let mut world = World::new().with_obligations(&["workspace"]);
    r.pass(&mut world);
    assert_eq!(
        world.calls_of("clean").len(),
        1,
        "the effect is performed once"
    );
    let rows = r.records();
    let intents = rows
        .iter()
        .filter(|row| body(row)["intent"] == "cleanup_performed")
        .count();
    assert_eq!(intents, 1, "the intent is not written twice: {rows:?}");
    assert!(
        rows.iter()
            .any(|row| body(row)["readback"]["cleanup_readback"] == "complete"),
        "the resumed pass journals its complete readback"
    );
    Ok(())
}

// ---- store lock and workspace removal: T07's flake and review N6 ------------------------------

/// T07-AP-81 · a store lock held for a moment by another holder is waited out, not refused.
/// `flock` belongs to the open file description, so a fork anywhere in the process duplicates
/// the lock descriptor until the child execs; this battery failed 3 of 20 runs multi-threaded
/// with `Locked` (0 of 20 single-threaded) until acquisition retried for `LOCK_SETTLE` (F167).
#[test]
fn a_momentarily_held_store_lock_is_waited_out() -> Result<(), Box<dyn std::error::Error>> {
    let area = Area::new("lock-wait");
    DirBuilder::new().mode(0o700).create(area.store())?;
    drop(
        Store::open(&area.store(), id(GEN), id(EPOCH), true, deadline())
            .map_err(|error| format!("{error:?}"))?,
    );
    let holder = fs::File::open(area.store().join("store.lock"))?;
    holder.lock()?;
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(40));
        drop(holder);
    });
    let opened = Store::open(&area.store(), id(GEN), id(EPOCH), false, deadline());
    release.join().map_err(|_| "holder thread panicked")?;
    assert!(opened.is_ok(), "{:?}", opened.err());
    Ok(())
}

/// T07-AP-82 · a genuine second writer is still refused, within `LOCK_SETTLE` of trying and well
/// before the caller's deadline: an operator's duplicate start must fail fast.
#[test]
fn a_store_lock_held_past_the_settle_window_refuses_well_before_the_deadline()
-> Result<(), Box<dyn std::error::Error>> {
    let area = Area::new("lock-refuse");
    DirBuilder::new().mode(0o700).create(area.store())?;
    let _first = Store::open(&area.store(), id(GEN), id(EPOCH), true, deadline())
        .map_err(|error| format!("{error:?}"))?;
    let started = Instant::now();
    assert!(matches!(
        Store::open(&area.store(), id(GEN), id(EPOCH), false, deadline()),
        Err(StoreError::Locked)
    ));
    let waited = started.elapsed();
    assert!(
        waited >= LOCK_SETTLE && waited < Duration::from_secs(2),
        "refused after {waited:?}; settle window {LOCK_SETTLE:?}, deadline 10 s"
    );
    Ok(())
}

/// T07-AP-83 · the workspace owner removes a nested workspace whole, descriptor-relative, and a
/// symlink inside it is unlinked as a link: its target survives (review N6).
#[test]
fn the_workspace_owner_removes_a_tree_without_following_links()
-> Result<(), Box<dyn std::error::Error>> {
    let area = Area::new("remove-tree");
    let outside = area.path.join("outside");
    let root = area.path.join("workspace");
    for dir in [&outside, &root, &root.join("a"), &root.join("a/b")] {
        DirBuilder::new().mode(0o700).create(dir)?;
    }
    fs::write(outside.join("keep"), b"keep")?;
    fs::write(root.join("a/b/deep"), b"x")?;
    fs::write(root.join("top"), b"y")?;
    std::os::unix::fs::symlink(&outside, root.join("a/link"))?;
    remove_owned(&root, deadline()).map_err(|error| format!("{error:?}"))?;
    assert!(
        fs::symlink_metadata(&root).is_err(),
        "the workspace is gone"
    );
    assert_eq!(fs::read(outside.join("keep"))?, b"keep");
    Ok(())
}

/// T07-AP-84 · a final component that is a symlink is refused, and its target is untouched.
#[test]
fn the_workspace_owner_refuses_a_symlinked_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let area = Area::new("remove-alias");
    let target = area.path.join("target");
    DirBuilder::new().mode(0o700).create(&target)?;
    fs::write(target.join("keep"), b"keep")?;
    std::os::unix::fs::symlink(&target, area.path.join("alias"))?;
    assert!(matches!(
        remove_owned(&area.path.join("alias"), deadline()),
        Err(WorkspaceError::Type)
    ));
    assert_eq!(fs::read(target.join("keep"))?, b"keep");
    Ok(())
}

/// T07-AP-85 · a directory not private to its owner is refused as custody and left intact.
#[test]
fn the_workspace_owner_refuses_a_directory_that_is_not_0700()
-> Result<(), Box<dyn std::error::Error>> {
    let area = Area::new("remove-shared");
    let root = area.path.join("shared");
    DirBuilder::new().mode(0o700).create(&root)?;
    fs::write(root.join("keep"), b"keep")?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755))?;
    assert!(matches!(
        remove_owned(&root, deadline()),
        Err(WorkspaceError::Custody)
    ));
    assert_eq!(fs::read(root.join("keep"))?, b"keep");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
