//! The live verifier (B14a-3c, design R15 in `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`):
//! the one production [`super::runtime::Verifier`] — it runs the fixed workload against an applied candidate inside
//! the check window, captures every completed step, reads the outputs back and builds the four run
//! records the ledger commits with the verification (R13 ruling d).
//!
//! This file is the compiled skeleton: the two pure functions the design fixes before the live half
//! is composed — the class profile's declaration read as the workload's [`Tools`], and the one
//! function that derives a check's verdict, cost and settlement from what the run reported (R15.3).

use super::class_profile::Declared;
use super::run_records::OutcomeName;
use super::runtime::declared_criteria;
use super::workload::{COMPILER_DESTINATION, Tools};
use crate::store::VerificationVerdict;
use crate::worker::namespace::{ReadOnlyFile, SHIM_DESTINATION};
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

/// What the run's teardown settled, each predicate observed separately (never a verdict).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cleanup {
    /// Every process and FIFO the run owned is terminated.
    pub processes_settled: bool,
    /// The run's tmpfs descriptors are released.
    pub scratch_released: bool,
    /// The paths the run retained for capture are removed.
    pub retained_removed: bool,
}

impl Cleanup {
    /// Settled only when every predicate holds.
    #[must_use]
    pub const fn settled(self) -> bool {
        self.processes_settled && self.scratch_released && self.retained_removed
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
/// survives it (R15.3, amended at the skeleton).
#[must_use]
pub fn checked(
    outcome: OutcomeName,
    cleanup: Cleanup,
    begun: Instant,
    observed: Instant,
) -> Checked {
    let (verdict, criteria) = match outcome {
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
    use super::{BWRAP, Checked, Cleanup, checked, tools};
    use crate::app::class_profile::{Declared, HostPin, Reviewed, RuntimeFile};
    use crate::app::run_records::OutcomeName;
    use crate::app::runtime::declared_criteria;
    use crate::app::workload::COMPILER_DESTINATION;
    use crate::store::VerificationVerdict;
    use crate::worker::namespace::SHIM_DESTINATION;
    use std::time::{Duration, Instant};

    fn declared() -> Declared {
        Declared {
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
            reviewed: Reviewed {
                expectation: [0xf4; 32],
                review: [0x05; 32],
            },
        }
    }

    /// R15 · the declaration read as tools, whole: every pin's host and digest, the two fixed
    /// destinations the profile does not declare, bwrap's fixed path, the directories in order.
    #[test]
    fn tools_carry_every_pin_to_its_fixed_destination() {
        let declared = declared();
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
        };
        let scratch_held = Cleanup {
            processes_settled: true,
            scratch_released: false,
            retained_removed: true,
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
                checked(outcome, settled, begun, at(47_977)),
                Checked {
                    verdict,
                    criteria,
                    used_ms: 47_977,
                    cleanup_settled: !pending,
                },
                "{outcome:?} settled"
            );
            assert_eq!(
                checked(outcome, scratch_held, begun, at(1_203)),
                Checked {
                    verdict,
                    criteria,
                    used_ms: 1_203,
                    cleanup_settled: false,
                },
                "{outcome:?} scratch held"
            );
        }
        assert_ne!(declared_criteria(), 0, "a match earns the class's bits");
    }
}
