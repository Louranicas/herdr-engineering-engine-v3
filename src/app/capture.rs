//! Retain trusted namespace observations without making a case or task decision.
//! Native wait status, launcher status, interruption and cleanup stay separate.

use super::evidence::Evidence;
use crate::check::collector::{self, Publisher, SinkError};
use crate::contracts::receipt::{
    self, ArtifactPageV1, ArtifactV1, ArtifactV1Availability, Maybe, Name, Payload, ProducerV1,
    ProducerV1Status, Ref, Text, TypedRef,
};
use crate::worker::namespace::{NamespaceError, NamespaceFacts, NamespaceReport, NativeStatus};
use crate::worker::process::{Interruption, SignalOutcome, Stream};
use serde::Serialize;
use std::collections::BTreeMap;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

const MAX_RAW_BYTES: usize = 16 * 1024 * 1024;
const MAX_FACT_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    CandidateStdout,
    CandidateStderr,
    LauncherStdout,
    LauncherStderr,
    NativeStatus,
    Facts,
}
impl Role {
    const fn name(self) -> &'static str {
        match self {
            Self::CandidateStdout => "candidate_stdout",
            Self::CandidateStderr => "candidate_stderr",
            Self::LauncherStdout => "launcher_stdout",
            Self::LauncherStderr => "launcher_stderr",
            Self::NativeStatus => "native_status",
            Self::Facts => "namespace_capture_facts",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Payload(Role),
    Producer,
    Inventory,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cause {
    Sink(SinkError),
    Publisher(collector::Error),
    Encoding,
    Bound,
}
#[derive(Clone, Debug)]
pub struct PublishedPayload {
    pub role: Role,
    pub reference: Payload,
}
#[derive(Debug)]
pub struct Error {
    pub stage: Stage,
    pub cause: Cause,
    pub published_payloads: Vec<PublishedPayload>,
    pub attempted_refs: Vec<Ref>,
}
#[derive(Debug)]
pub struct Payloads {
    pub candidate_stdout: Payload,
    pub candidate_stderr: Payload,
    pub launcher_stdout: Payload,
    pub launcher_stderr: Payload,
    pub native_status: Payload,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCounting {
    Unmeasured,
}
#[derive(Debug, Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent observed predicates must not collapse into a verdict"
)]
pub struct Flags {
    pub candidate_stdout_complete: bool,
    pub candidate_stderr_complete: bool,
    pub launcher_stdout_complete: bool,
    pub launcher_stderr_complete: bool,
    pub native_stream_complete: bool,
    pub candidate_stderr_nonempty: bool,
    pub launcher_stderr_nonempty: bool,
    pub native_classification_available: bool,
    pub diagnostic_counting: DiagnosticCounting,
}
#[derive(Debug)]
pub struct Captured {
    pub producer: ProducerV1,
    pub producer_ref: TypedRef<ProducerV1>,
    pub payloads: Payloads,
    pub facts_payload: Payload,
    pub flags: Flags,
    pub artifacts: Vec<ArtifactV1>,
    pub artifact_page: TypedRef<ArtifactPageV1>,
}

/// Publish only already-observed retained bytes and their separate factual metadata.
///
/// # Errors
/// Retains publication stage, prior payloads and attempted typed references on failure.
/// The caller still owns the report, pending cleanup and the Evidence object owner.
pub fn capture(evidence: &mut Evidence<'_>, report: &NamespaceReport) -> Result<Captured, Error> {
    let streams = stream_inputs(report);
    for (role, stream) in streams {
        if stream.bytes.len() > MAX_RAW_BYTES {
            return Err(failure(Stage::Payload(role), Cause::Bound, &[], &[]));
        }
    }
    let flags = flags(report);
    let facts_bytes = serde_json::to_vec(&report_facts(report, &flags))
        .map_err(|_| failure(Stage::Payload(Role::Facts), Cause::Encoding, &[], &[]))?;
    if facts_bytes.len() > MAX_FACT_BYTES {
        return Err(failure(Stage::Payload(Role::Facts), Cause::Bound, &[], &[]));
    }
    let mut published = Vec::with_capacity(6);
    let payloads = publish_streams(evidence, report, &mut published)?;
    let facts_payload = publish_raw(
        evidence,
        Role::Facts,
        &facts_bytes,
        "application/json",
        &mut published,
    )?;
    let producer = producer(report, &payloads)
        .map_err(|_| failure(Stage::Producer, Cause::Encoding, &published, &[]))?;
    let artifacts = artifact_rows(&published, &streams)
        .map_err(|_| failure(Stage::Inventory, Cause::Encoding, &published, &[]))?;
    let mut writer = Publisher::new(evidence);
    let producer_ref = writer.record(&producer).map_err(|cause| {
        failure(
            Stage::Producer,
            Cause::Publisher(cause),
            &published,
            writer.attempted_refs(),
        )
    })?;
    let artifact_page = writer.artifact_pages(&artifacts).map_err(|cause| {
        failure(
            Stage::Inventory,
            Cause::Publisher(cause),
            &published,
            writer.attempted_refs(),
        )
    })?;
    Ok(Captured {
        producer,
        producer_ref,
        payloads,
        facts_payload,
        flags,
        artifacts,
        artifact_page,
    })
}

fn failure(stage: Stage, cause: Cause, published: &[PublishedPayload], attempted: &[Ref]) -> Error {
    Error {
        stage,
        cause,
        published_payloads: published.to_vec(),
        attempted_refs: attempted.to_vec(),
    }
}
fn stream_inputs(report: &NamespaceReport) -> [(Role, &Stream); 5] {
    [
        (Role::CandidateStdout, &report.observer.candidate_stdout),
        (Role::CandidateStderr, &report.observer.candidate_stderr),
        (Role::LauncherStdout, &report.process.stdout),
        (Role::LauncherStderr, &report.process.stderr),
        (Role::NativeStatus, &report.observer.native_stream),
    ]
}
fn publish_raw(
    evidence: &mut Evidence<'_>,
    role: Role,
    bytes: &[u8],
    media: &str,
    published: &mut Vec<PublishedPayload>,
) -> Result<Payload, Error> {
    let reference = evidence
        .payload(bytes, media)
        .map_err(|cause| failure(Stage::Payload(role), Cause::Sink(cause), published, &[]))?;
    published.push(PublishedPayload {
        role,
        reference: reference.clone(),
    });
    Ok(reference)
}
fn publish_streams(
    evidence: &mut Evidence<'_>,
    report: &NamespaceReport,
    published: &mut Vec<PublishedPayload>,
) -> Result<Payloads, Error> {
    let mut one = |role, stream: &Stream| {
        publish_raw(
            evidence,
            role,
            &stream.bytes,
            "application/octet-stream",
            published,
        )
    };
    Ok(Payloads {
        candidate_stdout: one(Role::CandidateStdout, &report.observer.candidate_stdout)?,
        candidate_stderr: one(Role::CandidateStderr, &report.observer.candidate_stderr)?,
        launcher_stdout: one(Role::LauncherStdout, &report.process.stdout)?,
        launcher_stderr: one(Role::LauncherStderr, &report.process.stderr)?,
        native_status: one(Role::NativeStatus, &report.observer.native_stream)?,
    })
}
fn artifact_rows(
    published: &[PublishedPayload],
    streams: &[(Role, &Stream); 5],
) -> Result<Vec<ArtifactV1>, receipt::Error> {
    published
        .iter()
        .map(|item| {
            let stream = streams
                .iter()
                .find(|(role, _)| *role == item.role)
                .map(|(_, stream)| *stream);
            let complete = stream.is_none_or(stream_complete);
            Ok(ArtifactV1 {
                object: item.reference.as_ref().clone(),
                role: Name::new(item.role.name())?,
                required: true,
                truncated: stream.is_some_and(|value| {
                    value.truncated || value.observed_bytes > value.bytes.len() as u64
                }),
                availability: ArtifactV1Availability::Available,
                reason: Text::new(if complete {
                    "exact_retained_bytes"
                } else {
                    "retained_stream_incomplete"
                })?,
            })
        })
        .collect()
}
fn stream_complete(stream: &Stream) -> bool {
    stream.eof
        && !stream.truncated
        && !stream.failed
        && stream.observed_bytes == stream.bytes.len() as u64
}
fn classification(
    facts: &NamespaceFacts,
    native: &Stream,
    json_exit_matches: bool,
) -> Option<NativeStatus> {
    (facts.release_sent
        && facts.protection_verified
        && facts.native_status_eof
        && facts.native_status_consistent
        && stream_complete(native)
        && json_exit_matches)
        .then_some(facts.native_status)
        .flatten()
}
fn producer(report: &NamespaceReport, payloads: &Payloads) -> Result<ProducerV1, receipt::Error> {
    let facts = &report.observer.facts;
    let native = classification(
        facts,
        &report.observer.native_stream,
        report.json_exit_matches,
    );
    producer_from(facts, native, report.process.interruption, payloads)
}
fn producer_from(
    facts: &NamespaceFacts,
    native: Option<NativeStatus>,
    interruption: Option<Interruption>,
    payloads: &Payloads,
) -> Result<ProducerV1, receipt::Error> {
    let status = producer_status(facts, native);
    let exit_code = match native {
        Some(NativeStatus::Exit { code, .. }) => Maybe::present(u32::from(code)),
        _ => Maybe::unavailable(Text::new("no_classified_native_exit")?),
    };
    let signal = match native {
        Some(NativeStatus::Signal { signal, .. }) => Maybe::present(u32::from(signal)),
        _ => Maybe::unavailable(Text::new("no_classified_native_signal")?),
    };
    Ok(ProducerV1 {
        status,
        exit_code,
        signal,
        timeout: interruption == Some(Interruption::Timeout),
        stdout: Maybe::present(payloads.candidate_stdout.clone()),
        stderr: Maybe::present(payloads.candidate_stderr.clone()),
    })
}
fn producer_status(facts: &NamespaceFacts, classified: Option<NativeStatus>) -> ProducerV1Status {
    match classified {
        Some(NativeStatus::Exit { .. }) => ProducerV1Status::Exited,
        Some(NativeStatus::Signal { .. }) => ProducerV1Status::Signalled,
        None if !facts.release_sent && facts.native_status.is_none() => {
            ProducerV1Status::NotStarted
        }
        None => ProducerV1Status::Unknown,
    }
}
fn flags(report: &NamespaceReport) -> Flags {
    Flags {
        candidate_stdout_complete: stream_complete(&report.observer.candidate_stdout),
        candidate_stderr_complete: stream_complete(&report.observer.candidate_stderr),
        launcher_stdout_complete: stream_complete(&report.process.stdout),
        launcher_stderr_complete: stream_complete(&report.process.stderr),
        native_stream_complete: stream_complete(&report.observer.native_stream),
        candidate_stderr_nonempty: !report.observer.candidate_stderr.bytes.is_empty(),
        launcher_stderr_nonempty: !report.process.stderr.bytes.is_empty(),
        native_classification_available: classification(
            &report.observer.facts,
            &report.observer.native_stream,
            report.json_exit_matches,
        )
        .is_some(),
        diagnostic_counting: DiagnosticCounting::Unmeasured,
    }
}

#[derive(Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "raw stream predicates remain independent"
)]
struct StreamFacts {
    observed_bytes: String,
    retained_bytes: String,
    eof: bool,
    truncated: bool,
    read_failed: bool,
    length_consistent: bool,
}
impl From<&Stream> for StreamFacts {
    fn from(value: &Stream) -> Self {
        Self {
            observed_bytes: value.observed_bytes.to_string(),
            retained_bytes: value.bytes.len().to_string(),
            eof: value.eof,
            truncated: value.truncated,
            read_failed: value.failed,
            length_consistent: value.observed_bytes == value.bytes.len() as u64,
        }
    }
}
#[derive(Serialize)]
struct Streams {
    candidate_stdout: StreamFacts,
    candidate_stderr: StreamFacts,
    launcher_stdout: StreamFacts,
    launcher_stderr: StreamFacts,
    native_status: StreamFacts,
}
#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum NativeFact {
    Exit { code: u8, raw: i32 },
    Signal { signal: u8, raw: i32, core: bool },
}
impl From<NativeStatus> for NativeFact {
    fn from(value: NativeStatus) -> Self {
        match value {
            NativeStatus::Exit { code, raw } => Self::Exit { code, raw },
            NativeStatus::Signal { signal, raw, core } => Self::Signal { signal, raw, core },
        }
    }
}
#[derive(Serialize)]
struct Fingerprint {
    path_bytes_hex: String,
    sha256_hex: String,
}
fn hex(bytes: &[u8]) -> String {
    let alphabet = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for &byte in bytes {
        output.push(char::from(alphabet[usize::from(byte >> 4)]));
        output.push(char::from(alphabet[usize::from(byte & 15)]));
    }
    output
}
fn fingerprints(values: &BTreeMap<PathBuf, [u8; 32]>) -> Vec<Fingerprint> {
    values
        .iter()
        .map(|(path, digest)| Fingerprint {
            path_bytes_hex: hex(path.as_os_str().as_bytes()),
            sha256_hex: hex(digest),
        })
        .collect()
}
#[derive(Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "retain every namespace observation without combining predicates"
)]
struct NamespaceFactsView<'a> {
    adversarial_isolation: bool,
    pid1: Option<u32>,
    shim_pid: Option<u32>,
    child_snapshot: &'a [u32],
    initial_namespace_ids: BTreeMap<&'a str, String>,
    source_sha256_before: Vec<Fingerprint>,
    source_sha256_after: Vec<Fingerprint>,
    source_sha256_postrun_host: Vec<Fingerprint>,
    before_mount_sha256: Option<String>,
    after_mount_sha256: Option<String>,
    root_device_inode: Option<[String; 2]>,
    public_files_verified: String,
    protected_paths_absent: String,
    descriptors_verified: bool,
    protection_verified: bool,
    release_sent: bool,
    terminal_json: bool,
    json_exit_code: Option<u8>,
    native_status: Option<NativeFact>,
    native_status_eof: bool,
    native_status_consistent: bool,
    pid1_terminal: bool,
    shim_terminal: bool,
    channel_cleanup_complete: bool,
    namespace_term_sent: bool,
    namespace_kill_sent: bool,
    unresolved: &'a [&'static str],
}
fn namespace_facts(facts: &NamespaceFacts) -> NamespaceFactsView<'_> {
    NamespaceFactsView {
        adversarial_isolation: facts.adversarial_isolation,
        pid1: facts.pid1,
        shim_pid: facts.shim_pid,
        child_snapshot: &facts.child_snapshot,
        initial_namespace_ids: facts
            .initial_namespace_ids
            .iter()
            .map(|(name, value)| (name.as_str(), value.to_string()))
            .collect(),
        source_sha256_before: fingerprints(&facts.source_sha256_before),
        source_sha256_after: fingerprints(&facts.source_sha256_after),
        source_sha256_postrun_host: fingerprints(&facts.source_sha256_postrun_host),
        before_mount_sha256: facts.before_mount_sha256.as_ref().map(|value| hex(value)),
        after_mount_sha256: facts.after_mount_sha256.as_ref().map(|value| hex(value)),
        root_device_inode: facts
            .root_device_inode
            .map(|(device, inode)| [device.to_string(), inode.to_string()]),
        public_files_verified: facts.public_files_verified.to_string(),
        protected_paths_absent: facts.protected_paths_absent.to_string(),
        descriptors_verified: facts.descriptors_verified,
        protection_verified: facts.protection_verified,
        release_sent: facts.release_sent,
        terminal_json: facts.terminal_json,
        json_exit_code: facts.json_exit_code,
        native_status: facts.native_status.map(Into::into),
        native_status_eof: facts.native_status_eof,
        native_status_consistent: facts.native_status_consistent,
        pid1_terminal: facts.pid1_terminal,
        shim_terminal: facts.shim_terminal,
        channel_cleanup_complete: facts.channel_cleanup_complete,
        namespace_term_sent: facts.namespace_term_sent,
        namespace_kill_sent: facts.namespace_kill_sent,
        unresolved: &facts.unresolved,
    }
}
#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum SignalFact {
    NotAttempted,
    Sent,
    Failed { errno: i32 },
    NotOwned,
}
impl From<SignalOutcome> for SignalFact {
    fn from(value: SignalOutcome) -> Self {
        match value {
            SignalOutcome::NotAttempted => Self::NotAttempted,
            SignalOutcome::Sent => Self::Sent,
            SignalOutcome::Failed(errno) => Self::Failed { errno },
            SignalOutcome::NotOwned => Self::NotOwned,
        }
    }
}
#[derive(Serialize)]
struct Signals {
    group_term: SignalFact,
    group_kill: SignalFact,
    direct_term: SignalFact,
    direct_kill: SignalFact,
}
#[derive(Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "reaping, group settlement, observer and pending ownership differ"
)]
struct Launcher {
    leader_pid: u32,
    exit_code: Option<i32>,
    signal: Option<i32>,
    interruption: Option<&'static str>,
    elapsed_nanos: String,
    interruption_elapsed_nanos: Option<String>,
    interruption_timestamp_observed: bool,
    interruption_timing_consistent: bool,
    signals: Signals,
    leader_reaped: bool,
    process_group_settled: bool,
    observer_ready: bool,
    pending_child: bool,
}
fn interruption(value: Interruption) -> &'static str {
    match value {
        Interruption::Cancelled => "cancelled",
        Interruption::Timeout => "timeout",
        Interruption::OutputLimit => "output_limit",
        Interruption::PipeError => "pipe_error",
        Interruption::WaitError => "wait_error",
        Interruption::ResidualGroup => "residual_group",
        Interruption::ObserverError => "observer_error",
    }
}
fn observer_failure(value: NamespaceError) -> &'static str {
    match value {
        NamespaceError::InvalidPlan => "invalid_plan",
        NamespaceError::Cancelled => "cancelled",
        NamespaceError::Deadline => "deadline",
        NamespaceError::Bound => "bound",
        NamespaceError::Io => "io",
        NamespaceError::Digest => "digest",
        NamespaceError::Channel => "channel",
        NamespaceError::Status => "status",
        NamespaceError::Pid => "pid",
        NamespaceError::Namespace => "namespace",
        NamespaceError::Child => "child",
        NamespaceError::Root => "root",
        NamespaceError::Descriptor => "descriptor",
        NamespaceError::Stream => "stream",
        NamespaceError::Resources => "resources",
        NamespaceError::Scratch => "scratch",
    }
}
#[derive(Serialize)]
struct ReportFacts<'a> {
    kind: &'static str,
    version: u8,
    scope: &'static str,
    namespace: NamespaceFactsView<'a>,
    launcher: Launcher,
    streams: Streams,
    flags: &'a Flags,
    observer_failure: Option<&'static str>,
    json_exit_matches: bool,
    postrun_sources_match: bool,
    resource_scope: Option<serde_json::Value>,
    scratch: Option<serde_json::Value>,
    scratch_retained: bool,
}
fn report_facts<'a>(report: &'a NamespaceReport, flags: &'a Flags) -> ReportFacts<'a> {
    let process = &report.process;
    let stop_elapsed = process
        .interruption_observed_at
        .and_then(|observed| observed.checked_duration_since(process.started_at));
    let stop_timing_consistent = match (
        process.interruption,
        process.interruption_observed_at,
        stop_elapsed,
    ) {
        (None, None, None) => true,
        (Some(_), Some(_), Some(elapsed)) => elapsed <= process.elapsed,
        _ => false,
    };
    ReportFacts {
        kind: "hee3.namespace-capture",
        version: 1,
        scope: "cooperative_development_observation",
        namespace: namespace_facts(&report.observer.facts),
        launcher: Launcher {
            leader_pid: process.leader_pid,
            exit_code: process.exit_code,
            signal: process.signal,
            interruption: process.interruption.map(interruption),
            elapsed_nanos: process.elapsed.as_nanos().to_string(),
            interruption_elapsed_nanos: stop_elapsed.map(|value| value.as_nanos().to_string()),
            interruption_timestamp_observed: process.interruption_observed_at.is_some(),
            interruption_timing_consistent: stop_timing_consistent,
            signals: Signals {
                group_term: process.signals.group_term.into(),
                group_kill: process.signals.group_kill.into(),
                direct_term: process.signals.direct_term.into(),
                direct_kill: process.signals.direct_kill.into(),
            },
            leader_reaped: process.leader_reaped,
            process_group_settled: process.process_group_settled,
            observer_ready: process.observer_ready,
            pending_child: process.pending.is_some(),
        },
        streams: Streams {
            candidate_stdout: (&report.observer.candidate_stdout).into(),
            candidate_stderr: (&report.observer.candidate_stderr).into(),
            launcher_stdout: (&process.stdout).into(),
            launcher_stderr: (&process.stderr).into(),
            native_status: (&report.observer.native_stream).into(),
        },
        flags,
        observer_failure: report.observer.failure().map(observer_failure),
        json_exit_matches: report.json_exit_matches,
        postrun_sources_match: report.postrun_sources_match,
        resource_scope: report.observer.resource_scope().map(|scope| {
            let facts = scope.facts();
            let limits = |value: &crate::worker::resources::Limits| serde_json::json!({
                "cpu_max": value.cpu_max, "memory_max": value.memory_max,
                "memory_swap_max": value.memory_swap_max, "pids_max": value.pids_max,
                "io_weight": value.io_weight,
            });
            serde_json::json!({
                "unit": facts.unit, "aggregate": facts.aggregate, "control_group": facts.control_group,
                "leader_pid": facts.leader_pid, "candidate": limits(&facts.candidate), "parent": limits(&facts.parent),
                "observed_elapsed_nanos": facts.observed_at.checked_duration_since(process.started_at).map(|value| value.as_nanos().to_string()),
            })
        }),
        scratch: report.observer.scratch_facts().map(|facts| serde_json::json!({
            "device": facts.device, "inode": facts.inode, "capacity_bytes": facts.capacity_bytes,
            "mountinfo_utf8": String::from_utf8_lossy(&facts.mountinfo),
        })),
        scratch_retained: report.observer.scratch_retained(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    fn proved(status: NativeStatus) -> (NamespaceFacts, Stream) {
        (
            NamespaceFacts {
                release_sent: true,
                protection_verified: true,
                native_status_eof: true,
                native_status_consistent: true,
                native_status: Some(status),
                ..NamespaceFacts::default()
            },
            Stream {
                bytes: b"native status fixture".to_vec(),
                observed_bytes: 21,
                eof: true,
                truncated: false,
                failed: false,
            },
        )
    }

    #[test]
    fn high_ordinary_exit_and_native_abort_keep_distinct_status_and_raw_wait() {
        let (exited, stream) = proved(NativeStatus::Exit {
            code: 134,
            raw: 34_304,
        });
        let native = classification(&exited, &stream, true);
        assert_eq!(
            native,
            Some(NativeStatus::Exit {
                code: 134,
                raw: 34_304
            })
        );
        assert_eq!(producer_status(&exited, native), ProducerV1Status::Exited);
        let (signalled, stream) = proved(NativeStatus::Signal {
            signal: 6,
            raw: 6,
            core: false,
        });
        let native = classification(&signalled, &stream, true);
        assert_eq!(
            producer_status(&signalled, native),
            ProducerV1Status::Signalled
        );
        assert_eq!(
            serde_json::to_value(namespace_facts(&signalled)).unwrap()["native_status"],
            serde_json::json!({"status":"signal","signal":6,"raw":6,"core":false})
        );
    }

    #[test]
    fn missing_native_status_distinguishes_unreleased_from_released_unknown() {
        let mut facts = NamespaceFacts::default();
        assert_eq!(producer_status(&facts, None), ProducerV1Status::NotStarted);
        facts.release_sent = true;
        assert_eq!(producer_status(&facts, None), ProducerV1Status::Unknown);
        assert!(classification(&facts, &Stream::default(), true).is_none());
    }

    fn empty_payloads() -> Payloads {
        let reference = Payload::new(Ref {
            artifact_id: receipt::Id::new("00000000-0000-4000-8000-000000000001").unwrap(),
            sha256: receipt::Sha::new(
                "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            )
            .unwrap(),
            byte_length: 0,
            media_type: Name::new("application/octet-stream").unwrap(),
            schema_id: Name::new("hee3.raw/1").unwrap(),
        })
        .unwrap();
        Payloads {
            candidate_stdout: reference.clone(),
            candidate_stderr: reference.clone(),
            launcher_stdout: reference.clone(),
            launcher_stderr: reference.clone(),
            native_status: reference,
        }
    }

    #[test]
    fn interruption_names_do_not_replace_previously_observed_native_status() {
        let (facts, stream) = proved(NativeStatus::Exit { code: 0, raw: 0 });
        let native = classification(&facts, &stream, true);
        let payloads = empty_payloads();
        let cancelled =
            producer_from(&facts, native, Some(Interruption::Cancelled), &payloads).unwrap();
        assert_eq!(cancelled.status, ProducerV1Status::Exited);
        assert_eq!(cancelled.exit_code.value, Some(0));
        assert!(!cancelled.timeout);
        let timed_out =
            producer_from(&facts, native, Some(Interruption::Timeout), &payloads).unwrap();
        assert_eq!(timed_out.status, ProducerV1Status::Exited);
        assert!(timed_out.timeout);
        assert_eq!(interruption(Interruption::Cancelled), "cancelled");
        assert_eq!(interruption(Interruption::Timeout), "timeout");
    }

    #[test]
    fn empty_incomplete_and_truncated_streams_never_become_measured_clean_diagnostics() {
        let mut stream = Stream::default();
        assert!(!stream_complete(&stream));
        stream.eof = true;
        assert!(stream_complete(&stream));
        stream.observed_bytes = 1;
        assert!(!stream_complete(&stream));
        stream.bytes.push(b'x');
        stream.truncated = true;
        assert!(!stream_complete(&stream));
        let facts = serde_json::to_value(StreamFacts::from(&stream)).unwrap();
        assert_eq!(facts["observed_bytes"], "1");
        assert_eq!(facts["retained_bytes"], "1");
        assert_eq!(facts["truncated"], true);
        assert_eq!(
            serde_json::to_value(DiagnosticCounting::Unmeasured).unwrap(),
            "unmeasured"
        );
    }

    #[test]
    fn unqualified_native_proof_remains_unknown() {
        let (mut facts, mut stream) = proved(NativeStatus::Exit { code: 0, raw: 0 });
        facts.protection_verified = false;
        assert!(classification(&facts, &stream, true).is_none());
        facts.protection_verified = true;
        stream.failed = true;
        assert!(classification(&facts, &stream, true).is_none());
        stream.failed = false;
        assert!(classification(&facts, &stream, false).is_none());
        facts.release_sent = false;
        assert_eq!(producer_status(&facts, None), ProducerV1Status::Unknown);
    }

    #[test]
    fn namespace_facts_preserve_non_utf8_source_paths_unresolved_and_cleanup() {
        let mut facts = NamespaceFacts::default();
        facts.source_sha256_before.insert(
            PathBuf::from(OsString::from_vec(vec![b'/', 255])),
            [0xab; 32],
        );
        facts.unresolved.push("channel_identity_changed");
        facts.namespace_term_sent = true;
        let value = serde_json::to_value(namespace_facts(&facts)).unwrap();
        assert_eq!(value["source_sha256_before"][0]["path_bytes_hex"], "2fff");
        assert_eq!(
            value["source_sha256_before"][0]["sha256_hex"],
            "ab".repeat(32)
        );
        assert_eq!(
            value["unresolved"],
            serde_json::json!(["channel_identity_changed"])
        );
        assert_eq!(value["namespace_term_sent"], true);
        assert_eq!(value["channel_cleanup_complete"], false);
        assert!(value["native_status"].is_null());
        assert_eq!(value.as_object().unwrap().len(), 27);
    }
}
