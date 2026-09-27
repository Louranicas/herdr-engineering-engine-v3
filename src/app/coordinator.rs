//! The coordinator's start: select the active ledger generation, reconcile it through
//! [`startup::run`], and report what that left as the `health` the control socket serves
//! (review D-C3 step 2; RC02 paths).
//!
//! RC02: "one active generation selected by protected manifest". The manifest is
//! `<state root>/active.json`; the state root must be the operator's private 0700 directory and the
//! manifest a 0600 regular file of the operator's, read without following a link. This module never
//! creates either: commissioning a state root is the operator's act (RC02, T18). An engine started
//! without one serves `health` as `blocked` / `unavailable` and says why on its standard error.
//!
//! The mapping from a startup pass to `health` is a total `match` over every reconciliation, so a
//! new kind of decision cannot reach an operator unclassified: it will not compile until someone
//! decides whether it leaves work outstanding.

use crate::app::custody::{DirectoryError, FileError, PrivateDirectory};
use crate::app::evidence::fresh_id;
use crate::app::startup::{self, Counts, Host, LedgerAccess, Pass};
use crate::contracts::UuidV4;
use crate::contracts::control::{Database, Health, Recovery, Socket};
use crate::recovery::{Mode, Reconciliation};
use crate::store::{RecoveryLimits, Store};
use rustix::fs::{Mode as FileMode, OFlags};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The operator's state root, under their home (RC02).
pub const STATE_DIRECTORY: &str = ".local/state/herdr-engineering-engine-v3";
/// The operator's configuration root, under their home (RC02): every private directory the engine
/// reads its configuration from — the grants, the routes, the class and the native provider — is
/// named relative to it and joined by [`config_path`], so the root is spelled once (R21 N10).
pub const CONFIG_DIRECTORY: &str = ".config/herdr-engineering-engine-v3";

/// `relative` under the configuration root under `home`: `<home>/`[`CONFIG_DIRECTORY`]`/<relative>`.
#[must_use]
pub fn config_path(home: &Path, relative: &str) -> PathBuf {
    home.join(CONFIG_DIRECTORY).join(relative)
}
/// The manifest selecting the active generation.
pub const ACTIVE_MANIFEST: &str = "active.json";
/// The schema the manifest declares.
pub const ACTIVE_SCHEMA: &str = "hee3.active-generation/1";
/// The largest manifest read.
pub const MAX_MANIFEST_BYTES: u64 = 4096;
/// The recovery inventory bound at start: the profile the T07 batteries prove.
pub const START_LIMITS: RecoveryLimits = RecoveryLimits {
    rows: 1024,
    bytes: 1_048_576,
};

/// Why no generation could be selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unselected {
    /// There is no manifest: nothing has been commissioned.
    Absent,
    /// The state root or the manifest is not the operator's private one.
    Custody,
    /// The manifest is not one `hee3.active-generation/1` record.
    Malformed,
}

/// The generation and epoch the manifest selects, validated once where the manifest is read and
/// borrowed from the bytes that were read: nothing downstream re-parses or re-reads them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Active<'m> {
    /// The ledger generation directory under `generations/`.
    pub generation: UuidV4<'m>,
    /// The ledger epoch it must carry.
    pub epoch: UuidV4<'m>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record<'m> {
    schema: &'m str,
    generation: &'m str,
    epoch: &'m str,
}

/// The manifest's bytes, read once under custody; [`Manifest::active`] is the one reading of them.
#[derive(Debug)]
pub struct Manifest(Vec<u8>);

impl Manifest {
    /// The generation and epoch these bytes select.
    ///
    /// # Errors
    ///
    /// [`Unselected::Malformed`] unless the bytes are one `hee3.active-generation/1` record whose
    /// identities are canonical `UUIDv4` text.
    pub fn active(&self) -> Result<Active<'_>, Unselected> {
        let record: Record<'_> =
            serde_json::from_slice(&self.0).map_err(|_| Unselected::Malformed)?;
        if record.schema != ACTIVE_SCHEMA {
            return Err(Unselected::Malformed);
        }
        match (
            UuidV4::parse(record.generation),
            UuidV4::parse(record.epoch),
        ) {
            (Ok(generation), Ok(epoch)) => Ok(Active { generation, epoch }),
            _ => Err(Unselected::Malformed),
        }
    }
}

/// Read the active-generation manifest under `state_root`.
///
/// # Errors
///
/// [`Unselected::Absent`] when there is no state root or manifest; [`Unselected::Custody`] for a
/// root that is not the operator's 0700 directory or a manifest that is not their 0600 regular
/// file (a link is not followed); [`Unselected::Malformed`] for one larger than
/// [`MAX_MANIFEST_BYTES`] or unreadable.
pub fn read_manifest(state_root: &Path) -> Result<Manifest, Unselected> {
    let euid = rustix::process::geteuid().as_raw();
    let root = match rustix::fs::open(
        state_root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        FileMode::empty(),
    ) {
        Ok(root) => File::from(root),
        Err(rustix::io::Errno::NOENT) => return Err(Unselected::Absent),
        Err(_) => return Err(Unselected::Custody),
    };
    let meta = root.metadata().map_err(|_| Unselected::Custody)?;
    if meta.uid() != euid || meta.mode() & 0o777 != 0o700 {
        return Err(Unselected::Custody);
    }
    let manifest = match rustix::fs::openat(
        &root,
        ACTIVE_MANIFEST,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
        FileMode::empty(),
    ) {
        Ok(file) => File::from(file),
        Err(rustix::io::Errno::NOENT) => return Err(Unselected::Absent),
        Err(_) => return Err(Unselected::Custody),
    };
    let meta = manifest.metadata().map_err(|_| Unselected::Custody)?;
    if !meta.is_file() || meta.uid() != euid || meta.mode() & 0o777 != 0o600 {
        return Err(Unselected::Custody);
    }
    let mut bytes = Vec::new();
    manifest
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Unselected::Malformed)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(Unselected::Malformed);
    }
    Ok(Manifest(bytes))
}

/// Whether a reconciliation leaves work for an operator or a later pass.
#[must_use]
pub fn leaves_work_outstanding(reconciliation: &Reconciliation) -> bool {
    match reconciliation {
        // Settled: a refusal of a stale claim, a released workspace, a standing outcome, a
        // cursor refused or reduced to a snapshot.
        Reconciliation::StaleObservationRefused { .. }
        | Reconciliation::StaleGenerationRefused { .. }
        | Reconciliation::WorkspaceReleasable { .. }
        | Reconciliation::WorkspaceReuseRefused { .. }
        | Reconciliation::AcceptanceStands { .. }
        | Reconciliation::CancellationStands { .. }
        | Reconciliation::RefuseStaleCursor { .. }
        | Reconciliation::CursorSnapshotOnly { .. } => false,
        // Open: an outcome nobody knows, cleanup still owed, verification not done, a live
        // attempt still being observed.
        Reconciliation::RetainUnknown { .. }
        | Reconciliation::CleanupCandidate { .. }
        | Reconciliation::VerificationOutstanding { .. }
        | Reconciliation::ReattachObservationOnly { .. } => true,
    }
}

/// What a startup pass leaves, as `health` reports it.
#[must_use]
pub fn health_of(started: Result<&Pass, &str>, checked_unix_ms: u64) -> Health {
    let (recovery, database) = match started {
        Err(_) => (Recovery::Blocked, Database::Unavailable),
        Ok(pass) => {
            let database = match pass.ledger {
                LedgerAccess::Writable => Database::Ready,
                LedgerAccess::InspectionOnly => Database::Degraded,
            };
            let recovery = if pass.mode == Mode::Reconciliation {
                Recovery::Blocked
            } else if pass
                .attempts
                .iter()
                .any(|entry| leaves_work_outstanding(&entry.decision.reconciliation))
                || pass
                    .cursors
                    .iter()
                    .any(|entry| leaves_work_outstanding(&entry.decision.reconciliation))
            {
                Recovery::Pending
            } else {
                Recovery::Complete
            };
            (recovery, database)
        }
    };
    Health {
        recovery,
        database,
        socket: Socket::Owned,
        checked_unix_ms,
    }
}

/// The line a completed start prints: the generation, what the pass did (the cleanup tail
/// included, so a backlog is never silent — B03c) and the health it left.
#[must_use]
pub fn startup_line(generation: &str, counts: Counts, health: &Health) -> String {
    let Counts {
        attempts,
        writes,
        cleanup,
        cleanup_backlog,
    } = counts;
    format!(
        "generation {generation} reconciled: attempts={attempts} writes={writes} cleanup={cleanup} \
         cleanup_backlog={cleanup_backlog} recovery={} database={}",
        health.recovery.name(),
        health.database.name()
    )
}

/// A generation startup reconciled: the [`Active`] it was selected by, the pass, and the ledger
/// the pass left open writable. Only [`observe_at_start`] makes one, so the three cannot disagree.
#[derive(Debug)]
pub struct Reconciled<'m> {
    /// What the manifest selected when it was read.
    pub active: Active<'m>,
    /// What startup found and did.
    pub pass: Pass,
    store: Option<Store>,
}

/// What the start left: the `health` to serve, the line naming it, and the reconciled
/// generation when startup ran to the end.
#[derive(Debug)]
pub struct Started<'m> {
    pub health: Health,
    pub line: String,
    pub reconciled: Option<Reconciled<'m>>,
}

/// Reconcile the generation `manifest` selects under `state_root`: the observation `health`
/// serves, a line naming why when it is not ready, and the generation with its still-open ledger.
#[must_use]
pub fn observe_at_start<'m>(
    state_root: &Path,
    manifest: &'m Result<Manifest, Unselected>,
    checked_unix_ms: u64,
    deadline: Instant,
) -> Started<'m> {
    let refused = |why: String| Started {
        health: health_of(Err(&why), checked_unix_ms),
        line: why,
        reconciled: None,
    };
    let active = match manifest.as_ref().map_err(|unselected| *unselected) {
        Ok(manifest) => manifest.active(),
        Err(unselected) => Err(unselected),
    };
    let active = match active {
        Ok(active) => active,
        Err(unselected) => {
            return refused(format!(
                "no active generation at {} ({unselected:?})",
                state_root.display()
            ));
        }
    };
    let startup = startup::Startup {
        root: state_root,
        generation: active.generation,
        epoch: active.epoch,
        limits: START_LIMITS,
        restored_from: None,
        cursors: &[],
        claims: &[],
        deadline,
    };
    let mut host = Host::new(deadline);
    match startup::run_and_hold(&startup, &mut host) {
        Ok((pass, store)) => {
            let health = health_of(Ok(&pass), checked_unix_ms);
            let line = startup_line(&pass.generation, pass.counts(), &health);
            Started {
                health,
                line,
                reconciled: Some(Reconciled {
                    active,
                    pass,
                    store,
                }),
            }
        }
        Err(error) => refused(format!("startup refused: {error:?}")),
    }
}

/// The state root under `home`.
#[must_use]
pub fn state_root(home: &Path) -> PathBuf {
    home.join(STATE_DIRECTORY)
}

/// The attempts root the dispatcher materialises workspaces, job roots and plan roots under
/// (B14b-1, R20 round 2 A9 first half): ONE function of the state root, never a `Dispatch` field a
/// caller fills. B14b-2 records each attempt's root in the ledger at begin.
#[must_use]
pub fn attempts_root(state_root: &Path) -> PathBuf {
    state_root.join("attempts")
}

/// The attempts root's identity marker (B14b-2 closure C18): one file in the root.
pub const ROOT_ID_MARKER: &str = ".hee3-root-id";
/// The largest marker read.
pub const MAX_ROOT_ID_BYTES: u64 = 64;

/// Why an attempts root's identity could not be had (B14b-2 closure C18).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootIdError {
    /// Nothing is at the root's path.
    Absent,
    /// The root is not this user's private 0700 directory, or its marker is not this user's
    /// regular 0600 file reached without following a link.
    Custody,
    /// The root's path is not its own canonical path: it is relative, a directory on the way to it
    /// is reached through a link, or it is itself a link. The plan refuses such a root when the
    /// first task is dispatched (`plan_root_not_canonical`), so the door refuses it at startup,
    /// before any read or create (B14b-2 closure D5).
    NotCanonical,
    /// The root exists and carries no marker. It is refused, never marked after the fact and
    /// never re-created: what stands there is not a root this engine made.
    Unmarked,
    /// The marker is over [`MAX_ROOT_ID_BYTES`] or is not one `UuidV4`.
    Marker,
    /// No identity could be drawn before the caller's deadline.
    Entropy,
    /// Any other I/O failure, by its kind.
    Io(std::io::ErrorKind),
}

/// The attempts root's identity, read through the custody door (B14b-2 closure C18): the root must
/// be this user's private 0700 directory, opened without following a link, and its
/// [`ROOT_ID_MARKER`] this user's regular 0600 file, read without following a link under
/// [`MAX_ROOT_ID_BYTES`], holding exactly one `UuidV4`. The one reader: the runtime records what it
/// returns at every begin, and a restart compares what it returns with what the ledger recorded.
///
/// # Errors
/// [`RootIdError::Absent`] when nothing is at `root`; [`RootIdError::Unmarked`] for a root with no
/// marker; [`RootIdError::Custody`], [`RootIdError::Marker`] or [`RootIdError::Io`] otherwise.
pub fn read_root_id(root: &Path) -> Result<String, RootIdError> {
    let directory = PrivateDirectory::open(root).map_err(|error| match error {
        DirectoryError::NotFound => RootIdError::Absent,
        DirectoryError::Custody => RootIdError::Custody,
        DirectoryError::Io(error) => RootIdError::Io(error.kind()),
    })?;
    let bytes = directory
        .read(ROOT_ID_MARKER, MAX_ROOT_ID_BYTES)
        .map_err(|error| match error {
            FileError::NotFound => RootIdError::Unmarked,
            FileError::Custody => RootIdError::Custody,
            FileError::TooLarge => RootIdError::Marker,
            FileError::Io(error) => RootIdError::Io(error.kind()),
        })?;
    let id = String::from_utf8(bytes).map_err(|_| RootIdError::Marker)?;
    UuidV4::parse(&id).map_err(|_| RootIdError::Marker)?;
    Ok(id)
}

/// The attempts root at `root`, created when absent and marked by this door alone (B14b-2 closure
/// C18). A root is born marked: a directory is staged 0700 beside it, its marker (a fresh `UuidV4`
/// from the one id door, under the caller's `deadline`) written once and read back through
/// [`read_root_id`], and the directory renamed onto `root` only if nothing is there, so no crash
/// leaves a root without its marker. A root that exists is read, never re-marked: the id it returns
/// is the one written when the root was created, however many times the engine starts. A leftover
/// staged directory (a crash between the stage and the rename) is never renamed and never read.
/// Before any read or create, the root must be its own canonical path (`canonical_root`).
///
/// # Errors
/// [`RootIdError::NotCanonical`] for a root that is not its own canonical path; those of
/// [`read_root_id`] for a root that exists (an unmarked one is [`RootIdError::Unmarked`]);
/// [`RootIdError::Entropy`] when no id can be drawn; [`RootIdError::Custody`] for a root with no
/// parent or a name that is not UTF-8; [`RootIdError::Io`] for a parent that cannot be resolved, or
/// a stage or a rename that failed.
pub fn prepare_attempts_root(root: &Path, deadline: Instant) -> Result<String, RootIdError> {
    canonical_root(root)?;
    match read_root_id(root) {
        Err(RootIdError::Absent) => {}
        read => return read,
    }
    let (Some(parent), Some(name)) = (root.parent(), root.file_name().and_then(|n| n.to_str()))
    else {
        return Err(RootIdError::Custody);
    };
    let id = fresh_id(deadline).map_err(|_| RootIdError::Entropy)?;
    let id = id.as_str();
    let staged = parent.join(format!(".{name}.{id}.staged"));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&staged)
        .map_err(|error| RootIdError::Io(error.kind()))?;
    let placed = place_marked(&staged, root, id);
    if placed.is_err() {
        // Best effort: the staged directory is never read, so a leftover is inert.
        let _ = std::fs::remove_file(staged.join(ROOT_ID_MARKER));
        let _ = std::fs::remove_dir(&staged);
    }
    placed?;
    match read_root_id(root) {
        Ok(read) if read == id => Ok(read),
        Ok(_) => Err(RootIdError::Marker),
        Err(error) => Err(error),
    }
}

/// The attempts root is its own canonical path (B14b-2 closure D5), the rule the plan applies to
/// it when a task is dispatched (`plan_root_not_canonical`), decided here at startup instead of
/// stopping every admitted task: `root` is absolute, its parent resolves to itself, and a root that
/// resolves resolves to itself (a root that does not resolve is left to [`read_root_id`], which
/// names it). A root with no parent is left to the reader too.
///
/// # Errors
/// [`RootIdError::NotCanonical`] when one of those does not hold; [`RootIdError::Io`], by its kind,
/// for a parent that cannot be resolved.
fn canonical_root(root: &Path) -> Result<(), RootIdError> {
    if !root.is_absolute() {
        return Err(RootIdError::NotCanonical);
    }
    let Some(parent) = root.parent() else {
        return Ok(());
    };
    let resolved = parent
        .canonicalize()
        .map_err(|error| RootIdError::Io(error.kind()))?;
    if resolved != parent {
        return Err(RootIdError::NotCanonical);
    }
    match root.canonicalize() {
        Ok(resolved) if resolved != root => Err(RootIdError::NotCanonical),
        Ok(_) | Err(_) => Ok(()),
    }
}

/// Mark `staged` with `id`, read the marker back, and rename `staged` onto `root` only if nothing is
/// there; the parent is synced after the rename.
fn place_marked(staged: &Path, root: &Path, id: &str) -> Result<(), RootIdError> {
    let directory = PrivateDirectory::open(staged).map_err(|error| match error {
        DirectoryError::NotFound | DirectoryError::Custody => RootIdError::Custody,
        DirectoryError::Io(error) => RootIdError::Io(error.kind()),
    })?;
    directory
        .create_new(ROOT_ID_MARKER, id.as_bytes())
        .map_err(|error| match error {
            FileError::NotFound | FileError::Custody | FileError::TooLarge => RootIdError::Custody,
            FileError::Io(error) => RootIdError::Io(error.kind()),
        })?;
    if read_root_id(staged)? != id {
        return Err(RootIdError::Marker);
    }
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        staged,
        rustix::fs::CWD,
        root,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(|errno| RootIdError::Io(std::io::Error::from(errno).kind()))?;
    let parent = root.parent().ok_or(RootIdError::Custody)?;
    File::open(parent)
        .and_then(|parent| parent.sync_all())
        .map_err(|error| RootIdError::Io(error.kind()))
}

/// Hand the ledger startup reconciled, still open and still locked, to the task owner. Nothing is
/// re-read: not the manifest, not the store.
///
/// # Errors
///
/// A line naming why, when startup left the ledger inspection-only; the caller serves task
/// actions as `unavailable` and says so.
pub fn compose_tasks(reconciled: Reconciled<'_>) -> Result<crate::app::tasks::StoreTasks, String> {
    let store = reconciled
        .store
        .ok_or_else(|| "the ledger is inspection-only (reconciliation mode)".to_owned())?;
    Ok(crate::app::tasks::StoreTasks::new(
        store,
        reconciled.active.epoch.as_str().to_owned(),
    ))
}
