//! The receipt composed over the check's records (B14a-2c; design R16 round 2 in
//! `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`).
//!
//! This file is the compiled skeleton (2c-i): the pure rules the design fixes before the composer is
//! assembled — how a run record is cited in a receipt graph, the RC04 verdict mapped to the ledger's
//! (kept UNWIRED until R16-G3: the ledger's verdict stays `live_verifier::checked`'s while `decide`
//! cannot pass), the strict-empty diagnostics rule, the eleven identities with the sources the class
//! fixes, and the ledger as an [`Objects`] owner so an accepted receipt can be walked.

use super::capture::MAX_RAW_BYTES;
use super::runtime::declared_criteria;
use crate::check::decision::{DiagnosticState, Identity, IdentityFact, IdentityState};
use crate::check::graph::{self, Objects};
use crate::contracts::receipt::{
    CleanupContractV1, EnvironmentV1, HostV1, Id, LimitsV1, List, Maybe, Name, ObligationPageV1,
    ObligationV1, ObligationV1State, Payload, Ref, Text, TypedRef, U64, VerdictV1State,
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
        OBLIGATIONS, RUNTIME_OWNER, Readbacks, STRICT_EMPTY_DIAGNOSTICS, ZERO_EXTERNAL_COST,
        cited_kind, cleanup_contract, diagnostics_of, environment_rows, host_record, identities,
        limits, obligation_rows, record_role, verdict_of,
    };
    use crate::app::runtime::declared_criteria;
    use crate::check::decision::{DiagnosticState, Identity, IdentityState};
    use crate::contracts::receipt::{
        Id, Name, ObligationPageV1, ObligationV1State, Payload, Ref, Sha, Text, TypedRef,
        VerdictV1State,
    };
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
}
