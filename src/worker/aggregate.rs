//! One coordinator's transient aggregate; raw manager facts are not acceptance.
use super::{process, resources};
use crate::contracts::UuidV4;
use rustix::process::{geteuid, getpgrp, getppid, getsid};
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
    pub run_id: String,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Prepared,
    CreateRequested,
    SliceCreated,
    AttachRequested,
    Attached,
    RestoreRequested,
    Restored,
    StopRequested,
    Stopped,
}
/// What a create request came to (R22-2): a job path came back, nothing reached the manager (or it
/// answered with an error), or a failure after which the manager may still act on the request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CreateOutcome {
    Answered,
    Refused,
    Unknown(Error),
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
    pub observed_at: Instant,
}
#[derive(Debug)]
pub struct Live {
    pub coordinator_pid: u32,
    pub before: String,
    pub after: String,
    pub aggregate: String,
    pub limits: resources::Limits,
}
#[derive(Debug)]
pub struct Stopped {
    pub direct_empty_observed_at: Instant,
    pub manager: UnitObservation,
}
#[derive(Debug)]
pub struct Aggregate {
    config: Config,
    phase: Phase,
    unit: String,
    coordinator: String,
    origin: String,
    origin_unit: String,
    origin_subgroup: String,
    origin_fd: File,
    slice_fd: Option<File>,
    identity: [u32; 4],
    calls: Vec<Call>,
    empty_at: Option<Instant>,
    attach_refused: bool,
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

impl Aggregate {
    /// Capture the current coordinator and pin trusted manager inputs; no mutation.
    /// # Errors
    /// Refuses untrusted paths, source changes or an unsupported origin.
    pub fn prepare(config: Config, deadline: Instant) -> Result<Self, Error> {
        io::pin(&config, deadline)?;
        let attempt = UuidV4::parse(&config.run_id).map_err(|_| Error::Invalid)?;
        let unit = aggregate_unit(attempt);
        let stem = config.run_id.replace('-', "");
        let origin = io::membership(deadline)?;
        let (origin_unit, origin_subgroup) = origin_parts(&origin)?;
        let origin_fd = io::cgroup(&origin, deadline)?;
        Ok(Self {
            config,
            phase: Phase::Prepared,
            unit,
            coordinator: format!("hee3coordinator{stem}.scope"),
            origin,
            origin_unit,
            origin_subgroup,
            origin_fd,
            slice_fd: None,
            identity: identity()?,
            calls: vec![],
            empty_at: None,
            attach_refused: false,
        })
    }
    #[must_use]
    pub fn unit(&self) -> &str {
        &self.unit
    }
    #[must_use]
    pub fn coordinator_unit(&self) -> &str {
        &self.coordinator
    }
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }
    #[must_use]
    pub fn calls(&self) -> &[Call] {
        &self.calls
    }
    pub fn take_calls(&mut self) -> Vec<Call> {
        std::mem::take(&mut self.calls)
    }
    fn aggregate_path(&self) -> String {
        let uid = geteuid().as_raw();
        format!(
            "/user.slice/user-{uid}.slice/user@{uid}.service/{}",
            self.unit
        )
    }
    fn coordinator_path(&self) -> String {
        format!("{}/{}", self.aggregate_path(), self.coordinator)
    }
    fn same_owner(&self) -> Result<(), Error> {
        if identity()? == self.identity {
            Ok(())
        } else {
            Err(Error::Identity)
        }
    }
    /// Apply aggregate limits and attach this same process before candidate work.
    /// # Errors
    /// Any refusal leaves phase and raw calls available for reconciliation.
    pub fn start(&mut self, deadline: Instant, cancelled: &AtomicBool) -> Result<Live, Error> {
        if self.phase != Phase::Prepared {
            return Err(Error::State);
        }
        self.same_owner()?;
        if io::membership(deadline)? != self.origin {
            return Err(Error::Identity);
        }
        for name in [self.unit.clone(), self.coordinator.clone()] {
            absent(&self.config, &mut self.calls, name, deadline, cancelled)?;
        }
        self.phase = Phase::CreateRequested;
        create_slice(
            &self.config,
            &mut self.calls,
            &self.unit,
            deadline,
            cancelled,
        )?;
        self.phase = Phase::SliceCreated;
        self.slice_fd = Some(capture(&self.aggregate_path(), deadline, cancelled)?);
        self.phase = Phase::AttachRequested;
        let args = vec![
            "ssa(sv)a(sa(sv))".into(),
            self.coordinator.clone(),
            "fail".into(),
            "3".into(),
            "Slice".into(),
            "s".into(),
            self.unit.clone(),
            "PIDs".into(),
            "au".into(),
            "1".into(),
            self.identity[0].to_string(),
            "CollectMode".into(),
            "s".into(),
            "inactive-or-failed".into(),
            "0".into(),
        ];
        self.request_attach(args, deadline, cancelled)?;
        self.wait_membership(&self.coordinator_path(), deadline, cancelled)?;
        let after = io::membership(deadline)?;
        let aggregate = self.slice_fd.as_ref().ok_or(Error::State)?;
        let limits = aggregate_limits(aggregate, deadline)?;
        io::stable(aggregate, &self.aggregate_path(), deadline)?;
        self.same_owner()?;
        if io::membership(deadline)? != after {
            return Err(Error::Identity);
        }
        self.phase = Phase::Attached;
        Ok(Live {
            coordinator_pid: self.identity[0],
            before: self.origin.clone(),
            after,
            aggregate: self.unit.clone(),
            limits,
        })
    }
    /// Observe manager state separately from direct cgroup/namespace cleanup.
    /// # Errors
    /// Refuses a foreign aggregate, failed query or raced property disappearance.
    pub fn observe_candidate(
        &mut self,
        scope: &resources::Scope,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<UnitObservation, Error> {
        if scope.aggregate != self.unit {
            return Err(Error::Identity);
        }
        let name = scope.unit().map_err(|_| Error::Invalid)?;
        query(&self.config, &mut self.calls, &name, deadline, cancelled)
    }
    /// Restore only this current coordinator; never stop a unit containing it.
    /// # Errors
    /// Preserves pending phases on failed or incomplete migration.
    pub fn restore_origin(&mut self, deadline: Instant) -> Result<(), Error> {
        self.same_owner()?;
        let uncancelled = AtomicBool::new(false);
        if self.phase == Phase::AttachRequested
            && self.attach_refused
            && io::membership(deadline)? == self.origin
        {
            io::stable(&self.origin_fd, &self.origin, deadline)?;
            io::stable(
                self.slice_fd.as_ref().ok_or(Error::State)?,
                &self.aggregate_path(),
                deadline,
            )?;
            let coordinator = query(
                &self.config,
                &mut self.calls,
                &self.coordinator,
                deadline,
                &uncancelled,
            )?;
            if coordinator.load_state != "not-found"
                || coordinator.active_state != "inactive"
                || coordinator.sub_state != "dead"
            {
                return Err(Error::Busy);
            }
            io::stable(
                self.slice_fd.as_ref().ok_or(Error::State)?,
                &self.aggregate_path(),
                deadline,
            )?;
            io::stable(&self.origin_fd, &self.origin, deadline)?;
            if io::membership(deadline)? != self.origin {
                return Err(Error::Identity);
            }
            self.phase = Phase::Restored;
            return Ok(());
        }
        match self.phase {
            Phase::Attached | Phase::AttachRequested => {
                self.wait_membership(&self.coordinator_path(), deadline, &uncancelled)?;
                io::stable(&self.origin_fd, &self.origin, deadline)?;
                self.phase = Phase::RestoreRequested;
                let args = vec![
                    "ssau".into(),
                    self.origin_unit.clone(),
                    self.origin_subgroup.clone(),
                    "1".into(),
                    self.identity[0].to_string(),
                ];
                if !method(
                    &self.config,
                    &mut self.calls,
                    "AttachProcessesToUnit",
                    args,
                    deadline,
                    &uncancelled,
                )?
                .is_empty()
                {
                    return Err(Error::Manager);
                }
            }
            Phase::RestoreRequested | Phase::SliceCreated | Phase::Restored => {}
            _ => return Err(Error::State),
        }
        self.wait_membership(&self.origin, deadline, &uncancelled)?;
        io::stable(&self.origin_fd, &self.origin, deadline)?;
        self.phase = Phase::Restored;
        Ok(())
    }
    /// Requires direct aggregate populated0 after restoring this coordinator.
    /// # Errors
    /// Missing cgroup, live descendants or manager failure remain unresolved.
    pub fn stop_if_empty(&mut self, deadline: Instant) -> Result<Stopped, Error> {
        self.same_owner()?;
        if !matches!(self.phase, Phase::Restored | Phase::StopRequested)
            || io::membership(deadline)? != self.origin
        {
            return Err(Error::State);
        }
        io::stable(&self.origin_fd, &self.origin, deadline)?;
        if self.phase == Phase::Restored {
            if self.slice_fd.is_none() {
                self.slice_fd = Some(io::cgroup(&self.aggregate_path(), deadline)?);
            }
            let path = self.aggregate_path();
            let fd = self.slice_fd.as_ref().ok_or(Error::State)?;
            let (phase, empty_at) = (&mut self.phase, &mut self.empty_at);
            stop_empty(
                &self.config,
                &mut self.calls,
                fd,
                &path,
                &self.unit,
                deadline,
                |at| {
                    *empty_at = Some(at);
                    *phase = Phase::StopRequested;
                },
            )?;
        }
        let observation = await_settled(&self.config, &mut self.calls, &self.unit, deadline)?;
        self.phase = Phase::Stopped;
        Ok(Stopped {
            direct_empty_observed_at: self.empty_at.ok_or(Error::State)?,
            manager: observation,
        })
    }
    fn request_attach(
        &mut self,
        args: Vec<String>,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<(), Error> {
        let recorded = self.calls.len();
        let result = method(
            &self.config,
            &mut self.calls,
            "StartTransientUnit",
            args,
            deadline,
            cancelled,
        )
        .and_then(|bytes| job(&bytes));
        match create_outcome(self.calls.get(recorded), result) {
            CreateOutcome::Answered => Ok(()),
            CreateOutcome::Unknown(error) => Err(error),
            CreateOutcome::Refused => {
                self.attach_refused = true;
                result
            }
        }
    }
    fn wait_membership(
        &self,
        expected: &str,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<(), Error> {
        loop {
            io::tick(deadline)?;
            if cancelled.load(Ordering::Acquire) {
                return Err(Error::Cancelled);
            }
            self.same_owner()?;
            if io::membership(deadline)? == expected {
                return Ok(());
            }
            std::thread::sleep(
                Duration::from_millis(2).min(deadline.saturating_duration_since(Instant::now())),
            );
        }
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

fn identity() -> Result<[u32; 4], Error> {
    Ok([
        std::process::id(),
        getppid().map_or(0, |v| v.as_raw_nonzero().get().cast_unsigned()),
        getpgrp().as_raw_nonzero().get().cast_unsigned(),
        getsid(None)
            .map_err(|_| Error::Io)?
            .as_raw_nonzero()
            .get()
            .cast_unsigned(),
    ])
}
fn origin_parts(path: &str) -> Result<(String, String), Error> {
    let parts: Vec<_> = path.split('/').filter(|s| !s.is_empty()).collect();
    let index = parts
        .iter()
        .rposition(|s| {
            s.rsplit_once('.')
                .is_some_and(|(_, kind)| matches!(kind, "scope" | "service"))
        })
        .ok_or(Error::Invalid)?;
    Ok((
        parts[index].into(),
        format!("/{}", parts[index + 1..].join("/")),
    ))
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
pub(crate) fn aggregate_unit(attempt: UuidV4<'_>) -> String {
    format!("hee3aggregate{}.slice", attempt.as_str().replace('-', ""))
}
/// The manager has let the unit go: inactive, and not found or holding no control group.
fn settled(observation: &UnitObservation) -> bool {
    observation.active_state == "inactive"
        && (observation.load_state == "not-found"
            || observation.control_group.as_deref() == Some(""))
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

#[cfg(test)]
mod tests {
    use super::{Call, CreateOutcome, Error, aggregate_unit, create_arguments, create_outcome};
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
