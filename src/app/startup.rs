//! Startup reconciliation: the application shell that feeds the pure policy in
//! `crate::recovery` real readbacks and acts on its closed decision.
//!
//! The pass opens the ledger for inspection, reads the bounded inventory and the
//! per-task facts the inventory does not carry (terminal event ordinals, evidence
//! retrievability), then reopens the ledger normally when its mode permits and
//! re-reads the same inventory before acting. Physical facts come only through
//! the [`Physical`] seam, so a test hands the pass a recording double and the
//! runtime hands it [`Host`]. Every decision is recorded in the journal before
//! any effect, every effect is read back afterwards, and a record whose content
//! already exists is found rather than written again.

use crate::contracts::UuidV4;
use crate::contracts::roster::{Instance, ReceiptTime};
use crate::recovery::{
    self as policy, AcceptanceCandidate, Acknowledgement, AttemptFacts, AttemptState, Cleanup,
    CleanupReadback, CleanupTarget, Clock, CursorFacts, Decision, Dimension, Effect, Evidence,
    Lease, LedgerFacts, Mode, ObservationClaim, Observations, PiQueueCustody, ProcessCustody,
    Reconciliation, ReuseRefusal, TaskFacts, TaskHistory, TaskState, Verdict, Verification,
    WorkspaceReadback,
};
use crate::store::{
    self, DurableAttempt, DurableTask, DurableVerification, EvidenceAvailability,
    ReconciliationRecord, RecordKind, Recorded, RecoveryInventory, RecoveryLimits, Store,
    TerminalOrdinals,
};
use crate::worker::pi::{Binding, Command, Frame, Observation, Session, validate_record};
use rustix::process::{Pid, PidfdFlags, pidfd_open};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ---- process identity: the one classifier, shared with the development inspector --------

/// Kernel-described identity of one process, as a worker's roster observation
/// records it in `actual_identity`: the PID, field 22 of `/proc/<pid>/stat`
/// (start time in clock ticks) and the PID namespace link. Any other
/// `actual_identity` text (an executable path, a digest) is not a process
/// identity and is never classified.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub start_ticks: u64,
    pub namespace: String,
}

impl ProcessIdentity {
    /// Parse one roster observation's `actual_identity` text. `None` means the
    /// text is not a process identity; it never means the process is absent.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        if text.len() > 256 {
            return None;
        }
        let value: Self = serde_json::from_str(text).ok()?;
        (value.pid >= 1 && namespace_link(&value.namespace)).then_some(value)
    }
}

fn namespace_link(text: &str) -> bool {
    text.strip_prefix("pid:[")
        .and_then(|rest| rest.strip_suffix(']'))
        .is_some_and(|digits| {
            !digits.is_empty() && digits.len() <= 20 && digits.bytes().all(|b| b.is_ascii_digit())
        })
}

/// What `/proc/<pid>` says about the PID an observation named. It carries no
/// PID of its own: the read was made for the observed PID, so the policy cannot
/// compare two different PIDs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LiveIdentity {
    pub start_ticks: u64,
    pub namespace: String,
    /// Field 3 of `/proc/<pid>/stat`, retained for the reader; the policy does
    /// not classify on it (a zombie holding the same identity reads as live).
    pub state: char,
}

/// The outcome of one bounded `/proc` read: the classifier's only world input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveRead {
    Present(LiveIdentity),
    /// No such PID (`ENOENT`/`ESRCH`).
    Absent,
    /// Read refused or malformed; the text is kept, never interpreted.
    Unreadable(String),
}

/// The pure classifier: from what the ledger observed and what `/proc` now says
/// to exactly one custody arm. No I/O, no clock, no authority.
#[must_use]
pub fn classify(observed: &ProcessIdentity, live: &LiveRead) -> ProcessCustody {
    match live {
        LiveRead::Unreadable(error) => ProcessCustody::Unreadable {
            error: error.clone(),
        },
        LiveRead::Absent => ProcessCustody::Absent,
        LiveRead::Present(identity) => {
            let mut differs = Vec::new();
            if identity.start_ticks != observed.start_ticks {
                differs.push(Dimension::StartTicks);
            }
            if identity.namespace != observed.namespace {
                differs.push(Dimension::Namespace);
            }
            if differs.is_empty() {
                ProcessCustody::LiveSameIdentity
            } else {
                ProcessCustody::PidReused { differs }
            }
        }
    }
}

const PROC_STAT_LIMIT: u64 = 4096;

/// Thin I/O: one bounded read of `/proc/<pid>/stat` and one of `/proc/<pid>/ns/pid`.
/// A passed deadline reads nothing and is reported as unreadable.
#[must_use]
pub fn live_read(pid: u32, deadline: Instant) -> LiveRead {
    if Instant::now() >= deadline {
        return LiveRead::Unreadable("inspection deadline".into());
    }
    let stat = match read_stat(pid) {
        Ok(text) => text,
        Err(error) => return proc_read_failure(&error, "stat"),
    };
    let Some((state, start_ticks)) = parse_stat(&stat) else {
        return LiveRead::Unreadable("malformed stat".into());
    };
    let namespace = match fs::read_link(format!("/proc/{pid}/ns/pid")) {
        Ok(link) => link,
        Err(error) => return proc_read_failure(&error, "ns"),
    };
    let Some(namespace) = namespace.to_str() else {
        return LiveRead::Unreadable("non-UTF-8 namespace link".into());
    };
    LiveRead::Present(LiveIdentity {
        start_ticks,
        namespace: namespace.into(),
        state,
    })
}

/// Whether a `/proc` read failed because the process is not there, as opposed
/// to because it could not be read. Only `ENOENT` and `ESRCH` mean gone; every
/// other errno is an unreadable process, never an absent one — reporting a
/// permission failure as absence would let a custody decision be made from a
/// read that never happened. Public so the distinction is reachable by argument
/// rather than only by arranging a `/proc` entry this uid cannot read (F95).
#[must_use]
pub fn gone(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::NotFound
        || error.raw_os_error() == Some(rustix::io::Errno::SRCH.raw_os_error())
}

/// What one failed `/proc` read means for [`live_read`]: absent when the process is
/// [`gone`], otherwise unreadable with the read named. The whole decision, extracted from
/// the shell so each outcome is reachable by argument (F95): as match guards inside
/// `live_read` they could be replaced by `true` or `false` with no test able to notice,
/// because only an arranged `/proc` entry reaches the non-gone branch.
#[must_use]
pub fn proc_read_failure(error: &io::Error, what: &str) -> LiveRead {
    if gone(error) {
        LiveRead::Absent
    } else {
        LiveRead::Unreadable(format!("{what}: {error}"))
    }
}

/// What one `symlink_metadata` read says about a path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Presence {
    /// The path exists.
    Present,
    /// `ENOENT`: the path is not there.
    Absent,
    /// Any other failure: the read did not happen, so nothing is known.
    Unreadable,
}

/// Classify one path read. Only `NotFound` is absence; every other error is an unread
/// path, never a released one -- reporting a permission failure as absence would settle a
/// cleanup from a read that never happened. Pure, so each arm is reachable by argument.
#[must_use]
pub fn presence(read: &io::Result<()>) -> Presence {
    match read {
        Ok(()) => Presence::Present,
        Err(error) if error.kind() == io::ErrorKind::NotFound => Presence::Absent,
        Err(_) => Presence::Unreadable,
    }
}

fn read_stat(pid: u32) -> io::Result<String> {
    let mut text = String::new();
    File::open(format!("/proc/{pid}/stat"))?
        .take(PROC_STAT_LIMIT)
        .read_to_string(&mut text)?;
    if u64::try_from(text.len())
        .ok()
        .is_none_or(|length| length >= PROC_STAT_LIMIT)
    {
        return Err(io::Error::other("stat bound"));
    }
    Ok(text)
}

/// proc(5): field 2 `comm` may hold spaces and parentheses, so fields are
/// counted after the last `)`: field 3 is the state, field 22 the start ticks.
#[must_use]
pub fn parse_stat(stat: &str) -> Option<(char, u64)> {
    let close = stat.rfind(')')?;
    let mut fields = stat.get(close + 1..)?.split_whitespace();
    let mut state = fields.next()?.chars();
    let (first, second) = (state.next()?, state.next());
    if second.is_some() {
        return None;
    }
    let start_ticks = fields.nth(18)?.parse().ok()?;
    Some((first, start_ticks))
}

// ---- the seam ------------------------------------------------------------------------------

/// The attempt a physical question is asked about, as the ledger names it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Subject<'a> {
    pub task: &'a str,
    pub attempt: &'a str,
    pub generation: u64,
    pub workspace_ref: Option<&'a str>,
    pub session: Option<&'a str>,
}

/// An owned copy of a [`Subject`], for records and doubles.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubjectValue {
    pub task: String,
    pub attempt: String,
    pub generation: u64,
    pub workspace_ref: Option<String>,
    pub session: Option<String>,
}

impl Subject<'_> {
    #[must_use]
    pub fn to_value(&self) -> SubjectValue {
        SubjectValue {
            task: self.task.to_owned(),
            attempt: self.attempt.to_owned(),
            generation: self.generation,
            workspace_ref: self.workspace_ref.map(str::to_owned),
            session: self.session.map(str::to_owned),
        }
    }
}

/// The physical world as the pass reads and changes it. Readbacks answer from
/// the world; effects change it. Nothing here decides anything.
pub trait Physical {
    /// One read of `/proc` for the identity the ledger retained for this attempt.
    fn process(&mut self, subject: &Subject<'_>, identity: &ProcessIdentity) -> LiveRead;
    /// The attempt's Pi message queue, from a live session or `Unreconciled`.
    fn pi_queue(&mut self, subject: &Subject<'_>) -> PiQueueCustody;
    /// What remains of the attempt's cleanup obligations.
    fn cleanup(&mut self, subject: &Subject<'_>) -> CleanupReadback;
    /// Whether the attempt's workspace is released or still writable.
    fn workspace(&mut self, subject: &Subject<'_>) -> WorkspaceReadback;
    /// The dispatch acknowledgement the caller retained for this attempt, if any.
    fn acknowledgement(&mut self, subject: &Subject<'_>) -> Acknowledgement;
    /// A receiver clock reading in its own epoch, when one is available.
    fn clock(&mut self) -> Option<ReceiptTime>;
    /// Effect: attach observation to a live worker of the given identity. Never dispatches.
    /// # Errors
    /// The world's own reason the observation could not be attached.
    fn attach(&mut self, subject: &Subject<'_>, identity: &ProcessIdentity) -> Result<(), String>;
    /// Effect: perform one named remaining cleanup obligation.
    /// # Errors
    /// The world's own reason the obligation could not be performed.
    fn clean(&mut self, subject: &Subject<'_>, target: &str) -> Result<(), String>;
}

// ---- the runtime world -----------------------------------------------------------------------

/// A live Pi session transport: one line out, frames in. Owned by the caller.
pub struct PiLink {
    pub reader: Box<dyn BufRead>,
    pub writer: Box<dyn Write>,
}

/// The runtime's [`Physical`]: `/proc` for process identity, the caller's
/// materialized workspace directories for cleanup and workspace readback, the
/// caller's live Pi transports for the queue, and the caller's retained
/// acknowledgements. Nothing is inferred from a naming convention: an attempt
/// with no entry is `NotRead`/`Unreconciled`/`Unrecorded`.
pub struct Host {
    /// Attempt id → the workspace directory the application materialized for it.
    pub workspaces: BTreeMap<String, PathBuf>,
    /// Attempt id → the dispatch acknowledgement the application retained.
    pub acknowledgements: BTreeMap<String, Acknowledgement>,
    /// Attempt id → a live Pi session transport for it.
    pub pi: BTreeMap<String, PiLink>,
    /// PID file descriptors held after an attached observation (the effect).
    attached: BTreeMap<String, OwnedFd>,
    deadline: Instant,
}

impl Host {
    #[must_use]
    pub fn new(deadline: Instant) -> Self {
        Self {
            workspaces: BTreeMap::new(),
            acknowledgements: BTreeMap::new(),
            pi: BTreeMap::new(),
            attached: BTreeMap::new(),
            deadline,
        }
    }
    /// The attempts whose observation this host currently holds a handle for.
    #[must_use]
    pub fn attached(&self) -> Vec<&str> {
        self.attached.keys().map(String::as_str).collect()
    }
}

const WALK_ENTRY_LIMIT: usize = 4096;
const WALK_DEPTH_LIMIT: usize = 16;

/// How long one workspace removal may walk. The owner's entry and depth bounds cap the work;
/// this caps the wall time of an effect that `Physical::clean` has no deadline for.
const WORKSPACE_REMOVAL_BUDGET: std::time::Duration = std::time::Duration::from_secs(30);

fn owned_private_dir(path: &Path) -> Result<fs::Metadata, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| format!("metadata: {e}"))?;
    if !meta.is_dir() {
        return Err("not a directory".into());
    }
    if meta.uid() != rustix::process::geteuid().as_raw() {
        return Err("not owned by this uid".into());
    }
    if meta.mode() & 0o777 != 0o700 {
        return Err(format!("mode {:o} is not 0700", meta.mode() & 0o777));
    }
    Ok(meta)
}

fn walk_bytes(path: &Path, depth: usize, entries: &mut usize) -> Result<u64, String> {
    if depth > WALK_DEPTH_LIMIT {
        return Err("depth bound".into());
    }
    let mut total = 0_u64;
    for entry in fs::read_dir(path).map_err(|e| format!("read_dir: {e}"))? {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        *entries += 1;
        if *entries > WALK_ENTRY_LIMIT {
            return Err("entry bound".into());
        }
        let meta = entry
            .metadata()
            .map_err(|e| format!("entry metadata: {e}"))?;
        if meta.file_type().is_dir() {
            total = total
                .checked_add(walk_bytes(&entry.path(), depth + 1, entries)?)
                .ok_or("byte bound")?;
        } else if meta.file_type().is_file() {
            total = total.checked_add(meta.len()).ok_or("byte bound")?;
        }
    }
    Ok(total)
}

fn read_frame(link: &mut PiLink) -> Result<Frame, String> {
    let mut bytes = Vec::new();
    let reader: &mut dyn BufRead = &mut *link.reader;
    let count = reader
        .take(4097)
        .read_until(b'\n', &mut bytes)
        .map_err(|e| format!("read: {e}"))?;
    if count == 0 {
        return Err("eof".into());
    }
    if bytes.pop() != Some(b'\n') || bytes.len() > 4096 {
        return Err("frame bound".into());
    }
    let frame = Frame::parse(&bytes).map_err(|e| format!("frame: {e:?}"))?;
    validate_record(&frame).map_err(|e| format!("record: {e:?}"))?;
    Ok(frame)
}

fn pi_queue_over(link: &mut PiLink, subject: &Subject<'_>) -> Result<PiQueueCustody, String> {
    let task = UuidV4::parse(subject.task).map_err(|e| format!("task id: {e:?}"))?;
    let attempt = UuidV4::parse(subject.attempt).map_err(|e| format!("attempt id: {e:?}"))?;
    let generation = subject
        .generation
        .to_string()
        .parse()
        .map_err(|e| format!("generation: {e:?}"))?;
    let mut session = Session::new(Binding {
        task,
        attempt,
        generation,
    });
    let request = session
        .issue(Command::State, 0)
        .map_err(|e| format!("issue: {e:?}"))?;
    link.writer
        .write_all(&request)
        .and_then(|()| link.writer.flush())
        .map_err(|e| format!("write: {e}"))?;
    let frame = read_frame(link)?;
    let state = match session
        .observe(&frame, 1)
        .map_err(|e| format!("observe: {e:?}"))?
    {
        Observation::State(state) => state,
        other => return Err(format!("unexpected observation {other:?}")),
    };
    // `Session::readback` admits a fresh session only against an idle vendor
    // session with an empty queue and refuses a busy one (`Refusal::Session`), so
    // the codec has already decided: an accepted state is an idle queue, and a
    // queue this pass cannot adopt stays unreconciled through the refusal above.
    Ok(if state.idle {
        PiQueueCustody::Idle
    } else {
        PiQueueCustody::Unreconciled
    })
}

impl Physical for Host {
    fn process(&mut self, _subject: &Subject<'_>, identity: &ProcessIdentity) -> LiveRead {
        live_read(identity.pid, self.deadline)
    }
    fn pi_queue(&mut self, subject: &Subject<'_>) -> PiQueueCustody {
        match self.pi.get_mut(subject.attempt) {
            None => PiQueueCustody::Unreconciled,
            Some(link) => pi_queue_over(link, subject).unwrap_or(PiQueueCustody::Unreconciled),
        }
    }
    fn cleanup(&mut self, subject: &Subject<'_>) -> CleanupReadback {
        let Some(path) = self.workspaces.get(subject.attempt) else {
            return CleanupReadback::NotRead;
        };
        match presence(&fs::symlink_metadata(path).map(|_| ())) {
            Presence::Present => CleanupReadback::Partial {
                remaining: vec!["workspace".into()],
            },
            Presence::Absent => CleanupReadback::Complete,
            Presence::Unreadable => CleanupReadback::NotRead,
        }
    }
    fn workspace(&mut self, subject: &Subject<'_>) -> WorkspaceReadback {
        let Some(path) = self.workspaces.get(subject.attempt) else {
            return WorkspaceReadback::NotRead;
        };
        match presence(&fs::symlink_metadata(path).map(|_| ())) {
            Presence::Absent => WorkspaceReadback::Released,
            Presence::Unreadable => WorkspaceReadback::NotRead,
            Presence::Present => {
                if owned_private_dir(path).is_err() {
                    return WorkspaceReadback::NotRead;
                }
                let mut entries = 0;
                match walk_bytes(path, 0, &mut entries) {
                    Ok(bytes) => WorkspaceReadback::Writable { bytes },
                    Err(_) => WorkspaceReadback::NotRead,
                }
            }
        }
    }
    fn acknowledgement(&mut self, subject: &Subject<'_>) -> Acknowledgement {
        self.acknowledgements
            .get(subject.attempt)
            .copied()
            .unwrap_or(Acknowledgement::Unrecorded)
    }
    fn clock(&mut self) -> Option<ReceiptTime> {
        // No receiver clock epoch survives a restart; a lease is compared only in
        // its own epoch, so the startup answer is "unavailable", never a guess.
        None
    }
    fn attach(&mut self, subject: &Subject<'_>, identity: &ProcessIdentity) -> Result<(), String> {
        let pid = i32::try_from(identity.pid)
            .ok()
            .and_then(Pid::from_raw)
            .ok_or("pid out of range")?;
        let fd = pidfd_open(pid, PidfdFlags::empty()).map_err(|e| format!("pidfd_open: {e}"))?;
        match live_read(identity.pid, self.deadline) {
            LiveRead::Present(live)
                if live.start_ticks == identity.start_ticks
                    && live.namespace == identity.namespace =>
            {
                self.attached.insert(subject.attempt.to_owned(), fd);
                Ok(())
            }
            other => Err(format!("identity changed under the handle: {other:?}")),
        }
    }
    fn clean(&mut self, subject: &Subject<'_>, target: &str) -> Result<(), String> {
        if target != "workspace" {
            return Err(format!("no such obligation: {target}"));
        }
        let path = self
            .workspaces
            .get(subject.attempt)
            .ok_or("no workspace bound for this attempt")?;
        if !path.is_absolute() {
            return Err("workspace path is not absolute".into());
        }
        // The workspace owner removes it, descriptor-relative: the path is opened once, its
        // custody read from that descriptor, and the name re-checked before the final unlink
        // (review N6; this shell used to check the path and then remove the path).
        crate::worker::workspace::remove_owned(
            path,
            std::time::Instant::now() + WORKSPACE_REMOVAL_BUDGET,
        )
        .map_err(|error| format!("remove: {error:?}"))
    }
}

// ---- inputs the caller retains --------------------------------------------------------------

/// A late observation or result the caller retained, claiming an identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Claim {
    pub attempt: String,
    pub epoch: String,
    pub task_generation: u64,
    pub attempt_generation: u64,
}

/// An event cursor the caller retained, offered to this ledger.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Cursor {
    pub epoch: String,
    pub sequence: u64,
}

/// One startup pass over one ledger generation.
#[derive(Clone, Copy, Debug)]
pub struct Startup<'a> {
    pub root: &'a Path,
    pub generation: UuidV4<'a>,
    pub epoch: UuidV4<'a>,
    pub limits: RecoveryLimits,
    /// The epoch this ledger was restored from, when the caller holds the marker.
    pub restored_from: Option<&'a str>,
    pub cursors: &'a [Cursor],
    pub claims: &'a [Claim],
    pub deadline: Instant,
}

// ---- the receipt ---------------------------------------------------------------------------

/// The task history as handed to the policy, owned for the record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "history")]
pub enum HistoryValue {
    Open,
    Cancelled {
        ordinal: Option<u64>,
    },
    Accepted {
        event: String,
        ordinal: Option<u64>,
    },
    Both {
        cancellation: u64,
        acceptance_event: String,
        acceptance: u64,
    },
    Contradictory,
}

/// Exactly what the shell handed the policy for one attempt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Handed {
    pub subject: SubjectValue,
    pub identity: Option<ProcessIdentity>,
    pub process: ProcessCustody,
    pub pi_queue: PiQueueCustody,
    pub cleanup: CleanupReadback,
    pub workspace: WorkspaceReadback,
    pub acknowledgement: Acknowledgement,
    /// Only the epoch is recorded durably: the reading itself changes between
    /// passes and would defeat the content-derived record id.
    pub clock_epoch: Option<String>,
    pub claim: Option<Claim>,
    pub evidence: Evidence,
    pub history: HistoryValue,
    pub task_state: TaskState,
    pub attempt_state: AttemptState,
}

/// One performed cleanup obligation and what the world said.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CleanupEffect {
    pub target: String,
    pub performed: bool,
    pub error: Option<String>,
}

/// What the pass did with a decision. Every arm is a record; some carry effects.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum Action {
    /// R01/R02: the retained claim is stale; recorded, nothing else.
    StaleRefused,
    /// Recorded as unknown; no effect.
    RetainedUnknown,
    /// R06: observation attached to the live worker, then read back. Never a dispatch.
    ObservationAttached {
        generation: u64,
        readback: ProcessCustody,
    },
    AttachRefused {
        generation: u64,
        error: String,
    },
    /// R11: intent recorded first, the named obligations performed, cleanup read back,
    /// and the ledger's cleanup column settled only when the readback says complete.
    CleanupPerformed {
        targets: Vec<CleanupTarget>,
        effects: Vec<CleanupEffect>,
        readback: CleanupReadback,
        ledger_settled: bool,
        ledger_refusal: Option<String>,
    },
    /// R11 on a terminal task: recorded; releasing a workspace is a dispatch decision, not this pass's.
    ReleasableRecorded {
        task_state: TaskState,
    },
    /// R09: recorded by reason; no reuse, no effect.
    ReuseRefused {
        reason: ReuseRefusal,
    },
    /// R12: recorded; verification is the verifier's, not this pass's.
    VerificationOutstanding {
        task_state: TaskState,
    },
    /// R03/R04: the committed acceptance is history; the ledger head is read back against it.
    AcceptanceStands {
        event: String,
        head_event: Option<String>,
        agrees: bool,
    },
    /// R03/R05: the committed cancellation stands; a prepared acceptance is named as rejected.
    CancellationStands {
        rejected_acceptance: Option<String>,
        head_cancellation: bool,
        agrees: bool,
    },
    /// A cursor decision handed to the attempt actor: recorded as such, nothing done.
    NotAnAttemptDecision,
}

impl Action {
    /// The one spelling of each arm.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::StaleRefused => "stale_refused",
            Self::RetainedUnknown => "retained_unknown",
            Self::ObservationAttached { .. } => "observation_attached",
            Self::AttachRefused { .. } => "attach_refused",
            Self::CleanupPerformed { .. } => "cleanup_performed",
            Self::ReleasableRecorded { .. } => "releasable_recorded",
            Self::ReuseRefused { .. } => "reuse_refused",
            Self::VerificationOutstanding { .. } => "verification_outstanding",
            Self::AcceptanceStands { .. } => "acceptance_stands",
            Self::CancellationStands { .. } => "cancellation_stands",
            Self::NotAnAttemptDecision => "not_an_attempt_decision",
        }
    }
}

/// How the ledger could be reached for effects.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "ledger")]
pub enum LedgerAccess {
    /// Normal mode: decisions are recorded and effects performed.
    Writable,
    /// Reconciliation mode: the ledger refuses normal opening, so decisions are
    /// reported here and neither recorded nor acted on.
    InspectionOnly,
}

/// One attempt's entry in the receipt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Entry {
    pub task: String,
    pub attempt: String,
    pub handed: Handed,
    pub decision: Decision,
    pub rule: &'static str,
    /// `None` when the ledger was inspection-only: decided, not acted.
    pub action: Option<Action>,
    pub records: Vec<Recorded>,
}

/// One retained cursor's entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CursorEntry {
    pub cursor: Cursor,
    pub decision: Decision,
    pub rule: &'static str,
}

/// The receipt of one pass.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Pass {
    pub epoch: String,
    pub generation: String,
    pub mode: Mode,
    pub event_high_water: u64,
    pub ledger: LedgerAccess,
    pub attempts: Vec<Entry>,
    pub cursors: Vec<CursorEntry>,
    /// Journal rows this pass inserted (records found already present are not counted).
    pub writes: u64,
    /// Whether any decision permitted execution; the policy never does, and the
    /// pass reads the flag from every decision rather than assuming it.
    pub permits_execution: bool,
}

#[derive(Debug)]
pub enum Error {
    Store(store::Error),
    /// A ledger row spelled a value the policy does not accept.
    Facts {
        attempt: String,
        field: &'static str,
    },
    /// The inventory read for action differs from the one read for inspection.
    Changed,
    Encoding(serde_json::Error),
}

impl From<store::Error> for Error {
    fn from(value: store::Error) -> Self {
        Self::Store(value)
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Encoding(value)
    }
}

// ---- the pass ------------------------------------------------------------------------------

/// Everything one read of the ledger returns. Compared WHOLE between the inspection read
/// and the action read, by the derived equality: a field-by-field `||` chain let mutation
/// shard C weaken either link to `&&` unnoticed, and would have silently skipped any field
/// added here later. The derive cannot forget one.
#[derive(PartialEq)]
struct Inspected {
    inventory: RecoveryInventory,
    ordinals: BTreeMap<String, TerminalOrdinals>,
    evidence: BTreeMap<String, EvidenceAvailability>,
}

fn read(store: &mut Store, startup: &Startup<'_>) -> Result<Inspected, Error> {
    let inventory = store.recovery_inventory(startup.epoch, startup.limits, startup.deadline)?;
    let mut ordinals = BTreeMap::new();
    for task in &inventory.tasks {
        let id = UuidV4::parse(&task.head.id).map_err(|_| Error::Facts {
            attempt: task.head.id.clone(),
            field: "task id",
        })?;
        ordinals.insert(
            task.head.id.clone(),
            store.terminal_ordinals(id, startup.deadline)?,
        );
    }
    let mut evidence = BTreeMap::new();
    for row in &inventory.verifications {
        if !evidence.contains_key(&row.evidence) {
            evidence.insert(
                row.evidence.clone(),
                store.evidence_available(&row.evidence, startup.deadline)?,
            );
        }
    }
    Ok(Inspected {
        inventory,
        ordinals,
        evidence,
    })
}

/// The ledger read for action must equal the one inspected.
///
/// Extracted from [`run`] (review §3, T07): the branch is reachable only when another writer
/// changes the ledger between the inspection read and the writable read, which no input can
/// arrange, so the rule is a value function a test reaches directly; `run` makes one call.
///
/// # Errors
///
/// [`Error::Changed`] when the two reads differ.
pub fn confirm_unchanged<T: PartialEq>(inspected: &T, reread: &T) -> Result<(), Error> {
    if reread == inspected {
        Ok(())
    } else {
        Err(Error::Changed)
    }
}

/// Run one startup pass. Inspection first; effects only through a normally
/// opened ledger whose inventory equals the inspected one.
/// # Errors
/// Store refusals, an unparseable ledger row, or a ledger that changed between
/// the inspection read and the action read.
pub fn run(startup: &Startup<'_>, physical: &mut dyn Physical) -> Result<Pass, Error> {
    let inspected = {
        let mut store = Store::open_inspection(
            startup.root,
            startup.generation,
            startup.epoch,
            startup.deadline,
        )?;
        read(&mut store, startup)?
    };
    let mode = Mode::parse(&inspected.inventory.mode).ok_or_else(|| Error::Facts {
        attempt: String::new(),
        field: "ledger mode",
    })?;
    let mut writable = match mode {
        Mode::Normal => {
            let mut store = Store::open(
                startup.root,
                startup.generation,
                startup.epoch,
                false,
                startup.deadline,
            )?;
            confirm_unchanged(&inspected, &read(&mut store, startup)?)?;
            Some(store)
        }
        Mode::Reconciliation => None,
    };
    let ledger = LedgerFacts {
        epoch: &inspected.inventory.epoch,
        mode,
        event_high_water: inspected.inventory.event_high_water,
        restored_from: startup.restored_from,
    };
    let clock = physical.clock();
    let mut pass = Pass {
        epoch: inspected.inventory.epoch.clone(),
        generation: inspected.inventory.generation.clone(),
        mode,
        event_high_water: inspected.inventory.event_high_water,
        ledger: if writable.is_some() {
            LedgerAccess::Writable
        } else {
            LedgerAccess::InspectionOnly
        },
        attempts: Vec::new(),
        cursors: Vec::new(),
        writes: 0,
        permits_execution: false,
    };
    for attempt in &inspected.inventory.attempts {
        let entry = reconcile_attempt(
            &ledger,
            &inspected,
            attempt,
            startup,
            physical,
            clock.as_ref(),
            writable.as_mut(),
        )?;
        pass.permits_execution |= entry.decision.reconciliation.permits_execution();
        for record in &entry.records {
            if record.fresh {
                pass.writes += 1;
            }
        }
        pass.attempts.push(entry);
    }
    for cursor in startup.cursors {
        let decision = policy::reconcile_cursor(
            &ledger,
            &CursorFacts {
                epoch: &cursor.epoch,
                sequence: cursor.sequence,
            },
        );
        pass.permits_execution |= decision.reconciliation.permits_execution();
        pass.cursors.push(CursorEntry {
            cursor: cursor.clone(),
            rule: decision.rule.id(),
            decision,
        });
    }
    Ok(pass)
}

fn facts_error(attempt: &str, field: &'static str) -> Error {
    Error::Facts {
        attempt: attempt.to_owned(),
        field,
    }
}

fn generation_of(text: &str, attempt: &str, field: &'static str) -> Result<u64, Error> {
    text.parse::<crate::contracts::Generation>()
        .map(crate::contracts::Generation::value)
        .map_err(|_| facts_error(attempt, field))
}

fn history_value(history: &TaskHistory<'_>) -> HistoryValue {
    match *history {
        TaskHistory::Open => HistoryValue::Open,
        TaskHistory::Cancelled { ordinal } => HistoryValue::Cancelled { ordinal },
        TaskHistory::Accepted { event, ordinal } => HistoryValue::Accepted {
            event: event.to_owned(),
            ordinal,
        },
        TaskHistory::Both {
            cancellation,
            acceptance_event,
            acceptance,
        } => HistoryValue::Both {
            cancellation,
            acceptance_event: acceptance_event.to_owned(),
            acceptance,
        },
        TaskHistory::Contradictory => HistoryValue::Contradictory,
    }
}

/// The task's history from its flags and, where the journal holds them, the
/// commit ordinals; the policy orders two commits, this function only reports them.
fn task_history<'a>(task: &'a DurableTask, ordinals: Option<&TerminalOrdinals>) -> TaskHistory<'a> {
    let accepted_ordinal = ordinals
        .and_then(|o| o.accepted.as_ref())
        .map(|(_, seq)| *seq);
    let cancellation_ordinal = ordinals.and_then(|o| o.cancellation);
    match (task.head.cancellation, task.head.accepted_event.as_deref()) {
        (true, Some(event)) => match (cancellation_ordinal, accepted_ordinal) {
            (Some(cancellation), Some(acceptance)) => TaskHistory::Both {
                cancellation,
                acceptance_event: event,
                acceptance,
            },
            _ => TaskHistory::Contradictory,
        },
        (false, Some(event)) => TaskHistory::Accepted {
            event,
            ordinal: accepted_ordinal,
        },
        (true, None) => TaskHistory::Cancelled {
            ordinal: cancellation_ordinal,
        },
        (false, None) => TaskHistory::Open,
    }
}

fn task_facts<'a>(
    task: &'a DurableTask,
    verification: Option<&'a DurableVerification>,
    ordinals: Option<&TerminalOrdinals>,
    attempt: &str,
) -> Result<TaskFacts<'a>, Error> {
    let candidate = match verification {
        Some(row) if row.verdict == "passed" && task.head.accepted_event.is_none() => {
            AcceptanceCandidate::Prepared {
                verification_event: &row.event,
            }
        }
        _ => AcceptanceCandidate::None,
    };
    Ok(TaskFacts {
        generation: generation_of(&task.head.generation, attempt, "task generation")?,
        state: TaskState::parse(&task.head.state)
            .ok_or_else(|| facts_error(attempt, "task state"))?,
        history: task_history(task, ordinals),
        candidate,
    })
}

fn attempt_facts<'a>(
    attempt: &'a DurableAttempt,
    verification: Option<&'a DurableVerification>,
    instance: Option<&'a Instance>,
    evidence: Evidence,
    acknowledgement: Acknowledgement,
) -> Result<AttemptFacts<'a>, Error> {
    let id = attempt.id.as_str();
    Ok(AttemptFacts {
        id,
        generation: generation_of(&attempt.generation, id, "attempt generation")?,
        state: AttemptState::parse(&attempt.state)
            .ok_or_else(|| facts_error(id, "attempt state"))?,
        effect: Effect::parse(&attempt.effect).ok_or_else(|| facts_error(id, "effect"))?,
        cleanup: Cleanup::parse(&attempt.cleanup).ok_or_else(|| facts_error(id, "cleanup"))?,
        acknowledgement,
        lease: instance.map_or(Lease::NotLeased, |instance| Lease::Leased {
            clock_epoch: &instance.started.epoch,
            expires_monotonic_ms: instance.lease_expires_monotonic_ms,
        }),
        verification: match verification {
            Some(row) => Verification::Recorded {
                verdict: Verdict::parse(&row.verdict).ok_or_else(|| facts_error(id, "verdict"))?,
                cleanup_settled: row.cleanup_settled,
            },
            None => Verification::None,
        },
        evidence,
    })
}

fn identity_of(inventory: &RecoveryInventory, attempt: &DurableAttempt) -> Option<ProcessIdentity> {
    inventory
        .pins
        .iter()
        .filter(|pin| pin.attempt_id == attempt.id)
        .find_map(|pin| {
            pin.record
                .observation
                .as_ref()
                .and_then(|o| o.input.actual_identity.as_deref())
                .and_then(ProcessIdentity::parse)
        })
}

/// What the seam answered for one attempt, before the policy sees it.
struct Observed {
    identity: Option<ProcessIdentity>,
    process: ProcessCustody,
    pi_queue: PiQueueCustody,
    cleanup: CleanupReadback,
    workspace: WorkspaceReadback,
    acknowledgement: Acknowledgement,
    claim: Option<Claim>,
}

fn observe(
    inventory: &RecoveryInventory,
    attempt: &DurableAttempt,
    subject: &Subject<'_>,
    startup: &Startup<'_>,
    physical: &mut dyn Physical,
) -> Observed {
    let identity = identity_of(inventory, attempt);
    let process = identity
        .as_ref()
        .map_or(ProcessCustody::Unobserved, |identity| {
            classify(identity, &physical.process(subject, identity))
        });
    Observed {
        process,
        pi_queue: physical.pi_queue(subject),
        cleanup: physical.cleanup(subject),
        workspace: physical.workspace(subject),
        acknowledgement: physical.acknowledgement(subject),
        claim: startup
            .claims
            .iter()
            .find(|claim| claim.attempt == attempt.id)
            .cloned(),
        identity,
    }
}

/// The ledger and world an action reaches.
struct Effector<'s, 'p> {
    physical: &'p mut dyn Physical,
    store: &'s mut Store,
    attempt: UuidV4<'s>,
    deadline: Instant,
}

fn reconcile_attempt(
    ledger: &LedgerFacts<'_>,
    read: &Inspected,
    attempt: &DurableAttempt,
    startup: &Startup<'_>,
    physical: &mut dyn Physical,
    clock: Option<&ReceiptTime>,
    store: Option<&mut Store>,
) -> Result<Entry, Error> {
    let inventory = &read.inventory;
    let task = inventory
        .tasks
        .iter()
        .find(|task| task.head.id == attempt.task)
        .ok_or_else(|| facts_error(&attempt.id, "task row"))?;
    let verification = inventory
        .verifications
        .iter()
        .rfind(|row| row.attempt == attempt.id);
    let instance = inventory
        .instances
        .iter()
        .find(|instance| instance.attempt_id == attempt.id);
    let generation = generation_of(&attempt.generation, &attempt.id, "attempt generation")?;
    let subject = Subject {
        task: &attempt.task,
        attempt: &attempt.id,
        generation,
        workspace_ref: instance.map(|i| i.workspace_ref.as_str()),
        session: instance.map(|i| i.session_id.as_str()),
    };
    let seen = observe(inventory, attempt, &subject, startup, physical);
    let evidence = verification.map_or(Evidence::Unassessed, |row| {
        match read.evidence.get(&row.evidence) {
            Some(EvidenceAvailability::Published) => Evidence::Published,
            Some(EvidenceAvailability::Absent) | None => Evidence::Absent,
        }
    });
    let task_facts = task_facts(
        task,
        verification,
        read.ordinals.get(&task.head.id),
        &attempt.id,
    )?;
    let attempt_facts = attempt_facts(
        attempt,
        verification,
        instance,
        evidence,
        seen.acknowledgement,
    )?;
    let observed = Observations {
        claim: seen.claim.as_ref().map(|claim| ObservationClaim {
            epoch: &claim.epoch,
            task_generation: claim.task_generation,
            attempt_generation: claim.attempt_generation,
        }),
        clock: clock.map(|reading| Clock {
            epoch: &reading.epoch,
            monotonic_ms: reading.monotonic_ms,
        }),
        process: seen.process.clone(),
        pi_queue: seen.pi_queue.clone(),
        cleanup: seen.cleanup.clone(),
        workspace: seen.workspace,
    };
    let decision = policy::reconcile(ledger, &task_facts, &attempt_facts, &observed);
    let handed = Handed {
        subject: subject.to_value(),
        identity: seen.identity.clone(),
        process: seen.process,
        pi_queue: seen.pi_queue,
        cleanup: seen.cleanup,
        workspace: seen.workspace,
        acknowledgement: seen.acknowledgement,
        clock_epoch: clock.map(|reading| reading.epoch.clone()),
        claim: seen.claim,
        evidence,
        history: history_value(&task_facts.history),
        task_state: task_facts.state,
        attempt_state: attempt_facts.state,
    };
    let (action, records) = reach_ledger(
        store,
        physical,
        &Decided {
            decision: &decision,
            handed: &handed,
            subject: &subject,
            identity: seen.identity.as_ref(),
            task,
        },
        startup.deadline,
    )?;
    Ok(Entry {
        task: attempt.task.clone(),
        attempt: attempt.id.clone(),
        handed,
        rule: decision.rule.id(),
        decision,
        action,
        records,
    })
}

/// Act on one decision through a normally opened ledger, or, with no ledger to
/// reach, do nothing and record nothing. The `Effector` is built here so that
/// `reconcile_attempt` never holds a writable store alongside the read it made.
/// One attempt's decision together with the facts an effect needs, as one
/// value. The effect path takes the bundle rather than seven separate
/// arguments, so a new fact cannot be added to the effect without being added
/// to what the ledger records.
struct Decided<'a> {
    decision: &'a Decision,
    handed: &'a Handed,
    subject: &'a Subject<'a>,
    identity: Option<&'a ProcessIdentity>,
    task: &'a DurableTask,
}

fn reach_ledger(
    store: Option<&mut Store>,
    physical: &mut dyn Physical,
    decided: &Decided<'_>,
    deadline: Instant,
) -> Result<(Option<Action>, Vec<Recorded>), Error> {
    let mut records = Vec::new();
    let Some(store) = store else {
        return Ok((None, records));
    };
    let attempt = decided.subject.attempt;
    let attempt_id = UuidV4::parse(attempt).map_err(|_| facts_error(attempt, "attempt id"))?;
    let mut effector = Effector {
        physical,
        store,
        attempt: attempt_id,
        deadline,
    };
    let action = record_and_act(&mut effector, decided, &mut records)?;
    Ok((Some(action), records))
}

/// Record the decision before any effect, then act on it.
fn record_and_act(
    effector: &mut Effector<'_, '_>,
    decided: &Decided<'_>,
    records: &mut Vec<Recorded>,
) -> Result<Action, Error> {
    let Decided {
        decision,
        handed,
        subject,
        identity,
        task,
    } = decided;
    let body = serde_json::to_vec(&json!({
        "kind": "hee3-reconciliation-decided/1",
        "attempt": subject.attempt, "task": subject.task,
        "rule": decision.rule.id(), "decision": decision.reconciliation,
        "handed": handed, "intent": intended(&decision.reconciliation),
    }))?;
    records.push(effector.store.record_reconciliation(
        &ReconciliationRecord {
            attempt: effector.attempt,
            kind: RecordKind::Decided,
            body: &body,
            settle_cleanup: false,
        },
        effector.deadline,
    )?);
    act(
        &decision.reconciliation,
        subject,
        *identity,
        task,
        effector,
        records,
    )
}

/// The action a decision names, before it is performed.
#[must_use]
pub fn intended(reconciliation: &Reconciliation) -> &'static str {
    match reconciliation {
        Reconciliation::StaleObservationRefused { .. }
        | Reconciliation::StaleGenerationRefused { .. } => "stale_refused",
        Reconciliation::RetainUnknown { .. } => "retained_unknown",
        Reconciliation::ReattachObservationOnly { .. } => "observation_attached",
        Reconciliation::CleanupCandidate { .. } => "cleanup_performed",
        Reconciliation::WorkspaceReleasable { .. } => "releasable_recorded",
        Reconciliation::WorkspaceReuseRefused { .. } => "reuse_refused",
        Reconciliation::VerificationOutstanding { .. } => "verification_outstanding",
        Reconciliation::AcceptanceStands { .. } => "acceptance_stands",
        Reconciliation::CancellationStands { .. } => "cancellation_stands",
        Reconciliation::RefuseStaleCursor { .. } | Reconciliation::CursorSnapshotOnly { .. } => {
            "not_an_attempt_decision"
        }
    }
}

fn readback_record(
    effector: &mut Effector<'_, '_>,
    body: &Value,
    settle_cleanup: bool,
) -> Result<Recorded, Error> {
    let bytes = serde_json::to_vec(body)?;
    Ok(effector.store.record_reconciliation(
        &ReconciliationRecord {
            attempt: effector.attempt,
            kind: RecordKind::Readback,
            body: &bytes,
            settle_cleanup,
        },
        effector.deadline,
    )?)
}

fn attach(
    generation: u64,
    redispatch: bool,
    subject: &Subject<'_>,
    identity: Option<&ProcessIdentity>,
    effector: &mut Effector<'_, '_>,
    records: &mut Vec<Recorded>,
) -> Result<Action, Error> {
    // The arm carries `redispatch: false`; the pass reads it rather than assuming
    // it, and a `true` would be refused here, not obeyed.
    if redispatch {
        return Ok(Action::AttachRefused {
            generation,
            error: "policy permitted redispatch; refused".into(),
        });
    }
    let Some(identity) = identity else {
        return Ok(Action::AttachRefused {
            generation,
            error: "no process identity to attach to".into(),
        });
    };
    match effector.physical.attach(subject, identity) {
        Ok(()) => {
            let readback = classify(identity, &effector.physical.process(subject, identity));
            records.push(readback_record(
                effector,
                &json!({
                    "kind": "hee3-reconciliation-readback/1", "attempt": subject.attempt,
                    "effect": "observation_attached", "generation": generation,
                    "readback": readback,
                }),
                false,
            )?);
            Ok(Action::ObservationAttached {
                generation,
                readback,
            })
        }
        Err(error) => Ok(Action::AttachRefused { generation, error }),
    }
}

fn clean(
    what: &[CleanupTarget],
    subject: &Subject<'_>,
    effector: &mut Effector<'_, '_>,
    records: &mut Vec<Recorded>,
) -> Result<Action, Error> {
    let mut effects = Vec::new();
    let mut ledger_target = false;
    for target in what {
        match target {
            CleanupTarget::Remaining { name } => {
                let outcome = effector.physical.clean(subject, name);
                effects.push(CleanupEffect {
                    target: name.clone(),
                    performed: outcome.is_ok(),
                    error: outcome.err(),
                });
            }
            CleanupTarget::LedgerSettlement { .. } => ledger_target = true,
        }
    }
    let readback = effector.physical.cleanup(subject);
    let settle = ledger_target && readback == CleanupReadback::Complete;
    let mut body = json!({
        "kind": "hee3-reconciliation-readback/1", "attempt": subject.attempt,
        "effect": "cleanup", "targets": what, "effects": effects,
        "readback": readback, "settle_cleanup": settle,
    });
    let (ledger_settled, ledger_refusal) = match readback_record(effector, &body, settle) {
        Ok(recorded) => {
            let settled = recorded.cleanup_settled;
            records.push(recorded);
            (settled, None)
        }
        Err(Error::Store(store::Error::Conflict)) if settle => {
            // The row's state no longer admits settlement: record the readback
            // without it and say so.
            body["settle_cleanup"] = json!(false);
            body["settle_refused"] = json!("conflict");
            records.push(readback_record(effector, &body, false)?);
            (false, Some("conflict".to_owned()))
        }
        Err(error) => return Err(error),
    };
    Ok(Action::CleanupPerformed {
        targets: what.to_vec(),
        effects,
        readback,
        ledger_settled,
        ledger_refusal,
    })
}

fn act(
    reconciliation: &Reconciliation,
    subject: &Subject<'_>,
    identity: Option<&ProcessIdentity>,
    task: &DurableTask,
    effector: &mut Effector<'_, '_>,
    records: &mut Vec<Recorded>,
) -> Result<Action, Error> {
    Ok(match reconciliation {
        Reconciliation::StaleObservationRefused { .. }
        | Reconciliation::StaleGenerationRefused { .. } => Action::StaleRefused,
        Reconciliation::RetainUnknown { .. } => Action::RetainedUnknown,
        Reconciliation::ReattachObservationOnly {
            generation,
            redispatch,
            ..
        } => attach(
            *generation,
            *redispatch,
            subject,
            identity,
            effector,
            records,
        )?,
        Reconciliation::CleanupCandidate { what, .. } => clean(what, subject, effector, records)?,
        Reconciliation::WorkspaceReleasable { task_state, .. } => Action::ReleasableRecorded {
            task_state: *task_state,
        },
        Reconciliation::WorkspaceReuseRefused { reason, .. } => Action::ReuseRefused {
            reason: reason.clone(),
        },
        Reconciliation::VerificationOutstanding { task_state, .. } => {
            Action::VerificationOutstanding {
                task_state: *task_state,
            }
        }
        Reconciliation::AcceptanceStands { event, .. } => {
            let head_event = task.head.accepted_event.clone();
            Action::AcceptanceStands {
                event: event.clone(),
                agrees: head_event.as_deref() == Some(event.as_str()),
                head_event,
            }
        }
        Reconciliation::CancellationStands {
            rejected_acceptance,
            ..
        } => Action::CancellationStands {
            rejected_acceptance: rejected_acceptance.clone(),
            head_cancellation: task.head.cancellation,
            agrees: task.head.cancellation,
        },
        Reconciliation::RefuseStaleCursor { .. } | Reconciliation::CursorSnapshotOnly { .. } => {
            Action::NotAnAttemptDecision
        }
    })
}
