//! Fixed in-namespace exec shim for the T06 TH-DEV collector boundary.
//!
//! Bubblewrap launches this package-owned code after the coordinator has read
//! its JSON child identity and released the startup gate. The shim clears the
//! inherited environment, spawns exactly one candidate, waits for that child,
//! and reports its native wait status through a collector-owned FIFO.

use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fs::{FileType, Mode, OFlags, fcntl_getfl, fcntl_setfl, fstat, open},
    io::{Errno, read, write},
    process::{
        DumpableBehavior, Resource, Rlimit, dumpable_behavior, getrlimit, set_dumpable_behavior,
        setrlimit,
    },
};
use std::{
    fs::{self, File},
    os::unix::process::ExitStatusExt,
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant},
};

pub const STDOUT_FIFO: &str = "/channels/stdout";
pub const STDERR_FIFO: &str = "/channels/stderr";
pub const READY_FIFO: &str = "/channels/ready";
pub const RELEASE_FIFO: &str = "/channels/release";
pub const STATUS_FIFO: &str = "/channels/status";
pub const RELEASE_BYTE: u8 = 1;
pub const PROTECTED_RELEASE_BYTE: u8 = 2;
const READY_PREFIX: &str = "HEE3_NAMESPACE_READY_V1 pid=";
// Source readback can cover a declared finite toolchain closure before release.
// This startup gate is separate from the outer process cleanup reserve.
const READY_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_ARGUMENTS: usize = 256;
const MAX_ARGUMENT_BYTES: usize = 1_048_576;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShimError {
    InvalidExecutable,
    InvalidArgument,
    ArgumentLimit,
    InvalidWorkingDirectory,
    ChannelOpen,
    ChannelType,
    ChannelFlags,
    ReadyWrite,
    ReleaseTimeout,
    ReleaseRead,
    ReleaseValue,
    Exec,
    Protection,
    Status,
    Wait,
}

impl ShimError {
    #[must_use]
    pub const fn diagnostic(self) -> &'static str {
        match self {
            Self::InvalidExecutable => "namespace shim: invalid executable",
            Self::InvalidArgument => "namespace shim: invalid argument",
            Self::ArgumentLimit => "namespace shim: argument limit",
            Self::InvalidWorkingDirectory => "namespace shim: invalid working directory",
            Self::ChannelOpen => "namespace shim: channel open failed",
            Self::ChannelType => "namespace shim: channel is not fifo",
            Self::ChannelFlags => "namespace shim: channel flags failed",
            Self::ReadyWrite => "namespace shim: ready write failed",
            Self::ReleaseTimeout => "namespace shim: release timeout",
            Self::ReleaseRead => "namespace shim: release read failed",
            Self::ReleaseValue => "namespace shim: invalid release",
            Self::Exec => "namespace shim: exec failed",
            Self::Protection => "namespace shim: protection failed",
            Self::Status => "namespace shim: status failed",
            Self::Wait => "namespace shim: wait failed",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NamespaceExec<'a> {
    pub executable: &'a str,
    pub arguments: &'a [&'a str],
    pub working_directory: &'a str,
}

fn clean_absolute(value: &str) -> bool {
    value.starts_with('/')
        && value.len() <= 4096
        && !value.as_bytes().contains(&0)
        && value
            .split('/')
            .skip(1)
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn allowed_executable(value: &str) -> bool {
    clean_absolute(value)
        && ["/toolchain/bin/", "/work/bin/", "/frozen/bin/"]
            .iter()
            .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len())
}

fn allowed_working_directory(value: &str) -> bool {
    clean_absolute(value)
        && (value == "/work"
            || value.starts_with("/work/")
            || value == "/tmp"
            || value.starts_with("/tmp/"))
}

/// # Errors
/// Refuses disallowed executable/cwd paths, overbound arguments or NUL bytes.
pub fn validate(input: &NamespaceExec<'_>) -> Result<(), ShimError> {
    if !allowed_executable(input.executable) {
        return Err(ShimError::InvalidExecutable);
    }
    if !allowed_working_directory(input.working_directory) {
        return Err(ShimError::InvalidWorkingDirectory);
    }
    if input.arguments.len() > MAX_ARGUMENTS {
        return Err(ShimError::ArgumentLimit);
    }
    let mut bytes = 0_usize;
    for argument in input.arguments {
        if argument.as_bytes().contains(&0) || argument.len() > 65_536 {
            return Err(ShimError::InvalidArgument);
        }
        bytes = bytes
            .checked_add(argument.len())
            .ok_or(ShimError::ArgumentLimit)?;
    }
    if bytes > MAX_ARGUMENT_BYTES {
        return Err(ShimError::ArgumentLimit);
    }
    Ok(())
}

fn fifo(path: &str, access: OFlags, blocking_after_open: bool) -> Result<File, ShimError> {
    let fd = open(
        path,
        access | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|_| ShimError::ChannelOpen)?;
    let stat = fstat(&fd).map_err(|_| ShimError::ChannelType)?;
    if !FileType::from_raw_mode(stat.st_mode).is_fifo() {
        return Err(ShimError::ChannelType);
    }
    if blocking_after_open {
        let flags = fcntl_getfl(&fd).map_err(|_| ShimError::ChannelFlags)?;
        fcntl_setfl(&fd, flags - OFlags::NONBLOCK).map_err(|_| ShimError::ChannelFlags)?;
    }
    Ok(File::from(fd))
}

fn write_all_nonblocking(file: &File, bytes: &[u8]) -> Result<(), ShimError> {
    let mut offset = 0;
    while offset < bytes.len() {
        let written = write(file, &bytes[offset..]).map_err(|_| ShimError::ReadyWrite)?;
        if written == 0 {
            return Err(ShimError::ReadyWrite);
        }
        offset += written;
    }
    Ok(())
}

fn await_release(
    ready: &File,
    release: &File,
    frame: &[u8],
    expected: u8,
) -> Result<(), ShimError> {
    write_all_nonblocking(ready, frame)?;

    let deadline = Instant::now() + READY_TIMEOUT;
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(ShimError::ReleaseTimeout)?;
        let timeout = Timespec {
            tv_sec: i64::try_from(remaining.as_secs()).unwrap_or(i64::MAX),
            tv_nsec: i64::from(remaining.subsec_nanos()),
        };
        let mut descriptor = [PollFd::new(release, PollFlags::IN)];
        match poll(&mut descriptor, Some(&timeout)) {
            Ok(0) => return Err(ShimError::ReleaseTimeout),
            Ok(_) => {
                let mut bytes = [0_u8; 2];
                let count = read(release, &mut bytes).map_err(|_| ShimError::ReleaseRead)?;
                if count == 1 && bytes[0] == expected {
                    return Ok(());
                }
                return Err(ShimError::ReleaseValue);
            }
            Err(Errno::INTR) => {}
            Err(_) => return Err(ShimError::ReleaseRead),
        }
    }
}

fn command(input: &NamespaceExec<'_>, stdout: File, stderr: File) -> Command {
    let mut command = Command::new(input.executable);
    command
        .args(input.arguments)
        .current_dir(input.working_directory)
        .env_clear()
        .env("PATH", "/toolchain/bin")
        .env("HOME", "/work/home")
        .env("TMPDIR", "/tmp")
        .env("LANG", "C.UTF-8")
        .env("LC_ALL", "C.UTF-8")
        .env("TZ", "UTC")
        .env("CARGO_HOME", "/toolchain/cargo-home")
        .env("CARGO_TARGET_DIR", "/work/target")
        .env("CARGO_BUILD_JOBS", "2")
        .env(
            "JULIA_DEPOT_PATH",
            "/work/julia-depot:/toolchain/julia-depot",
        )
        .env("JULIA_NUM_THREADS", "1")
        .env("OPENBLAS_NUM_THREADS", "1")
        .env("OMP_NUM_THREADS", "1")
        .env("RUST_BACKTRACE", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    command
}

/// Validate and open the collector FIFOs, complete both startup gates, then
/// spawn and wait for exactly one trusted-owner-selected command.
///
/// The caller must emit the returned static diagnostic and exit 127 on error.
/// # Errors
/// Refuses invalid input, FIFO readiness/release failure, deadline or failed exec.
pub fn exec(input: &NamespaceExec<'_>) -> Result<ExitCode, ShimError> {
    validate(input)?;
    let stdout = fifo(STDOUT_FIFO, OFlags::WRONLY, true)?;
    let stderr = fifo(STDERR_FIFO, OFlags::WRONLY, true)?;
    let ready = fifo(READY_FIFO, OFlags::WRONLY, false)?;
    let release = fifo(RELEASE_FIFO, OFlags::RDONLY, false)?;
    let status = fifo(STATUS_FIFO, OFlags::WRONLY, false)?;
    let frame = format!("{READY_PREFIX}{}\n", std::process::id());
    await_release(&ready, &release, frame.as_bytes(), RELEASE_BYTE)?;
    set_dumpable_behavior(DumpableBehavior::NotDumpable).map_err(|_| ShimError::Protection)?;
    if dumpable_behavior().map_err(|_| ShimError::Protection)? != DumpableBehavior::NotDumpable {
        return Err(ShimError::Protection);
    }
    setrlimit(
        Resource::Core,
        Rlimit {
            current: Some(0),
            maximum: Some(0),
        },
    )
    .map_err(|_| ShimError::Protection)?;
    let core_limit = getrlimit(Resource::Core);
    if core_limit.current != Some(0) || core_limit.maximum != Some(0) {
        return Err(ShimError::Protection);
    }
    await_release(
        &ready,
        &release,
        b"HEE3_NAMESPACE_PROTECTED_V1 dumpable=0 core_limit=0\n",
        PROTECTED_RELEASE_BYTE,
    )?;
    drop(ready);
    drop(release);
    match fs::symlink_metadata(STATUS_FIFO) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        _ => return Err(ShimError::Protection),
    }
    let mut child = command(input, stdout, stderr)
        .spawn()
        .map_err(|_| ShimError::Exec)?;
    let observed = child.wait().map_err(|_| ShimError::Wait)?;
    let normal_code = observed.code();
    let signal = observed.signal();
    let dumped = observed.core_dumped();
    let raw = observed.into_raw();
    let (frame, code) = if let Some(code) = normal_code {
        (
            format!("HEE3_NATIVE_STATUS_V1 kind=exit code={code} raw={raw} core=false\n"),
            code,
        )
    } else if let Some(signal) = signal {
        (
            format!("HEE3_NATIVE_STATUS_V1 kind=signal signal={signal} raw={raw} core={dumped}\n"),
            128 + signal,
        )
    } else {
        return Err(ShimError::Status);
    };
    if frame.len() > 128 {
        return Err(ShimError::Status);
    }
    write_all_nonblocking(&status, frame.as_bytes()).map_err(|_| ShimError::Status)?;
    drop(status);
    Ok(ExitCode::from(
        u8::try_from(code).map_err(|_| ShimError::Status)?,
    ))
}

/// Application-subcommand boundary with a decisive, static setup failure.
#[must_use]
pub fn run(input: &NamespaceExec<'_>) -> ExitCode {
    match exec(input) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{}", error.diagnostic());
            ExitCode::from(127)
        }
    }
}
