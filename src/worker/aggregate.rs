//! One check's transient aggregate slice ([`Slice`]), named after the ledger attempt its caller
//! holds, and a service's main process id ([`main_pid`]), both over the one pinned busctl door;
//! raw manager facts are not acceptance.
use super::{process, resources};
use crate::contracts::UuidV4;
use rustix::process::geteuid;
use serde::Deserialize;
use std::ffi::OsString;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
#[path = "aggregate_io.rs"]
mod io;

const MANAGER: &str = "org.freedesktop.systemd1.Manager";
const OBJECT: &str = "/org/freedesktop/systemd1";
const SERVICE: &str = "org.freedesktop.systemd1";
#[derive(Debug)]
pub struct Config {
    pub busctl: PathBuf,
    pub busctl_sha256: String,
    pub runtime_dir: PathBuf,
}
/// busctl's fixed host path: the one the door pins (`aggregate_io::pin`) and every caller names.
pub const BUSCTL: &str = "/usr/bin/busctl";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Invalid,
    Bound,
    Deadline,
    Cancelled,
    Identity,
    Io,
    State,
    Manager,
    Process,
    Limits,
    Busy,
}
/// What a create request came to (R22-2): a job path came back, nothing reached the manager (or it
/// answered with an error), or a failure after which the manager may still act on the request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CreateOutcome {
    Answered,
    Refused,
    Unknown(Error),
}
/// Where one check's [`Slice`] stands (R22-2): nothing sent, the create asked, the slice created
/// with its cgroup held, the stop asked, or stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlicePhase {
    Prepared,
    CreateRequested,
    Created,
    StopRequested,
    Stopped,
}
/// What [`Slice::stop`] does from a phase ([`stop_step`]): there is no wedged step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StopStep {
    Nothing,
    Resolve,
    StopIfEmpty,
    AwaitSettled,
}
/// What the manager's own row says of a slice whose create was asked ([`resolve`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Resolved {
    Gone,
    StopUnrealised,
    CheckEmpty,
}
#[derive(Debug)]
pub struct Call {
    pub argv: Vec<OsString>,
    pub report: Result<process::ProcessReport, process::Refusal>,
}
#[derive(Debug)]
pub struct UnitObservation {
    pub name: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub control_group: Option<String>,
    /// The id of the job the manager holds queued or running for the unit, `0` when none (the
    /// row's eighth field). A unit whose start job is still queued reads loaded and inactive with
    /// no control group, which without the job is the row of a stopped unit.
    pub job: u32,
    pub observed_at: Instant,
}
/// One check's aggregate slice (R22-1): named after the ledger attempt, created with the aggregate
/// limits and stopped — never holding a process of its own. It has no attach: the only manager
/// requests it can make are the slice's create (`create_arguments`) and `StopUnit`, so the
/// caller is never moved, and a caller that consumes it on finish cannot hold one across checks.
#[derive(Debug)]
pub struct Slice {
    config: Config,
    phase: SlicePhase,
    unit: String,
    cgroup: Option<File>,
    calls: Vec<Call>,
    created: CreateOutcome,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply<T> {
    #[serde(rename = "type")]
    signature: String,
    data: T,
}
type UnitRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    u32,
    String,
    String,
);

impl Slice {
    /// Pin the manager's door and name the slice after `attempt`, the ledger attempt the caller
    /// holds (R22 C1a), through `aggregate_unit`; no manager call, no mutation.
    /// # Errors
    /// The door's pin refusal.
    pub fn prepare(config: Config, attempt: UuidV4<'_>, deadline: Instant) -> Result<Self, Error> {
        io::pin(&config, deadline)?;
        Ok(Self::named(config, attempt))
    }
    /// The policy half of [`Slice::prepare`], split from the pin so a test can reach it (F95): a
    /// `Prepared` slice named after exactly the attempt it is handed, with no call recorded.
    fn named(config: Config, attempt: UuidV4<'_>) -> Self {
        Self {
            config,
            phase: SlicePhase::Prepared,
            unit: aggregate_unit(attempt),
            cgroup: None,
            calls: vec![],
            created: CreateOutcome::Refused,
        }
    }
    /// Create the slice, from `Prepared` only: its name proved absent, the create asked (its
    /// outcome recorded by `create_outcome` before any error returns), then its cgroup captured
    /// with the aggregate limits read back.
    /// # Errors
    /// `State` from any other phase; the door's, the manager's or the readback's refusal — the
    /// phase then says what [`Slice::stop`] must undo.
    pub fn create(&mut self, deadline: Instant, cancelled: &AtomicBool) -> Result<(), Error> {
        if self.phase != SlicePhase::Prepared {
            return Err(Error::State);
        }
        absent(
            &self.config,
            &mut self.calls,
            self.unit.clone(),
            deadline,
            cancelled,
        )?;
        self.phase = SlicePhase::CreateRequested;
        let recorded = self.calls.len();
        let result = create_slice(
            &self.config,
            &mut self.calls,
            &self.unit,
            deadline,
            cancelled,
        );
        self.created = create_outcome(self.calls.get(recorded), result);
        result?;
        self.cgroup = Some(capture(&slice_path(&self.unit), deadline, cancelled)?);
        self.phase = SlicePhase::Created;
        Ok(())
    }
    /// Undo whatever the phase says was done, step by step through `stop_step` and `resolve`,
    /// under `deadline`; never cancelled. Each step either returns or moves the phase forward, so
    /// it ends at `Stopped` or at a named refusal.
    /// # Errors
    /// The manager's, the door's or the readback's refusal; `Busy` for a slice that is not empty;
    /// a create whose outcome is unknown and whose unit is not found, or still holds a queued job,
    /// is its own error (R22-2 row 2; B14b-2 closure D4).
    pub fn stop(&mut self, deadline: Instant) -> Result<(), Error> {
        let uncancelled = AtomicBool::new(false);
        let path = slice_path(&self.unit);
        loop {
            match stop_step(self.phase) {
                StopStep::Nothing => return Ok(()),
                StopStep::Resolve => {
                    let row = query(
                        &self.config,
                        &mut self.calls,
                        &self.unit,
                        deadline,
                        &uncancelled,
                    )?;
                    match resolve(&row, &path, self.created)? {
                        Resolved::Gone => self.phase = SlicePhase::Stopped,
                        Resolved::StopUnrealised => {
                            self.phase = SlicePhase::StopRequested;
                            job(&method(
                                &self.config,
                                &mut self.calls,
                                "StopUnit",
                                vec!["ss".into(), self.unit.clone(), "fail".into()],
                                deadline,
                                &uncancelled,
                            )?)?;
                        }
                        Resolved::CheckEmpty => {
                            self.cgroup = Some(io::cgroup(&path, deadline)?);
                            self.phase = SlicePhase::Created;
                        }
                    }
                }
                StopStep::StopIfEmpty => {
                    let fd = self.cgroup.as_ref().ok_or(Error::State)?;
                    let phase = &mut self.phase;
                    stop_empty(
                        &self.config,
                        &mut self.calls,
                        fd,
                        &path,
                        &self.unit,
                        deadline,
                        |_| *phase = SlicePhase::StopRequested,
                    )?;
                }
                StopStep::AwaitSettled => {
                    await_settled(&self.config, &mut self.calls, &self.unit, deadline)?;
                    self.phase = SlicePhase::Stopped;
                }
            }
        }
    }
    #[must_use]
    pub fn unit(&self) -> &str {
        &self.unit
    }
    #[must_use]
    pub fn calls(&self) -> &[Call] {
        &self.calls
    }
}
/// The one busctl door (R21 N6): the pinned `/usr/bin/busctl` (`io::pin`, re-checked per call)
/// run with a cleared environment holding only `XDG_RUNTIME_DIR`, against the user manager, with
/// no interactive authorization, under the caller's deadline and cancellation. A refused pin
/// starts nothing and returns `Err`; otherwise the call is returned whole for its owner to record,
/// and [`Call::stdout`] decides what it printed.
fn busctl(
    config: &Config,
    parameters: Vec<String>,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<Call, Error> {
    io::pin(config, deadline)?;
    let mut argv = vec![
        OsString::from("--user"),
        "--json=short".into(),
        "--no-pager".into(),
        "--allow-interactive-authorization=no".into(),
    ];
    argv.extend(parameters.into_iter().map(Into::into));
    let spec = process::ProcessSpec {
        executable: config.busctl.clone(),
        arguments: argv.clone(),
        directory: "/".into(),
        environment: vec![(
            "XDG_RUNTIME_DIR".into(),
            config.runtime_dir.clone().into_os_string(),
        )],
        input: vec![],
        stream_limit: 65536,
    };
    let report = process::run(&spec, deadline, cancelled);
    Ok(Call { argv, report })
}
/// One recorded manager call (R22 C8, the one home of the 64-call cap): refused `Bound` before the
/// 65th, else run through the busctl door and recorded whole before its output is read.
fn call(
    config: &Config,
    calls: &mut Vec<Call>,
    parameters: Vec<String>,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, Error> {
    if calls.len() >= 64 {
        return Err(Error::Bound);
    }
    let call = busctl(config, parameters, deadline, cancelled)?;
    calls.push(call);
    calls.last().ok_or(Error::Process)?.stdout()
}
/// One method of the user manager, recorded.
fn method(
    config: &Config,
    calls: &mut Vec<Call>,
    name: &str,
    parameters: Vec<String>,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, Error> {
    let mut argv = vec![
        "call".to_owned(),
        SERVICE.into(),
        OBJECT.into(),
        MANAGER.into(),
        name.into(),
    ];
    argv.extend(parameters);
    call(config, calls, argv, deadline, cancelled)
}
/// The manager's row for one unit and, unless not found, its `ControlGroup`.
fn query(
    config: &Config,
    calls: &mut Vec<Call>,
    name: &str,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<UnitObservation, Error> {
    let bytes = method(
        config,
        calls,
        "ListUnitsByNames",
        vec!["as".into(), "1".into(), name.into()],
        deadline,
        cancelled,
    )?;
    let (mut observation, object) = unit_reply(&bytes, name)?;
    if observation.load_state != "not-found" {
        let class = if name
            .rsplit_once('.')
            .is_some_and(|(_, kind)| kind == "slice")
        {
            "org.freedesktop.systemd1.Slice"
        } else {
            "org.freedesktop.systemd1.Scope"
        };
        let bytes = call(
            config,
            calls,
            vec![
                "get-property".into(),
                SERVICE.into(),
                object,
                class.into(),
                "ControlGroup".into(),
            ],
            deadline,
            cancelled,
        )?;
        let reply: Reply<String> = serde_json::from_slice(&bytes).map_err(|_| Error::Manager)?;
        if reply.signature != "s" || reply.data.len() > 2048 {
            return Err(Error::Manager);
        }
        observation.control_group = Some(reply.data);
    }
    observation.observed_at = Instant::now();
    io::tick(deadline)?;
    Ok(observation)
}
/// The manager lists no unit of this exact name (else `Identity`).
fn absent(
    config: &Config,
    calls: &mut Vec<Call>,
    name: String,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<(), Error> {
    let bytes = method(
        config,
        calls,
        "ListUnitsByPatterns",
        vec!["asas".into(), "0".into(), "1".into(), name],
        deadline,
        cancelled,
    )?;
    let reply: Reply<(Vec<UnitRow>,)> =
        serde_json::from_slice(&bytes).map_err(|_| Error::Manager)?;
    if reply.signature != "a(ssssssouso)" {
        return Err(Error::Manager);
    }
    if reply.data.0.is_empty() {
        Ok(())
    } else {
        Err(Error::Identity)
    }
}
/// Ask the manager to create the slice with its limits; a job path is the only answer.
fn create_slice(
    config: &Config,
    calls: &mut Vec<Call>,
    unit: &str,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<(), Error> {
    job(&method(
        config,
        calls,
        "StartTransientUnit",
        create_arguments(unit),
        deadline,
        cancelled,
    )?)
}
/// The created slice's cgroup directory, once it exists, with its limits read back.
fn capture(path: &str, deadline: Instant, cancelled: &AtomicBool) -> Result<File, Error> {
    loop {
        io::tick(deadline)?;
        if cancelled.load(Ordering::Acquire) {
            return Err(Error::Cancelled);
        }
        match io::cgroup(path, deadline) {
            Ok(file) => {
                aggregate_limits(&file, deadline)?;
                io::stable(&file, path, deadline)?;
                return Ok(file);
            }
            Err(Error::Io) => {}
            Err(error) => return Err(error),
        }
        std::thread::sleep(
            Duration::from_millis(2).min(deadline.saturating_duration_since(Instant::now())),
        );
    }
}
/// Stop the slice only when its own cgroup reads empty (else `Busy`): the emptiness readback, then
/// `requested` with the instant it was seen empty (the caller records the stop as asked before it
/// is sent), then `StopUnit`. Never cancelled.
fn stop_empty(
    config: &Config,
    calls: &mut Vec<Call>,
    fd: &File,
    path: &str,
    unit: &str,
    deadline: Instant,
    requested: impl FnOnce(Instant),
) -> Result<(), Error> {
    io::stable(fd, path, deadline)?;
    if !empty(&io::text(fd, "cgroup.events", deadline)?)? {
        return Err(Error::Busy);
    }
    io::stable(fd, path, deadline)?;
    requested(Instant::now());
    job(&method(
        config,
        calls,
        "StopUnit",
        vec!["ss".into(), unit.to_owned(), "fail".into()],
        deadline,
        &AtomicBool::new(false),
    )?)
}
/// Poll the manager until it has let the unit go ([`settled`]), under the deadline; never cancelled.
fn await_settled(
    config: &Config,
    calls: &mut Vec<Call>,
    unit: &str,
    deadline: Instant,
) -> Result<UnitObservation, Error> {
    let uncancelled = AtomicBool::new(false);
    loop {
        io::tick(deadline)?;
        let observation = query(config, calls, unit, deadline, &uncancelled)?;
        if settled(&observation) {
            return Ok(observation);
        }
        std::thread::sleep(
            Duration::from_millis(2).min(deadline.saturating_duration_since(Instant::now())),
        );
    }
}

impl Call {
    /// What the call printed, if it is an observation: the leader reaped, its group settled, both
    /// streams complete and unfailed (else `Process`), and exit 0 with no signal and no stderr
    /// (else `Manager`).
    ///
    /// # Errors
    /// `Process` or `Manager`, as above.
    fn stdout(&self) -> Result<Vec<u8>, Error> {
        let report = self.report.as_ref().map_err(|_| Error::Process)?;
        if report.pending.is_some()
            || !report.leader_reaped
            || !report.process_group_settled
            || report.interruption.is_some()
            || !report.stdout.eof
            || !report.stderr.eof
            || report.stdout.truncated
            || report.stderr.truncated
            || report.stdout.failed
            || report.stderr.failed
        {
            return Err(Error::Process);
        }
        if report.exit_code != Some(0) || report.signal.is_some() || !report.stderr.bytes.is_empty()
        {
            return Err(Error::Manager);
        }
        Ok(report.stdout.bytes.clone())
    }
}

/// A service unit's main process id as the user manager reports it (R21 N6), over the one busctl
/// door: `ListUnitsByNames` finds the unit's object path (read at run time, never a host constant;
/// K2), the unit must be loaded and active (else `State`), and `MainPID` is read from that path's
/// `Service` interface. Two calls, each under the caller's deadline and cancellation.
///
/// # Errors
/// `Invalid` for a name that is not a `.service` of at most 256 bytes; the door's errors; `State`
/// for a unit not loaded and active; `Manager` for any reply outside its shape, or `MainPID` 0.
pub fn main_pid(
    config: &Config,
    unit: &str,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<u32, Error> {
    if unit.len() > 256 || !unit.ends_with(".service") || unit.len() == ".service".len() {
        return Err(Error::Invalid);
    }
    let listed = busctl(
        config,
        vec![
            "call".into(),
            SERVICE.into(),
            OBJECT.into(),
            MANAGER.into(),
            "ListUnitsByNames".into(),
            "as".into(),
            "1".into(),
            unit.into(),
        ],
        deadline,
        cancelled,
    )?
    .stdout()?;
    let (observation, object) = unit_reply(&listed, unit)?;
    if observation.load_state != "loaded" || observation.active_state != "active" {
        return Err(Error::State);
    }
    let property = busctl(
        config,
        vec![
            "get-property".into(),
            SERVICE.into(),
            object,
            "org.freedesktop.systemd1.Service".into(),
            "MainPID".into(),
        ],
        deadline,
        cancelled,
    )?
    .stdout()?;
    main_pid_reply(&property)
}

/// One `MainPID` property reply: exactly `{"type":"u","data":N}`. `N` = 0 is the manager's answer
/// for a unit with no main process (measured: an unloaded unit's path reads 0 with rc 0), so it is
/// refused as `Manager`, never returned as a pid.
///
/// # Errors
/// `Manager` for any other shape, signature, or 0.
pub fn main_pid_reply(bytes: &[u8]) -> Result<u32, Error> {
    let reply: Reply<u32> = serde_json::from_slice(bytes).map_err(|_| Error::Manager)?;
    if reply.signature != "u" || reply.data == 0 {
        return Err(Error::Manager);
    }
    Ok(reply.data)
}

fn job(bytes: &[u8]) -> Result<(), Error> {
    let reply: Reply<(String,)> = serde_json::from_slice(bytes).map_err(|_| Error::Manager)?;
    let id = reply
        .data
        .0
        .strip_prefix("/org/freedesktop/systemd1/job/")
        .ok_or(Error::Manager)?;
    if reply.signature != "o"
        || id.is_empty()
        || id.len() > 20
        || !id.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(Error::Manager);
    }
    Ok(())
}
fn unit_reply(bytes: &[u8], name: &str) -> Result<(UnitObservation, String), Error> {
    let reply: Reply<(Vec<UnitRow>,)> =
        serde_json::from_slice(bytes).map_err(|_| Error::Manager)?;
    if reply.signature != "a(ssssssouso)" || reply.data.0.len() != 1 {
        return Err(Error::Manager);
    }
    let row = reply.data.0.into_iter().next().ok_or(Error::Manager)?;
    if (row.2 == "not-found" && row.7 != 0)
        || row.0 != name
        || !row.5.is_empty()
        || !row.6.starts_with("/org/freedesktop/systemd1/unit/")
        || row.6.len() > 512
        || [&row.2, &row.3, &row.4].iter().any(|s| s.len() > 64)
    {
        return Err(Error::Manager);
    }
    Ok((
        UnitObservation {
            name: row.0,
            load_state: row.2,
            active_state: row.3,
            sub_state: row.4,
            control_group: None,
            job: row.7,
            observed_at: Instant::now(),
        },
        row.6,
    ))
}
fn aggregate_limits(fd: &File, deadline: Instant) -> Result<resources::Limits, Error> {
    let value =
        |name| io::text(fd, name, deadline).map(|s| s.strip_suffix('\n').unwrap_or(&s).to_owned());
    let limits = resources::Limits {
        cpu_max: value("cpu.max")?,
        memory_max: value("memory.max")?,
        memory_swap_max: value("memory.swap.max")?,
        pids_max: value("pids.max")?,
        io_weight: Some(value("io.weight")?),
    };
    let declared = resources::AGGREGATE_LIMITS;
    if limits.cpu_max != declared.cpu_max()
        || limits.memory_max != declared.memory_bytes.to_string()
        || limits.memory_swap_max != declared.swap_bytes.to_string()
        || limits.pids_max != declared.tasks.to_string()
        || limits.io_weight != Some(format!("default {}", resources::AGGREGATE_IO_WEIGHT))
    {
        return Err(Error::Limits);
    }
    Ok(limits)
}
fn empty(events: &str) -> Result<bool, Error> {
    let mut populated = None;
    let mut frozen = false;
    for line in events.lines() {
        match line {
            "populated 0" if populated.is_none() => populated = Some(true),
            "populated 1" if populated.is_none() => populated = Some(false),
            "frozen 0" | "frozen 1" if !frozen => frozen = true,
            _ => return Err(Error::Invalid),
        }
    }
    populated.ok_or(Error::Invalid)
}
/// The one name of an attempt's aggregate slice (R22 C1a): `hee3aggregate<attempt without
/// dashes>.slice`. The attempt row is committed before its check runs, so the ledger names the
/// slice before the slice exists, and a slice left by a crash is found by exact name.
/// Reclaimer note (B14b-2 review round 2, D7; owner B17): a slice left pending is named only by
/// its attempt's `RunCleanup` record (`resources: pending`), never by startup's selected set, which
/// closes the attempt on its leaves alone — the reclaimer reads its population from the run records.
pub(crate) fn aggregate_unit(attempt: UuidV4<'_>) -> String {
    format!("hee3aggregate{}.slice", attempt.as_str().replace('-', ""))
}
/// The manager has let the unit go: inactive, not found or holding no control group, and holding
/// no job for it (a queued start job would still realise it; B14b-2 closure D4).
fn settled(observation: &UnitObservation) -> bool {
    observation.active_state == "inactive"
        && (observation.load_state == "not-found"
            || observation.control_group.as_deref() == Some(""))
        && observation.job == 0
}
/// What a create request came to, from the one call it recorded (`None` when none was) and its
/// result: a job path is `Answered`; nothing recorded, a spawn refusal, or the manager's own error
/// reply (non-zero exit, nothing printed) is `Refused`; anything else may still reach the manager
/// and is `Unknown`, carrying its error.
fn create_outcome(call: Option<&Call>, result: Result<(), Error>) -> CreateOutcome {
    let Err(error) = result else {
        return CreateOutcome::Answered;
    };
    let refused = call.is_none_or(|call| match &call.report {
        Err(_) => true,
        Ok(report) => {
            error == Error::Manager
                && report.exit_code.is_some_and(|code| code != 0)
                && report.stdout.bytes.is_empty()
        }
    });
    if refused {
        CreateOutcome::Refused
    } else {
        CreateOutcome::Unknown(error)
    }
}
/// The slice's `StartTransientUnit` arguments: its name and the aggregate limits, nothing else.
fn create_arguments(unit: &str) -> Vec<String> {
    let properties = resources::aggregate_properties();
    // D-Bus `a(sv)`: the count of (name, value) pairs precedes them, derived from the one vector.
    let mut args = vec![
        "ssa(sv)a(sa(sv))".into(),
        unit.to_owned(),
        "fail".into(),
        (properties.len() / 3).to_string(),
    ];
    args.extend(properties);
    args.push("0".into());
    args
}
/// A unit's cgroup path under this user's manager.
fn slice_path(unit: &str) -> String {
    let uid = geteuid().as_raw();
    format!("/user.slice/user-{uid}.slice/user@{uid}.service/{unit}")
}
/// The one door from each phase to what [`Slice::stop`] does (R22-2): nothing sent or already
/// stopped is nothing; an asked create is resolved against the manager's row; a created slice is
/// stopped only when empty; an asked stop is awaited.
const fn stop_step(phase: SlicePhase) -> StopStep {
    match phase {
        SlicePhase::Prepared | SlicePhase::Stopped => StopStep::Nothing,
        SlicePhase::CreateRequested => StopStep::Resolve,
        SlicePhase::Created => StopStep::StopIfEmpty,
        SlicePhase::StopRequested => StopStep::AwaitSettled,
    }
}
/// What the manager's own row `o` says of a slice whose create was asked (R22-2), rows in order:
/// a job the manager still holds for the unit (a queued start: loaded, inactive, no control group,
/// which without the job reads as stopped) is never gone — after a create whose outcome is unknown
/// it is that create's error, otherwise a slice to stop (B14b-2 closure D4); settled with a create
/// refused or answered is gone; not found after a create whose outcome is
/// unknown is that create's error (a lost reply may still land: never recorded settled); loaded,
/// inactive, holding no control group is gone; any other load state is `Manager`; no control group
/// while active or activating is an unrealised slice to stop; its own path is a slice to check
/// empty; any other path is `Identity`.
fn resolve(
    o: &UnitObservation,
    expected_path: &str,
    created: CreateOutcome,
) -> Result<Resolved, Error> {
    if o.job != 0 {
        return match created {
            CreateOutcome::Unknown(error) => Err(error),
            CreateOutcome::Refused | CreateOutcome::Answered => Ok(Resolved::StopUnrealised),
        };
    }
    if settled(o) && matches!(created, CreateOutcome::Refused | CreateOutcome::Answered) {
        return Ok(Resolved::Gone);
    }
    if let (true, CreateOutcome::Unknown(error)) = (o.load_state == "not-found", created) {
        return Err(error);
    }
    let group = o.control_group.as_deref();
    if o.load_state == "loaded" && o.active_state == "inactive" && group == Some("") {
        return Ok(Resolved::Gone);
    }
    if o.load_state != "loaded" {
        return Err(Error::Manager);
    }
    if group == Some("") && matches!(o.active_state.as_str(), "active" | "activating") {
        return Ok(Resolved::StopUnrealised);
    }
    if group == Some(expected_path) {
        Ok(Resolved::CheckEmpty)
    } else {
        Err(Error::Identity)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Call, Config, CreateOutcome, Error, Resolved, Slice, SlicePhase, StopStep, UnitObservation,
        aggregate_unit, create_arguments, create_outcome, resolve, settled, slice_path, stop_step,
        unit_reply,
    };
    use crate::contracts::UuidV4;
    use crate::worker::process::{Interruption, ProcessReport, Refusal, SignalFacts, Stream};
    use std::time::{Duration, Instant};

    /// Two attempts differing in every hex digit except the version digit (R22 §5).
    const A1: &str = "01234567-89ab-4cde-8f01-23456789abcd";
    const A2: &str = "fedcba98-7654-4321-b0fe-dcba98765432";

    fn attempt(value: &str) -> Result<UuidV4<'_>, Box<dyn std::error::Error>> {
        UuidV4::parse(value).map_err(|e| format!("{e:?}").into())
    }

    /// One busctl call as the door records it: a reaped leader, a settled group, both streams
    /// complete, with this exit code, stdout and interruption.
    fn call(exit_code: Option<i32>, stdout: &[u8], interruption: Option<Interruption>) -> Call {
        let stream = |bytes: &[u8]| Stream {
            bytes: bytes.to_vec(),
            observed_bytes: bytes.len() as u64,
            eof: true,
            truncated: false,
            failed: false,
        };
        Call {
            argv: vec!["call".into()],
            report: Ok(ProcessReport {
                started_at: Instant::now(),
                leader_pid: 4817,
                exit_code,
                signal: None,
                interruption,
                interruption_observed_at: None,
                stdout: stream(stdout),
                stderr: stream(if exit_code == Some(0) { b"" } else { b"Failed" }),
                elapsed: Duration::from_millis(3),
                signals: SignalFacts::default(),
                leader_reaped: true,
                process_group_settled: true,
                observer_ready: true,
                pending: None,
            }),
        }
    }

    /// R22-2 · the teardown's table, whole: every phase has a step, nothing sent and already
    /// stopped do nothing, and there is no wedged step.
    #[test]
    fn stop_step_has_a_door_for_every_slice_phase() {
        assert_eq!(
            [
                SlicePhase::Prepared,
                SlicePhase::CreateRequested,
                SlicePhase::Created,
                SlicePhase::StopRequested,
                SlicePhase::Stopped,
            ]
            .map(|phase| (phase, stop_step(phase))),
            [
                (SlicePhase::Prepared, StopStep::Nothing),
                (SlicePhase::CreateRequested, StopStep::Resolve),
                (SlicePhase::Created, StopStep::StopIfEmpty),
                (SlicePhase::StopRequested, StopStep::AwaitSettled),
                (SlicePhase::Stopped, StopStep::Nothing),
            ]
        );
    }

    /// One manager row: load, active and sub state, the `ControlGroup` read (`None` when the unit
    /// is not found and none was read), and the id of the job the manager holds for it.
    fn row(load: &str, active: &str, sub: &str, group: Option<&str>, job: u32) -> UnitObservation {
        UnitObservation {
            name: "hee3probe4817.slice".to_owned(),
            load_state: load.to_owned(),
            active_state: active.to_owned(),
            sub_state: sub.to_owned(),
            control_group: group.map(str::to_owned),
            job,
            observed_at: Instant::now(),
        }
    }

    /// R22-2 · `resolve` over the manager's own rows. From the host (F113): the T06 stopped slice
    /// (`loaded/inactive/dead`, `ControlGroup ""`), the probe's collected scope (`not-found`), and
    /// the probe's live slice at its own path. Hand-typed, self-consistent only (labelled so): an
    /// activating and an active unrealised slice, and a load state that is not `loaded`. A
    /// not-found unit after an unknown create is that create's error, never gone; the stopped
    /// slice is gone whatever the create came to (row 3 decides a row of its own). The job column
    /// is hand-typed on every row: the T06 rows recorded none, so they carry `0`, and `queued` —
    /// the stopped slice's row with a start job still queued (B14b-2 closure D4) — carries 4817.
    /// A queued job is never gone and never settled: after an unknown create it is that create's
    /// error, otherwise a slice to stop.
    #[test]
    fn resolve_reads_the_manager_s_own_rows() -> Result<(), Box<dyn std::error::Error>> {
        const P: &str = "/user.slice/user-1000.slice/user@1000.service/hee3probe4817.slice";
        let a1_path = slice_path(&aggregate_unit(attempt(A1)?));
        let stopped = row("loaded", "inactive", "dead", Some(""), 0);
        let collected = row("not-found", "inactive", "dead", None, 0);
        let live = row("loaded", "active", "active", Some(P), 0);
        // Hand-typed rows: self-consistent, not recorded from a host.
        let activating = row("loaded", "activating", "start", Some(""), 0);
        let unrealised = row("loaded", "active", "active", Some(""), 0);
        let errored = row("error", "active", "running", Some(P), 0);
        let queued = row("loaded", "inactive", "dead", Some(""), 4817);
        assert_eq!(
            [
                resolve(&stopped, P, CreateOutcome::Answered),
                resolve(&stopped, P, CreateOutcome::Unknown(Error::Process)),
                resolve(&collected, P, CreateOutcome::Refused),
                resolve(&collected, P, CreateOutcome::Unknown(Error::Deadline)),
                resolve(&live, P, CreateOutcome::Answered),
                resolve(&live, &a1_path, CreateOutcome::Answered),
                resolve(&activating, P, CreateOutcome::Answered),
                resolve(&unrealised, P, CreateOutcome::Unknown(Error::Process)),
                resolve(&errored, P, CreateOutcome::Answered),
                resolve(&queued, P, CreateOutcome::Unknown(Error::Process)),
                resolve(&queued, P, CreateOutcome::Answered),
            ],
            [
                Ok(Resolved::Gone),
                Ok(Resolved::Gone),
                Ok(Resolved::Gone),
                Err(Error::Deadline),
                Ok(Resolved::CheckEmpty),
                Err(Error::Identity),
                Ok(Resolved::StopUnrealised),
                Ok(Resolved::StopUnrealised),
                Err(Error::Manager),
                Err(Error::Process),
                Ok(Resolved::StopUnrealised),
            ]
        );
        assert_eq!(
            [&stopped, &collected, &queued].map(settled),
            [true, true, false],
            "a queued job is not settled"
        );
        Ok(())
    }

    /// B14b-2 closure D4 · the manager's row carries the job it holds for the unit. The first
    /// reply is recorded from this host's user manager (`busctl --user --json=short call …
    /// ListUnitsByNames as 1 app.slice`, 2026-09-27): no job, so `0`. The second is that reply
    /// with the stopped probe slice's name, load and active state, and a queued start job's three
    /// fields hand-typed (a queued job was not recorded): it carries 4817.
    #[test]
    fn unit_reply_carries_the_queued_job() -> Result<(), Box<dyn std::error::Error>> {
        let recorded = br#"{"type":"a(ssssssouso)","data":[[["app.slice","User Application Slice","loaded","active","active","","/org/freedesktop/systemd1/unit/app_2eslice",0,"","/"]]]}"#;
        let queued = br#"{"type":"a(ssssssouso)","data":[[["hee3probe4817.slice","hee3probe4817.slice","loaded","inactive","dead","","/org/freedesktop/systemd1/unit/hee3probe4817_2eslice",4817,"start","/org/freedesktop/systemd1/job/4817"]]]}"#;
        let (app, app_object) = unit_reply(recorded, "app.slice").map_err(|e| format!("{e:?}"))?;
        let (probe, probe_object) =
            unit_reply(queued, "hee3probe4817.slice").map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            [
                (
                    app.name.as_str(),
                    app.load_state.as_str(),
                    app.active_state.as_str(),
                    app.sub_state.as_str(),
                    app.job,
                    app_object.as_str(),
                ),
                (
                    probe.name.as_str(),
                    probe.load_state.as_str(),
                    probe.active_state.as_str(),
                    probe.sub_state.as_str(),
                    probe.job,
                    probe_object.as_str(),
                ),
            ],
            [
                (
                    "app.slice",
                    "loaded",
                    "active",
                    "active",
                    0,
                    "/org/freedesktop/systemd1/unit/app_2eslice",
                ),
                (
                    "hee3probe4817.slice",
                    "loaded",
                    "inactive",
                    "dead",
                    4817,
                    "/org/freedesktop/systemd1/unit/hee3probe4817_2eslice",
                ),
            ]
        );
        assert_eq!(
            (app.control_group, probe.control_group),
            (None, None),
            "the row alone reads no control group"
        );
        Ok(())
    }

    /// R22-2 · only a manager answer (or nothing reaching it) counts as refused; a call that may
    /// still have reached the manager is unknown, carrying its error; a job path is answered. The
    /// last two rows each hold one clause of the manager answer false (a failed exit whose call
    /// did not settle; a failed exit that printed), so each clause decides a row of its own.
    #[test]
    fn create_outcome_counts_only_a_manager_answer_as_refused() {
        let spawn_refused = Call {
            argv: vec!["call".into()],
            report: Err(Refusal::Spawn),
        };
        assert_eq!(
            [
                create_outcome(None, Err(Error::Invalid)),
                create_outcome(Some(&spawn_refused), Err(Error::Process)),
                create_outcome(Some(&call(Some(1), b"", None)), Err(Error::Manager)),
                create_outcome(
                    Some(&call(None, b"", Some(Interruption::Timeout))),
                    Err(Error::Process)
                ),
                create_outcome(Some(&call(Some(0), b"x", None)), Err(Error::Manager)),
                create_outcome(
                    Some(&call(
                        Some(0),
                        br#"{"type":"o","data":["/org/freedesktop/systemd1/job/4817"]}"#,
                        None
                    )),
                    Ok(())
                ),
                create_outcome(
                    Some(&call(Some(1), b"", Some(Interruption::ResidualGroup))),
                    Err(Error::Process)
                ),
                create_outcome(
                    Some(&call(
                        Some(1),
                        br#"{"type":"o","data":["/org/freedesktop/systemd1/job/4817"]}"#,
                        None
                    )),
                    Err(Error::Manager)
                ),
            ],
            [
                CreateOutcome::Refused,
                CreateOutcome::Refused,
                CreateOutcome::Refused,
                CreateOutcome::Unknown(Error::Process),
                CreateOutcome::Unknown(Error::Manager),
                CreateOutcome::Answered,
                CreateOutcome::Unknown(Error::Process),
                CreateOutcome::Unknown(Error::Manager),
            ]
        );
    }

    /// R22 C1a · the slice's one name, from the attempt, over two attempts differing in every digit.
    #[test]
    fn aggregate_unit_names_the_attempt() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            [aggregate_unit(attempt(A1)?), aggregate_unit(attempt(A2)?)],
            [
                "hee3aggregate0123456789ab4cde8f0123456789abcd.slice",
                "hee3aggregatefedcba9876544321b0fedcba98765432.slice",
            ]
        );
        Ok(())
    }

    /// The slice is named after the attempt `prepare` is handed, not one it derives (P7 plant A:
    /// a constant attempt passed every test while the name sat behind the pin). Two attempts
    /// differing in every digit, each asserted as its whole unit name.
    #[test]
    fn a_prepared_slice_is_named_after_the_handed_attempt() -> Result<(), Box<dyn std::error::Error>>
    {
        let config = || Config {
            busctl: "/nonexistent/busctl".into(),
            busctl_sha256: String::new(),
            runtime_dir: "/nonexistent".into(),
        };
        let first = Slice::named(config(), attempt(A1)?);
        let second = Slice::named(config(), attempt(A2)?);
        assert_eq!(
            [first.unit(), second.unit()],
            [
                "hee3aggregate0123456789ab4cde8f0123456789abcd.slice",
                "hee3aggregatefedcba9876544321b0fedcba98765432.slice",
            ]
        );
        assert!(first.calls().is_empty() && second.calls().is_empty());
        assert_eq!(
            [first.phase, second.phase],
            [SlicePhase::Prepared, SlicePhase::Prepared]
        );
        Ok(())
    }

    /// R22-1 · the slice's create request, whole, for each attempt: its limits and nothing else —
    /// no `PIDs`, so no process is ever placed in it by the request.
    #[test]
    fn the_slice_request_carries_no_process() -> Result<(), Box<dyn std::error::Error>> {
        for (value, unit) in [
            (A1, "hee3aggregate0123456789ab4cde8f0123456789abcd.slice"),
            (A2, "hee3aggregatefedcba9876544321b0fedcba98765432.slice"),
        ] {
            assert_eq!(
                create_arguments(&aggregate_unit(attempt(value)?)),
                [
                    "ssa(sv)a(sa(sv))",
                    unit,
                    "fail",
                    "6",
                    "CPUQuotaPerSecUSec",
                    "t",
                    "4000000",
                    "MemoryMax",
                    "t",
                    "17179869184",
                    "MemorySwapMax",
                    "t",
                    "0",
                    "TasksMax",
                    "t",
                    "256",
                    "IOWeight",
                    "t",
                    "25",
                    "CollectMode",
                    "s",
                    "inactive-or-failed",
                    "0",
                ]
            );
        }
        Ok(())
    }
}
