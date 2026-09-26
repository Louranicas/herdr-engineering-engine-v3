//! The receipt composed over the check's records (B14a-2c; design R16 round 2 in
//! `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`).
//!
//! This file is the compiled skeleton (2c-i): the pure rules the design fixes before the composer is
//! assembled — how a run record is cited in a receipt graph, the RC04 verdict mapped to the ledger's
//! (kept UNWIRED until R16-G3: the ledger's verdict stays `live_verifier::checked`'s while `decide`
//! cannot pass), the strict-empty diagnostics rule, the eleven identities with the sources the class
//! fixes, and the ledger as an [`Objects`] owner so an accepted receipt can be walked.

use super::capture::{Captured, MAX_RAW_BYTES};
use super::evidence::Evidence;
use super::run_records::{OutcomeName, RunCleanup, RunClock, Settlement as RecordSettlement};
use super::runtime::{CheckWindow, declared_criteria};
use crate::check::collector::{self, CaseObservation, Observed, Publisher, VerdictBinding};
use crate::check::consistency::{self, Prepared};
use crate::check::decision::{
    self, CheckerFact, CleanupFacts, Decision, DiagnosticPolicy, DiagnosticState, Diagnostics,
    EvidenceState, Identity, IdentityFact, IdentityState, IncompleteCause, LogState, OracleFact,
    ProcessFact, ProcessState, Streams, Termination,
};
use crate::check::graph::{self, Objects};
use crate::contracts::receipt::{
    ArtifactV1, ArtifactV1Availability, AvailabilityV1, AvailabilityV1RetentionPolicy,
    AvailabilityV1State, CaseV1Outcome, CleanupContractV1, EnvironmentV1, ExpectedProducerV1,
    ExpectedProducerV1Status, HostV1, Id, LimitsV1, List, Maybe, Name, ObligationPageV1,
    ObligationV1, ObligationV1State, ObservationsV1, ObservationsV1Cancellation,
    ObservationsV1Cleanup, OracleResultV1, OracleResultV1Result, Payload, ProducerV1,
    ProducerV1Status, ReceiptV1, Ref, Sha, Text, TypedRef, U64, VerdictV1State,
};
use crate::store::{Object, RunRecordKind, Store, VerificationVerdict};
use crate::worker::host;
use crate::worker::namespace::{MAX_CHANNEL, SCRATCH_BYTES};
use crate::worker::namespace_shim::ENVIRONMENT;
use crate::worker::resources::{ATTEMPT_LIMITS, TERM_GRACE};

/// The owner every runtime obligation and the cleanup contract name.
pub const RUNTIME_OWNER: &str = "hee3.runtime";
/// The four obligations the runtime's cleanup record settles, in its order.
pub const OBLIGATIONS: [&str; 4] = ["process", "scratch", "retained_paths", "aggregate"];
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
        .find(|kind| name.starts_with(kind.name()) || kind.name().starts_with(name))
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

/// The invocation's environment rows: the shim's own table (`namespace_shim::ENVIRONMENT`), one
/// row each, present values — the receipt and the namespace read one constant (R16 round 2, 9).
///
/// # Errors
/// Never in practice: every name and value is within the receipt's bounds.
pub fn environment_rows() -> Result<Vec<EnvironmentV1>, crate::contracts::receipt::Error> {
    ENVIRONMENT
        .into_iter()
        .map(|(name, value)| {
            Ok(EnvironmentV1 {
                name: Name::new(name)?,
                value: Maybe::present(Text::new(value)?),
                secret_handle: Maybe::unavailable(Text::new("no_secret")?),
            })
        })
        .collect()
}

/// Why the receipt's currency is unavailable: the class allows no external effect, so its cost
/// is zero and has no unit.
pub const ZERO_EXTERNAL_COST: &str = "zero_external_cost";

/// The invocation's limits, each from the door that enforces it (R16 round 2, decision 9): the
/// scope's cgroup limits (`ATTEMPT_LIMITS`), the channel and scratch bounds the namespace refuses
/// past, the capture's per-stream bound as the largest artifact, the stop grace, and the check's
/// own window (`wall_ms`) and cleanup deadline (`cleanup_deadline_ms`, equal to the whole check
/// deadline as the 003 lane recorded the task's). `external_requests` is zero because the namespace
/// runs `--unshare-all`; `compiler_jobs` and the thread counts are the environment's hints, not
/// enforced limits (R11 HIGH-2, recorded).
///
/// # Errors
/// `Scalar` when a value has no decimal rendering the receipt admits (never for these constants).
pub fn limits(
    wall_ms: u64,
    cleanup_deadline_ms: u64,
) -> Result<LimitsV1, crate::contracts::receipt::Error> {
    let decimal = |value: u64| U64::new(value.to_string());
    let count =
        |value: u64| u32::try_from(value).map_err(|_| crate::contracts::receipt::Error::Scalar);
    Ok(LimitsV1 {
        wall_ms: decimal(wall_ms)?,
        memory_bytes: decimal(ATTEMPT_LIMITS.memory_bytes)?,
        memory_swap_bytes: decimal(ATTEMPT_LIMITS.swap_bytes)?,
        scratch_bytes: decimal(SCRATCH_BYTES)?,
        stdout_bytes: decimal(MAX_CHANNEL as u64)?,
        stderr_bytes: decimal(MAX_CHANNEL as u64)?,
        artifact_bytes: decimal(MAX_RAW_BYTES as u64)?,
        external_requests: decimal(0)?,
        external_cost_microunits: decimal(0)?,
        term_grace_ms: decimal(u64::try_from(TERM_GRACE.as_millis()).unwrap_or(u64::MAX))?,
        cleanup_deadline_ms: decimal(cleanup_deadline_ms)?,
        cpu_quota_percent: count(ATTEMPT_LIMITS.cpu_percent)?,
        tasks_max: count(ATTEMPT_LIMITS.tasks)?,
        compiler_jobs: 2,
        julia_threads: 1,
        blas_threads: 1,
        currency: Maybe::unavailable(Text::new(ZERO_EXTERNAL_COST)?),
    })
}

/// The receipt's host record over the facts read (R16 round 2, decision 3) and the payload the raw
/// readings were published as.
///
/// # Errors
/// `Scalar`/`Bound` when a reading is not a receipt name (an os id past 128 bytes, a kernel line
/// past 4096).
pub fn host_record(
    facts: &host::Facts,
    raw: Payload,
) -> Result<HostV1, crate::contracts::receipt::Error> {
    Ok(HostV1 {
        os: Name::new(facts.os.as_str())?,
        release: Name::new(facts.release.as_str())?,
        architecture: Name::new(facts.architecture.as_str())?,
        kernel: Text::new(facts.kernel.as_str())?,
        boot_id: Name::new(facts.boot_id.as_str())?,
        logical_cpus: facts.logical_cpus,
        memory_bytes: U64::new(facts.memory_bytes.to_string())?,
        facts: raw,
    })
}

/// The cleanup contract's obligations as the pre-execution rows the receipt carries: the runtime's
/// four, `open`, material, owned by [`RUNTIME_OWNER`], each citing the readback specification
/// (R16 round 2, decision 9). `ids` are the fresh obligation ids, one per row.
///
/// # Errors
/// `Scalar` for an id that is not a UUID; `Bound` past the list's size (never for four).
pub fn obligation_rows(
    ids: &[String; 4],
    readback_specification: &Ref,
) -> Result<Vec<ObligationV1>, crate::contracts::receipt::Error> {
    OBLIGATIONS
        .into_iter()
        .zip(ids)
        .map(|(scope, id)| {
            Ok(ObligationV1 {
                obligation_id: Id::new(id.as_str())?,
                owner_id: Name::new(RUNTIME_OWNER)?,
                scope: Text::new(scope)?,
                material: true,
                state: ObligationV1State::Open,
                evidence: List::new(vec![readback_specification.clone()])?,
                reason: Text::new("settled by the runtime's cleanup record")?,
            })
        })
        .collect()
}

/// The cleanup contract the check runs under (R16 round 2, decision 9): the stop grace, the whole
/// check deadline (as the 003 lane recorded the task's), every descendant reaped, the obligation
/// page, and the readback specification as its payload.
///
/// # Errors
/// `Scalar` when a value has no decimal rendering the receipt admits.
pub fn cleanup_contract(
    deadline_ms: u64,
    obligations: TypedRef<ObligationPageV1>,
    readback_specification: Payload,
) -> Result<CleanupContractV1, crate::contracts::receipt::Error> {
    Ok(CleanupContractV1 {
        owner_id: Name::new(RUNTIME_OWNER)?,
        term_grace_ms: U64::new(
            u64::try_from(TERM_GRACE.as_millis())
                .unwrap_or(u64::MAX)
                .to_string(),
        )?,
        deadline_ms: U64::new(deadline_ms.to_string())?,
        require_empty_descendants: true,
        obligations,
        readback_specification,
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
        Composing, OBLIGATIONS, RUNTIME_OWNER, Readbacks, Refusal, STRICT_EMPTY_DIAGNOSTICS,
        ZERO_EXTERNAL_COST, cited_kind, cleanup_contract, compose, diagnostics_of,
        environment_rows, host_record, identities, limits, obligation_rows, record_role,
        verdict_of,
    };
    use crate::app::evidence::Evidence;
    use crate::app::run_records::OutcomeName;
    use crate::app::runtime::{CheckWindow, declared_criteria};
    use crate::check::collector;
    use crate::check::consistency::{self, Prepared};
    use crate::check::decision::{DiagnosticState, Identity, IdentityState};
    use crate::contracts::receipt::{
        Id, List, Name, ObligationPageV1, ObligationV1State, ObservationsV1Cleanup, Payload,
        ReceiptV1, Ref, Sha, Text, TypedRef, VerdictV1State,
    };
    use crate::store::{RunRecordKind, VerificationVerdict};
    use crate::worker::host;

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

    /// R16.9 · the environment rows equal the 003 lane's page (`648c5b89…`, 14 rows, read from the
    /// retained CAS — an independent source) name for name, value for value, in order.
    #[test]
    fn the_environment_rows_are_the_003_page() -> Result<(), Box<dyn std::error::Error>> {
        let rows = environment_rows()?;
        let found: Vec<(&str, Option<&str>)> = rows
            .iter()
            .map(|row| {
                (
                    row.name.as_str(),
                    row.value.value.as_ref().map(Text::as_str),
                )
            })
            .collect();
        let known = [
            ("PATH", "/toolchain/bin"),
            ("HOME", "/work/home"),
            ("TMPDIR", "/tmp"),
            ("LANG", "C.UTF-8"),
            ("LC_ALL", "C.UTF-8"),
            ("TZ", "UTC"),
            ("CARGO_HOME", "/toolchain/cargo-home"),
            ("CARGO_TARGET_DIR", "/work/target"),
            ("CARGO_BUILD_JOBS", "2"),
            (
                "JULIA_DEPOT_PATH",
                "/work/julia-depot:/toolchain/julia-depot",
            ),
            ("JULIA_NUM_THREADS", "1"),
            ("OPENBLAS_NUM_THREADS", "1"),
            ("OMP_NUM_THREADS", "1"),
            ("RUST_BACKTRACE", "0"),
        ];
        assert_eq!(found.len(), 14);
        for (index, (name, value)) in known.into_iter().enumerate() {
            assert_eq!(found[index], (name, Some(value)), "row {index}");
        }
        Ok(())
    }

    /// R16.9 · the limits equal the 003 lane's object (`0b36c56a…`, read from the retained CAS)
    /// field for field where the enforcer is the same, and differ exactly where recorded: the
    /// wall and cleanup deadline are the check's (handed in), and `artifact_bytes` is the capture's
    /// enforced per-stream bound (16 MiB) rather than 003's unenforced 64 MiB.
    #[test]
    fn the_limits_are_the_003_object_where_the_enforcer_is_the_same()
    -> Result<(), Box<dyn std::error::Error>> {
        let limits = limits(290_000, 300_000)?;
        let known = [
            (limits.wall_ms.get(), 290_000),
            (limits.memory_bytes.get(), 8_589_934_592),
            (limits.memory_swap_bytes.get(), 0),
            (limits.scratch_bytes.get(), 4_294_967_296),
            (limits.stdout_bytes.get(), 8_388_608),
            (limits.stderr_bytes.get(), 8_388_608),
            (limits.artifact_bytes.get(), 16_777_216),
            (limits.external_requests.get(), 0),
            (limits.external_cost_microunits.get(), 0),
            (limits.term_grace_ms.get(), 5_000),
            (limits.cleanup_deadline_ms.get(), 300_000),
            (u64::from(limits.cpu_quota_percent), 200),
            (u64::from(limits.tasks_max), 128),
            (u64::from(limits.compiler_jobs), 2),
            (u64::from(limits.julia_threads), 1),
            (u64::from(limits.blas_threads), 1),
        ];
        for (index, (found, expected)) in known.into_iter().enumerate() {
            assert_eq!(found, expected, "field {index}");
        }
        assert_eq!(limits.currency.value, None);
        assert_eq!(
            limits
                .currency
                .unavailable_reason
                .as_ref()
                .map(Text::as_str),
            Some(ZERO_EXTERNAL_COST)
        );
        Ok(())
    }

    /// R16.3 · the host record carries every fact read, whole (two fixtures differing in every field
    /// would need two hosts; this one is this host's reading), and refuses a fact the receipt cannot
    /// name.
    #[test]
    fn the_host_record_carries_every_fact() -> Result<(), Box<dyn std::error::Error>> {
        let facts = crate::worker::host::Facts {
            os: "fedora".to_owned(),
            release: "44".to_owned(),
            architecture: "x86_64".to_owned(),
            kernel: "7.2.5-200.fc44.x86_64".to_owned(),
            boot_id: "270eb2e7-bf61-4a5c-9618-2c7e71817fb4".to_owned(),
            logical_cpus: 16,
            memory_bytes: 100_926_410_752,
            raw: b"raw".to_vec(),
        };
        let raw = payload("raw")?;
        let record = host_record(&facts, raw.clone())?;
        assert_eq!(
            (
                record.os.as_str(),
                record.release.as_str(),
                record.architecture.as_str(),
                record.kernel.as_str(),
                record.boot_id.as_str(),
                record.logical_cpus,
                record.memory_bytes.get(),
            ),
            (
                "fedora",
                "44",
                "x86_64",
                "7.2.5-200.fc44.x86_64",
                "270eb2e7-bf61-4a5c-9618-2c7e71817fb4",
                16,
                100_926_410_752
            )
        );
        assert_eq!(record.facts, raw);
        let mut unnameable = facts;
        unnameable.os = "x".repeat(129);
        assert!(host_record(&unnameable, payload("raw")?).is_err());
        Ok(())
    }

    /// R16.9 · four open, material obligations owned by the runtime, each citing the readback
    /// specification, in the cleanup record's order; the contract carries the stop grace, the whole
    /// deadline and every descendant reaped.
    #[test]
    fn the_obligations_and_the_contract_name_the_runtime() -> Result<(), Box<dyn std::error::Error>>
    {
        let ids = [
            "28f40000-0000-4000-8000-000000000001".to_owned(),
            "28f40000-0000-4000-8000-000000000002".to_owned(),
            "28f40000-0000-4000-8000-000000000003".to_owned(),
            "28f40000-0000-4000-8000-000000000004".to_owned(),
        ];
        let spec = payload("readback specification")?;
        let rows = obligation_rows(&ids, spec.as_ref())?;
        assert_eq!(rows.len(), 4);
        for (index, row) in rows.iter().enumerate() {
            assert_eq!(
                (
                    row.obligation_id.as_str(),
                    row.owner_id.as_str(),
                    row.scope.as_str(),
                    row.material,
                    row.state,
                    row.evidence.as_slice().len(),
                ),
                (
                    ids[index].as_str(),
                    RUNTIME_OWNER,
                    OBLIGATIONS[index],
                    true,
                    ObligationV1State::Open,
                    1
                ),
                "row {index}"
            );
        }
        assert!(
            obligation_rows(
                &[
                    "not-a-uuid".to_owned(),
                    ids[1].clone(),
                    ids[2].clone(),
                    ids[3].clone()
                ],
                spec.as_ref()
            )
            .is_err()
        );
        let page = TypedRef::<ObligationPageV1>::new(reference(
            "page",
            "hee3.receipt/1:ObligationPageV1",
        )?)?;
        let contract = cleanup_contract(300_000, page, spec)?;
        assert_eq!(
            (
                contract.owner_id.as_str(),
                contract.term_grace_ms.get(),
                contract.deadline_ms.get(),
                contract.require_empty_descendants
            ),
            (RUNTIME_OWNER, 5_000, 300_000, true)
        );
        Ok(())
    }

    /// A raw payload reference for a test, over the bytes' own digest.
    fn payload(bytes: &str) -> Result<Payload, Box<dyn std::error::Error>> {
        Ok(Payload::new(reference(bytes, "hee3.raw/1")?)?)
    }

    fn reference(bytes: &str, schema: &str) -> Result<Ref, Box<dyn std::error::Error>> {
        Ok(Ref {
            artifact_id: Id::new(format!(
                "28f40000-0000-4000-8000-{:012x}",
                bytes.len() + schema.len()
            ))?,
            sha256: Sha::new(crate::app::evidence::digest(bytes.as_bytes()))?,
            byte_length: u32::try_from(bytes.len())?,
            media_type: Name::new("application/octet-stream")?,
            schema_id: Name::new(schema)?,
        })
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

    /// The world's prepared plan (`tests/fixtures/receipt-import/preparation.json`), as the
    /// receipt-import lane reads it.
    fn fixture_prepared() -> Result<Prepared, Box<dyn std::error::Error>> {
        use crate::contracts::receipt::{IdentityV1, InvocationV1, SubjectsV1};
        #[derive(serde::Deserialize)]
        struct PlannedCase {
            case_id: Name,
            primary_module_id: Name,
            criterion_ids: List<Name>,
            fixture_sha256: Sha,
            oracle_id: Name,
            expected: TypedRef<crate::contracts::receipt::ExpectationV1>,
            mandatory: bool,
            selected: bool,
            excluded: bool,
        }
        #[derive(serde::Deserialize)]
        struct Preparation {
            schema_sha256: Sha,
            identity: IdentityV1,
            subjects: SubjectsV1,
            invocation: InvocationV1,
            cases: Vec<PlannedCase>,
        }
        let preparation: Preparation = serde_json::from_str(include_str!(
            "../../tests/fixtures/receipt-import/preparation.json"
        ))?;
        Ok(Prepared {
            schema_sha256: preparation.schema_sha256,
            identity: preparation.identity,
            subjects: preparation.subjects,
            invocation: preparation.invocation,
            cases: preparation
                .cases
                .into_iter()
                .map(|case| consistency::CasePlan {
                    case_id: case.case_id,
                    primary_module_id: case.primary_module_id,
                    criterion_ids: case.criterion_ids,
                    fixture_sha256: case.fixture_sha256,
                    oracle_id: case.oracle_id,
                    expected: case.expected,
                    mandatory: case.mandatory,
                    selected: case.selected,
                    excluded: case.excluded,
                    reviewed_design: None,
                })
                .collect(),
            editable: consistency::Editable {
                path: crate::contracts::receipt::RelPath::new("src/lib.rs")?,
                bounds: crate::check::patch::CandidateBounds {
                    bytes: 65_536,
                    changed_lines: 200,
                },
            },
        })
    }

    /// The four records, built by their constructors over a run that never launched, each
    /// published into the staging under a fresh id: `(clock, cleanup, (kind, artifact id, object))`.
    fn built_records(
        staging: &crate::store::ArtifactStaging,
        deadline: std::time::Instant,
        window: &CheckWindow,
        observed: std::time::Instant,
    ) -> Result<BuiltRecords, Box<dyn std::error::Error>> {
        use crate::app::run_records::{
            Intents, ObligationRecord, Readbacks as ReadbacksRecord, RunCleanup, RunClock,
            RunOutcome, RunRecord as _, RuntimeClock, Settlement,
        };
        use crate::app::workload::{Outcome as RunOutcomeKind, Run};
        use crate::contracts::UuidV4;
        let clock = RunClock::observe(
            &RuntimeClock {
                origin: window.begun,
                origin_unix_ms: window.begun_unix_ms,
                work_until: window.until,
                deadline: window.teardown_until,
            },
            Intents::default(),
            None,
            observed,
        )
        .map_err(|e| format!("{e:?}"))?;
        let run = Run::unlaunched(RunOutcomeKind::SetupFailed);
        let outcome = RunOutcome::of(&run, &[]).map_err(|e| format!("{e:?}"))?;
        let obligations = [
            ObligationRecord {
                id: "process".to_owned(),
                state: Settlement::Settled,
            },
            ObligationRecord {
                id: "scratch".to_owned(),
                state: Settlement::Settled,
            },
            ObligationRecord {
                id: "retained_paths".to_owned(),
                state: Settlement::Settled,
            },
            ObligationRecord {
                id: "aggregate".to_owned(),
                state: Settlement::Unknown,
            },
        ];
        let cleanup = RunCleanup::of(Settlement::Settled, &obligations, &[]);
        let attempt = "28f60000-0000-4000-8000-000000000001";
        let readbacks = ReadbacksRecord::of(UuidV4::parse(attempt)?, true, true, &[]);
        let mut records = Vec::new();
        for (index, (kind, bytes)) in [
            (RunRecordKind::RunClock, clock.to_bytes()),
            (RunRecordKind::RunOutcome, outcome.to_bytes()),
            (RunRecordKind::RunCleanup, cleanup.to_bytes()),
            (RunRecordKind::Readbacks, readbacks.to_bytes()),
        ]
        .into_iter()
        .enumerate()
        {
            let bytes = bytes.map_err(|e| format!("{e:?}"))?;
            let id = format!("28f60000-0000-4000-8000-0000000000{index:02x}");
            let staging_id = format!("28f60000-0000-4000-8000-0000000000{:02x}", index + 16);
            let object = staging
                .publish(&bytes, UuidV4::parse(&staging_id)?, deadline)
                .map_err(|e| format!("{e:?}"))?;
            records.push((kind, id, object));
        }
        Ok((clock, cleanup, records))
    }

    type BuiltRecords = (
        crate::app::run_records::RunClock,
        crate::app::run_records::RunCleanup,
        Vec<(RunRecordKind, String, crate::store::Object)>,
    );

    /// This host's facts (2026-09-26), as `worker::host::facts` would read them.
    fn host_fixture() -> host::Facts {
        host::Facts {
            os: "fedora".to_owned(),
            release: "44".to_owned(),
            architecture: "x86_64".to_owned(),
            kernel: "7.2.5-200.fc44.x86_64".to_owned(),
            boot_id: "270eb2e7-bf61-4a5c-9618-2c7e71817fb4".to_owned(),
            logical_cpus: 16,
            memory_bytes: 100_926_410_752,
            raw: b"os-release\nversion\n".to_vec(),
        }
    }

    /// The receipt decodes, its verdict is the decision's, it cites the four records as
    /// `run_record:<kind>` rows equal to what was published, and its unsettled-obligations page
    /// carries exactly the aggregate (unknown) citing the cleanup record; returns the decoded root.
    fn assert_records_cited(
        sink: &Evidence<'_>,
        composed: &super::Composed,
        records: &[(RunRecordKind, String, crate::store::Object)],
    ) -> Result<ReceiptV1, Box<dyn std::error::Error>> {
        use crate::check::graph::Graph;
        use crate::contracts::receipt::{ArtifactV1, decode};
        let root: ReceiptV1 = decode(&composed.bytes)?;
        assert_eq!(root.verdict.state, VerdictV1State::Invalid);
        assert_eq!(root.artifacts.count, 4);
        let graph = Graph::resolve(sink, composed.root.as_ref()).map_err(|e| format!("{e:?}"))?;
        let rows = graph
            .rows(root.artifacts.inventory.as_ref())
            .map_err(|e| format!("{e:?}"))?;
        let mut cited: Vec<(RunRecordKind, String, String, u64)> = rows
            .into_iter()
            .map(|value| {
                let row: ArtifactV1 = serde_json::from_value(value.clone())?;
                let kind = cited_kind(row.role.as_str()).ok_or("a run-record role")?;
                Ok::<_, Box<dyn std::error::Error>>((
                    kind,
                    row.object.artifact_id.as_str().to_owned(),
                    row.object.sha256.as_str().to_owned(),
                    u64::from(row.object.byte_length),
                ))
            })
            .collect::<Result<_, _>>()?;
        cited.sort_by_key(|row| row.0);
        let mut published: Vec<(RunRecordKind, String, String, u64)> = records
            .iter()
            .map(|(kind, id, object)| {
                (*kind, id.clone(), object.digest().to_owned(), object.size())
            })
            .collect();
        published.sort_by_key(|row| row.0);
        assert_eq!(cited, published);
        // The unsettled obligations page carries exactly the aggregate (unknown), citing the
        // cleanup record it came from.
        let unresolved = graph
            .rows(root.observations.unresolved_obligations.as_ref())
            .map_err(|e| format!("{e:?}"))?;
        assert_eq!(unresolved.len(), 1);
        let row: crate::contracts::receipt::ObligationV1 =
            serde_json::from_value(unresolved[0].clone())?;
        let cleanup_row = records
            .iter()
            .find(|(kind, _, _)| *kind == RunRecordKind::RunCleanup)
            .ok_or("a cleanup record")?;
        assert_eq!(
            (
                row.scope.as_str(),
                row.state,
                row.material,
                row.owner_id.as_str(),
                row.evidence.as_slice().len(),
                row.evidence.as_slice()[0].artifact_id.as_str(),
            ),
            (
                "aggregate",
                ObligationV1State::Unknown,
                true,
                RUNTIME_OWNER,
                1,
                cleanup_row.1.as_str()
            )
        );
        Ok(root)
    }

    /// R16 round 2, proof (a): the receipt composed over constructor-built records and the world's
    /// prepared plan (`tests/fixtures/receipt-import/preparation.json`, its objects staged as the
    /// receipt-import lane does) publishes through a staged sink, passes the class's own
    /// `validate` inside `finalize`, carries `decide`'s verdict — `Invalid`, naming exactly the
    /// gaps R16-G3 records (the three unsourced identities, the unknown aggregate) beside the
    /// run's own reasons — and cites the four run records as `run_record:<kind>` artifact rows
    /// equal to what was published. A refused compose (no case in the plan) is named.
    #[test]
    fn a_receipt_composes_over_the_records_and_names_its_gaps()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::check::collector::Sink;
        use crate::check::decision::ReasonKind;
        use crate::store::ArtifactStaging;
        use std::os::unix::fs::DirBuilderExt;
        use std::time::{Duration, Instant};

        #[derive(serde::Deserialize)]
        struct FixtureObject {
            reference: Ref,
            bytes: String,
        }
        #[derive(serde::Deserialize)]
        struct Fixture {
            objects: Vec<FixtureObject>,
        }
        let prepared = fixture_prepared()?;
        let fixture: Fixture = serde_json::from_str(include_str!(
            "../../tests/fixtures/receipt-import/nonpass.json"
        ))?;
        let area = std::env::temp_dir().join(format!("hee3-u64-receipt-{}", std::process::id()));
        std::fs::DirBuilder::new().mode(0o700).create(&area)?;
        let deadline = Instant::now() + Duration::from_secs(60);
        let staging = ArtifactStaging::open(&area, true, deadline).map_err(|e| format!("{e:?}"))?;
        let mut sink = Evidence::staged(&staging, deadline);
        for row in &fixture.objects {
            sink.publish(&row.reference, row.bytes.as_bytes())
                .map_err(|e| format!("{e:?}"))?;
        }
        // The records, built by their constructors over a run that never launched.
        let begun = Instant::now();
        let window = CheckWindow {
            begun,
            begun_unix_ms: 1_758_900_000_000,
            until: begun + Duration::from_secs(290),
            teardown_until: begun + Duration::from_secs(300),
        };
        let observed = begun + Duration::from_millis(41);
        let (clock, cleanup, records) = built_records(&staging, deadline, &window, observed)?;
        let host_facts = host_fixture();
        let composing = Composing {
            prepared: &prepared,
            clock: &clock,
            cleanup: &cleanup,
            records: &records,
            captures: &[],
            outcome: OutcomeName::SetupFailed,
            evaluation: None,
            window,
            observed,
            cancelled: false,
            host: &host_facts,
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
            compile_stderr_empty: true,
            link_stderr_empty: true,
        };
        let composed = compose(&mut sink, &composing).map_err(|e| format!("{e:?}"))?;
        // The decision: Invalid, and its reasons name the gaps and the run.
        assert_eq!(composed.decision.state(), VerdictV1State::Invalid);
        let kinds: Vec<ReasonKind> = composed
            .decision
            .reasons()
            .iter()
            .map(|reason| reason.kind)
            .collect();
        for expected in [
            ReasonKind::UnavailableIdentity,
            ReasonKind::ResourcesUnsettled,
            ReasonKind::ProducerNotStarted,
        ] {
            assert!(kinds.contains(&expected), "{expected:?} in {kinds:?}");
        }
        let root = assert_records_cited(&sink, &composed, &records)?;
        assert_eq!(root.observations.start_unix_ms.get(), 1_758_900_000_000);
        assert_eq!(root.observations.end_unix_ms.get(), 1_758_900_000_041);
        assert_eq!(root.observations.cutoff_unix_ms.get(), 1_758_900_290_000);
        assert_eq!(root.observations.cleanup, ObservationsV1Cleanup::Settled);
        // A plan with no case is refused by name.
        let mut planless = prepared.clone();
        planless.cases.clear();
        let refused = Composing {
            prepared: &planless,
            ..composing
        };
        assert!(matches!(
            compose(&mut sink, &refused),
            Err(Refusal::Publisher {
                stage: "plan",
                error: collector::Error::CasePlan
            })
        ));
        std::fs::remove_dir_all(&area)?;
        Ok(())
    }
}

/// What the composer is handed for one check (R16 round 2, decision 1): the records the runtime
/// built and the identities it published them under, the step captures, the window and observation,
/// the run's facts, the host's, and the frozen plan. Nothing here is a digest to re-acquire.
pub struct Composing<'a> {
    pub prepared: &'a Prepared,
    pub clock: &'a RunClock,
    pub cleanup: &'a RunCleanup,
    pub records: &'a [(RunRecordKind, String, Object)],
    /// One capture per step, `None` for a refused step; the execute step's is the producer.
    pub captures: &'a [Option<Captured>],
    pub outcome: OutcomeName,
    /// The oracle's counts on a match or mismatch, from the run's own `Evaluation`.
    pub evaluation: Option<(usize, usize)>,
    pub window: CheckWindow,
    pub observed: Instant,
    pub cancelled: bool,
    pub host: &'a host::Facts,
    pub readbacks: Readbacks,
    /// Whether the compile and link steps' stderr were empty (the diagnostics rule).
    pub compile_stderr_empty: bool,
    pub link_stderr_empty: bool,
}

/// The composed receipt: its root reference and bytes (published through the sink), the decision
/// it carries, and the runtime's derived verdict beside it — the two are different claims until
/// R16-G3 wires the ledger to the decision.
pub struct Composed {
    pub root: TypedRef<ReceiptV1>,
    pub bytes: Vec<u8>,
    pub decision: Decision,
    pub summary: consistency::Summary,
}

/// Why a receipt could not be composed; the check is then recorded through the runtime's own
/// evidence (R16 round 2, decision 10).
#[derive(Debug)]
pub enum Refusal {
    /// The execute step has no capture: the receipt cannot name a producer.
    Uncaptured,
    /// A receipt value could not be encoded within the wire's bounds.
    Encoding(crate::contracts::receipt::Error),
    /// The publisher refused at the named stage (a publication, the plan's own validation, a
    /// bound).
    Publisher {
        stage: &'static str,
        error: collector::Error,
    },
}

/// Name the stage a publisher refusal came from.
fn at(stage: &'static str) -> impl Fn(collector::Error) -> Refusal {
    move |error| Refusal::Publisher { stage, error }
}

impl From<crate::contracts::receipt::Error> for Refusal {
    fn from(error: crate::contracts::receipt::Error) -> Self {
        Self::Encoding(error)
    }
}

/// The step labels the workload runs, in order; the last is the producer's.
const STEPS: [&str; 3] = ["compile-library", "link-driver", "execute-driver"];

/// Compose and publish the receipt for one check through `sink` (R16 round 2). Every field has
/// one source; the decision is `decide`'s over the evidence the receipt has, and `validate` runs
/// inside `finalize` before the root is published.
///
/// # Errors
/// Each [`Refusal`], by name, before the root is published.
pub fn compose(sink: &mut Evidence<'_>, composing: &Composing<'_>) -> Result<Composed, Refusal> {
    // The plan's one case is required before anything is registered or published.
    let plan = composing
        .prepared
        .cases
        .first()
        .ok_or_else(|| at("plan")(collector::Error::CasePlan))?;
    let execute = composing
        .captures
        .get(STEPS.len() - 1)
        .and_then(Option::as_ref);
    register_records(sink, composing.records)?;
    let facts = sink
        .payload(&composing.host.raw, "application/octet-stream")
        .map_err(|error| Refusal::Publisher {
            stage: "host facts",
            error: collector::Error::Sink(error),
        })?;
    let mut publisher = Publisher::new(sink);
    let producer = match execute {
        Some(captured) => captured.producer.clone(),
        None => not_started_producer()?,
    };
    let timing = composing.clock.timing();
    let observations =
        observations_of(&mut publisher, composing, facts, producer.clone(), &timing)?;
    // An unselected plan case is unmeasured whatever the run did (RC04's case rule); the class's
    // one case is selected, so this arm is the fixture lane's.
    let (case_outcome, oracle_result, oracle_fact) = if plan.selected {
        case_of(composing.outcome, execute.is_some())
    } else {
        (
            CaseV1Outcome::Unmeasured,
            OracleResultV1Result::Unavailable,
            OracleFact::Unavailable,
        )
    };
    let raw_evidence = execute.map_or_else(Vec::new, |captured| {
        vec![
            captured.payloads.candidate_stdout.as_ref().clone(),
            captured.payloads.candidate_stderr.as_ref().clone(),
        ]
    });
    let oracle = publisher
        .record(&OracleResultV1 {
            oracle_id: plan.oracle_id.clone(),
            expected: plan.expected.clone(),
            result: oracle_result,
            detector_id: Maybe::present(plan.case_id.clone()),
            raw_evidence_refs: List::new(raw_evidence.clone())?,
            reason: Text::new(match composing.evaluation {
                Some((matched, failed)) => format!("vectors matched {matched}, failed {failed}"),
                None => format!("no evaluation: {}", outcome_name(composing.outcome)),
            })?,
        })
        .map_err(at("oracle result"))?;
    let (baseline, diagnostic_state) =
        diagnostics_of(composing.compile_stderr_empty, composing.link_stderr_empty);
    let complete = composing.captures.len() == STEPS.len() && execute.is_some()
        || composing.outcome != OutcomeName::Matched && composing.outcome != OutcomeName::Mismatch;
    let observed = observed_of(
        &mut publisher,
        composing,
        observations,
        Case {
            plan,
            outcome: case_outcome,
            producer: &producer,
            raw_evidence,
            oracle,
        },
        baseline,
        complete,
        &timing,
    )?;
    let decision = decide_over(
        composing,
        Deciding {
            producer: &producer,
            execute,
            case_outcome,
            oracle_fact,
            diagnostic_state,
            complete,
            timing,
        },
    )?;
    let finalized = publisher
        .finalize(composing.prepared, &decision, observed)
        .map_err(at("finalize"))?;
    Ok(Composed {
        root: finalized.reference,
        bytes: finalized.bytes,
        decision,
        summary: finalized.summary,
    })
}

/// Register the records the receipt cites with the sink, so the graph walk inside `finalize`
/// resolves them: they are in the object store already; the sink's registry is what a walk reads.
fn register_records(
    sink: &mut Evidence<'_>,
    records: &[(RunRecordKind, String, Object)],
) -> Result<(), Refusal> {
    for (row, (_, _, object)) in record_rows(records)?.into_iter().zip(records) {
        sink.register(row.object, object.clone())
            .map_err(|error| Refusal::Publisher {
                stage: "records",
                error: collector::Error::Sink(error),
            })?;
    }
    Ok(())
}

/// The one case as observed, for `observed_of`.
struct Case<'a> {
    plan: &'a consistency::CasePlan,
    outcome: CaseV1Outcome,
    producer: &'a ProducerV1,
    raw_evidence: Vec<Ref>,
    oracle: TypedRef<OracleResultV1>,
}

/// Everything `finalize` derives its summaries from: the observations, the one case, the
/// diagnostics rule's baseline, every capture's artifact rows plus the run records cited as rows
/// (R16 round 2, decision 1), no campaigns, the verdict binding and the availability.
fn observed_of(
    publisher: &mut Publisher<'_, Evidence<'_>>,
    composing: &Composing<'_>,
    observations: ObservationsV1,
    case: Case<'_>,
    baseline: bool,
    complete: bool,
    timing: &decision::Timing,
) -> Result<Observed, Refusal> {
    let missing_objects = publisher
        .missing_object_pages(&[])
        .map_err(at("missing objects"))?;
    let mut artifacts: Vec<ArtifactV1> = composing
        .captures
        .iter()
        .flatten()
        .flat_map(|captured| captured.artifacts.iter().cloned())
        .collect();
    artifacts.extend(record_rows(composing.records)?);
    Ok(Observed {
        observations,
        cases: vec![CaseObservation {
            case_id: case.plan.case_id.clone(),
            executed: case.producer.status != ProducerV1Status::NotStarted,
            outcome: case.outcome,
            producer: case.producer.clone(),
            detector_id: case.plan.case_id.clone(),
            benign_pair_id: Maybe::unavailable(Text::new("no benign pair")?),
            raw_evidence_refs: List::new(case.raw_evidence)?,
            reason: Text::new(outcome_name(composing.outcome))?,
        }],
        diagnostic_baseline: baseline,
        diagnostics: Vec::new(),
        diagnostic_mismatch: Maybe::unavailable(Text::new(if baseline {
            STRICT_EMPTY_DIAGNOSTICS
        } else {
            "diagnostics_unavailable"
        })?),
        artifacts,
        artifacts_finalized: complete,
        campaigns: Vec::new(),
        verdict: VerdictBinding {
            oracle_result: case.oracle,
            intended_detector: case.plan.case_id.clone(),
            benign_pair: Maybe::unavailable(Text::new("no benign pair")?),
        },
        availability: AvailabilityV1 {
            observed_unix_ms: decimal(composing.window.begun_unix_ms + timing.observed_ms)?,
            state: if complete {
                AvailabilityV1State::Complete
            } else {
                AvailabilityV1State::Incomplete
            },
            missing_objects,
            retention_policy: AvailabilityV1RetentionPolicy::RetainV1,
        },
    })
}

/// The run outcome's wire name, for the receipt's reasons.
fn outcome_name(outcome: OutcomeName) -> String {
    serde_json::to_value(outcome)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// The receipt's observations: the host, the clock's instants as unix and monotonic-relative
/// values, an empty resources page (the runtime measures no consumption yet — R16-G1), the
/// producer, the cancellation and cleanup the records observed, and the unsettled obligations.
fn observations_of(
    publisher: &mut Publisher<'_, Evidence<'_>>,
    composing: &Composing<'_>,
    facts: Payload,
    producer: ProducerV1,
    timing: &decision::Timing,
) -> Result<ObservationsV1, Refusal> {
    let host = publisher
        .record(&host_record(composing.host, facts)?)
        .map_err(at("host"))?;
    let resources = publisher.resource_pages(&[]).map_err(at("resources"))?;
    let unresolved = unresolved_rows(composing.cleanup, composing.records)?;
    let unresolved_obligations = publisher
        .obligation_pages(&unresolved)
        .map_err(at("obligations"))?;
    Ok(ObservationsV1 {
        host,
        start_unix_ms: decimal(composing.window.begun_unix_ms)?,
        end_unix_ms: decimal(composing.window.begun_unix_ms + timing.observed_ms)?,
        start_monotonic_ns: decimal(0)?,
        end_monotonic_ns: decimal(timing.observed_ms.saturating_mul(1_000_000))?,
        cutoff_unix_ms: decimal(composing.window.begun_unix_ms + timing.work_deadline_ms)?,
        resources,
        producer,
        cancellation: if composing.cancelled {
            ObservationsV1Cancellation::Requested
        } else {
            ObservationsV1Cancellation::NotRequested
        },
        cleanup: match composing.cleanup.aggregate() {
            RecordSettlement::Settled => ObservationsV1Cleanup::Settled,
            RecordSettlement::Pending | RecordSettlement::Unknown => ObservationsV1Cleanup::Pending,
            RecordSettlement::Failed => ObservationsV1Cleanup::Failed,
        },
        unresolved_obligations,
    })
}

/// `decide` over the facts the records and captures carry (R16 round 2, decision 4): the eleven
/// identities, the frozen plan, the one case, the producer and checker (the driver's process),
/// the oracle fact, the capture's stream states, the diagnostics rule, the cleanup record's
/// settlements, the evidence state and the clock's timing.
#[derive(Clone, Copy)]
struct Deciding<'a> {
    producer: &'a ProducerV1,
    execute: Option<&'a Captured>,
    case_outcome: CaseV1Outcome,
    oracle_fact: OracleFact,
    diagnostic_state: DiagnosticState,
    complete: bool,
    timing: decision::Timing,
}

fn decide_over(composing: &Composing<'_>, deciding: Deciding<'_>) -> Result<Decision, Refusal> {
    let Deciding {
        producer,
        execute,
        case_outcome,
        oracle_fact,
        diagnostic_state,
        complete,
        timing,
    } = deciding;
    let plan = composing
        .prepared
        .cases
        .first()
        .ok_or_else(|| at("plan")(collector::Error::CasePlan))?;
    let process = ProcessFact {
        expected: ExpectedProducerV1 {
            status: ExpectedProducerV1Status::Exited,
            exit_code: Maybe::present(0),
            signal: Maybe::unavailable(Text::new("exit expected")?),
        },
        actual: process_state(producer),
        termination: match composing.outcome {
            OutcomeName::Timeout => Termination::Deadline,
            OutcomeName::Cancelled => Termination::Cancellation,
            OutcomeName::PendingCleanup | OutcomeName::SetupFailed => Termination::Unknown,
            _ => Termination::Ordinary,
        },
    };
    let facts = identities(composing.readbacks);
    let plans = decision_plans(composing.prepared)?;
    let case = decision::CaseObservation {
        case_id: plan.case_id.clone(),
        executed: execute.is_some(),
        outcome: case_outcome,
        incomplete_cause: incomplete_cause(composing.outcome),
        producer: process_state(producer),
    };
    Ok(decision::decide(&decision::Input {
        identities: &facts,
        plans: &plans,
        cases: &[case],
        producer: &process,
        checker: &CheckerFact::Process(process.clone()),
        oracle: oracle_fact,
        oracle_unavailable_cause: incomplete_cause(composing.outcome),
        logs: Streams {
            stdout: log_state(execute.map(|c| c.flags.candidate_stdout_complete)),
            stderr: log_state(execute.map(|c| c.flags.candidate_stderr_complete)),
        },
        diagnostics: Diagnostics {
            policy: DiagnosticPolicy::CleanBaseline,
            state: diagnostic_state,
        },
        cleanup: CleanupFacts {
            descendants: settlement(composing.cleanup, "process"),
            resources: settlement(composing.cleanup, "aggregate"),
            obligations: composing.cleanup.aggregate().into(),
        },
        evidence: if complete {
            EvidenceState::FinalizedComplete
        } else {
            EvidenceState::Incomplete
        },
        timing,
    }))
}

/// The frozen plan's cases as `decide` reads them: the class's one mandatory, selected, reviewed
/// case expects the driver to exit 0.
fn decision_plans(
    prepared: &Prepared,
) -> Result<Vec<decision::CasePlan>, crate::contracts::receipt::Error> {
    prepared
        .cases
        .iter()
        .map(|plan| {
            Ok(decision::CasePlan {
                case_id: plan.case_id.clone(),
                selection: if plan.excluded {
                    decision::Selection::Excluded
                } else if plan.mandatory && plan.selected {
                    decision::Selection::Required
                } else if plan.selected {
                    decision::Selection::Optional
                } else {
                    decision::Selection::Unselected
                },
                expected_producer: ExpectedProducerV1 {
                    status: ExpectedProducerV1Status::Exited,
                    exit_code: Maybe::present(0),
                    signal: Maybe::unavailable(Text::new("exit expected")?),
                },
                design: if plan.reviewed_design.is_some() {
                    decision::Design::Reviewed
                } else {
                    decision::Design::Unreviewed
                },
            })
        })
        .collect()
}

/// The producer of a run whose execute step never started.
fn not_started_producer() -> Result<ProducerV1, crate::contracts::receipt::Error> {
    let reason = || Text::new("not started");
    Ok(ProducerV1 {
        status: ProducerV1Status::NotStarted,
        exit_code: Maybe::unavailable(reason()?),
        signal: Maybe::unavailable(reason()?),
        timeout: false,
        stdout: Maybe::unavailable(reason()?),
        stderr: Maybe::unavailable(reason()?),
    })
}

fn decimal(value: u64) -> Result<U64, crate::contracts::receipt::Error> {
    U64::new(value.to_string())
}

/// The run records cited as artifact rows (R16 round 2, decision 1): `hee3.raw/1` references
/// under `run_record:<kind>`, required and available — the ledger's kind names their meaning.
fn record_rows(
    records: &[(RunRecordKind, String, Object)],
) -> Result<Vec<ArtifactV1>, crate::contracts::receipt::Error> {
    records
        .iter()
        .map(|(kind, id, object)| {
            Ok(ArtifactV1 {
                object: Ref {
                    artifact_id: Id::new(id.as_str())?,
                    sha256: Sha::new(object.digest())?,
                    byte_length: u32::try_from(object.size())
                        .map_err(|_| crate::contracts::receipt::Error::Bound)?,
                    media_type: Name::new("application/json")?,
                    schema_id: Name::new("hee3.raw/1")?,
                },
                role: record_role(*kind)?,
                required: true,
                truncated: false,
                availability: ArtifactV1Availability::Available,
                reason: Text::new(kind.schema_id())?,
            })
        })
        .collect()
}

/// The obligations the cleanup record left unsettled, as receipt rows citing the record itself.
fn unresolved_rows(
    cleanup: &RunCleanup,
    records: &[(RunRecordKind, String, Object)],
) -> Result<Vec<ObligationV1>, crate::contracts::receipt::Error> {
    let cited = records
        .iter()
        .find(|(kind, _, _)| *kind == RunRecordKind::RunCleanup)
        .map(|(_, id, object)| {
            Ok::<_, crate::contracts::receipt::Error>(Ref {
                artifact_id: Id::new(id.as_str())?,
                sha256: Sha::new(object.digest())?,
                byte_length: u32::try_from(object.size())
                    .map_err(|_| crate::contracts::receipt::Error::Bound)?,
                media_type: Name::new("application/json")?,
                schema_id: Name::new("hee3.raw/1")?,
            })
        })
        .transpose()?;
    cleanup
        .obligations()
        .iter()
        .filter(|obligation| obligation.state != RecordSettlement::Settled)
        .enumerate()
        .map(|(index, obligation)| {
            Ok(ObligationV1 {
                obligation_id: Id::new(format!("28f50000-0000-4000-8000-{index:012x}"))?,
                owner_id: Name::new(RUNTIME_OWNER)?,
                scope: Text::new(obligation.id.as_str())?,
                material: true,
                state: match obligation.state {
                    RecordSettlement::Unknown => ObligationV1State::Unknown,
                    _ => ObligationV1State::Open,
                },
                evidence: List::new(cited.iter().cloned().collect())?,
                reason: Text::new(format!("{:?}", obligation.state))?,
            })
        })
        .collect()
}

/// The case outcome, oracle result and oracle fact one run outcome earns — one arm per outcome.
fn case_of(
    outcome: OutcomeName,
    executed: bool,
) -> (CaseV1Outcome, OracleResultV1Result, OracleFact) {
    match outcome {
        OutcomeName::Matched => (
            CaseV1Outcome::Passed,
            OracleResultV1Result::Satisfied,
            OracleFact::Satisfied,
        ),
        OutcomeName::Mismatch => (
            CaseV1Outcome::Failed,
            OracleResultV1Result::Violated,
            OracleFact::Mismatch,
        ),
        OutcomeName::InvalidOutput => (
            CaseV1Outcome::Invalid,
            OracleResultV1Result::Error,
            OracleFact::Malformed,
        ),
        OutcomeName::Timeout => (
            CaseV1Outcome::Timeout,
            OracleResultV1Result::Unavailable,
            OracleFact::Unavailable,
        ),
        OutcomeName::Cancelled => (
            CaseV1Outcome::Skipped,
            OracleResultV1Result::Unavailable,
            OracleFact::Unavailable,
        ),
        OutcomeName::InvalidSubject
        | OutcomeName::ProducerFailed
        | OutcomeName::ProducerError
        | OutcomeName::LauncherFailed
        | OutcomeName::PendingCleanup
        | OutcomeName::SetupFailed => (
            if executed {
                CaseV1Outcome::Broken
            } else {
                CaseV1Outcome::Skipped
            },
            OracleResultV1Result::Unavailable,
            OracleFact::Unavailable,
        ),
    }
}

/// Why the oracle is unavailable, by the run's outcome.
fn incomplete_cause(outcome: OutcomeName) -> IncompleteCause {
    match outcome {
        OutcomeName::Timeout => IncompleteCause::Deadline,
        OutcomeName::Cancelled => IncompleteCause::Cancellation,
        OutcomeName::ProducerFailed | OutcomeName::ProducerError | OutcomeName::LauncherFailed => {
            IncompleteCause::ProducerFailure
        }
        _ => IncompleteCause::Unexplained,
    }
}

/// The producer's process state as the receipt recorded it.
fn process_state(producer: &ProducerV1) -> ProcessState {
    match producer.status {
        ProducerV1Status::Exited => producer
            .exit_code
            .value
            .map_or(ProcessState::Unknown, ProcessState::Exited),
        ProducerV1Status::Signalled => producer
            .signal
            .value
            .map_or(ProcessState::Unknown, ProcessState::Signalled),
        ProducerV1Status::NotStarted => ProcessState::NotStarted,
        ProducerV1Status::Unknown => ProcessState::Unknown,
    }
}

fn log_state(complete: Option<bool>) -> LogState {
    match complete {
        Some(true) => LogState::Complete,
        Some(false) => LogState::Truncated,
        None => LogState::Missing,
    }
}

/// One named obligation's settlement as `decide` reads it.
fn settlement(cleanup: &RunCleanup, id: &str) -> decision::Settlement {
    cleanup
        .obligations()
        .iter()
        .find(|obligation| obligation.id == id)
        .map_or(decision::Settlement::Unknown, |obligation| {
            obligation.state.into()
        })
}

impl From<RecordSettlement> for decision::Settlement {
    fn from(state: RecordSettlement) -> Self {
        match state {
            RecordSettlement::Settled => Self::Settled,
            RecordSettlement::Pending => Self::Pending,
            RecordSettlement::Failed => Self::Failed,
            RecordSettlement::Unknown => Self::Unknown,
        }
    }
}
