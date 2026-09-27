//! The dispatcher (B14b-1; design R20 round 2 in `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`):
//! the first production caller of [`super::runtime::dispatch`]. One thread in `serve`, one task at a
//! time: it waits for the store's next dispatchable task under the store's own guard
//! ([`StoreTasks::wait_dispatchable`]), runs it through `dispatch`, and classifies EVERY result into
//! one of three steps so that no result can leave the picked task re-pickable at once — the hot
//! loop the round-1 review named is unrepresentable here, because the classification is an
//! exhaustive `match` with no catch-all arm and the read never returns a task that has an attempt row.
//!
//! B14b-2 composes the production provider in `serve` (`native_provider::NativeProvider`, its roster
//! record installed once at start) and records each attempt's root in the ledger at begin. The
//! drain does not reach an in-flight exchange or check — one waits for it under the attempt's own
//! deadline (R21 D8, reversing R20 round 2 D5: the wake is B19/B21's, the engine unit's stop
//! timeout APP-22's). With no provider installed the
//! dispatcher enters the named state `unavailable: no native provider`, reports it once, stops
//! picking and leaves the task `admitted` — P2c-R1.5's "stop it `dispatch_unavailable`" revisited:
//! the task is the owner's and the missing configuration the operator's.

use super::runtime::{
    Admission, Admitted, CandidateSource, Dispatch, Error as RuntimeError, Outcome, Verifier,
    admit, drive,
};
use super::tasks::StoreTasks;
use crate::contracts::UuidV4;
use crate::contracts::roster::Selection;
use crate::store::{Dispatchable, Error as StoreError, ResolveRefusal};
use crate::task::driver;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

/// Where the dispatcher's candidate source and verifier come from, per dispatch (B14b-1 D7).
pub trait Provider {
    type Source: CandidateSource;
    type Verifier: Verifier;
    /// Open the pair for one dispatch, or say by name why none is available. `admitted` is the
    /// dispatch's own admitted state (R21 N1): its window, baseline, drain and class profile — the
    /// provider computes no window of its own.
    ///
    /// # Errors
    /// [`Unavailable`], the dispatcher's named state.
    fn open(
        &mut self,
        next: &Dispatchable,
        admitted: &Admitted<'_>,
    ) -> Result<(Self::Source, Self::Verifier), Unavailable>;
}

/// Why the dispatcher cannot dispatch: a named dispatcher state, never a task's failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unavailable {
    /// No native provider is configured (B14b-2).
    NoNativeProvider,
    /// The task owner has no class profile (not installed, or refused at read).
    NoClassProfile,
    /// The class declares no native model row (R21 D1): the native provider does not serve it.
    ClassNotNative,
    /// The operator's native file does not fit the class, by the declaration that does not.
    Native(NativeWhy),
    /// The daemon could not be resolved (R21 N6, N7): the resolver's refusal.
    Daemon(crate::worker::native::Error),
    /// The reviewed closure did not yield the class's candidate inputs (R21 N22).
    Closure,
    /// The class prompt refused an input, by the class profile's name for it (R21 N22).
    Prompt(super::candidates::ClassPromptError),
}

/// Which of the operator's native declarations does not fit the class (R21 D1, N11).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWhy {
    /// The operator's manifest pin is not the one the class's native row names.
    Manifest,
    /// The class's adapter row is not one this build knows.
    Adapter,
    /// The client's working directory is not canonical, the engine's, and 0700.
    Directory,
}

impl Unavailable {
    /// The state's name, whole: one literal per variant and payload, so a new one is a compile
    /// error here.
    #[must_use]
    pub const fn name(self) -> &'static str {
        use super::candidates::ClassPromptError as Input;
        use crate::worker::native::Error as Native;
        match self {
            Self::NoNativeProvider => "unavailable: no native provider (B14b-2)",
            Self::NoClassProfile => "unavailable: no class profile",
            Self::ClassNotNative => "unavailable: class declares no native model",
            Self::Native(NativeWhy::Manifest) => "unavailable: native manifest",
            Self::Native(NativeWhy::Adapter) => "unavailable: native adapter",
            Self::Native(NativeWhy::Directory) => "unavailable: native directory",
            Self::Daemon(Native::Profile) => "unavailable: daemon profile",
            Self::Daemon(Native::Subject) => "unavailable: daemon subject",
            Self::Daemon(Native::Deadline) => "unavailable: daemon deadline",
            Self::Daemon(Native::Cancelled) => "unavailable: daemon cancelled",
            Self::Daemon(Native::Json) => "unavailable: daemon json",
            Self::Daemon(Native::Identity) => "unavailable: daemon identity",
            Self::Daemon(Native::Usage) => "unavailable: daemon usage",
            Self::Daemon(Native::Response) => "unavailable: daemon response",
            Self::Daemon(Native::Process) => "unavailable: daemon process",
            Self::Daemon(Native::Contract(_)) => "unavailable: daemon contract",
            Self::Daemon(Native::Census(_)) => "unavailable: daemon census",
            Self::Daemon(Native::Candidates(_)) => "unavailable: daemon candidates",
            Self::Closure => "unavailable: reviewed closure",
            Self::Prompt(Input::Task) => "unavailable: prompt task",
            Self::Prompt(Input::Cargo) => "unavailable: prompt cargo",
            Self::Prompt(Input::Base) => "unavailable: prompt base",
        }
    }
}

/// The production provider until B14b-2 lands: none.
pub struct NoProvider;

/// A source that can never be asked: `NoProvider::open` refuses before one is built.
pub struct Never;

impl CandidateSource for Never {
    fn next(&mut self, _ask: &super::runtime::Ask<'_>) -> super::runtime::Answer {
        super::runtime::Answer {
            candidate: super::runtime::Candidate::Exhausted,
            settle: None,
        }
    }
    fn ready(
        &mut self,
        _deadline: Instant,
        _cancelled: &AtomicBool,
    ) -> Result<super::runtime::Readiness, crate::worker::native::Error> {
        Err(crate::worker::native::Error::Profile)
    }
    fn settle_retained(
        &mut self,
        _deadline: Instant,
        _cancelled: &AtomicBool,
    ) -> super::runtime::Custody {
        super::runtime::Custody::default()
    }
}

impl Verifier for Never {
    fn check(&mut self, plan: super::runtime::CheckPlan<'_>) -> super::runtime::Observed {
        super::runtime::Observed {
            run: Err(super::workload::Error::Layout),
            observed: plan.window.begun,
            resources: super::runtime::Resources::NotHeld,
        }
    }
}

impl Provider for NoProvider {
    type Source = Never;
    type Verifier = Never;
    fn open(
        &mut self,
        _next: &Dispatchable,
        _admitted: &Admitted<'_>,
    ) -> Result<(Never, Never), Unavailable> {
        Err(Unavailable::NoNativeProvider)
    }
}

/// What one dispatch came to, for the loop: exactly three, so the loop cannot be written wrong.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Step {
    /// The task is no longer pickable: refused or stopped by name, or accepted.
    TaskDone(&'static str),
    /// An attempt row exists and the task is the recovery's (B17/B21): the read never returns it.
    TaskLeft(&'static str),
    /// The dispatcher stops, by name.
    DispatcherStops(&'static str),
}

/// The exhaustive classification (R20 round 2 D3): every `Outcome`, every `RuntimeError` variant and
/// every `StoreError` kind it can carry — no catch-all arm, so a new variant is a compile error here.
#[must_use]
pub fn classify(result: &Result<Outcome, RuntimeError>) -> Step {
    match result {
        Ok(Outcome::Refused(refusal)) => Step::TaskDone(refusal.name()),
        // Drained at attempt 1 the task is still `admitted` and picked again; at attempt ≥ 2 it is
        // `repair_pending`, which the read never returns — left for recovery (B17; closure item 9).
        Ok(Outcome::Drained) => Step::TaskLeft("drained: left for recovery"),
        // The source refused to be made ready (R21 N4): nothing written for the attempt. At attempt
        // 1 the task stays `admitted` and a next read would return it at once, so the dispatcher
        // stops by name (the `open` refusal's rule, N2); at attempt ≥ 2 the task is the recovery's.
        Ok(Outcome::NotReady(_)) => Step::DispatcherStops("provider not ready"),
        Ok(Outcome::Driven(driver::Outcome::Accepted)) => Step::TaskDone("accepted"),
        Ok(Outcome::Driven(driver::Outcome::Stopped(_))) => Step::TaskDone("stopped"),
        Ok(Outcome::Driven(driver::Outcome::NeedsSettlement(_))) => {
            Step::TaskLeft("needs settlement")
        }
        Err(RuntimeError::Poisoned) => Step::DispatcherStops("the ledger's owner panicked"),
        Err(RuntimeError::PreDispatch(_)) => {
            Step::DispatcherStops("an error before any attempt row: the dispatcher's")
        }
        Err(RuntimeError::ConcurrentWriter) => Step::TaskLeft("concurrent writer"),
        Err(RuntimeError::Identity) => Step::TaskLeft("identity"),
        Err(RuntimeError::Entropy) => Step::TaskLeft("entropy"),
        Err(RuntimeError::Policy(_)) => Step::TaskLeft("policy"),
        Err(RuntimeError::Store(error)) => store_step(error),
    }
}

/// A store error out of `dispatch` after an attempt row may exist: the task is the recovery's, unless
/// the ledger itself can no longer be written (the dispatcher stops), its storage is full, or the
/// object inventory is full (DS17: `Disposition(Inventory)`, raised by the registration door at the
/// first registration past the bound and by the acceptance door when the accepted objects would
/// cross it — closure H2 and closure 2; `Full` is `SQLITE_FULL`, its own name).
fn store_step(error: &StoreError) -> Step {
    match error {
        StoreError::UncertainCommit => {
            Step::DispatcherStops("uncertain commit: the ledger is poisoned")
        }
        StoreError::Full => Step::DispatcherStops("the ledger's storage is full (SQLITE_FULL)"),
        StoreError::Disposition(ResolveRefusal::Inventory) => {
            Step::DispatcherStops("store object inventory full (DS17)")
        }
        StoreError::NotWritable | StoreError::RecoveryRequired | StoreError::InspectionOnly => {
            Step::DispatcherStops("the ledger is not writable")
        }
        StoreError::Invalid
        | StoreError::Forbidden
        | StoreError::Bound
        | StoreError::Conflict
        | StoreError::NotFound
        | StoreError::Cancelled
        | StoreError::Outstanding
        | StoreError::Budget
        | StoreError::Deadline
        | StoreError::Locked
        | StoreError::Custody
        | StoreError::TaskViewBound { .. }
        | StoreError::StartupBound { .. }
        | StoreError::StaleGeneration { .. }
        | StoreError::SnapshotAhead { .. }
        | StoreError::SnapshotMoved { .. }
        | StoreError::Disposition(_)
        | StoreError::EvidenceIdentity
        | StoreError::EvidenceBound { .. }
        | StoreError::EventsBound { .. }
        | StoreError::AlreadyStopped
        | StoreError::Corrupt
        | StoreError::UnsupportedSchema
        | StoreError::Chain(_)
        | StoreError::UpgradeRequired { .. }
        | StoreError::Runtime
        | StoreError::Constraint
        | StoreError::Cleanup { .. }
        | StoreError::Rollback { .. }
        | StoreError::Io(_)
        | StoreError::Os(_)
        | StoreError::Sqlite(_)
        | StoreError::Encoding(_) => Step::TaskLeft("store refusal after begin"),
        #[cfg(test)]
        StoreError::Injected(_) => Step::TaskLeft("injected"),
    }
}

/// Why the dispatcher stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Exit {
    /// The drain ended the wait.
    Drained,
    /// A provider could not be opened.
    Unavailable(Unavailable),
    /// A result the classification named as the dispatcher's own stop.
    Stopped(&'static str),
    /// The ledger's owner panicked.
    Poisoned,
}

/// The teardown share every dispatch is handed (M8: derived, never a new limit): the cleanup grace
/// the check already keeps back. The capture share is the owner's reservation less this, derived in
/// `prepare` (closure M1) — nothing here fixes it.
#[must_use]
pub fn teardown_share() -> u64 {
    u64::try_from(super::runtime::CHECK_TEARDOWN.as_millis()).unwrap_or(u64::MAX)
}

/// One dispatcher over one task owner: what every dispatch is handed, held together so the loop
/// takes one value (the eight loose arguments the first cut had were the lint's point).
pub struct Dispatcher<'a, P> {
    /// The task owner the socket serves: the one ledger and the one class profile (A11 — the
    /// profile is read from it inside `run`, never a second value).
    pub tasks: &'a StoreTasks,
    /// The attempts root under the state root (`coordinator::attempts_root`).
    pub attempts: &'a Path,
    pub provider: &'a mut P,
    /// The roster inputs a begin needs (B14a-1c): the rig's in the proofs; in production the native
    /// install's record and selection (R21 N3), fixed at `serve` start — empty with no provider,
    /// when no task reaches `begin`.
    pub agent_record_id: &'a str,
    pub selections: &'a [Selection],
    /// The engine's drain (`Drain::flag`): ends the wait, and is read by the runtime between attempts.
    pub drain: &'a AtomicBool,
}

impl<P: Provider> Dispatcher<'_, P> {
    /// Run until the dispatcher stops; `report` receives one line per step and the exit.
    pub fn run(self, report: &(dyn Fn(&str) + Sync)) -> Exit {
        run(self, report)
    }
}

fn run<P: Provider>(dispatcher: Dispatcher<'_, P>, report: &(dyn Fn(&str) + Sync)) -> Exit {
    let Dispatcher {
        tasks,
        attempts,
        provider,
        agent_record_id,
        selections,
        drain,
    } = dispatcher;
    let Ok(profile) = tasks.class_profile() else {
        report(&format!(
            "dispatcher: {}",
            Unavailable::NoClassProfile.name()
        ));
        return Exit::Unavailable(Unavailable::NoClassProfile);
    };
    let teardown_ms = teardown_share();
    let stopped = || drain.load(std::sync::atomic::Ordering::SeqCst);
    // The task the last step handled: a read that returns it again is a defect in the rules above
    // (a handled task is stopped, accepted, or has an attempt row), stopped by name, never looped.
    let mut handled: Option<String> = None;
    loop {
        let next = match tasks.wait_dispatchable(&stopped) {
            Err(super::tasks::Poisoned) => return Exit::Poisoned,
            Ok(Err(error)) => {
                report(&format!("dispatcher: the read refused ({error:?})"));
                return Exit::Stopped("the dispatch read refused");
            }
            Ok(Ok(None)) => return Exit::Drained,
            Ok(Ok(Some(next))) => next,
        };
        if handled.as_deref() == Some(next.task.as_str()) {
            report(&format!(
                "dispatcher: the read returned the task just handled ({}): stopping",
                next.task
            ));
            return Exit::Stopped("the read returned the task just handled");
        }
        let Ok(task) = UuidV4::parse(&next.task) else {
            report(&format!(
                "dispatcher: a stored task id is not a UuidV4 ({})",
                next.task
            ));
            return Exit::Stopped("a stored task id is malformed");
        };
        // Phase one before any provider (R20 round 2 A2/A4): the free checks and the captures; a
        // refusal stops the task by name here, and a provider is opened only for an admitted task.
        let admitted = admit(
            tasks,
            profile,
            Dispatch {
                principal: &next.owner,
                task,
                agent_record_id,
                selections,
                attempts,
                forbidden: &[],
                teardown_ms,
                capture_ms: None,
                drain,
            },
        );
        // The custody a driven dispatch's retained children came to (R21 N18), reported by name.
        let (result, custody) = match admitted {
            Err(error) => (Err(error), None),
            Ok(Admission::Refused(refusal)) => (Ok(Outcome::Refused(refusal)), None),
            Ok(Admission::Ready(ready)) => match provider.open(&next, &ready) {
                Err(why) => {
                    report(&format!("dispatcher: {}", why.name()));
                    return Exit::Unavailable(why);
                }
                Ok((mut source, mut verifier)) => {
                    match drive(tasks, *ready, &mut source, &mut verifier) {
                        Ok(driven) => (Ok(driven.outcome), Some(driven.custody)),
                        Err(error) => (Err(error), None),
                    }
                }
            },
        };
        let step = classify(&result);
        report(&format!(
            "dispatcher: task {} -> {step:?}{}{}",
            next.task,
            result
                .as_ref()
                .err()
                .map(|error| format!(" ({error:?})"))
                .unwrap_or_default(),
            custody
                .map(|custody| format!(
                    ", custody: settled={} pending={}",
                    custody.settled, custody.pending
                ))
                .unwrap_or_default()
        ));
        handled = Some(next.task.clone());
        if let Step::DispatcherStops(why) = step {
            return Exit::Stopped(why);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NativeWhy, Step, Unavailable, classify, teardown_share};
    use crate::app::candidates::ClassPromptError;
    use crate::app::runtime::{Error as RuntimeError, Outcome, Refusal};
    use crate::store::{Error as StoreError, ResolveRefusal};
    use crate::task::LoopRefusal;
    use crate::task::driver::{self, StopReason};
    use crate::worker::native::Error as Native;

    /// Every result the classification is pinned over, with its step.
    fn cases() -> Vec<(Result<Outcome, RuntimeError>, Step)> {
        vec![
            (
                Ok(Outcome::Refused(Refusal::CancelledBeforeDispatch)),
                Step::TaskDone("cancelled_before_dispatch"),
            ),
            (
                Ok(Outcome::Drained),
                Step::TaskLeft("drained: left for recovery"),
            ),
            (
                Ok(Outcome::NotReady(crate::worker::native::Error::Identity)),
                Step::DispatcherStops("provider not ready"),
            ),
            (
                Ok(Outcome::Driven(driver::Outcome::Accepted)),
                Step::TaskDone("accepted"),
            ),
            (
                Ok(Outcome::Driven(driver::Outcome::Stopped(
                    StopReason::WorkerFailed,
                ))),
                Step::TaskDone("stopped"),
            ),
            (
                Ok(Outcome::Driven(driver::Outcome::NeedsSettlement(
                    StopReason::Unsettled,
                ))),
                Step::TaskLeft("needs settlement"),
            ),
            (
                Err(RuntimeError::Poisoned),
                Step::DispatcherStops("the ledger's owner panicked"),
            ),
            (
                Err(RuntimeError::PreDispatch(Box::new(RuntimeError::Store(
                    StoreError::NotFound,
                )))),
                Step::DispatcherStops("an error before any attempt row: the dispatcher's"),
            ),
            (
                Err(RuntimeError::PreDispatch(Box::new(RuntimeError::Entropy))),
                Step::DispatcherStops("an error before any attempt row: the dispatcher's"),
            ),
            (
                Err(RuntimeError::ConcurrentWriter),
                Step::TaskLeft("concurrent writer"),
            ),
            (Err(RuntimeError::Identity), Step::TaskLeft("identity")),
            (Err(RuntimeError::Entropy), Step::TaskLeft("entropy")),
            (
                Err(RuntimeError::Policy(LoopRefusal::GenerationExhausted)),
                Step::TaskLeft("policy"),
            ),
            (
                Err(RuntimeError::Store(StoreError::UncertainCommit)),
                Step::DispatcherStops("uncertain commit: the ledger is poisoned"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Full)),
                Step::DispatcherStops("the ledger's storage is full (SQLITE_FULL)"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Disposition(
                    ResolveRefusal::Inventory,
                ))),
                Step::DispatcherStops("store object inventory full (DS17)"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Disposition(
                    ResolveRefusal::Resolved,
                ))),
                Step::TaskLeft("store refusal after begin"),
            ),
            (
                Err(RuntimeError::Store(StoreError::NotWritable)),
                Step::DispatcherStops("the ledger is not writable"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Conflict)),
                Step::TaskLeft("store refusal after begin"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Corrupt)),
                Step::TaskLeft("store refusal after begin"),
            ),
        ]
    }

    /// R20 round 2 D3 · every result lands in exactly one of three steps, by a table: a refusal and
    /// every driven outcome; each runtime error; the store kinds that stop the dispatcher against
    /// the kinds that leave the task to recovery. A new variant is a compile error in `classify`.
    #[test]
    fn every_dispatch_result_is_classified_into_one_of_three_steps() {
        let cases = cases();
        for (result, expected) in &cases {
            assert_eq!(classify(result), *expected, "{result:?}");
        }
        assert_eq!(
            Unavailable::NoNativeProvider.name(),
            "unavailable: no native provider (B14b-2)"
        );
        // The teardown share is the check's grace, derived; the capture share is not fixed here.
        assert_eq!(
            u128::from(teardown_share()),
            crate::app::runtime::CHECK_TEARDOWN.as_millis()
        );
        assert_eq!(
            Unavailable::NoClassProfile.name(),
            "unavailable: no class profile"
        );
        // R21 D2 · the native provider's states, every variant and payload by its whole name.
        let native = |why| Unavailable::Daemon(why);
        let names = [
            (
                Unavailable::ClassNotNative,
                "unavailable: class declares no native model",
            ),
            (
                Unavailable::Native(NativeWhy::Manifest),
                "unavailable: native manifest",
            ),
            (
                Unavailable::Native(NativeWhy::Adapter),
                "unavailable: native adapter",
            ),
            (
                Unavailable::Native(NativeWhy::Directory),
                "unavailable: native directory",
            ),
            (native(Native::Profile), "unavailable: daemon profile"),
            (native(Native::Subject), "unavailable: daemon subject"),
            (native(Native::Deadline), "unavailable: daemon deadline"),
            (native(Native::Cancelled), "unavailable: daemon cancelled"),
            (native(Native::Json), "unavailable: daemon json"),
            (native(Native::Identity), "unavailable: daemon identity"),
            (native(Native::Usage), "unavailable: daemon usage"),
            (native(Native::Response), "unavailable: daemon response"),
            (native(Native::Process), "unavailable: daemon process"),
            (
                native(Native::Contract(crate::worker::ContractError::Identity)),
                "unavailable: daemon contract",
            ),
            (
                native(Native::Census(
                    crate::worker::process::CensusError::Deadline,
                )),
                "unavailable: daemon census",
            ),
            (
                native(Native::Candidates(
                    crate::worker::process::DescendantBound {
                        found: 65,
                        limit: 64,
                    },
                )),
                "unavailable: daemon candidates",
            ),
            (Unavailable::Closure, "unavailable: reviewed closure"),
            (
                Unavailable::Prompt(ClassPromptError::Task),
                "unavailable: prompt task",
            ),
            (
                Unavailable::Prompt(ClassPromptError::Cargo),
                "unavailable: prompt cargo",
            ),
            (
                Unavailable::Prompt(ClassPromptError::Base),
                "unavailable: prompt base",
            ),
        ];
        for (state, name) in names {
            assert_eq!(state.name(), name, "{state:?}");
        }
    }
}
