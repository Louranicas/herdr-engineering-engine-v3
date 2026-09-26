//! B14a-1c · `app::runtime::dispatch` over a real ledger: the attempt lifecycle driven end to end
//! with a scripted candidate source and a verifier double, each recording what it was handed
//! (F101), and every verdict read back from the ledger rather than from the runtime's answer.
//!
//! Independent values: the refused candidate's subject is coreutils `sha256sum` of its bytes; the
//! event kinds, states and stop reasons are the store's and the design's (B14a-R1..R5) names. The
//! declared workspace digests are taken from `Snapshot::capture` — P2a pinned `content_digest`
//! against coreutils, and no verdict here is about the digest itself.

use super::t08_rig::{self, DaemonStandIn};
use habitat_engine::actions::control::{TaskRequest, Tasks};
use habitat_engine::app::candidates::{
    ClassPrompt, FilePins, NativeCandidates, Outcome as CandidateOutcome, Settle, render,
};
use habitat_engine::app::class_profile::{self, Profile};
use habitat_engine::app::dispatcher;
use habitat_engine::app::live_verifier::LiveVerifier;
use habitat_engine::app::runtime::{
    Answer as SourceAnswer, CHECK_TEARDOWN, Candidate, CandidateSource, CheckPlan, CheckWindow,
    Dispatch, Error as RuntimeError, Observed, Outcome, Previous, Refusal, Verifier, dispatch,
};
use habitat_engine::app::tasks::StoreTasks;
use habitat_engine::app::workload::{self, Outcome as RunOutcome, Run};
use habitat_engine::check::consistency::U64_CRITERIA;
use habitat_engine::check::u64_oracle::Evaluation;
use habitat_engine::contracts::control::{
    CancelReason, Precondition, ResourceKind, criteria_digest,
};
use habitat_engine::contracts::receipt::{Address as _, ReceiptV1};
use habitat_engine::contracts::roster::{
    Availability, Kind, Locality, ObservationInput, ObservationSource, RosterDefinitionV1,
    Selection, Update,
};
use habitat_engine::contracts::{Sha256Digest, UuidV4};
use habitat_engine::store::{
    Allocation, Principal, RequestSource, Store, Submission, VerificationVerdict,
};
use habitat_engine::task::control::Cancel;
use habitat_engine::task::control::Spec;
use habitat_engine::task::driver::{Outcome as Driven, StopReason};
use habitat_engine::worker::native::FULL_FILE;
use habitat_engine::worker::resources::Scope;
use habitat_engine::worker::workspace::Snapshot;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::tasks::{EPOCH, GENERATION, Scratch};

/// The schema every in-gate check records its evidence under since 2c-iii: the receipt itself
/// (R17 round 2, decision 3 — the evidence artifact id is the receipt's `run_id`). The runtime's
/// own `hee3.u64-check/1` remains only for a compose the composer refused.
const RECEIPT: &str = ReceiptV1::SCHEMA_ID;

type Outcome_ = Result<(), Box<dyn Error>>;
/// What the candidate source was handed, per request.
/// One request the script double was handed (F101: a model that records what it was given): the
/// previous verification, the attempt's identity, the invocation id the runtime minted, and the
/// recipe and workspace digests.
#[derive(Clone, Debug)]
struct Seen {
    previous: Option<Previous>,
    attempt: String,
    generation: u64,
    invocation: String,
    recipe: String,
    workspace: String,
}
type Asked = Rc<RefCell<Vec<Seen>>>;
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
/// The class's real base (`evaluation/.../base/src/lib.rs`): the bytes the reviewed closure pins, so
/// the native source's class prompt reads the same file the workspace holds (B14a-4).
const BASE_LIB: &[u8] = include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/base/src/lib.rs");
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
    /// The compiler pin the profile declares, when not the toolchain rustc's own digest.
    compiler_sha256: Option<&'a str>,
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
            compiler_sha256: None,
        }
    }
}

fn private(path: &Path) -> Result<(), Box<dyn Error>> {
    DirBuilder::new().mode(0o700).create(path)?;
    Ok(())
}

/// The toolchain binary behind `rustc` (`RUSTC` when the gate sets it, else `rustc --print
/// sysroot`), never a rustup proxy: the plan pins and probes the executable itself.
fn rustc() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(path) = std::env::var_os("RUSTC") {
        return Ok(PathBuf::from(path));
    }
    let sysroot = std::process::Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()?;
    if !sysroot.status.success() {
        return Err(format!("rustc --print sysroot: {}", sysroot.status).into());
    }
    let path = PathBuf::from(String::from_utf8(sysroot.stdout)?.trim()).join("bin/rustc");
    if !path.is_file() {
        return Err(format!("no toolchain rustc at {}", path.display()).into());
    }
    Ok(path)
}

fn hex_digest(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    sha2::Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut text, byte| {
            use std::fmt::Write as _;
            let _ = write!(text, "{byte:02x}");
            text
        })
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
    // The reviewed closure the shared plan publishes at dispatch: the 003 lane's, installed by
    // digest as the operator would (the same fixture the class-profile and plan proofs use).
    let reviewed = class.join("reviewed");
    private(&reviewed)?;
    for entry in fs::read_dir(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/reviewed-003"
    ))? {
        let entry = entry?;
        file(&reviewed.join(entry.file_name()), &fs::read(entry.path())?)?;
    }
    // The pins the shared plan reads under (R17 round 2): the toolchain's own rustc, a stand-in
    // shim whose bytes are its pin, and the declared grant and effect files beside the profile.
    let compiler = rustc()?;
    let compiler_digest = match shape.compiler_sha256 {
        Some(pinned) => pinned.to_owned(),
        None => format!("sha256:{}", hex_digest(&fs::read(&compiler)?)),
    };
    let shim_bytes = b"#!/bin/sh\nexit 0\n";
    let shim = class.join("stand-in-shim");
    file(&shim, shim_bytes)?;
    let shim_digest = format!("sha256:{}", hex_digest(shim_bytes));
    let authority = b"{\"authority\":\"WL-U64 fixed workload\",\"issuer\":\"operator\"}\n";
    let specification = b"{\"isolation\":\"bwrap --unshare-all; no network\"}\n";
    file(&class.join("authority.json"), authority)?;
    file(&class.join("isolation.json"), specification)?;
    let authority_digest = format!("sha256:{}", hex_digest(authority));
    let specification_digest = format!("sha256:{}", hex_digest(specification));
    let text = format!(
        "schema = \"hee3.class-profile/1\"\nclass = \"rust-library-change/1\"\n\n\
         [[workspace]]\nid = \"{}\"\nbaseline = \"base\"\nbaseline_digest = \"{}\"\n\
         protected = \"protected\"\nprotected_digest = \"{}\"\n\n\
         [pins]\ncompiler = {{ host = \"{}\", sha256 = \"{compiler_digest}\" }}\n\
         shim = {{ host = \"{}\", sha256 = \"{shim_digest}\" }}\nruntime_files = []\n\
         namespace_directories = []\nbusctl_sha256 = \"{zero}\"\nsystemd_run_sha256 = \"{zero}\"\n[reviewed]\nexpectation = {{ artifact_id = \"c220e7ce-0753-47ef-bdac-15710bc4981c\", sha256 = \"sha256:3a7faa5510790c20322ad5829091211eb8c04ab6016e16ec8391433dae3392b9\", byte_length = 794, media_type = \"application/json\", schema_id = \"hee3.receipt/1:ExpectationV1\" }}\nreview = {{ artifact_id = \"a47470c5-f11c-4f64-9ac8-6dcf80750ed6\", sha256 = \"sha256:f288225476120254f5c3a93266fc8f2a8763810462fbddd7c62161107cb39adb\", byte_length = 1145, media_type = \"application/json\", schema_id = \"hee3.receipt/1:ReviewV1\" }}\n[grant]\ngrant_id = \"28f90000-0000-4000-8000-000000000001\"\nissuer_id = \"operator\"\nauthority = {{ file = \"authority.json\", sha256 = \"{authority_digest}\" }}\n[effect]\neffect_id = \"fixed-u64-workload-output\"\nscope = \"the rig's scope\"\nspecification = {{ file = \"isolation.json\", sha256 = \"{specification_digest}\" }}\n",
        shape.declared,
        shape.baseline_digest.unwrap_or(&base_digest),
        shape.protected_digest.unwrap_or(&protected_digest),
        compiler.display(),
        shim.display(),
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
    // The dispatcher reads the class profile from the task owner (R20 round 2 A11): install it there.
    let tasks = StoreTasks::new(store, EPOCH.to_owned()).with_class_profile(Ok(profile.clone()));
    Ok(Rig {
        tasks,
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
    /// A worker settle to attach to every answer — a script that claims a provider was asked (the
    /// identity-refusal proof hands one naming another attempt).
    settle: Option<Settle>,
}

impl CandidateSource for Script<'_> {
    fn next(&mut self, ask: &habitat_engine::app::runtime::Ask<'_>) -> SourceAnswer {
        self.seen.borrow_mut().push(Seen {
            previous: ask.previous.cloned(),
            attempt: ask.attempt.as_str().to_owned(),
            generation: ask.generation.value(),
            invocation: ask.invocation.as_str().to_owned(),
            recipe: ask.recipe.as_str().to_owned(),
            workspace: ask.workspace.as_str().to_owned(),
        });
        if let Some(hook) = self.hook.as_mut() {
            hook();
        }
        // A script asks no provider: no worker settle travels with its candidate (R19.3) unless the
        // proof attached one.
        SourceAnswer {
            candidate: self.answers.pop_front().unwrap_or(Candidate::Exhausted),
            settle: self.settle.clone(),
        }
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
            settle: None,
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
    mut verifier: V,
    capture_ms: u64,
) -> Result<Outcome, RuntimeError> {
    let mut source = source;
    let drain = AtomicBool::new(false);
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
            capture_ms: Some(capture_ms),
            drain: &drain,
        },
        &mut source,
        &mut verifier,
    )
}

/// R18 A1 · the whole Ask, per attempt (F101): the attempt ids the ledger holds, generations 1 and
/// 2, two distinct invocation ids, the recipe the profile's digest, the workspace the baseline's.
fn assert_asked_whole(
    rig: &Rig,
    asked: &[Seen],
    baseline_digest: &str,
) -> Result<(), Box<dyn Error>> {
    let attempt_ids = rows(
        rig,
        "SELECT id FROM attempts WHERE task_id=? ORDER BY CAST(generation AS INTEGER)",
    )?;
    assert_eq!(
        asked
            .iter()
            .map(|seen| seen.attempt.clone())
            .collect::<Vec<_>>(),
        attempt_ids
            .iter()
            .map(|row| row[0].clone())
            .collect::<Vec<_>>()
    );
    assert_eq!((asked[0].generation, asked[1].generation), (1, 2));
    assert_ne!(asked[0].invocation, asked[1].invocation);
    assert!(UuidV4::parse(&asked[0].invocation).is_ok());
    assert!(
        asked
            .iter()
            .all(|seen| attempt_ids.iter().all(|row| row[0] != seen.invocation)),
        "an invocation id is minted for the call, never an attempt id reused"
    );
    assert_eq!(
        asked[1]
            .previous
            .as_ref()
            .map(|previous| previous.schema_id.as_str()),
        Some(RECEIPT),
        "the previous verification carries the schema its evidence was recorded under"
    );
    assert_eq!(asked[0].recipe, PROFILE_DIGEST);
    assert_eq!(asked[0].workspace, baseline_digest);
    Ok(())
}

/// The previous check as the candidate source was handed it: its verdict, no criteria, and the
/// receipt itself as evidence — its one case counted under `counted` (`failed` for a mismatch),
/// its artifact inventory the four run records.
fn assert_previous(
    previous: Option<&Option<Previous>>,
    verdict: VerificationVerdict,
    counted: &str,
) -> Result<(), Box<dyn Error>> {
    let previous = previous
        .and_then(Option::as_ref)
        .ok_or("a previous check")?;
    assert_eq!((previous.verdict, previous.criteria), (verdict, 0));
    let evidence: serde_json::Value = serde_json::from_slice(&previous.evidence)?;
    assert_eq!(evidence["protocol"], "hee3.receipt");
    assert_eq!(evidence["cases"]["discovered"], 1);
    assert_eq!(evidence["cases"][counted], 1, "{counted}");
    assert_eq!(evidence["artifacts"]["count"], 4);
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

/// B14a-5 (R19.3) · a settle naming another attempt is the source's error, refused at the seam before
/// any candidate is applied or any row written: the dispatch ends `Identity`, the attempt stays as
/// begun with no settled event, no run record exists and no check was asked.
#[test]
fn a_settle_naming_another_attempt_is_refused_before_any_write() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (mut source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    source.settle = Some(Settle {
        attempt: "28f00000-0000-4000-8000-0000000000ee".to_owned(),
        adapter: "ollama-fc44-12ff8654/2",
        input_tokens: None,
        output_tokens: None,
        wall_ms: 1,
        finish: None,
        identity_sha256: None,
        raw_sha256: None,
        outcome: CandidateOutcome::Replacement(SECOND.len()),
    });
    let (verifier, handed) = oracle(vec![]);
    let principal = owner();
    let outcome = run(&rig, &principal, source, verifier, 5_000);
    assert!(
        matches!(outcome, Err(RuntimeError::Identity)),
        "{outcome:?}"
    );
    assert!(handed.borrow().is_empty(), "no check was asked");
    assert_eq!(
        rows(
            &rig,
            "SELECT state,settled_event IS NULL FROM attempts WHERE task_id=?"
        )?,
        vec![vec!["running".to_owned(), "1".to_owned()]],
        "the attempt stays as begun, unsettled"
    );
    assert_eq!(
        rows(
            &rig,
            "SELECT count(*) FROM attempt_records r JOIN attempts a ON a.id=r.attempt_id \
             WHERE a.task_id=?"
        )?,
        vec![vec!["0".to_owned()]]
    );
    assert!(worker_settles(&rig)?.is_empty());
    Ok(())
}

// ------------------------------------------------ the dispatcher (B14b-1, R20 round 2)

/// A provider double for the dispatcher (F101: a model, not a script): it records every task it was
/// asked to open, serves a scripted source/verifier pair per open in order, and raises the stop flag
/// once it has served `stop_after` pairs — the dispatcher then ends its wait `Drained`.
struct ScriptedProvider<'h> {
    pairs: VecDeque<(Script<'h>, Oracle<'h>)>,
    opened: Rc<RefCell<Vec<String>>>,
    stop: &'h AtomicBool,
    stop_after: usize,
}

impl<'h> dispatcher::Provider for ScriptedProvider<'h> {
    type Source = Script<'h>;
    type Verifier = Oracle<'h>;
    fn open(
        &mut self,
        next: &habitat_engine::store::Dispatchable,
    ) -> Result<(Script<'h>, Oracle<'h>), dispatcher::Unavailable> {
        self.opened.borrow_mut().push(next.task.clone());
        if self.opened.borrow().len() >= self.stop_after {
            self.stop.store(true, Ordering::SeqCst);
        }
        self.pairs
            .pop_front()
            .ok_or(dispatcher::Unavailable::NoNativeProvider)
    }
}

/// The budget every dispatcher proof runs under (F102, closure H6): past it the watchdog raises the
/// stop and wakes the wait, and the proof FAILS on `timed_out` instead of hanging.
const DISPATCHER_BUDGET: Duration = Duration::from_secs(20);

/// Run the dispatcher over the rig until it exits, with `stop` as both the engine's drain and the
/// between-attempts drain flag; every step it reports is collected. A watchdog thread ends a run
/// that outlives `DISPATCHER_BUDGET` and the helper fails the proof by name.
fn run_dispatcher<P: dispatcher::Provider>(
    rig: &Rig,
    provider: &mut P,
    stop: &AtomicBool,
) -> (dispatcher::Exit, Vec<String>) {
    run_dispatcher_with(rig, provider, stop, &rig.selections)
}

fn run_dispatcher_with<P: dispatcher::Provider>(
    rig: &Rig,
    provider: &mut P,
    stop: &AtomicBool,
    selections: &[Selection],
) -> (dispatcher::Exit, Vec<String>) {
    let reported = std::sync::Mutex::new(Vec::new());
    let report = |line: &str| {
        if let Ok(mut lines) = reported.lock() {
            lines.push(line.to_owned());
        }
    };
    let done = AtomicBool::new(false);
    let timed_out = AtomicBool::new(false);
    let exit = std::thread::scope(|scope| {
        scope.spawn(|| {
            let started = Instant::now();
            while !done.load(Ordering::SeqCst) {
                if started.elapsed() >= DISPATCHER_BUDGET {
                    timed_out.store(true, Ordering::SeqCst);
                    stop.store(true, Ordering::SeqCst);
                    rig.tasks.wake();
                    return;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        });
        let exit = dispatcher::Dispatcher {
            tasks: &rig.tasks,
            attempts: &rig.attempts,
            provider,
            agent_record_id: &rig.agent,
            selections,
            drain: stop,
        }
        .run(&report);
        done.store(true, Ordering::SeqCst);
        exit
    });
    let lines = reported
        .lock()
        .map(|lines| lines.clone())
        .unwrap_or_default();
    assert!(
        !timed_out.load(Ordering::SeqCst),
        "the dispatcher did not end within {DISPATCHER_BUDGET:?}: {lines:?}"
    );
    (exit, lines)
}

/// B14b-1 (b) · the dispatcher picks the admitted task, drives it to ACCEPTED through the scripted
/// pair, notifies, and ends `Drained` when the drain is set; the provider was opened once, for that task.
#[test]
fn the_dispatcher_picks_the_admitted_task_and_drives_it_to_acceptance() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (mut verifier, _) = oracle(vec![matched(7)]);
    let stop = AtomicBool::new(false);
    // The drain is raised during the one check: acceptance does not read it, the next wait does —
    // so the task is accepted and the dispatcher then ends `Drained` (a stop raised at `open` would
    // be read by `begin` and drain the dispatch instead: measured, the skeleton's first run).
    let flag = &stop;
    verifier.hook = Some(Box::new(move || {
        flag.store(true, Ordering::SeqCst);
    }));
    let opened = Rc::new(RefCell::new(Vec::new()));
    let mut provider = ScriptedProvider {
        pairs: VecDeque::from(vec![(source, verifier)]),
        opened: Rc::clone(&opened),
        stop: &stop,
        stop_after: usize::MAX,
    };
    let (exit, lines) = run_dispatcher(&rig, &mut provider, &stop);
    assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    assert_eq!(*opened.borrow(), vec![TASK.to_owned()]);
    assert_eq!(state(&rig)?, "accepted");
    assert!(
        lines
            .iter()
            .any(|line| line.contains("TaskDone(\"accepted\")")),
        "{lines:?}"
    );
    Ok(())
}

/// B14b-1 (b), the round-1 review's first defect · a task cancelled before any attempt is picked
/// (it is `cancellation_requested`, not `admitted`), stopped by its own name through the
/// pre-dispatch path with no attempt row, and never picked again: the provider is never opened.
#[test]
fn a_task_cancelled_before_dispatch_is_stopped_by_name_and_never_picked_again() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    cancel(&rig, &principal, "28f10000-0000-4000-8000-0000000000d1");
    assert_eq!(state(&rig)?, "cancellation_requested");
    let stop = AtomicBool::new(false);
    let opened = Rc::new(RefCell::new(Vec::new()));
    // No pair: a pick that reached `open` would end the loop `Unavailable`, which the assertion
    // below distinguishes from the drain.
    let mut provider = ScriptedProvider {
        pairs: VecDeque::new(),
        opened: Rc::clone(&opened),
        stop: &stop,
        stop_after: usize::MAX,
    };
    // After the cancelled task is stopped the read returns None and the wait would block: a watcher
    // raises the stop once the ledger says `cancelled` (the budget is the helper's).
    let flag = &stop;
    let rig_ref = &rig;
    std::thread::scope(|scope| {
        scope.spawn(move || stop_when(rig_ref, TASK, "cancelled", flag));
        let (exit, lines) = run_dispatcher(&rig, &mut provider, &stop);
        assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    });
    assert!(opened.borrow().is_empty(), "the provider was never opened");
    assert_eq!(state(&rig)?, "cancelled");
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["cancelled_before_dispatch".to_owned()]]
    );
    assert_eq!(
        rows(&rig, "SELECT count(*) FROM attempts WHERE task_id=?")?,
        vec![vec!["0".to_owned()]]
    );
    Ok(())
}

/// B14b-1 (b) · with no provider configured the dispatcher enters its named unavailable state after
/// the free checks pass: it stops picking and the task stays `admitted` — never stopped for an
/// operator's missing configuration (P2c-R1.5 revisited).
#[test]
fn no_provider_is_a_named_dispatcher_state_and_leaves_the_task_admitted() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let stop = AtomicBool::new(false);
    let (exit, lines) = run_dispatcher(&rig, &mut dispatcher::NoProvider, &stop);
    assert_eq!(
        exit,
        dispatcher::Exit::Unavailable(dispatcher::Unavailable::NoNativeProvider),
        "{lines:?}"
    );
    assert_eq!(state(&rig)?, "admitted");
    assert!(
        lines
            .iter()
            .any(|line| line.contains("unavailable: no native provider (B14b-2)")),
        "{lines:?}"
    );
    Ok(())
}

/// B14b-1 (b), D5 (corrected, closure item 9) · the drain observed between attempts at attempt 2:
/// the verifier's hook raises it during the first check (a mismatch), so the next `begin` sees it —
/// the dispatch ends `Drained` with no stop written, the first attempt settled and the task
/// `repair_pending`, which the read never returns: STRANDED until recovery (B17), a stated gap; the
/// dispatcher exits `Drained` at its next wait.
#[test]
fn a_drain_between_attempts_leaves_the_task_resumable_with_no_stop_written() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let stop = AtomicBool::new(false);
    let (source, _) = script(vec![
        Candidate::Replacement(FIRST.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    let (mut verifier, _) = oracle(vec![mismatched(7), matched(7)]);
    let flag = &stop;
    verifier.hook = Some(Box::new(move || {
        flag.store(true, Ordering::SeqCst);
    }));
    let opened = Rc::new(RefCell::new(Vec::new()));
    let mut provider = ScriptedProvider {
        pairs: VecDeque::from(vec![(source, verifier)]),
        opened: Rc::clone(&opened),
        stop: &stop,
        stop_after: usize::MAX,
    };
    let (exit, lines) = run_dispatcher(&rig, &mut provider, &stop);
    assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    assert!(
        lines
            .iter()
            .any(|line| line.contains("TaskLeft(\"drained: left for recovery\")")),
        "{lines:?}"
    );
    assert_eq!(state(&rig)?, "repair_pending");
    assert_eq!(
        rows(&rig, "SELECT count(*) FROM task_stops WHERE task_id=?")?,
        vec![vec!["0".to_owned()]]
    );
    assert_eq!(
        rows(&rig, "SELECT count(*) FROM attempts WHERE task_id=?")?,
        vec![vec!["1".to_owned()]],
        "one attempt, settled; no second begun"
    );
    Ok(())
}

/// A watcher that raises the dispatcher's stop once `task` reaches `wanted`, then wakes the wait:
/// the dispatcher's own loop cannot end while a task is dispatchable, so the proofs end it from the
/// ledger's state (budgeted, F102).
fn stop_when(rig: &Rig, task: &str, wanted: &str, stop: &AtomicBool) {
    let started = Instant::now();
    while state_of(rig, task).is_ok_and(|s| s != wanted) {
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "task {task} never reached {wanted}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    stop.store(true, Ordering::SeqCst);
    rig.tasks.wake();
}

/// The state of any task in the rig's ledger.
fn state_of(rig: &Rig, task: &str) -> Result<String, Box<dyn Error>> {
    Ok(
        ledger(rig)?.query_row("SELECT state FROM tasks WHERE id=?", [task], |row| {
            row.get(0)
        })?,
    )
}

/// Submit a second task through the task-action door as `principal`, with `criteria`; returns its id.
fn submit_as(
    rig: &Rig,
    principal: &Principal,
    key: &str,
    criteria: Vec<String>,
) -> Result<String, Box<dyn Error>> {
    let now = 1_000_000;
    rig.tasks
        .submit(
            &TaskRequest {
                principal,
                idempotency_key: key,
                payload: format!("submit {key}").as_bytes(),
                deadline_unix_ms: now + 5_000,
                now_unix_ms: now,
            },
            &Spec {
                task_class: "rust-library-change/1",
                criteria,
                limit_ms: 1_200_000,
                work_ms: 600_000,
                verify_ms: 300_000,
                workspace_id: WORKSPACE.to_owned(),
            },
        )
        .map_err(|fault| format!("{fault:?}"))?;
    Ok(ledger(rig)?.query_row(
        "SELECT t.id FROM tasks t JOIN events e ON e.task_id=t.id AND e.kind='admitted' \
         WHERE t.id != ? ORDER BY e.sequence DESC LIMIT 1",
        [TASK],
        |row| row.get(0),
    )?)
}

/// B14b-1 (b), A4 · a task whose recorded owner is not the operator is stopped `owner_not_operator`
/// as a free check — through the store's one `operator()` rule — before any capture or provider; the
/// operator's own task, accepted first, is not touched again.
#[test]
fn a_non_operator_owner_is_stopped_by_name_before_any_provider() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let operator = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![matched(7)]);
    run(&rig, &operator, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(state(&rig)?, "accepted");
    let reader = Principal::new(1000, "reader").map_err(|e| format!("{e:?}"))?;
    let readers = submit_as(
        &rig,
        &reader,
        "28f10000-0000-4000-8000-0000000000e1",
        U64_CRITERIA.iter().map(|c| (*c).to_owned()).collect(),
    )?;
    let stop = AtomicBool::new(false);
    let opened = Rc::new(RefCell::new(Vec::new()));
    let mut provider = ScriptedProvider {
        pairs: VecDeque::new(),
        opened: Rc::clone(&opened),
        stop: &stop,
        stop_after: usize::MAX,
    };
    let rig_ref = &rig;
    let (readers_ref, flag) = (readers.as_str(), &stop);
    std::thread::scope(|scope| {
        scope.spawn(move || stop_when(rig_ref, readers_ref, "failed", flag));
        let (exit, lines) = run_dispatcher(&rig, &mut provider, &stop);
        assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    });
    assert!(opened.borrow().is_empty(), "no provider for a refused task");
    assert_eq!(state_of(&rig, &readers)?, "failed");
    let reason: String = ledger(&rig)?.query_row(
        "SELECT reason FROM task_stops WHERE task_id=?",
        [readers.as_str()],
        |row| row.get(0),
    )?;
    assert_eq!(reason, "owner_not_operator");
    Ok(())
}

/// B14b-1 (b), A2 · a selection the roster no longer permits makes `begin` refuse `Conflict` with no
/// attempt row; the task is stopped `begin_refused_conflict` through the pre-dispatch path — never
/// left `admitted` for the next read (the round-1 review's hot loop).
#[test]
fn a_stale_selection_stops_the_task_by_the_store_s_name_and_never_re_picks_it() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let mut stale = rig.selections.clone();
    for selection in &mut stale {
        selection.expected_revision = "99".to_owned();
    }
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![matched(7)]);
    let stop = AtomicBool::new(false);
    let opened = Rc::new(RefCell::new(Vec::new()));
    let mut provider = ScriptedProvider {
        pairs: VecDeque::from(vec![(source, verifier)]),
        opened: Rc::clone(&opened),
        stop: &stop,
        stop_after: usize::MAX,
    };
    let rig_ref = &rig;
    let flag = &stop;
    let (exit, lines) = std::thread::scope(|scope| {
        scope.spawn(move || stop_when(rig_ref, TASK, "failed", flag));
        run_dispatcher_with(&rig, &mut provider, &stop, &stale)
    });
    assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    assert_eq!(
        *opened.borrow(),
        vec![TASK.to_owned()],
        "opened once, never re-picked"
    );
    assert_eq!(state(&rig)?, "failed");
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["begin_refused_conflict".to_owned()]]
    );
    assert_eq!(
        rows(&rig, "SELECT count(*) FROM attempts WHERE task_id=?")?,
        vec![vec!["0".to_owned()]]
    );
    Ok(())
}

/// B14b-1 (b), D2 · a submit that lands while the dispatcher waits wakes it: the dispatcher is
/// started over a ledger with nothing dispatchable, a second task is submitted from another thread,
/// and the dispatcher picks exactly that task (the provider records it) before it exits `Unavailable`.
#[test]
fn a_submit_wakes_the_waiting_dispatcher_and_it_picks_the_new_task() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let operator = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![matched(7)]);
    run(&rig, &operator, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(state(&rig)?, "accepted");
    let stop = AtomicBool::new(false);
    let opened = Rc::new(RefCell::new(Vec::new()));
    let mut provider = ScriptedProvider {
        pairs: VecDeque::new(),
        opened: Rc::clone(&opened),
        stop: &stop,
        stop_after: usize::MAX,
    };
    let rig_ref = &rig;
    let submitted = std::sync::Mutex::new(None);
    let submitted_ref = &submitted;
    let exit = std::thread::scope(|scope| {
        scope.spawn(move || {
            // Let the dispatcher reach its wait first; a submit that lands before it is seen by the
            // read instead — either way the task is picked, which is the claim.
            std::thread::sleep(Duration::from_millis(200));
            let id = submit_as(
                rig_ref,
                &owner(),
                "28f10000-0000-4000-8000-0000000000e2",
                U64_CRITERIA.iter().map(|c| (*c).to_owned()).collect(),
            )
            .expect("a second admission");
            if let Ok(mut slot) = submitted_ref.lock() {
                *slot = Some(id);
            }
        });
        let (exit, _) = run_dispatcher(&rig, &mut provider, &stop);
        exit
    });
    assert_eq!(
        exit,
        dispatcher::Exit::Unavailable(dispatcher::Unavailable::NoNativeProvider)
    );
    let submitted = submitted
        .lock()
        .map(|slot| slot.clone())
        .unwrap_or_default();
    assert_eq!(
        opened.borrow().as_slice(),
        [submitted.ok_or("the second task")?]
    );
    Ok(())
}

/// B14b-1 (b), D5 (closure item 9) · a drain set BEFORE the first attempt begins: `dispatch` (which
/// reads the same flag at `begin`) ends `Drained` with nothing written and the task still `admitted`,
/// so a dispatcher run afterwards, the drain cleared, picks it again and drives it to acceptance.
/// STATED (re-check item 9): the first phase drives `dispatch` directly — under a set drain the
/// dispatcher's own wait returns `None` before `admit`; the production route to this outcome is a
/// drain landing between the read and `begin`, which no in-gate proof can time; and "the drain
/// cleared" is a restart, which the second phase stands in for.
#[test]
fn a_task_drained_before_its_first_attempt_is_picked_again_after_the_drain() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let stop = AtomicBool::new(true);
    let (mut source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (mut verifier, _) = oracle(vec![matched(7)]);
    let outcome = dispatch(
        &rig.tasks,
        &rig.profile,
        Dispatch {
            principal: &owner(),
            task: id(TASK),
            agent_record_id: &rig.agent,
            selections: &rig.selections,
            attempts: &rig.attempts,
            forbidden: &[],
            teardown_ms: rig.teardown_ms,
            capture_ms: Some(5_000),
            drain: &stop,
        },
        &mut source,
        &mut verifier,
    )
    .map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Drained);
    assert_eq!(
        state(&rig)?,
        "admitted",
        "nothing written: the task stays dispatchable"
    );
    assert_eq!(
        rows(&rig, "SELECT count(*) FROM attempts WHERE task_id=?")?,
        vec![vec!["0".to_owned()]]
    );
    // The drain cleared: the dispatcher picks the same task and accepts it.
    stop.store(false, Ordering::SeqCst);
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![matched(7)]);
    let opened = Rc::new(RefCell::new(Vec::new()));
    let mut provider = ScriptedProvider {
        pairs: VecDeque::from(vec![(source, verifier)]),
        opened: Rc::clone(&opened),
        stop: &stop,
        stop_after: usize::MAX,
    };
    let flag = &stop;
    let rig_ref = &rig;
    std::thread::scope(|scope| {
        scope.spawn(move || stop_when(rig_ref, TASK, "accepted", flag));
        let (exit, lines) = run_dispatcher(&rig, &mut provider, &stop);
        assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    });
    assert_eq!(*opened.borrow(), vec![TASK.to_owned()]);
    assert_eq!(state(&rig)?, "accepted");
    Ok(())
}

/// B14b-1 (b), D2 · the wait's stop takes precedence over its read: under a set stop, with a task
/// dispatchable, `wait_dispatchable` returns `None` without reading it — pinned here so the rule has
/// a proof that FAILS rather than hangs when the stop check is removed (closure 2, re-check 3).
#[test]
fn the_wait_returns_none_under_a_set_stop_before_reading() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    assert_eq!(state(&rig)?, "admitted", "a task is dispatchable");
    let stopped = || true;
    let waited = rig
        .tasks
        .wait_dispatchable(&stopped)
        .map_err(|_| "poisoned")?
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(waited, None, "the stop wins before the read");
    let running = || false;
    let picked = rig
        .tasks
        .wait_dispatchable(&running)
        .map_err(|_| "poisoned")?
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(picked.map(|next| next.task), Some(TASK.to_owned()));
    Ok(())
}

/// One worker settle as the ledger committed it: the attempt, the record's artifact id, the record.
struct WorkerSettleRow {
    attempt: String,
    artifact_id: String,
    record: serde_json::Value,
}

/// The worker settles the ledger committed with each attempt's SETTLING observation (B14a-5: the
/// record set `committed_run` returns is the one keyed by `attempts.settled_event`), in generation
/// order.
fn worker_settles(rig: &Rig) -> Result<Vec<WorkerSettleRow>, Box<dyn Error>> {
    rows(
        rig,
        "SELECT a.id,r.artifact_id,r.digest FROM attempt_records r \
         JOIN attempts a ON a.id=r.attempt_id AND a.settled_event=r.event_id \
         WHERE a.task_id=? AND r.kind='worker_settle' ORDER BY CAST(a.generation AS INTEGER)",
    )?
    .into_iter()
    .map(|row| {
        Ok(WorkerSettleRow {
            attempt: row[0].clone(),
            artifact_id: row[1].clone(),
            record: object_json(rig, &row[2])?,
        })
    })
    .collect()
}

/// The worker settle the fake's scenario predicts for one answered call (wall aside): the `/2` row,
/// the fixture's token counts, `stop`, the identity digest of the fake's `ps`, the raw digest of the
/// `answer`-th scripted response, and the outcome with its replacement length.
fn scripted_settle(
    attempt: &str,
    scenario: &serde_json::Value,
    answer: usize,
    outcome: &serde_json::Value,
    replacement_bytes: Option<usize>,
) -> serde_json::Value {
    let answered = &scenario["generated"][answer];
    serde_json::json!({
        "attempt": attempt,
        "adapter_profile": "ollama-fc44-12ff8654/2",
        "input_tokens": answered["prompt_eval_count"],
        "output_tokens": answered["eval_count"],
        "finish": "stop",
        "identity_sha256": t08_rig::digest(&t08_rig::rendered(&scenario["ps"])),
        "raw_sha256": t08_rig::digest(&t08_rig::rendered(&scenario["generated"][answer])),
        "outcome": outcome,
        "replacement_bytes": replacement_bytes,
    })
}

/// A worker settle with its wall time taken out — the one field the fake cannot pin — for a whole
/// comparison of the rest; the wall is returned beside it.
fn without_wall(mut record: serde_json::Value) -> Result<(serde_json::Value, u64), Box<dyn Error>> {
    let wall = record
        .as_object_mut()
        .ok_or("a record object")?
        .remove("wall_ms")
        .and_then(|wall| wall.as_u64())
        .ok_or("a wall_ms number")?;
    Ok((record, wall))
}

/// The scenario the native fake answers from, as `native_source` wrote it: the independent source
/// of every count and digest a worker settle records.
fn native_scenario(rig: &Rig) -> Result<serde_json::Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(
        rig.scratch.0.join("native/scenario.json"),
    )?)?)
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

/// Every object the ledger's registry holds — the table `OBJECT_INVENTORY_BOUND` and the backup count
/// (a refused plan's orphans sit in the CAS and are not in it).
fn artifact_count(rig: &Rig) -> Result<i64, Box<dyn Error>> {
    Ok(ledger(rig)?.query_row("SELECT count(*) FROM artifacts", [], |row| row.get(0))?)
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
    assert!(
        worker_settles(&rig)?.is_empty(),
        "a scripted source asks no provider: no worker settle is committed (R19.3)"
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
                RECEIPT.to_owned(),
                "0000000000000000".to_owned(),
            ],
            vec![
                "passed".to_owned(),
                handed[1].0.clone(),
                "7".to_owned(),
                "application/json".to_owned(),
                RECEIPT.to_owned(),
                "0000000000000001".to_owned(),
            ],
        ]
    );
    // The second request was handed the first check's verdict and its evidence — the runtime's
    // own record of the mismatched run, citing the four records it committed.
    let asked = asked.borrow();
    assert_eq!(asked[0].previous, None);
    assert_previous(
        asked.get(1).map(|seen| &seen.previous),
        VerificationVerdict::Failed,
        "failed",
    )?;
    let declared = &rig.profile.declared.workspaces[0];
    assert_asked_whole(&rig, &asked, &declared.baseline_digest)?;
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
    // N4, store side: a failed attempt and then a passed one leave 89 objects against the one-check
    // rig's 72 — a further attempt costs 17 (DS17).
    assert_eq!(artifact_count(&rig)?, 89);
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
            RECEIPT.to_owned(),
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
        vec![vec!["1".to_owned(), RECEIPT.to_owned(), "1".to_owned()]]
    );
    let previous = asked.borrow()[1].previous.clone().ok_or("no previous")?;
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
fn refusal_cases<'a>(other_criteria: &'a str, wrong: &'a str) -> [(Shape<'a>, u64, Refusal); 12] {
    [
        // R17 round 2, N2 · the shared plan refuses before dispatch: the compiler's bytes are not
        // the pin the profile declares, named as the stop `plan_pin_compiler`.
        (
            Shape {
                compiler_sha256: Some(wrong),
                ..Shape::default()
            },
            5_000,
            Refusal::Plan("plan_pin_compiler"),
        ),
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
    // The work reservation holds the dispatch's own plan (charged to attempt 1) and leaves a
    // window the hook then sleeps past: the overrun is the candidate's, not the plan's.
    let rig = rig(&Shape {
        work_ms: 1_200,
        teardown_ms: 0,
        ..Shape::default()
    })?;
    let principal = owner();
    let (mut source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    source.hook = Some(Box::new(|| {
        std::thread::sleep(Duration::from_millis(1_400));
    }));
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
    assert_eq!(rig.reserved_work_ms, 1_200);
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
            RECEIPT.to_owned(),
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

/// One cited run record as a receipt's artifact row names it: `(kind, artifact id, digest, bytes)`.
type CitedRecord = (String, String, String, String);

/// The `run_record:<kind>` rows of a receipt's artifact inventory, read back object by object
/// through the rig's store, in kind order.
fn receipt_run_records(
    rig: &Rig,
    root: &serde_json::Value,
) -> Result<Vec<CitedRecord>, Box<dyn Error>> {
    let inventory = object_json(
        rig,
        root["artifacts"]["inventory"]["sha256"]
            .as_str()
            .ok_or("an inventory reference")?,
    )?;
    assert_eq!(
        inventory["next"]["unavailable_reason"], "end_of_inventory",
        "one page"
    );
    let mut cited: Vec<CitedRecord> = inventory["rows"]
        .as_array()
        .ok_or("inventory rows")?
        .iter()
        .filter_map(|row| {
            let kind = row["role"].as_str()?.strip_prefix("run_record:")?;
            Some((
                kind.to_owned(),
                row["object"]["artifact_id"].as_str()?.to_owned(),
                row["object"]["sha256"].as_str()?.to_owned(),
                row["object"]["byte_length"].as_u64()?.to_string(),
            ))
        })
        .collect();
    cited.sort();
    Ok(cited)
}

/// The receipt's own decision under the double, whole: INVALID, its reasons in order (six unsourced
/// identities — R16-G3 — the accounting of a run with no producer, the unsettled resources); the
/// case cites the reviewed design the plan named (the profile's `[reviewed] review`) and nothing else,
/// and the oracle result cites the producer's streams only — none.
fn assert_receipt_decision(rig: &Rig, root: &serde_json::Value) -> Result<(), Box<dyn Error>> {
    assert_eq!(root["verdict"]["state"], "INVALID");
    // The decision's reasons, whole: six unsourced identities (R16-G3), the accounting of a run
    // with no producer, and the unsettled resources — what the double's run IS, named.
    assert_eq!(
        root["verdict"]["reasons"],
        serde_json::json!([
            "UnavailableIdentity",
            "UnavailableIdentity",
            "UnavailableIdentity",
            "UnavailableIdentity",
            "UnavailableIdentity",
            "UnavailableIdentity",
            "InvalidAccounting",
            "RequiredNotExecuted",
            "NoExecution",
            "ProducerNotStarted",
            "StdoutMissing",
            "StderrMissing",
            "DiagnosticsUnavailable",
            "ResourcesUnsettled",
            "EvidenceIncomplete",
        ])
    );
    // The case cites the reviewed design the plan named (the profile's `[reviewed] review`) and
    // nothing else under the double; the oracle result cites the producer's streams only — none.
    let review = serde_json::json!({
        "artifact_id": "a47470c5-f11c-4f64-9ac8-6dcf80750ed6",
        "sha256": "sha256:f288225476120254f5c3a93266fc8f2a8763810462fbddd7c62161107cb39adb",
        "byte_length": 1145,
        "media_type": "application/json",
        "schema_id": "hee3.receipt/1:ReviewV1",
    });
    let cases = object_json(
        rig,
        root["cases"]["inventory"]["sha256"]
            .as_str()
            .ok_or("a case page")?,
    )?;
    assert_eq!(
        cases["rows"][0]["raw_evidence_refs"],
        serde_json::json!([review])
    );
    let oracle = object_json(
        rig,
        root["verdict"]["oracle_result"]["sha256"]
            .as_str()
            .ok_or("an oracle result")?,
    )?;
    assert_eq!(oracle["raw_evidence_refs"], serde_json::json!([]));
    Ok(())
}

/// The passed check's readbacks, outcome and cleanup records, decoded from their committed objects:
/// the attempt they belong to, both subjects read back, no outputs (the model's run retains none);
/// matched with no steps; the three cleanup predicates settled and the aggregate — which the runtime
/// does not own — unknown (R15 round 2, MEDIUM-10).
fn assert_run_records_decoded(rig: &Rig, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
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
/// back through the receipt's graph (R17 2c-iii, `cited_receipt`): the evidence — the receipt
/// itself — cites exactly the committed set — kind, artifact id,
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
            RECEIPT.to_owned(),
            "0000000000000001".to_owned(),
        ]
    );
    // The evidence is the receipt (R17 2c-iii): its root is committed under the verification's
    // artifact id, which is its `run_id`; its artifact inventory cites exactly the committed set as
    // `run_record:<kind>` rows; the ledger's `passed` is the outcome-derived verdict (R16-G3) while
    // the receipt's own decision is INVALID — the double never runs a producer, and the decision
    // says so by name.
    let evidence = rows(
        &rig,
        "SELECT v.evidence_digest, v.evidence_artifact_id FROM verifications v \
         JOIN attempts a ON a.id=v.attempt_id WHERE a.task_id=?",
    )?;
    let root = object_json(&rig, &evidence[0][0])?;
    assert_eq!(root["protocol"], "hee3.receipt");
    assert_eq!(root["identity"]["run_id"], evidence[0][1].as_str());
    assert_receipt_decision(&rig, &root)?;
    // N4, store side: a one-check task leaves 72 objects in the ledger's registry — the roster and
    // the acceptance manifest inside the count (the rig with a failed attempt before the passed one
    // leaves 89: a further attempt costs 17). A fresh store admits about 4096 / 72 = 56 such tasks
    // (DS17).
    assert_eq!(artifact_count(&rig)?, 72);
    let cited = receipt_run_records(&rig, &root)?;
    assert_eq!(
        cited,
        records
            .iter()
            .map(|row| (
                row[0].clone(),
                row[1].clone(),
                row[2].clone(),
                row[3].clone()
            ))
            .collect::<Vec<_>>(),
        "the receipt's run_record rows are the committed set, in kind order"
    );
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
    assert_run_records_decoded(&rig, &records)?;
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
            RECEIPT.to_owned(),
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
    let verifier = LiveVerifier::new(bad_pin_scopes());
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::VerifierError))
    );
    assert_eq!(state(&rig)?, "failed");
    let found = verifications(&rig)?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0][0], "error");
    assert_eq!(found[0][4], RECEIPT);
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

// ---- B14a-4 · the native candidate source end to end over the shared offline fixture (R18 proof (d)).

const REFERENCE_LIB: &str =
    include_str!("../evaluation/tasks/WL-U64-PARSE-001/v1/reference/src/lib.rs");
const EVAL_TASK: &[u8] = include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/TASK.md");
const EVAL_CARGO: &[u8] = include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/base/Cargo.toml");
/// The measured DS13 prompt (retained fixture; its renderer is
/// `T00-plan-20260926/DS13-frame-renderer-20260926.py`), the whole second prompt is compared against.
const MEASURED: &str = include_str!("fixtures/native/ds13-frame-prompt.txt");
/// The reviewed closure's workload record: the pins the class prompt's inputs are read against.
const WORKLOAD_RECORD: &[u8] = include_bytes!(
    "fixtures/reviewed-003/a87e5ba9f699168556ef0859c0690113f0e1186592109dd797745593aff99121"
);
const NATIVE_MODEL: &str = "hee3-t28-native:qualification";
const NATIVE_LOADED: &str = "hee3-t28-native-loaded:qualification";

fn closure_pins() -> Result<FilePins, Box<dyn Error>> {
    let record: serde_json::Value = serde_json::from_slice(WORKLOAD_RECORD)?;
    let pin = |name: &str| -> Result<String, Box<dyn Error>> {
        Ok(format!(
            "sha256:{}",
            record["files_sha256"][name]
                .as_str()
                .ok_or("a files_sha256 entry")?
        ))
    };
    Ok(FilePins {
        task: pin("TASK.md")?,
        cargo: pin("base/Cargo.toml")?,
        base: pin("base/src/lib.rs")?,
    })
}

/// A native source over the shared fixture under the rig's scratch: the fake answers `response`
/// with `done_reason` to any prompt (the scenario's prompt is null — attempts differ by history),
/// under the `/2` row's literals, with the resident model at `context_length`.
fn native_source(
    rig: &Rig,
    stand_in: &DaemonStandIn,
    answers: &[(&str, &str)],
    context_length: u64,
) -> Result<NativeCandidates, Box<dyn Error>> {
    let root = rig.scratch.0.join("native");
    let (profile, mut scenario) =
        t08_rig::fixture(&root, stand_in, NATIVE_MODEL, NATIVE_LOADED, "", (552, 258));
    scenario["prompt"] = serde_json::Value::Null;
    scenario["expect"] =
        serde_json::json!({"raw": false, "options": {"num_ctx": 4096, "num_predict": 1024}});
    scenario["ps"]["models"][0]["context_length"] = serde_json::json!(context_length);
    // One generate answer per attempt, in order (the fake serves a list across calls).
    let template = scenario["generated"].clone();
    scenario["generated"] = serde_json::Value::Array(
        answers
            .iter()
            .map(|(response, done_reason)| {
                let mut answer = template.clone();
                answer["response"] = serde_json::json!(response);
                answer["done_reason"] = serde_json::json!(done_reason);
                answer
            })
            .collect(),
    );
    fs::write(root.join("scenario.json"), serde_json::to_vec(&scenario)?)?;
    let prompt = ClassPrompt::new(EVAL_TASK, EVAL_CARGO, BASE_LIB, &closure_pins()?)
        .map_err(|e| format!("{e:?}"))?;
    // The first attempt's prompt is the rendering over no history; pinned here as the runtime's own.
    assert!(
        render(&prompt, None)
            .map_err(|e| format!("{e:?}"))?
            .starts_with(std::str::from_utf8(EVAL_TASK)?)
    );
    Ok(NativeCandidates::new(profile, FULL_FILE, prompt))
}

fn stop_body(rig: &Rig) -> Result<serde_json::Value, Box<dyn Error>> {
    let digest = rows(
        rig,
        "SELECT evidence_digest FROM task_stops WHERE task_id=?",
    )?;
    object_json(rig, &digest[0][0])
}

/// R18 (d) · the native source drives a task to acceptance: the fake hands back the reference file
/// bare, the runtime applies it, and the verifier double is handed exactly the reference's bytes as
/// the editable — the candidate the model produced is the candidate the check saw.
#[test]
fn a_native_source_over_the_fake_drives_a_task_to_acceptance() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let stand_in = DaemonStandIn::spawn();
    let source = native_source(&rig, &stand_in, &[(REFERENCE_LIB, "stop")], 4096)?;
    let principal = owner();
    let (verifier, handed) = oracle(vec![matched(7)]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert_eq!(state(&rig)?, "accepted");
    let handed = handed.borrow();
    assert_eq!(handed.len(), 1);
    assert_eq!(
        handed[0].1,
        REFERENCE_LIB.as_bytes(),
        "the editable the check saw"
    );
    assert_eq!(verifications(&rig)?[0][0], "passed");
    // B14a-5 (R19.6a) · the worker's settle, committed by the attempt's settle, read back WHOLE with
    // the wall aside: the fake's scenario is the independent source of the counts and digests.
    let attempt_ids = rows(
        &rig,
        "SELECT id FROM attempts WHERE task_id=? ORDER BY CAST(generation AS INTEGER)",
    )?;
    let scenario = native_scenario(&rig)?;
    let settles = worker_settles(&rig)?;
    assert_eq!(settles.len(), 1);
    assert_eq!(settles[0].attempt, attempt_ids[0][0]);
    let (record, _wall) = without_wall(settles[0].record.clone())?;
    assert_eq!(
        record,
        scripted_settle(
            &attempt_ids[0][0],
            &scenario,
            0,
            &serde_json::json!("replacement"),
            Some(REFERENCE_LIB.len())
        )
    );
    Ok(())
}

/// R18 (d) · a native answer the grammar refuses is recorded as a refused candidate, whole: verdict
/// Failed at no cost, the refusal by its `candidate_*` name, `candidate_sha256` the digest of the text
/// the model returned; no verifier call for it. The fake then answers the second attempt with the
/// reference file, which the check passes: two attempts, two verifications, the task accepted.
#[test]
fn a_refused_native_answer_is_recorded_as_a_refused_candidate_whole() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let stand_in = DaemonStandIn::spawn();
    let source = native_source(
        &rig,
        &stand_in,
        &[("   \n", "stop"), (REFERENCE_LIB, "stop")],
        4096,
    )?;
    let principal = owner();
    let (verifier, handed) = oracle(vec![matched(7)]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert_eq!(
        handed.borrow().len(),
        1,
        "no verifier call for the refused candidate"
    );
    assert_eq!(handed.borrow()[0].1, REFERENCE_LIB.as_bytes());
    let found = verifications(&rig)?;
    assert_eq!(found.len(), 2);
    assert_eq!(
        [found[0][0].clone()]
            .into_iter()
            .chain(found[0][2..].iter().cloned())
            .collect::<Vec<_>>(),
        vec![
            "failed".to_owned(),
            "0".to_owned(),
            "application/json".to_owned(),
            "hee3.refused-candidate/1".to_owned(),
            "0000000000000000".to_owned(),
        ]
    );
    assert_eq!(found[1][0], "passed");
    // The second attempt's prompt, as the fake captured it: the history names the first attempt's
    // refusal (F6, end to end) and the frame's other parts are the class's.
    let captured: serde_json::Value = serde_json::from_slice(&fs::read(
        rig.scratch.0.join("native/captured-request.json"),
    )?)?;
    let prompt = captured["prompt"].as_str().ok_or("a captured prompt")?;
    // Whole: the measured DS13 frame (a retained fixture) with its history line swapped.
    assert_eq!(
        prompt,
        MEASURED.replacen(
            "\n\nFirst attempt.\n\n",
            "\n\nPrevious attempt: failed, 0 criteria satisfied, refused as candidate_empty.\n\n",
            1
        )
    );
    assert_eq!(captured["raw"], false);
    let evidence = rows(
        &rig,
        "SELECT v.evidence_digest FROM verifications v JOIN attempts a ON a.id=v.attempt_id \
         WHERE a.task_id=? ORDER BY CAST(a.generation AS INTEGER)",
    )?;
    assert_eq!(
        object_json(&rig, &evidence[0][0])?,
        serde_json::json!({
            "kind": "refused_candidate",
            "refusal": "candidate_empty",
            "candidate_sha256": t08_rig::digest(b"   \n"),
        })
    );
    // B14a-5 (R19.6b) · two settles, one per attempt: the refusal by its name with no length, then
    // the replacement; each attempt's raw digest is its own scripted answer's.
    let attempt_ids = rows(
        &rig,
        "SELECT id FROM attempts WHERE task_id=? ORDER BY CAST(generation AS INTEGER)",
    )?;
    let scenario = native_scenario(&rig)?;
    let settles = worker_settles(&rig)?;
    assert_eq!(settles.len(), 2);
    let (first, _) = without_wall(settles[0].record.clone())?;
    assert_eq!(
        first,
        scripted_settle(
            &attempt_ids[0][0],
            &scenario,
            0,
            &serde_json::json!({"refused": {"name": "candidate_empty"}}),
            None
        )
    );
    let (second, _) = without_wall(settles[1].record.clone())?;
    assert_eq!(
        second,
        scripted_settle(
            &attempt_ids[1][0],
            &scenario,
            1,
            &serde_json::json!("replacement"),
            Some(REFERENCE_LIB.len())
        )
    );
    assert_ne!(
        settles[0].artifact_id, settles[1].artifact_id,
        "each record under its own artifact id"
    );
    Ok(())
}

/// R18 (d), A3, A9 · a provider failure stops the task `worker_failed` after ONE ask — the resident
/// model at the qualified 512 context under the `/2` profile is `identity` at the readback before any
/// generate — and the stop body's `worker` field names it whole; no verification was recorded.
#[test]
fn a_provider_failure_stops_the_task_with_the_worker_named() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let stand_in = DaemonStandIn::spawn();
    let source = native_source(&rig, &stand_in, &[(REFERENCE_LIB, "stop")], 512)?;
    let principal = owner();
    let (verifier, handed) = oracle(vec![]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::WorkerFailed))
    );
    assert_eq!(state(&rig)?, "failed");
    assert!(handed.borrow().is_empty());
    assert_eq!(
        rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?,
        vec![vec!["worker_failed".to_owned()]]
    );
    let body = stop_body(&rig)?;
    assert_eq!(body["reason"], "worker_failed");
    assert_eq!(body["attempts"], 1);
    assert_eq!(
        body["worker"],
        serde_json::json!({
            "provider": "ollama-local",
            "error": "identity",
            "state": "not_dispatched",
            "retained": 0,
        })
    );
    // B14a-5 (R19.6c) · the failed attempt's settle names the provider failure; nothing was read
    // from the model, so no token, finish or digest is recorded.
    let attempt_ids = rows(&rig, "SELECT id FROM attempts WHERE task_id=?")?;
    let settles = worker_settles(&rig)?;
    assert_eq!(settles.len(), 1);
    let (record, _) = without_wall(settles[0].record.clone())?;
    assert_eq!(
        record,
        serde_json::json!({
            "attempt": attempt_ids[0][0],
            "adapter_profile": "ollama-fc44-12ff8654/2",
            "input_tokens": null,
            "output_tokens": null,
            "finish": null,
            "identity_sha256": null,
            "raw_sha256": null,
            "outcome": {"provider": {"name": "identity"}},
            "replacement_bytes": null,
        })
    );
    Ok(())
}

/// R18 (d), A2 · a work reservation past the adapter's own cap: the source passes the window through,
/// the adapter refuses `deadline` at its door before any exchange, and the task stops `worker_failed`
/// with the refusal named — the cap is met by name, never clamped to fit.
#[test]
fn a_work_reservation_past_the_adapter_cap_is_refused_by_name() -> Outcome_ {
    // The rig's submission limit is 1,200,000 ms: 950 s of work and 200 s of verify fit inside it,
    // and the work window (950 s less the teardown share) sits past the adapter's 900 s cap.
    let rig = rig(&Shape {
        work_ms: 950_000,
        verify_ms: 200_000,
        ..Shape::default()
    })?;
    let stand_in = DaemonStandIn::spawn();
    let source = native_source(&rig, &stand_in, &[(REFERENCE_LIB, "stop")], 4096)?;
    let principal = owner();
    let (verifier, _) = oracle(vec![]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::WorkerFailed))
    );
    let body = stop_body(&rig)?;
    assert_eq!(body["worker"]["error"], "deadline");
    assert_eq!(body["worker"]["state"], "not_dispatched");
    assert!(
        !rig.scratch.0.join("native/calls.log").exists(),
        "no exchange ran"
    );
    Ok(())
}
