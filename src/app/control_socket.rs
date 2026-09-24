//! IPC01: the private control socket the engine serves HEE3-Control/1 on (RC02 paths, RC03 §§3, 5).
//!
//! RC02 fixes the endpoint: `$XDG_RUNTIME_DIR/habitat-engine/control.sock`, in a private
//! directory, with "no fallback to shared `/tmp`". RC03 §5 fixes who is asking: "JSON never
//! supplies the authenticated principal. On IPC01, bind the accepted connection to kernel
//! `SO_PEERCRED`". This module owns exactly that projection; what a frame means is
//! `actions::control`'s.
//!
//! * **Custody is checked, never assumed.** The runtime root and the engine's directory must be
//!   this user's and mode 0700, opened without following a link; the socket's 0600 mode is read
//!   back after it is set (a successful chmod is not a changed state).
//! * **A live engine is never displaced.** Single-instance custody is an exclusive `flock` on
//!   [`LOCK_NAME`] in the engine's directory, taken before the socket path is judged and held by the
//!   [`Prepared`] value for as long as the engine runs; a held lock refuses the start. Only under
//!   that custody is an existing socket judged: one that accepts belongs to a running engine and
//!   refuses the bind, one that refuses connection is stale and is removed. The lock is a byte and
//!   identity mechanism: it is tried once and never waited on.
//! * **The principal is the peer.** Each accepted connection's uid comes from `SO_PEERCRED`. Only
//!   the operator — the uid this engine runs as — is served, under the configured local role
//!   [`OPERATOR_ROLE`]; any other peer is closed before a byte is read.
//! * **Every wait is bounded.** Reads time out after [`IDLE_TIMEOUT`], writes after
//!   [`WRITE_TIMEOUT`]; a stalled peer costs one bounded wait, not the engine.

use crate::actions::control::{Composed, Reply, serve_composed};
use crate::contracts::control::{FrameFault, FrameReader, ReadError};
use crate::store::Principal;
use rustix::fs::{Mode, OFlags};
use std::fs::{self, DirBuilder, File};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The engine's directory under the runtime root.
pub const RUNTIME_DIRECTORY: &str = "habitat-engine";
/// The socket's name in that directory.
pub const SOCKET_NAME: &str = "control.sock";
/// The single-instance lock's name in that directory.
pub const LOCK_NAME: &str = "control.lock";
/// The configured local role the operator is served under.
pub const OPERATOR_ROLE: &str = "operator";
/// How long a connection may sit without sending a byte.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(60);
/// How long one reply may take to leave.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// Why the endpoint could not be prepared or served.
#[derive(Debug)]
pub enum Error {
    /// `XDG_RUNTIME_DIR` is unset, empty or relative; there is no fallback.
    NoRuntimeDirectory,
    /// A path failed its custody check; the name says which.
    Custody(&'static str),
    /// Another engine holds the endpoint: it serves the socket, or is starting under custody.
    Live,
    /// The operating system refused.
    Io(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRuntimeDirectory => f.write_str("XDG_RUNTIME_DIR is unset, empty or relative"),
            Self::Custody(what) => write!(f, "{what} is not this user's private path"),
            Self::Live => f.write_str("another engine holds the control socket"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<rustix::io::Errno> for Error {
    fn from(value: rustix::io::Errno) -> Self {
        Self::Io(value.into())
    }
}

/// The runtime root named by `XDG_RUNTIME_DIR`, checked: absolute, a directory, this user's,
/// mode 0700 (the XDG base-directory contract), reached without following a final link.
///
/// # Errors
///
/// [`Error::NoRuntimeDirectory`] when the variable is absent, empty or relative;
/// [`Error::Custody`] when the directory is not this user's private directory.
pub fn runtime_root(variable: Option<&std::ffi::OsStr>) -> Result<PathBuf, Error> {
    let root = PathBuf::from(variable.ok_or(Error::NoRuntimeDirectory)?);
    if !root.is_absolute() {
        return Err(Error::NoRuntimeDirectory);
    }
    private_directory(&root, "runtime root")?;
    Ok(root)
}

fn private_directory(path: &Path, name: &'static str) -> Result<(), Error> {
    let directory = File::from(rustix::fs::open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )?);
    let meta = directory.metadata()?;
    if !meta.is_dir()
        || meta.uid() != rustix::process::geteuid().as_raw()
        || meta.mode() & 0o777 != 0o700
    {
        return Err(Error::Custody(name));
    }
    Ok(())
}

/// The endpoint prepared under single-instance custody. Custody lasts as long as this value: drop
/// it and another engine may start.
#[derive(Debug)]
pub struct Prepared {
    socket: PathBuf,
    _custody: File,
}

impl Prepared {
    /// The socket path [`bind`] will bind.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.socket
    }
}

/// Take single-instance custody of `directory`: an exclusive lock on its [`LOCK_NAME`], a 0600
/// regular file of this user's, opened without following a link.
fn custody(directory: &Path) -> Result<File, Error> {
    let lock = match rustix::fs::open(
        directory.join(LOCK_NAME),
        OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
    ) {
        Ok(lock) => File::from(lock),
        Err(rustix::io::Errno::LOOP) => return Err(Error::Custody("control lock")),
        Err(error) => return Err(error.into()),
    };
    let meta = lock.metadata()?;
    if !meta.is_file()
        || meta.uid() != rustix::process::geteuid().as_raw()
        || meta.mode() & 0o777 != 0o600
    {
        return Err(Error::Custody("control lock"));
    }
    match rustix::fs::flock(&lock, rustix::fs::FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => Ok(lock),
        Err(rustix::io::Errno::WOULDBLOCK) => Err(Error::Live),
        Err(error) => Err(error.into()),
    }
}

/// Prepare `<root>/habitat-engine/` under single-instance custody and return the socket path
/// within it, clearing a stale socket and refusing a live one.
///
/// # Errors
///
/// [`Error::Custody`] for a directory, lock or socket path that is not this user's private one;
/// [`Error::Live`] when another engine holds custody or serves the socket; [`Error::Io`] otherwise.
pub fn prepare(root: &Path) -> Result<Prepared, Error> {
    private_directory(root, "runtime root")?;
    let directory = root.join(RUNTIME_DIRECTORY);
    match DirBuilder::new().mode(0o700).create(&directory) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.into()),
    }
    private_directory(&directory, "engine runtime directory")?;
    let held = custody(&directory)?;
    let socket = directory.join(SOCKET_NAME);
    match fs::symlink_metadata(&socket) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
        Ok(meta) => {
            if !meta.file_type().is_socket() || meta.uid() != rustix::process::geteuid().as_raw() {
                return Err(Error::Custody("control socket path"));
            }
            match UnixStream::connect(&socket) {
                Ok(_) => return Err(Error::Live),
                Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {
                    fs::remove_file(&socket)?;
                }
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(Prepared {
        socket,
        _custody: held,
    })
}

/// Bind the prepared socket path and read its 0600 mode back. Binding needs the custody
/// [`prepare`] took; the caller keeps `prepared` for as long as it serves.
///
/// # Errors
///
/// [`Error::Custody`] when the mode did not take; [`Error::Io`] when the bind fails.
pub fn bind(prepared: &Prepared) -> Result<UnixListener, Error> {
    let socket = prepared.socket();
    let listener = UnixListener::bind(socket)?;
    fs::set_permissions(socket, fs::Permissions::from_mode(0o600))?;
    if fs::symlink_metadata(socket)?.mode() & 0o777 != 0o600 {
        return Err(Error::Custody("control socket mode"));
    }
    Ok(listener)
}

/// The kernel's account of who is on the other end of `stream` (`SO_PEERCRED`).
///
/// # Errors
///
/// The socket option's own failure.
pub fn peer_uid(stream: &UnixStream) -> io::Result<u32> {
    Ok(rustix::net::sockopt::socket_peercred(stream)?.uid.as_raw())
}

/// Who a peer is served as: the operator under [`OPERATOR_ROLE`], or nobody.
///
/// The decision is separate from the socket so it can be proved for a uid no test process can
/// become.
///
/// # Errors
///
/// A description of the refusal, naming the peer's uid, when `uid` is not `operator`.
pub fn admit_peer(uid: u32, operator: u32) -> Result<Principal, String> {
    if uid != operator {
        return Err(format!("peer uid {uid} is not the operator"));
    }
    Principal::new(uid, OPERATOR_ROLE).map_err(|_| "the operator role is invalid".to_owned())
}

/// How one connection ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ended {
    /// The peer closed cleanly after `served` replies.
    Clean { served: usize },
    /// The receiver closed it: a frame no reply may be written for (RC03 §3).
    Closed { served: usize, fault: FrameFault },
}

/// Serve one connection's frames in order until it ends. `now_unix_ms` is read once per frame,
/// so the deadline window is judged at each frame's receipt.
///
/// # Errors
///
/// A read or write the operating system refused, including a timeout.
pub fn serve_connection(
    input: impl Read,
    mut output: impl Write,
    principal: &Principal,
    composed: Composed<'_>,
    now_unix_ms: &dyn Fn() -> u64,
) -> io::Result<Ended> {
    let mut reader = FrameReader::new(input);
    let mut served = 0;
    loop {
        let frame = match reader.next_frame() {
            Ok(Some(frame)) => frame,
            Ok(None) => return Ok(Ended::Clean { served }),
            Err(ReadError::Fault(fault)) => return Ok(Ended::Closed { served, fault }),
            Err(ReadError::Io(error)) => return Err(error),
        };
        match serve_composed(&frame, now_unix_ms(), principal, composed) {
            Reply::Frame(bytes) => {
                output.write_all(&bytes)?;
                output.flush()?;
                served += 1;
            }
            Reply::Close(fault) => return Ok(Ended::Closed { served, fault }),
        }
    }
}

/// Accept and serve connections one at a time, forever. A peer that is not the operator is
/// closed unread; each connection's outcome is reported through `report`, which never receives
/// request content.
///
/// # Errors
///
/// Only a failure of `accept` itself; a failed connection is reported and the loop continues.
pub fn run(
    listener: &UnixListener,
    composed: Composed<'_>,
    now_unix_ms: &dyn Fn() -> u64,
    report: &mut dyn FnMut(&str),
) -> io::Result<()> {
    let operator = rustix::process::geteuid().as_raw();
    loop {
        let (stream, _) = listener.accept()?;
        let outcome = accept_one(&stream, operator, composed, now_unix_ms);
        report(&outcome);
    }
}

fn accept_one(
    stream: &UnixStream,
    operator: u32,
    composed: Composed<'_>,
    now_unix_ms: &dyn Fn() -> u64,
) -> String {
    let uid = match peer_uid(stream) {
        Ok(uid) => uid,
        Err(error) => return format!("connection refused: peer credential unreadable ({error})"),
    };
    let principal = match admit_peer(uid, operator) {
        Ok(principal) => principal,
        Err(refusal) => return format!("connection refused: {refusal}"),
    };
    if let Err(error) = stream
        .set_read_timeout(Some(IDLE_TIMEOUT))
        .and_then(|()| stream.set_write_timeout(Some(WRITE_TIMEOUT)))
    {
        return format!("connection refused: timeouts could not be set ({error})");
    }
    match serve_connection(stream, stream, &principal, composed, now_unix_ms) {
        Ok(Ended::Clean { served }) => format!("connection ended: served={served}"),
        Ok(Ended::Closed { served, fault }) => {
            format!("connection closed: served={served} fault={}", fault.name())
        }
        Err(error) => format!("connection failed: {error}"),
    }
}
