//! B14a-1c · `app::runtime::dispatch` over a real ledger: the attempt lifecycle driven end to end
//! with a scripted candidate source and a verifier double, each recording what it was handed
//! (F101), and every verdict read back from the ledger rather than from the runtime's answer.
//!
//! Independent values: the refused candidate's subject is coreutils `sha256sum` of its bytes; the
//! event kinds, states and stop reasons are the store's and the design's (B14a-R1..R5) names. The
//! declared workspace digests are taken from `Snapshot::capture` — P2a pinned `content_digest`
//! against coreutils, and no verdict here is about the digest itself.

use habitat_engine::actions::control::{TaskRequest, Tasks};
use habitat_engine::app::class_profile::{self, Profile};
use habitat_engine::app::live_verifier::LiveVerifier;
use habitat_engine::app::runtime::{
    CHECK_TEARDOWN, Candidate, CandidateSource, CheckPlan, CheckWindow, Dispatch,
    Error as RuntimeError, Observed, Outcome, Previous, Refusal, U64_CHECK_SCHEMA, Verifier,
    dispatch,
};
use habitat_engine::app::tasks::StoreTasks;
use habitat_engine::app::workload::{self, Outcome as RunOutcome, Run};
use habitat_engine::check::consistency::U64_CRITERIA;
use habitat_engine::check::u64_oracle::Evaluation;
use habitat_engine::contracts::control::{
    CancelReason, Precondition, ResourceKind, criteria_digest,
};
use habitat_engine::contracts::roster::{
    Availability, Kind, Locality, ObservationInput, ObservationSource, RosterDefinitionV1,
    Selection, Update,
};
use habitat_engine::contracts::{Sha256Digest, UuidV4};
use habitat_engine::store::{
    Allocation, Principal, RequestSource, Store, Submission, VerificationVerdict,
};
use habitat_engine::task::control::Cancel;
use habitat_engine::task::driver::{Outcome as Driven, StopReason};
use habitat_engine::worker::resources::Scope;
use habitat_engine::worker::workspace::Snapshot;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::tasks::{EPOCH, GENERATION, Scratch};

type Outcome_ = Result<(), Box<dyn Error>>;
/// What the candidate source was handed, per request.
type Asked = Rc<RefCell<Vec<Option<Previous>>>>;
/// What the verifier was handed, per check: the snapshot's content digest, its editable bytes,
/// the check window, and when the double was called (a model, not a script: F101).
type Handed = Rc<RefCell<Vec<(String, Vec<u8>, CheckWindow, Instant)>>>;

const TASK: &str = "28f10000-0000-4000-8000-000000000001";
const KEY: &str = "28f10000-0000-4000-8000-000000000002";
const ADMITTED: &str = "28f10000-0000-4000-8000-000000000003";
const WORKSPACE: &str = "28f10000-0000-4000-8000-000000000004";
const ROSTER_KEY: &str = "28f10000-0000-4000-8000-000000000005";
const OTHER_WORKSPACE: &str = "28f10000-0000-4000-8000-000000000006";
const PROFILE_DIGEST: &str =
    "sha256:5151515151515151515151515151515151515151515151515151515151515151";
const BASE_LIB: &[u8] = b"pub fn parse() -> u8 {\n    0\n}\n";
const FIRST: &[u8] = b"pub fn parse() -> u8 {\n    1\n}\n";
const SECOND: &[u8] = b"pub fn parse() -> u8 {\n    2\n}\n";
/// Not UTF-8: the class refuses it before any file is created.
const REFUSED: &[u8] = b"not utf-8 \xff\n";
/// coreutils: `printf 'not utf-8 \xff\n' | sha256sum`.
const REFUSED_SHA256: &str =
    "sha256:7c51da70eb7f3c6bbdf1eced2239f642230e4369d78a81af9f7da7749bfe3a57";

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}

fn id(value: &str) -> UuidV4<'_> {
    UuidV4::parse(value).unwrap()
}

fn owner() -> Principal {
    Principal::new(1000, "operator").unwrap()
}

/// A task, its ledger, its roster record and its installed workspace.
struct Rig {
    scratch: Scratch,
    tasks: StoreTasks,
    profile: Profile,
    attempts: PathBuf,
    agent: String,
    selections: Vec<Selection>,
    reserved_work_ms: u64,
    teardown_ms: u64,
}

struct Shape<'a> {
    criteria: String,
    work_ms: u64,
    declared: &'a str,
    baseline_digest: Option<&'a str>,
    protected_digest: Option<&'a str>,
    /// A workspace directory removed after its digest is declared, so its capture fails.
    removed: Option<&'static str>,
    teardown_ms: u64,
    verify_ms: u64,
}

impl Default for Shape<'_> {
    fn default() -> Self {
        Self {
            criteria: criteria_digest(&U64_CRITERIA),
            work_ms: 600_000,
            declared: WORKSPACE,
            baseline_digest: None,
            protected_digest: None,
            removed: None,
            teardown_ms: 1_000,
            verify_ms: 300_000,
        }
    }
}

fn private(path: &Path) -> Result<(), Box<dyn Error>> {
    DirBuilder::new().mode(0o700).create(path)?;
    Ok(())
}

fn file(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    use std::io::Write as _;
    let mut handle = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    handle.write_all(bytes)?;
    Ok(())
}

/// The roster record the attempt runs under, enabled and proven through the roster's doors.
fn roster(
    store: &mut Store,
    principal: &Principal,
) -> Result<(String, Vec<Selection>), Box<dyn Error>> {
    let input = Update {
        idempotency_key: ROSTER_KEY.to_owned(),
        record_id: None,
        expected_revision: None,
        definition: RosterDefinitionV1 {
            kind: Kind::Agent,
            display_name: "runtime-fixture".to_owned(),
            owner_id: "fixture-worker".to_owned(),
            version: "v1".to_owned(),
            capabilities: vec!["text".to_owned()],
            locality: Locality::Local,
            endpoint_ref: Some(ROSTER_KEY.to_owned()),
            limitations: "scripted candidate source, no worker".to_owned(),
        },
        audit_reason: "B14a-1c runtime fixture".to_owned(),
    };
    let raw = serde_json::to_vec(&serde_json::json!({
        "protocol":"hee3.control","version":1,"kind":"request","request_id":ROSTER_KEY,
        "action":"roster.update","action_version":1,"idempotency_key":ROSTER_KEY,
        "deadline_unix_ms":"1030000","authority":{"grant_id":ROSTER_KEY,"scope_sha256":PROFILE_DIGEST},
        "precondition":null,
        "body":{"record_id":null,"definition":input.definition,"audit_reason":input.audit_reason}
    }))?;
    let head = store
        .roster_apply(principal, &[input], RequestSource::Native(&raw), deadline())
        .map_err(|error| format!("{error:?}"))?
        .remove(0)
        .head;
    store
        .roster_observe_worker(
            principal,
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
                actual_identity: Some("fixture/worker".to_owned()),
                immutable_revision: None,
                capabilities: vec!["text".to_owned()],
                evidence_ref: ROSTER_KEY.to_owned(),
            },
            deadline(),
        )
        .map_err(|error| format!("{error:?}"))?;
    let selections = vec![Selection {
        record_id: head.record_id.clone(),
        expected_revision: head.record_version,
        capabilities: vec!["text".to_owned()],
        local_only: true,
        version: Some("v1".to_owned()),
        ttl_ms: 60_000,
    }];
    Ok((head.record_id, selections))
}

/// The installed workspace: a baseline holding the class's editable file, and a protected tree,
/// declared by a composed profile.
fn installed(root: &Path, shape: &Shape<'_>) -> Result<Profile, Box<dyn Error>> {
    let class = root.join("class");
    private(&class)?;
    let (base, protected) = (class.join("base"), class.join("protected"));
    private(&base)?;
    private(&base.join("src"))?;
    file(&base.join("src/lib.rs"), BASE_LIB)?;
    private(&protected)?;
    file(&protected.join("oracle.txt"), b"frozen oracle\n")?;
    // The class's real protected tree, so the live verifier's workload passes its preflight (R15):
    // the frozen oracle and the public wrapper the link stage mounts.
    file(
        &protected.join("oracle.json"),
        include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/oracle/cases.json"),
    )?;
    file(
        &protected.join("public-wrapper.rs"),
        include_bytes!("../evaluation/harnesses/u64-public-wrapper.rs"),
    )?;
    let digest_of = |root: &Path| -> Result<String, Box<dyn Error>> {
        Snapshot::capture(root, &[], deadline())
            .map_err(|error| format!("{error:?}"))?
            .content_digest()
            .ok_or_else(|| "no content digest".into())
    };
    let base_digest = digest_of(&base)?;
    let protected_digest = digest_of(&protected)?;
    let zero = format!("sha256:{}", "0".repeat(64));
    let text = format!(
        "schema = \"hee3.class-profile/1\"\nclass = \"rust-library-change/1\"\n\n\
         [[workspace]]\nid = \"{}\"\nbaseline = \"base\"\nbaseline_digest = \"{}\"\n\
         protected = \"protected\"\nprotected_digest = \"{}\"\n\n\
         [pins]\ncompiler = {{ host = \"/opt/rustc\", sha256 = \"{zero}\" }}\n\
         shim = {{ host = \"/opt/shim\", sha256 = \"{zero}\" }}\nruntime_files = []\n\
         namespace_directories = []\nbusctl_sha256 = \"{zero}\"\nsystemd_run_sha256 = \"{zero}\"\n[reviewed]\nexpectation = {{ artifact_id = \"28f70000-0000-4000-8000-00000000000e\", sha256 = \"sha256:7a1f9aa11864adf4fdc42e57bccf14c9721c288f749cf314525c8c183df64f13\", byte_length = 23, media_type = \"application/json\", schema_id = \"hee3.receipt/1:ExpectationV1\" }}\nreview = {{ artifact_id = \"28f70000-0000-4000-8000-00000000000f\", sha256 = \"sha256:7676d865aaf08640fa14c6526535f9e8e47da1a955bdf479688c5f3800423740\", byte_length = 23, media_type = \"application/json\", schema_id = \"hee3.receipt/1:ReviewV1\" }}\n",
        shape.declared,
        shape.baseline_digest.unwrap_or(&base_digest),
        shape.protected_digest.unwrap_or(&protected_digest),
    );
    if let Some(name) = shape.removed {
        fs::remove_dir_all(class.join(name))?;
    }
    Ok(Profile {
        declared: class_profile::compose(text.as_bytes()).map_err(|error| format!("{error:?}"))?,
        directory: class,
        digest: PROFILE_DIGEST.to_owned(),
    })
}

fn rig(shape: &Shape<'_>) -> Result<Rig, Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let state = scratch.0.join("state");
    private(&state)?;
    let mut store = Store::open(
        &state,
        UuidV4::parse(GENERATION)?,
        UuidV4::parse(EPOCH)?,
        true,
        deadline(),
    )
    .map_err(|error| format!("{error:?}"))?;
    let principal = owner();
    let (agent, selections) = roster(&mut store, &principal)?;
    store
        .submit(
            Submission {
                principal: &principal,
                key: id(KEY),
                task: id(TASK),
                event: id(ADMITTED),
                request_bytes: b"B14a-1c runtime fixture",
                criteria: Sha256Digest::parse(&shape.criteria)?,
                allocation: Allocation {
                    limit_ms: 1_200_000,
                    work_ms: shape.work_ms,
                    verify_ms: shape.verify_ms,
                },
                workspace_id: id(WORKSPACE),
            },
            deadline(),
        )
        .map_err(|error| format!("{error:?}"))?;
    let profile = installed(&scratch.0, shape)?;
    let attempts = scratch.0.join("attempts");
    private(&attempts)?;
    Ok(Rig {
        tasks: StoreTasks::new(store, EPOCH.to_owned()),
        scratch,
        profile,
        attempts,
        agent,
        selections,
        reserved_work_ms: shape.work_ms,
        teardown_ms: shape.teardown_ms,
    })
}

/// A candidate source that answers from a script and records every `previous` it was handed.
struct Script<'h> {
    answers: VecDeque<Candidate>,
    seen: Asked,
    hook: Option<Box<dyn FnMut() + 'h>>,
}

impl CandidateSource for Script<'_> {
    fn next(&mut self, previous: Option<&Previous>) -> Candidate {
        self.seen.borrow_mut().push(previous.cloned());
        if let Some(hook) = self.hook.as_mut() {
            hook();
        }
        self.answers.pop_front().unwrap_or(Candidate::Exhausted)
    }
}

/// What the verifier double observes for one check (a model, not a script — F101): the run's
/// outcome, how long the run took from the window's origin, and whether its cleanup settled.
struct Answer {
    run: Result<RunOutcome, workload::Error>,
    elapsed: Duration,
    cleanup_pending: bool,
}

/// A verifier that answers from a script of observations and records the content digest of
/// every snapshot it was handed, the editable file's bytes in it, the window and when it was called.
struct Oracle<'h> {
    answers: VecDeque<Answer>,
    seen: Handed,
    hook: Option<Box<dyn FnMut() + 'h>>,
}

impl Verifier for Oracle<'_> {
    fn check(&mut self, plan: CheckPlan<'_>) -> Observed {
        let editable = plan
            .subject
            .entries()
            .find(|entry| entry.path == "src/lib.rs")
            .and_then(|entry| match &entry.content {
                habitat_engine::worker::workspace::Content::File { bytes, .. } => {
                    Some(bytes.clone())
                }
                habitat_engine::worker::workspace::Content::Directory => None,
            })
            .unwrap_or_default();
        self.seen.borrow_mut().push((
            plan.subject.content_digest().unwrap_or_default(),
            editable,
            plan.window,
            Instant::now(),
        ));
        if let Some(hook) = self.hook.as_mut() {
            hook();
        }
        let answer = self.answers.pop_front().unwrap_or(Answer {
            run: Ok(RunOutcome::SetupFailed),
            elapsed: Duration::from_millis(1),
            cleanup_pending: false,
        });
        let observed = plan.window.begun + answer.elapsed;
        let run = answer.run.map(|outcome| {
            let decisive = matches!(
                outcome,
                RunOutcome::Matched(_) | RunOutcome::Mismatch(_) | RunOutcome::InvalidOutput(_)
            )
            .then(|| observed.checked_sub(Duration::from_millis(1)))
            .flatten();
            let mut run = Run::unlaunched(outcome);
            run.process_cleanup_complete = !answer.cleanup_pending;
            run.decisive = decisive;
            run
        });
        Observed { run, observed }
    }
}

/// A run the oracle matched in full, observed after `elapsed_ms`.
fn matched(elapsed_ms: u64) -> Answer {
    Answer {
        run: Ok(RunOutcome::Matched(Evaluation {
            vectors: Vec::new(),
            matched: 1,
            failed: 0,
        })),
        elapsed: Duration::from_millis(elapsed_ms),
        cleanup_pending: false,
    }
}

/// A run the oracle refused, observed after `elapsed_ms`.
fn mismatched(elapsed_ms: u64) -> Answer {
    Answer {
        run: Ok(RunOutcome::Mismatch(Evaluation {
            vectors: Vec::new(),
            matched: 0,
            failed: 1,
        })),
        elapsed: Duration::from_millis(elapsed_ms),
        cleanup_pending: false,
    }
}

/// A run the workload cancelled.
fn cancelled_run() -> Answer {
    Answer {
        run: Ok(RunOutcome::Cancelled),
        elapsed: Duration::from_millis(7),
        cleanup_pending: false,
    }
}

fn script<'h>(answers: Vec<Candidate>) -> (Script<'h>, Asked) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    (
        Script {
            answers: answers.into(),
            seen: Rc::clone(&seen),
            hook: None,
        },
        seen,
    )
}

fn oracle<'h>(answers: Vec<Answer>) -> (Oracle<'h>, Handed) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    (
        Oracle {
            answers: answers.into(),
            seen: Rc::clone(&seen),
            hook: None,
        },
        seen,
    )
}

fn run<C: CandidateSource, V: Verifier>(
    rig: &Rig,
    principal: &Principal,
    source: C,
    verifier: V,
    capture_ms: u64,
) -> Result<Outcome, RuntimeError> {
    dispatch(
        &rig.tasks,
        &rig.profile,
        Dispatch {
            principal,
            task: id(TASK),
            agent_record_id: &rig.agent,
            selections: &rig.selections,
            attempts: &rig.attempts,
            forbidden: &[],
            teardown_ms: rig.teardown_ms,
            capture_ms,
        },
        source,
        verifier,
    )
}

/// The previous check as the candidate source was handed it: its verdict, no criteria, and the
/// runtime's own evidence (`u64_check`) naming `outcome` and citing four records.
fn assert_previous(
    previous: Option<&Option<Previous>>,
    verdict: VerificationVerdict,
    outcome: &str,
) -> Result<(), Box<dyn Error>> {
    let previous = previous
        .and_then(Option::as_ref)
        .ok_or("a previous check")?;
    assert_eq!((previous.verdict, previous.criteria), (verdict, 0));
    let evidence: serde_json::Value = serde_json::from_slice(&previous.evidence)?;
    assert_eq!(evidence["kind"], "u64_check");
    assert_eq!(evidence["outcome"], outcome);
    assert_eq!(
        evidence["records"].as_object().map(serde_json::Map::len),
        Some(4)
    );
    Ok(())
}

/// A published object's bytes, read from the store's content-addressed directory by digest.
fn object_bytes(rig: &Rig, digest: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let hex = digest.strip_prefix("sha256:").ok_or("a sha256: digest")?;
    Ok(fs::read(
        rig.scratch
            .0
            .join("state/generations")
            .join(GENERATION)
            .join("objects/sha256")
            .join(&hex[..2])
            .join(hex),
    )?)
}

/// A published JSON object, decoded.
fn object_json(rig: &Rig, digest: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&object_bytes(rig, digest)?)?)
}

/// The run records the ledger committed with the task's verifications, in kind order:
/// `(kind, artifact_id, digest, size, verdict)`.
fn committed_records(rig: &Rig) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    rows(
        rig,
        "SELECT r.kind,r.artifact_id,r.digest,f.size,v.verdict FROM attempt_records r \
         JOIN artifacts f ON f.digest=r.digest JOIN verifications v ON v.event_id=r.event_id \
         JOIN attempts a ON a.id=r.attempt_id WHERE a.task_id=? \
         ORDER BY CAST(a.generation AS INTEGER), r.kind",
    )
}

/// Three bounded scopes whose systemd-run pin is wrong, as t06 uses them: the launcher refuses
/// before any process starts, so a live verifier reaches `LauncherFailed` in the gate.
const BAD_PIN: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
fn bad_pin_scopes() -> [Scope; 3] {
    [
        "28f10000-0000-4000-8000-0000000000e1",
        "28f10000-0000-4000-8000-0000000000e2",
        "28f10000-0000-4000-8000-0000000000e3",
    ]
    .map(|id| Scope {
        systemd_run: "/usr/bin/systemd-run".into(),
        systemd_run_sha256: BAD_PIN.into(),
        runtime_dir: format!("/run/user/{}", rustix::process::geteuid().as_raw()).into(),
        run_id: id.into(),
        aggregate: "hee3boundedcontrols.slice".into(),
    })
}

/// The ledger, read on its own connection.
fn ledger(rig: &Rig) -> Result<rusqlite::Connection, Box<dyn Error>> {
    Ok(rusqlite::Connection::open_with_flags(
        rig.scratch
            .0
            .join("state/generations")
            .join(GENERATION)
            .join("ledger.sqlite3"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}

fn state(rig: &Rig) -> Result<String, Box<dyn Error>> {
    Ok(
        ledger(rig)?.query_row("SELECT state FROM tasks WHERE id=?", [TASK], |row| {
            row.get(0)
        })?,
    )
}

fn rows(rig: &Rig, sql: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let db = ledger(rig)?;
    let mut statement = db.prepare(sql)?;
    let width = statement.column_count();
    let found = statement
        .query_map([TASK], |row| {
            (0..width)
                .map(|index| {
                    row.get::<_, Option<rusqlite::types::Value>>(index)
                        .map(|value| match value {
                            None | Some(rusqlite::types::Value::Null) => "NULL".to_owned(),
                            Some(rusqlite::types::Value::Integer(n)) => n.to_string(),
                            Some(rusqlite::types::Value::Text(text)) => text,
                            Some(other) => format!("{other:?}"),
                        })
                })
                .collect::<Result<Vec<_>, _>>()
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(found)
}

/// Verifications of the task's attempts in attempt order: verdict, subject, `used_ms`.
fn verifications(rig: &Rig) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    rows(
        rig,
        "SELECT v.verdict,v.subject_digest,v.used_ms,v.evidence_media_type,v.evidence_schema_id,\
         v.satisfied_criteria FROM verifications v JOIN attempts a \
         ON a.id=v.attempt_id WHERE a.task_id=? ORDER BY CAST(a.generation AS INTEGER)",
    )
}

/// R14 · each check was handed its own window: begun after its candidate was requested and no later
/// than the call; the cutoff is the verify reservation the ledger held THEN less the teardown share
/// (`reserved[i]`: 300,000 ms for a first check, less the first check's cost for the second, so a
/// window read once at dispatch cannot pass); the teardown share runs to the cutoff plus
/// `CHECK_TEARDOWN` (the task deadline is far).
fn assert_windows(
    handed: &[(String, Vec<u8>, CheckWindow, Instant)],
    requested: &[Instant],
    reserved: [u64; 2],
) {
    assert_eq!((handed.len(), requested.len()), (2, 2));
    let teardown_ms = u64::try_from(CHECK_TEARDOWN.as_millis()).unwrap_or(u64::MAX);
    for (index, reserved_ms) in reserved.into_iter().enumerate() {
        let (_, _, window, called) = &handed[index];
        assert!(
            requested[index] <= window.begun && window.begun <= *called,
            "check {index}"
        );
        assert_eq!(
            window.until.duration_since(window.begun),
            Duration::from_millis(reserved_ms - teardown_ms),
            "check {index}"
        );
        assert_eq!(
            window.teardown_until,
            window.until + CHECK_TEARDOWN,
            "check {index}"
        );
    }
}

/// B14a-1c · fail → `repair_pending` → verifying → accepted. Two attempts, each bound to the same
/// three digests; each verification's subject is what the verifier double was handed; the second
/// candidate request was handed the first check's verdict and evidence.
#[test]
fn a_failed_check_is_repaired_and_the_second_attempt_is_accepted() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (mut source, asked) = script(vec![
        Candidate::Replacement(FIRST.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    // When each candidate was requested: after its attempt began, before its check's window.
    let requested: Rc<RefCell<Vec<Instant>>> = Rc::new(RefCell::new(Vec::new()));
    let requested_at = Rc::clone(&requested);
    source.hook = Some(Box::new(move || {
        requested_at.borrow_mut().push(Instant::now());
    }));
    let (verifier, handed) = oracle(vec![mismatched(7), matched(7)]);
    let principal = owner();
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert_eq!(state(&rig)?, "accepted");
    let handed = handed.borrow();
    assert_eq!(
        handed
            .iter()
            .map(|(_, bytes, _, _)| bytes.as_slice())
            .collect::<Vec<_>>(),
        [FIRST, SECOND],
        "the verifier was handed each applied candidate, in order"
    );
    assert_ne!(handed[0].0, handed[1].0);
    // R14 · each check was handed its own window (asserted whole in `assert_windows`).
    assert_windows(&handed, &requested.borrow(), [300_000, 300_000 - 7]);
    assert_eq!(
        rows(
            &rig,
            "SELECT state,effect,cleanup,used_ms IS NOT NULL FROM attempts WHERE task_id=? \
             ORDER BY CAST(generation AS INTEGER)",
        )?,
        vec![
            vec![
                "settled".to_owned(),
                "none".to_owned(),
                "settled".to_owned(),
                "1".to_owned()
            ],
            vec![
                "settled".to_owned(),
                "none".to_owned(),
                "settled".to_owned(),
                "1".to_owned()
            ],
        ],
        "both attempts settled with a known cost and no effect"
    );
    assert_eq!(
        verifications(&rig)?,
        vec![
            // B09b: each row carries the schema the verifier named and the criteria it satisfied.
            vec![
                "failed".to_owned(),
                handed[0].0.clone(),
                "7".to_owned(),
                "application/json".to_owned(),
                U64_CHECK_SCHEMA.to_owned(),
                "0000000000000000".to_owned(),
            ],
            vec![
                "passed".to_owned(),
                handed[1].0.clone(),
                "7".to_owned(),
                "application/json".to_owned(),
                U64_CHECK_SCHEMA.to_owned(),
                "0000000000000001".to_owned(),
            ],
        ]
    );
    // The second request was handed the first check's verdict and its evidence — the runtime's
    // own record of the mismatched run, citing the four records it committed.
    let asked = asked.borrow();
    assert_eq!(asked[0], None);
    assert_previous(asked.get(1), VerificationVerdict::Failed, "mismatch")?;
    let declared = &rig.profile.declared.workspaces[0];
    let bound = vec![
        declared.baseline_digest.clone(),
        declared.protected_digest.clone(),
        PROFILE_DIGEST.to_owned(),
    ];
    assert_eq!(
        rows(
            &rig,
            "SELECT b.baseline_digest,b.protected_digest,b.profile_digest FROM attempt_bindings b \
             JOIN attempts a ON a.id=b.attempt_id WHERE b.task_id=? \
             ORDER BY CAST(a.generation AS INTEGER)",
        )?,
        vec![bound.clone(), bound],
        "both attempts are bound to what they were dispatched on"
    );
    Ok(())
}

/// B14a-1c · exhaustion after begin is truthful (B14a-R2.3): one attempt, never verified, the task
/// stops `failed` as `WorkerFailed`.
#[test]
fn an_exhausted_source_fails_the_task_after_one_attempt() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (source, asked) = script(vec![Candidate::Exhausted]);
    let (verifier, handed) = oracle(vec![]);
    let principal = owner();
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::WorkerFailed))
    );
    assert_eq!(state(&rig)?, "failed");
    assert_eq!(asked.borrow().len(), 1);
    assert!(handed.borrow().is_empty(), "no verifier call");
    assert!(verifications(&rig)?.is_empty());
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["worker_failed".to_owned()]]
    );
    Ok(())
}

/// B14a-1c · a candidate the class refuses is the runtime's own check (B14a-R2.4): recorded
/// `failed`, subject the sha256 of the candidate bytes, at no verifier call; the next attempt is
/// handed that verdict and is accepted.
#[test]
fn a_refused_candidate_is_recorded_failed_without_a_verifier_call() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (source, asked) = script(vec![
        Candidate::Replacement(REFUSED.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    let (verifier, handed) = oracle(vec![matched(7)]);
    let principal = owner();
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert_eq!(
        handed.borrow().len(),
        1,
        "the verifier saw only the second candidate"
    );
    let recorded = verifications(&rig)?;
    // B09b: the runtime's own class check names its evidence by its own schema and records the
    // criteria it satisfied (none); the verifier's check carries the schema the verifier named and
    // its criteria pattern — read from the row, not from the double.
    assert_eq!(
        recorded[0],
        vec![
            "failed".to_owned(),
            REFUSED_SHA256.to_owned(),
            "0".to_owned(),
            "application/json".to_owned(),
            "hee3.refused-candidate/1".to_owned(),
            "0000000000000000".to_owned(),
        ]
    );
    assert_eq!(
        recorded[1][3..],
        [
            "application/json".to_owned(),
            U64_CHECK_SCHEMA.to_owned(),
            "0000000000000001".to_owned(),
        ]
    );
    // B09b: the accepted object is the second check's receipt, under the identity the verification
    // recorded — one id per object across the verification and acceptance doors.
    let bound = rows(
        &rig,
        "SELECT o.artifact_id=v.evidence_artifact_id, o.schema_id, c.manifest_artifact_id=c.event_id \
         FROM acceptance_objects o JOIN acceptances c ON c.event_id=o.event_id \
         JOIN verifications v ON v.attempt_id=c.attempt_id WHERE c.task_id=?",
    )?;
    assert_eq!(
        bound,
        vec![vec![
            "1".to_owned(),
            U64_CHECK_SCHEMA.to_owned(),
            "1".to_owned()
        ]]
    );
    let previous = asked.borrow()[1].clone().ok_or("no previous")?;
    assert_eq!(
        (previous.verdict, previous.criteria),
        (VerificationVerdict::Failed, 0)
    );
    let evidence: serde_json::Value = serde_json::from_slice(&previous.evidence)?;
    assert_eq!(evidence["refusal"], "candidate_encoding");
    assert_eq!(evidence["candidate_sha256"], REFUSED_SHA256);
    assert!(
        fs::read_dir(&rig.attempts)?.count() <= 1,
        "the refused candidate left no retained path"
    );
    Ok(())
}

/// Every pre-dispatch refusal, each reached by one shape or capture share (R2.5 order).
fn refusal_cases<'a>(other_criteria: &'a str, wrong: &'a str) -> [(Shape<'a>, u64, Refusal); 11] {
    [
        (
            Shape {
                work_ms: 0,
                ..Shape::default()
            },
            5_000,
            Refusal::ReservationEmpty,
        ),
        // The boundary itself: 6,000 = 5,000 capture + 1,000 teardown leaves no work time.
        (
            Shape {
                work_ms: 6_000,
                ..Shape::default()
            },
            5_000,
            Refusal::ReservationTooSmall,
        ),
        (
            Shape {
                protected_digest: Some(wrong),
                ..Shape::default()
            },
            5_000,
            Refusal::ProtectedMismatch,
        ),
        (
            Shape {
                removed: Some("base"),
                ..Shape::default()
            },
            5_000,
            Refusal::BaselineCapture,
        ),
        (
            Shape {
                removed: Some("protected"),
                ..Shape::default()
            },
            5_000,
            Refusal::ProtectedCapture,
        ),
        (
            Shape {
                criteria: other_criteria.to_owned(),
                ..Shape::default()
            },
            5_000,
            Refusal::CriteriaNotClass,
        ),
        (Shape::default(), 600_001, Refusal::ReservationTooSmall),
        // R14.2: a verify reservation that holds nothing past the check's teardown share.
        (
            Shape {
                verify_ms: 10_000,
                ..Shape::default()
            },
            5_000,
            Refusal::VerifyReservationTooSmall,
        ),
        // Only the teardown share makes this one too small: 5,500 ≤ 5,000 capture + 1,000 teardown.
        (
            Shape {
                work_ms: 5_500,
                ..Shape::default()
            },
            5_000,
            Refusal::ReservationTooSmall,
        ),
        (
            Shape {
                declared: OTHER_WORKSPACE,
                ..Shape::default()
            },
            5_000,
            Refusal::WorkspaceNotDeclared,
        ),
        (
            Shape {
                baseline_digest: Some(wrong),
                ..Shape::default()
            },
            5_000,
            Refusal::BaselineMismatch,
        ),
    ]
}

/// B14a-1c · each pre-dispatch refusal stops the task by its name through `finish_preparation`,
/// with no attempt row and no source or verifier call (B14a-R1.4d, R1.5, R2.5).
#[test]
fn pre_dispatch_refusals_stop_the_task_before_any_attempt() -> Outcome_ {
    let other_criteria = format!("sha256:{}", "c".repeat(64));
    let wrong = format!("sha256:{}", "d".repeat(64));
    let cases = refusal_cases(&other_criteria, &wrong);
    for (shape, capture_ms, refusal) in cases {
        let rig = rig(&shape)?;
        let (source, asked) = script(vec![Candidate::Replacement(FIRST.to_vec())]);
        let (verifier, handed) = oracle(vec![]);
        let principal = owner();
        let outcome =
            run(&rig, &principal, source, verifier, capture_ms).map_err(|e| format!("{e:?}"))?;
        assert_eq!(outcome, Outcome::Refused(refusal), "{refusal:?}");
        assert_eq!(state(&rig)?, "failed", "{refusal:?}");
        assert!(rows(&rig, "SELECT id FROM attempts WHERE task_id=?")?.is_empty());
        assert!(asked.borrow().is_empty() && handed.borrow().is_empty());
        assert_eq!(
            rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
            vec![vec![refusal.name().to_owned()]],
            "{refusal:?}"
        );
    }
    Ok(())
}

/// Cancel the fixture task through the public task door, at the generation the ledger holds now.
fn cancel(rig: &Rig, principal: &Principal, key: &str) {
    let generation: String = ledger(rig)
        .unwrap()
        .query_row("SELECT generation FROM tasks WHERE id=?", [TASK], |row| {
            row.get(0)
        })
        .unwrap();
    let payload = format!("cancel {key}");
    let now = 1_000_000;
    let outcome = rig.tasks.cancel(
        &TaskRequest {
            principal,
            idempotency_key: key,
            payload: payload.as_bytes(),
            deadline_unix_ms: now + 5_000,
            now_unix_ms: now,
        },
        &Precondition {
            resource: ResourceKind::Task,
            id: TASK.to_owned(),
            generation: generation.parse().unwrap(),
        },
        &Cancel {
            reason: CancelReason::OperatorRequest,
            note: None,
        },
    );
    assert!(outcome.is_ok(), "cancel: {outcome:?}");
}

/// B14a-1c · a cancel during the check: the verifier's return is still recorded, then the driver
/// stops the task `cancelled` before any acceptance. The cancel goes through the public task door
/// while the check runs, so the runtime held no lock across it (a held lock would deadlock here).
#[test]
fn a_cancel_during_the_check_stops_the_task_cancelled() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (mut verifier, _) = oracle(vec![matched(7)]);
    verifier.hook = Some(Box::new(|| {
        cancel(&rig, &principal, "28f10000-0000-4000-8000-0000000000c1");
    }));
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::Cancelled))
    );
    assert_eq!(state(&rig)?, "cancelled");
    assert!(
        rows(
            &rig,
            "SELECT accepted_event FROM tasks WHERE id=? AND accepted_event IS NOT NULL"
        )?
        .is_empty()
    );
    assert_eq!(verifications(&rig)?.len(), 1);
    assert_eq!(verifications(&rig)?[0][0], "passed");
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["durable_cancellation".to_owned()]]
    );
    Ok(())
}

/// B14a-1c · a cancel during execute: the attempt settles, the check is recorded as not started at
/// no cost (B14a-R1.4a), and the task stops `cancelled` with no verifier call.
#[test]
fn a_cancel_during_execute_records_the_check_not_started() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (mut source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    source.hook = Some(Box::new(|| {
        cancel(&rig, &principal, "28f10000-0000-4000-8000-0000000000c2");
    }));
    let (verifier, handed) = oracle(vec![matched(7)]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::Cancelled))
    );
    assert_eq!(state(&rig)?, "cancelled");
    assert!(
        handed.borrow().is_empty(),
        "no verifier call after the cancel"
    );
    let recorded = verifications(&rig)?;
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        (recorded[0][0].as_str(), recorded[0][2].as_str()),
        ("cancelled", "0")
    );
    Ok(())
}

/// B14a-1c · a write from anyone but the runtime, other than one cancellation or an instance
/// observation, is detected at the runtime's next write and refused by name (B14a-R2.1, R3 G1).
/// No public door can write to a dispatching task here, so the second writer is a second
/// connection — the restart overlap the rule exists for. Each case is refused by one clause alone:
/// a foreign kind with its bump; a bump with no event (the generation clause); a real cancel plus a
/// foreign event at the same generation (the kind clause); two cancellations (the at-most-one
/// clause).
#[test]
fn a_second_writer_is_refused_as_a_concurrent_writer() -> Outcome_ {
    const BUMP: &str = "UPDATE tasks SET generation=CAST(CAST(generation AS INTEGER)+1 AS TEXT) \
                        WHERE id='28f10000-0000-4000-8000-000000000001';";
    let foreign = |event: &str, kind: &str| {
        format!(
            "INSERT INTO events(id,task_id,generation,kind,body) SELECT '{event}',id,generation,\
             '{kind}',x'7b7d' FROM tasks WHERE id='{TASK}';"
        )
    };
    let e = [
        "28f10000-0000-4000-8000-0000000000e1",
        "28f10000-0000-4000-8000-0000000000e2",
    ];
    let cases: [(&str, String, bool); 5] = [
        // A foreign event at the current generation, with no bump (review M3).
        (
            "unbumped foreign kind",
            foreign(e[0], "reconciliation_recorded"),
            false,
        ),
        (
            "foreign kind",
            format!("{BUMP}{}", foreign(e[0], "disposition_recorded")),
            false,
        ),
        ("bump alone", BUMP.to_owned(), false),
        (
            "cancel, then a foreign kind",
            foreign(e[0], "disposition_recorded"),
            true,
        ),
        (
            "two cancellations",
            format!(
                "{BUMP}{}{BUMP}{}",
                foreign(e[0], "cancellation_requested"),
                foreign(e[1], "cancellation_requested")
            ),
            false,
        ),
    ];
    for (name, sql, cancel_first) in cases {
        let rig = rig(&Shape::default())?;
        let principal = owner();
        let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
        let (mut verifier, _) = oracle(vec![matched(7)]);
        let path = rig
            .scratch
            .0
            .join("state/generations")
            .join(GENERATION)
            .join("ledger.sqlite3");
        let rig_ref = &rig;
        let principal_ref = &principal;
        verifier.hook = Some(Box::new(move || {
            if cancel_first {
                cancel(
                    rig_ref,
                    principal_ref,
                    "28f10000-0000-4000-8000-0000000000c4",
                );
            }
            rusqlite::Connection::open(&path)
                .unwrap()
                .execute_batch(&sql)
                .unwrap();
        }));
        let outcome = run(&rig, &principal, source, verifier, 5_000);
        assert!(
            matches!(outcome, Err(RuntimeError::ConcurrentWriter)),
            "{name}: {outcome:?}"
        );
        assert!(
            verifications(&rig)?.is_empty(),
            "{name}: nothing written after the foreign write"
        );
    }
    Ok(())
}

/// B14a-1c · a settle measured past what the reservation holds is an unknown cost, never a clean
/// failure: `used_ms` is not recorded, the task waits `effect_unknown`, and the stop needs
/// settlement.
#[test]
fn an_overrun_is_recorded_as_an_unknown_cost() -> Outcome_ {
    let rig = rig(&Shape {
        work_ms: 40,
        teardown_ms: 0,
        ..Shape::default()
    })?;
    let principal = owner();
    let (mut source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    source.hook = Some(Box::new(|| std::thread::sleep(Duration::from_millis(80))));
    let (verifier, handed) = oracle(vec![]);
    let outcome = run(&rig, &principal, source, verifier, 10).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::NeedsSettlement(StopReason::Unsettled))
    );
    assert_eq!(state(&rig)?, "effect_unknown");
    assert_eq!(
        rows(&rig, "SELECT used_ms FROM attempts WHERE task_id=?")?,
        vec![vec!["NULL".to_owned()]]
    );
    assert!(handed.borrow().is_empty());
    assert!(verifications(&rig)?.is_empty());
    assert_eq!(rig.reserved_work_ms, 40);
    Ok(())
}

/// B14a-1c review H1 · a check whose cleanup did not settle is an obligation the stop keeps: the
/// task waits `effect_unknown`, the check's cost is recorded (the runtime measured it), and the
/// outcome needs settlement. (R15: the cost is never "lost" — the runtime measures it — so the
/// unsettled shape is the workload's pending cleanup.)
#[test]
fn an_unsettled_check_needs_settlement() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let mut pending = matched(7);
    pending.cleanup_pending = true;
    let (verifier, _) = oracle(vec![pending]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::NeedsSettlement(StopReason::Unsettled))
    );
    assert_eq!(state(&rig)?, "effect_unknown");
    assert_eq!(verifications(&rig)?[0][2], "7");
    assert!(rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?.is_empty());
    Ok(())
}

/// B14a-1c review M4c · a check costing more than the verify reservation holds is recorded as an
/// unknown cost (R1.8), never refused into an error: 300,001 ms against a 300,000 ms reservation.
#[test]
fn a_check_past_the_verify_reservation_is_an_unknown_cost() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![matched(300_001)]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::NeedsSettlement(StopReason::Unsettled))
    );
    assert_eq!(state(&rig)?, "effect_unknown");
    assert_eq!(verifications(&rig)?[0][2], "NULL");
    Ok(())
}

/// R14.1 · a verify reservation with no time past the check's teardown share is a check the
/// runtime records as not run — a timeout at no cost under its own schema — never a stop that
/// strands the task. The first check spends 290,000 of 300,000 ms; the 10,000 left are exactly the
/// teardown share, so the second attempt's window is empty: the verifier is called once, the task
/// fails as `VerifierTimeout`.
#[test]
fn an_empty_check_window_is_recorded_as_a_timeout_at_no_cost() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![
        Candidate::Replacement(FIRST.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    let (verifier, handed) = oracle(vec![mismatched(290_000)]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::VerifierTimeout))
    );
    assert_eq!(state(&rig)?, "failed");
    let handed = handed.borrow();
    assert_eq!(
        handed.len(),
        1,
        "the verifier was not called for an empty window"
    );
    let found = verifications(&rig)?;
    assert_eq!(found.len(), 2);
    assert_eq!(
        found[0],
        vec![
            "failed".to_owned(),
            handed[0].0.clone(),
            "290000".to_owned(),
            "application/json".to_owned(),
            U64_CHECK_SCHEMA.to_owned(),
            "0000000000000000".to_owned(),
        ]
    );
    assert_eq!(found[1][..1].to_vec(), vec!["timeout".to_owned()]);
    assert_eq!(
        found[1][2..].to_vec(),
        vec![
            "0".to_owned(),
            "application/json".to_owned(),
            "hee3.check-window-empty/1".to_owned(),
            "0000000000000000".to_owned(),
        ]
    );
    // The second attempt's own subject: a digest the wire admits, not the first check's (the
    // double was never called for it, so no independent record of the applied snapshot exists;
    // the first case above pins that the subject IS the handed snapshot's digest).
    assert!(Sha256Digest::parse(&found[1][1]).is_ok());
    assert_ne!(found[1][1], handed[0].0, "the second attempt's own subject");
    // The evidence body, read back from the object the verification names (review of 2fd1e46).
    let evidence = rows(
        &rig,
        "SELECT v.evidence_digest FROM verifications v JOIN attempts a ON a.id=v.attempt_id \
         WHERE a.task_id=? AND v.verdict='timeout'",
    )?;
    assert_eq!(evidence.len(), 1);
    assert_eq!(
        object_json(&rig, &evidence[0][0])?,
        serde_json::json!({
            "kind": "check_window_empty",
            "reserved_verify_ms": 10_000,
            "teardown_ms": 10_000,
        })
    );
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["verifier_timeout".to_owned()]]
    );
    Ok(())
}

/// B14a-1c review M4b · a work reservation spent before the next attempt can begin stops the task
/// by policy instead of failing with a zero lease. The first attempt sleeps past its work window,
/// so its candidate is refused at the deadline; what remains is below the teardown share.
#[test]
fn a_spent_work_reservation_stops_the_task_by_policy() -> Outcome_ {
    let rig = rig(&Shape {
        work_ms: 1_500,
        ..Shape::default()
    })?;
    let principal = owner();
    let (mut source, asked) = script(vec![
        Candidate::Replacement(FIRST.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    source.hook = Some(Box::new(|| std::thread::sleep(Duration::from_millis(700))));
    let (verifier, handed) = oracle(vec![]);
    let outcome = run(&rig, &principal, source, verifier, 10).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::Policy(
            habitat_engine::task::LoopRefusal::Deadline
        )))
    );
    assert_eq!(state(&rig)?, "failed");
    assert_eq!(asked.borrow().len(), 1, "no second attempt began");
    assert!(handed.borrow().is_empty());
    assert_eq!(
        rows(&rig, "SELECT id FROM attempts WHERE task_id=?")?.len(),
        1
    );
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["task_policy_stop".to_owned()]]
    );
    Ok(())
}

/// B14a-1c review (mislabelled stop) · a verifier's `Cancelled` for a task nobody cancelled is the
/// verifier's error, recorded `error`; for a cancelled task it is recorded `cancelled` and the task
/// stops as a cancellation, never as an unsettled obligation.
#[test]
fn a_verifier_s_cancelled_is_a_cancellation_only_when_the_task_was_cancelled() -> Outcome_ {
    let uncancelled = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![cancelled_run()]);
    let outcome =
        run(&uncancelled, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::VerifierError))
    );
    assert_eq!(verifications(&uncancelled)?[0][0], "error");
    assert_eq!(
        rows(
            &uncancelled,
            "SELECT reason FROM task_stops WHERE task_id=?"
        )?,
        vec![vec!["verifier_error".to_owned()]]
    );

    let cancelled = rig(&Shape::default())?;
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (mut verifier, _) = oracle(vec![cancelled_run()]);
    verifier.hook = Some(Box::new(|| {
        cancel(
            &cancelled,
            &principal,
            "28f10000-0000-4000-8000-0000000000c5",
        );
    }));
    let outcome =
        run(&cancelled, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::Cancelled))
    );
    assert_eq!(state(&cancelled)?, "cancelled");
    assert_eq!(verifications(&cancelled)?[0][0], "cancelled");
    assert_eq!(
        rows(&cancelled, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["durable_cancellation".to_owned()]]
    );
    Ok(())
}

/// The passed check's readbacks, outcome and cleanup records, decoded from their committed objects:
/// the attempt they belong to, both subjects read back, no outputs (the model's run retains none);
/// matched with no steps; the three cleanup predicates settled and the aggregate — which the runtime
/// does not own — unknown (R15 round 2, MEDIUM-10).
fn assert_run_records_decoded(
    rig: &Rig,
    records: &[Vec<String>],
    evidence: &serde_json::Value,
) -> Result<(), Box<dyn Error>> {
    let object = |kind: &str| -> Result<serde_json::Value, Box<dyn Error>> {
        let row = records.iter().find(|row| row[0] == kind).ok_or(kind)?;
        object_json(rig, &row[2])
    };
    let attempt_id = rows(rig, "SELECT id FROM attempts WHERE task_id=?")?;
    assert_eq!(
        object("readbacks")?,
        serde_json::json!({
            "attempt": attempt_id[0][0],
            "subjects_verified": true,
            "protected_unchanged": true,
            "outputs": [],
        })
    );
    let outcome = object("run_outcome")?;
    assert_eq!(outcome["outcome"], "matched");
    assert_eq!(outcome["steps"], serde_json::json!([]));
    assert_eq!(evidence["capture_failed_at"], serde_json::Value::Null);
    let cleanup = object("run_cleanup")?;
    assert_eq!(cleanup["aggregate"], "settled");
    assert_eq!(
        cleanup["obligations"],
        serde_json::json!([
            {"id": "process", "state": "settled"},
            {"id": "scratch", "state": "settled"},
            {"id": "retained_paths", "state": "settled"},
            {"id": "resources", "state": "unknown"},
        ])
    );
    Ok(())
}

/// R15 · a passed check commits its four records with the verification and `accept` reads them
/// back: the evidence (`hee3.u64-check/1`) cites exactly the committed set — kind, artifact id,
/// digest and size — and the clock record read back from its object carries the window and the
/// observation the model handed the runtime (cutoff 300,000 − T, observed 7 ms, decisive 6 ms,
/// no timeout intent), so nothing about the run was re-typed on the way to the ledger.
#[test]
fn a_passed_check_commits_four_records_that_accept_reads_back() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, handed) = oracle(vec![matched(7)]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert_eq!(state(&rig)?, "accepted");
    let records = committed_records(&rig)?;
    assert_eq!(
        records
            .iter()
            .map(|row| (row[0].as_str(), row[4].as_str()))
            .collect::<Vec<_>>(),
        [
            ("readbacks", "passed"),
            ("run_cleanup", "passed"),
            ("run_clock", "passed"),
            ("run_outcome", "passed"),
        ]
    );
    let found = verifications(&rig)?;
    assert_eq!(
        found[0][2..].to_vec(),
        vec![
            "7".to_owned(),
            "application/json".to_owned(),
            U64_CHECK_SCHEMA.to_owned(),
            "0000000000000001".to_owned(),
        ]
    );
    // The evidence cites exactly the committed set.
    let evidence_digest = rows(
        &rig,
        "SELECT v.evidence_digest FROM verifications v JOIN attempts a ON a.id=v.attempt_id \
         WHERE a.task_id=?",
    )?;
    let evidence = object_json(&rig, &evidence_digest[0][0])?;
    assert_eq!(evidence["kind"], "u64_check");
    assert_eq!(evidence["outcome"], "matched");
    assert_eq!(evidence["captured"], true);
    let cited = evidence["records"].as_object().ok_or("a records map")?;
    assert_eq!(cited.len(), 4);
    for row in &records {
        let entry = &cited[&row[0]];
        assert_eq!(
            (
                entry["artifact_id"].as_str(),
                entry["sha256"].as_str(),
                entry["byte_length"].as_u64()
            ),
            (
                Some(row[1].as_str()),
                Some(row[2].as_str()),
                row[3].parse().ok()
            ),
            "{}",
            row[0]
        );
    }
    // The clock record, read back from its committed object.
    let clock_row = records
        .iter()
        .find(|row| row[0] == "run_clock")
        .ok_or("a clock")?;
    let clock = object_json(&rig, &clock_row[2])?;
    let handed = handed.borrow();
    let window = handed[0].2;
    let teardown_ms = u64::try_from(CHECK_TEARDOWN.as_millis())?;
    assert_eq!(clock["work_cutoff_ms"], 300_000 - teardown_ms);
    assert_eq!(clock["task_deadline_ms"], 300_000);
    assert_eq!(clock["observed_ms"], 7);
    assert_eq!(clock["decisive_ms"], 6);
    assert_eq!(clock["timeout_intent_ms"], serde_json::Value::Null);
    assert!(window.until > window.begun);
    assert_run_records_decoded(&rig, &records, &evidence)?;
    Ok(())
}

/// R15.4 · a workload that refused to launch at its deadline is still a run with four records: the
/// runtime records `timeout` at the measured cost under its own schema, the clock carries the
/// timeout intent at the cutoff, and the task fails as `verifier_timeout`.
#[test]
fn a_run_that_never_launched_is_recorded_with_four_records() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let teardown_ms = u64::try_from(CHECK_TEARDOWN.as_millis())?;
    let (verifier, _) = oracle(vec![Answer {
        run: Err(workload::Error::Deadline),
        elapsed: Duration::from_millis(300_000 - teardown_ms),
        cleanup_pending: false,
    }]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::VerifierTimeout))
    );
    assert_eq!(state(&rig)?, "failed");
    let found = verifications(&rig)?;
    assert_eq!(
        found[0][2..].to_vec(),
        vec![
            (300_000 - teardown_ms).to_string(),
            "application/json".to_owned(),
            U64_CHECK_SCHEMA.to_owned(),
            "0000000000000000".to_owned(),
        ]
    );
    assert_eq!(found[0][0], "timeout");
    let records = committed_records(&rig)?;
    assert_eq!(records.len(), 4);
    let clock_row = records
        .iter()
        .find(|row| row[0] == "run_clock")
        .ok_or("a clock")?;
    let clock = object_json(&rig, &clock_row[2])?;
    assert_eq!(clock["timeout_intent_ms"], 300_000 - teardown_ms);
    assert_eq!(clock["decisive_ms"], serde_json::Value::Null);
    let outcome_row = records
        .iter()
        .find(|row| row[0] == "run_outcome")
        .ok_or("an outcome")?;
    assert_eq!(object_json(&rig, &outcome_row[2])?["outcome"], "timeout");
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["verifier_timeout".to_owned()]]
    );
    Ok(())
}

/// R15.9(c) · the live verifier in the gate: the fixed workload refuses to launch its first stage
/// at the namespace's own pin validation — the rig's shim pin (`/opt/shim`, a zero digest) does
/// not match, so `prepare` refuses before any scope is consulted (the bad systemd-run pin behind it
/// is never reached; the review of d5a68c6 traced this). `LauncherFailed`: one refused step, no
/// process; the runtime records `error` with four records — the outcome record naming the refused
/// step and its refusal — and the task stops `verifier_error`; the check's job root is torn down.
#[test]
fn the_live_verifier_records_a_refused_launch_with_four_records() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let verifier = LiveVerifier::new(&rig.profile.declared, bad_pin_scopes());
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::VerifierError))
    );
    assert_eq!(state(&rig)?, "failed");
    let found = verifications(&rig)?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0][0], "error");
    assert_eq!(found[0][4], U64_CHECK_SCHEMA);
    let records = committed_records(&rig)?;
    assert_eq!(records.len(), 4, "{records:?}");
    let outcome_row = records
        .iter()
        .find(|row| row[0] == "run_outcome")
        .ok_or("an outcome")?;
    let recorded = object_json(&rig, &outcome_row[2])?;
    assert_eq!(recorded["outcome"], "launcher_failed");
    let steps = recorded["steps"].as_array().ok_or("steps")?;
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0]["label"], "compile-library");
    assert_eq!(steps[0]["capture"], serde_json::Value::Null);
    assert!(
        !steps[0]["refused"].is_null(),
        "the refusal is named: {}",
        steps[0]
    );
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["verifier_error".to_owned()]]
    );
    let check_roots = fs::read_dir(&rig.attempts)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".check"))
        .count();
    assert_eq!(check_roots, 0, "the check's job root was torn down");
    Ok(())
}

/// R15.4 (review of d5a68c6, MEDIUM-4) · a workload refused because a subject changed under it is
/// still a run with four records, and its outcome record says so: `subjects: changed`, never
/// "unchanged" for a refusal whose cause IS a failed subject readback. The verdict is `error`.
#[test]
fn a_subject_refusal_is_recorded_as_a_changed_subject() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![Answer {
        run: Err(workload::Error::Subject(
            habitat_engine::worker::workspace::Error::Changed,
        )),
        elapsed: Duration::from_millis(41),
        cleanup_pending: false,
    }]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::VerifierError))
    );
    let found = verifications(&rig)?;
    assert_eq!(
        (found[0][0].as_str(), found[0][2].as_str()),
        ("error", "41")
    );
    let records = committed_records(&rig)?;
    assert_eq!(records.len(), 4);
    let outcome_row = records
        .iter()
        .find(|row| row[0] == "run_outcome")
        .ok_or("an outcome")?;
    let recorded = object_json(&rig, &outcome_row[2])?;
    assert_eq!(recorded["outcome"], "invalid_subject");
    assert_eq!(recorded["observations"]["subjects"], "changed");
    assert_eq!(recorded["observations"]["process_cleanup"], "complete");
    Ok(())
}
