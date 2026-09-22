//! Fixed RC01 scope wrapping and bounded observational cgroup readback.
//! The caller owns fresh names, the aggregate, live PID custody and cleanup.
use super::process::ProcessSpec;
use crate::contracts::{Sha256Digest, UuidV4};
use rustix::fs::{Mode, OFlags, PROC_SUPER_MAGIC, fstatfs, open, openat};
use rustix::process::geteuid;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs::{File, Metadata};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::time::Instant;

const DIRECTORY: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);
const CGROUP2_MAGIC: i64 = 0x6367_7270;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Invalid,
    Bound,
    Deadline,
    Io,
    Identity,
    Limits,
}
#[derive(Clone, Debug)]
pub struct Scope {
    pub systemd_run: PathBuf,
    pub systemd_run_sha256: String,
    pub runtime_dir: PathBuf,
    pub run_id: String,
    pub aggregate: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Limits {
    pub cpu_max: String,
    pub memory_max: String,
    pub memory_swap_max: String,
    pub pids_max: String,
    pub io_weight: Option<String>,
}
#[derive(Debug)]
pub struct Facts {
    pub unit: String,
    pub aggregate: String,
    pub control_group: String,
    pub leader_pid: u32,
    pub candidate: Limits,
    pub parent: Limits,
    pub observed_at: Instant,
}
#[derive(Debug)]
pub struct VerifiedScope {
    facts: Facts,
    candidate: File,
    parent: File,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Population {
    Populated,
    EmptyObserved,
    Unknown(Error),
}

impl Scope {
    /// # Errors
    /// Refuses malformed UUIDs or names; freshness belongs to the coordinator.
    pub fn unit(&self) -> Result<String, Error> {
        UuidV4::parse(&self.run_id).map_err(|_| Error::Invalid)?;
        let stem = self
            .aggregate
            .strip_suffix(".slice")
            .ok_or(Error::Invalid)?;
        if stem.is_empty() || stem.len() > 80 || !stem.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return Err(Error::Invalid);
        }
        Ok(format!(
            "hee3-resource-{}.scope",
            self.run_id.replace('-', "")
        ))
    }
    fn parent_path(&self) -> String {
        let uid = geteuid().as_raw();
        format!(
            "/user.slice/user-{uid}.slice/user@{uid}.service/{}",
            self.aggregate
        )
    }
    fn launcher(&self, deadline: Instant) -> Result<(), Error> {
        tick(deadline)?;
        self.unit()?;
        Sha256Digest::parse(&self.systemd_run_sha256).map_err(|_| Error::Invalid)?;
        if self.systemd_run != Path::new("/usr/bin/systemd-run")
            || self.runtime_dir.as_os_str()
                != std::ffi::OsStr::new(&format!("/run/user/{}", geteuid().as_raw()))
        {
            return Err(Error::Invalid);
        }
        for path in ["/usr", "/usr/bin", "/run", "/run/user"] {
            let m = directory(Path::new(path), deadline)?
                .metadata()
                .map_err(|_| Error::Io)?;
            if m.uid() != 0 || m.mode() & 0o022 != 0 {
                return Err(Error::Identity);
            }
        }
        let runtime = directory(&self.runtime_dir, deadline)?;
        let m = runtime.metadata().map_err(|_| Error::Io)?;
        if m.uid() != geteuid().as_raw() || m.mode() & 0o7777 != 0o700 {
            return Err(Error::Identity);
        }
        let bin = directory(Path::new("/usr/bin"), deadline)?;
        let mut file = regular(&bin, "systemd-run", deadline)?;
        let before = file.metadata().map_err(|_| Error::Io)?;
        if before.uid() != 0
            || before.mode() & 0o7022 != 0
            || before.mode() & 0o111 == 0
            || before.len() > 4 * 1024 * 1024
        {
            return Err(Error::Identity);
        }
        let bytes = bounded(&mut file, 4 * 1024 * 1024, deadline)?;
        let digest = Sha256::digest(&bytes);
        let mut hex = String::with_capacity(64);
        for byte in digest {
            write!(&mut hex, "{byte:02x}").map_err(|_| Error::Invalid)?;
        }
        let after = file.metadata().map_err(|_| Error::Io)?;
        let named = regular(&bin, "systemd-run", deadline)?
            .metadata()
            .map_err(|_| Error::Io)?;
        if format!("sha256:{hex}") != self.systemd_run_sha256
            || bytes.len() as u64 != before.len()
            || !unchanged(&before, &after)
            || !unchanged(&before, &named)
        {
            return Err(Error::Identity);
        }
        same_directory(&runtime, &self.runtime_dir, deadline)?;
        tick(deadline)
    }
    /// Preserve the inner command and pipes; add only fixed scope controls.
    /// # Errors
    /// Refuses invalid identity, environment, source pin or original deadline.
    pub fn wrap(&self, mut child: ProcessSpec, deadline: Instant) -> Result<ProcessSpec, Error> {
        self.launcher(deadline)?;
        if child.executable != Path::new("/usr/bin/bwrap") || !child.environment.is_empty() {
            return Err(Error::Invalid);
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(Error::Deadline)?
            .as_micros();
        if remaining == 0 {
            return Err(Error::Deadline);
        }
        if remaining > 1_200_000_000 {
            return Err(Error::Bound);
        }
        let mut arguments = [
            "--user",
            "--scope",
            "--quiet",
            "--collect",
            "--no-ask-password",
            "--expand-environment=no",
        ]
        .map(Into::into)
        .to_vec();
        arguments.extend(
            [
                format!("--unit={}", self.unit()?),
                format!("--slice={}", self.aggregate),
                "--property=CPUQuota=200%".into(),
                "--property=MemoryMax=8589934592".into(),
                "--property=MemorySwapMax=0".into(),
                "--property=TasksMax=128".into(),
                "--property=TimeoutStopSec=5s".into(),
                format!("--property=RuntimeMaxSec={remaining}us"),
            ]
            .into_iter()
            .map(Into::into),
        );
        arguments.push(child.executable.clone().into_os_string());
        arguments.append(&mut child.arguments);
        child.executable.clone_from(&self.systemd_run);
        child.arguments = arguments;
        child.environment.push((
            "XDG_RUNTIME_DIR".into(),
            self.runtime_dir.clone().into_os_string(),
        ));
        tick(deadline)?;
        Ok(child)
    }
    /// Called while the existing owner retains the live leader before release.
    /// # Errors
    /// Refuses missing, substituted, malformed or mismatching profile readback.
    pub fn observe(&self, owned_live_pid: u32, deadline: Instant) -> Result<VerifiedScope, Error> {
        tick(deadline)?;
        let unit = self.unit()?;
        if owned_live_pid == 0 {
            return Err(Error::Invalid);
        }
        let proc_path = PathBuf::from(format!("/proc/{owned_live_pid}"));
        let process = directory(&proc_path, deadline)?;
        if fstatfs(&process).map_err(|_| Error::Io)?.f_type != PROC_SUPER_MAGIC {
            return Err(Error::Identity);
        }
        let parent_path = self.parent_path();
        let control_group = format!("{parent_path}/{unit}");
        let expected = format!("0::{control_group}\n");
        if read(&process, "cgroup", deadline)? != expected {
            return Err(Error::Identity);
        }
        let root = Path::new("/sys/fs/cgroup");
        let parent_name = root.join(parent_path.trim_start_matches('/'));
        let candidate_name = root.join(control_group.trim_start_matches('/'));
        let parent = directory(&parent_name, deadline)?;
        let candidate = directory(&candidate_name, deadline)?;
        for fd in [&parent, &candidate] {
            if fstatfs(fd).map_err(|_| Error::Io)?.f_type != CGROUP2_MAGIC {
                return Err(Error::Identity);
            }
        }
        let parent_limits = limits(&parent, true, deadline)?;
        let candidate_limits = limits(&candidate, false, deadline)?;
        validate_limits(&candidate_limits, false)?;
        validate_limits(&parent_limits, true)?;
        same_directory(&parent, &parent_name, deadline)?;
        same_directory(&candidate, &candidate_name, deadline)?;
        if read(&process, "cgroup", deadline)? != expected {
            return Err(Error::Identity);
        }
        same_directory(&process, &proc_path, deadline)?;
        tick(deadline)?;
        Ok(VerifiedScope {
            facts: Facts {
                unit,
                aggregate: self.aggregate.clone(),
                control_group,
                leader_pid: owned_live_pid,
                candidate: candidate_limits,
                parent: parent_limits,
                observed_at: Instant::now(),
            },
            candidate,
            parent,
        })
    }
}
impl VerifiedScope {
    #[must_use]
    pub const fn facts(&self) -> &Facts {
        &self.facts
    }
    /// Missing/deleted units never imply complete cleanup.
    #[must_use]
    pub fn population(&self, deadline: Instant) -> Population {
        match self.read_population(deadline) {
            Ok(value) => value,
            Err(error) => Population::Unknown(error),
        }
    }
    fn read_population(&self, deadline: Instant) -> Result<Population, Error> {
        let name =
            Path::new("/sys/fs/cgroup").join(self.facts.control_group.trim_start_matches('/'));
        same_directory(&self.candidate, &name, deadline)?;
        same_directory(&self.parent, name.parent().ok_or(Error::Invalid)?, deadline)?;
        let events = read(&self.candidate, "cgroup.events", deadline)?;
        let value = parse_population(&events)?;
        same_directory(&self.candidate, &name, deadline)?;
        tick(deadline)?;
        Ok(value)
    }
}
fn tick(deadline: Instant) -> Result<(), Error> {
    if Instant::now() >= deadline {
        Err(Error::Deadline)
    } else {
        Ok(())
    }
}
fn directory(path: &Path, deadline: Instant) -> Result<File, Error> {
    tick(deadline)?;
    if !path.is_absolute() {
        return Err(Error::Invalid);
    }
    let mut file = File::from(open("/", DIRECTORY, Mode::empty()).map_err(|_| Error::Io)?);
    for part in path.components() {
        tick(deadline)?;
        match part {
            Component::RootDir => {}
            Component::Normal(name) => {
                file = File::from(
                    openat(&file, name, DIRECTORY, Mode::empty()).map_err(|_| Error::Io)?,
                );
            }
            _ => return Err(Error::Invalid),
        }
    }
    Ok(file)
}
fn regular(parent: &File, name: &str, deadline: Instant) -> Result<File, Error> {
    tick(deadline)?;
    let file = File::from(
        openat(
            parent,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        )
        .map_err(|_| Error::Io)?,
    );
    if !file.metadata().map_err(|_| Error::Io)?.is_file() {
        return Err(Error::Identity);
    }
    Ok(file)
}
fn bounded(file: &mut File, cap: usize, deadline: Instant) -> Result<Vec<u8>, Error> {
    let mut result = Vec::new();
    loop {
        tick(deadline)?;
        let mut chunk = [0_u8; 4096];
        let n = file.read(&mut chunk).map_err(|_| Error::Io)?;
        if n == 0 {
            break;
        }
        if result.len().checked_add(n).is_none_or(|total| total > cap) {
            return Err(Error::Bound);
        }
        result.extend_from_slice(&chunk[..n]);
    }
    tick(deadline)?;
    Ok(result)
}
fn read(parent: &File, name: &str, deadline: Instant) -> Result<String, Error> {
    let mut file = regular(parent, name, deadline)?;
    String::from_utf8(bounded(&mut file, 4096, deadline)?).map_err(|_| Error::Invalid)
}
fn same_directory(file: &File, path: &Path, deadline: Instant) -> Result<(), Error> {
    let a = file.metadata().map_err(|_| Error::Io)?;
    let b = directory(path, deadline)?
        .metadata()
        .map_err(|_| Error::Io)?;
    if a.dev() != b.dev() || a.ino() != b.ino() {
        return Err(Error::Identity);
    }
    tick(deadline)
}
fn unchanged(a: &Metadata, b: &Metadata) -> bool {
    a.dev() == b.dev()
        && a.ino() == b.ino()
        && a.len() == b.len()
        && a.mode() == b.mode()
        && a.uid() == b.uid()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
}
fn limits(fd: &File, parent: bool, deadline: Instant) -> Result<Limits, Error> {
    let value =
        |name| read(fd, name, deadline).map(|s| s.strip_suffix('\n').unwrap_or(&s).to_owned());
    Ok(Limits {
        cpu_max: value("cpu.max")?,
        memory_max: value("memory.max")?,
        memory_swap_max: value("memory.swap.max")?,
        pids_max: value("pids.max")?,
        io_weight: if parent {
            Some(value("io.weight")?)
        } else {
            None
        },
    })
}
fn validate_limits(value: &Limits, parent: bool) -> Result<(), Error> {
    let (cpu, memory, tasks) = if parent {
        ("400000 100000", "17179869184", "256")
    } else {
        ("200000 100000", "8589934592", "128")
    };
    if value.cpu_max != cpu
        || value.memory_max != memory
        || value.memory_swap_max != "0"
        || value.pids_max != tasks
        || (parent && value.io_weight.as_deref() != Some("default 25"))
    {
        return Err(Error::Limits);
    }
    Ok(())
}
fn parse_population(events: &str) -> Result<Population, Error> {
    let mut populated = None;
    let mut frozen = false;
    for line in events.lines() {
        match line {
            "populated 0" if populated.is_none() => populated = Some(Population::EmptyObserved),
            "populated 1" if populated.is_none() => populated = Some(Population::Populated),
            "frozen 0" | "frozen 1" if !frozen => frozen = true,
            _ => return Err(Error::Invalid),
        }
    }
    populated.ok_or(Error::Invalid)
}
