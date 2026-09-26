//! The four run records a settle commits (B14a-2b-ii; design R12 part 1 in
//! `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`): what the runtime observed of a
//! run, sealed like [`crate::check::decision::Decision`] — fields private, one production
//! constructor each over the live fact, and one decoder that is reachable only through the
//! ledger's commitment ([`crate::store::Committed`], which only [`crate::store::Store::committed_run`]
//! constructs). So a run record is held either because the runtime observed it or because the
//! ledger committed its digest when the runtime wrote it: there is no constructor from free bytes.
//!
//! Each record serializes to JSON (`RECORD_MEDIA_TYPE`) under its kind's schema id, both owned by
//! `store::run_records`, so the receipt cites it by the identity the settle recorded.

use crate::app::workload::{Outcome, Run, Step};
use crate::check::decision::Timing;
use crate::contracts::UuidV4;
use crate::contracts::receipt::Ref;
use crate::store::{Committed, Error as StoreError, RunRecordKind, Store};
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
    /// The origin as unix milliseconds, read once by the runtime.
    pub origin_unix_ms: u64,
    /// The end of the attempt's work window (B14a-1c: the reservation read at begin, less the
    /// teardown share).
    pub work_until: Instant,
    /// The task deadline.
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

/// Why a step was refused before it ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

/// How the run ended: its outcome by name, each step's capture or refusal, and the four run
/// observations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RunOutcome {
    outcome: OutcomeName,
    steps: Vec<StepRecord>,
    observations: Observations,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunOutcomeFields {
    outcome: OutcomeName,
    steps: Vec<StepRecord>,
    observations: Observations,
}

impl RunOutcome {
    /// The one production constructor: the run as `workload` returned it, and the capture the
    /// live half published for each completed step (`None` for a refused one), one per step.
    ///
    /// # Errors
    /// [`Refusal::Steps`] when `captures` does not pair `run.steps` one to one — a completed step
    /// with no capture, a refused step with one, or a different count.
    pub fn of(run: &Run, captures: &[Option<Ref>]) -> Result<Self, Refusal> {
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

/// A run record: its kind, its bytes, and the one way to read it back — through the ledger's
/// commitment of it.
pub trait RunRecord: Sized + Serialize {
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
        Self::decode_bytes(&bytes)
    }

    /// This record's own decoder, reachable only through [`RunRecord::read`].
    ///
    /// # Errors
    /// [`Refusal::Encoding`].
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal>;
}

impl RunRecord for RunClock {
    const KIND: RunRecordKind = RunRecordKind::RunClock;
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal> {
        Self::decode(bytes)
    }
}

impl RunRecord for RunOutcome {
    const KIND: RunRecordKind = RunRecordKind::RunOutcome;
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal> {
        Self::decode(bytes)
    }
}

impl RunRecord for RunCleanup {
    const KIND: RunRecordKind = RunRecordKind::RunCleanup;
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal> {
        Self::decode(bytes)
    }
}

impl RunRecord for Readbacks {
    const KIND: RunRecordKind = RunRecordKind::Readbacks;
    fn decode_bytes(bytes: &[u8]) -> Result<Self, Refusal> {
        Self::decode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Cancellation, Intents, ObligationRecord, Observations, OutcomeName, OutputReadback,
        ProcessCleanup, Readbacks, Refusal, RefusedKind, RunCleanup, RunClock, RunOutcome,
        RunRecord, RuntimeClock, Scratch, Settlement, Subjects,
    };
    use crate::app::workload::{Outcome, Run, Step};
    use crate::contracts::UuidV4;
    use crate::contracts::receipt::{Id, Name, Ref, Sha};
    use crate::store::RunRecordKind;
    use crate::worker::namespace::NamespaceRunError;
    use crate::worker::process::Refusal as ProcessRefusal;
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
        let record = RunOutcome::of(&run, &[None, None])?;
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
        Ok(())
    }

    /// Captures must pair the steps one to one: a different count, or a capture for a refused
    /// step, is refused before any record exists; bytes pairing both or neither are refused too.
    #[test]
    fn a_run_outcome_refuses_captures_that_do_not_pair_its_steps() {
        let run = run(Outcome::SetupFailed, vec![refused("compile", true)]);
        assert!(matches!(
            RunOutcome::of(&run, &[]).err(),
            Some(Refusal::Steps)
        ));
        assert!(matches!(
            RunOutcome::of(&run, &[Some(reference(1))]).err(),
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

    /// The unresolved count is derived from the obligations, and bytes that disagree are refused.
    #[test]
    fn a_run_cleanup_derives_its_unresolved_count_and_refuses_bytes_that_disagree()
    -> Result<(), Refusal> {
        let obligations = [
            ObligationRecord {
                id: "aggregate".to_owned(),
                state: Settlement::Settled,
            },
            ObligationRecord {
                id: "scratch".to_owned(),
                state: Settlement::Pending,
            },
            ObligationRecord {
                id: "fifo".to_owned(),
                state: Settlement::Unknown,
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
                <Readbacks as RunRecord>::KIND
            ],
            [
                RunRecordKind::RunClock,
                RunRecordKind::RunOutcome,
                RunRecordKind::RunCleanup,
                RunRecordKind::Readbacks
            ]
        );
    }
}
