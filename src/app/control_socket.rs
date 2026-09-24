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
//! * **Every wait on an admitted peer is bounded.** Reads time out after [`IDLE_TIMEOUT`], writes
//!   after [`WRITE_TIMEOUT`]; a stalled peer costs its own connection, not the engine.
//! * **Admission is bounded and never displaces** (contract-decisions.md, "Initial connection and
//!   grant bounds"). At most [`CONNECTION_CAP`] connections are served at once, each on its own
//!   scoped thread with one request in flight; a peer over the cap is answered
//!   `resource_exhausted` for its first frame and closed, and no admitted peer is disturbed. That
//!   read carries no timer of its own (no new time limit is created here): [`REFUSALS_AT_ONCE`]
//!   such peers are answered at a time, and one more over capacity is closed unread. Each
//!   principal's requests pass a token bucket of [`BURST`] refilled at [`RATE_PER_SECOND`], judged
//!   on the injected clock; a request past it is answered `resource_exhausted` before any effect.

use crate::actions::control::{Composed, Grants, Reply, Tasks, serve_composed};
use crate::contracts::control::{
    ErrorCode, Fault, FrameFault, FrameReader, Health, ReadError, Received, Retry, receive,
};
use crate::store::Principal;
use rustix::fs::{Mode, OFlags};
use std::fs::{self, DirBuilder, File};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
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
/// IPC01 "admits at most8 simultaneous connections total" (contract-decisions.md, "Initial
/// connection and grant bounds").
pub const CONNECTION_CAP: usize = 8;
/// Over-capacity peers answered at once. One more is closed unread: its refusal is the close.
pub const REFUSALS_AT_ONCE: usize = 1;
/// Per-principal admission "capped at100 requests/second" (the same line).
pub const RATE_PER_SECOND: u64 = 100;
/// "with burst32" (the same line): the tokens a principal may spend at once.
pub const BURST: u64 = 32;
/// "an aggregate256 pending control requests" (the same line). Met by construction: each served
/// connection has one request in flight, so at most [`CONNECTION_CAP`] are pending.
pub const AGGREGATE_PENDING: usize = 256;
/// The rule a refused connection names.
pub const CONNECTION_RULE: &str = "at most 8 simultaneous connections";
/// The rule a refused request names.
pub const RATE_RULE: &str = "100 requests/second, burst 32";

const _: () = assert!(CONNECTION_CAP <= AGGREGATE_PENDING);

/// One token, in the thousandths the bucket counts in.
const TOKEN: u64 = 1000;

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

/// Per-principal request admission: a token bucket of [`BURST`] tokens per principal, refilled at
/// [`RATE_PER_SECOND`] by the clock the caller passes in, never by one of its own.
///
/// The table holds at most [`CONNECTION_CAP`] principals, since no more can be connected at once;
/// a full bucket is the same as an absent one, so it is the one a new principal replaces. A new
/// principal finding every bucket in use is refused.
#[derive(Debug, Default)]
pub struct Admission {
    buckets: Mutex<Vec<Bucket>>,
}

#[derive(Debug)]
struct Bucket {
    uid: u32,
    role: String,
    /// Thousandths of a token.
    level: u64,
    /// The latest instant the level was judged at; a clock that steps back refills nothing.
    at_unix_ms: u64,
}

impl Bucket {
    fn refill(&mut self, now_unix_ms: u64) {
        // RATE_PER_SECOND tokens a second is RATE_PER_SECOND thousandths a millisecond.
        let earned = now_unix_ms
            .saturating_sub(self.at_unix_ms)
            .saturating_mul(RATE_PER_SECOND);
        self.level = self.level.saturating_add(earned).min(BURST * TOKEN);
        self.at_unix_ms = self.at_unix_ms.max(now_unix_ms);
    }
}

impl Admission {
    /// An empty table: every principal starts with a full burst.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether one more request of `principal` at `now_unix_ms` is admitted; an admitted request
    /// spends one token.
    #[must_use]
    pub fn admit(&self, principal: &Principal, now_unix_ms: u64) -> bool {
        let mut buckets = self.buckets.lock().unwrap_or_else(PoisonError::into_inner);
        let found = buckets
            .iter()
            .position(|bucket| principal.is(bucket.uid, &bucket.role));
        let index = if let Some(index) = found {
            index
        } else {
            for bucket in buckets.iter_mut() {
                bucket.refill(now_unix_ms);
            }
            buckets.retain(|bucket| bucket.level < BURST * TOKEN);
            if buckets.len() >= CONNECTION_CAP {
                return false;
            }
            buckets.push(Bucket {
                uid: principal.uid(),
                role: principal.role().to_owned(),
                level: BURST * TOKEN,
                at_unix_ms: now_unix_ms,
            });
            buckets.len() - 1
        };
        let Some(bucket) = buckets.get_mut(index) else {
            return false;
        };
        bucket.refill(now_unix_ms);
        if bucket.level < TOKEN {
            return false;
        }
        bucket.level -= TOKEN;
        true
    }
}

/// The reply to a frame refused for capacity under `rule`: `resource_exhausted`, correlated to the
/// frame, or the close RC03 §3 requires for one no reply can be correlated to. Nothing is
/// dispatched, so nothing has an effect.
fn exhausted(payload: &[u8], now_unix_ms: u64, rule: &'static str) -> Reply {
    let fault = Fault::of(
        ErrorCode::ResourceExhausted,
        Retry::SameExactRequest,
        "capacity is exhausted; nothing was done",
    )
    .because(rule);
    match receive(payload, now_unix_ms) {
        Received::Closed(fault) => Reply::Close(fault),
        Received::Refused {
            request_id,
            request_sha256,
            ..
        } => Reply::Frame(fault.frame(&request_id, &request_sha256)),
        Received::Admitted(envelope) => {
            Reply::Frame(fault.frame(&envelope.request_id, &envelope.request_sha256))
        }
    }
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
/// so the deadline window and the principal's admission are judged at each frame's receipt; a
/// frame `admission` refuses is answered `resource_exhausted` and never dispatched.
///
/// # Errors
///
/// A read or write the operating system refused, including a timeout.
pub fn serve_connection(
    input: impl Read,
    mut output: impl Write,
    principal: &Principal,
    composed: Composed<'_>,
    admission: &Admission,
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
        let now = now_unix_ms();
        let reply = if admission.admit(principal, now) {
            serve_composed(&frame, now, principal, composed)
        } else {
            exhausted(&frame, now, RATE_RULE)
        };
        match reply {
            Reply::Frame(bytes) => {
                output.write_all(&bytes)?;
                output.flush()?;
                served += 1;
            }
            Reply::Close(fault) => return Ok(Ended::Closed { served, fault }),
        }
    }
}

/// What the coordinator composed, shareable by every connection's thread.
#[derive(Clone, Copy)]
pub struct Shared<'a> {
    /// The grant store.
    pub grants: &'a (dyn Grants + Sync),
    /// The coordinator's health observation, when composed.
    pub health: Option<&'a Health>,
    /// The task owner, when a writable ledger is composed.
    pub tasks: Option<&'a (dyn Tasks + Sync)>,
}

impl<'a> Shared<'a> {
    fn composed(self) -> Composed<'a> {
        Composed {
            grants: self.grants,
            health: self.health,
            tasks: self.tasks.map(|tasks| tasks as &dyn Tasks),
        }
    }
}

/// One place under a cap, given back when dropped: by the thread that served it, by a spawn that
/// failed, or by an unwinding thread.
struct Place<'a>(&'a AtomicUsize);

impl<'a> Place<'a> {
    fn take(taken: &'a AtomicUsize, cap: usize) -> Option<Self> {
        taken
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < cap).then_some(count + 1)
            })
            .ok()
            .map(|_| Self(taken))
    }
}

impl Drop for Place<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Accept connections forever, serving each admitted one on its own thread under
/// [`CONNECTION_CAP`]. A peer that is not the operator is closed unread; a peer over the cap is
/// refused whole without disturbing an admitted one. Each connection's outcome is reported through
/// `report`, which never receives request content.
///
/// # Errors
///
/// Only a failure of `accept` itself, returned once every connection already admitted has ended; a
/// failed connection is reported and the loop continues.
pub fn run(
    listener: &UnixListener,
    shared: Shared<'_>,
    now_unix_ms: &(dyn Fn() -> u64 + Sync),
    report: &(dyn Fn(&str) + Sync),
) -> io::Result<()> {
    let operator = rustix::process::geteuid().as_raw();
    let admission = Admission::new();
    let serving = AtomicUsize::new(0);
    let refusing = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        loop {
            let (stream, _) = listener.accept()?;
            let principal = match peer_of(&stream, operator) {
                Ok(principal) => principal,
                Err(refusal) => {
                    report(&refusal);
                    continue;
                }
            };
            let admission = &admission;
            let spawned = if let Some(place) = Place::take(&serving, CONNECTION_CAP) {
                std::thread::Builder::new().spawn_scoped(scope, move || {
                    let outcome =
                        serve_admitted(&stream, &principal, shared, admission, now_unix_ms);
                    drop(place);
                    report(&outcome);
                })
            } else if let Some(place) = Place::take(&refusing, REFUSALS_AT_ONCE) {
                std::thread::Builder::new().spawn_scoped(scope, move || {
                    let outcome = refuse_over_capacity(&stream, now_unix_ms);
                    drop(place);
                    report(&outcome);
                })
            } else {
                report("connection refused: over capacity; closed unread");
                continue;
            };
            if let Err(error) = spawned {
                report(&format!(
                    "connection refused: no thread to serve it ({error})"
                ));
            }
        }
    })
}

fn peer_of(stream: &UnixStream, operator: u32) -> Result<Principal, String> {
    let uid = peer_uid(stream)
        .map_err(|error| format!("connection refused: peer credential unreadable ({error})"))?;
    admit_peer(uid, operator).map_err(|refusal| format!("connection refused: {refusal}"))
}

fn serve_admitted(
    stream: &UnixStream,
    principal: &Principal,
    shared: Shared<'_>,
    admission: &Admission,
    now_unix_ms: &dyn Fn() -> u64,
) -> String {
    if let Err(error) = stream
        .set_read_timeout(Some(IDLE_TIMEOUT))
        .and_then(|()| stream.set_write_timeout(Some(WRITE_TIMEOUT)))
    {
        return format!("connection refused: timeouts could not be set ({error})");
    }
    match serve_connection(
        stream,
        stream,
        principal,
        shared.composed(),
        admission,
        now_unix_ms,
    ) {
        Ok(Ended::Clean { served }) => format!("connection ended: served={served}"),
        Ok(Ended::Closed { served, fault }) => {
            format!("connection closed: served={served} fault={}", fault.name())
        }
        Err(error) => format!("connection failed: {error}"),
    }
}

/// Answer an over-capacity peer's first frame `resource_exhausted` and close. Nothing is dispatched.
fn refuse_over_capacity(stream: &UnixStream, now_unix_ms: &dyn Fn() -> u64) -> String {
    let frame = match FrameReader::new(stream).next_frame() {
        Ok(Some(frame)) => frame,
        Ok(None) => return "connection refused: over capacity; closed before a frame".to_owned(),
        Err(ReadError::Fault(fault)) => {
            return format!("connection refused: over capacity; fault={}", fault.name());
        }
        Err(ReadError::Io(error)) => return format!("connection refused: over capacity ({error})"),
    };
    match exhausted(&frame, now_unix_ms(), CONNECTION_RULE) {
        Reply::Frame(bytes) => match (&*stream).write_all(&bytes) {
            Ok(()) => "connection refused: over capacity; answered resource_exhausted".to_owned(),
            Err(error) => format!("connection refused: over capacity ({error})"),
        },
        Reply::Close(fault) => {
            format!("connection refused: over capacity; fault={}", fault.name())
        }
    }
}
