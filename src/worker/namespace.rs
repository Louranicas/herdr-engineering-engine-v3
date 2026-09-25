//! Constructive Bubblewrap custody for the cooperative T06 TH-DEV profile.
//! Candidate bytes are observations only and never configure this controller.

use super::process::{
    Interruption, Observer, ObserverDecision, ProcessObservation, ProcessReport, ProcessSpec,
    Refusal, Stream, run_observed,
};
use super::{resources, workspace};
use rustix::{
    event::{PollFd, PollFlags, poll},
    fd::OwnedFd,
    fs::{AtFlags, FileType, Mode, OFlags, fstat, mkfifoat, open, openat, statat, unlinkat},
    process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal},
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs::{self, File},
    io::{Read, Write},
    os::{
        unix::ffi::OsStrExt,
        unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

const BWRAP_SHA256: [u8; 32] = [
    0x6d, 0xa0, 0x6f, 0x15, 0x2b, 0x08, 0x65, 0x17, 0x2d, 0x73, 0x34, 0x8c, 0x34, 0xcb, 0x88, 0x48,
    0x7c, 0x32, 0x6c, 0xe2, 0xf2, 0x1c, 0xd9, 0x80, 0xfc, 0x25, 0xff, 0x10, 0xc4, 0xdb, 0xcd, 0xfb,
];
/// Where the shim is always mounted.
pub const SHIM_DESTINATION: &str = "/shim/namespace-shim";
/// The most read-only files (and, separately, namespace directories) one plan may mount.
pub const MAX_MOUNTS: usize = 512;
const MAX_PUBLIC: usize = MAX_MOUNTS + 2;
const MAX_PROTECTED: usize = 32;
const MAX_CHANNEL: usize = 8 * 1024 * 1024;
const MAX_STATUS: usize = 16 * 1024;
const MAX_SOURCE_BYTES: u64 = 1024 * 1024 * 1024;
const READY: &[u8] = b"HEE3_NAMESPACE_READY_V1 pid=2\n";

#[derive(Clone, Debug)]
pub struct ReadOnlyFile {
    pub host: PathBuf,
    pub namespace: PathBuf,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct PublicFile {
    pub namespace: PathBuf,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct BwrapPlan {
    pub bwrap: PathBuf,
    pub shim: ReadOnlyFile,
    pub candidate: ReadOnlyFile,
    pub read_only_files: Vec<ReadOnlyFile>,
    pub namespace_directories: Vec<PathBuf>,
    pub work_host: PathBuf,
    pub channels_host: PathBuf,
    pub public_files: Vec<PublicFile>,
    pub protected_paths: Vec<PathBuf>,
    pub shim_arguments: Vec<OsString>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamespaceError {
    InvalidPlan,
    Cancelled,
    Deadline,
    Bound,
    Io,
    Digest,
    Channel,
    Status,
    Pid,
    Namespace,
    Child,
    Root,
    Descriptor,
    Stream,
    Resources,
    Scratch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeStatus {
    Exit { code: u8, raw: i32 },
    Signal { signal: u8, raw: i32, core: bool },
}

#[derive(Debug, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent receipt predicates must remain separately observable"
)]
pub struct NamespaceFacts {
    pub adversarial_isolation: bool,
    pub pid1: Option<u32>,
    pub shim_pid: Option<u32>,
    pub child_snapshot: Vec<u32>,
    pub initial_namespace_ids: BTreeMap<String, u64>,
    pub source_sha256_before: BTreeMap<PathBuf, [u8; 32]>,
    pub source_sha256_after: BTreeMap<PathBuf, [u8; 32]>,
    pub source_sha256_postrun_host: BTreeMap<PathBuf, [u8; 32]>,
    pub before_mount_sha256: Option<[u8; 32]>,
    pub after_mount_sha256: Option<[u8; 32]>,
    pub root_device_inode: Option<(u64, u64)>,
    pub public_files_verified: usize,
    pub protected_paths_absent: usize,
    pub descriptors_verified: bool,
    pub protection_verified: bool,
    pub release_sent: bool,
    pub terminal_json: bool,
    pub json_exit_code: Option<u8>,
    pub native_status: Option<NativeStatus>,
    pub native_status_eof: bool,
    pub native_status_consistent: bool,
    pub pid1_terminal: bool,
    pub shim_terminal: bool,
    pub channel_cleanup_complete: bool,
    pub namespace_term_sent: bool,
    pub namespace_kill_sent: bool,
    pub unresolved: Vec<&'static str>,
}

pub struct PreparedNamespace {
    spec: ProcessSpec,
    observer: NamespaceObserver,
}

#[derive(Debug)]
pub struct PrepareFailure {
    pub error: NamespaceError,
    pub partial_channels: Option<PathBuf>,
    pub cleanup_complete: bool,
}

pub struct NamespaceReport {
    pub process: ProcessReport,
    pub observer: NamespaceObserver,
    pub json_exit_matches: bool,
    pub postrun_sources_match: bool,
}

#[derive(Debug)]
pub struct ScratchFacts {
    pub device: u64,
    pub inode: u64,
    pub capacity_bytes: u64,
    pub mountinfo: Vec<u8>,
}

impl NamespaceReport {
    /// Export only after the process owner and both namespace pidfds report termination.
    /// The returned copy remains untrusted candidate data; the retained mount is separate custody.
    /// # Errors
    /// Refuses missing scratch, pending writers or unsafe/overbound export.
    pub fn export_scratch(
        &self,
        parent: &Path,
        name: &str,
        protected: &[workspace::FileIdentity],
        deadline: Instant,
    ) -> Result<workspace::Materialized, workspace::MaterializeError> {
        let root = self
            .observer
            .scratch
            .as_ref()
            .filter(|_| self.writers_terminal())
            .ok_or(workspace::MaterializeError {
                error: workspace::Error::Custody,
                partial_path: None,
            })?;
        workspace::export_directory(root, parent, name, protected, deadline)
    }

    /// Explicitly release the retained tmpfs after export/error retention is complete.
    #[must_use]
    pub fn release_scratch(&mut self) -> bool {
        if !self.writers_terminal() {
            return false;
        }
        self.observer.scratch.take();
        true
    }

    fn writers_terminal(&self) -> bool {
        self.process.leader_reaped
            && self.process.process_group_settled
            && self.process.pending.is_none()
            && self.observer.facts.pid1_terminal
            && self.observer.facts.shim_terminal
    }

    pub fn cleanup_channels(&mut self) -> bool {
        if !self.process.leader_reaped
            || !self.process.process_group_settled
            || self.process.pending.is_some()
            || !self.process.stdout.eof
            || !self.process.stderr.eof
        {
            return false;
        }
        self.observer.cleanup_channels()
    }
}

#[derive(Debug)]
pub enum NamespaceRunError {
    Prepare(PrepareFailure),
    Process {
        refusal: Refusal,
        channel_cleanup_complete: bool,
    },
}

impl PreparedNamespace {
    #[must_use]
    pub fn process_spec(&self) -> &ProcessSpec {
        &self.spec
    }
    pub fn observer(&mut self) -> &mut NamespaceObserver {
        &mut self.observer
    }
    #[must_use]
    pub fn into_parts(self) -> (ProcessSpec, NamespaceObserver) {
        (self.spec, self.observer)
    }
}

/// First nonempty bounded stdout drain, sampled by the trusted owner after that drain.
/// It records receipt of bytes; it is not a producer write or flush timestamp.
#[derive(Clone, Copy, Debug)]
pub struct FirstStdoutObservation {
    pub observed_at: Instant,
    pub observed_bytes: u64,
    pub retained_bytes: usize,
}
impl FirstStdoutObservation {
    fn capture(slot: &mut Option<Self>, stream: &Stream) {
        if slot.is_none() && stream.observed_bytes != 0 {
            *slot = Some(Self {
                observed_at: Instant::now(),
                observed_bytes: stream.observed_bytes,
                retained_bytes: stream.bytes.len(),
            });
        }
    }
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "independent custody predicates remain separately observable"
)]
pub struct NamespaceObserver {
    plan: BwrapPlan,
    stdout: File,
    stderr: File,
    ready: File,
    native: File,
    channel_directory: File,
    channel_directory_identity: FileIdentity,
    pub native_stream: Stream,
    release: Option<File>,
    pub candidate_stdout: Stream,
    pub first_stdout_observation: Option<FirstStdoutObservation>,
    pub candidate_stderr: Stream,
    pub facts: NamespaceFacts,
    status_used: usize,
    initial_seen: bool,
    ready_bytes: Vec<u8>,
    ready_eof: bool,
    pid1_fd: Option<OwnedFd>,
    shim_fd: Option<OwnedFd>,
    failed: Option<NamespaceError>,
    channel_identities: BTreeMap<&'static str, FileIdentity>,
    structural_verified: bool,
    readback_index: usize,
    readback_file: Option<(File, Sha256, u64)>,
    readback_complete: bool,
    resource_plan: Option<resources::Scope>,
    resource_scope: Option<resources::VerifiedScope>,
    scratch: Option<File>,
    scratch_facts: Option<ScratchFacts>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

fn clean_absolute(path: &Path) -> bool {
    path.is_absolute()
        && path.as_os_str().as_bytes().len() <= 4096
        && !path.as_os_str().as_bytes().contains(&0)
        && path
            .components()
            .all(|part| !matches!(part, std::path::Component::ParentDir))
}

fn sha256(path: &Path, deadline: Instant) -> Result<[u8; 32], NamespaceError> {
    let fd = open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
    )
    .map_err(|_| NamespaceError::Digest)?;
    let before = fstat(&fd).map_err(|_| NamespaceError::Digest)?;
    if !FileType::from_raw_mode(before.st_mode).is_file()
        || before.st_size < 0
        || before.st_size.cast_unsigned() > 256 * 1024 * 1024
    {
        return Err(NamespaceError::Digest);
    }
    let mut file = File::from(fd);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 8192];
    let mut observed = 0_u64;
    loop {
        if Instant::now() >= deadline {
            return Err(NamespaceError::Deadline);
        }
        let count = file.read(&mut buffer).map_err(|_| NamespaceError::Digest)?;
        if count == 0 {
            break;
        }
        observed = observed
            .checked_add(count as u64)
            .filter(|total| *total <= 256 * 1024 * 1024)
            .ok_or(NamespaceError::Bound)?;
        digest.update(&buffer[..count]);
    }
    let after = fstat(&file).map_err(|_| NamespaceError::Digest)?;
    if observed != before.st_size.cast_unsigned()
        || (
            before.st_dev,
            before.st_ino,
            before.st_size,
            before.st_mtime,
            before.st_mtime_nsec,
            before.st_ctime,
            before.st_ctime_nsec,
        ) != (
            after.st_dev,
            after.st_ino,
            after.st_size,
            after.st_mtime,
            after.st_mtime_nsec,
            after.st_ctime,
            after.st_ctime_nsec,
        )
    {
        return Err(NamespaceError::Digest);
    }
    Ok(digest.finalize().into())
}

fn read_bounded(path: &Path, cap: usize, deadline: Instant) -> Result<Vec<u8>, NamespaceError> {
    let mut file = File::open(path).map_err(|_| NamespaceError::Io)?;
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        if Instant::now() >= deadline {
            return Err(NamespaceError::Deadline);
        }
        let count = file.read(&mut buffer).map_err(|_| NamespaceError::Io)?;
        if count == 0 {
            break;
        }
        if bytes
            .len()
            .checked_add(count)
            .as_ref()
            .is_none_or(|total| *total > cap)
        {
            return Err(NamespaceError::Bound);
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    Ok(bytes)
}

/// Whether `path` is a clean absolute path (no `..`, no NUL, at most 4096 bytes): the shape every
/// host path a plan names must have. Hash-free, so a declaration can be checked before any use
/// (B14-P2b) by the same rule a plan is.
#[must_use]
pub fn host_shape(path: &Path) -> bool {
    clean_absolute(path)
}

/// Whether a read-only file may be mounted at `namespace`: clean, absolute and under one of the
/// plan's file prefixes. Hash-free: the one shape rule for a file destination, shared by
/// [`validate_file`] and the class profile's declaration check (B14-P2b), so the two cannot differ.
#[must_use]
pub fn file_shape(namespace: &Path) -> bool {
    clean_absolute(namespace)
        && [
            "/toolchain",
            "/frozen",
            "/shim",
            "/work/bin",
            "/lib64/",
            "/usr/bin/",
            "/usr/lib64/",
            "/usr/lib/gcc/",
            "/usr/libexec/gcc/",
        ]
        .iter()
        .any(|prefix| namespace.starts_with(prefix))
}

/// Whether a plan may create the namespace directory `path`: clean, absolute and one of the fixed
/// runtime directories or beneath one. Hash-free, shared with the class profile (B14-P2b).
#[must_use]
pub fn directory_shape(path: &Path) -> bool {
    clean_absolute(path)
        && [
            "/shim",
            "/work",
            "/tmp",
            "/toolchain",
            "/frozen",
            "/channels",
            "/lib64",
            "/usr",
            "/usr/bin",
            "/usr/lib",
            "/usr/lib64",
            "/usr/lib/gcc",
            "/usr/libexec",
            "/usr/libexec/gcc",
        ]
        .iter()
        .any(|prefix| path == Path::new(prefix) || path.starts_with(format!("{prefix}/")))
}

fn validate_file(file: &ReadOnlyFile, deadline: Instant) -> Result<(), NamespaceError> {
    if !clean_absolute(&file.host)
        || !file_shape(&file.namespace)
        || sha256(&file.host, deadline)? != file.sha256
    {
        return Err(NamespaceError::InvalidPlan);
    }
    Ok(())
}

fn validate(plan: &BwrapPlan, deadline: Instant) -> Result<(), NamespaceError> {
    if Instant::now() >= deadline {
        return Err(NamespaceError::Deadline);
    }
    if plan.bwrap != Path::new("/usr/bin/bwrap")
        || sha256(&plan.bwrap, deadline)? != BWRAP_SHA256
        || plan.read_only_files.len() > MAX_MOUNTS
        || plan.namespace_directories.len() > MAX_MOUNTS
        || plan.public_files.len() > MAX_PUBLIC
        || plan.protected_paths.len() > MAX_PROTECTED
        || !clean_absolute(&plan.work_host)
        || !clean_absolute(&plan.channels_host)
        || plan.channels_host.exists()
        || !plan.work_host.is_dir()
    {
        return Err(NamespaceError::InvalidPlan);
    }
    validate_file(&plan.shim, deadline)?;
    validate_file(&plan.candidate, deadline)?;
    if plan.shim.namespace != Path::new(SHIM_DESTINATION)
        || !(plan.candidate.namespace.starts_with("/frozen/bin/")
            || plan.candidate.namespace.starts_with("/work/bin/")
            || plan.candidate.namespace.starts_with("/toolchain/bin/"))
    {
        return Err(NamespaceError::InvalidPlan);
    }
    let mut destinations = BTreeSet::new();
    let mut source_bytes = 0_u64;
    for file in plan
        .read_only_files
        .iter()
        .chain([&plan.shim, &plan.candidate])
    {
        validate_file(file, deadline)?;
        source_bytes = source_bytes
            .checked_add(
                fs::symlink_metadata(&file.host)
                    .map_err(|_| NamespaceError::InvalidPlan)?
                    .len(),
            )
            .filter(|total| *total <= MAX_SOURCE_BYTES)
            .ok_or(NamespaceError::Bound)?;
        if !destinations.insert(file.namespace.clone()) {
            return Err(NamespaceError::InvalidPlan);
        }
    }
    let public: BTreeSet<_> = plan
        .public_files
        .iter()
        .map(|file| file.namespace.clone())
        .collect();
    if public.len() != plan.public_files.len() || public != destinations {
        return Err(NamespaceError::InvalidPlan);
    }
    for public in &plan.public_files {
        let expected = plan
            .read_only_files
            .iter()
            .chain([&plan.shim, &plan.candidate])
            .find(|file| file.namespace == public.namespace)
            .ok_or(NamespaceError::InvalidPlan)?;
        if expected.sha256 != public.sha256 {
            return Err(NamespaceError::InvalidPlan);
        }
    }
    if plan
        .namespace_directories
        .iter()
        .any(|p| !directory_shape(p))
        || plan
            .public_files
            .iter()
            .any(|p| !clean_absolute(&p.namespace))
        || plan.protected_paths.iter().any(|p| !clean_absolute(p))
        || plan.shim_arguments.len() > 253
        || plan
            .shim_arguments
            .iter()
            .any(|value| value.as_bytes().len() > 4096 || value.as_bytes().contains(&0))
        || plan
            .shim_arguments
            .iter()
            .map(|value| value.as_bytes().len())
            .sum::<usize>()
            > 60_000
    {
        return Err(NamespaceError::InvalidPlan);
    }
    Ok(())
}

#[expect(
    clippy::type_complexity,
    reason = "five distinct FIFO handles and their identity receipt are returned together"
)]
fn make_channels(
    root: &Path,
) -> Result<
    (
        File,
        File,
        File,
        File,
        File,
        BTreeMap<&'static str, FileIdentity>,
        File,
    ),
    NamespaceError,
> {
    fs::create_dir(root).map_err(|_| NamespaceError::Channel)?;
    (|| {
        fs::set_permissions(root, fs::Permissions::from_mode(0o700))
            .map_err(|_| NamespaceError::Channel)?;
        let directory = open(
            root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| NamespaceError::Channel)?;
        for name in ["stdout", "stderr", "ready", "release", "status"] {
            mkfifoat(&directory, name, Mode::RUSR | Mode::WUSR)
                .map_err(|_| NamespaceError::Channel)?;
        }
        let open_fifo = |name: &str, flags| -> Result<File, NamespaceError> {
            let fd = openat(
                &directory,
                name,
                flags | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|_| NamespaceError::Channel)?;
            if !FileType::from_raw_mode(fstat(&fd).map_err(|_| NamespaceError::Channel)?.st_mode)
                .is_fifo()
            {
                return Err(NamespaceError::Channel);
            }
            Ok(File::from(fd))
        };
        let stdout = open_fifo("stdout", OFlags::RDONLY)?;
        let stderr = open_fifo("stderr", OFlags::RDONLY)?;
        let ready = open_fifo("ready", OFlags::RDONLY)?;
        let release = open_fifo("release", OFlags::RDWR)?;
        let native = open_fifo("status", OFlags::RDONLY)?;
        let mut identities = BTreeMap::new();
        for (name, file) in [
            ("stdout", &stdout),
            ("stderr", &stderr),
            ("ready", &ready),
            ("release", &release),
            ("status", &native),
        ] {
            let stat = fstat(file).map_err(|_| NamespaceError::Channel)?;
            identities.insert(
                name,
                FileIdentity {
                    device: stat.st_dev,
                    inode: stat.st_ino,
                },
            );
        }
        Ok((
            stdout,
            stderr,
            ready,
            release,
            native,
            identities,
            File::from(directory),
        ))
    })()
}

fn remove_channels(root: &Path) -> bool {
    let mut complete = true;
    for name in ["stdout", "stderr", "ready", "release", "status"] {
        if let Err(error) = fs::remove_file(root.join(name))
            && error.kind() != std::io::ErrorKind::NotFound
        {
            complete = false;
        }
    }
    if let Err(error) = fs::remove_dir(root)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        complete = false;
    }
    complete
}

/// Validate a pinned plan and create its private bounded channels.
///
/// # Errors
/// Refuses invalid inputs, hashes, paths, budgets or channel creation; partial custody is retained.
pub fn prepare(plan: BwrapPlan, deadline: Instant) -> Result<PreparedNamespace, PrepareFailure> {
    prepare_inner(plan, None, deadline)
}

/// Prepare the fixed RC01 scope and one 4-GiB scratch filesystem.
/// `work_host` is validated but never mounted by this profile. All inputs must be
/// finite read-only files outside /work; the new scratch starts with empty runtime dirs.
/// # Errors
/// Refuses the ordinary plan failures, unsafe scratch mounts or scope identity failure.
pub fn prepare_bounded(
    plan: BwrapPlan,
    scope: resources::Scope,
    deadline: Instant,
) -> Result<PreparedNamespace, PrepareFailure> {
    prepare_inner(plan, Some(scope), deadline)
}

#[expect(
    clippy::too_many_lines,
    reason = "one atomic namespace plan materialization"
)]
fn prepare_inner(
    plan: BwrapPlan,
    scope: Option<resources::Scope>,
    deadline: Instant,
) -> Result<PreparedNamespace, PrepareFailure> {
    validate(&plan, deadline).map_err(|error| PrepareFailure {
        error,
        partial_channels: None,
        cleanup_complete: true,
    })?;
    if scope.is_some()
        && (plan
            .public_files
            .iter()
            .any(|file| file.namespace.starts_with("/work"))
            || plan
                .namespace_directories
                .iter()
                .any(|path| path.starts_with("/work") && path != Path::new("/work")))
    {
        return Err(PrepareFailure {
            error: NamespaceError::InvalidPlan,
            partial_channels: None,
            cleanup_complete: true,
        });
    }
    let source_sha256_before = plan
        .read_only_files
        .iter()
        .chain([&plan.shim, &plan.candidate])
        .map(|file| (file.namespace.clone(), file.sha256))
        .collect();
    let mut arguments: Vec<OsString> = [
        "--unshare-all",
        "--unshare-user",
        "--die-with-parent",
        "--new-session",
        "--clearenv",
        "--cap-drop",
        "ALL",
        "--disable-userns",
        "--json-status-fd",
        "1",
        "--tmpfs",
        "/",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    if scope.is_some() {
        arguments.extend(
            [
                "--size",
                "4294967296",
                "--perms",
                "0700",
                "--tmpfs",
                "/work",
            ]
            .map(Into::into),
        );
        for name in ["home", "target", "julia-depot", "tmp"] {
            arguments.extend([
                "--perms".into(),
                "0700".into(),
                "--dir".into(),
                format!("/work/{name}").into(),
            ]);
        }
    }
    for directory in &plan.namespace_directories {
        if scope.is_some() && (directory == Path::new("/work") || directory == Path::new("/tmp")) {
            continue;
        }
        arguments.push("--dir".into());
        arguments.push(directory.as_os_str().to_owned());
    }
    if scope.is_some() {
        arguments.extend(["--symlink", "/work/tmp", "/tmp"].map(Into::into));
    } else {
        arguments.extend([
            "--bind".into(),
            plan.work_host.as_os_str().to_owned(),
            "/work".into(),
        ]);
    }
    arguments.extend([
        "--ro-bind".into(),
        plan.channels_host.as_os_str().to_owned(),
        "/channels".into(),
    ]);
    for file in plan
        .read_only_files
        .iter()
        .chain([&plan.shim, &plan.candidate])
    {
        arguments.push("--ro-bind".into());
        arguments.push(file.host.as_os_str().to_owned());
        arguments.push(file.namespace.as_os_str().to_owned());
    }
    if scope.is_some() {
        arguments.extend(
            [
                "--remount-ro",
                "/dev",
                "--remount-ro",
                "/proc",
                "--remount-ro",
                "/",
            ]
            .map(Into::into),
        );
    }
    arguments.extend(["--chdir".into(), "/work".into()]);
    let mut encoded = Vec::new();
    for argument in &arguments {
        if argument.as_bytes().contains(&0) {
            return Err(PrepareFailure {
                error: NamespaceError::InvalidPlan,
                partial_channels: None,
                cleanup_complete: true,
            });
        }
        encoded.extend_from_slice(argument.as_bytes());
        encoded.push(0);
        if encoded.len() > 1024 * 1024 {
            return Err(PrepareFailure {
                error: NamespaceError::InvalidPlan,
                partial_channels: None,
                cleanup_complete: true,
            });
        }
    }
    let mut spec = ProcessSpec {
        executable: plan.bwrap.clone(),
        arguments: [
            "--args".into(),
            "0".into(),
            plan.shim.namespace.as_os_str().to_owned(),
        ]
        .into_iter()
        .chain(plan.shim_arguments.iter().cloned())
        .collect(),
        directory: "/".into(),
        environment: vec![],
        input: encoded,
        stream_limit: MAX_STATUS,
    };
    if let Some(scope) = &scope {
        spec = scope.wrap(spec, deadline).map_err(|_| PrepareFailure {
            error: NamespaceError::Resources,
            partial_channels: None,
            cleanup_complete: true,
        })?;
    }
    let (stdout, stderr, ready, release, native, channel_identities, channel_directory) =
        make_channels(&plan.channels_host).map_err(|error| {
            let exists = plan.channels_host.try_exists().unwrap_or(true);
            PrepareFailure {
                error,
                partial_channels: exists.then(|| plan.channels_host.clone()),
                cleanup_complete: !exists,
            }
        })?;
    let directory_stat = fstat(&channel_directory).map_err(|_| PrepareFailure {
        error: NamespaceError::Channel,
        partial_channels: Some(plan.channels_host.clone()),
        cleanup_complete: false,
    })?;
    let channel_directory_identity = FileIdentity {
        device: directory_stat.st_dev,
        inode: directory_stat.st_ino,
    };
    Ok(PreparedNamespace {
        spec,
        observer: NamespaceObserver {
            plan,
            stdout,
            stderr,
            ready,
            native,
            channel_directory,
            channel_directory_identity,
            native_stream: Stream::default(),
            release: Some(release),
            candidate_stdout: Stream::default(),
            first_stdout_observation: None,
            candidate_stderr: Stream::default(),
            facts: NamespaceFacts {
                source_sha256_before,
                ..NamespaceFacts::default()
            },
            status_used: 0,
            initial_seen: false,
            ready_bytes: vec![],
            ready_eof: false,
            pid1_fd: None,
            shim_fd: None,
            failed: None,
            channel_identities,
            structural_verified: false,
            readback_index: 0,
            readback_file: None,
            readback_complete: false,
            resource_plan: scope,
            resource_scope: None,
            scratch: None,
            scratch_facts: None,
        },
    })
}

/// Run exactly one prepared namespace through the generic bounded process owner.
/// The returned observer retains namespace pidfds and all constructive facts.
///
/// # Errors
/// Refuses plan preparation or process start and preserves channel cleanup state.
pub fn run_namespace(
    plan: BwrapPlan,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<NamespaceReport, NamespaceRunError> {
    run_prepared(
        prepare(plan, deadline).map_err(NamespaceRunError::Prepare)?,
        deadline,
        cancelled,
    )
}

/// Run the bounded profile through the same process and namespace owner.
/// # Errors
/// Preserves preparation/process refusal and any unresolved channel custody.
pub fn run_bounded_namespace(
    plan: BwrapPlan,
    scope: resources::Scope,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<NamespaceReport, NamespaceRunError> {
    run_prepared(
        prepare_bounded(plan, scope, deadline).map_err(NamespaceRunError::Prepare)?,
        deadline,
        cancelled,
    )
}

fn run_prepared(
    prepared: PreparedNamespace,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<NamespaceReport, NamespaceRunError> {
    let (spec, mut observer) = prepared.into_parts();
    let process = match run_observed(&spec, deadline, cancelled, &mut observer) {
        Ok(report) => report,
        Err(refusal) => {
            let channel_cleanup_complete =
                if observer.facts.pid1_terminal && observer.facts.shim_terminal {
                    observer.cleanup_channels()
                } else {
                    observer.cleanup_unspawned_channels()
                };
            return Err(NamespaceRunError::Process {
                refusal,
                channel_cleanup_complete,
            });
        }
    };
    let json_exit_matches = matches!(
        (observer.facts.json_exit_code.map(i32::from), process.exit_code),
        (Some(json), Some(process)) if json == process
    );
    if !json_exit_matches {
        observer
            .facts
            .unresolved
            .push("bwrap JSON/process exit mismatch");
    }
    // Cancellation and other interrupted outcomes cannot be accepted. Return physical
    // custody promptly instead of delaying the aggregate owner with closure hashing.
    // Pre-launch host hashes and in-namespace pre-release readback remain mandatory;
    // post-run host hashes are explicitly unavailable on this path. Uninterrupted
    // outcomes retain the exact full post-run hash loop below.
    let mut postrun_sources_match = should_hash_postrun(process.interruption);
    if should_hash_postrun(process.interruption) {
        for file in observer
            .plan
            .read_only_files
            .iter()
            .chain([&observer.plan.shim, &observer.plan.candidate])
        {
            match sha256(&file.host, deadline) {
                Ok(value) if value == file.sha256 => {
                    observer
                        .facts
                        .source_sha256_postrun_host
                        .insert(file.namespace.clone(), value);
                }
                _ => postrun_sources_match = false,
            }
        }
    } else {
        observer
            .facts
            .unresolved
            .push("post-run host source hashes unavailable after interruption");
    }
    if !postrun_sources_match && should_hash_postrun(process.interruption) {
        observer
            .facts
            .unresolved
            .push("post-run host source mismatch or deadline");
    }
    Ok(NamespaceReport {
        process,
        observer,
        json_exit_matches,
        postrun_sources_match,
    })
}

fn drain_limit(
    file: &mut File,
    stream: &mut Stream,
    allow_eof: bool,
    limit: usize,
) -> Result<(), NamespaceError> {
    let mut overflow = false;
    for _ in 0..16 {
        let mut bytes = [0_u8; 8192];
        match file.read(&mut bytes) {
            Ok(0) => {
                if allow_eof {
                    stream.eof = true;
                }
                return if overflow {
                    Err(NamespaceError::Stream)
                } else {
                    Ok(())
                };
            }
            Ok(n) => {
                stream.observed_bytes = stream.observed_bytes.saturating_add(n as u64);
                let retained = limit.saturating_sub(stream.bytes.len()).min(n);
                stream.bytes.extend_from_slice(&bytes[..retained]);
                if retained != n {
                    stream.truncated = true;
                    overflow = true;
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) =>
            {
                return if overflow {
                    Err(NamespaceError::Stream)
                } else {
                    Ok(())
                };
            }
            Err(_) => {
                stream.failed = true;
                return Err(NamespaceError::Stream);
            }
        }
    }
    if overflow {
        Err(NamespaceError::Stream)
    } else {
        Ok(())
    }
}

fn drain(file: &mut File, stream: &mut Stream, allow_eof: bool) -> Result<(), NamespaceError> {
    drain_limit(file, stream, allow_eof, MAX_CHANNEL)
}

fn status_object(line: &str) -> Result<BTreeMap<String, u64>, NamespaceError> {
    let body = line
        .trim()
        .strip_prefix('{')
        .and_then(|v| v.strip_suffix('}'))
        .ok_or(NamespaceError::Status)?;
    let mut values = BTreeMap::new();
    for field in body.split(',') {
        let (key, value) = field.split_once(':').ok_or(NamespaceError::Status)?;
        let key = key
            .trim()
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .ok_or(NamespaceError::Status)?;
        if key.is_empty()
            || !key.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
            || values
                .insert(
                    key.into(),
                    value.trim().parse().map_err(|_| NamespaceError::Status)?,
                )
                .is_some()
        {
            return Err(NamespaceError::Status);
        }
    }
    Ok(values)
}

fn pid(value: u64) -> Result<Pid, NamespaceError> {
    let raw = i32::try_from(value).map_err(|_| NamespaceError::Pid)?;
    Pid::from_raw(raw).ok_or(NamespaceError::Pid)
}

fn terminal(fd: &OwnedFd) -> bool {
    let mut pollfd = [PollFd::new(fd, PollFlags::IN)];
    poll(&mut pollfd, Some(&rustix::event::Timespec::default()))
        .is_ok_and(|n| n == 1 && pollfd[0].revents().contains(PollFlags::IN))
}

fn proc_link(pid: u32, name: &str) -> Result<String, NamespaceError> {
    fs::read_link(format!("/proc/{pid}/ns/{name}"))
        .map_err(|_| NamespaceError::Namespace)?
        .into_os_string()
        .into_string()
        .map_err(|_| NamespaceError::Namespace)
}

fn inode(link: &str) -> Option<u64> {
    link.split_once('[')?.1.strip_suffix(']')?.parse().ok()
}

fn terminal_status(bytes: &[u8]) -> Result<Option<u8>, NamespaceError> {
    if bytes.is_empty() {
        return Ok(None);
    }
    let Some(end) = bytes.iter().position(|b| *b == b'\n') else {
        return Ok(None);
    };
    if bytes[end + 1..].iter().any(|b| !b.is_ascii_whitespace()) {
        return Err(NamespaceError::Status);
    }
    let line = std::str::from_utf8(&bytes[..end]).map_err(|_| NamespaceError::Status)?;
    let values = status_object(line)?;
    if values.len() != 1 || !values.contains_key("exit-code") {
        return Err(NamespaceError::Status);
    }
    Ok(Some(
        u8::try_from(*values.get("exit-code").ok_or(NamespaceError::Status)?)
            .map_err(|_| NamespaceError::Status)?,
    ))
}

fn canonical_u8(value: &str) -> Result<u8, NamespaceError> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(NamespaceError::Status);
    }
    value.parse().map_err(|_| NamespaceError::Status)
}

fn canonical_i32(value: &str) -> Result<i32, NamespaceError> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(NamespaceError::Status);
    }
    value.parse().map_err(|_| NamespaceError::Status)
}

fn native_status(bytes: &[u8]) -> Result<Option<NativeStatus>, NamespaceError> {
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() > 128 || !bytes.ends_with(b"\n") || bytes[..bytes.len() - 1].contains(&b'\n') {
        return Err(NamespaceError::Status);
    }
    let line =
        std::str::from_utf8(&bytes[..bytes.len() - 1]).map_err(|_| NamespaceError::Status)?;
    let fields: Vec<_> = line.split(' ').collect();
    if fields.len() != 5 || fields[0] != "HEE3_NATIVE_STATUS_V1" {
        return Err(NamespaceError::Status);
    }
    let raw = canonical_i32(
        fields[3]
            .strip_prefix("raw=")
            .ok_or(NamespaceError::Status)?,
    )?;
    if fields[1] == "kind=exit" && fields[4] == "core=false" {
        let code = canonical_u8(
            fields[2]
                .strip_prefix("code=")
                .ok_or(NamespaceError::Status)?,
        )?;
        if raw != i32::from(code) << 8 {
            return Err(NamespaceError::Status);
        }
        return Ok(Some(NativeStatus::Exit { code, raw }));
    }
    if fields[1] == "kind=signal" {
        let signal = canonical_u8(
            fields[2]
                .strip_prefix("signal=")
                .ok_or(NamespaceError::Status)?,
        )?;
        let core = match fields[4].strip_prefix("core=") {
            Some("true") => true,
            Some("false") => false,
            _ => return Err(NamespaceError::Status),
        };
        if signal == 0 || signal > 64 || raw != i32::from(signal) | if core { 128 } else { 0 } {
            return Err(NamespaceError::Status);
        }
        return Ok(Some(NativeStatus::Signal { signal, raw, core }));
    }
    Err(NamespaceError::Status)
}

const fn status_consistent(native: NativeStatus, json: u8) -> bool {
    match native {
        NativeStatus::Exit { code, .. } => code == json,
        NativeStatus::Signal { signal, .. } => 128_u8.saturating_add(signal) == json,
    }
}

impl NamespaceObserver {
    #[must_use]
    pub const fn resource_scope(&self) -> Option<&resources::VerifiedScope> {
        self.resource_scope.as_ref()
    }

    #[must_use]
    pub const fn scratch_facts(&self) -> Option<&ScratchFacts> {
        self.scratch_facts.as_ref()
    }

    #[must_use]
    pub const fn scratch_retained(&self) -> bool {
        self.scratch.is_some()
    }

    fn inspect_resources(
        &mut self,
        observation: &ProcessObservation<'_>,
    ) -> Result<(), NamespaceError> {
        let Some(scope) = &self.resource_plan else {
            return Ok(());
        };
        if observation.leader_terminal {
            return Err(NamespaceError::Resources);
        }
        let verified = scope
            .observe(observation.leader_pid, observation.work_deadline)
            .map_err(|_| NamespaceError::Resources)?;
        let pid1 = self.facts.pid1.ok_or(NamespaceError::Pid)?;
        let root = PathBuf::from(format!("/proc/{pid1}/root"));
        let scratch = File::from(
            open(
                root.join("work"),
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|_| NamespaceError::Scratch)?,
        );
        let metadata = scratch.metadata().map_err(|_| NamespaceError::Scratch)?;
        let filesystem = rustix::fs::fstatfs(&scratch).map_err(|_| NamespaceError::Scratch)?;
        let capacity = rustix::fs::fstatvfs(&scratch).map_err(|_| NamespaceError::Scratch)?;
        let capacity_bytes = capacity
            .f_blocks
            .checked_mul(capacity.f_frsize)
            .ok_or(NamespaceError::Scratch)?;
        if metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o7777 != 0o700
            || filesystem.f_type != 0x0102_1994
            || capacity_bytes != 4_294_967_296
        {
            return Err(NamespaceError::Scratch);
        }
        let mountinfo = read_bounded(
            Path::new(&format!("/proc/{pid1}/mountinfo")),
            1024 * 1024,
            observation.work_deadline,
        )?;
        check_scratch_mounts(&mountinfo)?;
        self.scratch_facts = Some(ScratchFacts {
            device: metadata.dev(),
            inode: metadata.ino(),
            capacity_bytes,
            mountinfo,
        });
        self.scratch = Some(scratch);
        self.resource_scope = Some(verified);
        Ok(())
    }

    #[must_use]
    pub const fn failure(&self) -> Option<NamespaceError> {
        self.failed
    }

    fn stop_namespace(&mut self, observation: &ProcessObservation<'_>) {
        let stopping = observation.stopping_since.is_some()
            || observation.interruption.is_some()
            || observation.now >= observation.work_deadline;
        if !stopping {
            return;
        }
        let reason = match observation.interruption {
            Some(Interruption::Cancelled) => NamespaceError::Cancelled,
            Some(Interruption::OutputLimit | Interruption::PipeError) => NamespaceError::Stream,
            Some(Interruption::Timeout) | None => NamespaceError::Deadline,
            Some(
                Interruption::WaitError | Interruption::ResidualGroup | Interruption::ObserverError,
            ) => NamespaceError::Child,
        };
        self.failed.get_or_insert(reason);
        self.release.take();
        if !self.facts.namespace_term_sent {
            let shim = self
                .shim_fd
                .as_ref()
                .is_some_and(|fd| pidfd_send_signal(fd, Signal::TERM).is_ok());
            let pid1 = self
                .pid1_fd
                .as_ref()
                .is_some_and(|fd| pidfd_send_signal(fd, Signal::TERM).is_ok());
            self.facts.namespace_term_sent = shim || pid1;
        }
        let kill = observation.stopping_since.is_some_and(|start| {
            observation.now.saturating_duration_since(start) >= Duration::from_secs(5)
        }) || observation
            .cleanup_deadline
            .is_some_and(|deadline| observation.now >= deadline);
        if kill && !self.facts.namespace_kill_sent {
            let shim = self
                .shim_fd
                .as_ref()
                .is_some_and(|fd| pidfd_send_signal(fd, Signal::KILL).is_ok());
            let pid1 = self
                .pid1_fd
                .as_ref()
                .is_some_and(|fd| pidfd_send_signal(fd, Signal::KILL).is_ok());
            self.facts.namespace_kill_sent = shim || pid1;
        }
    }

    /// Remove only the four controller-created FIFO names and their directory.
    /// Refuses while either retained namespace process may still be live.
    fn cleanup_channels(&mut self) -> bool {
        if !self.facts.pid1_terminal || !self.facts.shim_terminal || self.release.is_some() {
            return false;
        }
        if !self.candidate_stdout.eof
            || !self.candidate_stderr.eof
            || !self.ready_eof
            || !self.facts.native_status_eof
        {
            return false;
        }
        if !self.channels_owned_after_run() {
            return false;
        }
        let complete = remove_channels(&self.plan.channels_host);
        self.facts.channel_cleanup_complete = complete;
        complete
    }

    fn initial(&mut self, bytes: &[u8], deadline: Instant) -> Result<(), NamespaceError> {
        if bytes.len() > MAX_STATUS {
            return Err(NamespaceError::Status);
        }
        let Some(end) = bytes.iter().position(|b| *b == b'\n') else {
            return Ok(());
        };
        let line = std::str::from_utf8(&bytes[..end]).map_err(|_| NamespaceError::Status)?;
        let values = status_object(line)?;
        let expected: BTreeSet<_> = [
            "child-pid",
            "cgroup-namespace",
            "ipc-namespace",
            "mnt-namespace",
            "net-namespace",
            "pid-namespace",
            "uts-namespace",
        ]
        .into_iter()
        .collect();
        if values.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected {
            return Err(NamespaceError::Status);
        }
        let child = *values.get("child-pid").ok_or(NamespaceError::Status)?;
        let child_u32 = u32::try_from(child).map_err(|_| NamespaceError::Pid)?;
        for (json, name) in [
            ("\"pid-namespace\"", "pid"),
            ("\"mnt-namespace\"", "mnt"),
            ("\"user-namespace\"", "user"),
            ("\"net-namespace\"", "net"),
        ] {
            if let Some(expected) = values.get(json.trim_matches('"')).copied() {
                let actual =
                    inode(&proc_link(child_u32, name)?).ok_or(NamespaceError::Namespace)?;
                let host = inode(&proc_link(std::process::id(), name)?)
                    .ok_or(NamespaceError::Namespace)?;
                if expected != actual || actual == host {
                    return Err(NamespaceError::Namespace);
                }
                self.facts.initial_namespace_ids.insert(name.into(), actual);
            } else if name != "user" {
                return Err(NamespaceError::Status);
            }
        }
        let user = inode(&proc_link(child_u32, "user")?).ok_or(NamespaceError::Namespace)?;
        let host_user =
            inode(&proc_link(std::process::id(), "user")?).ok_or(NamespaceError::Namespace)?;
        if user == host_user {
            return Err(NamespaceError::Namespace);
        }
        self.facts.initial_namespace_ids.insert("user".into(), user);
        let mount = read_bounded(
            Path::new(&format!("/proc/{child_u32}/mountinfo")),
            1024 * 1024,
            deadline,
        )?;
        self.facts.before_mount_sha256 = Some(Sha256::digest(&mount).into());
        self.pid1_fd =
            Some(pidfd_open(pid(child)?, PidfdFlags::empty()).map_err(|_| NamespaceError::Pid)?);
        self.facts.pid1 = Some(child_u32);
        self.status_used = end + 1;
        self.initial_seen = true;
        Ok(())
    }

    #[expect(clippy::too_many_lines, reason = "one atomic pre-release custody gate")]
    fn inspect_ready(&mut self, deadline: Instant) -> Result<bool, NamespaceError> {
        if self.ready_bytes != READY {
            return Err(NamespaceError::Status);
        }
        let pid1 = self.facts.pid1.ok_or(NamespaceError::Pid)?;
        let children_bytes = read_bounded(
            Path::new(&format!("/proc/{pid1}/task/{pid1}/children")),
            4096,
            deadline,
        )?;
        let children = std::str::from_utf8(&children_bytes).map_err(|_| NamespaceError::Child)?;
        let parsed: Vec<u32> = children
            .split_ascii_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()
            .map_err(|_| NamespaceError::Child)?;
        self.facts.child_snapshot.clone_from(&parsed);
        let Some(shim) = parsed.first().copied() else {
            return Ok(false);
        };
        if parsed.len() != 1 {
            return Err(NamespaceError::Child);
        }
        self.shim_fd = Some(
            pidfd_open(pid(u64::from(shim))?, PidfdFlags::empty())
                .map_err(|_| NamespaceError::Pid)?,
        );
        self.facts.shim_pid = Some(shim);
        let status_bytes = read_bounded(
            Path::new(&format!("/proc/{shim}/status")),
            64 * 1024,
            deadline,
        )?;
        let status = std::str::from_utf8(&status_bytes).map_err(|_| NamespaceError::Pid)?;
        if !status.lines().any(|line| {
            line.strip_prefix("NSpid:")
                .and_then(|values| values.split_ascii_whitespace().last())
                == Some("2")
        }) {
            return Err(NamespaceError::Pid);
        }
        let mount = read_bounded(
            Path::new(&format!("/proc/{shim}/mountinfo")),
            1024 * 1024,
            deadline,
        )?;
        self.facts.after_mount_sha256 = Some(Sha256::digest(&mount).into());
        let root = PathBuf::from(format!("/proc/{shim}/root"));
        let metadata = fs::metadata(&root).map_err(|_| NamespaceError::Root)?;
        self.facts.root_device_inode = Some((metadata.dev(), metadata.ino()));
        for protected in &self.plan.protected_paths {
            if root
                .join(
                    protected
                        .strip_prefix("/")
                        .map_err(|_| NamespaceError::Root)?,
                )
                .try_exists()
                .map_err(|_| NamespaceError::Root)?
            {
                return Err(NamespaceError::Root);
            }
            self.facts.protected_paths_absent += 1;
        }
        let mut targets = BTreeMap::new();
        let directory =
            fs::read_dir(format!("/proc/{shim}/fd")).map_err(|_| NamespaceError::Descriptor)?;
        for (count, entry) in directory.enumerate() {
            if count >= 16 {
                return Err(NamespaceError::Descriptor);
            }
            let entry = entry.map_err(|_| NamespaceError::Descriptor)?;
            let number = entry
                .file_name()
                .to_str()
                .and_then(|v| v.parse::<u32>().ok())
                .ok_or(NamespaceError::Descriptor)?;
            let target = fs::read_link(entry.path()).map_err(|_| NamespaceError::Descriptor)?;
            if targets.insert(number, target).is_some() {
                return Err(NamespaceError::Descriptor);
            }
        }
        if targets.len() != 8
            || targets.get(&0).map(PathBuf::as_path) != Some(Path::new("/dev/null"))
            || targets.get(&1).map(PathBuf::as_path) != Some(Path::new("/dev/null"))
            || !targets
                .get(&2)
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.starts_with("pipe:["))
        {
            return Err(NamespaceError::Descriptor);
        }
        for (fd, name) in [
            (3, "stdout"),
            (4, "stderr"),
            (5, "ready"),
            (6, "release"),
            (7, "status"),
        ] {
            if targets.get(&fd).map(PathBuf::as_path)
                != Some(Path::new(&format!("/channels/{name}")))
            {
                return Err(NamespaceError::Descriptor);
            }
            let metadata = fs::metadata(format!("/proc/{shim}/fd/{fd}"))
                .map_err(|_| NamespaceError::Descriptor)?;
            let expected = self
                .channel_identities
                .get(name)
                .ok_or(NamespaceError::Descriptor)?;
            if metadata.dev() != expected.device || metadata.ino() != expected.inode {
                return Err(NamespaceError::Descriptor);
            }
        }
        self.facts.descriptors_verified = true;
        self.structural_verified = true;
        Ok(true)
    }

    fn advance_readback(&mut self, deadline: Instant) -> Result<bool, NamespaceError> {
        if self.readback_index >= self.plan.public_files.len() {
            return Ok(true);
        }
        let public = &self.plan.public_files[self.readback_index];
        if self.readback_file.is_none() {
            let shim = self.facts.shim_pid.ok_or(NamespaceError::Pid)?;
            let path = PathBuf::from(format!("/proc/{shim}/root")).join(
                public
                    .namespace
                    .strip_prefix("/")
                    .map_err(|_| NamespaceError::Root)?,
            );
            let file = File::open(path).map_err(|_| NamespaceError::Digest)?;
            let size = file.metadata().map_err(|_| NamespaceError::Digest)?.len();
            if size > 256 * 1024 * 1024 {
                return Err(NamespaceError::Bound);
            }
            self.readback_file = Some((file, Sha256::new(), 0));
        }
        let (file, digest, count) = self.readback_file.as_mut().ok_or(NamespaceError::Digest)?;
        let mut buffer = [0_u8; 8192];
        for _ in 0..16 {
            if Instant::now() >= deadline {
                return Err(NamespaceError::Deadline);
            }
            let size = file.read(&mut buffer).map_err(|_| NamespaceError::Digest)?;
            if size == 0 {
                let observed: [u8; 32] = digest.clone().finalize().into();
                if observed != public.sha256 {
                    return Err(NamespaceError::Digest);
                }
                self.facts
                    .source_sha256_after
                    .insert(public.namespace.clone(), observed);
                self.facts.public_files_verified += 1;
                self.readback_index += 1;
                self.readback_file = None;
                return Ok(self.readback_index >= self.plan.public_files.len());
            }
            *count = count
                .checked_add(size as u64)
                .ok_or(NamespaceError::Bound)?;
            if *count > 256 * 1024 * 1024 {
                return Err(NamespaceError::Bound);
            }
            digest.update(&buffer[..size]);
        }
        Ok(false)
    }

    fn release_candidate(&mut self, deadline: Instant) -> Result<(), NamespaceError> {
        if Instant::now() >= deadline {
            return Err(NamespaceError::Deadline);
        }
        self.release
            .as_mut()
            .ok_or(NamespaceError::Channel)?
            .write_all(&[1])
            .map_err(|_| NamespaceError::Channel)?;
        self.facts.release_sent = true;
        Ok(())
    }

    fn accept_protection_attestation(&mut self, deadline: Instant) -> Result<(), NamespaceError> {
        if Instant::now() >= deadline {
            return Err(NamespaceError::Deadline);
        }
        let shim = self.facts.shim_pid.ok_or(NamespaceError::Pid)?;
        let status = read_bounded(
            Path::new(&format!("/proc/{shim}/status")),
            64 * 1024,
            deadline,
        )?;
        if !status.windows(6).any(|value| value == b"NSpid:") {
            return Err(NamespaceError::Pid);
        }
        self.remove_status(shim)?;
        self.release
            .as_mut()
            .ok_or(NamespaceError::Channel)?
            .write_all(&[2])
            .map_err(|_| NamespaceError::Channel)?;
        self.release.take();
        self.facts.protection_verified = true;
        Ok(())
    }

    fn remove_status(&mut self, shim: u32) -> Result<(), NamespaceError> {
        let expected = self
            .channel_identities
            .get("status")
            .ok_or(NamespaceError::Channel)?;
        remove_status_at(
            &self.channel_directory,
            self.channel_directory_identity,
            expected,
            &self.plan.channels_host,
            Path::new(&format!("/proc/{shim}/root/channels/status")),
            &mut self.facts.unresolved,
        )
    }
}

fn check_scratch_mounts(bytes: &[u8]) -> Result<(), NamespaceError> {
    let text = std::str::from_utf8(bytes).map_err(|_| NamespaceError::Scratch)?;
    let mut required = BTreeSet::from(["/", "/dev", "/proc", "/channels", "/work"]);
    for line in text.lines() {
        let (before, after) = line.split_once(" - ").ok_or(NamespaceError::Scratch)?;
        let fields: Vec<_> = before.split_ascii_whitespace().collect();
        if fields.len() < 6 {
            return Err(NamespaceError::Scratch);
        }
        let path = fields[4];
        let read_only = fields[5].split(',').any(|flag| flag == "ro");
        let writable = fields[5].split(',').any(|flag| flag == "rw");
        if read_only == writable {
            return Err(NamespaceError::Scratch);
        }
        if required.remove(path) && ((path == "/work") == read_only) {
            return Err(NamespaceError::Scratch);
        }
        if writable && path != "/work" {
            let filesystem = after
                .split_ascii_whitespace()
                .next()
                .ok_or(NamespaceError::Scratch)?;
            let device = matches!(
                path,
                "/dev/null"
                    | "/dev/zero"
                    | "/dev/full"
                    | "/dev/random"
                    | "/dev/urandom"
                    | "/dev/tty"
            );
            if !(device || path == "/dev/pts" && filesystem == "devpts") {
                return Err(NamespaceError::Scratch);
            }
        }
    }
    if required.is_empty() {
        Ok(())
    } else {
        Err(NamespaceError::Scratch)
    }
}

fn remove_status_at(
    directory: &File,
    directory_identity: FileIdentity,
    expected: &FileIdentity,
    named_directory_path: &Path,
    namespace_status: &Path,
    unresolved: &mut Vec<&'static str>,
) -> Result<(), NamespaceError> {
    let named_directory =
        fs::symlink_metadata(named_directory_path).map_err(|_| NamespaceError::Channel)?;
    if !named_directory.file_type().is_dir()
        || named_directory.dev() != directory_identity.device
        || named_directory.ino() != directory_identity.inode
    {
        unresolved.push("channel directory substituted");
        return Err(NamespaceError::Channel);
    }
    let metadata = statat(directory, "status", AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|_| NamespaceError::Channel)?;
    if !FileType::from_raw_mode(metadata.st_mode).is_fifo()
        || metadata.st_dev != expected.device
        || metadata.st_ino != expected.inode
    {
        unresolved.push("status inode substituted");
        return Err(NamespaceError::Channel);
    }
    unlinkat(directory, "status", AtFlags::empty()).map_err(|_| NamespaceError::Channel)?;
    match statat(directory, "status", AtFlags::SYMLINK_NOFOLLOW) {
        Err(error) if error == rustix::io::Errno::NOENT => {}
        _ => return Err(NamespaceError::Channel),
    }
    if namespace_status
        .try_exists()
        .map_err(|_| NamespaceError::Channel)?
    {
        return Err(NamespaceError::Channel);
    }
    Ok(())
}

const fn should_hash_postrun(interruption: Option<Interruption>) -> bool {
    interruption.is_none()
}

const fn observer_settled(failed: bool, physically_settled: bool, facts: &NamespaceFacts) -> bool {
    physically_settled
        && (failed
            || facts.terminal_json
                && facts.native_status.is_some()
                && facts.native_status_consistent
                && facts.protection_verified)
}

impl Observer for NamespaceObserver {
    #[expect(clippy::too_many_lines, reason = "ordered observer state transition")]
    fn observe(&mut self, observation: &ProcessObservation<'_>) -> ObserverDecision {
        self.stop_namespace(observation);
        let outcome = if self.failed.is_some() {
            Ok(())
        } else {
            (|| {
                if self.readback_complete && !self.facts.release_sent {
                    self.inspect_resources(observation)?;
                    self.release_candidate(observation.work_deadline)?;
                }
                let stdout_result = drain(
                    &mut self.stdout,
                    &mut self.candidate_stdout,
                    self.facts.release_sent,
                );
                FirstStdoutObservation::capture(
                    &mut self.first_stdout_observation,
                    &self.candidate_stdout,
                );
                stdout_result?;
                drain(
                    &mut self.stderr,
                    &mut self.candidate_stderr,
                    self.facts.release_sent,
                )?;
                drain_limit(
                    &mut self.native,
                    &mut self.native_stream,
                    self.facts.release_sent,
                    128,
                )
                .map_err(|_| NamespaceError::Status)?;
                self.facts.native_status_eof = self.native_stream.eof;
                if self.facts.native_status_eof && self.facts.native_status.is_none() {
                    self.facts.native_status = native_status(&self.native_stream.bytes)?;
                }
                if !self.initial_seen {
                    self.initial(&observation.stdout.bytes, observation.work_deadline)?;
                }
                if self.initial_seen && !self.facts.release_sent {
                    let mut bytes = [0_u8; 64];
                    match self.ready.read(&mut bytes) {
                        Ok(0) => {}
                        Ok(n) => self.ready_bytes.extend_from_slice(&bytes[..n]),
                        Err(e)
                            if matches!(
                                e.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                            ) => {}
                        Err(_) => return Err(NamespaceError::Channel),
                    }
                    if self.ready_bytes.ends_with(b"\n") && !self.structural_verified {
                        let _ = self.inspect_ready(observation.work_deadline)?;
                    }
                    if self.structural_verified
                        && self.advance_readback(observation.work_deadline)?
                    {
                        self.readback_complete = true;
                    }
                } else if self.facts.release_sent && !self.ready_eof {
                    let mut bytes = [0_u8; 64];
                    match self.ready.read(&mut bytes) {
                        Ok(0) => self.ready_eof = true,
                        Ok(n) => {
                            self.ready_bytes.extend_from_slice(&bytes[..n]);
                            let expected = b"HEE3_NAMESPACE_PROTECTED_V1 dumpable=0 core_limit=0\n";
                            if self.ready_bytes.len() > 128
                                || !expected.starts_with(
                                    &self.ready_bytes[self
                                        .ready_bytes
                                        .iter()
                                        .position(|b| *b == b'\n')
                                        .map_or(0, |i| i + 1)..],
                                )
                            {
                                return Err(NamespaceError::Status);
                            }
                            if self.ready_bytes.ends_with(expected)
                                && !self.facts.protection_verified
                            {
                                self.accept_protection_attestation(observation.work_deadline)?;
                            }
                        }
                        Err(e)
                            if matches!(
                                e.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                            ) => {}
                        Err(_) => return Err(NamespaceError::Channel),
                    }
                }
                if observation.stdout.bytes.len() < self.status_used {
                    return Err(NamespaceError::Status);
                }
                self.facts.json_exit_code =
                    terminal_status(&observation.stdout.bytes[self.status_used..])?;
                self.facts.terminal_json = self.facts.json_exit_code.is_some();
                if let (Some(native), Some(json)) =
                    (self.facts.native_status, self.facts.json_exit_code)
                {
                    self.facts.native_status_consistent = status_consistent(native, json);
                    if !self.facts.native_status_consistent {
                        return Err(NamespaceError::Status);
                    }
                }
                Ok(())
            })()
        };
        if let Err(error) = outcome {
            self.failed.get_or_insert(error);
            if self.facts.unresolved.is_empty() {
                self.facts.unresolved.push("namespace observer failure");
            }
        }
        if self.failed.is_some() {
            self.release.take();
            let _ = drain(&mut self.stdout, &mut self.candidate_stdout, true);
            FirstStdoutObservation::capture(
                &mut self.first_stdout_observation,
                &self.candidate_stdout,
            );
            let _ = drain(&mut self.stderr, &mut self.candidate_stderr, true);
            let _ = drain_limit(&mut self.native, &mut self.native_stream, true, 128);
            self.facts.native_status_eof = self.native_stream.eof;
            let mut byte = [0_u8; 1];
            if matches!(self.ready.read(&mut byte), Ok(0)) {
                self.ready_eof = true;
            }
        }
        if observation.stdout.bytes.len() >= self.status_used {
            match terminal_status(&observation.stdout.bytes[self.status_used..]) {
                Ok(value) => {
                    self.facts.json_exit_code = value;
                    self.facts.terminal_json = value.is_some();
                }
                Err(error) => {
                    self.failed.get_or_insert(error);
                }
            }
        }
        if let Some(fd) = &self.pid1_fd {
            self.facts.pid1_terminal = terminal(fd);
        }
        if let Some(fd) = &self.shim_fd {
            self.facts.shim_terminal = terminal(fd);
        }
        if self.facts.terminal_json
            && self.facts.native_status_eof
            && self.facts.native_status.is_none()
        {
            self.failed.get_or_insert(NamespaceError::Status);
        }
        // A causal failure remains a failure, but it must not force the generic owner
        // to wait its full cleanup cutoff after every physical custody predicate is
        // already settled. Success retains the stricter status/protection predicates.
        let physically_settled = observation.leader_terminal
            && self.facts.pid1_terminal
            && self.facts.shim_terminal
            && self.candidate_stdout.eof
            && self.candidate_stderr.eof
            && self.ready_eof
            && self.facts.native_status_eof
            && self.release.is_none();
        let ready = observer_settled(self.failed.is_some(), physically_settled, &self.facts);
        ObserverDecision {
            hold_stdin: false,
            ready_to_finish: ready,
            stop: self.failed.map(|_| Interruption::ObserverError),
        }
    }
}

impl NamespaceObserver {
    fn channels_owned(&mut self) -> bool {
        for name in ["stdout", "stderr", "ready", "release", "status"] {
            let Ok(metadata) = fs::symlink_metadata(self.plan.channels_host.join(name)) else {
                return false;
            };
            let Some(expected) = self.channel_identities.get(name) else {
                return false;
            };
            if !metadata.file_type().is_fifo()
                || metadata.dev() != expected.device
                || metadata.ino() != expected.inode
            {
                self.facts.unresolved.push("channel inode substituted");
                return false;
            }
        }
        true
    }

    fn channels_owned_after_run(&mut self) -> bool {
        for name in ["stdout", "stderr", "ready", "release"] {
            let Ok(metadata) = fs::symlink_metadata(self.plan.channels_host.join(name)) else {
                return false;
            };
            let Some(expected) = self.channel_identities.get(name) else {
                return false;
            };
            if !metadata.file_type().is_fifo()
                || metadata.dev() != expected.device
                || metadata.ino() != expected.inode
            {
                self.facts.unresolved.push("channel inode substituted");
                return false;
            }
        }
        match fs::symlink_metadata(self.plan.channels_host.join("status")) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Ok(metadata) => self
                .channel_identities
                .get("status")
                .is_some_and(|expected| {
                    metadata.file_type().is_fifo()
                        && metadata.dev() == expected.device
                        && metadata.ino() == expected.inode
                }),
            _ => false,
        }
    }

    fn cleanup_unspawned_channels(&mut self) -> bool {
        if !self.channels_owned() {
            return false;
        }
        self.release.take();
        let complete = remove_channels(&self.plan.channels_host);
        self.facts.channel_cleanup_complete = complete;
        complete
    }
}

#[cfg(test)]
mod native_status_tests {
    use super::{NativeStatus, native_status, status_consistent};

    #[test]
    fn ordinary_high_exit_is_not_a_signal() {
        let value =
            native_status(b"HEE3_NATIVE_STATUS_V1 kind=exit code=134 raw=34304 core=false\n");
        assert_eq!(
            value,
            Ok(Some(NativeStatus::Exit {
                code: 134,
                raw: 34_304
            }))
        );
        assert!(status_consistent(value.unwrap().unwrap(), 134));
    }

    #[test]
    fn signal_and_core_bit_are_native_facts() {
        let value =
            native_status(b"HEE3_NATIVE_STATUS_V1 kind=signal signal=6 raw=134 core=true\n");
        assert_eq!(
            value,
            Ok(Some(NativeStatus::Signal {
                signal: 6,
                raw: 134,
                core: true
            }))
        );
        assert!(status_consistent(value.unwrap().unwrap(), 134));
    }

    #[test]
    fn malformed_noncanonical_or_inconsistent_frames_refuse() {
        assert!(
            native_status(b"HEE3_NATIVE_STATUS_V1 kind=exit code=0134 raw=34304 core=false\n")
                .is_err()
        );
        assert!(
            native_status(b"HEE3_NATIVE_STATUS_V1 kind=signal signal=6 raw=6 core=true\n").is_err()
        );
        assert!(!status_consistent(
            NativeStatus::Exit { code: 1, raw: 256 },
            2
        ));
    }
}

#[cfg(test)]
mod readonly_status_tests {
    use super::{FileIdentity, NamespaceError, make_channels, remove_channels, remove_status_at};
    use rustix::fs::{Mode, mkfifoat};
    use std::{
        fs,
        os::unix::fs::{FileTypeExt, MetadataExt},
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "hee3-ro-channel-{label}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn missing_or_substituted_status_is_never_unlinked_as_owned() {
        for substitute in [false, true] {
            let root = root(if substitute { "substitute" } else { "missing" });
            let (stdout, stderr, ready, release, native, identities, directory) =
                make_channels(&root).unwrap();
            let stat = rustix::fs::fstat(&directory).unwrap();
            let directory_identity = FileIdentity {
                device: stat.st_dev,
                inode: stat.st_ino,
            };
            fs::remove_file(root.join("status")).unwrap();
            if substitute {
                mkfifoat(&directory, "status", Mode::RUSR | Mode::WUSR).unwrap();
            }
            let replacement = fs::symlink_metadata(root.join("status")).ok();
            let mut unresolved = vec![];
            assert_eq!(
                remove_status_at(
                    &directory,
                    directory_identity,
                    identities.get("status").unwrap(),
                    &root,
                    &root.join("namespace-view-is-absent"),
                    &mut unresolved,
                ),
                Err(NamespaceError::Channel)
            );
            if let Some(expected) = replacement {
                let retained = fs::symlink_metadata(root.join("status")).unwrap();
                assert!(retained.file_type().is_fifo());
                assert_eq!(
                    (retained.dev(), retained.ino()),
                    (expected.dev(), expected.ino())
                );
            }
            drop((stdout, stderr, ready, release, native, directory));
            assert!(remove_channels(&root));
        }
    }
}

#[cfg(test)]
mod cancellation_settlement_tests {
    use super::{
        Interruption, NamespaceFacts, NativeStatus, observer_settled, should_hash_postrun,
    };

    #[test]
    fn failed_observation_requires_complete_physical_settlement() {
        let facts = NamespaceFacts::default();
        assert!(!observer_settled(true, false, &facts));
        assert!(observer_settled(true, true, &facts));
    }

    #[test]
    fn uninterrupted_success_retains_strict_facts_and_posthash() {
        let changes: [fn(&mut NamespaceFacts); 4] = [
            |facts| facts.terminal_json = false,
            |facts| facts.native_status = None,
            |facts| facts.native_status_consistent = false,
            |facts| facts.protection_verified = false,
        ];
        for change in changes {
            let mut facts = NamespaceFacts {
                terminal_json: true,
                native_status: Some(NativeStatus::Exit { code: 0, raw: 0 }),
                native_status_consistent: true,
                protection_verified: true,
                ..NamespaceFacts::default()
            };
            assert!(observer_settled(false, true, &facts));
            assert!(!observer_settled(false, false, &facts));
            change(&mut facts);
            assert!(!observer_settled(false, true, &facts));
        }
        assert!(should_hash_postrun(None));
        assert!(!should_hash_postrun(Some(Interruption::Cancelled)));
    }
}

#[cfg(test)]
mod stdout_observation_tests {
    use super::{FirstStdoutObservation, Stream};
    use std::time::Instant;

    #[test]
    fn no_bytes_has_no_observation_and_later_drains_do_not_replace_first() {
        let mut first = None;
        let mut stream = Stream::default();
        FirstStdoutObservation::capture(&mut first, &stream);
        assert!(first.is_none());
        stream.bytes.push(b'R');
        stream.observed_bytes = 1;
        let before = Instant::now();
        FirstStdoutObservation::capture(&mut first, &stream);
        let after = Instant::now();
        let recorded = first.unwrap();
        assert!((before..=after).contains(&recorded.observed_at));
        assert_eq!((recorded.observed_bytes, recorded.retained_bytes), (1, 1));
        stream.bytes.extend_from_slice(b"EADY\n");
        stream.observed_bytes = 6;
        FirstStdoutObservation::capture(&mut first, &stream);
        let retained = first.unwrap();
        assert_eq!(retained.observed_at, recorded.observed_at);
        assert_eq!((retained.observed_bytes, retained.retained_bytes), (1, 1));
    }

    #[test]
    fn failed_or_truncated_stream_preserves_actual_counts_without_success_claim() {
        let stream = Stream {
            bytes: vec![b'x'],
            observed_bytes: 3,
            failed: true,
            truncated: true,
            eof: false,
        };
        let mut first = None;
        FirstStdoutObservation::capture(&mut first, &stream);
        let first = first.unwrap();
        assert_eq!((first.observed_bytes, first.retained_bytes), (3, 1));
        assert!(stream.failed && stream.truncated);
    }
}
