//! The four run records a settle commits (B14a-2b-ii; design R12 part 1 in
//! `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`): what the runtime observed of a
//! run, sealed like [`crate::check::decision::Decision`] — fields private, one production
//! constructor each over the live fact, and one decoder that is reachable only through the
//! ledger's commitment ([`crate::store::Committed`], which only the ledger's own reads —
//! [`crate::store::Store::committed_run`] and [`crate::store::Store::committed_check`] — construct). So a run record is held either because the runtime observed it or because the
//! ledger committed its digest when the runtime wrote it: there is no constructor from free bytes.
//!
//! Each record serializes to JSON (`RECORD_MEDIA_TYPE`) under its kind's schema id, both owned by
//! `store::run_records`, so the receipt cites it by the identity the settle recorded.

use crate::app::candidates::{Outcome as CandidateOutcome, Settle};
use crate::app::workload::{Outcome, Run, Step};
use crate::check::decision::Timing;
use crate::contracts::UuidV4;
use crate::contracts::receipt::Ref;
use crate::store::{Committed, Error as StoreError, RunRecordKind, Store};
use crate::worker::Finish;
use crate::worker::aggregate;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Why a record could not be built from the live fact, or read back from its commitment.
#[derive(Debug)]
pub enum Refusal {
    /// An instant precedes the run's origin, or an offset does not fit.
    Clock,
    /// The captures handed in do not pair the run's steps one to one.
    Steps,
    /// The commitment is of another kind than the record read.
    Kind {
        /// The kind the commitment names.
        found: RunRecordKind,
        /// The kind that was read.
        expected: RunRecordKind,
    },
    /// The committed bytes are not this record's shape.
    Encoding,
    /// The ledger refused the read.
    Store(StoreError),
}

impl From<StoreError> for Refusal {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

/// The runtime's clock for one run, as `app::runtime` holds it: the origin the run's offsets are
/// measured from, that origin as unix time, the work window's end and the task deadline.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeClock {
    /// Where the run's offsets start.
    pub origin: Instant,
    /// The origin as unix milliseconds, read once by the runtime beside `origin` (for a check,
    /// `CheckWindow::begun_unix_ms`).
    pub origin_unix_ms: u64,
    /// The end of the observation's work window — the attempt's for a settle (B14a-1c: the work
    /// reservation read at begin, less the teardown share), the check's for a check (B14a-3b:
    /// `CheckWindow::until`). Which one is fixed by `store::Committed::observation()`, never by
    /// this field alone (R14, amending R13.3: the schema's `work_cutoff_ms` is pinned).
    pub work_until: Instant,
    /// The observation's cleanup deadline: the task deadline for a settle, `CheckWindow::
    /// teardown_until` for a check (R15.5, amending the field's first meaning; the schema's
    /// `task_deadline_ms` is pinned and `timing()` reads it as the cleanup deadline).
    pub deadline: Instant,
}

/// The stop intents the runtime issued during the run, if any.
#[derive(Clone, Copy, Debug, Default)]
pub struct Intents {
    /// When the runtime asked the run to stop for its deadline.
    pub timeout: Option<Instant>,
    /// When the runtime asked the run to stop for a cancellation.
    pub cancellation: Option<Instant>,
}

fn offset_ms(origin: Instant, at: Instant) -> Result<u64, Refusal> {
    let elapsed = at.checked_duration_since(origin).ok_or(Refusal::Clock)?;
    u64::try_from(elapsed.as_millis()).map_err(|_| Refusal::Clock)
}

fn optional_offset_ms(origin: Instant, at: Option<Instant>) -> Result<Option<u64>, Refusal> {
    at.map(|at| offset_ms(origin, at)).transpose()
}

/// When the run began and ended, by the runtime's clock, and the windows it ran under: exactly
/// what [`Timing`] needs, plus the origin.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RunClock {
    /// The origin as unix milliseconds; every other field is milliseconds from it.
    #[serde(rename = "origin_unix_ms")]
    origin_unix: u64,
    #[serde(rename = "work_cutoff_ms")]
    work_cutoff: u64,
    #[serde(rename = "task_deadline_ms")]
    task_deadline: u64,
    #[serde(rename = "timeout_intent_ms")]
    timeout_intent: Option<u64>,
    #[serde(rename = "cancellation_intent_ms")]
    cancellation_intent: Option<u64>,
    #[serde(rename = "decisive_ms")]
    decisive: Option<u64>,
    #[serde(rename = "observed_ms")]
    observed: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunClockFields {
    #[serde(rename = "origin_unix_ms")]
    origin_unix: u64,
    #[serde(rename = "work_cutoff_ms")]
    work_cutoff: u64,
    #[serde(rename = "task_deadline_ms")]
    task_deadline: u64,
    #[serde(rename = "timeout_intent_ms")]
    timeout_intent: Option<u64>,
    #[serde(rename = "cancellation_intent_ms")]
    cancellation_intent: Option<u64>,
    #[serde(rename = "decisive_ms")]
    decisive: Option<u64>,
    #[serde(rename = "observed_ms")]
    observed: u64,
}

impl RunClock {
    /// The one production constructor: offsets from the runtime's own clock, refused if any
    /// instant precedes the origin.
    ///
    /// # Errors
    /// [`Refusal::Clock`].
    pub fn observe(
        clock: &RuntimeClock,
        intents: Intents,
        decisive: Option<Instant>,
        observed: Instant,
    ) -> Result<Self, Refusal> {
        Ok(Self {
            origin_unix: clock.origin_unix_ms,
            work_cutoff: offset_ms(clock.origin, clock.work_until)?,
            task_deadline: offset_ms(clock.origin, clock.deadline)?,
            timeout_intent: optional_offset_ms(clock.origin, intents.timeout)?,
            cancellation_intent: optional_offset_ms(clock.origin, intents.cancellation)?,
            decisive: optional_offset_ms(clock.origin, decisive)?,
            observed: offset_ms(clock.origin, observed)?,
        })
    }

    /// The origin as unix milliseconds.
    #[must_use]
    pub const fn origin_unix_ms(&self) -> u64 {
        self.origin_unix
    }

    /// What the decision reads: derived here, never re-typed.
    #[must_use]
    pub const fn timing(&self) -> Timing {
        Timing {
            work_deadline_ms: self.work_cutoff,
            cleanup_deadline_ms: self.task_deadline,
            observed_ms: self.observed,
            decisive_ms: self.decisive,
            timeout_intent_ms: self.timeout_intent,
            cancellation_intent_ms: self.cancellation_intent,
        }
    }

    fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        let fields: RunClockFields =
            serde_json::from_slice(bytes).map_err(|_| Refusal::Encoding)?;
        Ok(Self {
            origin_unix: fields.origin_unix,
            work_cutoff: fields.work_cutoff,
            task_deadline: fields.task_deadline,
            timeout_intent: fields.timeout_intent,
            cancellation_intent: fields.cancellation_intent,
            decisive: fields.decisive,
            observed: fields.observed,
        })
    }
}

/// The name of a [`Outcome`], without its payload: what a record states about how the run ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeName {
    Matched,
    Mismatch,
    InvalidOutput,
    InvalidSubject,
    ProducerFailed,
    ProducerError,
    LauncherFailed,
    Timeout,
    Cancelled,
    PendingCleanup,
    SetupFailed,
}

impl OutcomeName {
    /// The name of `outcome`: one arm per variant, so a new outcome is a compile error here.
    #[must_use]
    pub const fn of(outcome: &Outcome) -> Self {
        match outcome {
            Outcome::Matched(_) => Self::Matched,
            Outcome::Mismatch(_) => Self::Mismatch,
            Outcome::InvalidOutput(_) => Self::InvalidOutput,
            Outcome::InvalidSubject => Self::InvalidSubject,
            Outcome::ProducerFailed => Self::ProducerFailed,
            Outcome::ProducerError => Self::ProducerError,
            Outcome::LauncherFailed => Self::LauncherFailed,
            Outcome::Timeout => Self::Timeout,
            Outcome::Cancelled => Self::Cancelled,
            Outcome::PendingCleanup => Self::PendingCleanup,
            Outcome::SetupFailed => Self::SetupFailed,
        }
    }
}

/// The name of a check's aggregate refusal (R22-2): what a run outcome states about a slice that
/// refused before the workload ran, and what the `resources` obligation states about a teardown
/// that did not settle.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateRefusal {
    Invalid,
    Bound,
    Deadline,
    Cancelled,
    Identity,
    Io,
    State,
    Manager,
    Process,
    Limits,
    Busy,
}

impl AggregateRefusal {
    /// The name of `error`: one arm per variant, so a new refusal is a compile error here.
    #[must_use]
    pub const fn of(error: aggregate::Error) -> Self {
        match error {
            aggregate::Error::Invalid => Self::Invalid,
            aggregate::Error::Bound => Self::Bound,
            aggregate::Error::Deadline => Self::Deadline,
            aggregate::Error::Cancelled => Self::Cancelled,
            aggregate::Error::Identity => Self::Identity,
            aggregate::Error::Io => Self::Io,
            aggregate::Error::State => Self::State,
            aggregate::Error::Manager => Self::Manager,
            aggregate::Error::Process => Self::Process,
            aggregate::Error::Limits => Self::Limits,
            aggregate::Error::Busy => Self::Busy,
        }
    }
}

/// Why a step was refused before it ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum RefusedKind {
    /// The namespace could not be prepared.
    Prepare,
    /// The process was refused; whether its channels were cleaned up.
    Process {
        /// The channel cleanup settled.
        channel_cleanup_complete: bool,
    },
}

/// One step of the run: what it was, and either the capture published for it or why it was
/// refused.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepRecord {
    label: String,
    capture: Option<Ref>,
    refused: Option<RefusedKind>,
}

impl StepRecord {
    /// The step's label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// The capture published for a completed step.
    #[must_use]
    pub const fn capture(&self) -> Option<&Ref> {
        self.capture.as_ref()
    }

    /// Why a refused step did not run.
    #[must_use]
    pub const fn refused(&self) -> Option<RefusedKind> {
        self.refused
    }
}

/// Whether the process and FIFO custody settled, independent of verdict and retained artifacts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessCleanup {
    Complete,
    Incomplete,
}

/// Whether the frozen subjects read back unchanged after the run.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Subjects {
    Unchanged,
    Changed,
}

/// Whether the run observed a cancellation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cancellation {
    Observed,
    NotObserved,
}

/// Whether the retained tmpfs descriptors were released.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scratch {
    Released,
    Retained,
}

/// The four independent observations a run reports, each named for what was observed rather
/// than a bool whose meaning lives in the field name.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observations {
    /// Process and FIFO custody.
    pub process_cleanup: ProcessCleanup,
    /// The frozen subjects.
    pub subjects: Subjects,
    /// A cancellation.
    pub cancellation: Cancellation,
    /// The retained scratch.
    pub scratch: Scratch,
}

impl Observations {
    /// The run's four observations, from `workload`'s own predicates.
    #[must_use]
    pub const fn of(run: &Run) -> Self {
        Self {
            process_cleanup: if run.process_cleanup_complete {
                ProcessCleanup::Complete
            } else {
                ProcessCleanup::Incomplete
            },
            subjects: if run.subjects_unchanged {
                Subjects::Unchanged
            } else {
                Subjects::Changed
            },
            cancellation: if run.cancellation_observed {
                Cancellation::Observed
            } else {
                Cancellation::NotObserved
            },
            scratch: if run.scratch_released {
                Scratch::Released
            } else {
                Scratch::Retained
            },
        }
    }
}

/// How the run ended: its outcome by name, each step's capture or refusal, the four run
/// observations, and — only when the check's aggregate refused before the workload ran — that
/// refusal's name (R22-2; absent otherwise, so a record without one encodes as before).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RunOutcome {
    outcome: OutcomeName,
    steps: Vec<StepRecord>,
    observations: Observations,
    #[serde(skip_serializing_if = "Option::is_none")]
    aggregate_refusal: Option<AggregateRefusal>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunOutcomeFields {
    outcome: OutcomeName,
    steps: Vec<StepRecord>,
    observations: Observations,
    #[serde(default)]
    aggregate_refusal: Option<AggregateRefusal>,
}

impl RunOutcome {
    /// The one production constructor: the run as `workload` returned it, the capture the live
    /// half published for each completed step (`None` for a refused one), one per step, and the
    /// check's aggregate refusal, if the aggregate refused before the workload ran.
    ///
    /// # Errors
    /// [`Refusal::Steps`] when `captures` does not pair `run.steps` one to one — a completed step
    /// with no capture, a refused step with one, or a different count.
    pub fn of(
        run: &Run,
        captures: &[Option<Ref>],
        aggregate_refusal: Option<AggregateRefusal>,
    ) -> Result<Self, Refusal> {
        if captures.len() != run.steps.len() {
            return Err(Refusal::Steps);
        }
        let mut steps = Vec::with_capacity(run.steps.len());
        for (step, capture) in run.steps.iter().zip(captures) {
            let record = match (step, capture) {
                (Step::Completed { label, .. }, Some(capture)) => StepRecord {
                    label: (*label).to_owned(),
                    capture: Some(capture.clone()),
                    refused: None,
                },
                (Step::Refused { label, error }, None) => StepRecord {
                    label: (*label).to_owned(),
                    capture: None,
                    refused: Some(match error {
                        crate::worker::namespace::NamespaceRunError::Prepare(_) => {
                            RefusedKind::Prepare
                        }
                        crate::worker::namespace::NamespaceRunError::Process {
                            channel_cleanup_complete,
                            ..
                        } => RefusedKind::Process {
                            channel_cleanup_complete: *channel_cleanup_complete,
                        },
                    }),
                },
                _ => return Err(Refusal::Steps),
            };
            steps.push(record);
        }
        Ok(Self {
            outcome: OutcomeName::of(&run.outcome),
            steps,
            observations: Observations::of(run),
            aggregate_refusal,
        })
    }

    /// How the run ended.
    #[must_use]
    pub const fn outcome(&self) -> OutcomeName {
        self.outcome
    }

    /// The steps, in run order.
    #[must_use]
    pub fn steps(&self) -> &[StepRecord] {
        &self.steps
    }

    /// The four run observations.
    #[must_use]
    pub const fn observations(&self) -> Observations {
        self.observations
    }

    fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        let fields: RunOutcomeFields =
            serde_json::from_slice(bytes).map_err(|_| Refusal::Encoding)?;
        for step in &fields.steps {
            if step.capture.is_some() == step.refused.is_some() {
                return Err(Refusal::Encoding);
            }
        }
        Ok(Self {
            outcome: fields.outcome,
            steps: fields.steps,
            observations: fields.observations,
            aggregate_refusal: fields.aggregate_refusal,
        })
    }
}

/// How an aggregate or an obligation settled: the receipt's cleanup vocabulary for a run that
/// reached its settle (`not_started` cannot be observed by then).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Settlement {
    Settled,
    Pending,
    Failed,
    Unknown,
}

/// One obligation the run left, and how it stands.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationRecord {
    /// The obligation's identity.
    pub id: String,
    /// How it stands.
    pub state: Settlement,
    /// The refusal that left it unsettled, where one is named (R22-2: the `resources` obligation
    /// carries its teardown's refusal); absent otherwise, so a record without one encodes as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<AggregateRefusal>,
}

/// What the teardown settled and what it could not.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RunCleanup {
    aggregate: Settlement,
    obligations: Vec<ObligationRecord>,
    unresolved: u64,
    evidence: Vec<Ref>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunCleanupFields {
    aggregate: Settlement,
    obligations: Vec<ObligationRecord>,
    unresolved: u64,
    evidence: Vec<Ref>,
}

impl RunCleanup {
    /// The one production constructor. `unresolved` is derived from the obligations, never
    /// supplied.
    #[must_use]
    pub fn of(aggregate: Settlement, obligations: &[ObligationRecord], evidence: &[Ref]) -> Self {
        Self {
            aggregate,
            obligations: obligations.to_vec(),
            unresolved: Self::count_unresolved(obligations),
            evidence: evidence.to_vec(),
        }
    }

    fn count_unresolved(obligations: &[ObligationRecord]) -> u64 {
        obligations
            .iter()
            .filter(|obligation| obligation.state != Settlement::Settled)
            .count() as u64
    }

    /// How the aggregate settled.
    #[must_use]
    pub const fn aggregate(&self) -> Settlement {
        self.aggregate
    }

    /// Every obligation the run left.
    #[must_use]
    pub fn obligations(&self) -> &[ObligationRecord] {
        &self.obligations
    }

    /// How many obligations are not settled.
    #[must_use]
    pub const fn unresolved(&self) -> u64 {
        self.unresolved
    }

    /// The cleanup's evidence.
    #[must_use]
    pub fn evidence(&self) -> &[Ref] {
        &self.evidence
    }

    fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        let fields: RunCleanupFields =
            serde_json::from_slice(bytes).map_err(|_| Refusal::Encoding)?;
        if fields.unresolved != Self::count_unresolved(&fields.obligations) {
            return Err(Refusal::Encoding);
        }
        Ok(Self {
            aggregate: fields.aggregate,
            obligations: fields.obligations,
            unresolved: fields.unresolved,
            evidence: fields.evidence,
        })
    }
}

/// One output the live half read back after the run, and whether it was what the run left.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputReadback {
    /// The output's reference.
    pub output: Ref,
    /// The readback matched the object the reference names.
    pub matched: bool,
}

/// The readbacks the live half performed after the run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Readbacks {
    attempt: String,
    subjects_verified: bool,
    protected_unchanged: bool,
    outputs: Vec<OutputReadback>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadbacksFields {
    attempt: String,
    subjects_verified: bool,
    protected_unchanged: bool,
    outputs: Vec<OutputReadback>,
}

impl Readbacks {
    /// The one production constructor.
    #[must_use]
    pub fn of(
        attempt: UuidV4<'_>,
        subjects_verified: bool,
        protected_unchanged: bool,
        outputs: &[OutputReadback],
    ) -> Self {
        Self {
            attempt: attempt.as_str().to_owned(),
            subjects_verified,
            protected_unchanged,
            outputs: outputs.to_vec(),
        }
    }

    /// The attempt the readbacks belong to.
    #[must_use]
    pub fn attempt(&self) -> &str {
        &self.attempt
    }

    /// The subjects were verified unchanged.
    #[must_use]
    pub const fn subjects_verified(&self) -> bool {
        self.subjects_verified
    }

    /// The protected snapshot was unchanged.
    #[must_use]
    pub const fn protected_unchanged(&self) -> bool {
        self.protected_unchanged
    }

    /// Each output's readback.
    #[must_use]
    pub fn outputs(&self) -> &[OutputReadback] {
        &self.outputs
    }

    fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        let fields: ReadbacksFields =
            serde_json::from_slice(bytes).map_err(|_| Refusal::Encoding)?;
        UuidV4::parse(&fields.attempt).map_err(|_| Refusal::Encoding)?;
        Ok(Self {
            attempt: fields.attempt,
            subjects_verified: fields.subjects_verified,
            protected_unchanged: fields.protected_unchanged,
            outputs: fields.outputs,
        })
    }
}

/// The decoders, sealed: the trait lives in a private module, so no code outside this one can name
/// it, and `decode_bytes` is reachable only through [`RunRecord::read`] (review of b2b7b85, gap 1:
/// a required method of a public trait is public by construction).
mod sealed {
    use super::Refusal;

    pub trait Decode: Sized {
        fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal>;
    }
}

/// A run record: its kind, its bytes, and the one way to read it back — through the ledger's
/// commitment of it.
pub trait RunRecord: Sized + Serialize + sealed::Decode {
    /// The kind the settle commits this record under.
    const KIND: RunRecordKind;

    /// The record's bytes, as the settle publishes them.
    ///
    /// # Errors
    /// [`Refusal::Encoding`] when the record cannot be serialized (it always can).
    fn to_bytes(&self) -> Result<Vec<u8>, Refusal> {
        serde_json::to_vec(self).map_err(|_| Refusal::Encoding)
    }

    /// The record the ledger committed as `committed`: refused unless the commitment is of this
    /// kind, the bytes are the committed object's (the store verifies digest and size), and they
    /// decode to a record the constructor could have made.
    ///
    /// # Errors
    /// [`Refusal::Kind`], [`Refusal::Store`], [`Refusal::Encoding`].
    fn read(committed: &Committed, store: &Store, deadline: Instant) -> Result<Self, Refusal> {
        let found = committed.kind();
        if found != Self::KIND {
            return Err(Refusal::Kind {
                found,
                expected: Self::KIND,
            });
        }
        let bytes = store.read_object(committed.object(), deadline)?;
        <Self as sealed::Decode>::decode_bytes(&bytes)
    }
}

impl sealed::Decode for RunClock {
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal> {
        Self::decode(bytes)
    }
}

impl RunRecord for RunClock {
    const KIND: RunRecordKind = RunRecordKind::RunClock;
}

impl sealed::Decode for RunOutcome {
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal> {
        Self::decode(bytes)
    }
}

impl RunRecord for RunOutcome {
    const KIND: RunRecordKind = RunRecordKind::RunOutcome;
}

impl sealed::Decode for RunCleanup {
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal> {
        Self::decode(bytes)
    }
}

impl RunRecord for RunCleanup {
    const KIND: RunRecordKind = RunRecordKind::RunCleanup;
}

impl sealed::Decode for Readbacks {
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal> {
        Self::decode(bytes)
    }
}

impl RunRecord for Readbacks {
    const KIND: RunRecordKind = RunRecordKind::Readbacks;
}

/// How the model's answer ended, as the worker reported it: [`Finish`] spelled for the record, one
/// arm per variant so a new finish is a compile error here.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishName {
    Stop,
    Length,
    Refusal,
    Error,
    Cancelled,
}

impl FinishName {
    #[must_use]
    pub const fn of(finish: Finish) -> Self {
        match finish {
            Finish::Stop => Self::Stop,
            Finish::Length => Self::Length,
            Finish::Refusal => Self::Refusal,
            Finish::Error => Self::Error,
            Finish::Cancelled => Self::Cancelled,
        }
    }
}

/// What the worker's one call came to, without payload: the names the source itself spells
/// (`Refusal::name`, `native::Error::name`), so the record says what the stop body and the
/// refused-candidate evidence say.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkerOutcome {
    /// A replacement was handed to the runtime; `replacement_bytes` is its length.
    Replacement,
    /// The class grammar refused the answer under this `candidate_*` name.
    Refused { name: String },
    /// The provider failed by this name.
    Provider { name: String },
}

/// The worker's settle of one attempt (B14a-5, R19.2): what the model was asked under which adapter
/// row, what it cost in tokens and wall time, the identity and raw digests, and how it ended.
/// Sealed like the other records: one production constructor over the settle the source handed the
/// runtime WITH the candidate ([`crate::app::runtime::Answer`]), decoded only through the ledger's
/// commitment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorkerSettle {
    attempt: String,
    adapter_profile: String,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    wall_ms: u64,
    finish: Option<FinishName>,
    identity_sha256: Option<String>,
    raw_sha256: Option<String>,
    outcome: WorkerOutcome,
    replacement_bytes: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerSettleFields {
    attempt: String,
    adapter_profile: String,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    wall_ms: u64,
    finish: Option<FinishName>,
    identity_sha256: Option<String>,
    raw_sha256: Option<String>,
    outcome: WorkerOutcome,
    replacement_bytes: Option<u64>,
}

impl WorkerSettle {
    /// The one production constructor: over the settle the source handed back with its candidate.
    #[must_use]
    pub fn of(settle: &Settle) -> Self {
        let (outcome, replacement_bytes) = match &settle.outcome {
            CandidateOutcome::Replacement(len) => (
                WorkerOutcome::Replacement,
                Some(u64::try_from(*len).unwrap_or(u64::MAX)),
            ),
            CandidateOutcome::Refused(refusal) => (
                WorkerOutcome::Refused {
                    name: refusal.name().to_owned(),
                },
                None,
            ),
            CandidateOutcome::Provider(error) => (
                WorkerOutcome::Provider {
                    name: error.name().to_owned(),
                },
                None,
            ),
        };
        Self {
            attempt: settle.attempt.clone(),
            adapter_profile: settle.adapter.to_owned(),
            input_tokens: settle.input_tokens,
            output_tokens: settle.output_tokens,
            wall_ms: settle.wall_ms,
            finish: settle.finish.map(FinishName::of),
            identity_sha256: settle.identity_sha256.clone(),
            raw_sha256: settle.raw_sha256.clone(),
            outcome,
            replacement_bytes,
        }
    }

    /// The attempt the settle is of.
    #[must_use]
    pub fn attempt(&self) -> &str {
        &self.attempt
    }

    /// The adapter row the request named.
    #[must_use]
    pub fn adapter_profile(&self) -> &str {
        &self.adapter_profile
    }

    /// Prompt tokens the provider reported, when it did.
    #[must_use]
    pub const fn input_tokens(&self) -> Option<u64> {
        self.input_tokens
    }

    /// Completion tokens the provider reported, when it did.
    #[must_use]
    pub const fn output_tokens(&self) -> Option<u64> {
        self.output_tokens
    }

    /// The wall time of the call, from the source's ask to its answer.
    #[must_use]
    pub const fn wall_ms(&self) -> u64 {
        self.wall_ms
    }

    /// How the answer ended, when one was reported.
    #[must_use]
    pub const fn finish(&self) -> Option<FinishName> {
        self.finish
    }

    /// The digest of the provider's identity readback, when one was read.
    #[must_use]
    pub fn identity_sha256(&self) -> Option<&str> {
        self.identity_sha256.as_deref()
    }

    /// The digest of the provider's raw response, when one was received.
    #[must_use]
    pub fn raw_sha256(&self) -> Option<&str> {
        self.raw_sha256.as_deref()
    }

    /// What the call came to.
    #[must_use]
    pub const fn outcome(&self) -> &WorkerOutcome {
        &self.outcome
    }

    /// The replacement's length, for a `Replacement` outcome.
    #[must_use]
    pub const fn replacement_bytes(&self) -> Option<u64> {
        self.replacement_bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        let fields: WorkerSettleFields =
            serde_json::from_slice(bytes).map_err(|_| Refusal::Encoding)?;
        UuidV4::parse(&fields.attempt).map_err(|_| Refusal::Encoding)?;
        // A length travels with a replacement and with nothing else: the constructor's shape.
        if matches!(fields.outcome, WorkerOutcome::Replacement)
            != fields.replacement_bytes.is_some()
        {
            return Err(Refusal::Encoding);
        }
        Ok(Self {
            attempt: fields.attempt,
            adapter_profile: fields.adapter_profile,
            input_tokens: fields.input_tokens,
            output_tokens: fields.output_tokens,
            wall_ms: fields.wall_ms,
            finish: fields.finish,
            identity_sha256: fields.identity_sha256,
            raw_sha256: fields.raw_sha256,
            outcome: fields.outcome,
            replacement_bytes: fields.replacement_bytes,
        })
    }
}

impl sealed::Decode for WorkerSettle {
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal> {
        Self::decode(bytes)
    }
}

impl RunRecord for WorkerSettle {
    const KIND: RunRecordKind = RunRecordKind::WorkerSettle;
}

#[cfg(test)]
mod tests {
    use super::sealed::Decode as _;
    use super::{
        AggregateRefusal, Cancellation, FinishName, Intents, ObligationRecord, Observations,
        OutcomeName, OutputReadback, ProcessCleanup, Readbacks, Refusal, RefusedKind, RunCleanup,
        RunClock, RunOutcome, RunRecord, RuntimeClock, Scratch, Settlement, Subjects, WorkerSettle,
    };
    use crate::app::candidates::{
        Outcome as CandidateOutcome, Refusal as CandidateRefusal, Settle,
    };
    use crate::app::workload::{Outcome, Run, Step};
    use crate::contracts::UuidV4;
    use crate::contracts::receipt::{Id, Name, Ref, Sha};
    use crate::store::RunRecordKind;
    use crate::worker::Finish;
    use crate::worker::namespace::NamespaceRunError;
    use crate::worker::native::Error as NativeError;
    use crate::worker::process::Refusal as ProcessRefusal;
    use serde_json::json;
    use std::time::{Duration, Instant};

    fn reference(n: u8) -> Ref {
        Ref {
            artifact_id: Id::new(format!("28f00000-0000-4000-8000-0000000000{n:02x}")).unwrap(),
            sha256: Sha::new(format!("sha256:{}", format!("{n:02x}").repeat(32))).unwrap(),
            byte_length: u32::from(n) + 1,
            media_type: Name::new("application/json").unwrap(),
            schema_id: Name::new("hee3.raw/1").unwrap(),
        }
    }

    fn clock(origin: Instant) -> RuntimeClock {
        RuntimeClock {
            origin,
            origin_unix_ms: 1_790_000_000_000,
            work_until: origin + Duration::from_mins(15),
            deadline: origin + Duration::from_mins(20),
        }
    }

    /// Two clocks differing in every field round-trip through their bytes whole, and `timing` is
    /// the decision's input field for field (F124).
    #[test]
    fn a_run_clock_round_trips_whole_and_derives_the_decisions_timing() -> Result<(), Refusal> {
        let origin = Instant::now();
        let first = RunClock::observe(
            &clock(origin),
            Intents {
                timeout: Some(origin + Duration::from_millis(900_500)),
                cancellation: None,
            },
            Some(origin + Duration::from_mins(2)),
            origin + Duration::from_secs(901),
        )?;
        let second = RunClock::observe(
            &RuntimeClock {
                origin,
                origin_unix_ms: 1_790_000_123_456,
                work_until: origin + Duration::from_millis(500),
                deadline: origin + Duration::from_millis(9_000),
            },
            Intents {
                timeout: None,
                cancellation: Some(origin + Duration::from_millis(300)),
            },
            None,
            origin + Duration::from_millis(7_000),
        )?;
        for (record, expected) in [
            (
                &first,
                (
                    1_790_000_000_000,
                    900_000,
                    1_200_000,
                    Some(900_500),
                    None,
                    Some(120_000),
                    901_000,
                ),
            ),
            (
                &second,
                (1_790_000_123_456, 500, 9_000, None, Some(300), None, 7_000),
            ),
        ] {
            let read = RunClock::decode_bytes(&record.to_bytes()?)?;
            assert_eq!(&read, record);
            let timing = read.timing();
            assert_eq!(
                (
                    read.origin_unix_ms(),
                    timing.work_deadline_ms,
                    timing.cleanup_deadline_ms,
                    timing.timeout_intent_ms,
                    timing.cancellation_intent_ms,
                    timing.decisive_ms,
                    timing.observed_ms
                ),
                expected
            );
        }
        assert_ne!(first, second);
        Ok(())
    }

    /// An instant before the origin is refused by name; a field the record does not have, too.
    #[test]
    fn a_run_clock_refuses_an_instant_before_its_origin_and_an_unknown_field() {
        let before = Instant::now();
        let origin = before + Duration::from_secs(60);
        let early = RunClock::observe(&clock(origin), Intents::default(), None, before);
        assert!(matches!(early.err(), Some(Refusal::Clock)));
        let bytes = br#"{"origin_unix_ms":1,"work_cutoff_ms":2,"task_deadline_ms":3,"timeout_intent_ms":null,"cancellation_intent_ms":null,"decisive_ms":null,"observed_ms":4,"extra":0}"#;
        assert!(matches!(
            RunClock::decode_bytes(bytes).err(),
            Some(Refusal::Encoding)
        ));
    }

    fn run(outcome: Outcome, steps: Vec<Step>) -> Run {
        Run {
            outcome,
            steps,
            outputs: Vec::new(),
            retained_paths: Vec::new(),
            process_cleanup_complete: true,
            subjects_unchanged: false,
            cancellation_observed: true,
            scratch_released: false,
            decisive: None,
        }
    }

    fn refused(label: &'static str, channel_cleanup_complete: bool) -> Step {
        Step::Refused {
            label,
            error: NamespaceRunError::Process {
                refusal: ProcessRefusal::Deadline,
                channel_cleanup_complete,
            },
        }
    }

    /// A run of refused steps (the shape a test can build without a live namespace) records its
    /// outcome by name, each step's refusal, and the four observations; it round-trips whole.
    #[test]
    fn a_run_outcome_records_each_step_and_round_trips_whole() -> Result<(), Refusal> {
        let run = run(
            Outcome::Timeout,
            vec![refused("compile", true), refused("link", false)],
        );
        let record = RunOutcome::of(&run, &[None, None], None)?;
        assert_eq!(record.outcome(), OutcomeName::Timeout);
        assert_eq!(
            record
                .steps()
                .iter()
                .map(|step| (step.label(), step.capture().is_some(), step.refused()))
                .collect::<Vec<_>>(),
            [
                (
                    "compile",
                    false,
                    Some(RefusedKind::Process {
                        channel_cleanup_complete: true
                    })
                ),
                (
                    "link",
                    false,
                    Some(RefusedKind::Process {
                        channel_cleanup_complete: false
                    })
                ),
            ]
        );
        assert_eq!(
            record.observations(),
            Observations {
                process_cleanup: ProcessCleanup::Complete,
                subjects: Subjects::Changed,
                cancellation: Cancellation::Observed,
                scratch: Scratch::Retained,
            }
        );
        assert_eq!(RunOutcome::decode_bytes(&record.to_bytes()?)?, record);
        // R22-2 · no refusal: no key, so today's encoding is unchanged; a refusal is carried by
        // name and round-trips whole.
        let json = |bytes: Vec<u8>| {
            serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|_| Refusal::Encoding)
        };
        assert_eq!(json(record.to_bytes()?)?.get("aggregate_refusal"), None);
        let refused = RunOutcome::of(&run, &[None, None], Some(AggregateRefusal::Limits))?;
        assert_eq!(
            json(refused.to_bytes()?)?.get("aggregate_refusal"),
            Some(&json!("limits"))
        );
        assert_eq!(RunOutcome::decode_bytes(&refused.to_bytes()?)?, refused);
        Ok(())
    }

    /// Captures must pair the steps one to one: a different count, or a capture for a refused
    /// step, is refused before any record exists; bytes pairing both or neither are refused too.
    #[test]
    fn a_run_outcome_refuses_captures_that_do_not_pair_its_steps() {
        let run = run(Outcome::SetupFailed, vec![refused("compile", true)]);
        assert!(matches!(
            RunOutcome::of(&run, &[], None).err(),
            Some(Refusal::Steps)
        ));
        assert!(matches!(
            RunOutcome::of(&run, &[Some(reference(1))], None).err(),
            Some(Refusal::Steps)
        ));
        let both = br#"{"outcome":"timeout","steps":[{"label":"compile","capture":{"artifact_id":"28f00000-0000-4000-8000-000000000001","sha256":"sha256:0101010101010101010101010101010101010101010101010101010101010101","byte_length":2,"media_type":"application/json","schema_id":"hee3.raw/1"},"refused":"prepare"}],"observations":{"process_cleanup":"complete","subjects":"unchanged","cancellation":"not_observed","scratch":"released"}}"#;
        assert!(matches!(
            RunOutcome::decode_bytes(both).err(),
            Some(Refusal::Encoding)
        ));
    }

    /// Every outcome has a name, and the names are the wire's spellings.
    #[test]
    fn every_outcome_is_named() {
        let names: Vec<OutcomeName> = [
            Outcome::InvalidSubject,
            Outcome::ProducerFailed,
            Outcome::ProducerError,
            Outcome::LauncherFailed,
            Outcome::Timeout,
            Outcome::Cancelled,
            Outcome::PendingCleanup,
            Outcome::SetupFailed,
        ]
        .iter()
        .map(OutcomeName::of)
        .collect();
        assert_eq!(
            names,
            [
                OutcomeName::InvalidSubject,
                OutcomeName::ProducerFailed,
                OutcomeName::ProducerError,
                OutcomeName::LauncherFailed,
                OutcomeName::Timeout,
                OutcomeName::Cancelled,
                OutcomeName::PendingCleanup,
                OutcomeName::SetupFailed,
            ]
        );
        assert_eq!(
            serde_json::to_string(&OutcomeName::PendingCleanup).unwrap(),
            "\"pending_cleanup\""
        );
    }

    /// R22-2 · every aggregate refusal has a name, and the names are the wire's spellings: the whole
    /// table over all eleven refusals. An obligation carrying one round-trips whole through the
    /// cleanup record; one without carries no key.
    #[test]
    fn aggregate_refusals_are_named_on_the_wire() -> Result<(), Refusal> {
        use crate::worker::aggregate::Error;
        let names = [
            Error::Invalid,
            Error::Bound,
            Error::Deadline,
            Error::Cancelled,
            Error::Identity,
            Error::Io,
            Error::State,
            Error::Manager,
            Error::Process,
            Error::Limits,
            Error::Busy,
        ]
        .map(|error| {
            serde_json::to_value(AggregateRefusal::of(error)).map_err(|_| Refusal::Encoding)
        });
        assert_eq!(
            names.into_iter().collect::<Result<Vec<_>, _>>()?,
            [
                "invalid",
                "bound",
                "deadline",
                "cancelled",
                "identity",
                "io",
                "state",
                "manager",
                "process",
                "limits",
                "busy"
            ]
            .map(|name| json!(name))
        );
        let obligations = [
            ObligationRecord {
                id: "scratch".to_owned(),
                state: Settlement::Settled,
                refusal: None,
            },
            ObligationRecord {
                id: "resources".to_owned(),
                state: Settlement::Pending,
                refusal: Some(AggregateRefusal::Busy),
            },
        ];
        let record = RunCleanup::of(Settlement::Pending, &obligations, &[]);
        let bytes = record.to_bytes()?;
        let wire: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| Refusal::Encoding)?;
        assert_eq!(
            wire["obligations"],
            json!([
                {"id": "scratch", "state": "settled"},
                {"id": "resources", "state": "pending", "refusal": "busy"},
            ])
        );
        assert_eq!(RunCleanup::decode_bytes(&bytes)?, record);
        Ok(())
    }

    /// The unresolved count is derived from the obligations, and bytes that disagree are refused.
    #[test]
    fn a_run_cleanup_derives_its_unresolved_count_and_refuses_bytes_that_disagree()
    -> Result<(), Refusal> {
        let obligations = [
            ObligationRecord {
                id: "process".to_owned(),
                state: Settlement::Settled,
                refusal: None,
            },
            ObligationRecord {
                id: "scratch".to_owned(),
                state: Settlement::Pending,
                refusal: None,
            },
            ObligationRecord {
                id: "fifo".to_owned(),
                state: Settlement::Unknown,
                refusal: None,
            },
        ];
        let record = RunCleanup::of(Settlement::Failed, &obligations, &[reference(3)]);
        assert_eq!(
            (
                record.aggregate(),
                record.unresolved(),
                record.evidence().len()
            ),
            (Settlement::Failed, 2, 1)
        );
        assert_eq!(record.obligations(), obligations);
        let bytes = record.to_bytes()?;
        assert_eq!(RunCleanup::decode_bytes(&bytes)?, record);
        let fitted = String::from_utf8(bytes)
            .unwrap()
            .replace("\"unresolved\":2", "\"unresolved\":0");
        assert!(matches!(
            RunCleanup::decode_bytes(fitted.as_bytes()).err(),
            Some(Refusal::Encoding)
        ));
        Ok(())
    }

    /// Readbacks round-trip whole, and an attempt that is not a `UUIDv4` is refused at decode.
    #[test]
    fn readbacks_round_trip_whole_and_refuse_a_malformed_attempt() -> Result<(), Refusal> {
        let attempt = UuidV4::parse("28f00000-0000-4000-8000-0000000000a7").unwrap();
        let outputs = [
            OutputReadback {
                output: reference(5),
                matched: true,
            },
            OutputReadback {
                output: reference(6),
                matched: false,
            },
        ];
        let record = Readbacks::of(attempt, true, false, &outputs);
        assert_eq!(
            (
                record.attempt(),
                record.subjects_verified(),
                record.protected_unchanged()
            ),
            (attempt.as_str(), true, false)
        );
        assert_eq!(record.outputs(), outputs);
        assert_eq!(Readbacks::decode_bytes(&record.to_bytes()?)?, record);
        let malformed = br#"{"attempt":"not-a-uuid","subjects_verified":true,"protected_unchanged":true,"outputs":[]}"#;
        assert!(matches!(
            Readbacks::decode_bytes(malformed).err(),
            Some(Refusal::Encoding)
        ));
        Ok(())
    }

    /// Each record's kind is the store's, one per type.
    #[test]
    fn each_record_has_its_kind() {
        assert_eq!(
            [
                <RunClock as RunRecord>::KIND,
                <RunOutcome as RunRecord>::KIND,
                <RunCleanup as RunRecord>::KIND,
                <Readbacks as RunRecord>::KIND,
                <WorkerSettle as RunRecord>::KIND
            ],
            [
                RunRecordKind::RunClock,
                RunRecordKind::RunOutcome,
                RunRecordKind::RunCleanup,
                RunRecordKind::Readbacks,
                RunRecordKind::WorkerSettle
            ]
        );
    }

    /// B14a-5 (R19.2) · three worker settles differing in every field — a replacement, a grammar
    /// refusal, a provider failure — encode to the pinned spelling WHOLE and round-trip through
    /// their bytes; the finish and outcome names are the source's own.
    #[test]
    fn a_worker_settle_encodes_whole_and_round_trips() -> Result<(), Refusal> {
        let sha = |c: char| Some(format!("sha256:{}", c.to_string().repeat(64)));
        let cases = [
            (
                Settle {
                    attempt: "28f00000-0000-4000-8000-0000000000a1".to_owned(),
                    adapter: "ollama-fc44-12ff8654/2",
                    input_tokens: Some(552),
                    output_tokens: Some(258),
                    wall_ms: 4_321,
                    finish: Some(Finish::Stop),
                    identity_sha256: sha('a'),
                    raw_sha256: sha('b'),
                    outcome: CandidateOutcome::Replacement(1_776),
                },
                json!({
                    "attempt": "28f00000-0000-4000-8000-0000000000a1",
                    "adapter_profile": "ollama-fc44-12ff8654/2",
                    "input_tokens": 552, "output_tokens": 258, "wall_ms": 4321,
                    "finish": "stop",
                    "identity_sha256": sha('a'), "raw_sha256": sha('b'),
                    "outcome": "replacement", "replacement_bytes": 1776,
                }),
            ),
            (
                Settle {
                    attempt: "28f00000-0000-4000-8000-0000000000b2".to_owned(),
                    adapter: "ollama-fc44-12ff8654/1",
                    input_tokens: None,
                    output_tokens: Some(64),
                    wall_ms: 7,
                    finish: Some(Finish::Length),
                    identity_sha256: None,
                    raw_sha256: sha('c'),
                    outcome: CandidateOutcome::Refused(CandidateRefusal::Truncated),
                },
                json!({
                    "attempt": "28f00000-0000-4000-8000-0000000000b2",
                    "adapter_profile": "ollama-fc44-12ff8654/1",
                    "input_tokens": null, "output_tokens": 64, "wall_ms": 7,
                    "finish": "length",
                    "identity_sha256": null, "raw_sha256": sha('c'),
                    "outcome": {"refused": {"name": "candidate_truncated"}},
                    "replacement_bytes": null,
                }),
            ),
            (
                Settle {
                    attempt: "28f00000-0000-4000-8000-0000000000c3".to_owned(),
                    adapter: "ollama-fc44-12ff8654/2",
                    input_tokens: Some(1),
                    output_tokens: None,
                    wall_ms: 900_000,
                    finish: None,
                    identity_sha256: sha('d'),
                    raw_sha256: None,
                    outcome: CandidateOutcome::Provider(NativeError::Identity),
                },
                json!({
                    "attempt": "28f00000-0000-4000-8000-0000000000c3",
                    "adapter_profile": "ollama-fc44-12ff8654/2",
                    "input_tokens": 1, "output_tokens": null, "wall_ms": 900_000,
                    "finish": null,
                    "identity_sha256": sha('d'), "raw_sha256": null,
                    "outcome": {"provider": {"name": "identity"}},
                    "replacement_bytes": null,
                }),
            ),
        ];
        for (settle, expected) in cases {
            let record = WorkerSettle::of(&settle);
            let bytes = record.to_bytes()?;
            let encoded: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|_| Refusal::Encoding)?;
            assert_eq!(encoded, expected);
            assert_eq!(WorkerSettle::decode_bytes(&bytes)?, record);
            assert_eq!(record.attempt(), settle.attempt);
            assert_eq!(record.adapter_profile(), settle.adapter);
            assert_eq!(record.wall_ms(), settle.wall_ms);
        }
        // Every finish spelled, one arm per variant.
        let spelled: Vec<String> = [
            Finish::Stop,
            Finish::Length,
            Finish::Refusal,
            Finish::Error,
            Finish::Cancelled,
        ]
        .into_iter()
        .map(|finish| serde_json::to_string(&FinishName::of(finish)).map_err(|_| Refusal::Encoding))
        .collect::<Result<_, _>>()?;
        assert_eq!(
            spelled,
            [
                "\"stop\"",
                "\"length\"",
                "\"refusal\"",
                "\"error\"",
                "\"cancelled\""
            ]
        );
        Ok(())
    }

    /// B14a-5 · four shapes the constructor cannot make are refused at decode: a replacement with no
    /// length, a refusal with one, an attempt that is not a UUID, and a field the record does not
    /// have. (Not every such shape: a provider outcome with tokens, or a finish of `length` on a
    /// replacement, decodes — the looseness matches the sibling records and the objects are
    /// digest-pinned; R19 round 1 finding 9, stated.)
    #[test]
    fn a_worker_settle_refuses_four_shapes_the_constructor_cannot_make() {
        let base = json!({
            "attempt": "28f00000-0000-4000-8000-0000000000a1",
            "adapter_profile": "ollama-fc44-12ff8654/2",
            "input_tokens": 552, "output_tokens": 258, "wall_ms": 4321, "finish": "stop",
            "identity_sha256": null, "raw_sha256": null,
            "outcome": "replacement", "replacement_bytes": 1776,
        });
        assert!(WorkerSettle::decode_bytes(base.to_string().as_bytes()).is_ok());
        let cases = [
            (
                "a replacement without its length",
                json!({"replacement_bytes": null}),
            ),
            (
                "a refusal carrying a length",
                json!({"outcome": {"refused": {"name": "candidate_empty"}}}),
            ),
            (
                "an attempt that is not a UUID",
                json!({"attempt": "attempt-1"}),
            ),
            ("a field the record does not have", json!({"tokens": 3})),
        ];
        for (case, edit) in cases {
            let mut faulty = base.clone();
            for (key, value) in edit.as_object().into_iter().flatten() {
                faulty[key] = value.clone();
            }
            assert!(
                matches!(
                    WorkerSettle::decode_bytes(faulty.to_string().as_bytes()),
                    Err(Refusal::Encoding)
                ),
                "{case}"
            );
        }
    }
}
