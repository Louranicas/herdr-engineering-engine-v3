//! The attempt lifecycle over the real ledger (B14a-1c, design B14a-R1..R5): `StoreRuntime`
//! implements the driver's ports by composing Store, the class's candidate application and a
//! verifier seam. It reaches the ledger only through `StoreTasks::with_store`, one hold per
//! write together with the head read that write depends on, and holds no `Store` between calls,
//! so no workload can run under the lock.
//!
//! Not here: the real workload (B14a-3), the receipt composer (B14a-2), the native candidate
//! source (B14a-4) and the dispatcher that chooses a [`Dispatch`] from the roster (B14b). This
//! module is the trusted check owner for one verdict only: a candidate the class refuses is
//! recorded `Failed` with no criterion satisfied, without a verifier call (B14a-R2.4).

use super::capture;
use super::class_profile::{Profile, Workspace};
use super::evidence::{Evidence as Sink, digest, fresh_id};
use super::live_verifier::{Cleanup, checked};
use super::plan;
use super::repair::{self, Failure};
use super::run_records::{
    Intents, ObligationRecord, OutcomeName, OutputReadback, Readbacks, RunCleanup, RunClock,
    RunOutcome, RunRecord as _, RuntimeClock, Settlement as RecordSettlement,
};
use super::tasks::{Poisoned, StoreTasks};
use super::u64_receipt;
use super::workload::{self, Outcome as WorkloadOutcome, Run, Step, Tools};
use crate::check::collector;
use crate::check::consistency::{U64_BOUNDS, U64_CRITERIA, U64_EDITABLE};
use crate::check::decision::CLEANUP_GRACE_MS;
use crate::contracts::control::criteria_digest;
use crate::contracts::receipt::Name;
use crate::contracts::receipt::{Address as _, ReceiptV1, Ref};
use crate::contracts::roster::{MAX_HISTORY, Selection};
use crate::contracts::{Generation, Sha256Digest, UuidV4};
use crate::store::{
    self, Binding, Effect, EvidenceIdentity, Expected, Identified, Object, Principal, RosterStart,
    RunRecord as StoreRunRecord, RunRecordKind, Settlement, Stop, Store, TaskHead, Verification,
    VerificationVerdict,
};
use crate::task::LoopRefusal;
use crate::task::driver::{self, Acceptance, Checked as DriverChecked, StopReason, Work};
use crate::worker::host;
use crate::worker::resources::TERM_GRACE;
use crate::worker::workspace::{self, FileIdentity, Snapshot};
use std::fs;
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// A candidate source's answer: the editable file's next full text, or nothing more to offer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Candidate {
    Replacement(Vec<u8>),
    Exhausted,
    /// The source's text is not a candidate by the class grammar (B14a-4, R18 A5): recorded as a
    /// refused candidate under the refusal's name through the one existing path, never an edit;
    /// `text` is what was refused, so the record's `candidate_sha256` names it.
    Refused {
        refusal: super::candidates::Refusal,
        text: Vec<u8>,
    },
    /// The provider could not be asked, or answered without a final candidate (R18 A3): the task
    /// stops — no further generate after a lost response — with the failure named in the stop body;
    /// `cleanup_settled` is every exchange's custody, `retained` the children the source now holds.
    /// STATED GAP (review F2): with `cleanup_settled` false the attempt is `Unsettled`, an unsettled
    /// attempt stops nothing (B14a-R1.4), and nothing in B14a-4 settles it later — B14b, which
    /// settles the retained children, records the failure then.
    Provider {
        error: crate::worker::native::Error,
        state: crate::worker::native::ProviderState,
        cleanup_settled: bool,
        retained: usize,
    },
}

/// A provider failure the last attempt ended in, for the stop body (R18 A3).
#[derive(Clone, Copy, Debug)]
struct ProviderFailure {
    error: crate::worker::native::Error,
    state: crate::worker::native::ProviderState,
    retained: usize,
}

/// What the previous verification observed, handed to the next candidate request (F6).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Previous {
    pub verdict: VerificationVerdict,
    pub criteria: u64,
    pub evidence: Vec<u8>,
    /// The schema the evidence was recorded under, so a source can read the runtime's own records
    /// (`hee3.refused-candidate/1`) and leave a receipt unread (B14a-4, A14).
    pub schema_id: String,
}

/// What a source is asked for one attempt (B14a-4, R18 A1): everything an adapter needs and the
/// source could otherwise re-acquire — the attempt's binding, an invocation id the runtime minted,
/// the recipe (the class profile's digest) and workspace (the baseline's) digests, the dispatch's
/// origin, the attempt's work window, the runtime's cancel flag — and the previous verification.
pub struct Ask<'a> {
    pub task: UuidV4<'a>,
    pub attempt: UuidV4<'a>,
    pub generation: Generation,
    pub invocation: UuidV4<'a>,
    pub recipe: Sha256Digest<'a>,
    pub workspace: Sha256Digest<'a>,
    pub origin: Instant,
    pub work_until: Instant,
    pub cancelled: &'a AtomicBool,
    pub previous: Option<&'a Previous>,
}

/// Where candidates come from. `next` is asked once per attempt, inside `execute`.
pub trait CandidateSource {
    fn next(&mut self, ask: &Ask<'_>) -> Candidate;
}

/// One independent check of an applied candidate: its verdict, the criterion bits it satisfied,
/// the evidence bytes to retain, its measured cost (`None` when unknown) and whether its own
/// cleanup settled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Check {
    pub verdict: VerificationVerdict,
    pub criteria: u64,
    pub evidence: Vec<u8>,
    /// The schema `evidence` decodes under, named by the verifier that produced it (B09b): the
    /// ledger records it beside the object so a view returns the reference as recorded.
    pub schema_id: String,
    pub used_ms: Option<u64>,
    pub cleanup_settled: bool,
}

/// Every check's evidence is JSON; the media type is the runtime's, the schema the verifier's.
const CHECK_MEDIA_TYPE: &str = "application/json";
/// The schema of the runtime's own idle verification, recorded when a cancelled task's last
/// attempt was never checked.
const IDLE_VERIFICATION_SCHEMA: &str = "hee3.idle-verification/1";
/// The schema of the runtime's own class check of a refused candidate (B14a-R2.4).
pub(crate) const REFUSED_CANDIDATE_SCHEMA: &str = "hee3.refused-candidate/1";
/// The schema of the runtime's own record of a check it did not run: the verify reservation held
/// no time past the check's teardown share (B14a-3b, R14.1).
const CHECK_WINDOW_EMPTY_SCHEMA: &str = "hee3.check-window-empty/1";
/// The schema of a verifier's check as the runtime records it (B14a-3c, R15.7): the outcome, whether
/// every step was captured, and the run records it cites — each `{artifact_id, sha256,
/// byte_length}` under its kind — which `accept` reads back against the ledger's commitment.
pub const U64_CHECK_SCHEMA: &str = "hee3.u64-check/1";
/// The schema of a task stop's evidence — the `kind` the stop body names.
const TASK_STOP_SCHEMA: &str = "hee3.task-stop/1";
/// The schema of a pre-dispatch refusal's evidence — the `kind` its body names.
const PRE_DISPATCH_REFUSAL_SCHEMA: &str = "hee3.pre-dispatch-refusal/1";
/// A per-check plan that refused before the producer ran (R17 round 2, N2): the runtime's fourth
/// own evidence shape — no run records, the refusal named, verdict `Error`.
const PLAN_REFUSED_SCHEMA: &str = "hee3.plan-refused/1";

/// What one check is handed (R15 round 2): the frozen applied snapshot (never a path to mutate),
/// the protected snapshot, a private job root of its own beside the attempt's directory, the window
/// it may run in (B14a-3b: `until` is its cutoff, `teardown_until` how long its teardown may take)
/// and the runtime's cancellation flag.
pub struct CheckPlan<'a> {
    pub subject: &'a Snapshot,
    pub protected: &'a Snapshot,
    pub job_root: &'a Path,
    /// The pins the verifier launches with: the runtime's own value, the one its plan described
    /// (R17 round 2, decision 2) — never re-derived by the verifier.
    pub tools: &'a Tools,
    pub window: CheckWindow,
    pub cancelled: &'a AtomicBool,
}

/// What the verifier saw: the workload's run, or its refusal to launch, and when the observation
/// was complete. The verifier decides nothing and publishes nothing — the runtime derives the
/// check from this and records it (R15 round 2).
pub struct Observed {
    pub run: Result<Run, workload::Error>,
    pub observed: Instant,
}

/// The check of an applied candidate: it observes a run of the workload inside the plan's window.
pub trait Verifier {
    fn check(&mut self, plan: CheckPlan<'_>) -> Observed;
}

/// What the check keeps back for its own teardown after its cutoff — stopping the scope
/// (`TERM_GRACE`), the subject readbacks and the scratch release. Derived from RC04's cleanup
/// grace, spelled once in `check::decision`, so a check that honours its window is never late by
/// construction (R14.2); it must hold at least the scope's stop grace.
pub const CHECK_TEARDOWN: Duration = Duration::from_millis(CLEANUP_GRACE_MS);
const _: () = assert!(CHECK_TEARDOWN.as_millis() >= TERM_GRACE.as_millis());

/// The window one check may run in (R14): from `begun`, its cutoff `until` — the verify
/// reservation less [`CHECK_TEARDOWN`], or the task deadline less the same, whichever is first —
/// and `teardown_until`, how long its teardown may run past the cutoff within the task deadline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckWindow {
    /// When the window was computed: the check's own origin.
    pub begun: Instant,
    /// The origin as unix milliseconds, read once with it (the clock record's `origin_unix_ms`).
    pub begun_unix_ms: u64,
    /// The check's cutoff.
    pub until: Instant,
    /// The end of the check's teardown share.
    pub teardown_until: Instant,
}

/// The check window from the verify reservation the ledger holds now, or `None` when no time is
/// left for a check after its teardown share (R14.1: the runtime then records the check as not
/// run). Pure over its arguments, so every branch is reachable by choosing them (F95).
#[must_use]
pub(crate) fn check_window(
    now: Instant,
    now_unix_ms: u64,
    reserved_verify_ms: u64,
    task_deadline: Instant,
) -> Option<CheckWindow> {
    let lease = reserved_verify_ms.checked_sub(millis(CHECK_TEARDOWN))?;
    let until = task_deadline
        .checked_sub(CHECK_TEARDOWN)?
        .min(now + Duration::from_millis(lease));
    if until <= now {
        return None;
    }
    Some(CheckWindow {
        begun: now,
        begun_unix_ms: now_unix_ms,
        until,
        teardown_until: task_deadline.min(until + CHECK_TEARDOWN),
    })
}

/// What the dispatcher decides for one task (B14a-R1.7): the roster record and selections the
/// attempt runs under, the runtime's one attempts directory, the identities a captured workspace
/// may not contain, and the two shares of the owner's reservation (B14a-R1.8, R2.5) — no limit is
/// created here.
#[derive(Clone, Copy, Debug)]
pub struct Dispatch<'a> {
    pub principal: &'a Principal,
    pub task: UuidV4<'a>,
    pub agent_record_id: &'a str,
    pub selections: &'a [Selection],
    pub attempts: &'a Path,
    pub forbidden: &'a [FileIdentity],
    /// Kept back from each work deadline for teardown.
    pub teardown_ms: u64,
    /// The capture's own share: a reservation below it is refused before any capture.
    pub capture_ms: u64,
}

/// Why a task was stopped before any attempt began, named in its stop (B14a-R1.4d, R1.5, R2.5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    CriteriaNotClass,
    ReservationEmpty,
    ReservationTooSmall,
    /// The verify reservation holds nothing past the check's teardown share (R14.2).
    VerifyReservationTooSmall,
    WorkspaceNotDeclared,
    BaselineCapture,
    BaselineMismatch,
    ProtectedCapture,
    ProtectedMismatch,
    /// The shared plan could not be published before dispatch (R17 round 2, N2): the refusal's
    /// name is the stop's reason (`plan_pin_compiler`, `plan_closure`, …); its detail and the ids
    /// of any objects it left in the CAS are not recorded by the stop (N13, open for B14a-4).
    Plan(&'static str),
}

impl Refusal {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Plan(kind) => kind,
            Self::CriteriaNotClass => "criteria_not_class",
            Self::ReservationEmpty => "reservation_empty",
            Self::ReservationTooSmall => "reservation_too_small",
            Self::VerifyReservationTooSmall => "verify_reservation_too_small",
            Self::WorkspaceNotDeclared => "workspace_not_declared",
            Self::BaselineCapture => "baseline_capture",
            Self::BaselineMismatch => "baseline_mismatch",
            Self::ProtectedCapture => "protected_capture",
            Self::ProtectedMismatch => "protected_mismatch",
        }
    }
}

/// What one dispatch came to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// Stopped before any attempt, by name; no attempt row exists.
    Refused(Refusal),
    Driven(driver::Outcome),
}

#[derive(Debug)]
pub enum Error {
    Store(store::Error),
    /// The ledger's owner panicked while holding it.
    Poisoned,
    /// Another writer touched the task other than by one cancellation or an instance observation
    /// (B14a-R2.1, R3 G1): a second dispatcher or a restart overlap, detected, never absorbed.
    ConcurrentWriter,
    /// A stored value the runtime wrote or reads is not the shape it must be.
    Identity,
    /// No entropy for a fresh identity before the deadline.
    Entropy,
    /// The driver's own policy refused (criteria cardinality, or an undeclared bit).
    Policy(LoopRefusal),
}

impl From<store::Error> for Error {
    fn from(error: store::Error) -> Self {
        Self::Store(error)
    }
}

impl From<Poisoned> for Error {
    fn from(_: Poisoned) -> Self {
        Self::Poisoned
    }
}

/// A port's failure: an error, or a stop the driver has no port to express — a begin that meets a
/// cancellation committed after the driver's last read, or a work reservation spent before an
/// attempt could begin. [`dispatch`] takes the stop, so neither strands the task (review M4).
#[derive(Debug)]
enum Fault {
    Error(Error),
    Stop(StopReason),
}

impl From<Error> for Fault {
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}

impl From<store::Error> for Fault {
    fn from(error: store::Error) -> Self {
        Self::Error(Error::Store(error))
    }
}

impl From<Poisoned> for Fault {
    fn from(_: Poisoned) -> Self {
        Self::Error(Error::Poisoned)
    }
}

/// The opaque ticket for one begun attempt.
#[derive(Debug)]
pub struct Attempt {
    index: usize,
}

/// A passing check's retained proof: the evidence object and the subject it bound.
#[derive(Debug)]
pub struct Evidence {
    object: Object,
    subject: String,
    /// The identity the verification recorded the object under (B09b): the acceptance names the
    /// same object by the same id.
    artifact_id: String,
    schema_id: String,
}

#[derive(Debug)]
struct Begun {
    id: String,
    generation: String,
    /// Where this attempt's measured cost starts: the origin for the first (preparation is
    /// charged to it, B14a-R2.5), its begin for later ones.
    charged_from: Instant,
    /// The end of this attempt's work window: its charge start plus what the reservation held at
    /// its begin, less the teardown share (B14a-R1.8, re-read at every begin).
    work_until: Instant,
    applied: Option<Snapshot>,
    /// A candidate the class refused, and the refusal's name.
    refused: Option<(Vec<u8>, &'static str)>,
    /// The provider failure this attempt ended in, when it did (R18 A3).
    provider: Option<ProviderFailure>,
    /// The work settled with a known cost and settled cleanup.
    settled: bool,
    verified: bool,
    /// The check, once recorded, had a known cost and settled cleanup (review H1).
    check_settled: bool,
    /// The per-check plan this attempt's check ran under (R17 round 2, shape C), when one was.
    planned: Option<plan::Planned>,
    /// The run id of the receipt this attempt's check recorded, for the next attempt's
    /// `parent_run` (L3); `None` when the check's evidence was not a receipt.
    receipt_run: Option<String>,
}

/// What a verification commits beside its row: the run records and the objects they cite.
#[derive(Clone, Copy)]
struct Committing<'a> {
    records: &'a [StoreRunRecord<'a>],
    cited: &'a [Object],
    /// The evidence object when the composer already published it (the receipt root under the
    /// verification's artifact id — R16r2.6): `commit` then publishes nothing itself.
    root: Option<&'a Object>,
}

impl Committing<'static> {
    /// The runtime's own checks: no run, no records.
    const NONE: Self = Self {
        records: &[],
        cited: &[],
        root: None,
    };
}

/// What the verifier's observation is recorded with: the window it ran in, the job root to tear
/// down, the applied snapshot to read back, the subject digest and the ids minted at plan time.
struct Observation<'a> {
    observed: Observed,
    window: CheckWindow,
    job_root: &'a Path,
    applied: &'a Snapshot,
    subject: &'a str,
    ids: &'a [String; 3],
}

/// What a check's teardown settled and read back, with the two records derived from it.
struct Settled {
    seed_verified: bool,
    subjects_verified: bool,
    protected_unchanged: bool,
    cleanup: Cleanup,
    cleanup_record: RunCleanup,
    clock: RunClock,
}

/// What the 3c evidence is built from when no receipt is recorded: the derived check, the
/// captured objects, the records and the run.
struct Fallback<'a> {
    cited_objects: Vec<Object>,
    published: &'a [(RunRecordKind, String, Object)],
    run: &'a Run,
    failed_at: Option<&'a str>,
    derived: super::live_verifier::Checked,
    root_id: &'a str,
}

/// A composed receipt with every object its sink holds, or the composer's refusal.
type Composition = Result<(u64_receipt::Composed, Vec<(Ref, Object)>), ComposeRefusal>;

/// Why the runtime could not compose a receipt: the host's facts unreadable, no fresh ids for the
/// obligation rows, or the composer's own refusal — each named in the 3c evidence.
#[derive(Debug)]
enum ComposeRefusal {
    HostFacts(host::Error),
    FreshIds(Error),
    Compose(u64_receipt::Refusal),
}

impl From<u64_receipt::Refusal> for ComposeRefusal {
    fn from(refusal: u64_receipt::Refusal) -> Self {
        Self::Compose(refusal)
    }
}

impl ComposeRefusal {
    /// The refusal as the 3c evidence names it.
    fn describe(&self) -> String {
        match self {
            Self::HostFacts(error) => format!("host_facts: {error:?}"),
            Self::FreshIds(error) => format!("fresh_ids: {error:?}"),
            Self::Compose(refusal) => format!("{refusal:?}"),
        }
    }
}

/// What one receipt is composed from, beside the plan: the check's records and run.
#[derive(Clone, Copy)]
struct ComposeInputs<'a> {
    clock: &'a RunClock,
    outcome: &'a RunOutcome,
    cleanup: &'a RunCleanup,
    records: &'a [(RunRecordKind, String, Object)],
    captures: &'a [Option<capture::Captured>],
    run: &'a Run,
    root_id: &'a str,
    seed_verified: bool,
    subjects_verified: bool,
    protected_unchanged: bool,
}

/// One verification as the ledger committed it, for the runtime's own bookkeeping.
struct Committed {
    generation: String,
    event: String,
    object: Object,
    artifact_id: String,
    verdict: VerificationVerdict,
    criteria: u64,
    schema_id: String,
    reconciled: bool,
    evidence: Vec<u8>,
}

/// The attempt lifecycle for one task over the real ledger.
struct StoreRuntime<'a, C, V> {
    tasks: &'a StoreTasks,
    dispatch: Dispatch<'a>,
    source: C,
    verifier: V,
    baseline: Snapshot,
    protected: Snapshot,
    /// The pins every check launches with and the plan describes: one value (R17 round 2, 2).
    tools: Tools,
    /// The class profile the dispatch read: the plan's declarations and the readback's digest.
    profile: &'a Profile,
    /// The shared part of every plan this dispatch composes, published once at dispatch.
    shared: plan::Shared,
    /// The flag a check's workload reads; nothing sets it in B14a-3c (a durable cancel during a
    /// check is honoured at the driver's next read — B19/B21 own the wake).
    cancelled: AtomicBool,
    digests: [String; 3],
    origin: Instant,
    deadline: Instant,
    /// The generation after the runtime's own last write (at dispatch: the one it read).
    own: u64,
    /// The runtime's own last event (at dispatch: the task's latest), the anchor every
    /// second-writer check reads from.
    last_event: String,
    attempts: Vec<Begun>,
    previous: Option<Previous>,
}

/// The most task events that may follow the runtime's own last write: every instance observation
/// the roster's history can hold, plus one cancellation (R3 G1). Derived, not chosen.
const EVENTS_LIMIT: u64 = MAX_HISTORY as u64 + 1;

/// Dispatch one admitted task: the pre-dispatch checks, then the driver over `StoreRuntime`.
///
/// # Errors
/// A store, lock, identity or entropy failure, or a second writer, from any step.
pub fn dispatch<C: CandidateSource, V: Verifier>(
    tasks: &StoreTasks,
    profile: &Profile,
    dispatch: Dispatch<'_>,
    source: C,
    verifier: V,
) -> Result<Outcome, Error> {
    let origin = Instant::now();
    let deadline = origin + crate::task::TASK_LIMIT;
    let (head, anchor) = tasks.with_store(|store| -> Result<_, Error> {
        let head = store.get(dispatch.principal, dispatch.task, deadline)?;
        let anchor = store.last_event(dispatch.principal, dispatch.task, deadline)?;
        Ok((head, anchor))
    })??;
    let prepared = match prepare(&head, profile, &dispatch, origin, deadline) {
        Ok(prepared) => prepared,
        Err(refusal) => {
            refuse(tasks, &dispatch, refusal, origin, deadline)?;
            return Ok(Outcome::Refused(refusal));
        }
    };
    // The shared plan, once per dispatch, in a hold of its own (R17 round 2, shape S): its
    // refusal is a stop before any attempt, named by kind. It runs under the dispatch's own
    // deadline — the capture share bounds the two snapshot captures, not megabytes of pinned
    // bytes and a compiler probe — and its time is attempt 1's, charged from the origin
    // (B14a-R2.5; R17 round 2 N6 revisited: the deadline passed through is the task's).
    let tools = super::live_verifier::tools(&profile.declared);
    let cancelled = AtomicBool::new(false);
    let plan_root = dispatch
        .attempts
        .join(format!("{}.plan", dispatch.task.as_str()));
    let shared = tasks.with_store(|store| {
        let mut sink = Sink::new(store, deadline);
        plan::shared(
            &mut sink,
            &plan::Inputs {
                tools: &tools,
                profile,
                baseline: &prepared.baseline,
                protected: &prepared.protected,
                plan_root: &plan_root,
                deadline,
                cancelled: &cancelled,
            },
        )
    })?;
    let shared = match shared {
        Ok(shared) => shared,
        Err(refusal) => {
            let stop = Refusal::Plan(refusal.name());
            refuse(tasks, &dispatch, stop, origin, deadline)?;
            return Ok(Outcome::Refused(stop));
        }
    };
    let mut runtime = StoreRuntime {
        tasks,
        dispatch,
        source,
        verifier,
        baseline: prepared.baseline,
        protected: prepared.protected,
        tools,
        profile,
        shared,
        cancelled,
        digests: prepared.digests,
        origin,
        deadline,
        own: number(&head.generation)?,
        last_event: anchor,
        attempts: Vec::new(),
        previous: None,
    };
    let count = u8::try_from(U64_CRITERIA.len()).map_err(|_| Error::Identity)?;
    match driver::run(&mut runtime, count) {
        Ok(outcome) => Ok(Outcome::Driven(outcome)),
        Err(driver::Error::Runtime(Fault::Stop(reason))) => {
            Ok(Outcome::Driven(if runtime.stop_task(reason)? {
                driver::Outcome::Stopped(reason)
            } else {
                driver::Outcome::NeedsSettlement(reason)
            }))
        }
        Err(driver::Error::Runtime(Fault::Error(error))) => Err(error),
        Err(driver::Error::Policy(refusal)) => Err(Error::Policy(refusal)),
    }
}

struct Prepared {
    baseline: Snapshot,
    /// The protected snapshot, kept whole: every check runs the workload against it (R15.2).
    protected: Snapshot,
    digests: [String; 3],
}

/// The pre-dispatch checks, free ones first (B14a-R2.5): the class's criteria, the reservation,
/// the declared workspace, then the two captures and their digests.
fn prepare(
    head: &TaskHead,
    profile: &Profile,
    dispatch: &Dispatch<'_>,
    origin: Instant,
    deadline: Instant,
) -> Result<Prepared, Refusal> {
    if head.criteria != criteria_digest(&U64_CRITERIA) {
        return Err(Refusal::CriteriaNotClass);
    }
    if head.reserved_work_ms == 0 {
        return Err(Refusal::ReservationEmpty);
    }
    // The capture and the teardown are both shares of the work reservation (B14a-R1.8, R2.5): a
    // reservation that cannot hold both and leave work time is refused before any capture.
    if head.reserved_work_ms <= dispatch.capture_ms.saturating_add(dispatch.teardown_ms) {
        return Err(Refusal::ReservationTooSmall);
    }
    // A check keeps its teardown share back from the verify reservation the same way (R14.2): one
    // that could never hold a check is refused here, before any capture.
    if head.reserved_verify_ms <= millis(CHECK_TEARDOWN) {
        return Err(Refusal::VerifyReservationTooSmall);
    }
    let workspace: &Workspace = head
        .workspace_id
        .as_deref()
        .and_then(|id| profile.declared.workspaces.iter().find(|row| row.id == id))
        .ok_or(Refusal::WorkspaceNotDeclared)?;
    let capture_deadline = deadline.min(origin + Duration::from_millis(dispatch.capture_ms));
    let baseline = Snapshot::capture(
        &profile.directory.join(&workspace.baseline),
        dispatch.forbidden,
        capture_deadline,
    )
    .map_err(|_| Refusal::BaselineCapture)?;
    if baseline.content_digest().as_deref() != Some(workspace.baseline_digest.as_str()) {
        return Err(Refusal::BaselineMismatch);
    }
    let protected = Snapshot::capture(
        &profile.directory.join(&workspace.protected),
        dispatch.forbidden,
        capture_deadline,
    )
    .map_err(|_| Refusal::ProtectedCapture)?;
    if protected.content_digest().as_deref() != Some(workspace.protected_digest.as_str()) {
        return Err(Refusal::ProtectedMismatch);
    }
    Ok(Prepared {
        baseline,
        protected,
        digests: binding_digests(
            &workspace.baseline_digest,
            &workspace.protected_digest,
            profile,
        ),
    })
}

/// The one place a binding's three digests are assembled (F4 disposition), each from its own
/// source: the two declared digests the captures were just compared to, and the profile's own.
fn binding_digests(baseline: &str, protected: &str, profile: &Profile) -> [String; 3] {
    [
        baseline.to_owned(),
        protected.to_owned(),
        profile.digest.clone(),
    ]
}

/// Stop a task refused before dispatch through `finish_preparation`, charging the time since the
/// origin; no attempt row is written. The head is read in the stop's own hold (review M1): a
/// cancellation committed during the captures is stopped with it, not stranded.
fn refuse(
    tasks: &StoreTasks,
    dispatch: &Dispatch<'_>,
    refusal: Refusal,
    origin: Instant,
    deadline: Instant,
) -> Result<(), Error> {
    let (staging, event, artifact) = (fresh(deadline)?, fresh(deadline)?, fresh(deadline)?);
    let reason = Name::new(refusal.name()).map_err(|_| Error::Identity)?;
    tasks.with_store(|store| -> Result<(), Error> {
        let head = store.get(dispatch.principal, dispatch.task, deadline)?;
        let observed_ms = millis(origin.elapsed());
        let bytes = serde_json::to_vec(&serde_json::json!({
            "kind": PRE_DISPATCH_REFUSAL_SCHEMA,
            "task": dispatch.task.as_str(),
            "refusal": refusal.name(),
            "observed_ms": observed_ms,
        }))
        .map_err(|_| Error::Identity)?;
        let object = store.publish(&bytes, uuid(&staging)?, deadline)?;
        store.finish_preparation(
            dispatch.principal,
            Stop {
                task: dispatch.task,
                generation: parse_generation(&head.generation)?,
                reason: &reason,
                evidence: &object,
                identity: EvidenceIdentity {
                    artifact_id: uuid(&artifact)?,
                    media_type: CHECK_MEDIA_TYPE,
                    schema_id: PRE_DISPATCH_REFUSAL_SCHEMA,
                },
                event: uuid(&event)?,
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(preparation_charge(observed_ms, head.reserved_work_ms)),
                cleanup_settled: true,
                ready_to_verify: false,
            },
            deadline,
        )?;
        Ok(())
    })?
}

/// Remove a refused candidate's retained path and read it back as absent: `true` only when the
/// removal succeeded and the name no longer resolves (`NotFound`, never any other error). A
/// failure is a cleanup that did not settle, not an error of the runtime (review M4a).
fn remove(failure: &Failure, deadline: Instant) -> bool {
    let Some(path) = &failure.partial_path else {
        return true;
    };
    workspace::remove_owned(path, deadline).is_ok()
        && matches!(
            std::fs::symlink_metadata(path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        )
}

impl<C: CandidateSource, V: Verifier> StoreRuntime<'_, C, V> {
    /// The head, admitted only if every task event after the runtime's own last one (or, before
    /// its first write, after the task's latest at dispatch) is one cancellation or an instance
    /// observation, and the generation accounts for exactly those (B14a-R2.1, R3 G1). Events are
    /// always read, so a foreign write at the current generation is seen too (review M3). Called
    /// inside the hold of the write it guards.
    fn current(&self, store: &Store) -> Result<TaskHead, Error> {
        let head = store.get(self.dispatch.principal, self.dispatch.task, self.deadline)?;
        let events = store.events_after(
            self.dispatch.principal,
            self.dispatch.task,
            uuid(&self.last_event)?,
            EVENTS_LIMIT,
            self.deadline,
        )?;
        let mut cancellations = 0_u64;
        for event in &events {
            match event.kind.as_str() {
                "cancellation_requested" => cancellations += 1,
                "roster.instance_observed" => {}
                _ => return Err(Error::ConcurrentWriter),
            }
        }
        if cancellations > 1 || number(&head.generation)? != self.own + cancellations {
            return Err(Error::ConcurrentWriter);
        }
        Ok(head)
    }

    fn written(&mut self, generation: &str, event: String) -> Result<(), Error> {
        self.own = number(generation)?;
        self.last_event = event;
        Ok(())
    }

    fn begun(&self, attempt: &Attempt) -> Result<&Begun, Error> {
        self.attempts.get(attempt.index).ok_or(Error::Identity)
    }

    /// One settle of the current attempt, in one hold with its head read. `used_ms` is `None`
    /// when the measured time overran what the reservation holds now (an overrun is not a clean
    /// failure).
    fn settle(
        &mut self,
        index: usize,
        cleanup_settled: bool,
        ready_to_verify: bool,
    ) -> Result<(), Error> {
        let begun = self.attempts.get(index).ok_or(Error::Identity)?;
        let used = millis(begun.charged_from.elapsed());
        let event = fresh(self.deadline)?;
        let (generation, known) = self.tasks.with_store(|store| -> Result<_, Error> {
            let head = self.current(store)?;
            let used_ms = (used <= head.reserved_work_ms).then_some(used);
            let generation = store.settle_attempt(
                &expected(&head, begun)?,
                Settlement {
                    effect: Effect::None,
                    used_ms,
                    cleanup_settled,
                    ready_to_verify,
                },
                uuid(&event)?,
                self.deadline,
            )?;
            Ok((generation, used_ms.is_some()))
        })??;
        self.written(&generation, event)?;
        if let Some(begun) = self.attempts.get_mut(index) {
            begun.settled = known && cleanup_settled;
        }
        Ok(())
    }

    /// Record one check of the current attempt, in one hold with its head read, and remember it
    /// for the next candidate. A cost past what the verify reservation holds is recorded unknown
    /// (B14a-R1.8, review M4c); a `Cancelled` verdict for a task nobody cancelled is the
    /// verifier's error, never a cancellation. Returns the evidence and whether the check settled.
    /// Publish `check`'s evidence and commit the verification with `records`, in the caller's
    /// hold: one transaction for the verdict, the receipt and the run records (R13.2).
    fn commit(
        &self,
        store: &mut Store,
        begun: &Begun,
        check: &Check,
        subject: &str,
        ids: &[String; 3],
        committing: Committing<'_>,
    ) -> Result<Committed, Error> {
        let [staging, event, artifact] = ids;
        let head = self.current(store)?;
        let used_ms = check
            .used_ms
            .filter(|used| *used <= head.reserved_verify_ms);
        // A `Cancelled` verdict with no cancellation on the task is an error, not a cancellation.
        // (Criteria outside the class's set — re-review LOW-3 — are no longer representable: every
        // check's criteria come from `checked()` or are zero.)
        let verdict = if check.verdict == VerificationVerdict::Cancelled && !head.cancellation {
            VerificationVerdict::Error
        } else {
            check.verdict
        };
        let object = match committing.root {
            Some(root) => root.clone(),
            None => store.publish(&check.evidence, uuid(staging)?, self.deadline)?,
        };
        let generation = store.record_verification_with_records(
            &expected(&head, begun)?,
            &Verification {
                verdict,
                subject: Sha256Digest::parse(subject).map_err(|_| Error::Identity)?,
                evidence: object.clone(),
                identity: EvidenceIdentity {
                    artifact_id: uuid(artifact)?,
                    media_type: CHECK_MEDIA_TYPE,
                    schema_id: &check.schema_id,
                },
                satisfied_criteria: Some(check.criteria),
                used_ms,
                cleanup_settled: check.cleanup_settled,
            },
            committing.records,
            committing.cited,
            uuid(event)?,
            self.deadline,
        )?;
        Ok(Committed {
            generation,
            event: event.clone(),
            object,
            artifact_id: artifact.clone(),
            verdict,
            criteria: check.criteria,
            schema_id: check.schema_id.clone(),
            reconciled: used_ms.is_some() && check.cleanup_settled,
            evidence: check.evidence.clone(),
        })
    }

    /// Record one of the runtime's own checks (a refused candidate, an empty window): no run, no
    /// records.
    fn record(&mut self, index: usize, check: &Check, subject: &str) -> Result<Committed, Error> {
        let begun = self.attempts.get(index).ok_or(Error::Identity)?;
        let ids: [String; 3] = fresh_ids(self.deadline)?;
        let committed = self.tasks.with_store(|store| {
            self.commit(store, begun, check, subject, &ids, Committing::NONE)
        })??;
        self.recorded(index, committed)
    }

    /// Record what the verifier observed (R15 round 2): inside one hold, capture every completed
    /// step, read the outputs and the subjects back, tear the job root down, build the four run
    /// records, publish them, derive the check by the one function and commit it all together.
    fn record_observed(
        &mut self,
        index: usize,
        observation: Observation<'_>,
    ) -> Result<Committed, Error> {
        let Observation {
            observed,
            window,
            job_root,
            applied,
            subject,
            ids,
        } = observation;
        let begun = self.attempts.get(index).ok_or(Error::Identity)?;
        let record_ids: [String; 8] = fresh_ids(self.deadline)?;
        let committed = self
            .tasks
            .with_store(|store| -> Result<Committed, Error> {
                let run = launched(observed.run)?;
                let teardown = window.teardown_until.min(self.deadline);
                let capture = capture_run(store, &run, teardown, self.deadline)?;
                let complete = capture.complete();
                let capture_refs = capture.refs();
                let Captures {
                    steps: captures,
                    outputs,
                    objects: cited_objects,
                    failed_at,
                } = capture;
                let Settled {
                    seed_verified,
                    subjects_verified,
                    protected_unchanged,
                    cleanup,
                    cleanup_record,
                    clock,
                } = self.settled(&run, teardown, job_root, applied, window, observed.observed)?;
                let outcome_record = if complete {
                    Some(RunOutcome::of(&run, &capture_refs).map_err(|_| Error::Identity)?)
                } else {
                    None
                };
                let readbacks = Readbacks::of(
                    uuid(&begun.id)?,
                    subjects_verified,
                    protected_unchanged,
                    &outputs,
                );
                let derived = checked(
                    OutcomeName::of(&run.outcome),
                    cleanup,
                    complete,
                    window.begun,
                    observed.observed,
                );
                // Publish every record under a fresh identity; the evidence cites each one.
                let published = publish_four(
                    store,
                    self.deadline,
                    &record_ids,
                    (&clock, outcome_record.as_ref(), &cleanup_record, &readbacks),
                )?;
                let records = store_records(&published)?;
                // The receipt, when a plan exists and the run was captured whole (R17 round 2,
                // decision 7 and R16r2.10): compose over a sink holding the shared and per-check
                // objects; a refused compose keeps the 3c evidence and names the refusal in it.
                let inputs = outcome_record.as_ref().map(|outcome| ComposeInputs {
                    clock: &clock,
                    outcome,
                    cleanup: &cleanup_record,
                    records: &published,
                    captures: &captures,
                    run: &run,
                    root_id: &ids[2],
                    seed_verified,
                    subjects_verified,
                    protected_unchanged,
                });
                let (check, cited, root) = self.evidence(
                    store,
                    begun,
                    inputs,
                    Fallback {
                        cited_objects,
                        published: &published,
                        run: &run,
                        failed_at: failed_at.as_deref(),
                        derived,
                        root_id: &ids[2],
                    },
                )?;
                self.commit(
                    store,
                    begun,
                    &check,
                    subject,
                    ids,
                    Committing {
                        records: &records,
                        cited: &cited,
                        root: root.as_ref(),
                    },
                )
            })??;
        self.recorded(index, committed)
    }

    /// The check's own teardown and readbacks: the subjects read back, the job root and everything
    /// the run retained under it removed (settled only within the teardown share), the cleanup
    /// and clock records derived.
    fn settled(
        &self,
        run: &Run,
        teardown: Instant,
        job_root: &Path,
        applied: &Snapshot,
        window: CheckWindow,
        observed: Instant,
    ) -> Result<Settled, Error> {
        let seed_verified = self.baseline.readback_source(teardown).is_ok();
        let subjects_verified = applied.readback_source(teardown).is_ok();
        let protected_unchanged = self.protected.readback_source(teardown).is_ok();
        let removed = fs::remove_dir_all(job_root).is_ok() || !job_root.exists();
        let retained_removed = removed && Instant::now() < teardown;
        let (cleanup, cleanup_record) = cleanup_of(run, retained_removed);
        let clock = clock_of(window, observed, run.decisive)?;
        Ok(Settled {
            seed_verified,
            subjects_verified,
            protected_unchanged,
            cleanup,
            cleanup_record,
            clock,
        })
    }

    /// The per-check plan in a hold of its own (R17 round 2, shape C): the result subject, the
    /// patch, the limits and cleanup contract, the identity — `prepare_u64` over them.
    fn plan_check(
        &self,
        index: usize,
        applied: &Snapshot,
        ids: &[String; 3],
        obligation_ids: &[String; 4],
        window: CheckWindow,
    ) -> Result<PlanOutcome, Error> {
        let begun = self.attempts.get(index).ok_or(Error::Identity)?;
        let parent_run = index
            .checked_sub(1)
            .and_then(|previous| self.attempts.get(previous))
            .and_then(|previous| previous.receipt_run.clone());
        let wall_ms = millis(
            window
                .teardown_until
                .saturating_duration_since(window.begun),
        );
        let inputs = plan::CheckInputs {
            shared: &self.shared,
            profile: self.profile,
            baseline: &self.baseline,
            applied,
            task: self.dispatch.task.as_str(),
            attempt: &begun.id,
            run: &ids[2],
            generation: &begun.generation,
            parent_run: parent_run.as_deref(),
            obligation_ids,
            wall_ms,
            // Within the cutoff (decision 8): the plan's time is the check's, and the teardown
            // share is the run's.
            deadline: window.until.min(self.deadline),
        };
        self.tasks
            .with_store(|store| -> Result<PlanOutcome, Error> {
                // A cancellation committed since the driver's last read (decision 8, N7), read in the
                // plan's own hold: no plan is published and no check runs.
                if self.current(store)?.cancellation {
                    return Ok(PlanOutcome::Cancelled);
                }
                let mut sink = Sink::new(store, self.deadline);
                Ok(match plan::check(&mut sink, &inputs) {
                    Ok(planned) => PlanOutcome::Planned(Box::new(planned)),
                    // The objects a refused plan left in the CAS, by id (N13): uncited, not backed
                    // up, listed so a reclaim door can find them.
                    Err(refusal) => {
                        PlanOutcome::Refused(refusal, sink.registered().keys().cloned().collect())
                    }
                })
            })?
    }

    /// The check's evidence: the receipt when a plan exists and the run was captured whole
    /// (`inputs` present), else the 3c evidence — see [`evidence_for`].
    fn evidence(
        &self,
        store: &Store,
        begun: &Begun,
        inputs: Option<ComposeInputs<'_>>,
        fallback: Fallback<'_>,
    ) -> Result<(Check, Vec<Object>, Option<Object>), Error> {
        let composed = begun
            .planned
            .as_ref()
            .zip(inputs)
            .map(|(planned, inputs)| self.compose_receipt(store, planned, inputs));
        evidence_for(composed, fallback)
    }

    /// Compose the receipt over a sink holding the shared and per-check objects (decision 7),
    /// with the readbacks derived as N1 states; returns the composed receipt and every object the
    /// compose sink holds, for the commit's citation.
    fn compose_receipt(
        &self,
        store: &Store,
        planned: &plan::Planned,
        inputs: ComposeInputs<'_>,
    ) -> Composition {
        let mut sink = Sink::new(store, self.deadline);
        for (reference, object) in self.shared.objects.iter().chain(&planned.objects) {
            sink.register(reference.clone(), object.clone())
                .map_err(|error| u64_receipt::Refusal::Publisher {
                    stage: "carry-over",
                    error: collector::Error::Sink(error),
                })?;
        }
        let facts = host::facts().map_err(ComposeRefusal::HostFacts)?;
        let unsettled = inputs
            .cleanup
            .obligations()
            .iter()
            .filter(|obligation| obligation.state != RecordSettlement::Settled)
            .count();
        let obligation_ids: [String; 4] =
            fresh_ids(self.deadline).map_err(ComposeRefusal::FreshIds)?;
        let readbacks = self.readbacks_of(
            inputs.run,
            inputs.seed_verified,
            inputs.subjects_verified,
            inputs.protected_unchanged,
        );
        let composed = u64_receipt::compose(
            &mut sink,
            &u64_receipt::Composing {
                prepared: &planned.prepared,
                clock: inputs.clock,
                outcome: inputs.outcome,
                cleanup: inputs.cleanup,
                records: inputs.records,
                captures: inputs.captures,
                evaluation: match &inputs.run.outcome {
                    WorkloadOutcome::Matched(evaluation)
                    | WorkloadOutcome::Mismatch(evaluation) => {
                        Some((evaluation.matched, evaluation.failed))
                    }
                    _ => None,
                },
                obligation_ids: &obligation_ids[..unsettled.min(4)],
                root_id: Some(inputs.root_id),
                host: &facts,
                readbacks,
            },
        )?;
        Ok((composed, sink.into_registered().into_values().collect()))
    }

    /// Each readback value from a named observation (R17 round 2, N1): the seed from the
    /// baseline's readback, the result from the applied's, fixtures/oracle/harness from the
    /// protected tree's, toolchain and launcher only when every stage LAUNCHED and the pinned
    /// files still hash to their pins after the run, the profile from a re-read of `profile.toml`.
    fn readbacks_of(
        &self,
        run: &Run,
        seed_verified: bool,
        subjects_verified: bool,
        protected_unchanged: bool,
    ) -> u64_receipt::Readbacks {
        let launched = every_stage_launched(
            run.steps
                .iter()
                .map(|step| matches!(step, Step::Completed { .. })),
        );
        let pins = launched.then(|| {
            [
                (&self.tools.compiler.host, self.tools.compiler.sha256),
                (&self.tools.shim.host, self.tools.shim.sha256),
                (&self.tools.bwrap, crate::worker::namespace::BWRAP_SHA256),
            ]
            .iter()
            .all(|(path, pin)| {
                crate::worker::namespace::sha256(path, self.deadline)
                    .is_ok_and(|digest| digest == *pin)
            })
        });
        let profile = super::class_profile::read(&self.profile.directory)
            .ok()
            .map(|read| read.digest == self.profile.digest);
        u64_receipt::Readbacks {
            seed: Some(seed_verified),
            result: Some(subjects_verified),
            fixtures: Some(protected_unchanged),
            oracle: Some(protected_unchanged),
            harness: Some(protected_unchanged),
            launcher: pins,
            toolchain: pins,
            profile,
        }
    }

    /// The runtime's own bookkeeping after a verification committed.
    fn recorded(&mut self, index: usize, committed: Committed) -> Result<Committed, Error> {
        self.written(&committed.generation, committed.event.clone())?;
        if let Some(begun) = self.attempts.get_mut(index) {
            begun.verified = true;
            begun.check_settled = committed.reconciled;
            begun.receipt_run = (committed.schema_id == ReceiptV1::SCHEMA_ID)
                .then(|| committed.artifact_id.clone());
        }
        self.previous = Some(Previous {
            verdict: committed.verdict,
            criteria: committed.criteria,
            evidence: committed.evidence.clone(),
            schema_id: committed.schema_id.clone(),
        });
        Ok(committed)
    }

    /// What the driver is told of a committed verification.
    fn checked(committed: Committed, subject: String) -> DriverChecked<Evidence> {
        // An unknown cost or unsettled cleanup is an obligation the stop must keep (review H1).
        if !committed.reconciled {
            return DriverChecked::Unsettled;
        }
        match committed.verdict {
            VerificationVerdict::Passed => DriverChecked::Passed {
                evidence: Evidence {
                    object: committed.object,
                    subject,
                    artifact_id: committed.artifact_id,
                    schema_id: committed.schema_id,
                },
                criteria: committed.criteria,
            },
            VerificationVerdict::Failed => DriverChecked::Failed {
                criteria: committed.criteria,
            },
            VerificationVerdict::Invalid => DriverChecked::Invalid,
            VerificationVerdict::Error => DriverChecked::Error,
            VerificationVerdict::Timeout => DriverChecked::Timeout,
            // The task was cancelled (`commit` turned any other `Cancelled` into `Error`): nothing
            // was satisfied, and the driver's next cancellation read stops it as cancelled.
            VerificationVerdict::Cancelled => DriverChecked::Failed { criteria: 0 },
        }
    }

    /// The stop's body: kind, reason, the attempt count — and, when the last attempt ended in a
    /// provider failure, an OPTIONAL additive `worker` field naming it (R18 A3, decision R18.3: an
    /// additive optional field does not revise `hee3.task-stop/1`; readers tolerate its absence).
    fn stop_body(&self, name: &str) -> Result<Vec<u8>, Error> {
        let mut body = serde_json::json!({
            "kind": TASK_STOP_SCHEMA, "reason": name, "attempts": self.attempts.len(),
        });
        if let Some(failure) = self.attempts.last().and_then(|begun| begun.provider) {
            body["worker"] = serde_json::json!({
                "provider": crate::worker::native::PROVIDER,
                "error": failure.error.name(),
                "state": failure.state.name(),
                "retained": failure.retained,
            });
        }
        serde_json::to_vec(&body).map_err(|_| Error::Identity)
    }

    /// Stop the task, deciding and writing in one hold (review M2). (b) An attempt whose work or
    /// check did not settle stops nothing: its obligations and reservations stay (R1.4). (a) A
    /// cancellation with no check of the last attempt records the check as not started, at no
    /// cost, before the stop. (c) `finish_preparation` exactly when no attempt row exists.
    fn stop_task(&mut self, reason: StopReason) -> Result<bool, Error> {
        if self
            .attempts
            .last()
            .is_some_and(|begun| !begun.settled || !begun.check_settled)
        {
            return Ok(false);
        }
        let name = stop_name(reason);
        let stop_bytes = self.stop_body(name)?;
        let idle_bytes = serde_json::to_vec(&serde_json::json!({
            "kind": "verification_not_started", "durable_cancellation": true,
        }))
        .map_err(|_| Error::Identity)?;
        let ids: [String; 6] = fresh_ids(self.deadline)?;
        let reason = Name::new(name).map_err(|_| Error::Identity)?;
        let last = self.attempts.last();
        let idle_subject = match last {
            Some(begun) => Some(match (&begun.applied, &begun.refused) {
                (Some(applied), _) => applied.content_digest().ok_or(Error::Identity)?,
                (None, Some((candidate, _))) => digest(candidate),
                (None, None) => self.digests[0].clone(),
            }),
            None => None,
        };
        let stopped = self.tasks.with_store(|store| -> Result<_, Error> {
            let head = self.current(store)?;
            let mut generation = head.generation.clone();
            if head.cancellation
                && let (Some(begun), Some(subject)) = (last, &idle_subject)
                && !begun.verified
            {
                let object = store.publish(&idle_bytes, uuid(&ids[0])?, self.deadline)?;
                generation = store.record_verification(
                    &Expected {
                        task: self.dispatch.task,
                        task_generation: parse_generation(&generation)?,
                        attempt: uuid(&begun.id)?,
                        attempt_generation: parse_generation(&begun.generation)?,
                    },
                    &Verification {
                        verdict: VerificationVerdict::Cancelled,
                        subject: Sha256Digest::parse(subject).map_err(|_| Error::Identity)?,
                        evidence: object,
                        identity: EvidenceIdentity {
                            artifact_id: uuid(&ids[4])?,
                            media_type: CHECK_MEDIA_TYPE,
                            schema_id: IDLE_VERIFICATION_SCHEMA,
                        },
                        satisfied_criteria: None,
                        used_ms: Some(0),
                        cleanup_settled: true,
                    },
                    uuid(&ids[1])?,
                    self.deadline,
                )?;
            }
            let object = store.publish(&stop_bytes, uuid(&ids[2])?, self.deadline)?;
            let stop = Stop {
                task: self.dispatch.task,
                generation: parse_generation(&generation)?,
                reason: &reason,
                evidence: &object,
                identity: EvidenceIdentity {
                    artifact_id: uuid(&ids[5])?,
                    media_type: CHECK_MEDIA_TYPE,
                    schema_id: TASK_STOP_SCHEMA,
                },
                event: uuid(&ids[3])?,
            };
            Ok(if last.is_none() {
                store.finish_preparation(
                    self.dispatch.principal,
                    stop,
                    Settlement {
                        effect: Effect::None,
                        used_ms: Some(preparation_charge(
                            millis(self.origin.elapsed()),
                            head.reserved_work_ms,
                        )),
                        cleanup_settled: true,
                        ready_to_verify: false,
                    },
                    self.deadline,
                )?
            } else {
                store.finish_unaccepted(self.dispatch.principal, stop, self.deadline)?
            })
        })??;
        let [_, _, _, event, _, _] = ids;
        self.written(&stopped.generation, event)?;
        Ok(true)
    }
}

impl<C: CandidateSource, V: Verifier> driver::Runtime for StoreRuntime<'_, C, V> {
    type Error = Fault;
    type Attempt = Attempt;
    type Evidence = Evidence;

    fn elapsed(&self) -> Duration {
        self.origin.elapsed()
    }

    fn cancellation_requested(&mut self) -> Result<bool, Self::Error> {
        let head = self.tasks.with_store(|store| self.current(store))??;
        Ok(head.cancellation)
    }

    fn begin(&mut self, ordinal: Generation) -> Result<Self::Attempt, Self::Error> {
        let (attempt, event, session) = (
            fresh(self.deadline)?,
            fresh(self.deadline)?,
            fresh(self.deadline)?,
        );
        let [baseline, protected, profile] = &self.digests;
        let binding = Binding {
            baseline: Sha256Digest::parse(baseline).map_err(|_| Error::Identity)?,
            protected: Sha256Digest::parse(protected).map_err(|_| Error::Identity)?,
            profile: Sha256Digest::parse(profile).map_err(|_| Error::Identity)?,
        };
        // The first attempt's settle charges the preparation too (B14a-R2.5).
        let charged_from = if self.attempts.is_empty() {
            self.origin
        } else {
            Instant::now()
        };
        let begun = self.tasks.with_store(|store| -> Result<_, Error> {
            let head = self.current(store)?;
            // A cancellation committed after the driver's last read is a stop, not an error.
            if head.cancellation {
                return Ok(Err(StopReason::Cancelled));
            }
            // The work window re-reads the reservation at every begin (R1.8); a spent one cannot
            // begin, and the task stops by policy rather than failing with a zero lease.
            let window = head
                .reserved_work_ms
                .saturating_sub(self.dispatch.teardown_ms);
            let work_until = self
                .deadline
                .min(charged_from + Duration::from_millis(window));
            let lease_ms = millis(work_until.saturating_duration_since(Instant::now()));
            if lease_ms == 0 {
                return Ok(Err(StopReason::Policy(LoopRefusal::Deadline)));
            }
            let workspace = head.workspace_id.as_deref().ok_or(Error::Identity)?;
            let roster = store.begin_bound_attempt(
                RosterStart {
                    principal: self.dispatch.principal,
                    task: self.dispatch.task,
                    expected: parse_generation(&head.generation)?,
                    attempt: uuid(&attempt)?,
                    event: uuid(&event)?,
                    agent_record_id: self.dispatch.agent_record_id,
                    session: uuid(&session)?,
                    workspace: uuid(workspace)?,
                    selections: self.dispatch.selections,
                    lease_ms,
                },
                &binding,
                self.deadline,
            )?;
            Ok(Ok((roster, work_until)))
        })??;
        let (roster, work_until) = begun.map_err(Fault::Stop)?;
        if roster.attempt.generation != ordinal.to_string() {
            return Err(Error::Identity.into());
        }
        self.written(&roster.attempt.task_generation, event)?;
        self.attempts.push(Begun {
            id: roster.attempt.id,
            generation: roster.attempt.generation,
            charged_from,
            work_until,
            applied: None,
            refused: None,
            settled: false,
            verified: false,
            check_settled: true,
            planned: None,
            receipt_run: None,
            provider: None,
        });
        Ok(Attempt {
            index: self.attempts.len() - 1,
        })
    }

    fn execute(&mut self, attempt: &Self::Attempt) -> Result<Work, Self::Error> {
        let index = attempt.index;
        let (id, work_until) = {
            let begun = self.begun(attempt)?;
            (begun.id.clone(), begun.work_until)
        };
        let generation = self.begun(attempt)?.generation.clone();
        let [invocation]: [String; 1] = fresh_ids(self.deadline)?;
        let ask = Ask {
            task: self.dispatch.task,
            attempt: uuid(&id)?,
            generation: parse_generation(&generation)?,
            invocation: uuid(&invocation)?,
            recipe: Sha256Digest::parse(&self.profile.digest).map_err(|_| Error::Identity)?,
            workspace: Sha256Digest::parse(&self.digests[0]).map_err(|_| Error::Identity)?,
            origin: self.origin,
            work_until,
            cancelled: &self.cancelled,
            previous: self.previous.as_ref(),
        };
        match self.source.next(&ask) {
            // Exhaustion after begin is truthful (B14a-R2.3): the attempt is not ready to verify.
            Candidate::Exhausted => {
                self.settle(index, true, false)?;
                Ok(if self.begun(attempt)?.settled {
                    Work::Failed
                } else {
                    Work::Unsettled
                })
            }
            Candidate::Replacement(bytes) => {
                let applied = repair::apply_candidate(
                    &self.baseline,
                    U64_EDITABLE,
                    &bytes,
                    U64_BOUNDS,
                    self.dispatch.attempts,
                    &id,
                    work_until,
                );
                let cleanup_settled = match applied {
                    Ok(snapshot) => {
                        if let Some(begun) = self.attempts.get_mut(index) {
                            begun.applied = Some(snapshot);
                        }
                        true
                    }
                    Err(failure) => {
                        let removed = remove(
                            &failure,
                            teardown_deadline(work_until, self.dispatch.teardown_ms, self.deadline),
                        );
                        if let Some(begun) = self.attempts.get_mut(index) {
                            begun.refused = Some((bytes, refusal_name(failure.error)));
                        }
                        removed
                    }
                };
                self.settle(index, cleanup_settled, true)?;
                Ok(if self.begun(attempt)?.settled {
                    Work::ReadyForCheck
                } else {
                    Work::Unsettled
                })
            }
            // The source's text refused by the class grammar (B14a-4, A5): nothing was
            // materialised, so the cleanup is settled; the check records it as a refused candidate
            // under the refusal's name, through the one path a repair refusal takes.
            Candidate::Refused { refusal, text } => {
                if let Some(begun) = self.attempts.get_mut(index) {
                    begun.refused = Some((text, refusal.name()));
                }
                self.settle(index, true, true)?;
                Ok(if self.begun(attempt)?.settled {
                    Work::ReadyForCheck
                } else {
                    Work::Unsettled
                })
            }
            // The provider failed (A3): the attempt is not ready to verify and nothing is asked
            // again — a lost response is a readback, never a redispatch; the stop names it.
            Candidate::Provider {
                error,
                state,
                cleanup_settled,
                retained,
            } => {
                if let Some(begun) = self.attempts.get_mut(index) {
                    begun.provider = Some(ProviderFailure {
                        error,
                        state,
                        retained,
                    });
                }
                self.settle(index, cleanup_settled, false)?;
                Ok(if self.begun(attempt)?.settled {
                    Work::Failed
                } else {
                    Work::Unsettled
                })
            }
        }
    }

    fn verify(
        &mut self,
        attempt: &Self::Attempt,
    ) -> Result<DriverChecked<Self::Evidence>, Self::Error> {
        let index = attempt.index;
        let begun = self.attempts.get(index).ok_or(Error::Identity)?;
        let (check, subject) = if let Some((candidate, refusal)) = &begun.refused {
            // The runtime's own class check: no verifier call, nothing satisfied (B14a-R2.4).
            let subject = digest(candidate);
            let evidence = serde_json::to_vec(&serde_json::json!({
                "kind": "refused_candidate", "refusal": refusal, "candidate_sha256": subject,
            }))
            .map_err(|_| Error::Identity)?;
            let check = Check {
                verdict: VerificationVerdict::Failed,
                criteria: 0,
                evidence,
                schema_id: REFUSED_CANDIDATE_SCHEMA.to_owned(),
                used_ms: Some(0),
                cleanup_settled: true,
            };
            (check, subject)
        } else {
            let applied = begun.applied.as_ref().ok_or(Error::Identity)?;
            let subject = applied.content_digest().ok_or(Error::Identity)?;
            // The window comes from the reservation the ledger holds now, read in a hold of its
            // own so a second writer is caught before a check is spent (R14.3, R14.4).
            let reserved_verify_ms = self.tasks.with_store(|store| -> Result<u64, Error> {
                Ok(self.current(store)?.reserved_verify_ms)
            })??;
            if let Some(window) = check_window(
                Instant::now(),
                unix_ms_now()?,
                reserved_verify_ms,
                self.deadline,
            ) {
                // The verifier observes inside a job root of its own, a SIBLING of the applied
                // snapshot's directory (never under it: the workload refuses a root inside its
                // subject, and creating one would change the subject); the runtime then
                // records what was observed (R15 round 2).
                let id = begun.id.clone();
                let applied = applied.clone();
                // The per-check plan (R17 round 2, shape C), in a hold of its own, before the
                // producer runs; its ids are the verification's, the run id the root's.
                let ids: [String; 3] = fresh_ids(self.deadline)?;
                let obligation_ids: [String; 4] = fresh_ids(self.deadline)?;
                let planned = self.plan_check(index, &applied, &ids, &obligation_ids, window)?;
                let planned = match planned {
                    PlanOutcome::Planned(planned) => *planned,
                    PlanOutcome::Refused(refusal, orphans) => {
                        // A refused plan is a check that never ran (N2): recorded under its own
                        // schema, `Error`, at the cost the plan spent, naming its orphans.
                        let check =
                            plan_refused_check(&refusal, &orphans, millis(window.begun.elapsed()))?;
                        let recorded = self.record(index, &check, &subject)?;
                        return Ok(Self::checked(recorded, subject));
                    }
                    // Nothing is recorded here: the driver's next cancellation read stops the
                    // task, and `stop_task` records the check as not started (B14a-R1.4a).
                    PlanOutcome::Cancelled => return Ok(DriverChecked::Failed { criteria: 0 }),
                };
                if let Some(begun) = self.attempts.get_mut(index) {
                    begun.planned = Some(planned);
                }
                let job_root = self.dispatch.attempts.join(format!("{id}.check"));
                fs::DirBuilder::new()
                    .mode(0o700)
                    .create(&job_root)
                    .map_err(|_| Error::Identity)?;
                let observed = self.verifier.check(CheckPlan {
                    subject: &applied,
                    protected: &self.protected,
                    job_root: &job_root,
                    tools: &self.tools,
                    window,
                    cancelled: &self.cancelled,
                });
                let recorded = self.record_observed(
                    index,
                    Observation {
                        observed,
                        window,
                        job_root: &job_root,
                        applied: &applied,
                        subject: &subject,
                        ids: &ids,
                    },
                )?;
                return Ok(Self::checked(recorded, subject));
            }
            let check = Check {
                verdict: VerificationVerdict::Timeout,
                criteria: 0,
                evidence: serde_json::to_vec(&serde_json::json!({
                    "kind": "check_window_empty",
                    "reserved_verify_ms": reserved_verify_ms,
                    "teardown_ms": millis(CHECK_TEARDOWN),
                }))
                .map_err(|_| Error::Identity)?,
                schema_id: CHECK_WINDOW_EMPTY_SCHEMA.to_owned(),
                used_ms: Some(0),
                cleanup_settled: true,
            };
            (check, subject)
        };
        let recorded = self.record(index, &check, &subject)?;
        Ok(Self::checked(recorded, subject))
    }

    fn accept(
        &mut self,
        attempt: &Self::Attempt,
        evidence: Self::Evidence,
    ) -> Result<Acceptance, Self::Error> {
        let begun = self.begun(attempt)?;
        let event = fresh(self.deadline)?;
        // Prepare and accept share one hold (B14a-R1.1); a cancellation committed first is named
        // by the store before the compare-and-set, and answered with no re-read (R1.2).
        let answer = self
            .tasks
            .with_store(|store| -> Result<Option<String>, Error> {
                let head = self.current(store)?;
                let expected = expected(&head, begun)?;
                // The accepted object is the verification's receipt, named as the verification
                // named it (B09b) — and only once the run records it cites are read back against
                // the ledger's commitment (DS2 §2.6, R15.7): the identified object comes from that
                // comparison, so acceptance cannot proceed without it.
                let committed = store.committed_check(
                    self.dispatch.principal,
                    uuid(&begun.id)?,
                    self.deadline,
                )?;
                let lookup = |kind| {
                    committed.record(kind).map(|record| {
                        (
                            record.artifact_id(),
                            record.object().digest(),
                            record.object().size(),
                        )
                    })
                };
                // A receipt is walked as a graph over the ledger's objects and its run-record
                // artifact rows compared to the commitment (R16r2.6, R17 2c-iii); the runtime's
                // own JSON shapes keep their field walk.
                let identity = if evidence.schema_id == ReceiptV1::SCHEMA_ID {
                    cited_receipt(store, self.deadline, &evidence, lookup)?
                } else {
                    let bytes = store.read_object(&evidence.object, self.deadline)?;
                    cited(&evidence.schema_id, &evidence.artifact_id, &bytes, lookup)?
                };
                let identified = Identified {
                    object: evidence.object.clone(),
                    identity,
                };
                let result = store
                    .prepare_verified_acceptance(
                        &expected,
                        uuid(&event)?,
                        Sha256Digest::parse(&evidence.subject).map_err(|_| Error::Identity)?,
                        &evidence.object,
                        std::slice::from_ref(&identified),
                        self.deadline,
                    )
                    .and_then(|published| store.accept(&published, 0, self.deadline));
                match result {
                    Ok(_) => Ok(Some(
                        store
                            .get(self.dispatch.principal, self.dispatch.task, self.deadline)?
                            .generation,
                    )),
                    Err(store::Error::Cancelled) => Ok(None),
                    Err(error) => Err(error.into()),
                }
            })??;
        match answer {
            Some(generation) => {
                self.written(&generation, event)?;
                Ok(Acceptance::Accepted)
            }
            None => Ok(Acceptance::CancelledFirst),
        }
    }

    fn stop(&mut self, reason: StopReason) -> Result<bool, Self::Error> {
        Ok(self.stop_task(reason)?)
    }
}

const fn stop_name(reason: StopReason) -> &'static str {
    match reason {
        StopReason::Policy(_) => "task_policy_stop",
        StopReason::WorkerFailed => "worker_failed",
        StopReason::InvalidCheck => "invalid_check",
        StopReason::VerifierError => "verifier_error",
        StopReason::VerifierTimeout => "verifier_timeout",
        StopReason::Cancelled => "durable_cancellation",
        StopReason::Unsettled => "unsettled_obligation",
    }
}

const fn refusal_name(error: repair::Error) -> &'static str {
    match error {
        repair::Error::Path => "candidate_path",
        repair::Error::Bound => "candidate_bound",
        repair::Error::Deadline => "candidate_deadline",
        repair::Error::Workspace(_) => "candidate_workspace",
        repair::Error::Changed => "candidate_changed",
        repair::Error::Io => "candidate_io",
        repair::Error::Encoding => "candidate_encoding",
        repair::Error::Changes => "candidate_changes",
    }
}

/// What a stop before any attempt charges: the observed preparation time, never more than the work
/// reservation holds (review MEDIUM-1). The store refuses a charge past the reservation, so an
/// honest refusal of an empty or tiny reservation would otherwise strand the task; the observed
/// figure travels in the stop's evidence.
const fn preparation_charge(observed_ms: u64, reserved_ms: u64) -> u64 {
    if observed_ms < reserved_ms {
        observed_ms
    } else {
        reserved_ms
    }
}

/// The deadline a refused candidate's removal is given: its work window plus the teardown share
/// held back for exactly this, never past the task's deadline (review MEDIUM-2).
fn teardown_deadline(work_until: Instant, teardown_ms: u64, task_deadline: Instant) -> Instant {
    task_deadline.min(work_until + Duration::from_millis(teardown_ms))
}

/// Every criterion bit the class declares.
pub(crate) fn declared_criteria() -> u64 {
    match U64_CRITERIA.len() {
        64.. => u64::MAX,
        count => (1_u64 << count) - 1,
    }
}

fn expected<'b>(head: &'b TaskHead, begun: &'b Begun) -> Result<Expected<'b>, Error> {
    Ok(Expected {
        task: uuid(&head.id)?,
        task_generation: parse_generation(&head.generation)?,
        attempt: uuid(&begun.id)?,
        attempt_generation: parse_generation(&begun.generation)?,
    })
}

/// `N` fresh identities, or the first entropy refusal.
fn fresh_ids<const N: usize>(deadline: Instant) -> Result<[String; N], Error> {
    let mut ids = Vec::with_capacity(N);
    for _ in 0..N {
        ids.push(fresh(deadline)?);
    }
    ids.try_into().map_err(|_| Error::Entropy)
}

fn fresh(deadline: Instant) -> Result<String, Error> {
    fresh_id(deadline)
        .map(|id| id.as_str().to_owned())
        .map_err(|_| Error::Entropy)
}

fn uuid(value: &str) -> Result<UuidV4<'_>, Error> {
    UuidV4::parse(value).map_err(|_| Error::Identity)
}

fn parse_generation(value: &str) -> Result<Generation, Error> {
    value.parse().map_err(|_| Error::Identity)
}

fn number(value: &str) -> Result<u64, Error> {
    value.parse().map_err(|_| Error::Identity)
}

/// What the runtime captured of a run: one reference per step (`None` for a refused step), the
/// outputs read back, and whether every publication succeeded.
struct Captures {
    /// One capture per step, `None` for a refused step: the composer's producer facts and the
    /// run-outcome record's references both come from here.
    steps: Vec<Option<capture::Captured>>,
    outputs: Vec<OutputReadback>,
    /// Every object the captures and readbacks published: the ledger registers them with the
    /// verification, so the inventory bound counts them and a backup copies them.
    objects: Vec<Object>,
    /// Which capture failed, if one did (the step's label, or `"output"` for a readback identity).
    failed_at: Option<String>,
}

impl Captures {
    /// Whether every step and output was captured.
    fn complete(&self) -> bool {
        self.failed_at.is_none()
    }

    /// Each step's producer reference, for the run-outcome record.
    fn refs(&self) -> Vec<Option<Ref>> {
        self.steps
            .iter()
            .map(|step| {
                step.as_ref()
                    .map(|captured| captured.producer_ref.as_ref().clone())
            })
            .collect()
    }
}

/// Capture every completed step through the evidence sink and read every output back against
/// what the run retained, publishing each output's identity; a publication that fails leaves the
/// capture incomplete (the check then earns nothing, R15.3), never a fault.
fn capture_run(
    store: &Store,
    run: &Run,
    teardown: Instant,
    deadline: Instant,
) -> Result<Captures, Error> {
    let mut sink = Sink::new(store, deadline);
    let objects = |sink: Sink<'_>| {
        sink.into_registered()
            .into_values()
            .map(|(_, object)| object)
            .collect()
    };
    let mut steps = Vec::with_capacity(run.steps.len());
    for step in &run.steps {
        let Step::Completed { report, label } = step else {
            steps.push(None);
            continue;
        };
        let Ok(captured) = capture::capture(&mut sink, report) else {
            return Ok(Captures {
                steps,
                outputs: Vec::new(),
                objects: objects(sink),
                failed_at: Some((*label).to_owned()),
            });
        };
        steps.push(Some(captured));
    }
    let mut outputs = Vec::with_capacity(run.outputs.len());
    for output in &run.outputs {
        let matched = output.readback_source(teardown).is_ok();
        let bytes = serde_json::to_vec(&serde_json::json!({
            "kind": "output_readback",
            "root": output.root().display().to_string(),
            "content_digest": output.content_digest(),
        }))
        .map_err(|_| Error::Identity)?;
        let Ok(payload) = sink.payload(&bytes, CHECK_MEDIA_TYPE) else {
            return Ok(Captures {
                steps,
                outputs,
                objects: objects(sink),
                failed_at: Some("output".to_owned()),
            });
        };
        outputs.push(OutputReadback {
            output: payload.into_inner(),
            matched,
        });
    }
    Ok(Captures {
        steps,
        outputs,
        objects: objects(sink),
        failed_at: None,
    })
}

/// The run's cleanup as the runtime observed it, and the record of it: the process and scratch
/// predicates the workload reported, the retained paths the runtime removed, and the aggregate —
/// observed by the dispatcher that owns it (B14b), `Unknown` here.
fn cleanup_of(run: &Run, retained_removed: bool) -> (Cleanup, RunCleanup) {
    let cleanup = Cleanup {
        processes_settled: run.process_cleanup_complete,
        scratch_released: run.scratch_released,
        retained_removed,
    };
    let state = |settled: bool| {
        if settled {
            RecordSettlement::Settled
        } else {
            RecordSettlement::Pending
        }
    };
    let obligation = |id: &str, state: RecordSettlement| ObligationRecord {
        id: id.to_owned(),
        state,
    };
    let obligations = [
        obligation("process", state(run.process_cleanup_complete)),
        obligation("scratch", state(run.scratch_released)),
        obligation("retained_paths", state(retained_removed)),
        obligation("resources", RecordSettlement::Unknown),
    ];
    let record = RunCleanup::of(state(cleanup.settled()), &obligations, &[]);
    (cleanup, record)
}

/// The published records as the ledger commits them beside the verification.
fn store_records(
    published: &[(RunRecordKind, String, Object)],
) -> Result<Vec<StoreRunRecord<'_>>, Error> {
    published
        .iter()
        .map(|(kind, id, object)| {
            Ok(StoreRunRecord {
                kind: *kind,
                artifact_id: uuid(id)?,
                object,
            })
        })
        .collect()
}

/// The four run records encoded and published under the eight minted ids (the outcome record
/// absent when the run was not captured whole).
fn publish_four(
    store: &Store,
    deadline: Instant,
    record_ids: &[String; 8],
    records: (&RunClock, Option<&RunOutcome>, &RunCleanup, &Readbacks),
) -> Result<Vec<(RunRecordKind, String, Object)>, Error> {
    let (clock, outcome, cleanup, readbacks) = records;
    let encode =
        |bytes: Result<Vec<u8>, super::run_records::Refusal>| bytes.map_err(|_| Error::Identity);
    publish_records(
        store,
        deadline,
        record_ids,
        [
            Some(encode(clock.to_bytes())?),
            outcome
                .map(|record| encode(record.to_bytes()))
                .transpose()?,
            Some(encode(cleanup.to_bytes())?),
            Some(encode(readbacks.to_bytes())?),
        ],
    )
}

/// Publish the encoded records present, each under its fresh artifact id (`ids[0..4]`) through
/// its fresh staging id (`ids[4..8]`), in kind order.
fn publish_records(
    store: &Store,
    deadline: Instant,
    ids: &[String; 8],
    encoded: [Option<Vec<u8>>; 4],
) -> Result<Vec<(RunRecordKind, String, Object)>, Error> {
    let kinds = [
        RunRecordKind::RunClock,
        RunRecordKind::RunOutcome,
        RunRecordKind::RunCleanup,
        RunRecordKind::Readbacks,
    ];
    let mut published = Vec::with_capacity(4);
    for (slot, (kind, bytes)) in kinds.into_iter().zip(encoded).enumerate() {
        let Some(bytes) = bytes else {
            continue;
        };
        let object = store.publish(&bytes, uuid(&ids[slot + 4])?, deadline)?;
        published.push((kind, ids[slot].clone(), object));
    }
    Ok(published)
}

/// The run to record: a refusal to launch is still a run with its four records (R15.4), except a
/// `Layout` refusal, which is the runtime's own fault (the job root or scopes it built).
fn launched(run: Result<Run, workload::Error>) -> Result<Run, Error> {
    match run {
        Ok(run) => Ok(run),
        Err(workload::Error::Layout) => Err(Error::Identity),
        Err(workload::Error::Deadline) => Ok(Run::unlaunched(WorkloadOutcome::Timeout)),
        // The refusal's cause IS a failed subject readback: the record must not say "unchanged".
        Err(workload::Error::Subject(_)) => {
            let mut run = Run::unlaunched(WorkloadOutcome::InvalidSubject);
            run.subjects_unchanged = false;
            Ok(run)
        }
        Err(workload::Error::Oracle | workload::Error::Io) => {
            Ok(Run::unlaunched(WorkloadOutcome::SetupFailed))
        }
    }
}

/// The check's clock record over its window: the intent follows the deadline being reached, never
/// the outcome's name (R15.5); `deadline` carries the observation's cleanup deadline (the second
/// pinned-field amendment, recorded).
fn clock_of(
    window: CheckWindow,
    observed: Instant,
    decisive: Option<Instant>,
) -> Result<RunClock, Error> {
    let intents = Intents {
        timeout: (observed >= window.until).then_some(window.until),
        cancellation: None,
    };
    RunClock::observe(
        &RuntimeClock {
            origin: window.begun,
            origin_unix_ms: window.begun_unix_ms,
            work_until: window.until,
            deadline: window.teardown_until,
        },
        intents,
        decisive,
        observed,
    )
    .map_err(|_| Error::Identity)
}

/// The check's evidence: the composed receipt with its root object and every carried object cited
/// when compose succeeded; the 3c evidence naming the refusal when it did not; the 3c evidence as
/// it was when no receipt was attempted.
fn evidence_for(
    composed: Option<Composition>,
    fallback: Fallback<'_>,
) -> Result<(Check, Vec<Object>, Option<Object>), Error> {
    let Fallback {
        cited_objects,
        published,
        run,
        failed_at,
        derived,
        root_id,
    } = fallback;
    match composed {
        Some(Ok((composed, objects))) => {
            let mut cited = cited_objects;
            cited.extend(objects.iter().map(|(_, object)| object.clone()));
            let root = objects
                .iter()
                .find(|(reference, _)| reference.artifact_id.as_str() == root_id)
                .map(|(_, object)| object.clone())
                .ok_or(Error::Identity)?;
            Ok((
                Check {
                    verdict: derived.verdict,
                    criteria: derived.criteria,
                    evidence: composed.bytes,
                    schema_id: ReceiptV1::SCHEMA_ID.to_owned(),
                    used_ms: Some(derived.used_ms),
                    cleanup_settled: derived.cleanup_settled,
                },
                cited,
                Some(root),
            ))
        }
        Some(Err(refusal)) => Ok((
            check_of(
                published,
                OutcomeName::of(&run.outcome),
                failed_at,
                derived,
                Some(&refusal.describe()),
            )?,
            cited_objects,
            None,
        )),
        None => Ok((
            check_of(
                published,
                OutcomeName::of(&run.outcome),
                failed_at,
                derived,
                None,
            )?,
            cited_objects,
            None,
        )),
    }
}

/// The check the runtime records for an observed run: the derived verdict, and evidence under
/// [`U64_CHECK_SCHEMA`] citing every published record by kind (R15.7).
fn check_of(
    published: &[(RunRecordKind, String, Object)],
    outcome: OutcomeName,
    capture_failed_at: Option<&str>,
    derived: super::live_verifier::Checked,
    compose_refused: Option<&str>,
) -> Result<Check, Error> {
    let cited: serde_json::Map<String, serde_json::Value> = published
        .iter()
        .map(|(kind, id, object)| {
            (
                kind.name().to_owned(),
                serde_json::json!({
                    "artifact_id": id,
                    "sha256": object.digest(),
                    "byte_length": object.size(),
                }),
            )
        })
        .collect();
    Ok(Check {
        verdict: derived.verdict,
        criteria: derived.criteria,
        evidence: serde_json::to_vec(&serde_json::json!({
            "kind": "u64_check",
            "outcome": outcome,
            "captured": capture_failed_at.is_none(),
            "capture_failed_at": capture_failed_at,
            "compose_refused": compose_refused,
            "records": cited,
        }))
        .map_err(|_| Error::Identity)?,
        schema_id: U64_CHECK_SCHEMA.to_owned(),
        used_ms: Some(derived.used_ms),
        cleanup_settled: derived.cleanup_settled,
    })
}

/// The wall clock now, in unix milliseconds — read once, beside the monotonic origin it pairs with.
fn unix_ms_now() -> Result<u64, Error> {
    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::Identity)?;
    u64::try_from(since.as_millis()).map_err(|_| Error::Identity)
}

/// The accepted object's identity, once the run records its evidence cites equal the ledger's
/// commitment (DS2 §2.6, R15.7): the evidence must be the runtime's own (`hee3.u64-check/1`), and
/// for every kind, what it cites and what `committed(kind)` holds must both be absent or equal on
/// artifact id, digest and size — with a clock always committed. Pure over its inputs (F95);
/// `accept` can build the accepted object only from what this returns.
fn cited<'a>(
    schema_id: &'a str,
    artifact_id: &'a str,
    bytes: &[u8],
    committed: impl Fn(RunRecordKind) -> Option<(&'a str, &'a str, u64)>,
) -> Result<EvidenceIdentity<'a>, Error> {
    if schema_id != U64_CHECK_SCHEMA {
        return Err(Error::Identity);
    }
    let body: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| Error::Identity)?;
    if body.get("kind").and_then(serde_json::Value::as_str) != Some("u64_check") {
        return Err(Error::Identity);
    }
    let records = body
        .get("records")
        .and_then(serde_json::Value::as_object)
        .ok_or(Error::Identity)?;
    let known = |key: &String| RunRecordKind::ALL.iter().any(|kind| kind.name() == key);
    if !records.keys().all(known) || committed(RunRecordKind::RunClock).is_none() {
        return Err(Error::Identity);
    }
    for kind in RunRecordKind::ALL {
        let cited = records.get(kind.name()).map(|entry| {
            (
                entry.get("artifact_id").and_then(serde_json::Value::as_str),
                entry.get("sha256").and_then(serde_json::Value::as_str),
                entry.get("byte_length").and_then(serde_json::Value::as_u64),
            )
        });
        match (cited, committed(kind)) {
            (None, None) => {}
            (
                Some((Some(id), Some(digest), Some(size))),
                Some((held_id, held_digest, held_size)),
            ) if id == held_id && digest == held_digest && size == held_size => {}
            _ => return Err(Error::Identity),
        }
    }
    Ok(EvidenceIdentity {
        artifact_id: uuid(artifact_id)?,
        media_type: CHECK_MEDIA_TYPE,
        schema_id,
    })
}

/// The identity a receipt is accepted under: its graph resolved from the ledger's objects, its
/// `run_id` the artifact id it is committed under (R17 round 2, decision 3), every
/// `run_record:<kind>` artifact row equal to the committed record of that kind and every committed
/// record cited, the clock present — the receipt form of [`cited`].
fn cited_receipt<'a>(
    store: &Store,
    deadline: Instant,
    evidence: &'a Evidence,
    committed: impl Fn(RunRecordKind) -> Option<(&'a str, &'a str, u64)>,
) -> Result<EvidenceIdentity<'a>, Error> {
    use crate::contracts::receipt::{ArtifactV1, Id, Name, Sha, decode};
    let root = Ref {
        artifact_id: Id::new(evidence.artifact_id.as_str()).map_err(|_| Error::Identity)?,
        sha256: Sha::new(evidence.object.digest()).map_err(|_| Error::Identity)?,
        byte_length: u32::try_from(evidence.object.size()).map_err(|_| Error::Identity)?,
        media_type: Name::new(CHECK_MEDIA_TYPE).map_err(|_| Error::Identity)?,
        schema_id: Name::new(ReceiptV1::SCHEMA_ID).map_err(|_| Error::Identity)?,
    };
    let graph = crate::check::graph::Graph::resolve(
        &u64_receipt::LedgerObjects::new(store, deadline),
        &root,
    )
    .map_err(|_| Error::Identity)?;
    let receipt: ReceiptV1 = decode(graph.get(&root).map_err(|_| Error::Identity)?.bytes())
        .map_err(|_| Error::Identity)?;
    if receipt.identity.run_id.as_str() != evidence.artifact_id {
        return Err(Error::Identity);
    }
    let mut cited: Vec<(RunRecordKind, String, String, u64)> = Vec::new();
    for row in graph
        .rows(receipt.artifacts.inventory.as_ref())
        .map_err(|_| Error::Identity)?
    {
        let artifact: ArtifactV1 =
            serde_json::from_value(row.clone()).map_err(|_| Error::Identity)?;
        if let Some(kind) = u64_receipt::cited_kind(artifact.role.as_str()) {
            if cited.iter().any(|(held, ..)| *held == kind) {
                return Err(Error::Identity);
            }
            cited.push((
                kind,
                artifact.object.artifact_id.as_str().to_owned(),
                artifact.object.sha256.as_str().to_owned(),
                u64::from(artifact.object.byte_length),
            ));
        }
    }
    if committed(RunRecordKind::RunClock).is_none() {
        return Err(Error::Identity);
    }
    for kind in RunRecordKind::ALL {
        let row = cited.iter().find(|(held, ..)| *held == kind);
        match (row, committed(kind)) {
            (None, None) => {}
            (Some((_, id, digest, size)), Some((held_id, held_digest, held_size)))
                if id == held_id && digest == held_digest && *size == held_size => {}
            _ => return Err(Error::Identity),
        }
    }
    Ok(EvidenceIdentity {
        artifact_id: uuid(&evidence.artifact_id)?,
        media_type: CHECK_MEDIA_TYPE,
        schema_id: &evidence.schema_id,
    })
}

/// N1 (R17 round 2, decision 10): the toolchain and launcher readbacks may read `Some(true)` only
/// when EVERY stage of the workload launched — all three steps present and each `Completed`. An
/// unlaunched run has no steps, and a partial run stops early; neither is that observation.
fn every_stage_launched(completed: impl ExactSizeIterator<Item = bool>) -> bool {
    completed.len() == u64_receipt::STEPS.len() && completed.into_iter().all(|step| step)
}

/// What the per-check plan's hold came to: the plan, its refusal with the ids the refusal left in
/// the CAS, or a task cancelled since the driver's last read (decision 8, N7).
enum PlanOutcome {
    Planned(Box<plan::Planned>),
    Refused(plan::Refusal, Vec<String>),
    Cancelled,
}

/// The check recorded for a per-check plan the runtime refused (N2): `Error` under
/// [`PLAN_REFUSED_SCHEMA`], no criteria, the plan's own cost, its cleanup settled (nothing ran),
/// naming the refusal and the ids of the objects it left in the CAS (N13).
fn plan_refused_check(
    refusal: &plan::Refusal,
    orphans: &[String],
    used_ms: u64,
) -> Result<Check, Error> {
    Ok(Check {
        verdict: VerificationVerdict::Error,
        criteria: 0,
        evidence: serde_json::to_vec(&serde_json::json!({
            "kind": "plan_refused",
            "refusal": refusal.name(),
            "detail": format!("{refusal:?}"),
            "orphans": orphans,
        }))
        .map_err(|_| Error::Identity)?,
        schema_id: PLAN_REFUSED_SCHEMA.to_owned(),
        used_ms: Some(used_ms),
        cleanup_settled: true,
    })
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::{
        CHECK_MEDIA_TYPE, CHECK_TEARDOWN, Error, U64_CHECK_SCHEMA, check_window, cited,
        declared_criteria, preparation_charge, teardown_deadline,
    };
    use crate::store::RunRecordKind;
    use std::time::{Duration, Instant};

    /// Review MEDIUM-1 · the charge is the observed time below the reservation and the reservation
    /// at and above it: an empty reservation charges 0 whatever was observed.
    #[test]
    fn a_preparation_charge_never_exceeds_the_reservation() {
        assert_eq!(
            [(0, 0), (7, 0), (7, 9), (9, 9), (12, 9), (u64::MAX, 3)]
                .map(|(observed, reserved)| preparation_charge(observed, reserved)),
            [0, 0, 7, 9, 9, 3]
        );
    }

    /// Review MEDIUM-2 · removal gets the teardown share past the work window, capped at the task
    /// deadline.
    #[test]
    fn a_removal_gets_the_teardown_share_within_the_task_deadline() {
        let now = Instant::now();
        let work_until = now + Duration::from_millis(500);
        assert_eq!(
            teardown_deadline(work_until, 250, now + Duration::from_secs(60)),
            now + Duration::from_millis(750)
        );
        assert_eq!(
            teardown_deadline(work_until, 250, now + Duration::from_millis(600)),
            now + Duration::from_millis(600)
        );
    }

    /// The class declares one criterion: bit 0 alone.
    #[test]
    fn the_class_declares_exactly_its_criteria_bits() {
        assert_eq!(declared_criteria(), 1);
    }

    /// R14 · the check window off the origin: the reservation ends first (`until − now == R − T`);
    /// the deadline cuts (`until == deadline − T`, teardown to the deadline); `R == T` and `R < T`
    /// leave no window, separating `<=` from `<`; a deadline inside the teardown leaves none.
    #[test]
    fn a_check_window_keeps_the_teardown_on_both_arms_and_refuses_an_empty_one()
    -> Result<(), Box<dyn std::error::Error>> {
        let now = Instant::now();
        let teardown_ms = u64::try_from(CHECK_TEARDOWN.as_millis())?;
        let far = now + Duration::from_mins(20);
        let window = check_window(now, 1_700_000_000_000, 300_000, far)
            .ok_or("a far deadline leaves a window")?;
        assert_eq!(
            (
                window.begun,
                window.begun_unix_ms,
                window.until,
                window.teardown_until
            ),
            (
                now,
                1_700_000_000_000,
                now + Duration::from_millis(300_000 - teardown_ms),
                now + Duration::from_millis(300_000)
            )
        );
        let near = now + Duration::from_secs(45);
        let cut = check_window(now, 1_700_000_000_000, 300_000, near)
            .ok_or("a near deadline cuts the window")?;
        assert_eq!(
            (cut.begun, cut.until, cut.teardown_until),
            (
                now,
                near.checked_sub(CHECK_TEARDOWN)
                    .ok_or("the deadline holds the teardown")?,
                near
            )
        );
        assert_eq!(
            check_window(now, 1_700_000_000_000, teardown_ms, far),
            None,
            "R == T"
        );
        assert_eq!(
            check_window(now, 1_700_000_000_000, teardown_ms - 1, far),
            None,
            "R < T"
        );
        assert_eq!(
            check_window(now, 1_700_000_000_000, teardown_ms + 1, far).map(|w| w.until),
            Some(now + Duration::from_millis(1))
        );
        assert_eq!(
            check_window(now, 1_700_000_000_000, 300_000, now + CHECK_TEARDOWN),
            None,
            "deadline inside the teardown"
        );
        Ok(())
    }

    /// R15.7 · the cited records against the commitment, off the origin: the matching evidence
    /// yields the identity; a swapped id, digest or size, a cited kind the ledger did not commit, a
    /// committed kind the evidence omits, an extra key, another schema, malformed bytes and a set
    /// with no clock are each refused `Identity`.
    #[test]
    fn cited_records_must_equal_the_committed_set() -> Result<(), Box<dyn std::error::Error>> {
        const ID: &str = "28f30000-0000-4000-8000-0000000000a7";
        const CLOCK: (&str, &str, u64) = (
            "28f30000-0000-4000-8000-0000000000c1",
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            193,
        );
        const CLEANUP: (&str, &str, u64) = (
            "28f30000-0000-4000-8000-0000000000c3",
            "sha256:3333333333333333333333333333333333333333333333333333333333333333",
            2_048,
        );
        let held = |kind: RunRecordKind| match kind {
            RunRecordKind::RunClock => Some(CLOCK),
            RunRecordKind::RunCleanup => Some(CLEANUP),
            _ => None,
        };
        let body = |clock: (&str, &str, u64), cleanup: Option<(&str, &str, u64)>| {
            let mut records = serde_json::json!({ "run_clock": {
                "artifact_id": clock.0, "sha256": clock.1, "byte_length": clock.2 } });
            if let Some(cleanup) = cleanup {
                records["run_cleanup"] = serde_json::json!({
                    "artifact_id": cleanup.0, "sha256": cleanup.1, "byte_length": cleanup.2 });
            }
            serde_json::to_vec(&serde_json::json!({ "kind": "u64_check", "records": records }))
        };
        let sound = body(CLOCK, Some(CLEANUP))?;
        let identity = cited(U64_CHECK_SCHEMA, ID, &sound, held).map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            (
                identity.artifact_id.as_str(),
                identity.media_type,
                identity.schema_id
            ),
            (ID, CHECK_MEDIA_TYPE, U64_CHECK_SCHEMA)
        );
        let refused: [(&str, Vec<u8>); 8] = [
            (
                "id swapped",
                body((CLEANUP.0, CLOCK.1, CLOCK.2), Some(CLEANUP))?,
            ),
            (
                "digest swapped",
                body((CLOCK.0, CLEANUP.1, CLOCK.2), Some(CLEANUP))?,
            ),
            (
                "size off by one",
                body((CLOCK.0, CLOCK.1, CLOCK.2 + 1), Some(CLEANUP))?,
            ),
            ("a committed kind not cited", body(CLOCK, None)?),
            (
                "a cited kind not committed",
                body(CLOCK, Some(CLEANUP)).map(|mut bytes| {
                    bytes.truncate(bytes.len() - 2);
                    bytes.extend_from_slice(
                        br#","readbacks":{"artifact_id":"x","sha256":"y","byte_length":1}}}"#,
                    );
                    bytes
                })?,
            ),
            (
                "a key that is no record kind",
                body(CLOCK, Some(CLEANUP)).map(|mut bytes| {
                    bytes.truncate(bytes.len() - 2);
                    bytes.extend_from_slice(
                        br#","receipt":{"artifact_id":"x","sha256":"y","byte_length":1}}}"#,
                    );
                    bytes
                })?,
            ),
            (
                "another kind of evidence",
                serde_json::to_vec(
                    &serde_json::json!({ "kind": "refused_candidate", "records": {
                    "run_clock": { "artifact_id": CLOCK.0, "sha256": CLOCK.1, "byte_length": CLOCK.2 },
                    "run_cleanup": { "artifact_id": CLEANUP.0, "sha256": CLEANUP.1, "byte_length": CLEANUP.2 } } }),
                )?,
            ),
            ("malformed", b"{".to_vec()),
        ];
        for (case, bytes) in &refused {
            assert!(
                matches!(
                    cited(U64_CHECK_SCHEMA, ID, bytes, held),
                    Err(Error::Identity)
                ),
                "{case}"
            );
        }
        assert!(matches!(
            cited("hee3.scripted-check/1", ID, &sound, held),
            Err(Error::Identity)
        ));
        assert!(matches!(
            cited(U64_CHECK_SCHEMA, ID, &sound, |_| None),
            Err(Error::Identity)
        ));
        Ok(())
    }

    /// The committed run records as the ledger holds them: `(kind, artifact id, object)`.
    type Committed = Vec<(RunRecordKind, String, crate::store::Object)>;

    /// The committed set as `cited_receipt` reads it: `(artifact id, digest, size)` by kind.
    fn lookup<'a>(
        records: &'a [(RunRecordKind, String, crate::store::Object)],
    ) -> impl Fn(RunRecordKind) -> Option<(&'a str, &'a str, u64)> + 'a {
        move |kind| {
            records
                .iter()
                .find(|(held, ..)| *held == kind)
                .map(|(_, id, object)| (id.as_str(), object.digest(), object.size()))
        }
    }

    /// A receipt composed into the ledger through the runtime's own sink over the fixture plan and
    /// `cited` of the built records, its root committed under `root_id`; returns the root object.
    fn composed_into(
        store: &crate::store::Store,
        deadline: Instant,
        built: &crate::app::u64_receipt::fixtures::BuiltRecords,
        cited: &[(RunRecordKind, String, crate::store::Object)],
        root_id: &str,
    ) -> Result<
        Result<crate::store::Object, crate::app::u64_receipt::Refusal>,
        Box<dyn std::error::Error>,
    > {
        use super::Sink;
        use crate::app::u64_receipt::fixtures::{fixture_objects, fixture_prepared, host_fixture};
        use crate::app::u64_receipt::{Composing, Readbacks, compose};
        use crate::check::collector::Sink as _;
        let (clock, outcome, cleanup, _) = built;
        let prepared = fixture_prepared()?;
        let host = host_fixture();
        let obligation_ids = ["28fb0000-0000-4000-8000-000000000031".to_owned()];
        let mut sink = Sink::new(store, deadline);
        for (reference, bytes) in fixture_objects()? {
            sink.publish(&reference, &bytes)
                .map_err(|e| format!("{e:?}"))?;
        }
        let composed = compose(
            &mut sink,
            &Composing {
                prepared: &prepared,
                clock,
                outcome,
                cleanup,
                records: cited,
                captures: &[],
                evaluation: None,
                obligation_ids: &obligation_ids,
                root_id: Some(root_id),
                host: &host,
                readbacks: Readbacks {
                    seed: Some(true),
                    result: Some(true),
                    fixtures: Some(true),
                    oracle: Some(true),
                    harness: Some(true),
                    launcher: Some(true),
                    toolchain: Some(true),
                    profile: Some(true),
                },
            },
        );
        Ok(composed.map(|composed| {
            let root = composed.root.as_ref();
            assert_eq!(root.artifact_id.as_str(), root_id);
            crate::store::Object::of(root.sha256.as_str(), u64::from(root.byte_length))
        }))
    }

    /// The commitment varied one field at a time from `records`: the clock's id, digest and size
    /// (taken from or moved off the cleanup's), a cited kind not committed, and no clock at all.
    fn varied_commitments(
        records: &Committed,
    ) -> Result<[(&'static str, Committed); 5], Box<dyn std::error::Error>> {
        use crate::store::Object;
        let index = |kind: RunRecordKind| {
            records
                .iter()
                .position(|(held, ..)| *held == kind)
                .ok_or(format!("{kind:?} committed"))
        };
        let (clock_at, cleanup_at) = (
            index(RunRecordKind::RunClock)?,
            index(RunRecordKind::RunCleanup)?,
        );
        let varied = |edit: &dyn Fn(&mut Committed)| {
            let mut varied = records.clone();
            edit(&mut varied);
            varied
        };
        Ok([
            (
                "id swapped",
                varied(&|v| v[clock_at].1 = records[cleanup_at].1.clone()),
            ),
            (
                "digest swapped",
                varied(&|v| {
                    v[clock_at].2 =
                        Object::of(records[cleanup_at].2.digest(), records[clock_at].2.size());
                }),
            ),
            (
                "size off by one",
                varied(&|v| {
                    v[clock_at].2 =
                        Object::of(records[clock_at].2.digest(), records[clock_at].2.size() + 1);
                }),
            ),
            (
                "a cited kind not committed",
                varied(&|v| v.retain(|(kind, ..)| *kind != RunRecordKind::Readbacks)),
            ),
            (
                "no clock",
                varied(&|v| v.retain(|(kind, ..)| *kind != RunRecordKind::RunClock)),
            ),
        ])
    }

    /// Four receipts over the fixture plan, each committed under `ROOT`: one citing all four
    /// records, one citing three (a committed kind not cited), one citing the clock twice (the same
    /// object under a second id), one citing no clock at all.
    fn receipts(
        store: &crate::store::Store,
        deadline: Instant,
        built: &crate::app::u64_receipt::fixtures::BuiltRecords,
        records: &Committed,
    ) -> Result<[crate::store::Object; 4], Box<dyn std::error::Error>> {
        const ROOT: &str = "73000000-0000-4000-8000-000000000002";
        let clock_at = records
            .iter()
            .position(|(kind, ..)| *kind == RunRecordKind::RunClock)
            .ok_or("a clock")?;
        let mut twice = records.clone();
        twice.push((
            RunRecordKind::RunClock,
            "28fb0000-0000-4000-8000-0000000000c2".to_owned(),
            records[clock_at].2.clone(),
        ));
        let mut clockless = records.clone();
        clockless.remove(clock_at);
        let composed = |cited: &[(RunRecordKind, String, crate::store::Object)]| -> Result<
            crate::store::Object,
            Box<dyn std::error::Error>,
        > {
            Ok(composed_into(store, deadline, built, cited, ROOT)?.map_err(|e| format!("{e:?}"))?)
        };
        let whole = composed(records)?;
        let partial = composed(&records[..3])?;
        assert_ne!(whole, partial, "three records cited is another receipt");
        // A root id that is not the plan's run id is refused at the composer's finalize door
        // (decision 3), before any page of the receipt is published.
        assert!(
            matches!(
                composed_into(
                    store,
                    deadline,
                    built,
                    records,
                    "73000000-0000-4000-8000-0000000000ee"
                )?,
                Err(crate::app::u64_receipt::Refusal::Publisher {
                    stage: "finalize",
                    error: crate::check::collector::Error::RootId,
                })
            ),
            "a root id that is not the run id"
        );
        Ok([whole, partial, composed(&twice)?, composed(&clockless)?])
    }

    /// The evidence `accept` is handed for a receipt: `object` under `artifact_id`, the receipt schema.
    fn receipt_evidence(artifact_id: &str, object: &crate::store::Object) -> super::Evidence {
        use crate::contracts::receipt::{Address as _, ReceiptV1};
        super::Evidence {
            object: object.clone(),
            subject: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                .to_owned(),
            artifact_id: artifact_id.to_owned(),
            schema_id: ReceiptV1::SCHEMA_ID.to_owned(),
        }
    }

    /// R17 round 2, 2c-iii · the accept walk over a receipt: composed into the ledger through the
    /// runtime's own sink and read back as the graph the ledger holds, the receipt yields its
    /// identity — its root under the verification's artifact id, which is its `run_id` (decision
    /// 3). It is refused `Identity` when the commitment differs from what it cites (an id, a
    /// digest, a size, a cited kind not committed, a committed kind not cited, a kind cited twice,
    /// no clock committed, no clock on either side), when the evidence names another artifact id,
    /// when the evidence's object is not a receipt, and when the root is not in the ledger at all.
    #[test]
    fn a_receipt_is_accepted_only_as_the_graph_the_ledger_committed()
    -> Result<(), Box<dyn std::error::Error>> {
        use super::{CheckWindow, cited_receipt};
        use crate::app::u64_receipt::fixtures::built_records;
        use crate::contracts::UuidV4;
        use crate::contracts::receipt::{Address as _, ReceiptV1};
        use crate::store::{Object, Store};
        use std::os::unix::fs::DirBuilderExt;

        // The fixture plan's `run_id`: the root is committed under it (decision 3).
        const ROOT: &str = "73000000-0000-4000-8000-000000000002";
        const OTHER: &str = "73000000-0000-4000-8000-0000000000ee";
        let area =
            std::env::temp_dir().join(format!("hee3-runtime-receipt-{}", std::process::id()));
        std::fs::DirBuilder::new().mode(0o700).create(&area)?;
        let deadline = Instant::now() + Duration::from_secs(60);
        let store = Store::open(
            &area,
            UuidV4::parse("28fb0000-0000-4000-8000-000000000001")?,
            UuidV4::parse("28fb0000-0000-4000-8000-000000000002")?,
            true,
            deadline,
        )
        .map_err(|e| format!("{e:?}"))?;
        let begun = Instant::now();
        let window = CheckWindow {
            begun,
            begun_unix_ms: 1_758_900_000_000,
            until: begun + Duration::from_secs(290),
            teardown_until: begun + Duration::from_secs(300),
        };
        let built = built_records(
            &|bytes, id| store.publish(bytes, id, deadline),
            &window,
            begun + Duration::from_millis(41),
        )?;
        let records: Committed = built.3.clone();
        let [whole, partial, doubled, unclocked] = receipts(&store, deadline, &built, &records)?;
        let clock_at = records
            .iter()
            .position(|(kind, ..)| *kind == RunRecordKind::RunClock)
            .ok_or("a clock")?;
        let mut clockless = records.clone();
        clockless.remove(clock_at);
        let evidence = receipt_evidence;
        let sound = evidence(ROOT, &whole);
        let identity = cited_receipt(&store, deadline, &sound, lookup(&records))
            .map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            (
                identity.artifact_id.as_str(),
                identity.media_type,
                identity.schema_id
            ),
            (ROOT, CHECK_MEDIA_TYPE, ReceiptV1::SCHEMA_ID)
        );
        for (label, commitment) in &varied_commitments(&records)? {
            assert!(
                matches!(
                    cited_receipt(&store, deadline, &sound, lookup(commitment)),
                    Err(Error::Identity)
                ),
                "{label}"
            );
        }
        // The evidence itself: a committed kind the receipt does not cite, a kind cited twice,
        // another artifact id over the same bytes, a committed object that is not a receipt, and a
        // root the ledger does not hold.
        let absent = Object::of(
            "sha256:00000000000000000000000000000000000000000000000000000000000000ab",
            17,
        );
        for (label, wrong) in [
            ("a committed kind not cited", evidence(ROOT, &partial)),
            ("a kind cited twice", evidence(ROOT, &doubled)),
            ("another artifact id", evidence(OTHER, &whole)),
            ("not a receipt", evidence(ROOT, &records[clock_at].2)),
            ("not in the ledger", evidence(ROOT, &absent)),
        ] {
            assert!(
                matches!(
                    cited_receipt(&store, deadline, &wrong, lookup(&records)),
                    Err(Error::Identity)
                ),
                "{label}"
            );
        }
        assert!(
            matches!(
                cited_receipt(
                    &store,
                    deadline,
                    &evidence(ROOT, &unclocked),
                    lookup(&clockless)
                ),
                Err(Error::Identity)
            ),
            "no clock on either side"
        );
        drop(store);
        std::fs::remove_dir_all(&area)?;
        Ok(())
    }

    /// N1 · the toolchain and launcher readbacks need EVERY stage launched: no steps (an
    /// unlaunched run), one or two completed steps (a partial run) and a refused step are each
    /// `false`; three completed steps alone are `true`. The review of d062102 (HIGH): `all` on an
    /// empty run read `true`.
    #[test]
    fn every_stage_launched_needs_all_three_steps_completed() {
        use super::every_stage_launched;
        assert!(!every_stage_launched([].into_iter()), "no steps");
        assert!(!every_stage_launched([true].into_iter()), "one step");
        assert!(!every_stage_launched([true, true].into_iter()), "two steps");
        assert!(
            !every_stage_launched([true, false, true].into_iter()),
            "a refused step"
        );
        assert!(!every_stage_launched([true, true, false].into_iter()));
        assert!(every_stage_launched([true, true, true].into_iter()));
        assert!(
            !every_stage_launched([true, true, true, true].into_iter()),
            "four steps is not the workload"
        );
    }

    /// N2, N13 · the plan-refused check, asserted whole over two refusals differing in every field
    /// (F129): `Error`, no criteria, the plan's cost, cleanup settled, the refusal's name and detail,
    /// and the orphan ids in order.
    #[test]
    fn a_refused_plan_is_recorded_with_its_cost_and_its_orphans()
    -> Result<(), Box<dyn std::error::Error>> {
        use super::{PLAN_REFUSED_SCHEMA, plan_refused_check};
        use crate::app::plan::Refusal;
        use crate::store::VerificationVerdict;
        let orphans = [
            "28fc0000-0000-4000-8000-000000000001".to_owned(),
            "28fc0000-0000-4000-8000-000000000002".to_owned(),
        ];
        for (refusal, orphans, used_ms, name) in [
            (Refusal::Deadline, &orphans[..], 417_u64, "plan_deadline"),
            (
                Refusal::Editable("src/lib.rs"),
                &orphans[..1],
                9,
                "plan_editable",
            ),
        ] {
            let check =
                plan_refused_check(&refusal, orphans, used_ms).map_err(|e| format!("{e:?}"))?;
            assert_eq!(
                (
                    check.verdict,
                    check.criteria,
                    check.schema_id.as_str(),
                    check.used_ms,
                    check.cleanup_settled
                ),
                (
                    VerificationVerdict::Error,
                    0,
                    PLAN_REFUSED_SCHEMA,
                    Some(used_ms),
                    true
                )
            );
            let evidence: serde_json::Value = serde_json::from_slice(&check.evidence)?;
            assert_eq!(
                evidence,
                serde_json::json!({
                    "kind": "plan_refused",
                    "refusal": name,
                    "detail": format!("{refusal:?}"),
                    "orphans": orphans,
                })
            );
        }
        Ok(())
    }
}
