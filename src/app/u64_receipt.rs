//! The receipt composed over the check's records (B14a-2c; design R16 round 2 in
//! `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`).
//!
//! This file is the compiled skeleton (2c-i): the pure rules the design fixes before the composer is
//! assembled — how a run record is cited in a receipt graph, the RC04 verdict mapped to the ledger's
//! (kept UNWIRED until R16-G3: the ledger's verdict stays `live_verifier::checked`'s while `decide`
//! cannot pass), the strict-empty diagnostics rule, the eleven identities with the sources the class
//! fixes, and the ledger as an [`Objects`] owner so an accepted receipt can be walked.

use super::runtime::declared_criteria;
use crate::check::decision::{DiagnosticState, Identity, IdentityFact, IdentityState};
use crate::check::graph::{self, Objects};
use crate::contracts::receipt::{Name, Ref, VerdictV1State};
use crate::store::{Object, RunRecordKind, Store, VerificationVerdict};
use std::io::{Cursor, Read};
use std::time::Instant;

/// The prefix of the `ArtifactV1.role` under which a run record is cited: a `hee3.raw/1` reference
/// whose meaning the ledger's kind names (R16 round 2, decision 1 — a record's own schema id is not a
/// receipt schema and cannot appear in a `Ref`).
pub const RUN_RECORD_ROLE: &str = "run_record:";

/// The artifact role citing a run record of `kind`.
///
/// # Errors
/// Never in practice: every role is `RUN_RECORD_ROLE` plus a kind name, within `Name`'s bound.
pub fn record_role(kind: RunRecordKind) -> Result<Name, crate::contracts::receipt::Error> {
    Name::new(format!("{RUN_RECORD_ROLE}{}", kind.name()))
}

/// The kind an artifact role cites, if it is a run-record role.
#[must_use]
pub fn cited_kind(role: &str) -> Option<RunRecordKind> {
    let name = role.strip_prefix(RUN_RECORD_ROLE)?;
    RunRecordKind::ALL
        .into_iter()
        .find(|kind| kind.name() == name)
}

/// The ledger's verdict and criteria for an RC04 verdict state — one arm per state, so a new state
/// is a compile error here. Kept unwired (R16-G3): until every identity has a source and the
/// aggregate's settlement is observed, `decide` cannot pass and the ledger keeps the outcome-derived
/// verdict; this is what replaces it when the last gap closes.
#[must_use]
pub fn verdict_of(state: VerdictV1State) -> (VerificationVerdict, u64) {
    match state {
        VerdictV1State::PassCandidate => (VerificationVerdict::Passed, declared_criteria()),
        VerdictV1State::Fail => (VerificationVerdict::Failed, 0),
        VerdictV1State::Invalid => (VerificationVerdict::Invalid, 0),
        VerdictV1State::Error | VerdictV1State::Unmeasured => (VerificationVerdict::Error, 0),
        VerdictV1State::Timeout => (VerificationVerdict::Timeout, 0),
        VerdictV1State::Cancelled => (VerificationVerdict::Cancelled, 0),
    }
}

/// The reason the diagnostics baseline is asserted when the compile and link steps wrote nothing.
pub const STRICT_EMPTY_DIAGNOSTICS: &str = "strict_empty_diagnostics_observed";

/// The diagnostics fact for a run, by the strict-empty rule the 003 lane proved (R16 round 2,
/// decision 2): both toolchain steps' stderr empty is a clean baseline of zero; any bytes are an
/// unavailable count — there is no `rustc` diagnostic parser (R16-G2), and a guess is not a count.
#[must_use]
pub const fn diagnostics_of(
    compile_stderr_empty: bool,
    link_stderr_empty: bool,
) -> (bool, DiagnosticState) {
    if compile_stderr_empty && link_stderr_empty {
        (
            true,
            DiagnosticState::Complete {
                warnings: 0,
                errors: 0,
            },
        )
    } else {
        (false, DiagnosticState::Unavailable)
    }
}

/// What the runtime knows of each subject it can name (R16 round 2, decision 3): whether the
/// readback matched. `None` where the class fixes the subject but the runtime cannot read it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Readbacks {
    pub seed: Option<bool>,
    pub result: Option<bool>,
    pub fixtures: Option<bool>,
    pub oracle: Option<bool>,
    pub harness: Option<bool>,
    pub launcher: Option<bool>,
    pub toolchain: Option<bool>,
    pub profile: Option<bool>,
}

/// The eleven identity facts `decide` requires, in its declaration order: eight with a source the
/// class fixes, three (`Collector`, `Locks`, `Standards`) `Unavailable` by name until B14b and the
/// class profile supply them (R16-G3). A readback that matched is `Matched`, one that did not is
/// `Changed`, one the runtime could not perform is `Unavailable`.
#[must_use]
pub fn identities(readbacks: Readbacks) -> [IdentityFact; 11] {
    let state = |readback: Option<bool>| match readback {
        Some(true) => IdentityState::Matched,
        Some(false) => IdentityState::Changed,
        None => IdentityState::Unavailable,
    };
    [
        (Identity::Seed, readbacks.seed),
        (Identity::Result, readbacks.result),
        (Identity::Fixtures, readbacks.fixtures),
        (Identity::Oracle, readbacks.oracle),
        (Identity::Harness, readbacks.harness),
        (Identity::Collector, None),
        (Identity::Launcher, readbacks.launcher),
        (Identity::Locks, None),
        (Identity::Toolchain, readbacks.toolchain),
        (Identity::Profile, readbacks.profile),
        (Identity::Standards, None),
    ]
    .map(|(subject, readback)| IdentityFact {
        subject,
        state: state(readback),
    })
}

/// The ledger as an [`Objects`] owner (R16 round 2, decision 6): a reference resolves to the object
/// the content-addressed store holds under its digest and size — `read_object` verifies both against
/// the bytes, so a reference that lies is `Io` and a reference to nothing is `Missing`. Registration
/// in `artifacts` is not consulted: the walk that uses this compares what it finds against
/// `committed_check`, which is where the ledger's commitment is read.
pub struct LedgerObjects<'a> {
    store: &'a Store,
    deadline: Instant,
}

impl<'a> LedgerObjects<'a> {
    #[must_use]
    pub const fn new(store: &'a Store, deadline: Instant) -> Self {
        Self { store, deadline }
    }
}

impl Objects for LedgerObjects<'_> {
    fn open(&self, reference: &Ref) -> Result<Box<dyn Read + '_>, graph::Error> {
        let object = Object::of(reference.sha256.as_str(), u64::from(reference.byte_length));
        let bytes =
            self.store
                .read_object(&object, self.deadline)
                .map_err(|error| match error {
                    crate::store::Error::NotFound => graph::Error::Missing,
                    _ => graph::Error::Io,
                })?;
        Ok(Box::new(Cursor::new(bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Readbacks, STRICT_EMPTY_DIAGNOSTICS, cited_kind, diagnostics_of, identities, record_role,
        verdict_of,
    };
    use crate::app::runtime::declared_criteria;
    use crate::check::decision::{DiagnosticState, Identity, IdentityState};
    use crate::contracts::receipt::VerdictV1State;
    use crate::store::{RunRecordKind, VerificationVerdict};

    /// R16.1 · every kind's role round-trips, and a role that is not a run record's cites nothing.
    #[test]
    fn a_record_role_names_its_kind_and_nothing_else() -> Result<(), Box<dyn std::error::Error>> {
        for kind in RunRecordKind::ALL {
            let role = record_role(kind)?;
            assert_eq!(cited_kind(role.as_str()), Some(kind), "{role:?}");
        }
        for role in [
            "run_record:",
            "run_record:receipt",
            "capture:compile-library",
            "",
        ] {
            assert_eq!(cited_kind(role), None, "{role}");
        }
        assert_eq!(
            STRICT_EMPTY_DIAGNOSTICS,
            "strict_empty_diagnostics_observed"
        );
        Ok(())
    }

    /// R16.7 · the seven RC04 states, one ledger verdict each; criteria only on a pass.
    #[test]
    fn every_rc04_state_maps_to_one_ledger_verdict() {
        let expected = [
            (
                VerdictV1State::PassCandidate,
                VerificationVerdict::Passed,
                declared_criteria(),
            ),
            (VerdictV1State::Fail, VerificationVerdict::Failed, 0),
            (VerdictV1State::Invalid, VerificationVerdict::Invalid, 0),
            (VerdictV1State::Error, VerificationVerdict::Error, 0),
            (VerdictV1State::Timeout, VerificationVerdict::Timeout, 0),
            (VerdictV1State::Cancelled, VerificationVerdict::Cancelled, 0),
            (VerdictV1State::Unmeasured, VerificationVerdict::Error, 0),
        ];
        assert_eq!(expected.len(), 7, "every state, once");
        for (state, verdict, criteria) in expected {
            assert_eq!(verdict_of(state), (verdict, criteria), "{state:?}");
        }
        assert_ne!(declared_criteria(), 0);
    }

    /// R16.2 · the strict-empty rule: both empty is a clean zero; either with bytes is unavailable.
    #[test]
    fn diagnostics_are_a_clean_zero_only_when_both_stderrs_are_empty() {
        assert!(matches!(
            diagnostics_of(true, true),
            (
                true,
                DiagnosticState::Complete {
                    warnings: 0,
                    errors: 0
                }
            )
        ));
        for (compile, link) in [(false, true), (true, false), (false, false)] {
            assert!(matches!(
                diagnostics_of(compile, link),
                (false, DiagnosticState::Unavailable)
            ));
        }
    }

    /// R16.3 · eleven facts in `decide`'s order; three unavailable by name whatever was read back;
    /// two fixtures differing in every readback.
    #[test]
    fn identities_carry_eight_sources_and_name_three_gaps() {
        let all = Readbacks {
            seed: Some(true),
            result: Some(false),
            fixtures: Some(true),
            oracle: None,
            harness: Some(false),
            launcher: Some(true),
            toolchain: None,
            profile: Some(true),
        };
        let facts = identities(all);
        assert_eq!(facts.len(), 11);
        let states: Vec<(Identity, IdentityState)> = facts
            .iter()
            .map(|fact| (fact.subject, fact.state))
            .collect();
        assert_eq!(
            states,
            [
                (Identity::Seed, IdentityState::Matched),
                (Identity::Result, IdentityState::Changed),
                (Identity::Fixtures, IdentityState::Matched),
                (Identity::Oracle, IdentityState::Unavailable),
                (Identity::Harness, IdentityState::Changed),
                (Identity::Collector, IdentityState::Unavailable),
                (Identity::Launcher, IdentityState::Matched),
                (Identity::Locks, IdentityState::Unavailable),
                (Identity::Toolchain, IdentityState::Unavailable),
                (Identity::Profile, IdentityState::Matched),
                (Identity::Standards, IdentityState::Unavailable),
            ]
        );
        let none = identities(Readbacks::default());
        assert!(
            none.iter()
                .all(|fact| fact.state == IdentityState::Unavailable)
        );
        let every = identities(Readbacks {
            seed: Some(true),
            result: Some(true),
            fixtures: Some(true),
            oracle: Some(true),
            harness: Some(true),
            launcher: Some(true),
            toolchain: Some(true),
            profile: Some(true),
        });
        assert_eq!(
            every
                .iter()
                .filter(|fact| fact.state == IdentityState::Unavailable)
                .count(),
            3,
            "Collector, Locks and Standards have no source yet"
        );
    }
}
