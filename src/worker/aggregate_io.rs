use super::{Config, Error};
use crate::contracts::Sha256Digest;
use rustix::fs::{Mode, OFlags, fstatfs, open, openat};
use rustix::process::geteuid;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs::{File, Metadata};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path};
use std::time::Instant;
const DIR: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);
pub(super) fn tick(deadline: Instant) -> Result<(), Error> {
    if Instant::now() >= deadline {
        Err(Error::Deadline)
    } else {
        Ok(())
    }
}
pub(super) fn directory(path: &Path, deadline: Instant) -> Result<File, Error> {
    tick(deadline)?;
    if !path.is_absolute() {
        return Err(Error::Invalid);
    }
    let mut file = File::from(open("/", DIR, Mode::empty()).map_err(|_| Error::Io)?);
    for item in path.components() {
        tick(deadline)?;
        match item {
            Component::RootDir => {}
            Component::Normal(name) => {
                file = File::from(openat(&file, name, DIR, Mode::empty()).map_err(|_| Error::Io)?);
            }
            _ => return Err(Error::Invalid),
        }
    }
    Ok(file)
}
pub(super) fn cgroup(path: &str, deadline: Instant) -> Result<File, Error> {
    let file = directory(
        &Path::new("/sys/fs/cgroup").join(path.trim_start_matches('/')),
        deadline,
    )?;
    if fstatfs(&file).map_err(|_| Error::Io)?.f_type != 0x6367_7270 {
        return Err(Error::Identity);
    }
    Ok(file)
}
pub(super) fn stable(file: &File, path: &str, deadline: Instant) -> Result<(), Error> {
    let now = cgroup(path, deadline)?.metadata().map_err(|_| Error::Io)?;
    let before = file.metadata().map_err(|_| Error::Io)?;
    if before.dev() != now.dev() || before.ino() != now.ino() {
        return Err(Error::Identity);
    }
    tick(deadline)
}
pub(super) fn read(
    file: &File,
    name: &str,
    cap: usize,
    deadline: Instant,
) -> Result<Vec<u8>, Error> {
    tick(deadline)?;
    let mut input = File::from(
        openat(
            file,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        )
        .map_err(|_| Error::Io)?,
    );
    if !input.metadata().map_err(|_| Error::Io)?.is_file() {
        return Err(Error::Identity);
    }
    let mut bytes = Vec::new();
    loop {
        tick(deadline)?;
        let mut chunk = [0; 4096];
        let count = input.read(&mut chunk).map_err(|_| Error::Io)?;
        if count == 0 {
            break;
        }
        if bytes.len() + count > cap {
            return Err(Error::Bound);
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    tick(deadline)?;
    Ok(bytes)
}
pub(super) fn text(file: &File, name: &str, deadline: Instant) -> Result<String, Error> {
    String::from_utf8(read(file, name, 4096, deadline)?).map_err(|_| Error::Invalid)
}
pub(super) fn membership(deadline: Instant) -> Result<String, Error> {
    let proc = directory(
        Path::new(&format!("/proc/{}", std::process::id())),
        deadline,
    )?;
    if fstatfs(&proc).map_err(|_| Error::Io)?.f_type != rustix::fs::PROC_SUPER_MAGIC {
        return Err(Error::Identity);
    }
    let raw = text(&proc, "cgroup", deadline)?;
    let path = raw
        .strip_prefix("0::")
        .and_then(|s| s.strip_suffix('\n'))
        .ok_or(Error::Invalid)?;
    if !path.starts_with('/')
        || path.len() > 2048
        || !path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/@._-".contains(&b))
        || path
            .split('/')
            .skip(1)
            .any(|c| c.is_empty() || c == "." || c == "..")
    {
        return Err(Error::Invalid);
    }
    Ok(path.to_owned())
}
fn same(a: &Metadata, b: &Metadata) -> bool {
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
pub(super) fn pin(config: &Config, deadline: Instant) -> Result<(), Error> {
    tick(deadline)?;
    Sha256Digest::parse(&config.busctl_sha256).map_err(|_| Error::Invalid)?;
    if config.busctl != Path::new("/usr/bin/busctl")
        || config.runtime_dir.as_os_str()
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
    let runtime = directory(&config.runtime_dir, deadline)?
        .metadata()
        .map_err(|_| Error::Io)?;
    if runtime.uid() != geteuid().as_raw() || runtime.mode() & 0o7777 != 0o700 {
        return Err(Error::Identity);
    }
    let dir = directory(Path::new("/usr/bin"), deadline)?;
    let file = File::from(
        openat(
            &dir,
            "busctl",
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        )
        .map_err(|_| Error::Io)?,
    );
    let before = file.metadata().map_err(|_| Error::Io)?;
    if !before.is_file()
        || before.uid() != 0
        || before.mode() & 0o7022 != 0
        || before.mode() & 0o111 == 0
        || before.len() > 4 * 1024 * 1024
    {
        return Err(Error::Identity);
    }
    let bytes = read(&dir, "busctl", 4 * 1024 * 1024, deadline)?;
    let mut hash = "sha256:".to_owned();
    for byte in Sha256::digest(&bytes) {
        write!(&mut hash, "{byte:02x}").map_err(|_| Error::Invalid)?;
    }
    let named = File::from(
        openat(
            &dir,
            "busctl",
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        )
        .map_err(|_| Error::Io)?,
    )
    .metadata()
    .map_err(|_| Error::Io)?;
    if hash != config.busctl_sha256
        || bytes.len() as u64 != before.len()
        || !same(&before, &file.metadata().map_err(|_| Error::Io)?)
        || !same(&before, &named)
    {
        return Err(Error::Identity);
    }
    tick(deadline)
}
