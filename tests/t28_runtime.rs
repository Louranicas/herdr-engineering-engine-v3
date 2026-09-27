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
use habitat_engine::app::live_verifier::{Aggregates, LiveVerifier};
use habitat_engine::app::native_provider::{self, Installed};
use habitat_engine::app::runtime::{
    Admission, Admitted, Answer as SourceAnswer, CHECK_TEARDOWN, Candidate, CandidateSource,
    CheckPlan, CheckWindow, Custody, Dispatch, Error as RuntimeError, Observed, Outcome, Previous,
    Readiness, Refusal, Resources, Unlaunched, Verifier, admit, dispatch, drive,
};
use habitat_engine::app::tasks::StoreTasks;
use habitat_engine::app::workload::{self, Outcome as RunOutcome, Run};
use habitat_engine::check::consistency::U64_CRITERIA;
use habitat_engine::check::u64_oracle::Evaluation;
use habitat_engine::contracts::control::{
    CancelReason, Precondition, ResourceKind, criteria_digest,
};
use habitat_engine::contracts::receipt::{Address as _, ReceiptV1};
use habitat_engine::contracts::roster::{Kind, Locality, RosterDefinitionV1, Selection, Update};
use habitat_engine::contracts::{Sha256Digest, UuidV4};
use habitat_engine::store::{
    Allocation, Principal, RequestSource, Store, Submission, VerificationVerdict,
};
use habitat_engine::task::TASK_LIMIT;
use habitat_engine::task::control::Cancel;
use habitat_engine::task::control::Spec;
use habitat_engine::task::driver::{Outcome as Driven, StopReason};
use habitat_engine::worker::native::FULL_FILE;
use habitat_engine::worker::workspace::{Error as WorkspaceError, Snapshot};
use std::collections::VecDeque;
use std::error::Error;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError, mpsc};
use std::time::{Duration, Instant};

use super::tasks::{EPOCH, GENERATION, Scratch};

/// The schema every in-gate check records its evidence under since 2c-iii: the receipt itself
/// (R17 round 2, decision 3 — the evidence artifact id is the receipt's `run_id`). The runtime's
/// own `hee3.u64-check/1` remains only for a compose the composer refused.
const RECEIPT: &str = ReceiptV1::SCHEMA_ID;

type Outcome_ = Result<(), Box<dyn Error>>;
/// What the candidate source was handed, per request.
/// One request the script double was handed (F101: a model that records what it was given): the
/// previous verification, the attempt's identity, the invocation id the runtime minted, the
/// recipe and workspace digests, and the instant the attempt's work window is charged from.
#[derive(Clone, Debug)]
struct Seen {
    previous: Option<Previous>,
    attempt: String,
    generation: u64,
    invocation: String,
    recipe: String,
    workspace: String,
    charged_from: Instant,
}
type Asked = Arc<Mutex<Vec<Seen>>>;
/// What the verifier was handed, per check: the snapshot's content digest, its editable bytes,
/// the check window, and when the double was called (a model, not a script: F101).
type Handed = Arc<Mutex<Vec<(String, Vec<u8>, CheckWindow, Instant)>>>;

/// What a double recorded so far, copied out (a poisoned log still yields what it holds: a double
/// that panicked mid-record must not hide what it saw before).
fn taken<T: Clone>(log: &Mutex<Vec<T>>) -> Vec<T> {
    log.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// Record one value in a double's log; returns how many it now holds.
fn record<T>(log: &Mutex<Vec<T>>, value: T) -> usize {
    let mut log = log.lock().unwrap_or_else(PoisonError::into_inner);
    log.push(value);
    log.len()
}

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
    /// The id the door prepared `attempts` with, as `serve` carries it into every dispatch.
    root_id: String,
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
    /// The baseline also holds the class's `Cargo.toml` (K4: the native prompt reads it from the
    /// baseline snapshot); the rig's other proofs keep the one-file baseline they pin.
    base_cargo: bool,
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
            base_cargo: false,
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

/// The roster record the attempt runs under, installed through the roster's door. No observation is
/// written here: the runtime observes the record before each attempt, from the source's readiness
/// (R21 N4), so a proof that reaches `begin` does so on the runtime's own observation.
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
    let text = class_directory(&class, shape)?;
    Ok(Profile {
        declared: class_profile::compose(text.as_bytes()).map_err(|error| format!("{error:?}"))?,
        directory: class,
        digest: PROFILE_DIGEST.to_owned(),
    })
}

/// The class `installed` composes, as `serve` reads it through `main` (R21 S21): the rig's class
/// directory with the class's `Cargo.toml` in the baseline, and its `profile.toml` text under
/// `hee3.class-profile/2` with the native row naming `manifest_sha256` and the `/2` adapter.
pub(super) fn native_class_text(
    class: &Path,
    manifest_sha256: &str,
) -> Result<String, Box<dyn Error>> {
    let text = class_directory(
        class,
        &Shape {
            base_cargo: true,
            ..Shape::default()
        },
    )?;
    let v1 = "schema = \"hee3.class-profile/1\"\n";
    assert_eq!(text.matches(v1).count(), 1, "one schema line");
    Ok(format!(
        "{}\n[native]\nmodel = \"{NATIVE_MODEL}\"\nmanifest_sha256 = \"{manifest_sha256}\"\nadapter = \"{}\"\n",
        text.replacen(v1, "schema = \"hee3.class-profile/2\"\n", 1),
        FULL_FILE.id
    ))
}

/// The workspace the rig's class declares, as a task spec names it.
pub(super) const CLASS_WORKSPACE: &str = WORKSPACE;

/// Populate `class` (created 0700) as the rig's class directory and return its profile text.
fn class_directory(class: &Path, shape: &Shape<'_>) -> Result<String, Box<dyn Error>> {
    private(class)?;
    let (base, protected) = (class.join("base"), class.join("protected"));
    private(&base)?;
    private(&base.join("src"))?;
    file(&base.join("src/lib.rs"), BASE_LIB)?;
    if shape.base_cargo {
        file(&base.join("Cargo.toml"), EVAL_CARGO)?;
    }
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
    Ok(text)
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
    // Created and marked through the one door, as `serve` makes it (B14b-2 closure C18); its id is
    // kept, as `serve` keeps it (B14b-2 review round 2, D9).
    let root_id = habitat_engine::app::coordinator::prepare_attempts_root(&attempts, deadline())
        .map_err(|e| format!("{e:?}"))?;
    // The dispatcher reads the class profile from the task owner (R20 round 2 A11): install it there.
    let tasks = StoreTasks::new(store, EPOCH.to_owned()).with_class_profile(Ok(profile.clone()));
    Ok(Rig {
        tasks,
        scratch,
        profile,
        attempts,
        root_id,
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
    hook: Option<Box<dyn FnMut() + Send + 'h>>,
    /// A worker settle to attach to every answer — a script that claims a provider was asked (the
    /// identity-refusal proof hands one naming another attempt).
    settle: Option<Settle>,
    /// The readiness `ready` answers with (R21 N4), and the custody `settle_retained` answers with
    /// (N18): scripted, so a proof can arrange either.
    readiness: Result<Readiness, habitat_engine::worker::native::Error>,
    custody: Custody,
    /// Every `ready` and `settle_retained` call: which, the deadline and the cancellation it was
    /// handed (F101). The runtime asks `ready` before each attempt's begin (R21 N4).
    readied: Readied,
    /// Run inside every `ready`, after its call is recorded, with the flag `ready` was handed (closure
    /// C3): a proof raises the engine's drain here, and records which flag the source was handed.
    ready_hook: Option<ReadyHook<'h>>,
    /// A model of the retained children (closure C4): handed the deadline `settle_retained` was
    /// handed, it answers the custody in place of the scripted `custody`.
    settle_model: Option<SettleModel<'h>>,
}

/// What a source's `ready`/`settle_retained` were handed, per call: which, the deadline, the flag.
type Readied = Arc<Mutex<Vec<(&'static str, Instant, bool)>>>;

/// What a proof runs inside a source's `ready`, handed the flag `ready` was handed (closure C3).
type ReadyHook<'h> = Box<dyn FnMut(&AtomicBool) + Send + 'h>;

/// A model of a source's retained children: the custody one settle turn comes to under the deadline
/// it was handed (closure C4).
type SettleModel<'h> = Box<dyn FnMut(Instant) -> Custody + Send + 'h>;

impl CandidateSource for Script<'_> {
    fn next(&mut self, ask: &habitat_engine::app::runtime::Ask<'_>) -> SourceAnswer {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Seen {
                previous: ask.previous.cloned(),
                attempt: ask.attempt.as_str().to_owned(),
                generation: ask.generation.value(),
                invocation: ask.invocation.as_str().to_owned(),
                recipe: ask.recipe.as_str().to_owned(),
                workspace: ask.workspace.as_str().to_owned(),
                charged_from: ask.charged_from,
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
    fn ready(
        &mut self,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<Readiness, habitat_engine::worker::native::Error> {
        record(
            &self.readied,
            ("ready", deadline, cancelled.load(Ordering::Acquire)),
        );
        if let Some(hook) = self.ready_hook.as_mut() {
            hook(cancelled);
        }
        self.readiness.clone()
    }
    fn settle_retained(&mut self, deadline: Instant, cancelled: &AtomicBool) -> Custody {
        record(
            &self.readied,
            (
                "settle_retained",
                deadline,
                cancelled.load(Ordering::Acquire),
            ),
        );
        match self.settle_model.as_mut() {
            Some(model) => model(deadline),
            None => self.custody,
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
    hook: Option<Box<dyn FnMut() + Send + 'h>>,
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
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((
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
        let run = answer
            .run
            .map(|outcome| {
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
            })
            .map_err(Unlaunched::Workload);
        Observed {
            run,
            observed,
            resources: Resources::NotHeld,
        }
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
    let seen = Arc::new(Mutex::new(Vec::new()));
    (
        Script {
            answers: answers.into(),
            seen: Arc::clone(&seen),
            hook: None,
            settle: None,
            readiness: Ok(Readiness {
                actual_identity: "fixture/worker".to_owned(),
                immutable_revision: None,
                capabilities: vec!["text".to_owned()],
                evidence: b"fixture catalogue".to_vec(),
            }),
            custody: Custody::default(),
            readied: Arc::new(Mutex::new(Vec::new())),
            ready_hook: None,
            settle_model: None,
        },
        seen,
    )
}

fn oracle<'h>(answers: Vec<Answer>) -> (Oracle<'h>, Handed) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    (
        Oracle {
            answers: answers.into(),
            seen: Arc::clone(&seen),
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
            root_id: &rig.root_id,
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
    assert!(taken(&handed).is_empty(), "no check was asked");
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
/// asked to open and what it was handed of the admitted state, serves a scripted source/verifier
/// pair per open in order, and raises the stop flag once it has served `stop_after` pairs — the
/// dispatcher then ends its wait `Drained`. It owns all it holds (`Send + 'static`), so the
/// dispatcher runs on a thread of its own (R21 N21).
struct ScriptedProvider {
    pairs: VecDeque<(Script<'static>, Oracle<'static>)>,
    opened: Arc<Mutex<Vec<String>>>,
    admitted: Arc<Mutex<Vec<Opened>>>,
    stop: Arc<AtomicBool>,
    stop_after: usize,
}

/// What one `open` was handed of the admitted state (R21 N1, F101): the dispatch window, the
/// baseline's content digest, the class profile's digest, and whether the drain it was handed is
/// the dispatcher's own flag (the one this double raises).
#[derive(Clone, Debug, Eq, PartialEq)]
struct Opened {
    window: (Instant, Instant),
    baseline: Option<String>,
    profile: String,
    drain_is_the_dispatcher_s: bool,
}

impl dispatcher::Provider for ScriptedProvider {
    type Source = Script<'static>;
    type Verifier = Oracle<'static>;
    fn open(
        &mut self,
        next: &habitat_engine::store::Dispatchable,
        admitted: &habitat_engine::app::runtime::Admitted<'_>,
    ) -> Result<(Script<'static>, Oracle<'static>), dispatcher::Unavailable> {
        record(
            &self.admitted,
            Opened {
                window: admitted.window(),
                baseline: admitted.baseline().content_digest(),
                profile: admitted.profile().digest.clone(),
                drain_is_the_dispatcher_s: std::ptr::eq(admitted.drain(), Arc::as_ptr(&self.stop)),
            },
        );
        if record(&self.opened, next.task.clone()) >= self.stop_after {
            self.stop.store(true, Ordering::SeqCst);
        }
        self.pairs
            .pop_front()
            .ok_or(dispatcher::Unavailable::NoNativeProvider(
                dispatcher::NoNative::NotInstalled,
            ))
    }
}

/// A scripted provider over `pairs` that never raises the stop itself, and the log of what it opened.
fn provider_of(
    pairs: Vec<(Script<'static>, Oracle<'static>)>,
    stop: &Arc<AtomicBool>,
) -> (ScriptedProvider, Arc<Mutex<Vec<String>>>) {
    let opened = Arc::new(Mutex::new(Vec::new()));
    (
        ScriptedProvider {
            pairs: pairs.into(),
            opened: Arc::clone(&opened),
            admitted: Arc::new(Mutex::new(Vec::new())),
            stop: Arc::clone(stop),
            stop_after: usize::MAX,
        },
        opened,
    )
}

/// The budget every dispatcher proof runs under (F102, closure H6): past it the helper raises the
/// stop, wakes the wait and FAILS the proof by name instead of hanging.
const DISPATCHER_BUDGET: Duration = Duration::from_secs(20);

/// Run the dispatcher over the rig with the rig's own selections, under `DISPATCHER_BUDGET`.
fn run_dispatcher<P: dispatcher::Provider + Send + 'static>(
    rig: &Arc<Rig>,
    provider: P,
    stop: &Arc<AtomicBool>,
) -> Result<(dispatcher::Exit, Vec<String>), String> {
    run_dispatcher_owned(
        Arc::clone(rig),
        provider,
        Arc::clone(stop),
        rig.selections.clone(),
        DISPATCHER_BUDGET,
    )
}

/// Run the dispatcher over the rig until it exits, with `stop` as both the engine's drain and the
/// between-attempts drain flag; every step it reports is collected. The dispatcher runs on a thread
/// of its own over owned state (R21 N21: an unscoped spawn, since a scoped thread is joined and so
/// cannot be left behind), and this thread waits for its exit on one deadline, `budget`. Past it the
/// stop is raised and the wait woken — a dispatcher that honours its stop then ends by itself — and
/// the proof fails by name, `the dispatcher did not end within <budget>`, whether the dispatcher
/// honours the stop or not: one that ignores it is left running and cannot hang the proof.
fn run_dispatcher_owned<P: dispatcher::Provider + Send + 'static>(
    rig: Arc<Rig>,
    provider: P,
    stop: Arc<AtomicBool>,
    selections: Vec<Selection>,
    budget: Duration,
) -> Result<(dispatcher::Exit, Vec<String>), String> {
    let (report_to, reported) = mpsc::channel::<String>();
    let (exit_to, exited) = mpsc::channel();
    // The thread owns the rig and the stop; this side keeps its own handles to raise the stop and
    // wake the wait at the budget.
    let (waker_rig, waker_stop) = (Arc::clone(&rig), Arc::clone(&stop));
    let handle = std::thread::Builder::new()
        .name("t28-dispatcher".to_owned())
        .spawn(move || {
            let mut provider = provider;
            // The receiver outlives every send until the proof gives up on this thread; a line sent
            // after that has no reader, which is the point of giving up.
            let report = move |line: &str| {
                let _ = report_to.send(line.to_owned());
            };
            let exit = dispatcher::Dispatcher {
                tasks: &rig.tasks,
                attempts: &rig.attempts,
                root_id: &rig.root_id,
                provider: &mut provider,
                agent_record_id: &rig.agent,
                selections: &selections,
                drain: &stop,
            }
            .run(&report);
            let _ = exit_to.send(exit);
        })
        .map_err(|error| format!("the dispatcher thread did not start: {error}"))?;
    match exited.recv_timeout(budget) {
        Ok(exit) => {
            handle
                .join()
                .map_err(|_| "the dispatcher thread panicked after its exit".to_owned())?;
            Ok((exit, reported.try_iter().collect()))
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            waker_stop.store(true, Ordering::SeqCst);
            waker_rig.tasks.wake();
            let lines: Vec<String> = reported.try_iter().collect();
            Err(format!(
                "the dispatcher did not end within {budget:?}: {lines:?}"
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            let lines: Vec<String> = reported.try_iter().collect();
            let why = handle.join().err().map(|payload| {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                    .unwrap_or_default()
            });
            Err(format!(
                "the dispatcher thread ended without an exit ({why:?}): {lines:?}"
            ))
        }
    }
}

/// A provider whose `open` sleeps `nap` and reads no stop: the stand-in for a dispatcher that
/// ignores its drain. It records the task it was handed, sent when its `open` returns.
struct Sleeper {
    nap: Duration,
    woke: mpsc::Sender<String>,
}

impl dispatcher::Provider for Sleeper {
    type Source = Script<'static>;
    type Verifier = Oracle<'static>;
    fn open(
        &mut self,
        next: &habitat_engine::store::Dispatchable,
        _admitted: &habitat_engine::app::runtime::Admitted<'_>,
    ) -> Result<(Script<'static>, Oracle<'static>), dispatcher::Unavailable> {
        std::thread::sleep(self.nap);
        let _ = self.woke.send(next.task.clone());
        Err(dispatcher::Unavailable::NoNativeProvider(
            dispatcher::NoNative::NotInstalled,
        ))
    }
}

/// R21 N21 (D9), the helper's own control · a dispatcher that ignores its stop fails the proof by
/// name at the budget instead of hanging it: the provider's `open` sleeps 1 s under a 200 ms budget
/// (Q15: short, to spare `t21_process`'s margin). The helper answers `Err` naming the budget before
/// the sleep ends, having raised the stop; the thread it left behind ends by itself — its `open`
/// returns the task it was handed and it releases the rig (budgeted, F102) — and holds no stand-in
/// process, so no descendant outlives the proof.
#[test]
fn a_dispatcher_that_ignores_its_stop_fails_the_proof_by_name() -> Outcome_ {
    let rig = Arc::new(rig(&Shape::default())?);
    let stop = Arc::new(AtomicBool::new(false));
    let (woke, opened) = mpsc::channel();
    let nap = Duration::from_secs(1);
    let started = Instant::now();
    let result = run_dispatcher_owned(
        Arc::clone(&rig),
        Sleeper { nap, woke },
        Arc::clone(&stop),
        rig.selections.clone(),
        Duration::from_millis(200),
    );
    let failed_after = started.elapsed();
    let message = result
        .err()
        .ok_or("the proof passed a dispatcher that ignored its stop")?;
    assert!(
        message.starts_with("the dispatcher did not end within 200ms"),
        "{message}"
    );
    assert!(
        failed_after < nap,
        "failed at the budget, not after the sleep: {failed_after:?} against {nap:?}"
    );
    assert!(stop.load(Ordering::SeqCst), "the helper raised the stop");
    let budget = Duration::from_secs(5);
    assert_eq!(opened.recv_timeout(budget)?, TASK, "the left thread's open");
    let released = Instant::now();
    while Arc::strong_count(&rig) > 1 {
        assert!(
            released.elapsed() < budget,
            "the left thread still holds the rig after {:?} (budget {budget:?}): {} holders",
            released.elapsed(),
            Arc::strong_count(&rig)
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

/// B14b-1 (b) · the dispatcher picks the admitted task, drives it to ACCEPTED through the scripted
/// pair, notifies, and ends `Drained` when the drain is set; the provider was opened once, for that
/// task. R21 N1 · `open` was handed the admitted state: a window `TASK_LIMIT` wide, the baseline the
/// rig installed (its digest read here by a capture of its own), the rig's class profile and the
/// dispatcher's own drain; and the source was asked ready under the attempt's work window (N4, as
/// closure C3 amends it).
#[test]
fn the_dispatcher_picks_the_admitted_task_and_drives_it_to_acceptance() -> Outcome_ {
    let rig = Arc::new(rig(&Shape::default())?);
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let readied = Arc::clone(&source.readied);
    let (mut verifier, _) = oracle(vec![matched(7)]);
    let stop = Arc::new(AtomicBool::new(false));
    // The drain is raised during the one check: acceptance does not read it, the next wait does —
    // so the task is accepted and the dispatcher then ends `Drained` (a stop raised at `open` would
    // be read by `begin` and drain the dispatch instead: measured, the skeleton's first run).
    let flag = Arc::clone(&stop);
    verifier.hook = Some(Box::new(move || {
        flag.store(true, Ordering::SeqCst);
    }));
    let (provider, opened) = provider_of(vec![(source, verifier)], &stop);
    let handed = Arc::clone(&provider.admitted);
    let (exit, lines) = run_dispatcher(&rig, provider, &stop)?;
    assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    assert_eq!(taken(&opened), vec![TASK.to_owned()]);
    assert_eq!(state(&rig)?, "accepted");
    assert!(
        lines
            .iter()
            .any(|line| line.contains("TaskDone(\"accepted\")")),
        "{lines:?}"
    );
    let handed = taken(&handed);
    assert_eq!(handed.len(), 1);
    let (origin, until) = handed[0].window;
    assert_eq!(until.duration_since(origin), TASK_LIMIT);
    let baseline = Snapshot::capture(&rig.profile.directory.join("base"), &[], deadline())
        .map_err(|e| format!("{e:?}"))?
        .content_digest();
    assert_eq!(
        handed[0],
        Opened {
            window: (origin, until),
            baseline,
            profile: PROFILE_DIGEST.to_owned(),
            drain_is_the_dispatcher_s: true,
        }
    );
    // Closure C3: ready is handed the attempt's work window, which closes before the dispatch;
    // closure C4: the custody turn its teardown bound, which leaves the settle the remainder.
    let teardown = u64::try_from(CHECK_TEARDOWN.as_millis())?;
    let work_until = window_end(origin, Shape::default().work_ms, teardown, until);
    let bound = teardown_bound(work_until, teardown, until);
    assert!(work_until < bound && bound < until);
    assert_eq!(
        taken(&readied),
        vec![
            ("ready", work_until, false),
            ("settle_retained", bound, false)
        ]
    );
    Ok(())
}

/// B14b-1 (b), the round-1 review's first defect · a task cancelled before any attempt is picked
/// (it is `cancellation_requested`, not `admitted`), stopped by its own name through the
/// pre-dispatch path with no attempt row, and never picked again: the provider is never opened.
#[test]
fn a_task_cancelled_before_dispatch_is_stopped_by_name_and_never_picked_again() -> Outcome_ {
    let rig = Arc::new(rig(&Shape::default())?);
    let principal = owner();
    cancel(&rig, &principal, "28f10000-0000-4000-8000-0000000000d1");
    assert_eq!(state(&rig)?, "cancellation_requested");
    let stop = Arc::new(AtomicBool::new(false));
    // No pair: a pick that reached `open` would end the loop `Unavailable`, which the assertion
    // below distinguishes from the drain.
    let (provider, opened) = provider_of(Vec::new(), &stop);
    // After the cancelled task is stopped the read returns None and the wait would block: a watcher
    // raises the stop once the ledger says `cancelled` (the budget is the helper's).
    let (exit, lines) = std::thread::scope(|scope| {
        scope.spawn(|| stop_when(&rig, TASK, "cancelled", &stop));
        run_dispatcher(&rig, provider, &stop)
    })?;
    assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    assert!(taken(&opened).is_empty(), "the provider was never opened");
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
    let rig = Arc::new(rig(&Shape::default())?);
    let stop = Arc::new(AtomicBool::new(false));
    let (exit, lines) = run_dispatcher(
        &rig,
        dispatcher::NoProvider(dispatcher::NoNative::NotInstalled),
        &stop,
    )?;
    assert_eq!(
        exit,
        dispatcher::Exit::Unavailable(dispatcher::Unavailable::NoNativeProvider(
            dispatcher::NoNative::NotInstalled
        )),
        "{lines:?}"
    );
    assert_eq!(state(&rig)?, "admitted");
    assert!(
        lines
            .iter()
            .any(|line| line.contains("unavailable: no native provider (not installed)")),
        "{lines:?}"
    );
    Ok(())
}

/// A provider whose `open` refuses with `why`, first raising the drain it was handed when `raise`
/// is set — the shape of a resolve cancelled by a SIGTERM that landed during `open`. It records the
/// task it was handed and whether the drain it raised is the dispatcher's own.
struct RefusingOpen {
    why: dispatcher::Unavailable,
    raise: bool,
    engine: Arc<AtomicBool>,
    handed: Arc<Mutex<Vec<(String, bool)>>>,
}

impl dispatcher::Provider for RefusingOpen {
    type Source = Script<'static>;
    type Verifier = Oracle<'static>;
    fn open(
        &mut self,
        next: &habitat_engine::store::Dispatchable,
        admitted: &habitat_engine::app::runtime::Admitted<'_>,
    ) -> Result<(Script<'static>, Oracle<'static>), dispatcher::Unavailable> {
        record(
            &self.handed,
            (
                next.task.clone(),
                std::ptr::eq(admitted.drain(), Arc::as_ptr(&self.engine)),
            ),
        );
        if self.raise {
            admitted.drain().store(true, Ordering::SeqCst);
        }
        Err(self.why)
    }
}

/// R21 round-1 LOW F6 · `open` is handed the drain as its resolve's cancel flag, so a SIGTERM that
/// lands during `open` comes back as `Daemon(Cancelled)`: that is the drain's exit, `Drained`, and
/// is said as a drain — never a provider unavailability. The drain decides it, not the name: the
/// same refusal with no drain raised stays the named unavailable state, and another refusal under a
/// raised drain stays its own name. The task stays `admitted` in all three.
#[test]
fn a_drain_that_cancels_open_ends_the_dispatcher_drained_not_unavailable() -> Outcome_ {
    use habitat_engine::worker::native::Error as Native;
    let cancelled = dispatcher::Unavailable::Daemon(Native::Cancelled);
    let deadline = dispatcher::Unavailable::Daemon(Native::Deadline);
    for (why, raise, exit, said) in [
        (
            cancelled,
            true,
            dispatcher::Exit::Drained,
            "dispatcher: drained while opening the provider (unavailable: daemon cancelled)",
        ),
        (
            cancelled,
            false,
            dispatcher::Exit::Unavailable(cancelled),
            "dispatcher: unavailable: daemon cancelled",
        ),
        (
            deadline,
            true,
            dispatcher::Exit::Unavailable(deadline),
            "dispatcher: unavailable: daemon deadline",
        ),
    ] {
        let rig = Arc::new(rig(&Shape::default())?);
        let stop = Arc::new(AtomicBool::new(false));
        let handed = Arc::new(Mutex::new(Vec::new()));
        let provider = RefusingOpen {
            why,
            raise,
            engine: Arc::clone(&stop),
            handed: Arc::clone(&handed),
        };
        let (got, lines) = run_dispatcher(&rig, provider, &stop)?;
        assert_eq!(
            (got, lines),
            (exit, vec![said.to_owned()]),
            "{why:?} {raise}"
        );
        assert_eq!(taken(&handed), vec![(TASK.to_owned(), true)]);
        assert_eq!(state(&rig)?, "admitted");
    }
    Ok(())
}

/// B14b-1 (b), D5 (corrected, closure item 9) · the drain observed between attempts at attempt 2:
/// the verifier's hook raises it during the first check (a mismatch), so the next `begin` sees it —
/// the dispatch ends `Drained` with no stop written, the first attempt settled and the task
/// `repair_pending`, which the read never returns: STRANDED until recovery (B17), a stated gap; the
/// dispatcher exits `Drained` at its next wait.
#[test]
fn a_drain_between_attempts_strands_the_task_in_repair_pending_with_no_stop_written() -> Outcome_ {
    let rig = Arc::new(rig(&Shape::default())?);
    let stop = Arc::new(AtomicBool::new(false));
    let (source, _) = script(vec![
        Candidate::Replacement(FIRST.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    let (mut verifier, _) = oracle(vec![mismatched(7), matched(7)]);
    let flag = Arc::clone(&stop);
    verifier.hook = Some(Box::new(move || {
        flag.store(true, Ordering::SeqCst);
    }));
    let (provider, _) = provider_of(vec![(source, verifier)], &stop);
    let (exit, lines) = run_dispatcher(&rig, provider, &stop)?;
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
    let rig = Arc::new(rig(&Shape::default())?);
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
    let stop = Arc::new(AtomicBool::new(false));
    let (provider, opened) = provider_of(Vec::new(), &stop);
    let (exit, lines) = std::thread::scope(|scope| {
        scope.spawn(|| stop_when(&rig, &readers, "failed", &stop));
        run_dispatcher(&rig, provider, &stop)
    })?;
    assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    assert!(taken(&opened).is_empty(), "no provider for a refused task");
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
    let rig = Arc::new(rig(&Shape::default())?);
    let mut stale = rig.selections.clone();
    for selection in &mut stale {
        selection.expected_revision = "99".to_owned();
    }
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![matched(7)]);
    let stop = Arc::new(AtomicBool::new(false));
    let (provider, opened) = provider_of(vec![(source, verifier)], &stop);
    let (exit, lines) = std::thread::scope(|scope| {
        scope.spawn(|| stop_when(&rig, TASK, "failed", &stop));
        run_dispatcher_owned(
            Arc::clone(&rig),
            provider,
            Arc::clone(&stop),
            stale,
            DISPATCHER_BUDGET,
        )
    })?;
    assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    assert_eq!(
        taken(&opened),
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
    let rig = Arc::new(rig(&Shape::default())?);
    let operator = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![matched(7)]);
    run(&rig, &operator, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(state(&rig)?, "accepted");
    let stop = Arc::new(AtomicBool::new(false));
    let (provider, opened) = provider_of(Vec::new(), &stop);
    let submitted = Mutex::new(None);
    let (exit, _) = std::thread::scope(|scope| {
        scope.spawn(|| {
            // Let the dispatcher reach its wait first; a submit that lands before it is seen by the
            // read instead — either way the task is picked, which is the claim.
            std::thread::sleep(Duration::from_millis(200));
            let id = submit_as(
                &rig,
                &owner(),
                "28f10000-0000-4000-8000-0000000000e2",
                U64_CRITERIA.iter().map(|c| (*c).to_owned()).collect(),
            )
            .expect("a second admission");
            if let Ok(mut slot) = submitted.lock() {
                *slot = Some(id);
            }
        });
        run_dispatcher(&rig, provider, &stop)
    })?;
    assert_eq!(
        exit,
        dispatcher::Exit::Unavailable(dispatcher::Unavailable::NoNativeProvider(
            dispatcher::NoNative::NotInstalled
        ))
    );
    let submitted = submitted
        .lock()
        .map(|slot| slot.clone())
        .unwrap_or_default();
    assert_eq!(
        taken(&opened).as_slice(),
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
    let rig = Arc::new(rig(&Shape::default())?);
    let stop = Arc::new(AtomicBool::new(true));
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
            root_id: &rig.root_id,
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
    let (provider, opened) = provider_of(vec![(source, verifier)], &stop);
    let (exit, lines) = std::thread::scope(|scope| {
        scope.spawn(|| stop_when(&rig, TASK, "accepted", &stop));
        run_dispatcher(&rig, provider, &stop)
    })?;
    assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    assert_eq!(taken(&opened), vec![TASK.to_owned()]);
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

/// A systemd-run pin no host binary has: the live verifier's launcher refuses at it, so no process
/// starts in the gate (as t06 uses it).
const BAD_PIN: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// What the aggregate double was handed, per call (F101): the attempt `prepare` was given and its
/// deadline; the held attempt `create` was given, its deadline and the address of the cancellation
/// flag; the held attempt `finish` was handed by value and its deadline — so a finish of anything
/// but what was prepared is visible.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Held {
    Prepare(String, Instant),
    Create(String, Instant, usize),
    Finish(String, Instant),
}

/// The check's aggregate lifecycle as a model (R22-1): `prepare` holds the attempt it was given
/// unless scripted to refuse, `create` answers with the slice the held attempt names
/// (`hee3aggregate<id without dashes>.slice`, the shape the production slice derives) unless
/// scripted otherwise, and `finish` pops its scripted results in call order (`Ok` once they run
/// out); every call is recorded with what it was handed.
struct Slices {
    log: Arc<Mutex<Vec<Held>>>,
    prepare: Option<habitat_engine::worker::aggregate::Error>,
    create: Option<Result<String, habitat_engine::worker::aggregate::Error>>,
    finish: VecDeque<Result<(), habitat_engine::worker::aggregate::Error>>,
}

impl Slices {
    fn new() -> (Self, Arc<Mutex<Vec<Held>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                log: Arc::clone(&log),
                prepare: None,
                create: None,
                finish: VecDeque::new(),
            },
            log,
        )
    }
}

impl Aggregates for Slices {
    type Held = String;
    fn prepare(
        &mut self,
        attempt: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<String, habitat_engine::worker::aggregate::Error> {
        record(
            &self.log,
            Held::Prepare(attempt.as_str().to_owned(), deadline),
        );
        self.prepare
            .map_or_else(|| Ok(attempt.as_str().to_owned()), Err)
    }
    fn create(
        &mut self,
        held: &mut String,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<String, habitat_engine::worker::aggregate::Error> {
        record(
            &self.log,
            Held::Create(
                held.clone(),
                deadline,
                std::ptr::from_ref(cancelled) as usize,
            ),
        );
        self.create
            .clone()
            .unwrap_or_else(|| Ok(format!("hee3aggregate{}.slice", held.replace('-', ""))))
    }
    fn finish(
        &mut self,
        held: String,
        deadline: Instant,
    ) -> Result<(), habitat_engine::worker::aggregate::Error> {
        record(&self.log, Held::Finish(held, deadline));
        self.finish.pop_front().unwrap_or(Ok(()))
    }
}

/// The live verifier over the rig's class, the bad systemd-run pin and the aggregate double.
fn live_verifier(slices: Slices) -> LiveVerifier<Slices> {
    LiveVerifier::new(
        BAD_PIN.to_owned(),
        format!("/run/user/{}", rustix::process::geteuid().as_raw()).into(),
        slices,
    )
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

/// R21 N23 · each attempt is asked from its own charge start: the dispatch origin for attempt 1
/// (the preparation is charged to it, B14a-R2.5), its own begin for attempt 2 — never the dispatch
/// origin again, which would refuse a later native attempt `Deadline` at `origin + MAX_RUN` though
/// its own run is shorter. The rig records the dispatch origin from the admitted state
/// (`Admitted::window`), and the check double records when the first check was called; attempt 2
/// begins after that check returns, so its charge start is later than that call.
#[test]
fn each_attempt_is_asked_from_its_own_charge_start() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (mut source, asked) = script(vec![
        Candidate::Replacement(FIRST.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    let (mut verifier, handed) = oracle(vec![mismatched(7), matched(7)]);
    let principal = owner();
    let drain = AtomicBool::new(false);
    let admission = admit(
        &rig.tasks,
        &rig.profile,
        Dispatch {
            principal: &principal,
            task: id(TASK),
            agent_record_id: &rig.agent,
            selections: &rig.selections,
            attempts: &rig.attempts,
            root_id: &rig.root_id,
            forbidden: &[],
            teardown_ms: rig.teardown_ms,
            capture_ms: Some(5_000),
            drain: &drain,
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    let Admission::Ready(admitted) = admission else {
        return Err("the task was refused at admission".into());
    };
    let (origin, deadline) = admitted.window();
    assert_eq!(deadline.duration_since(origin), TASK_LIMIT);
    let outcome = drive(&rig.tasks, *admitted, &mut source, &mut verifier)
        .map_err(|e| format!("{e:?}"))?
        .outcome;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    let (asked, handed) = (taken(&asked), taken(&handed));
    assert_eq!((asked.len(), handed.len()), (2, 2));
    assert_eq!(
        asked[0].charged_from, origin,
        "attempt 1 is charged from the dispatch origin"
    );
    assert!(
        asked[1].charged_from > handed[0].3,
        "attempt 2 is charged from its own begin, after the first check was called: {:?} after it",
        asked[1].charged_from.checked_duration_since(handed[0].3)
    );
    assert!(asked[1].charged_from > asked[0].charged_from);
    Ok(())
}

/// The rig's task admitted as the owner under `drain`, or the refusal named.
fn admitted<'a>(
    rig: &'a Rig,
    principal: &'a Principal,
    drain: &'a AtomicBool,
) -> Result<Box<Admitted<'a>>, Box<dyn Error>> {
    admitted_with(rig, principal, drain, &rig.profile)
}

/// The rig's task admitted under `profile` (the class a provider reads through `Admitted`).
fn admitted_with<'a>(
    rig: &'a Rig,
    principal: &'a Principal,
    drain: &'a AtomicBool,
    profile: &'a Profile,
) -> Result<Box<Admitted<'a>>, Box<dyn Error>> {
    let admission = admit(
        &rig.tasks,
        profile,
        Dispatch {
            principal,
            task: id(TASK),
            agent_record_id: &rig.agent,
            selections: &rig.selections,
            attempts: &rig.attempts,
            root_id: &rig.root_id,
            forbidden: &[],
            teardown_ms: rig.teardown_ms,
            capture_ms: Some(5_000),
            drain,
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    match admission {
        Admission::Ready(admitted) => Ok(admitted),
        Admission::Refused(refusal) => Err(format!("refused at admission: {refusal:?}").into()),
    }
}

/// The count one `SELECT count(*) …` query returns over the whole ledger.
fn count(rig: &Rig, sql: &str) -> Result<i64, Box<dyn Error>> {
    Ok(ledger(rig)?.query_row(sql, [], |row| row.get(0))?)
}

/// R21 N4, proof (b) · a source not ready before attempt 1 ends the dispatch `NotReady` by the
/// refusal's name with nothing written: the task stays `admitted`, no attempt row and no roster
/// observation exist, no candidate was asked and no check run; the source was asked ready once,
/// under the attempt's work window (closure C3) and an unraised drain.
#[test]
fn a_provider_not_ready_at_attempt_one_leaves_the_task_admitted_and_writes_nothing() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (mut source, asked) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    source.readiness = Err(habitat_engine::worker::native::Error::Identity);
    let readied = Arc::clone(&source.readied);
    let (mut verifier, handed) = oracle(vec![matched(7)]);
    let principal = owner();
    let drain = AtomicBool::new(false);
    let admitted = admitted(&rig, &principal, &drain)?;
    let (origin, until) = admitted.window();
    let outcome = drive(&rig.tasks, *admitted, &mut source, &mut verifier)
        .map_err(|e| format!("{e:?}"))?
        .outcome;
    assert_eq!(
        outcome,
        Outcome::NotReady(habitat_engine::worker::native::Error::Identity)
    );
    assert_eq!(state(&rig)?, "admitted");
    assert_eq!(count(&rig, "SELECT count(*) FROM attempts")?, 0);
    assert_eq!(count(&rig, "SELECT count(*) FROM roster_observations")?, 0);
    assert!(taken(&asked).is_empty(), "no candidate was asked");
    assert!(taken(&handed).is_empty(), "no check ran");
    // Closure C3: under the attempt's work window, which closes before the dispatch; closure C4:
    // the NotReady exit runs one custody turn under that window's teardown bound.
    let work_until = window_end(origin, Shape::default().work_ms, rig.teardown_ms, until);
    let bound = teardown_bound(work_until, rig.teardown_ms, until);
    assert!(work_until < bound && bound < until);
    assert_eq!(
        taken(&readied),
        vec![
            ("ready", work_until, false),
            ("settle_retained", bound, false)
        ]
    );
    Ok(())
}

/// One roster observation row: its id, its event sequence and its body.
type ObservationRow = (String, i64, Vec<u8>);

/// Every roster observation in the ledger, in sequence order.
fn observations(rig: &Rig) -> Result<Vec<ObservationRow>, Box<dyn Error>> {
    let db = ledger(rig)?;
    let mut statement =
        db.prepare("SELECT id,sequence,body FROM roster_observations ORDER BY sequence")?;
    let found = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(found)
}

/// The sequences of the rig task's `attempt_started` events, in order.
fn attempt_starts(rig: &Rig) -> Result<Vec<i64>, Box<dyn Error>> {
    let db = ledger(rig)?;
    let mut statement = db.prepare(
        "SELECT sequence FROM events WHERE task_id=? AND kind='attempt_started' ORDER BY sequence",
    )?;
    let found = statement
        .query_map([TASK], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(found)
}

/// The bytes a scripted readiness claims were read from its provider: a catalogue, as `/api/tags`
/// renders one — published by the runtime as the observation's evidence.
const READY_EVIDENCE: &[u8] = b"{\"models\":[{\"name\":\"hee3-t28-ready:qualification\"}]}\n";

/// Closure C3 · the work windows of a two-attempt dispatch under the default shape, by the design's
/// rule: attempt 1's charged from the origin over the whole reservation, attempt 2's from its own
/// charge start over what attempt 1 left of it, as the ledger recorded attempt 1's use. Both close
/// before the dispatch, the second after the first.
fn two_windows(
    rig: &Rig,
    (origin, second): (Instant, Instant),
    until: Instant,
) -> Result<[Instant; 2], Box<dyn Error>> {
    let used = rows(
        rig,
        "SELECT used_ms FROM attempts WHERE task_id=? ORDER BY CAST(generation AS INTEGER)",
    )?;
    let first_used: u64 = used
        .first()
        .and_then(|row| row.first())
        .ok_or("a used_ms")?
        .parse()?;
    let reserved = Shape::default().work_ms;
    let windows = [
        window_end(origin, reserved, rig.teardown_ms, until),
        window_end(second, reserved - first_used, rig.teardown_ms, until),
    ];
    assert!(windows[1] < until && windows[0] < windows[1], "{windows:?}");
    Ok(windows)
}

/// R21 N4 · each attempt is preceded by its own provider-response observation, written by the runtime
/// from the source's readiness in the hold of that attempt's begin: two attempts (a mismatch, then a
/// match), two observations — each confirmed `provider_response`, its input the readiness whole bound
/// to the roster head (record, revision, owner, endpoint), its evidence ref the fresh id the
/// readiness's bytes were published under — each written after the previous attempt began and
/// before its own attempt began, and each attempt's roster pin carries the observation that
/// preceded it. The source was asked ready twice, each under its own attempt's work window (closure
/// C3).
#[test]
fn each_attempt_is_preceded_by_its_own_provider_response_observation() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (mut source, asked) = script(vec![
        Candidate::Replacement(FIRST.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    source.readiness = Ok(Readiness {
        actual_identity: "5a0c7d1e-boot:4242:31337".to_owned(),
        immutable_revision: Some(
            "sha256:9e1f0000000000000000000000000000000000000000000000000000000000a7".to_owned(),
        ),
        capabilities: vec!["text".to_owned()],
        evidence: READY_EVIDENCE.to_vec(),
    });
    let readied = Arc::clone(&source.readied);
    let (mut verifier, _) = oracle(vec![mismatched(7), matched(7)]);
    let principal = owner();
    let drain = AtomicBool::new(false);
    let admitted = admitted(&rig, &principal, &drain)?;
    let (origin, until) = admitted.window();
    let outcome = drive(&rig.tasks, *admitted, &mut source, &mut verifier)
        .map_err(|e| format!("{e:?}"))?
        .outcome;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    let asked = taken(&asked);
    assert_eq!(asked.len(), 2);
    let windows = two_windows(&rig, (origin, asked[1].charged_from), until)?;
    // Closure C4: each custody turn under its own attempt's teardown bound.
    let bounds = windows.map(|window| teardown_bound(window, rig.teardown_ms, until));
    assert_eq!(
        taken(&readied),
        vec![
            ("ready", windows[0], false),
            ("settle_retained", bounds[0], false),
            ("ready", windows[1], false),
            ("settle_retained", bounds[1], false),
        ]
    );
    let (observed, started) = (observations(&rig)?, attempt_starts(&rig)?);
    assert_eq!((observed.len(), started.len()), (2, 2));
    assert!(
        observed[0].1 < started[0] && started[0] < observed[1].1 && observed[1].1 < started[1],
        "observations {:?} interleave the begins {started:?}",
        observed.iter().map(|o| o.1).collect::<Vec<_>>()
    );
    let mut evidence_refs = Vec::new();
    for (id, _, body) in &observed {
        let body: serde_json::Value = serde_json::from_slice(body)?;
        assert_eq!(body["id"], id.as_str());
        assert_eq!(body["confirmed_source"], "provider_response");
        let evidence_ref = body["input"]["evidence_ref"]
            .as_str()
            .ok_or("an evidence ref")?
            .to_owned();
        assert!(UuidV4::parse(&evidence_ref).is_ok(), "{evidence_ref}");
        assert_eq!(
            body["input"],
            serde_json::json!({
                "record_id": rig.agent,
                "record_version": "1",
                "owner_id": "fixture-worker",
                "endpoint_ref": ROSTER_KEY,
                "instance_id": null,
                "instance_generation": null,
                "source": "provider_response",
                "observed_unix_ms": null,
                "availability": "available",
                "actual_identity": "5a0c7d1e-boot:4242:31337",
                "immutable_revision":
                    "sha256:9e1f0000000000000000000000000000000000000000000000000000000000a7",
                "capabilities": ["text"],
                "evidence_ref": evidence_ref,
            })
        );
        evidence_refs.push(evidence_ref);
    }
    assert_ne!(evidence_refs[0], evidence_refs[1], "a fresh id per attempt");
    // The readiness's bytes were published: the object their digest names holds them.
    assert_eq!(
        object_bytes(&rig, &t08_rig::digest(READY_EVIDENCE))?,
        READY_EVIDENCE
    );
    let attempts = rows(
        &rig,
        "SELECT id FROM attempts WHERE task_id=? ORDER BY CAST(generation AS INTEGER)",
    )?;
    assert_eq!(
        rows(
            &rig,
            "SELECT p.attempt_id,json_extract(p.body,'$.record.observation.id') FROM roster_pins p \
             JOIN attempts a ON a.id=p.attempt_id WHERE a.task_id=? \
             ORDER BY CAST(a.generation AS INTEGER)",
        )?,
        vec![
            vec![attempts[0][0].clone(), observed[0].0.clone()],
            vec![attempts[1][0].clone(), observed[1].0.clone()],
        ]
    );
    Ok(())
}

/// What one custody dispatch came to: the dispatcher's lines, the source's ready/settle log, the
/// dispatch window `open` was handed, and the rig.
type CustodyRun = (
    Vec<String>,
    Vec<(&'static str, Instant, bool)>,
    (Instant, Instant),
    Arc<Rig>,
);

/// One dispatch over a scripted answer and a scripted custody, through the dispatcher: the lines it
/// reported, what the source's `ready`/`settle_retained` were handed, the dispatch deadline `open`
/// was handed, and the rig to read the ledger from. The source's hook raises the drain inside the
/// one ask, so the dispatch runs to its end and the dispatcher's next wait ends `Drained`.
fn custody_run(answer: Candidate, custody: Custody) -> Result<CustodyRun, Box<dyn Error>> {
    let rig = Arc::new(rig(&Shape::default())?);
    let (mut source, _) = script(vec![answer]);
    source.custody = custody;
    let readied = Arc::clone(&source.readied);
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    source.hook = Some(Box::new(move || flag.store(true, Ordering::SeqCst)));
    let (verifier, _) = oracle(vec![matched(7)]);
    let (provider, _) = provider_of(vec![(source, verifier)], &stop);
    let opened = Arc::clone(&provider.admitted);
    let (exit, lines) = run_dispatcher(&rig, provider, &stop)?;
    assert_eq!(exit, dispatcher::Exit::Drained, "{lines:?}");
    let window = taken(&opened).first().ok_or("an open")?.window;
    Ok((lines, taken(&readied), window, rig))
}

/// R21 N18, K5 · the source's retained children are settled inside `drive` after EVERY answer,
/// before the attempt's settle, under the attempt's teardown bound (N18 as closure C4 amends it:
/// before the dispatch deadline, so the settle keeps the remainder), and the attempt's cleanup is
/// settled only when the arm's own is and none is still pending; the dispatcher reports the custody
/// by name.
/// Three fixtures differing in every field: (1) a success arm — a replacement, whose custody the
/// source does not pass on (K5) — with one child still pending: the attempt is not settled and the
/// task is left `needs settlement`, no check run; (2) a provider arm whose own cleanup settled, with
/// two retained children now settled: the attempt settles and the task stops `worker_failed`; (3) a
/// provider arm whose own cleanup did NOT settle, with its child now settled: still unsettled — a
/// source's unsettled cleanup is never read as settled on a pending count of zero, since a reaped
/// leader with a live group leaves nothing retained to settle.
#[test]
fn a_retained_child_is_settled_inside_drive_before_the_attempt_s_settle() -> Outcome_ {
    let provider = |cleanup_settled, retained| Candidate::Provider {
        error: habitat_engine::worker::native::Error::Identity,
        state: habitat_engine::worker::native::ProviderState::NotDispatched,
        cleanup_settled,
        retained,
    };
    let fixtures = [
        (
            Candidate::Replacement(SECOND.to_vec()),
            Custody {
                settled: 0,
                pending: 1,
            },
            "TaskLeft(\"needs settlement\"), custody: settled=0 pending=1",
            ("1", 0),
        ),
        (
            provider(true, 2),
            Custody {
                settled: 2,
                pending: 0,
            },
            "TaskDone(\"stopped\"), custody: settled=2 pending=0",
            ("0", 1),
        ),
        (
            provider(false, 1),
            Custody {
                settled: 1,
                pending: 0,
            },
            "TaskLeft(\"needs settlement\"), custody: settled=1 pending=0",
            ("1", 0),
        ),
    ];
    for (answer, custody, step, (unsettled, stops)) in fixtures {
        let (lines, readied, (origin, until), rig) = custody_run(answer, custody)?;
        let line = format!("dispatcher: task {TASK} -> {step}");
        assert!(lines.contains(&line), "{line} in {lines:?}");
        // Closure C3: ready under the attempt's work window, which closes before the dispatch;
        // closure C4 (amending N18): the custody turn under its teardown bound, strictly before the
        // dispatch deadline, so the attempt's settle keeps the remainder.
        let teardown = u64::try_from(CHECK_TEARDOWN.as_millis())?;
        let work_until = window_end(origin, Shape::default().work_ms, teardown, until);
        let bound = teardown_bound(work_until, teardown, until);
        assert!(work_until < bound && bound < until, "{step}");
        assert_eq!(
            readied,
            vec![
                ("ready", work_until, false),
                ("settle_retained", bound, false)
            ],
            "{step}"
        );
        assert_eq!(
            rows(
                &rig,
                "SELECT settled_event IS NULL FROM attempts WHERE task_id=?"
            )?,
            vec![vec![unsettled.to_owned()]],
            "{step}"
        );
        assert_eq!(
            count(&rig, "SELECT count(*) FROM verifications")?,
            0,
            "{step}"
        );
        assert_eq!(
            count(&rig, "SELECT count(*) FROM task_stops")?,
            stops,
            "{step}"
        );
    }
    Ok(())
}

/// Closure C4 (FT2-01, M6) · a source not ready still has its retained children settled — one custody
/// turn on the `NotReady` exit, under the attempt's teardown bound (its work window plus the
/// teardown share, before the dispatch deadline) — and the dispatcher reports the refusal's name and
/// the custody on its one line. Two fixtures differing in every field; nothing is written and the
/// task stays `admitted`.
#[test]
fn a_provider_not_ready_still_settles_and_reports_its_custody_by_name() -> Outcome_ {
    use habitat_engine::worker::native::Error as Native;
    let fixtures = [
        (
            Native::Identity,
            Custody {
                settled: 2,
                pending: 1,
            },
            "identity, custody: settled=2 pending=1",
        ),
        (
            Native::Deadline,
            Custody {
                settled: 0,
                pending: 3,
            },
            "deadline, custody: settled=0 pending=3",
        ),
    ];
    let teardown = u64::try_from(CHECK_TEARDOWN.as_millis())?;
    for (error, custody, tail) in fixtures {
        let rig = Arc::new(rig(&Shape::default())?);
        let (mut source, asked) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
        source.readiness = Err(error);
        source.custody = custody;
        let readied = Arc::clone(&source.readied);
        let (verifier, handed) = oracle(vec![matched(7)]);
        let stop = Arc::new(AtomicBool::new(false));
        let (provider, _) = provider_of(vec![(source, verifier)], &stop);
        let opened = Arc::clone(&provider.admitted);
        let (exit, lines) = run_dispatcher(&rig, provider, &stop)?;
        assert_eq!(
            lines,
            vec![format!(
                "dispatcher: task {TASK} -> DispatcherStops(\"provider not ready\"): {tail}"
            )]
        );
        assert_eq!(exit, dispatcher::Exit::Stopped("provider not ready"));
        let (origin, until) = taken(&opened).first().ok_or("an open")?.window;
        let work_until = window_end(origin, Shape::default().work_ms, teardown, until);
        let bound = teardown_bound(work_until, teardown, until);
        assert!(bound < until, "{tail}");
        assert_eq!(
            taken(&readied),
            vec![
                ("ready", work_until, false),
                ("settle_retained", bound, false)
            ],
            "{tail}"
        );
        assert_eq!(state(&rig)?, "admitted");
        assert_eq!(count(&rig, "SELECT count(*) FROM attempts")?, 0);
        assert!(taken(&asked).is_empty() && taken(&handed).is_empty());
    }
    Ok(())
}

/// How long a model child is polled toward the bound it was handed (F102): a bound past this is
/// not waited for, so a bound handed too far away fails the proof by the recorded deadline instead
/// of hanging it.
const SETTLE_MODEL_BUDGET: Duration = Duration::from_secs(5);

/// A retained child that never settles, as a model (F101): polled to the bound it was handed, it is
/// still pending there — one child pending, none settled. A bound past `SETTLE_MODEL_BUDGET` is not
/// waited for (the recorded deadline says which bound it was handed).
fn pending_to_its_bound(deadline: Instant) -> Custody {
    let budget = Instant::now() + SETTLE_MODEL_BUDGET;
    while Instant::now() < deadline && deadline <= budget {
        std::thread::sleep(Duration::from_millis(10));
    }
    Custody {
        settled: 0,
        pending: 1,
    }
}

/// Closure C4 (FT2-04; N18 amended) · the custody turn inside `execute` is bounded by the attempt's
/// teardown bound — its work window plus the teardown share — not the dispatch deadline, so a child
/// still pending at that bound leaves the attempt's settle its remainder: the settle is written, the
/// attempt stays unsettled and the task is left `needs settlement` with the child counted pending —
/// never an entropy failure after the whole dispatch window was spent polling.
#[test]
fn a_child_pending_to_its_bound_leaves_the_task_needs_settlement_not_entropy() -> Outcome_ {
    // The work window is the reservation less the 1 000 ms teardown share, measured from the
    // dispatch origin, which includes preparation: 3 500 ms leaves 2 500 ms, so a loaded run's
    // preparation is not read as a spent window (B14b-2 review round 2, D8; 1 500 left 500 ms).
    // The bound it yields is still inside `SETTLE_MODEL_BUDGET`.
    let rig = rig(&Shape {
        work_ms: 3_500,
        ..Shape::default()
    })?;
    let (mut source, asked) = script(vec![Candidate::Provider {
        error: habitat_engine::worker::native::Error::Identity,
        state: habitat_engine::worker::native::ProviderState::NotDispatched,
        cleanup_settled: true,
        retained: 1,
    }]);
    source.settle_model = Some(Box::new(pending_to_its_bound));
    let readied = Arc::clone(&source.readied);
    let (mut verifier, handed) = oracle(vec![]);
    let (principal, drain) = (owner(), AtomicBool::new(false));
    let admission = admit(
        &rig.tasks,
        &rig.profile,
        Dispatch {
            principal: &principal,
            task: id(TASK),
            agent_record_id: &rig.agent,
            selections: &rig.selections,
            attempts: &rig.attempts,
            root_id: &rig.root_id,
            forbidden: &[],
            teardown_ms: rig.teardown_ms,
            // The captures run under the owner's reservation less the teardown share, as `serve`
            // runs them: a fixed 10 ms share was refused `ProtectedCapture` under load (D8).
            capture_ms: None,
            drain: &drain,
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    let admitted = match admission {
        Admission::Ready(admitted) => admitted,
        Admission::Refused(refusal) => {
            return Err(format!("refused at admission: {refusal:?}").into());
        }
    };
    let (origin, until) = admitted.window();
    let dispatched =
        drive(&rig.tasks, *admitted, &mut source, &mut verifier).map_err(|e| format!("{e:?}"))?;
    let work_until = window_end(origin, 3_500, rig.teardown_ms, until);
    let bound = teardown_bound(work_until, rig.teardown_ms, until);
    assert!(bound < until);
    assert_eq!(
        taken(&readied),
        vec![
            ("ready", work_until, false),
            ("settle_retained", bound, false)
        ]
    );
    assert_eq!(
        dispatched.outcome,
        Outcome::Driven(Driven::NeedsSettlement(StopReason::Unsettled))
    );
    assert_eq!(
        dispatcher::classify(&Ok(dispatched.outcome)),
        dispatcher::Step::TaskLeft("needs settlement")
    );
    assert_eq!(
        dispatched.custody,
        Custody {
            settled: 0,
            pending: 1
        }
    );
    assert_eq!(
        rows(
            &rig,
            "SELECT settled_event IS NULL FROM attempts WHERE task_id=?"
        )?,
        vec![vec!["1".to_owned()]]
    );
    assert_eq!(count(&rig, "SELECT count(*) FROM task_stops")?, 0);
    assert_eq!(taken(&asked).len(), 1);
    assert!(taken(&handed).is_empty(), "no check ran");
    Ok(())
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
    let requested: Arc<Mutex<Vec<Instant>>> = Arc::new(Mutex::new(Vec::new()));
    let requested_at = Arc::clone(&requested);
    source.hook = Some(Box::new(move || {
        record(&requested_at, Instant::now());
    }));
    let (verifier, handed) = oracle(vec![mismatched(7), matched(7)]);
    let principal = owner();
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert_eq!(state(&rig)?, "accepted");
    let handed = taken(&handed);
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
    assert_windows(&handed, &taken(&requested), [300_000, 300_000 - 7]);
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
    let asked = taken(&asked);
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

/// The names under `directory`, sorted: the world a check ran beside.
fn entries(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .map(|read| {
            read.filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// B14b-2 S13 (R21 N13) · the ledger's root names the directories the runtime materialised: over two
/// attempts, each `attempt_paths` row holds the rig's attempts directory and the id its marker holds,
/// read here from the file itself (B14b-2 closure C18), and while each check ran the
/// directory held exactly that attempt's workspace `<attempt>` and job root `<attempt>.check` beside
/// the earlier attempts' workspaces (the job root is torn down after its check). The names are
/// derived here from the ledger's attempt ids, not from the runtime.
#[test]
fn the_ledger_root_names_the_directories_the_runtime_materialised() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (source, _) = script(vec![
        Candidate::Replacement(FIRST.to_vec()),
        Candidate::Replacement(SECOND.to_vec()),
    ]);
    let (mut verifier, _) = oracle(vec![mismatched(7), matched(7)]);
    let seen: Arc<Mutex<Vec<Vec<String>>>> = Arc::new(Mutex::new(Vec::new()));
    let (seen_by_hook, attempts) = (Arc::clone(&seen), rig.attempts.clone());
    verifier.hook = Some(Box::new(move || {
        record(&seen_by_hook, entries(&attempts));
    }));
    let principal = owner();
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    let ids: Vec<String> = rows(
        &rig,
        "SELECT id FROM attempts WHERE task_id=? ORDER BY CAST(generation AS INTEGER)",
    )?
    .into_iter()
    .flatten()
    .collect();
    let [first, second] = ids.as_slice() else {
        return Err(format!("two attempts, found {ids:?}").into());
    };
    let root = rig.attempts.to_str().ok_or("a UTF-8 rig root")?.to_owned();
    let root_id = fs::read_to_string(
        rig.attempts
            .join(habitat_engine::app::coordinator::ROOT_ID_MARKER),
    )?;
    assert!(UuidV4::parse(&root_id).is_ok(), "{root_id:?}");
    assert_eq!(
        rows(
            &rig,
            "SELECT p.attempt_id,p.root,p.root_id FROM attempt_paths p \
             JOIN attempts a ON a.id=p.attempt_id \
             WHERE a.task_id=? ORDER BY CAST(a.generation AS INTEGER)",
        )?,
        vec![
            vec![first.clone(), root.clone(), root_id.clone()],
            vec![second.clone(), root, root_id]
        ]
    );
    // The root's own marker (closure C18) sits beside the leaves throughout.
    let marker = habitat_engine::app::coordinator::ROOT_ID_MARKER.to_owned();
    let mut during_first = vec![marker.clone(), first.clone(), format!("{first}.check")];
    during_first.sort();
    let mut during_second = vec![
        marker,
        first.clone(),
        second.clone(),
        format!("{second}.check"),
    ];
    during_second.sort();
    assert_eq!(taken(&seen), vec![during_first, during_second]);
    Ok(())
}

/// R21 closure C13 (OC3, FT3-03, L11) · a `.plan` root left by a dispatch killed inside the shared
/// plan (created, never removed; the task still admitted, no attempt row) is the engine's own
/// leftover, not the owner's refusal: the next dispatch removes it through the workspace owner and
/// drives the task. At `b5f309b` the outcome was `Refused(Plan("plan_root_exists"))` and the task
/// was stopped by name.
#[test]
fn a_plan_root_left_by_a_killed_dispatch_is_removed_and_the_task_is_driven() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let leftover = rig.attempts.join(format!("{TASK}.plan"));
    fs::DirBuilder::new().mode(0o700).create(&leftover)?;
    fs::write(leftover.join("x"), b"left by a killed dispatch")?;
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![matched(7)]);
    let principal = owner();
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert!(!leftover.exists(), "the leftover plan root is removed");
    assert!(rows(&rig, "SELECT reason FROM task_stops WHERE task_id=?")?.is_empty());
    assert_eq!(state(&rig)?, "accepted");
    // A leftover the owner refuses (not 0700) is left whole, and the refusal is the dispatcher's:
    // no stop is written, the task stays admitted, nothing was asked.
    let refusing = self::rig(&Shape::default())?;
    let leftover = refusing.attempts.join(format!("{TASK}.plan"));
    fs::DirBuilder::new().mode(0o755).create(&leftover)?;
    fs::write(leftover.join("x"), b"left by a killed dispatch")?;
    let (source, asked) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (verifier, _) = oracle(vec![matched(7)]);
    let refused = run(&refusing, &principal, source, verifier, 5_000);
    assert!(
        matches!(
            &refused,
            Err(RuntimeError::PreDispatch(inner))
                if matches!(**inner, RuntimeError::PlanRoot(WorkspaceError::Custody))
        ),
        "{refused:?}"
    );
    assert!(
        leftover.join("x").exists(),
        "a refused leftover is left whole"
    );
    assert!(rows(&refusing, "SELECT reason FROM task_stops WHERE task_id=?")?.is_empty());
    assert_eq!(state(&refusing)?, "admitted");
    assert!(taken(&asked).is_empty(), "no provider was asked");
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
    assert_eq!(taken(&asked).len(), 1);
    assert!(taken(&handed).is_empty(), "no verifier call");
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
        taken(&handed).len(),
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
    let previous = taken(&asked)[1].previous.clone().ok_or("no previous")?;
    assert_eq!(
        (previous.verdict, previous.criteria),
        (VerificationVerdict::Failed, 0)
    );
    let evidence: serde_json::Value = serde_json::from_slice(&previous.evidence)?;
    assert_eq!(evidence["refusal"], "candidate_encoding");
    assert_eq!(evidence["candidate_sha256"], REFUSED_SHA256);
    // Beside the root's own marker (closure C18), at most the one attempt's leaf.
    let left: Vec<String> = entries(&rig.attempts)
        .into_iter()
        .filter(|name| name != habitat_engine::app::coordinator::ROOT_ID_MARKER)
        .collect();
    assert!(
        left.len() <= 1,
        "the refused candidate left no retained path: {left:?}"
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
        assert!(taken(&asked).is_empty() && taken(&handed).is_empty());
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
        taken(&handed).is_empty(),
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
    assert!(taken(&handed).is_empty());
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
    let handed = taken(&handed);
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
    assert_eq!(taken(&asked).len(), 1, "no second attempt began");
    assert!(taken(&handed).is_empty());
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

/// The instant an attempt's work window closes, by the design's rule (B14a-R1.8, R2.5), not the
/// runtime's code: its charge start plus the work reservation the ledger holds less the teardown
/// share, never past the dispatch deadline.
fn window_end(
    charged_from: Instant,
    reserved_ms: u64,
    teardown_ms: u64,
    until: Instant,
) -> Instant {
    until.min(charged_from + Duration::from_millis(reserved_ms.saturating_sub(teardown_ms)))
}

/// The bound an attempt's custody turn is handed, by the design's rule (review MEDIUM-2; N18 as
/// closure C4 amends it): its work window's end plus the teardown share held back for cleanup, never
/// past the dispatch deadline.
fn teardown_bound(work_until: Instant, teardown_ms: u64, until: Instant) -> Instant {
    until.min(work_until + Duration::from_millis(teardown_ms))
}

/// Closure C3 (FT2-03) · a work reservation spent before the next attempt is refused before the
/// provider is asked: the second begin reads the head before `ready`, finds no work window past the
/// teardown share, and stops the task by policy — the source was asked ready once, for attempt 1.
#[test]
fn a_spent_reservation_is_refused_before_the_provider_is_asked() -> Outcome_ {
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
    let readied = Arc::clone(&source.readied);
    let (verifier, handed) = oracle(vec![]);
    let outcome = run(&rig, &principal, source, verifier, 10).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::Stopped(StopReason::Policy(
            habitat_engine::task::LoopRefusal::Deadline
        )))
    );
    assert_eq!(taken(&asked).len(), 1, "no second attempt began");
    assert!(taken(&handed).is_empty());
    let readied = taken(&readied);
    assert_eq!(
        readied.iter().filter(|call| call.0 == "ready").count(),
        1,
        "{readied:?}"
    );
    Ok(())
}

/// Closure C3 (M1; B14b-1 D5) · a drain raised while the source is made ready ends the dispatch
/// `Drained` with nothing written: `ready` is handed the engine's drain as its cancellation and the
/// attempt's work window as its deadline, the drain is read again when `ready` returns, and no
/// attempt row, roster observation, candidate or check exists — the task stays `admitted`.
#[test]
fn a_drain_raised_inside_ready_ends_drained_with_no_attempt_row() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let drain = AtomicBool::new(false);
    let handed_the_drain = Arc::new(Mutex::new(Vec::new()));
    let (mut source, asked) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (log, raised) = (Arc::clone(&handed_the_drain), &drain);
    source.ready_hook = Some(Box::new(move |handed: &AtomicBool| {
        raised.store(true, Ordering::SeqCst);
        record(&log, std::ptr::eq(handed, raised));
    }));
    let readied = Arc::clone(&source.readied);
    let (mut verifier, handed) = oracle(vec![matched(7)]);
    let principal = owner();
    let admitted = admitted(&rig, &principal, &drain)?;
    let (origin, until) = admitted.window();
    let outcome = drive(&rig.tasks, *admitted, &mut source, &mut verifier)
        .map_err(|e| format!("{e:?}"))?
        .outcome;
    assert_eq!(outcome, Outcome::Drained);
    assert_eq!(count(&rig, "SELECT count(*) FROM attempts")?, 0);
    assert_eq!(count(&rig, "SELECT count(*) FROM roster_observations")?, 0);
    assert_eq!(state(&rig)?, "admitted");
    assert!(taken(&asked).is_empty(), "no candidate was asked");
    assert!(taken(&handed).is_empty(), "no check ran");
    assert_eq!(
        taken(&handed_the_drain),
        vec![true],
        "ready was handed the engine's drain"
    );
    let work_until = window_end(origin, Shape::default().work_ms, rig.teardown_ms, until);
    assert!(
        work_until < until,
        "the work window closes before the dispatch"
    );
    // Closure C4: the Drained exit runs one custody turn under the window's teardown bound, handed
    // the engine's drain (B14b-2 review round 2, D3) -- raised here, so the turn may end at once.
    let bound = teardown_bound(work_until, rig.teardown_ms, until);
    assert_eq!(
        taken(&readied),
        vec![
            ("ready", work_until, false),
            ("settle_retained", bound, true)
        ]
    );
    Ok(())
}

/// B14b-2 review round 2, D3 (N18 amended; C4's open Stop-exit item) · on a stop exit the stop is
/// written before the exit's custody turn. The owner cancels while `ready` runs, the dispatch stops
/// the task, and the turn that settles the child `ready` retained already reads the task
/// `cancelled` -- bounded by the attempt's teardown bound and handed the engine's drain (unraised
/// here), not the runtime's own flag. A turn run before the stop reads `admitted`: a kill during its
/// wait, which can be minutes on a default reservation, would leave the stop unwritten.
#[test]
fn a_stop_exit_writes_the_stop_before_its_custody_turn() -> Outcome_ {
    let rig = rig(&Shape {
        work_ms: 3_500,
        ..Shape::default()
    })?;
    let principal = owner();
    let states = Arc::new(Mutex::new(Vec::new()));
    let (mut source, asked) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (seen, observed, cancelling) = (Arc::clone(&states), &rig, &principal);
    source.ready_hook = Some(Box::new(move |_: &AtomicBool| {
        record(&seen, state(observed).unwrap_or_else(|e| format!("<{e}>")));
        cancel(observed, cancelling, "28f10000-0000-4000-8000-0000000000c3");
    }));
    let seen = Arc::clone(&states);
    source.settle_model = Some(Box::new(move |_: Instant| {
        record(&seen, state(observed).unwrap_or_else(|e| format!("<{e}>")));
        Custody {
            settled: 1,
            pending: 0,
        }
    }));
    let readied = Arc::clone(&source.readied);
    let (mut verifier, handed) = oracle(vec![]);
    let drain = AtomicBool::new(false);
    let admission = admit(
        &rig.tasks,
        &rig.profile,
        Dispatch {
            principal: &principal,
            task: id(TASK),
            agent_record_id: &rig.agent,
            selections: &rig.selections,
            attempts: &rig.attempts,
            root_id: &rig.root_id,
            forbidden: &[],
            teardown_ms: rig.teardown_ms,
            // The captures run under the owner's reservation less the teardown share, as `serve`
            // runs them: a fixed 10 ms share was refused `ProtectedCapture` under load (D8).
            capture_ms: None,
            drain: &drain,
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    let admitted = match admission {
        Admission::Ready(admitted) => admitted,
        Admission::Refused(refusal) => {
            return Err(format!("refused at admission: {refusal:?}").into());
        }
    };
    let (origin, until) = admitted.window();
    let dispatched =
        drive(&rig.tasks, *admitted, &mut source, &mut verifier).map_err(|e| format!("{e:?}"))?;
    let work_until = window_end(origin, 3_500, rig.teardown_ms, until);
    let bound = teardown_bound(work_until, rig.teardown_ms, until);
    assert!(bound < until);
    assert_eq!(
        taken(&readied),
        vec![
            ("ready", work_until, false),
            ("settle_retained", bound, false)
        ]
    );
    assert_eq!(
        taken(&states),
        vec!["admitted".to_owned(), "cancelled".to_owned()],
        "the task state each call read: the stop is written before the custody turn"
    );
    assert_eq!(
        (dispatched.outcome, dispatched.custody),
        (
            Outcome::Driven(Driven::Stopped(StopReason::Cancelled)),
            Custody {
                settled: 1,
                pending: 0
            }
        )
    );
    assert!(taken(&asked).is_empty(), "no candidate was asked");
    assert!(taken(&handed).is_empty(), "no check ran");
    Ok(())
}

/// B14b-2 review round 2, D9 · a begin under an attempts root replaced since `serve` prepared it is
/// refused by name. After admission and before the drive, the root at the path is moved aside and a
/// second root, marked by the one door with another id, is renamed onto the path: the begin's read
/// finds a well-formed marker that is not the id the dispatch carries. The error is the reader's
/// kind, `AttemptsRoot(Changed)`, never erased, inside `PreDispatch` (no attempt row yet: the
/// dispatcher's stop, closure H3); the source is never asked ready (the root is read first), and no
/// attempt row, roster observation, candidate or check exists.
#[test]
fn a_begin_under_a_swapped_attempts_root_is_refused_by_name() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let (mut source, asked) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let readied = Arc::clone(&source.readied);
    let (mut verifier, handed) = oracle(vec![matched(7)]);
    let principal = owner();
    let drain = AtomicBool::new(false);
    let admitted = admitted(&rig, &principal, &drain)?;
    let other = rig.scratch.0.join("attempts-other");
    let other_id = habitat_engine::app::coordinator::prepare_attempts_root(&other, deadline())
        .map_err(|e| format!("{e:?}"))?;
    assert_ne!(other_id, rig.root_id, "two roots, two ids");
    fs::rename(&rig.attempts, rig.scratch.0.join("attempts-first"))?;
    fs::rename(&other, &rig.attempts)?;
    let error = drive(&rig.tasks, *admitted, &mut source, &mut verifier)
        .err()
        .map(|undispatched| format!("{:?}", undispatched.error));
    assert_eq!(error.as_deref(), Some("PreDispatch(AttemptsRoot(Changed))"));
    assert_eq!(count(&rig, "SELECT count(*) FROM attempts")?, 0);
    assert_eq!(count(&rig, "SELECT count(*) FROM roster_observations")?, 0);
    assert_eq!(
        taken(&readied)
            .into_iter()
            .map(|(label, _, _)| label)
            .collect::<Vec<_>>(),
        vec!["settle_retained"],
        "the source was never asked ready; only the exit's custody turn ran"
    );
    assert!(taken(&asked).is_empty(), "no candidate was asked");
    assert!(taken(&handed).is_empty(), "no check ran");
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
    let handed = taken(&handed);
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
    let (slices, held) = Slices::new();
    let verifier = live_verifier(slices);
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
    // R21 S20a · the check held its own aggregate and settled it: one start, one finish, and the
    // cleanup record's "resources" obligation reads what the verifier observed.
    assert_eq!(
        taken(&held)
            .iter()
            .map(|call| match call {
                Held::Prepare(..) => "prepare",
                Held::Create(..) => "create",
                Held::Finish(..) => "finish",
            })
            .collect::<Vec<_>>(),
        vec!["prepare", "create", "finish"]
    );
    assert_eq!(
        cleanup_object(&rig)?["obligations"],
        serde_json::json!([
            {"id": "process", "state": "settled"},
            {"id": "scratch", "state": "settled"},
            {"id": "retained_paths", "state": "settled"},
            {"id": "resources", "state": "settled"},
        ])
    );
    Ok(())
}

/// The task's one committed cleanup record, decoded from its object.
fn cleanup_object(rig: &Rig) -> Result<serde_json::Value, Box<dyn Error>> {
    let records = committed_records(rig)?;
    let row = records
        .iter()
        .find(|row| row[0] == "run_cleanup")
        .ok_or("a cleanup record")?;
    object_json(rig, &row[2])
}

/// The attempts the aggregate double prepared, in call order.
fn prepared(log: Vec<Held>) -> Vec<String> {
    log.into_iter()
        .filter_map(|call| match call {
            Held::Prepare(id, _) => Some(id),
            Held::Create(..) | Held::Finish(..) => None,
        })
        .collect()
}

/// The task's attempt ids, in the ledger's generation order.
fn attempt_ids(rig: &Rig) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(rows(
        rig,
        "SELECT id FROM attempts WHERE task_id=? ORDER BY CAST(generation AS INTEGER)",
    )?
    .into_iter()
    .flatten()
    .collect())
}

/// One direct check of the live verifier over the rig's class: the baseline as the subject, the
/// protected tree, a fresh job root under the scratch, the class's tools, `window` and the ledger
/// `attempt` it verifies.
fn live_check(
    rig: &Rig,
    verifier: &mut LiveVerifier<Slices>,
    window: CheckWindow,
    attempt: &str,
    job_root: &Path,
    cancelled: &AtomicBool,
) -> Result<Observed, Box<dyn Error>> {
    let capture = |name: &str| {
        Snapshot::capture(&rig.profile.directory.join(name), &[], deadline())
            .map_err(|e| format!("{e:?}"))
    };
    let (subject, protected) = (capture("base")?, capture("protected")?);
    private(job_root)?;
    let tools = habitat_engine::app::live_verifier::tools(&rig.profile.declared);
    Ok(verifier.check(CheckPlan {
        subject: &subject,
        protected: &protected,
        job_root,
        tools: &tools,
        window,
        attempt: UuidV4::parse(attempt).map_err(|e| format!("{e:?}"))?,
        cancelled,
    }))
}

/// R22-1 · the check owns its slice: each `check` prepares ONE slice named after the plan's
/// attempt (R22 C1a) under the check's cutoff, creates it under the cutoff and the plan's own
/// cancellation, builds its three scopes on the unit `create` returned (the scopes' only source),
/// and hands the slice it prepared to `finish` under the teardown deadline — so two checks over
/// windows differing in every field prepare, create and finish two different slices, each inside
/// its own window. A refused prepare holds nothing and finishes nothing; a refused create still
/// finishes what it prepared; neither launches, and each is named. A refused finish is observed
/// `Pending` with its refusal, does not refuse the next check, and through the runtime is named in
/// the cleanup record's `resources` obligation.
#[test]
fn the_live_verifier_names_each_check_s_slice_after_its_attempt_and_finishes_what_it_created()
-> Outcome_ {
    use habitat_engine::worker::aggregate::Error as Refused;
    // Two ledger attempts differing in every hex digit but the version digit (R22 §5).
    const A1: &str = "01234567-89ab-4cde-8f01-23456789abcd";
    const A2: &str = "fedcba98-7654-4321-b0fe-dcba98765432";
    let rig = rig(&Shape::default())?;
    let now = Instant::now();
    let window = |begun_ms: u64, unix: u64, until_s: u64, teardown_s: u64| CheckWindow {
        begun: now + Duration::from_millis(begun_ms),
        begun_unix_ms: unix,
        until: now + Duration::from_secs(until_s),
        teardown_until: now + Duration::from_secs(teardown_s),
    };
    let (w1, w2) = (window(0, 1_000, 20, 30), window(3, 2_000, 25, 40));
    let cancelled = AtomicBool::new(false);
    let flag = std::ptr::from_ref(&cancelled) as usize;
    let root = |name: &str| rig.scratch.0.join(name);
    let launcher_failed = |observed: &Observed| {
        observed
            .run
            .as_ref()
            .is_ok_and(|run| matches!(run.outcome, RunOutcome::LauncherFailed))
    };
    // (A) two checks: two slices, each prepared and created under its cutoff and finished — the
    // one it prepared — under its teardown.
    let (slices, held) = Slices::new();
    let mut verifier = live_verifier(slices);
    let first = live_check(&rig, &mut verifier, w1, A1, &root("check-a1"), &cancelled)?;
    let second = live_check(&rig, &mut verifier, w2, A2, &root("check-a2"), &cancelled)?;
    assert_eq!(
        taken(&held),
        vec![
            Held::Prepare(A1.to_owned(), w1.until),
            Held::Create(A1.to_owned(), w1.until, flag),
            Held::Finish(A1.to_owned(), w1.teardown_until),
            Held::Prepare(A2.to_owned(), w2.until),
            Held::Create(A2.to_owned(), w2.until, flag),
            Held::Finish(A2.to_owned(), w2.teardown_until),
        ]
    );
    // The launcher refused at the class's pins: three distinct scopes on a valid slice reached it
    // (a reused id or a malformed slice is `Layout` before any launch), and nothing started.
    for observed in [&first, &second] {
        assert!(
            launcher_failed(observed),
            "{:?}",
            observed.run.as_ref().err()
        );
        assert_eq!(observed.resources, Resources::Settled);
    }
    // (B) a refused prepare: nothing held, nothing finished, no launch, named, settled.
    let (mut slices, held) = Slices::new();
    slices.prepare = Some(Refused::Invalid);
    let mut verifier = live_verifier(slices);
    let job_root = root("check-b");
    let refused = live_check(&rig, &mut verifier, w1, A1, &job_root, &cancelled)?;
    assert_eq!(taken(&held), vec![Held::Prepare(A1.to_owned(), w1.until)]);
    assert!(
        matches!(refused.run, Err(Unlaunched::Aggregate(Refused::Invalid))),
        "{:?}",
        refused.run.as_ref().err()
    );
    assert_eq!(fs::read_dir(&job_root)?.count(), 0, "nothing launched");
    assert_eq!(refused.resources, Resources::Settled);
    // (C) a refused create: what was prepared is finished, and the refusal is named.
    let (mut slices, held) = Slices::new();
    slices.create = Some(Err(Refused::Limits));
    let mut verifier = live_verifier(slices);
    let job_root = root("check-c");
    let limits = live_check(&rig, &mut verifier, w2, A2, &job_root, &cancelled)?;
    assert_eq!(
        taken(&held),
        vec![
            Held::Prepare(A2.to_owned(), w2.until),
            Held::Create(A2.to_owned(), w2.until, flag),
            Held::Finish(A2.to_owned(), w2.teardown_until),
        ]
    );
    assert!(
        matches!(limits.run, Err(Unlaunched::Aggregate(Refused::Limits))),
        "{:?}",
        limits.run.as_ref().err()
    );
    assert_eq!(fs::read_dir(&job_root)?.count(), 0, "nothing launched");
    // (D) the unit `create` returned is the scopes' only source: a name no slice has is `Layout`.
    let (mut slices, _) = Slices::new();
    slices.create = Some(Ok("not-a-slice".to_owned()));
    let mut verifier = live_verifier(slices);
    let malformed = live_check(&rig, &mut verifier, w2, A2, &root("check-d"), &cancelled)?;
    assert!(
        matches!(
            malformed.run,
            Err(Unlaunched::Workload(workload::Error::Layout))
        ),
        "{:?}",
        malformed.run.as_ref().err()
    );
    // (E) a refused finish is observed pending with its refusal, and does not refuse the next
    // check on the same verifier: the next slice is prepared, created and finished, and settles.
    let (mut slices, held) = Slices::new();
    slices.finish = VecDeque::from([Err(Refused::Busy), Ok(())]);
    let mut verifier = live_verifier(slices);
    let busy = live_check(&rig, &mut verifier, w1, A1, &root("check-e1"), &cancelled)?;
    let next = live_check(&rig, &mut verifier, w2, A2, &root("check-e2"), &cancelled)?;
    assert_eq!(
        [busy.resources, next.resources],
        [Resources::Pending(Refused::Busy), Resources::Settled]
    );
    assert!(launcher_failed(&next), "the next check ran");
    assert_eq!(prepared(taken(&held)), [A1, A2]);
    // (F) through the runtime.
    a_refused_finish_is_named_through_the_runtime(&rig)
}

/// Fixture (F) of the test above, through the runtime: every slice prepared is the ledger's own
/// attempt id, in the ledger's order (R22 C1a); the pending slice is the cleanup record's, named,
/// and the check's cleanup is unsettled, so the task needs settlement (B14a-1c review H1).
fn a_refused_finish_is_named_through_the_runtime(rig: &Rig) -> Outcome_ {
    let principal = owner();
    let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
    let (mut slices, through_runtime) = Slices::new();
    slices.finish = VecDeque::from([Err(habitat_engine::worker::aggregate::Error::Busy)]);
    let outcome =
        run(rig, &principal, source, live_verifier(slices), 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::Driven(Driven::NeedsSettlement(StopReason::Unsettled))
    );
    let attempts = attempt_ids(rig)?;
    assert!(!attempts.is_empty(), "the run began an attempt");
    assert_eq!(prepared(taken(&through_runtime)), attempts);
    let cleanup = cleanup_object(rig)?;
    assert_eq!(
        (&cleanup["aggregate"], &cleanup["obligations"]),
        (
            &serde_json::json!("pending"),
            &serde_json::json!([
                {"id": "process", "state": "settled"},
                {"id": "scratch", "state": "settled"},
                {"id": "retained_paths", "state": "settled"},
                {"id": "resources", "state": "pending", "refusal": "busy"},
            ])
        )
    );
    Ok(())
}

/// R22-2 · a slice that refused before the workload ran is named in the run's outcome record, and
/// the one table decides the outcome: through the runtime, a create refused `Limits` is a setup
/// failure naming `limits`; one refused `Deadline` is a timeout naming `deadline`. Each committed
/// outcome object is decoded whole — no steps, the four observations, the refusal.
#[test]
fn an_aggregate_start_refusal_is_named_in_the_outcome_record() -> Outcome_ {
    use habitat_engine::worker::aggregate::Error as Refused;
    let observations = serde_json::json!({
        "process_cleanup": "complete",
        "subjects": "unchanged",
        "cancellation": "not_observed",
        "scratch": "released",
    });
    for (refusal, outcome, name) in [
        (Refused::Limits, "setup_failed", "limits"),
        (Refused::Deadline, "timeout", "deadline"),
    ] {
        let rig = rig(&Shape::default())?;
        let principal = owner();
        let (source, _) = script(vec![Candidate::Replacement(SECOND.to_vec())]);
        let (mut slices, _) = Slices::new();
        slices.create = Some(Err(refusal));
        run(&rig, &principal, source, live_verifier(slices), 5_000)
            .map_err(|e| format!("{e:?}"))?;
        let recorded = committed_records(&rig)?
            .iter()
            .filter(|row| row[0] == "run_outcome")
            .map(|row| object_json(&rig, &row[2]))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            recorded,
            vec![serde_json::json!({
                "outcome": outcome,
                "steps": [],
                "observations": observations,
                "aggregate_refusal": name,
            })],
            "{refusal:?}"
        );
    }
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
    let profile = native_fixture(rig, stand_in, answers, context_length)?;
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

/// The native install under the rig's scratch (`native/`) and the scenario the fake answers from:
/// one generate answer per attempt, the `/2` row's request, the resident model at `context_length`.
fn native_fixture(
    rig: &Rig,
    stand_in: &DaemonStandIn,
    answers: &[(&str, &str)],
    context_length: u64,
) -> Result<habitat_engine::worker::native::Profile, Box<dyn Error>> {
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
    Ok(profile)
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
    let handed = taken(&handed);
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
        taken(&handed).len(),
        1,
        "no verifier call for the refused candidate"
    );
    assert_eq!(taken(&handed)[0].1, REFERENCE_LIB.as_bytes());
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

/// R21 N4, proof (b), over the fake · the resident model at the qualified 512 context, against the
/// `/2` row's 4096, is refused by the readiness readback before any attempt (R18 A9's `identity`,
/// now met before `begin`): the source loads the model once — the load's own generate, its prompt
/// empty — and the readback after it still reads 512, so the dispatch ends `NotReady(Identity)`. The
/// task stays `admitted` with no attempt row, no roster observation, no stop and no check; the fake's
/// call log is the readback, then the load.
#[test]
fn a_native_model_resident_at_another_context_is_loaded_once_and_not_ready() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let stand_in = DaemonStandIn::spawn();
    let source = native_source(&rig, &stand_in, &[(REFERENCE_LIB, "stop")], 512)?;
    let principal = owner();
    let (verifier, handed) = oracle(vec![]);
    let outcome = run(&rig, &principal, source, verifier, 5_000).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        outcome,
        Outcome::NotReady(habitat_engine::worker::native::Error::Identity)
    );
    assert_eq!(state(&rig)?, "admitted");
    assert!(taken(&handed).is_empty());
    assert_eq!(count(&rig, "SELECT count(*) FROM attempts")?, 0);
    assert_eq!(count(&rig, "SELECT count(*) FROM roster_observations")?, 0);
    assert_eq!(count(&rig, "SELECT count(*) FROM task_stops")?, 0);
    assert_eq!(
        fs::read_to_string(rig.scratch.0.join("native/calls.log"))?,
        "version\ntags\nps\nversion\ntags\ngenerate\nps\n"
    );
    let captured: serde_json::Value = serde_json::from_slice(&fs::read(
        rig.scratch.0.join("native/captured-request.json"),
    )?)?;
    assert_eq!(
        captured["prompt"], "",
        "the load's generate, not a candidate's"
    );
    Ok(())
}

/// R18 (d), A2 · a work reservation past the adapter's own cap: the source passes the window through,
/// the adapter refuses `deadline` at its door before the attempt's exchange (the readiness readback
/// before `begin` is the only exchange that ran, R21 N4), and the task stops `worker_failed` with the
/// refusal named in the stop body and the worker settle — the cap is met by name, never clamped.
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
    assert_eq!(body["reason"], "worker_failed");
    assert_eq!(body["attempts"], 1);
    assert_eq!(
        body["worker"],
        serde_json::json!({
            "provider": "ollama-local",
            "error": "deadline",
            "state": "not_dispatched",
            "retained": 0,
        })
    );
    // The readiness readback ran before the attempt (R21 N4) and found the model resident; the
    // attempt's own exchange never ran.
    assert_eq!(
        fs::read_to_string(rig.scratch.0.join("native/calls.log"))?,
        "version\ntags\nps\n"
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
            "outcome": {"provider": {"name": "deadline"}},
            "replacement_bytes": null,
        })
    );
    Ok(())
}

/// The operator's native-provider file the install proofs read (R21 D1): the rig's own values are
/// not needed to install, only the roster keys and the bytes the request publishes.
const OPERATOR_FILE: &str = r#"schema = "hee3.native/1"
directory = "/srv/hee/native"

[client]
path = "/usr/bin/curl"
sha256 = "sha256:a57a75f1b0c309eb4a21cc82efd24645be868c3a2689912b26fa4b7e940dcdd6"
bytes = 218440

[install]
manifest = { path = "/srv/models/manifests/llama3.2/3b", sha256 = "sha256:a80c4f17acd55265feec403c7aef86be0c25983ab279d83f3bcd3abbcb5b8b72", bytes = 1005 }
blobs = "/srv/models/blobs"

[daemon]
unit = "ollama.service"
scope = "user"
executable = { sha256 = "sha256:12ff8654a500a29048e2a40ff297e778f98c31742ecbd354dc948dbab3cea1aa", bytes = 32276424 }

[roster]
idempotency_key = "28f10000-0000-4000-8000-0000000000a1"
endpoint_ref = "28f10000-0000-4000-8000-0000000000a2"
"#;

/// The native record's stored definition and revision, read on the ledger's own connection.
fn native_record(rig: &Rig, id: &str) -> Result<(String, serde_json::Value), Box<dyn Error>> {
    let (revision, definition): (String, Vec<u8>) = ledger(rig)?.query_row(
        "SELECT revision,definition FROM roster_records WHERE id=?",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok((revision, serde_json::from_slice(&definition)?))
}

/// R21 N12 · the install is keyed by the operator file's request key: a repeat applies nothing and
/// returns the same record; an edited file is ONE revision of that record under a fresh key (never a
/// second record, never a `Conflict`), and installing the edited file again applies nothing. Each
/// install returns the one selection the dispatcher begins under, pinned to the head it read.
#[test]
fn the_install_is_idempotent_and_an_edit_is_one_revision() -> Outcome_ {
    let rig = rig(&Shape::default())?;
    let principal = owner();
    let install = |text: &str| -> Result<Installed, Box<dyn Error>> {
        let file = native_provider::compose(text.as_bytes()).map_err(|e| format!("{e:?}"))?;
        Ok(native_provider::install(
            &rig.tasks,
            &principal,
            &file,
            text.as_bytes(),
            FULL_FILE,
            deadline(),
        )
        .map_err(|e| format!("{e:?}"))?)
    };
    let operations = || count(&rig, "SELECT count(*) FROM operations");
    let selection = |record_id: &str, revision: &str| Selection {
        record_id: record_id.to_owned(),
        expected_revision: revision.to_owned(),
        capabilities: vec!["text".to_owned()],
        local_only: true,
        version: Some("ollama-fc44-12ff8654/2".to_owned()),
        ttl_ms: 60_000,
    };
    let before = operations()?;
    let first = install(OPERATOR_FILE)?;
    assert_eq!(operations()?, before + 1, "the create is one operation");
    assert_ne!(first.record_id, rig.agent, "a record of its own");
    assert_eq!(
        first,
        Installed {
            record_id: first.record_id.clone(),
            selections: vec![selection(&first.record_id, "1")],
        }
    );
    let definition = |endpoint: &str| {
        serde_json::json!({
            "kind": "agent",
            "display_name": "native",
            "owner_id": "operator",
            "version": "ollama-fc44-12ff8654/2",
            "capabilities": ["text"],
            "locality": "local",
            "endpoint_ref": endpoint,
            "limitations": "adapter ollama-fc44-12ff8654/2: num_ctx 4096, num_predict 1024, templated",
        })
    };
    assert_eq!(
        native_record(&rig, &first.record_id)?,
        (
            "1".to_owned(),
            definition("28f10000-0000-4000-8000-0000000000a2")
        )
    );
    // The same file again: the same record at the same revision, nothing applied.
    assert_eq!(install(OPERATOR_FILE)?, first);
    assert_eq!(operations()?, before + 1, "a repeat applies nothing");
    // An edited endpoint: one revision of the same record, under a key of its own.
    let edited = OPERATOR_FILE.replace(
        "endpoint_ref = \"28f10000-0000-4000-8000-0000000000a2\"",
        "endpoint_ref = \"28f10000-0000-4000-9000-0000000000b7\"",
    );
    assert_ne!(edited, OPERATOR_FILE);
    let second = install(&edited)?;
    assert_eq!(
        second,
        Installed {
            record_id: first.record_id.clone(),
            selections: vec![selection(&first.record_id, "2")],
        }
    );
    assert_eq!(operations()?, before + 2, "the edit is one operation");
    assert_eq!(
        native_record(&rig, &first.record_id)?,
        (
            "2".to_owned(),
            definition("28f10000-0000-4000-9000-0000000000b7")
        )
    );
    // The edited file again: the head already says it, so nothing is applied.
    assert_eq!(install(&edited)?, second);
    assert_eq!(
        operations()?,
        before + 2,
        "a repeat of the edit applies nothing"
    );
    assert_eq!(count(&rig, "SELECT count(*) FROM roster_records")?, 2);
    Ok(())
}

/// The daemon stand-in's unit name, as the operator file declares it and the seam double is asked.
const STAND_IN_UNIT: &str = "hee3-t08-stand-in.service";

/// A `MainPid` double (R21 N7): records the unit, the deadline and the flag's address it was handed
/// (F101), and answers with its scripted pid — the stand-in's own in the gate.
struct StandInPid {
    pid: Result<u32, habitat_engine::worker::native::Error>,
    seen: AskedPid,
}

/// What the `MainPid` double was handed, per call: the unit, the deadline, the flag's address.
type AskedPid = Arc<Mutex<Vec<(String, Instant, usize)>>>;

impl habitat_engine::worker::native::MainPid for StandInPid {
    fn main_pid(
        &mut self,
        unit: &str,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<u32, habitat_engine::worker::native::Error> {
        record(
            &self.seen,
            (
                unit.to_owned(),
                deadline,
                std::ptr::from_ref(cancelled) as usize,
            ),
        );
        self.pid
    }
}

/// The operator file over the native fixture `profile` (its client, manifest, blobs, directory and
/// daemon executable) and the stand-in's unit, with the manifest pin `manifest_sha256`.
fn operator_file(
    profile: &habitat_engine::worker::native::Profile,
    manifest_sha256: &str,
) -> Result<native_provider::NativeFile, Box<dyn Error>> {
    let text = format!(
        "schema = \"hee3.native/1\"\ndirectory = \"{}\"\n\n[client]\npath = \"{}\"\nsha256 = \"{}\"\nbytes = {}\n\n\
         [install]\nmanifest = {{ path = \"{}\", sha256 = \"{manifest_sha256}\", bytes = {} }}\nblobs = \"{}\"\n\n\
         [daemon]\nunit = \"{STAND_IN_UNIT}\"\nscope = \"user\"\nexecutable = {{ sha256 = \"{}\", bytes = {} }}\n\n\
         [roster]\nidempotency_key = \"28f10000-0000-4000-8000-0000000000a1\"\n\
         endpoint_ref = \"28f10000-0000-4000-8000-0000000000a2\"\n",
        profile.directory.display(),
        profile.client.path.display(),
        profile.client.sha256,
        profile.client.bytes,
        profile.manifest.path.display(),
        profile.manifest.bytes,
        profile.blobs.display(),
        profile.daemon.executable_sha256,
        profile.daemon.executable_bytes,
    );
    Ok(native_provider::compose(text.as_bytes()).map_err(|e| format!("{e:?}"))?)
}

/// The rig's class with its native row (R21 N8): the fixture's model, `manifest_sha256`, the `/2`
/// adapter row. A declared value: the `/2` schema's reading is `class_profile`'s own proof (S16).
fn native_class(rig: &Rig, manifest_sha256: &str) -> Profile {
    let mut profile = rig.profile.clone();
    profile.declared.native = Some(class_profile::Native {
        model: NATIVE_MODEL.to_owned(),
        manifest_sha256: manifest_sha256.to_owned(),
        adapter: FULL_FILE.id.to_owned(),
    });
    profile
}

/// The provider over `file` and a pid double answering `pid`, with the engine's runtime directory.
fn native_provider(
    file: native_provider::NativeFile,
    pid: Result<u32, habitat_engine::worker::native::Error>,
) -> (native_provider::NativeProvider<StandInPid>, AskedPid) {
    native_provider_at(
        file,
        pid,
        format!("/run/user/{}", rustix::process::geteuid().as_raw()).into(),
    )
}

/// The provider over `file` and a pid double answering `pid`, with `runtime_dir` as its runtime
/// directory.
fn native_provider_at(
    file: native_provider::NativeFile,
    pid: Result<u32, habitat_engine::worker::native::Error>,
    runtime_dir: PathBuf,
) -> (native_provider::NativeProvider<StandInPid>, AskedPid) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    (
        native_provider::NativeProvider::new(
            file,
            StandInPid {
                pid,
                seen: Arc::clone(&seen),
            },
            runtime_dir,
        ),
        seen,
    )
}

/// What `open` said, by the dispatcher's name for it.
fn opened_name<S, V>(opened: Result<(S, V), dispatcher::Unavailable>) -> &'static str {
    opened.err().map_or("opened", dispatcher::Unavailable::name)
}

/// R21 D2 (i)(ii)(v)(vi) as amended (N1, N6, N7, N22; S20a) · the production provider opens over
/// the stand-in: the daemon resolved through the `MainPid` seam under the dispatch deadline and the
/// dispatch's drain, the prompt read from the closure's pins and the admitted baseline — and the
/// source it opened drives the task to acceptance through the runtime. Each refusal is the
/// dispatcher's named state: a class with no native row, an operator manifest pin the class does
/// not name, a daemon the seam cannot give, a baseline without the class's `Cargo.toml`.
#[test]
fn the_native_provider_opens_over_the_stand_in_and_names_each_refusal() -> Outcome_ {
    let principal = owner();
    let next = habitat_engine::store::Dispatchable {
        task: TASK.to_owned(),
        generation: "1".to_owned(),
        owner: owner(),
        cancellation: false,
    };
    let stand_in = DaemonStandIn::spawn();
    let pid = stand_in.daemon().pid;
    let bench = rig(&Shape {
        base_cargo: true,
        ..Shape::default()
    })?;
    let fixture = native_fixture(&bench, &stand_in, &[(REFERENCE_LIB, "stop")], 4096)?;
    let manifest = fixture.manifest.sha256.clone();
    let class = native_class(&bench, &manifest);
    let drain = AtomicBool::new(false);
    let admitted = admitted_with(&bench, &principal, &drain, &class)?;
    let (_, until) = admitted.window();
    // The operator's manifest pin is not the class's: refused before the daemon is asked.
    let other = format!("sha256:{}", "5".repeat(64));
    let (mut provider, seen) = native_provider(operator_file(&fixture, &other)?, Ok(pid));
    assert_eq!(
        opened_name(dispatcher::Provider::open(&mut provider, &next, &admitted)),
        "unavailable: native manifest"
    );
    assert!(taken(&seen).is_empty(), "the daemon was not asked");
    // No daemon from the seam: its refusal, by name, asked once as the dispatch asks.
    let flag = std::ptr::from_ref(admitted.drain()) as usize;
    let (mut provider, seen) = native_provider(
        operator_file(&fixture, &manifest)?,
        Err(habitat_engine::worker::native::Error::Identity),
    );
    assert_eq!(
        opened_name(dispatcher::Provider::open(&mut provider, &next, &admitted)),
        "unavailable: daemon identity"
    );
    assert_eq!(taken(&seen), vec![(STAND_IN_UNIT.to_owned(), until, flag)]);
    // The stand-in: opened, and the source drives the task to acceptance.
    let (mut provider, seen) = native_provider(operator_file(&fixture, &manifest)?, Ok(pid));
    let (mut source, _live) = dispatcher::Provider::open(&mut provider, &next, &admitted)
        .map_err(dispatcher::Unavailable::name)?;
    assert_eq!(taken(&seen), vec![(STAND_IN_UNIT.to_owned(), until, flag)]);
    let (mut verifier, handed) = oracle(vec![matched(7)]);
    let outcome = drive(&bench.tasks, *admitted, &mut source, &mut verifier)
        .map_err(|e| format!("{e:?}"))?
        .outcome;
    assert_eq!(outcome, Outcome::Driven(Driven::Accepted));
    assert_eq!(state(&bench)?, "accepted");
    assert_eq!(taken(&handed)[0].1, REFERENCE_LIB.as_bytes());
    // A class with no native row, and a baseline without the class's `Cargo.toml`.
    for (base_cargo, native, expected) in [
        (true, false, "unavailable: class declares no native model"),
        (false, true, "unavailable: prompt cargo"),
    ] {
        let rig = rig(&Shape {
            base_cargo,
            ..Shape::default()
        })?;
        let fixture = native_fixture(&rig, &stand_in, &[(REFERENCE_LIB, "stop")], 4096)?;
        let manifest = fixture.manifest.sha256.clone();
        let class = if native {
            native_class(&rig, &manifest)
        } else {
            rig.profile.clone()
        };
        let admitted = admitted_with(&rig, &principal, &drain, &class)?;
        let (mut provider, _) = native_provider(operator_file(&fixture, &manifest)?, Ok(pid));
        assert_eq!(
            opened_name(dispatcher::Provider::open(&mut provider, &next, &admitted)),
            expected
        );
    }
    Ok(())
}

/// R22 step 4 (C7) · `open`'s composition is pinned: the live verifier it returns carries the
/// class's systemd-run pin and the provider's runtime directory, and its aggregate lifecycle the
/// class's busctl pin over the same runtime directory. Two fixtures differing in every field, so a
/// swapped or constant pin cannot pass either.
#[test]
fn the_native_provider_composes_the_live_verifier_from_the_class_s_pins() -> Outcome_ {
    let principal = owner();
    let next = habitat_engine::store::Dispatchable {
        task: TASK.to_owned(),
        generation: "1".to_owned(),
        owner: owner(),
        cancellation: false,
    };
    let stand_in = DaemonStandIn::spawn();
    let pid = stand_in.daemon().pid;
    let drain = AtomicBool::new(false);
    for (systemd_run, busctl, runtime_dir) in [
        (
            format!("sha256:{}", "a1".repeat(32)),
            format!("sha256:{}", "b2".repeat(32)),
            "/run/hee3-c7/first",
        ),
        (
            format!("sha256:{}", "3c".repeat(32)),
            format!("sha256:{}", "4d".repeat(32)),
            "/var/tmp/hee3-c7-second",
        ),
    ] {
        let bench = rig(&Shape {
            base_cargo: true,
            ..Shape::default()
        })?;
        let fixture = native_fixture(&bench, &stand_in, &[(REFERENCE_LIB, "stop")], 4096)?;
        let manifest = fixture.manifest.sha256.clone();
        let mut class = native_class(&bench, &manifest);
        class.declared.systemd_run_sha256.clone_from(&systemd_run);
        class.declared.busctl_sha256.clone_from(&busctl);
        let admitted = admitted_with(&bench, &principal, &drain, &class)?;
        let (mut provider, _) = native_provider_at(
            operator_file(&fixture, &manifest)?,
            Ok(pid),
            runtime_dir.into(),
        );
        let (_, verifier) = dispatcher::Provider::open(&mut provider, &next, &admitted)
            .map_err(dispatcher::Unavailable::name)?;
        assert_eq!(
            format!("{verifier:?}"),
            format!(
                "LiveVerifier {{ systemd_run_sha256: \"{systemd_run}\", runtime_dir: \"{runtime_dir}\", \
                 aggregates: Manager {{ busctl_sha256: \"{busctl}\", runtime_dir: \"{runtime_dir}\" }} }}"
            )
        );
    }
    Ok(())
}

/// R21 round-1 LOW L2 (N11) · `open` refuses a client working directory that is not canonical, the
/// engine's and 0700 as `unavailable: native directory`, after the daemon is resolved (the seam's
/// log holds the one ask, as the dispatch asks it): a 0755 directory and an absent one. The
/// fixture's own 0700 directory, handed the same way, opens.
#[test]
fn a_client_directory_that_is_not_private_is_refused_at_open_by_name() -> Outcome_ {
    use std::os::unix::fs::PermissionsExt;
    let principal = owner();
    let next = habitat_engine::store::Dispatchable {
        task: TASK.to_owned(),
        generation: "1".to_owned(),
        owner: owner(),
        cancellation: false,
    };
    let stand_in = DaemonStandIn::spawn();
    let pid = stand_in.daemon().pid;
    let bench = rig(&Shape {
        base_cargo: true,
        ..Shape::default()
    })?;
    let fixture = native_fixture(&bench, &stand_in, &[(REFERENCE_LIB, "stop")], 4096)?;
    let manifest = fixture.manifest.sha256.clone();
    let class = native_class(&bench, &manifest);
    let drain = AtomicBool::new(false);
    let admitted = admitted_with(&bench, &principal, &drain, &class)?;
    let (_, until) = admitted.window();
    let flag = std::ptr::from_ref(admitted.drain()) as usize;
    let shared = bench.scratch.0.join("client-0755");
    DirBuilder::new().mode(0o700).create(&shared)?;
    fs::set_permissions(&shared, fs::Permissions::from_mode(0o755))?;
    for (directory, expected) in [
        (shared, "unavailable: native directory"),
        (
            bench.scratch.0.join("client-absent"),
            "unavailable: native directory",
        ),
        (fixture.directory.clone(), "opened"),
    ] {
        let file = operator_file(
            &habitat_engine::worker::native::Profile {
                directory: directory.clone(),
                ..fixture.clone()
            },
            &manifest,
        )?;
        assert_eq!(file.directory, directory);
        let (mut provider, seen) = native_provider(file, Ok(pid));
        assert_eq!(
            opened_name(dispatcher::Provider::open(&mut provider, &next, &admitted)),
            expected,
            "{}",
            directory.display()
        );
        assert_eq!(taken(&seen), vec![(STAND_IN_UNIT.to_owned(), until, flag)]);
    }
    Ok(())
}
