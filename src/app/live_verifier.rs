//! The live verifier (B14a-3c, design R15 in `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`):
//! the one production [`super::runtime::Verifier`] — it runs the fixed workload against an applied candidate inside
//! the check window, captures every completed step, reads the outputs back and builds the four run
//! records the ledger commits with the verification (R13 ruling d).
//!
//! The verifier OBSERVES and the runtime RECORDS (R15 round 2): [`LiveVerifier`] runs the workload
//! inside the window it is handed and returns what it saw; `app::runtime` captures, reads back,
//! derives the check with [`checked`] and commits the records with the verification, inside one
//! hold. Two pure functions live here beside it: the class profile's declaration read as the
//! workload's [`Tools`], and the one derivation of a check's verdict, cost and settlement (R15.3).
//!
//! The aggregate slice the three scopes run under is the check's own (R21 S20a, R22-1): the
//! dispatcher composes the lifecycle ([`Manager`] over `worker::aggregate::Slice`, behind
//! [`Aggregates`]); each `check` prepares one slice named after its ledger attempt, creates it
//! under the check's cutoff, and hands it by value to `finish` under the teardown deadline on
//! every path that prepared one. Nothing is held between checks, no process but the workload's
//! scopes is placed in the slice, and serve is never moved.

use super::class_profile::Declared;
use super::evidence::fresh_id;
use super::run_records::OutcomeName;
use super::runtime::{CheckPlan, Observed, Resources, Unlaunched, Verifier, declared_criteria};
use super::workload::{self, COMPILER_DESTINATION, Plan, Run, Tools, collect_bounded};
use crate::contracts::UuidV4;
use crate::store::VerificationVerdict;
use crate::worker::aggregate::{self, Slice};
use crate::worker::namespace::{ReadOnlyFile, SHIM_DESTINATION};
use crate::worker::resources::{SYSTEMD_RUN, Scope};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

/// bwrap's fixed host path: the namespace door refuses any other
/// (`worker::namespace`), so the profile does not declare it.
pub const BWRAP: &str = "/usr/bin/bwrap";

/// The workload's tools from the class profile's declaration: a total conversion — every pin is
/// checked at use by its own door (the namespace compares each file's digest when it mounts it),
/// never here, so nothing is re-derived and nothing can disagree with the door.
#[must_use]
pub fn tools(declared: &Declared) -> Tools {
    Tools {
        bwrap: BWRAP.into(),
        compiler: ReadOnlyFile {
            host: declared.compiler.host.clone(),
            namespace: COMPILER_DESTINATION.into(),
            sha256: declared.compiler.sha256,
        },
        shim: ReadOnlyFile {
            host: declared.shim.host.clone(),
            namespace: SHIM_DESTINATION.into(),
            sha256: declared.shim.sha256,
        },
        runtime_files: declared
            .runtime_files
            .iter()
            .map(|file| ReadOnlyFile {
                host: file.host.clone(),
                namespace: file.namespace.clone(),
                sha256: file.sha256,
            })
            .collect(),
        namespace_directories: declared.namespace_directories.clone(),
    }
}

/// One check's aggregate slice (R21 S20a, R22-1): prepared, created and finished once per check.
pub trait Aggregates {
    /// What one check holds between its create and its finish; `finish` consumes it.
    type Held;
    /// Name the check's slice after the ledger `attempt` (R22 C1a); mutates nothing.
    ///
    /// # Errors
    /// The slice's refusal; nothing was asked of the manager, so there is nothing to finish.
    fn prepare(
        &mut self,
        attempt: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Self::Held, aggregate::Error>;
    /// Create the prepared slice and return its unit — the one name the check's scopes are built
    /// on, never re-formatted by the caller.
    ///
    /// # Errors
    /// The slice's refusal; `held` then says what `finish` must undo.
    fn create(
        &mut self,
        held: &mut Self::Held,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<String, aggregate::Error>;
    /// Stop what `held` created, consuming it: nothing outlives the check.
    ///
    /// # Errors
    /// The teardown's refusal, which the check records as its pending resources.
    fn finish(&mut self, held: Self::Held, deadline: Instant) -> Result<(), aggregate::Error>;
}

/// The production lifecycle over `worker::aggregate::Slice`: busctl at its fixed path under the
/// class's pin and the engine's own runtime directory. It holds nothing between checks.
#[derive(Debug)]
pub struct Manager {
    busctl_sha256: String,
    runtime_dir: PathBuf,
}

impl Manager {
    #[must_use]
    pub const fn new(busctl_sha256: String, runtime_dir: PathBuf) -> Self {
        Self {
            busctl_sha256,
            runtime_dir,
        }
    }
}

impl Aggregates for Manager {
    type Held = Slice;

    fn prepare(
        &mut self,
        attempt: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Slice, aggregate::Error> {
        Slice::prepare(
            aggregate::Config {
                busctl: aggregate::BUSCTL.into(),
                busctl_sha256: self.busctl_sha256.clone(),
                runtime_dir: self.runtime_dir.clone(),
            },
            attempt,
            deadline,
        )
    }

    fn create(
        &mut self,
        held: &mut Slice,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<String, aggregate::Error> {
        held.create(deadline, cancelled)?;
        Ok(held.unit().to_owned())
    }

    /// The slice's own teardown table (`Slice::stop`); a refusal is returned, and the slice goes
    /// with it — the next check prepares a fresh one whatever this one came to.
    fn finish(&mut self, mut held: Slice, deadline: Instant) -> Result<(), aggregate::Error> {
        held.stop(deadline)
    }
}

/// The production verifier: the fixed workload under three bounded scopes in the check's own
/// aggregate, run against the applied candidate inside the check window, returned as observed —
/// nothing is published or decided here. Its `Debug` is the whole composition: `open`'s pins are
/// asserted through it (R22 C7).
#[derive(Debug)]
pub struct LiveVerifier<A> {
    systemd_run_sha256: String,
    runtime_dir: PathBuf,
    aggregates: A,
}

impl<A: Aggregates> LiveVerifier<A> {
    /// Over the class's systemd-run pin, the engine's runtime directory and the aggregate lifecycle
    /// the dispatcher composes; the tools arrive with each check's plan, the same value the
    /// runtime's own plan described (R17 round 2, decision 2).
    #[must_use]
    pub const fn new(systemd_run_sha256: String, runtime_dir: PathBuf, aggregates: A) -> Self {
        Self {
            systemd_run_sha256,
            runtime_dir,
            aggregates,
        }
    }

    /// Three fresh ids under the cutoff (the three scopes'), the slice named after the plan's
    /// attempt (R22 C1a) prepared and created under the cutoff and the plan's own cancellation,
    /// the scopes built on the unit it returned. Returns the slice it prepared, if any, beside the
    /// run: a failed prepare holds nothing; a failed create holds what must be finished. An id
    /// that cannot be drawn is the workload's `Io`; the slice's refusal is named as its own.
    fn bounded(&mut self, plan: &CheckPlan<'_>) -> (Option<A::Held>, Result<Run, Unlaunched>) {
        let until = plan.window.until;
        let [first, second, third] = [(); 3].map(|()| fresh_id(until));
        let id = |drawn: Result<crate::contracts::receipt::Id, _>| {
            drawn
                .map(|id| id.as_str().to_owned())
                .map_err(|_| Unlaunched::Workload(workload::Error::Io))
        };
        let (first, second, third) = match (id(first), id(second), id(third)) {
            (Ok(first), Ok(second), Ok(third)) => (first, second, third),
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                return (None, Err(error));
            }
        };
        let mut held = match self.aggregates.prepare(plan.attempt, until) {
            Ok(held) => held,
            Err(error) => return (None, Err(Unlaunched::Aggregate(error))),
        };
        let unit = match self.aggregates.create(&mut held, until, plan.cancelled) {
            Ok(unit) => unit,
            Err(error) => return (Some(held), Err(Unlaunched::Aggregate(error))),
        };
        let scopes = [first, second, third].map(|run_id| Scope {
            systemd_run: SYSTEMD_RUN.into(),
            systemd_run_sha256: self.systemd_run_sha256.clone(),
            runtime_dir: self.runtime_dir.clone(),
            run_id,
            aggregate: unit.clone(),
        });
        let run = collect_bounded(
            &Plan {
                source: plan.subject,
                protected: plan.protected,
                job_root: plan.job_root,
                tools: plan.tools,
                deadline: until,
                teardown_deadline: plan.window.teardown_until,
                cancelled: plan.cancelled,
            },
            &scopes,
        )
        .map_err(Unlaunched::Workload);
        (Some(held), run)
    }
}

impl<A: Aggregates> Verifier for LiveVerifier<A> {
    /// The run is observed when the workload returns; the slice, if one was prepared, is
    /// finished after it under the teardown deadline, and its refusal is carried by name. A
    /// prepare that failed asked nothing of the manager: nothing to finish, settled.
    fn check(&mut self, plan: CheckPlan<'_>) -> Observed {
        let (held, run) = self.bounded(&plan);
        let observed = Instant::now();
        let resources = match held {
            None => Resources::Settled,
            Some(held) => match self.aggregates.finish(held, plan.window.teardown_until) {
                Ok(()) => Resources::Settled,
                // Named only in this run's RunCleanup record (`resources: pending`): the reclaimer
                // (B17) must read its population from the run records, since startup closes the
                // attempt on its leaves alone (B14b-2 review round 2, D7).
                Err(error) => Resources::Pending(error),
            },
        };
        Observed {
            run,
            observed,
            resources,
        }
    }
}

/// What the run's teardown settled, each predicate observed separately (never a verdict).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cleanup {
    /// Every process and FIFO the run owned is terminated.
    pub processes_settled: bool,
    /// The run's tmpfs descriptors are released.
    pub scratch_released: bool,
    /// The paths the run retained for capture are removed.
    pub retained_removed: bool,
    /// The check's aggregate as the verifier observed it (R21 S20a): only `Pending(_)` is unsettled.
    pub resources: Resources,
}

impl Cleanup {
    /// Settled only when every predicate holds.
    #[must_use]
    pub const fn settled(self) -> bool {
        self.processes_settled
            && self.scratch_released
            && self.retained_removed
            && !matches!(self.resources, Resources::Pending(_))
    }
}

/// What one check earned (R15.3): derived by this one function, so the store's row, the receipt
/// and the run records cannot disagree about the same run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Checked {
    pub verdict: VerificationVerdict,
    /// The criterion bits the check satisfied: the class's whole set on a match, none otherwise.
    pub criteria: u64,
    /// The check's cost, measured by the runtime from the window's origin — never asked of the run.
    pub used_ms: u64,
    pub cleanup_settled: bool,
}

/// The verdict, criteria, cost and settlement one run earned. One arm per outcome, so a new
/// outcome is a compile error here. `PendingCleanup` is an `Error` with the cleanup unsettled: the
/// workload replaces the steps' outcome when its teardown did not settle, so nothing earned
/// survives it (R15.3, amended at the skeleton). A run whose steps could not all be captured
/// (`captured == false`) earned nothing either: its evidence cannot cite what was not published.
#[must_use]
pub fn checked(
    outcome: OutcomeName,
    cleanup: Cleanup,
    captured: bool,
    begun: Instant,
    observed: Instant,
) -> Checked {
    let (verdict, criteria) = match outcome {
        _ if !captured => (VerificationVerdict::Error, 0),
        OutcomeName::Matched => (VerificationVerdict::Passed, declared_criteria()),
        OutcomeName::Mismatch | OutcomeName::InvalidOutput => (VerificationVerdict::Failed, 0),
        OutcomeName::Timeout => (VerificationVerdict::Timeout, 0),
        OutcomeName::Cancelled => (VerificationVerdict::Cancelled, 0),
        OutcomeName::InvalidSubject
        | OutcomeName::SetupFailed
        | OutcomeName::LauncherFailed
        | OutcomeName::ProducerFailed
        | OutcomeName::ProducerError
        | OutcomeName::PendingCleanup => (VerificationVerdict::Error, 0),
    };
    Checked {
        verdict,
        criteria,
        used_ms: u64::try_from(observed.saturating_duration_since(begun).as_millis())
            .unwrap_or(u64::MAX),
        cleanup_settled: cleanup.settled() && outcome != OutcomeName::PendingCleanup,
    }
}

#[cfg(test)]
mod tests {
    use super::{Aggregates, BWRAP, Checked, Cleanup, Manager, Resources, UuidV4, checked, tools};
    use crate::app::class_profile::{
        Declared, DeclaredFile, Effect, Grant, HostPin, Reviewed, RuntimeFile,
    };
    use crate::app::run_records::OutcomeName;
    use crate::app::runtime::declared_criteria;
    use crate::app::workload::COMPILER_DESTINATION;
    use crate::contracts::receipt::{self, Address, Id, Name, Ref, Sha, TypedRef};
    use crate::store::VerificationVerdict;
    use crate::worker::namespace::SHIM_DESTINATION;
    use std::time::{Duration, Instant};

    /// One reviewed reference of the fixture: a v4 id, a digest spelled as one byte repeated (a
    /// fixture spelling, not a hash; nothing in these tests reads a reviewed record), its schema.
    fn reference<T: Address>(id: &str, byte: u8) -> Result<TypedRef<T>, receipt::Error> {
        TypedRef::new(Ref {
            artifact_id: Id::new(id)?,
            sha256: Sha::new(format!("sha256:{}", format!("{byte:02x}").repeat(32)))?,
            byte_length: 1,
            media_type: Name::new("application/json")?,
            schema_id: Name::new(T::SCHEMA_ID)?,
        })
    }

    fn declared() -> Result<Declared, Box<dyn std::error::Error>> {
        Ok(Declared {
            workspaces: Vec::new(),
            compiler: HostPin {
                host: "/opt/toolchain/rustc".into(),
                sha256: [0xa1; 32],
            },
            shim: HostPin {
                host: "/opt/shim/namespace-shim".into(),
                sha256: [0xb2; 32],
            },
            runtime_files: vec![RuntimeFile {
                host: "/usr/lib64/libc.so.6".into(),
                namespace: "/lib64/libc.so.6".into(),
                sha256: [0xc3; 32],
            }],
            namespace_directories: vec!["/lib64".into(), "/usr/lib64".into()],
            busctl_sha256: format!("sha256:{}", "d".repeat(64)),
            systemd_run_sha256: format!("sha256:{}", "e".repeat(64)),
            reviewed: Reviewed::new(
                reference("28f70000-0000-4000-8000-00000000000e", 0xf4)?,
                reference("28f70000-0000-4000-8000-00000000000f", 0x05)?,
            )
            .map_err(|e| format!("{e:?}"))?,
            grant: Grant {
                grant_id: Id::new("28f90000-0000-4000-8000-000000000001")?,
                issuer_id: Name::new("operator")?,
                authority: DeclaredFile {
                    file: "authority.json".to_owned(),
                    sha256: Sha::new(format!("sha256:{}", "a".repeat(64)))?,
                },
            },
            effect: Effect {
                effect_id: Name::new("fixed-u64-workload-output")?,
                scope: crate::contracts::receipt::Text::new("the fixture's scope")?,
                specification: DeclaredFile {
                    file: "isolation.json".to_owned(),
                    sha256: Sha::new(format!("sha256:{}", "b".repeat(64)))?,
                },
            },
            native: None,
        })
    }

    /// R15 · the declaration read as tools, whole: every pin's host and digest, the two fixed
    /// destinations the profile does not declare, bwrap's fixed path, the directories in order.
    #[test]
    fn tools_carry_every_pin_to_its_fixed_destination() -> Result<(), Box<dyn std::error::Error>> {
        let declared = declared()?;
        let tools = tools(&declared);
        assert_eq!(tools.bwrap.as_os_str(), BWRAP);
        assert_eq!(
            (
                tools.compiler.host.as_os_str(),
                tools.compiler.namespace.as_os_str(),
                tools.compiler.sha256
            ),
            (
                declared.compiler.host.as_os_str(),
                std::ffi::OsStr::new(COMPILER_DESTINATION),
                [0xa1; 32]
            )
        );
        assert_eq!(
            (
                tools.shim.host.as_os_str(),
                tools.shim.namespace.as_os_str(),
                tools.shim.sha256
            ),
            (
                declared.shim.host.as_os_str(),
                std::ffi::OsStr::new(SHIM_DESTINATION),
                [0xb2; 32]
            )
        );
        assert_eq!(tools.runtime_files.len(), 1);
        assert_eq!(
            (
                tools.runtime_files[0].host.as_os_str(),
                tools.runtime_files[0].namespace.as_os_str(),
                tools.runtime_files[0].sha256
            ),
            (
                declared.runtime_files[0].host.as_os_str(),
                declared.runtime_files[0].namespace.as_os_str(),
                [0xc3; 32]
            )
        );
        assert_eq!(tools.namespace_directories, declared.namespace_directories);
        Ok(())
    }

    /// R15.3 · one arm per outcome: the verdict and criteria each earns, the cost measured from the
    /// window's origin, the settlement from the teardown's predicates — over two fixtures differing
    /// in every field, and the pending-cleanup outcome unsettled whatever the predicates say.
    #[test]
    fn a_check_is_derived_from_the_outcome_by_one_function() {
        let begun = Instant::now();
        let settled = Cleanup {
            processes_settled: true,
            scratch_released: true,
            retained_removed: true,
            resources: Resources::Settled,
        };
        let scratch_held = Cleanup {
            processes_settled: true,
            scratch_released: false,
            retained_removed: true,
            resources: Resources::Settled,
        };
        let at = |ms: u64| begun + Duration::from_millis(ms);
        let expected = [
            (
                OutcomeName::Matched,
                VerificationVerdict::Passed,
                declared_criteria(),
            ),
            (OutcomeName::Mismatch, VerificationVerdict::Failed, 0),
            (OutcomeName::InvalidOutput, VerificationVerdict::Failed, 0),
            (OutcomeName::InvalidSubject, VerificationVerdict::Error, 0),
            (OutcomeName::ProducerFailed, VerificationVerdict::Error, 0),
            (OutcomeName::ProducerError, VerificationVerdict::Error, 0),
            (OutcomeName::LauncherFailed, VerificationVerdict::Error, 0),
            (OutcomeName::Timeout, VerificationVerdict::Timeout, 0),
            (OutcomeName::Cancelled, VerificationVerdict::Cancelled, 0),
            (OutcomeName::PendingCleanup, VerificationVerdict::Error, 0),
            (OutcomeName::SetupFailed, VerificationVerdict::Error, 0),
        ];
        assert_eq!(expected.len(), 11, "every outcome, once");
        for (outcome, verdict, criteria) in expected {
            let pending = outcome == OutcomeName::PendingCleanup;
            assert_eq!(
                checked(outcome, settled, true, begun, at(47_977)),
                Checked {
                    verdict,
                    criteria,
                    used_ms: 47_977,
                    cleanup_settled: !pending,
                },
                "{outcome:?} settled"
            );
            assert_eq!(
                checked(outcome, scratch_held, true, begun, at(1_203)),
                Checked {
                    verdict,
                    criteria,
                    used_ms: 1_203,
                    cleanup_settled: false,
                },
                "{outcome:?} scratch held"
            );
        }
        for (label, cleanup) in [
            (
                "processes live",
                Cleanup {
                    processes_settled: false,
                    ..settled
                },
            ),
            (
                "retained paths kept",
                Cleanup {
                    retained_removed: false,
                    ..settled
                },
            ),
            (
                "aggregate held",
                Cleanup {
                    resources: Resources::Pending(crate::worker::aggregate::Error::Busy),
                    ..settled
                },
            ),
        ] {
            assert!(
                !checked(OutcomeName::Matched, cleanup, true, begun, at(5)).cleanup_settled,
                "{label}"
            );
        }
        assert_ne!(declared_criteria(), 0, "a match earns the class's bits");
        // A step that could not be captured: nothing earned, whatever the outcome said.
        assert_eq!(
            checked(OutcomeName::Matched, settled, false, begun, at(31)),
            Checked {
                verdict: VerificationVerdict::Error,
                criteria: 0,
                used_ms: 31,
                cleanup_settled: true,
            }
        );
    }

    /// R22-1 · the production lifecycle prepares nothing it cannot pin: a runtime directory that is
    /// not `/run/user/<euid>` is refused `Invalid` by the busctl door's pin before any call runs,
    /// in any environment — for each of two attempts, since the manager holds nothing between
    /// checks and the second prepare is refused the same way (never `State`).
    #[test]
    fn the_manager_prepares_nothing_under_an_unpinned_runtime_directory()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = Manager::new(
            format!("sha256:{}", "0".repeat(64)),
            "/run/user/not-a-uid".into(),
        );
        let deadline = Instant::now() + Duration::from_secs(1);
        for attempt in [
            "01234567-89ab-4cde-8f01-23456789abcd",
            "fedcba98-7654-4321-b0fe-dcba98765432",
        ] {
            let attempt = UuidV4::parse(attempt).map_err(|e| format!("{e:?}"))?;
            assert_eq!(
                manager.prepare(attempt, deadline).map(|_| ()),
                Err(crate::worker::aggregate::Error::Invalid)
            );
        }
        Ok(())
    }
}
