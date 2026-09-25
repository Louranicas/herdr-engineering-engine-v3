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
    /// Another writer moved the task other than by one cancellation or an instance observation
    /// (B14a-R2.1, R3 G1): a second dispatcher or a restart overlap, detected, never absorbed.
    ConcurrentWriter,
    /// A stored value the runtime wrote or reads is not the shape it must be.
    Identity,
    /// No entropy for a fresh identity before the deadline.
    Entropy,
    /// A candidate's retained path could not be removed and read back as gone.
    Cleanup(workspace::Error),
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
    began: Instant,
    applied: Option<Snapshot>,
    /// A candidate the class refused, and the refusal's name.
    refused: Option<(Vec<u8>, &'static str)>,
    settled: bool,
    verified: bool,
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
    reserved_work_ms: u64,
    own: u64,
    last_event: Option<String>,
    cancelled_at_start: bool,
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
    let head =
        tasks.with_store(|store| store.get(dispatch.principal, dispatch.task, deadline))??;
    let prepared = match prepare(&head, profile, &dispatch, origin, deadline) {
        Ok(prepared) => prepared,
        Err(refusal) => {
            refuse(tasks, &dispatch, &head, refusal, origin, deadline)?;
            return Ok(Outcome::Refused(refusal));
        }
    };
    let own = number(&head.generation)?;
    let mut runtime = StoreRuntime {
        tasks,
        dispatch,
        source,
        verifier,
        baseline: prepared.baseline,
        digests: prepared.digests,
        origin,
        deadline,
        reserved_work_ms: head.reserved_work_ms,
        own,
        last_event: None,
        cancelled_at_start: head.cancellation,
        attempts: Vec::new(),
        previous: None,
    };
    let count = u8::try_from(U64_CRITERIA.len()).map_err(|_| Error::Identity)?;
    driver::run(&mut runtime, count)
        .map(Outcome::Driven)
        .map_err(|error| match error {
            driver::Error::Runtime(error) => error,
            driver::Error::Policy(refusal) => Error::Policy(refusal),
        })
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
/// origin; no attempt row is written.
fn refuse(
    tasks: &StoreTasks,
    dispatch: &Dispatch<'_>,
    head: &TaskHead,
    refusal: Refusal,
    origin: Instant,
    deadline: Instant,
) -> Result<(), Error> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "kind": "hee3.pre-dispatch-refusal/1",
        "task": dispatch.task.as_str(),
        "refusal": refusal.name(),
    }))
    .map_err(|_| Error::Identity)?;
    let (staging, event) = (fresh(deadline)?, fresh(deadline)?);
    let reason = Name::new(refusal.name()).map_err(|_| Error::Identity)?;
    let generation = parse_generation(&head.generation)?;
    let used_ms = millis(origin.elapsed());
    tasks.with_store(|store| -> Result<(), Error> {
        let object = store.publish(&bytes, uuid(&staging)?, deadline)?;
        store.finish_preparation(
            dispatch.principal,
            Stop {
                task: dispatch.task,
                generation,
                reason: &reason,
                evidence: &object,
                event: uuid(&event)?,
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(used_ms),
                cleanup_settled: true,
                ready_to_verify: false,
            },
            deadline,
        )?;
        Ok(())
    })?
}

impl<C: CandidateSource, V: Verifier> StoreRuntime<'_, C, V> {
    /// The head, admitted only if every write since the runtime's own last one is a single
    /// cancellation or an instance observation, and its generation accounts for exactly those
    /// (B14a-R2.1, R3 G1). Called inside the hold of the write it guards.
    fn current(&self, store: &Store) -> Result<TaskHead, Error> {
        let head = store.get(self.dispatch.principal, self.dispatch.task, self.deadline)?;
        let generation = number(&head.generation)?;
        if generation == self.own {
            return Ok(head);
        }
        let cancellations = match &self.last_event {
            // Before its first write only a cancellation can move the task (an observation writes
            // at the current generation).
            None => u64::from(head.cancellation && !self.cancelled_at_start),
            Some(event) => {
                let events = store.events_after(
                    self.dispatch.principal,
                    self.dispatch.task,
                    uuid(event)?,
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
                if cancellations > 1 {
                    return Err(Error::ConcurrentWriter);
                }
                cancellations
            }
        };
        if generation != self.own + cancellations {
            return Err(Error::ConcurrentWriter);
        }
        Ok(head)
    }

    fn written(&mut self, generation: &str, event: String) -> Result<(), Error> {
        self.own = number(generation)?;
        self.last_event = Some(event);
        Ok(())
    }

    fn begun(&self, attempt: &Attempt) -> Result<&Begun, Error> {
        self.attempts.get(attempt.index).ok_or(Error::Identity)
    }

    /// The work deadline: the reservation from the origin, less the teardown share.
    fn work_deadline(&self) -> Instant {
        let work = self
            .reserved_work_ms
            .saturating_sub(self.dispatch.teardown_ms);
        self.deadline.min(self.origin + Duration::from_millis(work))
    }

    /// One settle of the current attempt, in one hold with its head read. `used_ms` is `None`
    /// when the measured time overran the reservation (an overrun is not a clean failure).
    fn settle(
        &mut self,
        index: usize,
        cleanup_settled: bool,
        ready_to_verify: bool,
    ) -> Result<(), Error> {
        let begun = self.attempts.get(index).ok_or(Error::Identity)?;
        let used = millis(begun.began.elapsed());
        let event = fresh(self.deadline)?;
        let (generation, known) = self.tasks.with_store(|store| -> Result<_, Error> {
            let head = self.current(store)?;
            // An overrun of what the reservation holds now is an unknown cost, never a clean one.
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

    /// Record one verification of the current attempt and remember it for the next candidate.
    fn record(
        &mut self,
        index: usize,
        check: &Check,
        subject: &str,
    ) -> Result<Option<Object>, Error> {
        let begun = self.attempts.get(index).ok_or(Error::Identity)?;
        let (staging, event) = (fresh(self.deadline)?, fresh(self.deadline)?);
        let (generation, object) =
            self.tasks
                .with_store(|store| -> Result<(String, Object), Error> {
                    let head = self.current(store)?;
                    let object = store.publish(&check.evidence, uuid(&staging)?, self.deadline)?;
                    let generation = store.record_verification(
                        &expected(&head, begun)?,
                        &Verification {
                            verdict: check.verdict,
                            subject: Sha256Digest::parse(subject).map_err(|_| Error::Identity)?,
                            evidence: object.clone(),
                            used_ms: check.used_ms,
                            cleanup_settled: check.cleanup_settled,
                        },
                        uuid(&event)?,
                        self.deadline,
                    )?;
                    Ok((generation, object))
                })??;
        self.written(&generation, event)?;
        if let Some(begun) = self.attempts.get_mut(index) {
            begun.verified = true;
        }
        self.previous = Some(Previous {
            verdict: check.verdict,
            criteria: check.criteria,
            evidence: check.evidence.clone(),
        });
        Ok(Some(object))
    }

    /// Remove a refused candidate's retained path and read it back as gone (B14a-R2.4).
    fn remove(&self, failure: &Failure) -> Result<bool, Error> {
        let Some(path) = &failure.partial_path else {
            return Ok(true);
        };
        workspace::remove_owned(path, self.work_deadline()).map_err(Error::Cleanup)?;
        Ok(std::fs::symlink_metadata(path).is_err())
    }
}

impl<C: CandidateSource, V: Verifier> driver::Runtime for StoreRuntime<'_, C, V> {
    type Error = Error;
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
        let lease_ms = millis(
            self.work_deadline()
                .saturating_duration_since(Instant::now()),
        );
        let [baseline, protected, profile] = &self.digests;
        let binding = Binding {
            baseline: Sha256Digest::parse(baseline).map_err(|_| Error::Identity)?,
            protected: Sha256Digest::parse(protected).map_err(|_| Error::Identity)?,
            profile: Sha256Digest::parse(profile).map_err(|_| Error::Identity)?,
        };
        let began = Instant::now();
        let roster = self.tasks.with_store(|store| -> Result<_, Error> {
            let head = self.current(store)?;
            let workspace = head.workspace_id.as_deref().ok_or(Error::Identity)?;
            Ok(store.begin_bound_attempt(
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
            )?)
        })??;
        if roster.attempt.generation != ordinal.to_string() {
            return Err(Error::Identity);
        }
        self.written(&roster.attempt.task_generation, event)?;
        self.attempts.push(Begun {
            id: roster.attempt.id,
            generation: roster.attempt.generation,
            // The first attempt's settle charges the preparation too (B14a-R2.5).
            began: if self.attempts.is_empty() {
                self.origin
            } else {
                began
            },
            applied: None,
            refused: None,
            settled: false,
            verified: false,
        });
        Ok(Attempt {
            index: self.attempts.len() - 1,
        })
    }

    fn execute(&mut self, attempt: &Self::Attempt) -> Result<Work, Self::Error> {
        let index = attempt.index;
        let id = self.begun(attempt)?.id.clone();
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
                    self.work_deadline(),
                );
                let cleanup_settled = match applied {
                    Ok(snapshot) => {
                        if let Some(begun) = self.attempts.get_mut(index) {
                            begun.applied = Some(snapshot);
                        }
                        true
                    }
                    Err(failure) => {
                        let removed = self.remove(&failure)?;
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
        let object = self
            .record(index, &check, &subject)?
            .ok_or(Error::Identity)?;
        if check.used_ms.is_none() || !check.cleanup_settled {
            return Ok(DriverChecked::Unsettled);
        }
        Ok(match check.verdict {
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
            VerificationVerdict::Cancelled => DriverChecked::Unsettled,
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
        // (b) An unsettled attempt stops nothing: its obligations and reservations stay (R1.4).
        if self.attempts.last().is_some_and(|begun| !begun.settled) {
            return Ok(false);
        }
        let cancelled = self
            .tasks
            .with_store(|store| self.current(store))??
            .cancellation;
        // (a) A cancellation with no verification of the last attempt records the check as not
        // started, at no cost, before the stop.
        let last = self.attempts.len().checked_sub(1);
        if cancelled
            && let Some(index) = last
            && let Some(begun) = self.attempts.get(index)
            && !begun.verified
        {
            let subject = match (&begun.applied, &begun.refused) {
                (Some(applied), _) => applied.content_digest().ok_or(Error::Identity)?,
                (None, Some((candidate, _))) => digest(candidate),
                (None, None) => self.digests[0].clone(),
            };
            let evidence = serde_json::to_vec(&serde_json::json!({
                "kind": "verification_not_started", "durable_cancellation": true,
            }))
            .map_err(|_| Error::Identity)?;
            let check = Check {
                verdict: VerificationVerdict::Cancelled,
                criteria: 0,
                evidence,
                used_ms: Some(0),
                cleanup_settled: true,
            };
            self.record(index, &check, &subject)?;
        }
        let name = stop_name(reason);
        let bytes = serde_json::to_vec(&serde_json::json!({
            "kind": "hee3.task-stop/1", "reason": name, "attempts": self.attempts.len(),
        }))
        .map_err(|_| Error::Identity)?;
        let (staging, event) = (fresh(self.deadline)?, fresh(self.deadline)?);
        let reason = Name::new(name).map_err(|_| Error::Identity)?;
        let preparation = self.attempts.is_empty().then(|| Settlement {
            effect: Effect::None,
            used_ms: Some(millis(self.origin.elapsed())),
            cleanup_settled: true,
            ready_to_verify: false,
        });
        let stopped = self.tasks.with_store(|store| -> Result<_, Error> {
            let head = self.current(store)?;
            let object = store.publish(&bytes, uuid(&staging)?, self.deadline)?;
            let stop = Stop {
                task: self.dispatch.task,
                generation: parse_generation(&head.generation)?,
                reason: &reason,
                evidence: &object,
                event: uuid(&event)?,
            };
            // (c) `finish_preparation` exactly when no attempt row exists.
            Ok(match preparation {
                Some(preparation) => store.finish_preparation(
                    self.dispatch.principal,
                    stop,
                    preparation,
                    self.deadline,
                )?,
                None => store.finish_unaccepted(self.dispatch.principal, stop, self.deadline)?,
            })
        })??;
        self.written(&stopped.generation, event)?;
        Ok(true)
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
