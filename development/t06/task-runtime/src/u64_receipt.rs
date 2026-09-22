//! Fixed WL-U64-PARSE-001 receipt composition from trusted application observations.
//! This is not a process owner, admission decision, or general workload registry.

#[path = "u64_receipt/binding.rs"]
mod binding;
#[path = "u64_receipt/observed.rs"]
mod observed;
#[path = "u64_receipt/preparation.rs"]
mod preparation;
pub use preparation::{FrozenExpectation, publish_expectation};

use habitat_engine::app::{capture, evidence::Evidence, subjects, workload};
use habitat_engine::check::{collector as c, consistency, decision as d, graph, u64_oracle};
use habitat_engine::contracts::receipt as r;
use habitat_engine::worker::{
    namespace::NamespaceReport,
    workspace::{Content, Snapshot},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::time::Instant;

const WRAPPER_SHA: &str = "2fb599c68f57cb8b777f595f1217762531f9c6784c72765d15e83c553b4c5f59";
const PATCH_SHA: &str = "12db53d34f3a388d19d1b971d2bd5dfeb67f8c0e1a459d25e1012954c7b22f8e";
// Finite input subjects, not expected verdicts. Both candidates use the same
// independently reviewed oracle; adding this incomplete repair permits real
// no-progress observations without selecting or manufacturing a check result.
const INCOMPLETE_PATCH_SHA: &str =
    "d71988d3e4e0cd9714066665be10165cb1ac2f932c446e05cea6157ac977c9a0";
const REPAIRED_SHA: &str = "83222548d636d138bb269b5c9a8c970c0b47c5420961875c8cea83103e39e163";
const INCOMPLETE_SHA: &str = "e882f958059fc3293adfc0187a1b14434a27055654a9031d2a8f5554bbf851eb";

pub(crate) fn repair_pair(patch: &[u8], result: &Snapshot) -> bool {
    file(result, "src/lib.rs").is_ok_and(|source| {
        let pair = (hash(patch), hash(source));
        [
            (PATCH_SHA, REPAIRED_SHA),
            (INCOMPLETE_PATCH_SHA, INCOMPLETE_SHA),
        ]
        .iter()
        .any(|(patch, source)| pair.0 == *patch && pair.1 == *source)
    })
}
const LABELS: [&str; 3] = ["compile-library", "link-driver", "execute-driver"];

/// Fixed local recipe evidence; every cutoff uses the same task origin.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub candidate_cutoff_ms: u64,
    pub verification_cutoff_ms: u64,
    pub total_wall_ms: u64,
}
pub struct SubjectBinding<'a> {
    pub snapshot: &'a Snapshot,
    pub origin: r::SubjectFileV1Origin,
}
pub struct SourceBindings<'a> {
    pub seed: SubjectBinding<'a>,
    pub result: SubjectBinding<'a>,
    pub fixtures: SubjectBinding<'a>,
    pub oracle: SubjectBinding<'a>,
    pub harness: SubjectBinding<'a>,
    pub collector: SubjectBinding<'a>,
    pub launcher: SubjectBinding<'a>,
}
pub enum Patch<'a> {
    Baseline,
    Repair(&'a [u8]),
}
pub struct Preparation<'a> {
    pub prepared: &'a consistency::Prepared,
    pub sources: SourceBindings<'a>,
    pub protected: &'a Snapshot,
    pub tools: &'a workload::Tools,
    pub patch: Patch<'a>,
    pub recipe: r::Payload,
    pub host: r::TypedRef<r::HostV1>,
    pub review_provenance: r::Payload,
    pub deadline: Instant,
}
struct BoundSource {
    snapshot: Snapshot,
    reference: r::TypedRef<r::SubjectV1>,
    origin: r::SubjectFileV1Origin,
    identity: d::Identity,
}
/// Owned pre-execution state. No caller can replace its prepared identities after dispatch.
pub struct Frozen {
    prepared: consistency::Prepared,
    sources: Vec<BoundSource>,
    protected: Snapshot,
    oracle_bytes: Vec<u8>,
    recipe: Recipe,
    recipe_ref: r::Payload,
    host: r::TypedRef<r::HostV1>,
    provenance: r::Payload,
    expectation: r::ExpectationV1,
    tool_pins: std::collections::BTreeMap<std::path::PathBuf, [u8; 32]>,
    compiler_pin: (std::path::PathBuf, [u8; 32]),
}
impl Frozen {
    #[must_use]
    pub const fn prepared(&self) -> &consistency::Prepared {
        &self.prepared
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Clock {
    pub origin: Instant,
    pub start_unix_ms: u64,
    pub end_unix_ms: u64,
    pub candidate_deadline: Instant,
    /// Predeclared coordinator verification cutoff, not the earlier candidate cutoff.
    pub work_deadline: Instant,
    pub cleanup_deadline: Instant,
    pub observed_at: Instant,
    /// Actual durable cancellation observation, never reconstructed from a desired outcome.
    pub cancellation_observed_at: Option<Instant>,
}
pub struct Cleanup {
    pub aggregate: d::Settlement,
    pub obligations: d::Settlement,
    pub evidence: r::Payload,
    pub unresolved: Vec<r::ObligationV1>,
}
pub struct OracleObservation {
    checked_at: Instant,
    stdout_sha256: String,
    terminal_end: Instant,
    result: Option<Result<u64_oracle::Evaluation, u64_oracle::OutputError>>,
}
pub struct Collected {
    pub receipt: c::Finalized,
    pub decision: d::Decision,
    pub decision_inputs: r::Payload,
    pub oracle_observation: r::Payload,
}
#[derive(Debug)]
pub enum ErrorKind {
    Deadline,
    Binding,
    Review,
    Recipe,
    UnsupportedRun,
    Encoding,
    Graph(graph::Error),
    Source(subjects::Error),
    Capture(Box<capture::Error>),
    Publication(c::Error),
}
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub attempted_refs: Vec<r::Ref>,
}
fn fail(kind: ErrorKind) -> Error {
    Error {
        kind,
        attempted_refs: Vec::new(),
    }
}
fn scalar<T>(value: Result<T, r::Error>) -> Result<T, Error> {
    value.map_err(|_| fail(ErrorKind::Encoding))
}
fn text(value: impl Into<String>) -> Result<r::Text, Error> {
    scalar(r::Text::new(value))
}
fn name(value: &str) -> Result<r::Name, Error> {
    scalar(r::Name::new(value))
}
fn none<T>(why: &str) -> Result<r::Maybe<T>, Error> {
    Ok(r::Maybe::unavailable(text(why)?))
}
fn number(value: u64) -> Result<r::U64, Error> {
    scalar(r::U64::new(value.to_string()))
}
fn list<T: r::Validate>(value: Vec<T>) -> Result<r::List<T>, Error> {
    scalar(r::List::new(value))
}
fn budget(deadline: Instant) -> Result<(), Error> {
    if Instant::now() >= deadline {
        Err(fail(ErrorKind::Deadline))
    } else {
        Ok(())
    }
}
fn hash(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut out, byte| {
            write!(out, "{byte:02x}").expect("String formatting");
            out
        })
}
fn bytes(evidence: &Evidence<'_>, reference: &r::Ref) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    evidence
        .open(reference)
        .map_err(|e| fail(ErrorKind::Graph(e)))?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut out)
        .map_err(|_| fail(ErrorKind::Graph(graph::Error::Io)))?;
    if out.len() > 16 * 1024 * 1024 {
        return Err(fail(ErrorKind::Encoding));
    }
    Ok(out)
}
fn raw(evidence: &mut Evidence<'_>, value: &[u8]) -> Result<r::Payload, Error> {
    evidence
        .payload(value, "application/octet-stream")
        .map_err(|e| fail(ErrorKind::Publication(c::Error::Sink(e))))
}
fn json_raw(evidence: &mut Evidence<'_>, value: &impl Serialize) -> Result<r::Payload, Error> {
    evidence
        .payload(
            &serde_json::to_vec(value).map_err(|_| fail(ErrorKind::Encoding))?,
            "application/json",
        )
        .map_err(|e| fail(ErrorKind::Publication(c::Error::Sink(e))))
}
fn typed<T: r::ReceiptRecord>(
    evidence: &Evidence<'_>,
    reference: &r::TypedRef<T>,
) -> Result<T, Error> {
    let closure = graph::Graph::resolve(evidence, reference.as_ref())
        .map_err(|e| fail(ErrorKind::Graph(e)))?;
    scalar(r::decode(
        closure
            .get(reference.as_ref())
            .map_err(|e| fail(ErrorKind::Graph(e)))?
            .bytes(),
    ))
}
fn record<T: r::ReceiptRecord>(
    evidence: &mut Evidence<'_>,
    value: &T,
) -> Result<r::TypedRef<T>, Error> {
    let mut publisher = c::Publisher::new(evidence);
    publisher.record(value).map_err(|e| Error {
        kind: ErrorKind::Publication(e),
        attempted_refs: publisher.attempted_refs().to_vec(),
    })
}
fn file<'a>(snapshot: &'a Snapshot, path: &str) -> Result<&'a [u8], Error> {
    snapshot
        .entries()
        .find_map(|e| match &e.content {
            Content::File { bytes, .. } if e.path == path => Some(bytes.as_slice()),
            _ => None,
        })
        .ok_or_else(|| fail(ErrorKind::Binding))
}
use habitat_engine::check::graph::Objects as _;

/// Verify owned preparation, frozen role bindings and independently reviewed expectations.
/// # Errors
/// Refuses changed/incomplete subjects, absent review, unsupported recipe or patch mismatch.
pub fn prepare(evidence: &Evidence<'_>, input: Preparation<'_>) -> Result<Frozen, Error> {
    preparation::prepare(evidence, input)
}

/// Run the actual bounded in-process oracle over retained driver stdout.
/// This may occur after aggregate cleanup; its timestamp is sampled after evaluation.
/// # Errors
/// Refuses incomplete/refused stage sequences or absent/invalid exact oracle inputs.
pub fn observe(frozen: &Frozen, run: &workload::Run) -> Result<OracleObservation, Error> {
    let reports = observed::reports(run)?;
    let report = *reports
        .last()
        .ok_or_else(|| fail(ErrorKind::UnsupportedRun))?;
    let oracle = u64_oracle::FrozenOracle::from_bytes(&frozen.oracle_bytes)
        .map_err(|_| fail(ErrorKind::Binding))?;
    let result = (reports.len() == 3 && reports.iter().all(|r| observed::normal(r)))
        .then(|| oracle.evaluate(&report.observer.candidate_stdout.bytes));
    Ok(OracleObservation {
        checked_at: Instant::now(),
        stdout_sha256: hash(&report.observer.candidate_stdout.bytes),
        terminal_end: report
            .process
            .started_at
            .checked_add(report.process.elapsed)
            .ok_or_else(|| fail(ErrorKind::Binding))?,
        result,
    })
}

/// Derive a decision and publish its exact consistent RC04 graph from retained observations.
/// # Errors
/// Refuses missing preparation, substituted observations, invalid timing, unsupported refusals,
/// or publication/graph failure. The caller retains Run, Evidence and all cleanup obligations.
pub fn finalize(
    evidence: &mut Evidence<'_>,
    frozen: &Frozen,
    run: &workload::Run,
    oracle: &OracleObservation,
    clock: Clock,
    cleanup: &Cleanup,
    deadline: Instant,
) -> Result<Collected, Error> {
    observed::finalize(evidence, frozen, run, oracle, &clock, cleanup, deadline)
}
