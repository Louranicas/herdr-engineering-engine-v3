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
        UuidV4::parse(&config.run_id).map_err(|_| Error::Invalid)?;
        let stem = config.run_id.replace('-', "");
        let origin = io::membership(deadline)?;
        let (origin_unit, origin_subgroup) = origin_parts(&origin)?;
        let origin_fd = io::cgroup(&origin, deadline)?;
        Ok(Self {
            config,
            phase: Phase::Prepared,
            unit: format!("hee3aggregate{stem}.slice"),
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
            let bytes = self.method(
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
            if !reply.data.0.is_empty() {
                return Err(Error::Identity);
            }
        }
        self.phase = Phase::CreateRequested;
        let args = vec![
            "ssa(sv)a(sa(sv))".into(),
            self.unit.clone(),
            "fail".into(),
            "6".into(),
            "CPUQuotaPerSecUSec".into(),
            "t".into(),
            "4000000".into(),
            "MemoryMax".into(),
            "t".into(),
            "17179869184".into(),
            "MemorySwapMax".into(),
            "t".into(),
            "0".into(),
            "TasksMax".into(),
            "t".into(),
            "256".into(),
            "IOWeight".into(),
            "t".into(),
            "25".into(),
            "CollectMode".into(),
            "s".into(),
            "inactive-or-failed".into(),
            "0".into(),
        ];
        job(&self.method("StartTransientUnit", args, deadline, cancelled)?)?;
        self.phase = Phase::SliceCreated;
        self.slice_fd = Some(self.capture_slice(deadline, cancelled)?);
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
        self.query(&name, deadline, cancelled)
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
            let coordinator = self.query(&self.coordinator.clone(), deadline, &uncancelled)?;
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
                if !self
                    .method("AttachProcessesToUnit", args, deadline, &uncancelled)?
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
        let uncancelled = AtomicBool::new(false);
        if self.phase == Phase::Restored {
            if self.slice_fd.is_none() {
                self.slice_fd = Some(io::cgroup(&self.aggregate_path(), deadline)?);
            }
            let fd = self.slice_fd.as_ref().ok_or(Error::State)?;
            io::stable(fd, &self.aggregate_path(), deadline)?;
            if !empty(&io::text(fd, "cgroup.events", deadline)?)? {
                return Err(Error::Busy);
            }
            io::stable(fd, &self.aggregate_path(), deadline)?;
            self.empty_at = Some(Instant::now());
            self.phase = Phase::StopRequested;
            job(&self.method(
                "StopUnit",
                vec!["ss".into(), self.unit.clone(), "fail".into()],
                deadline,
                &uncancelled,
            )?)?;
        }
        loop {
            io::tick(deadline)?;
            let observation = self.query(&self.unit.clone(), deadline, &uncancelled)?;
            if observation.active_state == "inactive"
                && (observation.load_state == "not-found"
                    || observation.control_group.as_deref() == Some(""))
            {
                self.phase = Phase::Stopped;
                return Ok(Stopped {
                    direct_empty_observed_at: self.empty_at.ok_or(Error::State)?,
                    manager: observation,
                });
            }
            std::thread::sleep(
                Duration::from_millis(2).min(deadline.saturating_duration_since(Instant::now())),
            );
        }
    }
    fn request_attach(
        &mut self,
        args: Vec<String>,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<(), Error> {
        let recorded = self.calls.len();
        let response = match self.method("StartTransientUnit", args, deadline, cancelled) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.attach_refused = self.calls.len() == recorded
                    || self.calls.last().is_some_and(|call| match &call.report {
                        Err(_) => true,
                        Ok(report) => {
                            error == Error::Manager
                                && report.exit_code.is_some_and(|code| code != 0)
                                && report.stdout.bytes.is_empty()
                        }
                    });
                return Err(error);
            }
        };
        job(&response)
    }
    fn capture_slice(&self, deadline: Instant, cancelled: &AtomicBool) -> Result<File, Error> {
        loop {
            io::tick(deadline)?;
            if cancelled.load(Ordering::Acquire) {
                return Err(Error::Cancelled);
            }
            match io::cgroup(&self.aggregate_path(), deadline) {
                Ok(file) => {
                    aggregate_limits(&file, deadline)?;
                    io::stable(&file, &self.aggregate_path(), deadline)?;
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
    fn method(
        &mut self,
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
        self.command(argv, deadline, cancelled)
    }
    fn command(
        &mut self,
        parameters: Vec<String>,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<Vec<u8>, Error> {
        if self.calls.len() >= 64 {
            return Err(Error::Bound);
        }
        io::pin(&self.config, deadline)?;
        let mut argv = vec![
            OsString::from("--user"),
            "--json=short".into(),
            "--no-pager".into(),
            "--allow-interactive-authorization=no".into(),
        ];
        argv.extend(parameters.into_iter().map(Into::into));
        let spec = process::ProcessSpec {
            executable: self.config.busctl.clone(),
            arguments: argv.clone(),
            directory: "/".into(),
            environment: vec![(
                "XDG_RUNTIME_DIR".into(),
                self.config.runtime_dir.clone().into_os_string(),
            )],
            input: vec![],
            stream_limit: 65536,
        };
        let report = process::run(&spec, deadline, cancelled);
        self.calls.push(Call { argv, report });
        let report = self
            .calls
            .last()
            .and_then(|c| c.report.as_ref().ok())
            .ok_or(Error::Process)?;
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
    fn query(
        &mut self,
        name: &str,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<UnitObservation, Error> {
        let bytes = self.method(
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
            let bytes = self.command(
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
            let reply: Reply<String> =
                serde_json::from_slice(&bytes).map_err(|_| Error::Manager)?;
            if reply.signature != "s" || reply.data.len() > 2048 {
                return Err(Error::Manager);
            }
            observation.control_group = Some(reply.data);
        }
        observation.observed_at = Instant::now();
        io::tick(deadline)?;
        Ok(observation)
    }
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
    if limits.cpu_max != "400000 100000"
        || limits.memory_max != "17179869184"
        || limits.memory_swap_max != "0"
        || limits.pids_max != "256"
        || limits.io_weight.as_deref() != Some("default 25")
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
