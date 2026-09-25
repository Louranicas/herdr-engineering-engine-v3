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
use habitat_engine::app::runtime::{
    Candidate, CandidateSource, Check, Dispatch, Error as RuntimeError, Outcome, Previous, Refusal,
    Verifier, dispatch,
};
use habitat_engine::app::tasks::StoreTasks;
use habitat_engine::check::consistency::U64_CRITERIA;
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
/// What the verifier was handed, per check: the snapshot's content digest and its editable bytes.
type Handed = Rc<RefCell<Vec<(String, Vec<u8>, Instant)>>>;

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
         namespace_directories = []\nbusctl_sha256 = \"{zero}\"\nsystemd_run_sha256 = \"{zero}\"\n",
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
                    verify_ms: 300_000,
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

/// A verifier that answers from a script and records the content digest of every snapshot it
/// was handed, and the editable file's bytes in it.
struct Oracle<'h> {
    answers: VecDeque<Check>,
    seen: Handed,
    hook: Option<Box<dyn FnMut() + 'h>>,
}

impl Verifier for Oracle<'_> {
    fn check(&mut self, subject: &Snapshot, deadline: Instant) -> Check {
        let editable = subject
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
            subject.content_digest().unwrap_or_default(),
            editable,
            deadline,
        ));
        if let Some(hook) = self.hook.as_mut() {
            hook();
        }
        self.answers.pop_front().unwrap_or(Check {
            verdict: VerificationVerdict::Error,
            criteria: 0,
            evidence: b"unscripted".to_vec(),
            used_ms: Some(1),
            cleanup_settled: true,
        })
    }
}

fn check(verdict: VerificationVerdict, criteria: u64, evidence: &[u8]) -> Check {
    Check {
        verdict,
        criteria,
        evidence: evidence.to_vec(),
        used_ms: Some(7),
        cleanup_settled: true,
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

fn oracle<'h>(answers: Vec<Check>) -> (Oracle<'h>, Handed) {
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
        "SELECT v.verdict,v.subject_digest,v.used_ms FROM verifications v JOIN attempts a \
         ON a.id=v.attempt_id WHERE a.task_id=? ORDER BY CAST(a.generation AS INTEGER)",
    )
}

/// B14a-1c · fail → `repair_pending` → verifying → accepted. Two attempts, each bound to the same
/// three digests; each verification's subject is what the verifier double was handed; the second
/// candidate request was handed the first check's verdict and evidence.
#[test]
fn a_failed_check_is_repaired_and_the_second_attempt_is_accepted() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (source, asked) = script(vec![
        Candidate::Replacement(FIRST.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    let (verifier, handed) = oracle(vec![
        check(VerificationVerdict::Failed, 0, b"first: wrong"),
        check(VerificationVerdict::Passed, 1, b"second: exact"),
    ]);
    let principal = owner();
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert_eq!(state(&rig)?, "accepted");
    let handed = handed.borrow();
    assert_eq!(
        handed
            .iter()
            .map(|(_, bytes, _)| bytes.as_slice())
            .collect::<Vec<_>>(),
        [FIRST, SECOND],
        "the verifier was handed each applied candidate, in order"
    );
    assert_ne!(handed[0].0, handed[1].0);
    // Each check was handed the task's deadline: in the future when read, within the task limit.
    for (_, _, handed_deadline) in handed.iter() {
        assert!(*handed_deadline <= Instant::now() + habitat_engine::task::TASK_LIMIT);
        assert!(*handed_deadline > Instant::now());
    }
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
            vec!["failed".to_owned(), handed[0].0.clone(), "7".to_owned()],
            vec!["passed".to_owned(), handed[1].0.clone(), "7".to_owned()],
        ]
    );
    assert_eq!(
        *asked.borrow(),
        vec![
            None,
            Some(Previous {
                verdict: VerificationVerdict::Failed,
                criteria: 0,
                evidence: b"first: wrong".to_vec(),
            }),
        ]
    );
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
    let (verifier, handed) = oracle(vec![check(VerificationVerdict::Passed, 1, b"exact")]);
    let principal = owner();
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert_eq!(
        handed.borrow().len(),
        1,
        "the verifier saw only the second candidate"
    );
    let recorded = verifications(&rig)?;
    assert_eq!(
        recorded[0],
        vec![
            "failed".to_owned(),
            REFUSED_SHA256.to_owned(),
            "0".to_owned()
        ]
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

/// B14a-1c · each pre-dispatch refusal stops the task by its name through `finish_preparation`,
/// with no attempt row and no source or verifier call (B14a-R1.4d, R1.5, R2.5).
#[test]
fn pre_dispatch_refusals_stop_the_task_before_any_attempt() -> Outcome_ {
    let other_criteria = format!("sha256:{}", "c".repeat(64));
    let wrong = format!("sha256:{}", "d".repeat(64));
    let cases: [(Shape<'_>, u64, Refusal); 10] = [
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
                protected_digest: Some(&wrong),
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
                criteria: other_criteria,
                ..Shape::default()
            },
            5_000,
            Refusal::CriteriaNotClass,
        ),
        (Shape::default(), 600_001, Refusal::ReservationTooSmall),
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
                baseline_digest: Some(&wrong),
                ..Shape::default()
            },
            5_000,
            Refusal::BaselineMismatch,
        ),
    ];
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
    let (mut verifier, _) = oracle(vec![check(VerificationVerdict::Passed, 1, b"exact")]);
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
    let (verifier, handed) = oracle(vec![check(VerificationVerdict::Passed, 1, b"exact")]);
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
        let (mut verifier, _) = oracle(vec![check(VerificationVerdict::Passed, 1, b"exact")]);
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

/// B14a-1c review H1 · a check with an unknown cost is an obligation the stop keeps: the task
/// waits `effect_unknown`, the check's cost is not recorded, and the outcome needs settlement.
#[test]
fn an_unsettled_check_needs_settlement() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let mut unknown = check(VerificationVerdict::Passed, 1, b"exact, cost lost");
    unknown.used_ms = None;
    let (verifier, _) = oracle(vec![unknown]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::NeedsSettlement(StopReason::Unsettled))
    );
    assert_eq!(state(&rig)?, "effect_unknown");
    assert_eq!(verifications(&rig)?[0][2], "NULL");
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
    let mut over = check(VerificationVerdict::Passed, 1, b"exact, too slow");
    over.used_ms = Some(300_001);
    let (verifier, _) = oracle(vec![over]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::NeedsSettlement(StopReason::Unsettled))
    );
    assert_eq!(state(&rig)?, "effect_unknown");
    assert_eq!(verifications(&rig)?[0][2], "NULL");
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
    let (verifier, _) = oracle(vec![check(VerificationVerdict::Cancelled, 0, b"stopped")]);
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
    let (mut verifier, _) = oracle(vec![check(VerificationVerdict::Cancelled, 0, b"stopped")]);
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

/// B14a-1c re-review LOW-3 · a check reporting a criterion bit the class does not declare is an
/// invalid check, recorded `invalid`, and stops the task as one — never a stranded task.
#[test]
fn an_undeclared_criterion_bit_is_an_invalid_check() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![check(
        VerificationVerdict::Failed,
        0b10,
        b"a bit of its own",
    )]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::InvalidCheck))
    );
    assert_eq!(state(&rig)?, "failed");
    assert_eq!(verifications(&rig)?[0][0], "invalid");
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["invalid_check".to_owned()]]
    );
    Ok(())
}
