//! The attempt lifecycle over the real ledger (B14a-1c, design B14a-R1..R5): [`StoreRuntime`]
//! implements the driver's ports by composing Store, the class's candidate application and a
//! verifier seam. It reaches the ledger only through [`StoreTasks::with_store`], one hold per
//! write together with the head read that write depends on, and holds no `Store` between calls,
//! so no workload can run under the lock.
//!
//! Not here: the real workload (B14a-3), the receipt composer (B14a-2), the native candidate
//! source (B14a-4) and the dispatcher that chooses a [`Dispatch`] from the roster (B14b). This
//! module is the trusted check owner for one verdict only: a candidate the class refuses is
//! recorded `Failed` with no criterion satisfied, without a verifier call (B14a-R2.4).

use super::class_profile::{Profile, Workspace};
use super::evidence::{digest, fresh_id};
use super::repair::{self, Failure};
use super::tasks::{Poisoned, StoreTasks};
use crate::check::consistency::{U64_BOUNDS, U64_CRITERIA, U64_EDITABLE};
use crate::contracts::control::criteria_digest;
use crate::contracts::receipt::Name;
use crate::contracts::roster::{MAX_HISTORY, Selection};
use crate::contracts::{Generation, Sha256Digest, UuidV4};
use crate::store::{
    self, Binding, Effect, Expected, Object, Principal, RosterStart, Settlement, Stop, Store,
    TaskHead, Verification, VerificationVerdict,
};
use crate::task::LoopRefusal;
use crate::task::driver::{self, Acceptance, Checked as DriverChecked, StopReason, Work};
use crate::worker::workspace::{self, FileIdentity, Snapshot};
use std::path::Path;
use std::time::{Duration, Instant};

/// A candidate source's answer: the editable file's next full text, or nothing more to offer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Candidate {
    Replacement(Vec<u8>),
    Exhausted,
}

/// What the previous verification observed, handed to the next candidate request (F6).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Previous {
    pub verdict: VerificationVerdict,
    pub criteria: u64,
    pub evidence: Vec<u8>,
}

/// Where candidates come from. `next` is asked once per attempt, inside `execute`.
pub trait CandidateSource {
    fn next(&mut self, previous: Option<&Previous>) -> Candidate;
}

/// One independent check of an applied candidate: its verdict, the criterion bits it satisfied,
/// the evidence bytes to retain, its measured cost (`None` when unknown) and whether its own
/// cleanup settled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Check {
    pub verdict: VerificationVerdict,
    pub criteria: u64,
    pub evidence: Vec<u8>,
    pub used_ms: Option<u64>,
    pub cleanup_settled: bool,
}

/// The check of an applied candidate. It receives the frozen snapshot, never a path to mutate.
pub trait Verifier {
    fn check(&mut self, subject: &Snapshot, deadline: Instant) -> Check;
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
    WorkspaceNotDeclared,
    BaselineCapture,
    BaselineMismatch,
    ProtectedCapture,
    ProtectedMismatch,
}

impl Refusal {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CriteriaNotClass => "criteria_not_class",
            Self::ReservationEmpty => "reservation_empty",
            Self::ReservationTooSmall => "reservation_too_small",
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
    /// The work settled with a known cost and settled cleanup.
    settled: bool,
    verified: bool,
    /// The check, once recorded, had a known cost and settled cleanup (review H1).
    check_settled: bool,
}

/// The attempt lifecycle for one task over the real ledger.
struct StoreRuntime<'a, C, V> {
    tasks: &'a StoreTasks,
    dispatch: Dispatch<'a>,
    source: C,
    verifier: V,
    baseline: Snapshot,
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

/// Dispatch one admitted task: the pre-dispatch checks, then the driver over [`StoreRuntime`].
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
    let mut runtime = StoreRuntime {
        tasks,
        dispatch,
        source,
        verifier,
        baseline: prepared.baseline,
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
    let (staging, event) = (fresh(deadline)?, fresh(deadline)?);
    let reason = Name::new(refusal.name()).map_err(|_| Error::Identity)?;
    tasks.with_store(|store| -> Result<(), Error> {
        let head = store.get(dispatch.principal, dispatch.task, deadline)?;
        let observed_ms = millis(origin.elapsed());
        let bytes = serde_json::to_vec(&serde_json::json!({
            "kind": "hee3.pre-dispatch-refusal/1",
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
    fn record(
        &mut self,
        index: usize,
        check: &Check,
        subject: &str,
    ) -> Result<(Object, VerificationVerdict, bool), Error> {
        let begun = self.attempts.get(index).ok_or(Error::Identity)?;
        let (staging, event) = (fresh(self.deadline)?, fresh(self.deadline)?);
        let (generation, object, verdict, reconciled) =
            self.tasks.with_store(|store| -> Result<_, Error> {
                let head = self.current(store)?;
                let used_ms = check
                    .used_ms
                    .filter(|used| *used <= head.reserved_verify_ms);
                let verdict = if check.criteria & !declared_criteria() != 0 {
                    // Bits outside the class's criteria are an invalid check, never a stranded
                    // task (re-review LOW-3), recorded as such.
                    VerificationVerdict::Invalid
                } else if check.verdict == VerificationVerdict::Cancelled && !head.cancellation {
                    VerificationVerdict::Error
                } else {
                    check.verdict
                };
                let object = store.publish(&check.evidence, uuid(&staging)?, self.deadline)?;
                let generation = store.record_verification(
                    &expected(&head, begun)?,
                    &Verification {
                        verdict,
                        subject: Sha256Digest::parse(subject).map_err(|_| Error::Identity)?,
                        evidence: object.clone(),
                        used_ms,
                        cleanup_settled: check.cleanup_settled,
                    },
                    uuid(&event)?,
                    self.deadline,
                )?;
                Ok((
                    generation,
                    object,
                    verdict,
                    used_ms.is_some() && check.cleanup_settled,
                ))
            })??;
        self.written(&generation, event)?;
        if let Some(begun) = self.attempts.get_mut(index) {
            begun.verified = true;
            begun.check_settled = reconciled;
        }
        self.previous = Some(Previous {
            verdict,
            criteria: check.criteria,
            evidence: check.evidence.clone(),
        });
        Ok((object, verdict, reconciled))
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
        let stop_bytes = serde_json::to_vec(&serde_json::json!({
            "kind": "hee3.task-stop/1", "reason": name, "attempts": self.attempts.len(),
        }))
        .map_err(|_| Error::Identity)?;
        let idle_bytes = serde_json::to_vec(&serde_json::json!({
            "kind": "verification_not_started", "durable_cancellation": true,
        }))
        .map_err(|_| Error::Identity)?;
        let ids = [
            fresh(self.deadline)?,
            fresh(self.deadline)?,
            fresh(self.deadline)?,
            fresh(self.deadline)?,
        ];
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
        let [_, _, _, event] = ids;
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
        match self.source.next(self.previous.as_ref()) {
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
                used_ms: Some(0),
                cleanup_settled: true,
            };
            (check, subject)
        } else {
            let applied = begun.applied.as_ref().ok_or(Error::Identity)?;
            let subject = applied.content_digest().ok_or(Error::Identity)?;
            (self.verifier.check(applied, self.deadline), subject)
        };
        let (object, verdict, reconciled) = self.record(index, &check, &subject)?;
        // An unknown cost or unsettled cleanup is an obligation the stop must keep (review H1).
        if !reconciled {
            return Ok(DriverChecked::Unsettled);
        }
        Ok(match verdict {
            VerificationVerdict::Passed => DriverChecked::Passed {
                evidence: Evidence { object, subject },
                criteria: check.criteria,
            },
            VerificationVerdict::Failed => DriverChecked::Failed {
                criteria: check.criteria,
            },
            VerificationVerdict::Invalid => DriverChecked::Invalid,
            VerificationVerdict::Error => DriverChecked::Error,
            VerificationVerdict::Timeout => DriverChecked::Timeout,
            // The task was cancelled (`record` turned any other `Cancelled` into `Error`): nothing
            // was satisfied, and the driver's next cancellation read stops it as cancelled.
            VerificationVerdict::Cancelled => DriverChecked::Failed { criteria: 0 },
        })
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
                let result = store
                    .prepare_verified_acceptance(
                        &expected,
                        uuid(&event)?,
                        Sha256Digest::parse(&evidence.subject).map_err(|_| Error::Identity)?,
                        &evidence.object,
                        std::slice::from_ref(&evidence.object),
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
fn declared_criteria() -> u64 {
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

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::{declared_criteria, preparation_charge, teardown_deadline};
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
}
