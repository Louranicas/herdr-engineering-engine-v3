//! Bounded Linux child and pipe custody. This leaf does not authenticate evidence.
//!
//! The caller must exclusively own child waits (including SIGCHLD policy). Stable
//! Rust cannot atomically acquire a pidfd during spawn. We verify a post-spawn
//! pidfd with waitid before retaining it, and revoke numeric group authority on
//! ECHILD. Process-group census is observational, not namespace containment.

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::fs::{Mode, OFlags, fcntl_getfl, fcntl_setfl, open};
use rustix::process::{
    Pid, PidfdFlags, Signal, WaitId, WaitIdOptions, kill_process_group, pidfd_open,
    pidfd_send_signal, waitid,
};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{File, ReadDir};
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::PathBuf;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const MAX_STREAM: usize = 8 * 1024 * 1024;
const MAX_INPUT: usize = 1024 * 1024;
const CLEANUP: Duration = Duration::from_secs(10);
const TERM_GRACE: Duration = Duration::from_secs(5);
const WAIT_FLAGS: WaitIdOptions = WaitIdOptions::EXITED
    .union(WaitIdOptions::NOHANG)
    .union(WaitIdOptions::NOWAIT);

/// Literal trusted-owner configuration, never decoded from candidate output.
#[derive(Debug)]
pub struct ProcessSpec {
    pub executable: PathBuf,
    pub arguments: Vec<OsString>,
    pub directory: PathBuf,
    pub environment: Vec<(OsString, OsString)>,
    pub input: Vec<u8>,
    pub stream_limit: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    Invalid,
    Bound,
    Deadline,
    Cancelled,
    Spawn,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interruption {
    Cancelled,
    Timeout,
    OutputLimit,
    PipeError,
    WaitError,
    ResidualGroup,
    ObserverError,
}

/// Raw observations; EOF and truncation are separate facts.
#[derive(Debug, Default)]
pub struct Stream {
    pub bytes: Vec<u8>,
    pub observed_bytes: u64,
    pub eof: bool,
    pub truncated: bool,
    pub failed: bool,
}
impl Stream {
    /// One bounded turn. The caller must configure the reader as nonblocking.
    pub(crate) fn drain(&mut self, reader: &mut impl Read, cap: usize) {
        if self.eof || self.failed {
            return;
        }
        for _ in 0..16 {
            let mut buffer = [0_u8; 8192];
            match reader.read(&mut buffer) {
                Ok(0) => {
                    self.eof = true;
                    return;
                }
                Ok(size) => {
                    self.observed_bytes = self.observed_bytes.saturating_add(size as u64);
                    let retained = size.min(cap.saturating_sub(self.bytes.len()));
                    self.bytes.extend_from_slice(&buffer[..retained]);
                    self.truncated |= retained != size;
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(_) => {
                    self.failed = true;
                    return;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SignalOutcome {
    #[default]
    NotAttempted,
    Sent,
    Failed(i32),
    NotOwned,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SignalFacts {
    pub group_term: SignalOutcome,
    pub group_kill: SignalOutcome,
    pub direct_term: SignalOutcome,
    pub direct_kill: SignalOutcome,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaitOwnership {
    Waitable,
    Lost,
    Reaped,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupState {
    Unknown,
    Live,
    Empty,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CleanupPoll {
    pub ownership: WaitOwnership,
    pub leader_terminal: bool,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub group: GroupState,
}

#[derive(Debug)]
pub struct ProcessReport {
    /// Caller-comparable monotonic origin sampled before spawn.
    pub started_at: Instant,
    pub leader_pid: u32,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub interruption: Option<Interruption>,
    /// First trusted observation of the retained interruption. This is also the
    /// origin of the existing stop grace; it is never inferred from the cutoff.
    pub interruption_observed_at: Option<Instant>,
    pub stdout: Stream,
    pub stderr: Stream,
    pub elapsed: Duration,
    pub signals: SignalFacts,
    pub leader_reaped: bool,
    /// Readback of the original group while the leader remains waitable.
    pub process_group_settled: bool,
    pub observer_ready: bool,
    /// Retain and poll incomplete direct-child ownership; Drop is only best effort.
    pub pending: Option<PendingChild>,
}

/// Trusted caller code; each invocation must be bounded and nonblocking.
pub trait Observer {
    fn observe(&mut self, observation: &ProcessObservation<'_>) -> ObserverDecision;
}
#[derive(Debug)]
pub struct ProcessObservation<'a> {
    pub leader_pid: u32,
    pub leader_terminal: bool,
    pub stdout: &'a Stream,
    pub stderr: &'a Stream,
    pub now: Instant,
    pub work_deadline: Instant,
    pub stopping_since: Option<Instant>,
    pub cleanup_deadline: Option<Instant>,
    pub interruption: Option<Interruption>,
}
#[derive(Clone, Copy, Debug)]
pub struct ObserverDecision {
    pub hold_stdin: bool,
    pub ready_to_finish: bool,
    pub stop: Option<Interruption>,
}
struct OrdinaryObserver;
impl Observer for OrdinaryObserver {
    fn observe(&mut self, _: &ProcessObservation<'_>) -> ObserverDecision {
        ObserverDecision {
            hold_stdin: false,
            ready_to_finish: true,
            stop: None,
        }
    }
}

/// Unsettled ownership, including any still-held stdin startup gate. Dropping this
/// value cannot prove cleanup. Keep it and call `poll_cleanup` with caller deadlines.
#[derive(Debug)]
pub struct PendingChild {
    child: Child,
    pid: Pid,
    pidfd: Option<OwnedFd>,
    ownership: WaitOwnership,
    terminal: bool,
    exit_code: Option<i32>,
    signal: Option<i32>,
    group: GroupState,
    census: Option<Census>,
    signals: SignalFacts,
}
impl PendingChild {
    fn new(child: Child) -> (Self, bool) {
        let pid = Pid::from_child(&child);
        let mut owner = Self {
            child,
            pid,
            pidfd: None,
            ownership: WaitOwnership::Waitable,
            terminal: false,
            exit_code: None,
            signal: None,
            group: GroupState::Unknown,
            census: None,
            signals: SignalFacts::default(),
        };
        let acquired = match pidfd_open(pid, PidfdFlags::NONBLOCK) {
            Ok(fd) => match waitid(WaitId::PidFd(fd.as_fd()), WAIT_FLAGS) {
                Ok(status) => {
                    owner.terminal = status.is_some();
                    owner.pidfd = Some(fd);
                    true
                }
                Err(rustix::io::Errno::CHILD) => {
                    owner.lose_wait();
                    false
                }
                Err(_) => false,
            },
            Err(_) => false,
        };
        // An unverified descriptor is never retained or used as a signal target.
        (owner, acquired)
    }

    #[must_use]
    pub fn id(&self) -> u32 {
        self.child.id()
    }
    #[must_use]
    pub fn signals(&self) -> &SignalFacts {
        &self.signals
    }

    /// One bounded reconciliation turn; no sleeping or blocking wait. A past
    /// deadline performs no progress. This does not reconcile an external observer.
    pub fn poll_cleanup(&mut self, deadline: Instant) -> CleanupPoll {
        if Instant::now() < deadline {
            let _ = self.observe_wait();
            if Instant::now() < deadline {
                self.send(Signal::KILL);
            }
            if self.terminal
                && self.ownership == WaitOwnership::Waitable
                && Instant::now() < deadline
            {
                let _ = self.scan(deadline, None);
            }
            if self.group == GroupState::Empty && Instant::now() < deadline {
                let _ = self.reap();
            }
        }
        CleanupPoll {
            ownership: self.ownership,
            leader_terminal: self.terminal,
            exit_code: self.exit_code,
            signal: self.signal,
            group: self.group,
        }
    }

    fn lose_wait(&mut self) {
        self.ownership = WaitOwnership::Lost;
        self.group = GroupState::Unknown;
        self.census = None;
    }

    fn observe_wait(&mut self) -> Result<(), rustix::io::Errno> {
        if self.ownership == WaitOwnership::Reaped {
            return Ok(());
        }
        if self.ownership == WaitOwnership::Lost {
            if let Some(fd) = &self.pidfd {
                let mut fds = [PollFd::new(fd, PollFlags::IN)];
                poll(
                    &mut fds,
                    Some(&Timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    }),
                )?;
                if fds[0].revents().contains(PollFlags::IN) {
                    self.terminal = true;
                }
            }
            return Err(rustix::io::Errno::CHILD);
        }
        let target = self
            .pidfd
            .as_ref()
            .map_or(WaitId::Pid(self.pid), |fd| WaitId::PidFd(fd.as_fd()));
        match waitid(target, WAIT_FLAGS) {
            Ok(status) => {
                self.terminal |= status.is_some();
                Ok(())
            }
            Err(error) => {
                if error == rustix::io::Errno::CHILD {
                    self.lose_wait();
                }
                Err(error)
            }
        }
    }

    fn send(&mut self, signal: Signal) {
        let previous = if signal == Signal::TERM {
            self.signals.group_term
        } else {
            self.signals.group_kill
        };
        if previous == SignalOutcome::NotAttempted {
            // Revalidate immediately before every numeric group signal, including
            // Drop. ECHILD removes authority; a retained pidfd never restores it.
            let outcome = match self.observe_wait() {
                Ok(()) if self.ownership == WaitOwnership::Waitable => {
                    signal_outcome(kill_process_group(self.pid, signal))
                }
                Err(rustix::io::Errno::CHILD) | Ok(()) => SignalOutcome::NotOwned,
                Err(error) => SignalOutcome::Failed(error.raw_os_error()),
            };
            if signal == Signal::TERM {
                self.signals.group_term = outcome;
            } else {
                self.signals.group_kill = outcome;
            }
        }
        let previous = if signal == Signal::TERM {
            self.signals.direct_term
        } else {
            self.signals.direct_kill
        };
        if previous == SignalOutcome::NotAttempted {
            let outcome = self.pidfd.as_ref().map_or(SignalOutcome::NotOwned, |fd| {
                signal_outcome(pidfd_send_signal(fd, signal))
            });
            if signal == Signal::TERM {
                self.signals.direct_term = outcome;
            } else {
                self.signals.direct_kill = outcome;
            }
        }
    }

    fn scan(&mut self, deadline: Instant, cancelled: Option<&AtomicBool>) -> Result<(), ScanError> {
        if self.group == GroupState::Empty {
            return Ok(());
        }
        if self.census.is_none() {
            check_scan_budget(deadline, cancelled)?;
            self.census = Some(Census {
                entries: std::fs::read_dir("/proc").map_err(|_| ScanError::Io)?,
                count: 0,
            });
        }
        let result = self
            .census
            .as_mut()
            .ok_or(ScanError::Io)?
            .step(self.pid, deadline, cancelled);
        match result {
            Ok(GroupState::Unknown) => {
                self.group = GroupState::Unknown;
                Ok(())
            }
            Ok(state) => {
                self.group = state;
                self.census = None;
                Ok(())
            }
            Err(error) => {
                self.group = GroupState::Unknown;
                self.census = None;
                Err(error)
            }
        }
    }

    fn reap(&mut self) -> Result<(), ()> {
        if !self.terminal || self.ownership != WaitOwnership::Waitable {
            return Ok(());
        }
        match self.child.try_wait() {
            Ok(Some(status)) => {
                self.exit_code = status.code();
                self.signal = status.signal();
                self.ownership = WaitOwnership::Reaped;
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(error) => {
                if error.raw_os_error() == Some(rustix::io::Errno::CHILD.raw_os_error()) {
                    self.lose_wait();
                }
                Err(())
            }
        }
    }
}
impl Drop for PendingChild {
    fn drop(&mut self) {
        if self.ownership != WaitOwnership::Reaped {
            self.send(Signal::KILL);
            let _ = self.observe_wait();
            let _ = self.reap();
        }
    }
}
fn signal_outcome(result: Result<(), rustix::io::Errno>) -> SignalOutcome {
    match result {
        Ok(()) => SignalOutcome::Sent,
        Err(error) => SignalOutcome::Failed(error.raw_os_error()),
    }
}

impl ProcessSpec {
    fn validate(&self) -> Result<(), Refusal> {
        if !self.executable.is_absolute()
            || !self.directory.is_absolute()
            || self.executable.as_os_str().as_bytes().contains(&0)
            || self.directory.as_os_str().as_bytes().contains(&0)
        {
            return Err(Refusal::Invalid);
        }
        if self.arguments.len() > 256
            || self.environment.len() > 32
            || self.input.len() > MAX_INPUT
            || !(1..=MAX_STREAM).contains(&self.stream_limit)
        {
            return Err(Refusal::Bound);
        }
        let mut bytes = 0_usize;
        for arg in &self.arguments {
            let value = arg.as_bytes();
            if value.len() > 4096 || value.contains(&0) {
                return Err(Refusal::Bound);
            }
            bytes += value.len();
        }
        if bytes > 65_536 {
            return Err(Refusal::Bound);
        }
        let mut names = BTreeSet::new();
        for (key, value) in &self.environment {
            let key = key.as_bytes();
            if key.is_empty()
                || key.len() > 64
                || !key.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_')
                || !names.insert(key)
            {
                return Err(Refusal::Invalid);
            }
            if value.as_bytes().len() > 4096 || value.as_bytes().contains(&0) {
                return Err(Refusal::Bound);
            }
        }
        Ok(())
    }
}

fn nonblocking(fd: &impl AsFd) -> io::Result<()> {
    fcntl_setfl(fd, fcntl_getfl(fd)? | OFlags::NONBLOCK)?;
    Ok(())
}

/// Launch with cleared environment, explicit cwd and owned nonblocking pipes.
/// Deadline excludes the caller's cleanup reserve. A terminal exit is a process
/// observation, never an acceptance result. Child waits require an exclusive owner.
///
/// # Errors
/// Refuses invalid inputs, expired deadlines, prior cancellation or spawn failure.
pub fn run(
    spec: &ProcessSpec,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<ProcessReport, Refusal> {
    run_observed(spec, deadline, cancelled, &mut OrdinaryObserver)
}

/// Launch with a trusted bounded observer. Stdin starts held, including EOF, and
/// releases once only if the observer permits it before stopping. The observer is
/// polled after leader termination until readiness or the original cleanup cutoff.
///
/// # Errors
/// Refuses invalid inputs, expired deadlines, prior cancellation or spawn failure.
pub fn run_observed(
    spec: &ProcessSpec,
    deadline: Instant,
    cancelled: &AtomicBool,
    observer: &mut impl Observer,
) -> Result<ProcessReport, Refusal> {
    spec.validate()?;
    let started = Instant::now();
    if deadline <= started || deadline.duration_since(started) > Duration::from_mins(20) {
        return Err(Refusal::Deadline);
    }
    if cancelled.load(Ordering::Acquire) {
        return Err(Refusal::Cancelled);
    }
    let child = Command::new(&spec.executable)
        .args(&spec.arguments)
        .current_dir(&spec.directory)
        .env_clear()
        .envs(spec.environment.iter().cloned())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|_| Refusal::Spawn)?;
    let (mut owner, acquired) = PendingChild::new(child);
    let mut report = ProcessReport {
        started_at: started,
        leader_pid: owner.id(),
        exit_code: None,
        signal: None,
        interruption: if acquired {
            None
        } else {
            Some(Interruption::WaitError)
        },
        interruption_observed_at: None,
        stdout: Stream::default(),
        stderr: Stream::default(),
        elapsed: Duration::ZERO,
        signals: SignalFacts::default(),
        leader_reaped: false,
        process_group_settled: false,
        observer_ready: false,
        pending: None,
    };
    let mut pipes = Pipes::new(&mut owner.child, &mut report);
    drive(
        spec,
        deadline,
        cancelled,
        observer,
        &mut owner,
        &mut report,
        &mut pipes,
    );
    if owner.group == GroupState::Empty && owner.reap().is_err() {
        report.interruption.get_or_insert(Interruption::WaitError);
    }
    observe_limits(&mut report, deadline, cancelled);
    report.elapsed = started.elapsed();
    report.exit_code = owner.exit_code;
    report.signal = owner.signal;
    report.leader_reaped = owner.ownership == WaitOwnership::Reaped;
    report.process_group_settled = owner.group == GroupState::Empty;
    report.signals = owner.signals;
    if !report.leader_reaped {
        owner.child.stdin = pipes.input;
        owner.child.stdout = pipes.output;
        owner.child.stderr = pipes.error;
        report.pending = Some(owner);
    }
    Ok(report)
}

fn drive(
    spec: &ProcessSpec,
    deadline: Instant,
    cancelled: &AtomicBool,
    observer: &mut impl Observer,
    owner: &mut PendingChild,
    report: &mut ProcessReport,
    pipes: &mut Pipes,
) {
    let mut stopping = None;
    loop {
        observe_limits(report, deadline, cancelled);
        start_stopping(report, &mut stopping);
        pipes.read(report, spec.stream_limit);
        if let Err(e) = owner.observe_wait()
            && e != rustix::io::Errno::INTR
        {
            report.interruption.get_or_insert(Interruption::WaitError);
        }
        observe_limits(report, deadline, cancelled);
        start_stopping(report, &mut stopping);
        if owner.terminal && owner.ownership == WaitOwnership::Waitable {
            let cutoff = stopping.map_or(deadline, |stop| stop + CLEANUP);
            let cancel_scan = if stopping.is_none() {
                Some(cancelled)
            } else {
                None
            };
            match owner.scan(cutoff, cancel_scan) {
                Ok(()) if owner.group == GroupState::Live => {
                    report
                        .interruption
                        .get_or_insert(Interruption::ResidualGroup);
                }
                Ok(()) => {}
                Err(ScanError::Stopped(reason)) => {
                    report.interruption.get_or_insert(reason);
                }
                Err(ScanError::Io) => {
                    report.interruption.get_or_insert(Interruption::WaitError);
                }
            }
        }
        // A census turn and the observer can cross a cutoff. Never decide success
        // using a timestamp or cancellation observation from before that work.
        observe_limits(report, deadline, cancelled);
        start_stopping(report, &mut stopping);
        let decision = observer.observe(&ProcessObservation {
            leader_pid: owner.id(),
            leader_terminal: owner.terminal,
            stdout: &report.stdout,
            stderr: &report.stderr,
            now: Instant::now(),
            work_deadline: deadline,
            stopping_since: stopping,
            cleanup_deadline: stopping.map(|stop| stop + CLEANUP),
            interruption: report.interruption,
        });
        report.observer_ready = decision.ready_to_finish;
        if let Some(reason) = decision.stop {
            report.interruption.get_or_insert(reason);
        }
        observe_limits(report, deadline, cancelled);
        start_stopping(report, &mut stopping);
        pipes.write(spec, report, !decision.hold_stdin, stopping.is_some());
        observe_limits(report, deadline, cancelled);
        start_stopping(report, &mut stopping);
        if let Some(stop) = stopping {
            owner.send(Signal::TERM);
            if Instant::now().duration_since(stop) >= TERM_GRACE {
                owner.send(Signal::KILL);
            }
            if Instant::now().duration_since(stop) >= CLEANUP {
                break;
            }
        }
        if owner.terminal
            && owner.group == GroupState::Empty
            && report.observer_ready
            && (report.stdout.eof || report.stdout.failed)
            && (report.stderr.eof || report.stderr.failed)
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

struct Pipes {
    input: Option<ChildStdin>,
    output: Option<ChildStdout>,
    error: Option<ChildStderr>,
    written: usize,
    gate_released: bool,
}
impl Pipes {
    fn new(child: &mut Child, report: &mut ProcessReport) -> Self {
        let input = child.stdin.take();
        let mut output = child.stdout.take();
        let mut error = child.stderr.take();
        let configured = input.as_ref().is_some_and(|fd| nonblocking(fd).is_ok())
            && output.as_ref().is_some_and(|fd| nonblocking(fd).is_ok())
            && error.as_ref().is_some_and(|fd| nonblocking(fd).is_ok());
        if !configured {
            // Preserve stdin so a setup error cannot release a held gate.
            output = None;
            error = None;
            report.stdout.failed = true;
            report.stderr.failed = true;
            report.interruption.get_or_insert(Interruption::PipeError);
        }
        Self {
            input,
            output,
            error,
            written: 0,
            gate_released: false,
        }
    }
    fn read(&mut self, report: &mut ProcessReport, cap: usize) {
        if let Some(pipe) = &mut self.output {
            report.stdout.drain(pipe, cap);
        }
        if let Some(pipe) = &mut self.error {
            report.stderr.drain(pipe, cap);
        }
    }
    fn write(
        &mut self,
        spec: &ProcessSpec,
        report: &mut ProcessReport,
        release: bool,
        stopping: bool,
    ) {
        if release && !stopping {
            self.gate_released = true;
        }
        if !self.gate_released {
            return;
        }
        if self.written == spec.input.len() || stopping {
            self.input = None;
        }
        if let Some(pipe) = &mut self.input {
            let end = (self.written + 8192).min(spec.input.len());
            match pipe.write(&spec.input[self.written..end]) {
                Ok(0) => {
                    report.interruption.get_or_insert(Interruption::PipeError);
                }
                Ok(size) => {
                    self.written += size;
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                    ) => {}
                Err(_) => {
                    report.interruption.get_or_insert(Interruption::PipeError);
                }
            }
        }
    }
}

fn observe_limits(report: &mut ProcessReport, deadline: Instant, cancelled: &AtomicBool) {
    if cancelled.load(Ordering::Acquire) {
        report.interruption.get_or_insert(Interruption::Cancelled);
    }
    if Instant::now() >= deadline {
        report.interruption.get_or_insert(Interruption::Timeout);
    }
    if report.stdout.truncated || report.stderr.truncated {
        report.interruption.get_or_insert(Interruption::OutputLimit);
    }
    if report.stdout.failed || report.stderr.failed {
        report.interruption.get_or_insert(Interruption::PipeError);
    }
    if report.interruption.is_some() {
        report
            .interruption_observed_at
            .get_or_insert_with(Instant::now);
    }
}
fn start_stopping(report: &mut ProcessReport, stopping: &mut Option<Instant>) {
    if report.interruption.is_some() && stopping.is_none() {
        *stopping = Some(
            *report
                .interruption_observed_at
                .get_or_insert_with(Instant::now),
        );
    }
}

#[derive(Debug)]
struct Census {
    entries: ReadDir,
    count: usize,
}
#[derive(Clone, Copy, Debug)]
enum ScanError {
    Io,
    Stopped(Interruption),
}
fn check_scan_budget(deadline: Instant, cancelled: Option<&AtomicBool>) -> Result<(), ScanError> {
    if cancelled.is_some_and(|flag| flag.load(Ordering::Acquire)) {
        return Err(ScanError::Stopped(Interruption::Cancelled));
    }
    if Instant::now() >= deadline {
        return Err(ScanError::Stopped(Interruption::Timeout));
    }
    Ok(())
}
impl Census {
    fn step(
        &mut self,
        group: Pid,
        deadline: Instant,
        cancelled: Option<&AtomicBool>,
    ) -> Result<GroupState, ScanError> {
        // Native /proc only; at most 64 entries and 4097 stat bytes per entry per
        // turn, at most 65536 entries per complete observational census.
        for _ in 0..64 {
            check_scan_budget(deadline, cancelled)?;
            let Some(entry) = self.entries.next() else {
                check_scan_budget(deadline, cancelled)?;
                return Ok(GroupState::Empty);
            };
            self.count += 1;
            if self.count > 65_536 {
                return Err(ScanError::Io);
            }
            let entry = entry.map_err(|_| ScanError::Io)?;
            if entry
                .file_name()
                .to_str()
                .and_then(|s| s.parse::<u32>().ok())
                .is_none()
            {
                continue;
            }
            check_scan_budget(deadline, cancelled)?;
            let fd = match open(
                entry.path().join("stat"),
                OFlags::RDONLY | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            ) {
                Ok(fd) => fd,
                Err(error) if exited_during_census(Some(error.raw_os_error())) => continue,
                Err(_) => return Err(ScanError::Io),
            };
            check_scan_budget(deadline, cancelled)?;
            let mut bytes = [0_u8; 4097];
            let size = match File::from(fd).read(&mut bytes) {
                Ok(size) => size,
                Err(error) if exited_during_census(error.raw_os_error()) => continue,
                Err(_) => return Err(ScanError::Io),
            };
            check_scan_budget(deadline, cancelled)?;
            if size == 0 || size == bytes.len() {
                return Err(ScanError::Io);
            }
            if stat_is_live(&bytes[..size], group)? {
                return Ok(GroupState::Live);
            }
        }
        Ok(GroupState::Unknown)
    }
}
/// Whether a failed `/proc/<pid>/stat` open or read means only that the process exited
/// between the census listing it and reading it: `ENOENT` at the open, `ESRCH` at the open or
/// at a read on a descriptor opened while it was alive. Such a process is not a member of the
/// group -- it is gone -- and the census moves on.
///
/// Every other errno is an unobserved census. Before this, `ESRCH` on the read was reported
/// as I/O failure, so ANY process on the machine exiting in that window turned a clean
/// exchange into `Interruption::WaitError`: `t08_contract` failed about one run in four under
/// load, on whichever case's exchange lost the race. Pure, so it is reachable by argument
/// rather than only by arranging a process to die mid-census (F95).
#[must_use]
pub fn exited_during_census(raw_os_error: Option<i32>) -> bool {
    raw_os_error.is_some_and(|code| {
        code == rustix::io::Errno::NOENT.raw_os_error()
            || code == rustix::io::Errno::SRCH.raw_os_error()
    })
}

fn stat_is_live(bytes: &[u8], group: Pid) -> Result<bool, ScanError> {
    // Process comm may be arbitrary non-UTF8 bytes and contain ') '. Split at the
    // final delimiter, then parse only the kernel-authored ASCII suffix.
    let offset = bytes
        .windows(2)
        .rposition(|pair| pair == b") ")
        .ok_or(ScanError::Io)?;
    let suffix = std::str::from_utf8(&bytes[offset + 2..]).map_err(|_| ScanError::Io)?;
    let mut fields = suffix.split_ascii_whitespace();
    let state = fields.next().ok_or(ScanError::Io)?;
    let pgrp = fields
        .nth(1)
        .and_then(|s| s.parse::<i32>().ok())
        .ok_or(ScanError::Io)?;
    Ok(pgrp == group.as_raw_nonzero().get() && !matches!(state, "Z" | "X"))
}
