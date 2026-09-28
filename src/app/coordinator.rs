//! The coordinator's start: select the active ledger generation, reconcile it through
//! [`startup::run`], and report what that left as the `health` the control socket serves
//! (review D-C3 step 2; RC02 paths).
//!
//! RC02: "one active generation selected by protected manifest". The manifest is
//! `<state root>/active.json`; the state root must be the operator's private 0700 directory and the
//! manifest a 0600 regular file of the operator's, read without following a link. `serve` never
//! creates either: commissioning a state root is the operator's act (RC02, T18), and [`commission`]
//! is its one door (OPS-1, `habitat-engine commission <deadline-seconds>`) — the one creator of the
//! state root, its manifest and its first ledger. An engine started without one serves `health` as
//! `blocked` / `unavailable` and says why on its standard error.
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
use serde::{Deserialize, Serialize};
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

/// The manifest's one record: the reader decodes it ([`Manifest::active`]) and the writer encodes
/// it ([`render_manifest`]), so the writer cannot emit a field the reader refuses (OPS-1, I4).
#[derive(Deserialize, Serialize)]
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

/// The manifest's bytes for `active`: [`Record`] encoded, the one writer of the format (OPS-1, I4).
///
/// # Errors
/// The encoder's refusal; a record of three strings has none in practice, and none is hidden.
fn render_manifest(active: Active<'_>) -> serde_json::Result<Vec<u8>> {
    serde_json::to_vec(&Record {
        schema: ACTIVE_SCHEMA,
        generation: active.generation.as_str(),
        epoch: active.epoch.as_str(),
    })
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
    /// No identity could be drawn: the system's entropy refused, with time left.
    Entropy,
    /// The caller's deadline was spent before an identity could be drawn (B14b-2 review round 2,
    /// FT-6): told apart from entropy that refused, and nothing staged or created.
    Deadline,
    /// Any other I/O failure, by its kind.
    Io(std::io::ErrorKind),
    /// The root at the path carries another id than the one `serve` prepared at its start: read
    /// at a begin and compared with the id the dispatch carries (B14b-2 review round 2, D9). The
    /// root was replaced under a running engine; no attempt begins under it.
    Changed,
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
/// [`RootIdError::Deadline`] when `deadline` is spent before an id is drawn, and
/// [`RootIdError::Entropy`] when entropy refused with time left; [`RootIdError::Custody`] for a
/// root with no parent or a name that is not UTF-8; [`RootIdError::Io`] for a parent that cannot
/// be resolved, or a stage or a rename that failed.
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
    // The one id door refuses a spent deadline and refused entropy alike; the clock tells them
    // apart here (FT-6). Entropy refusing with time left is reachable only by arranging the host.
    let id = fresh_id(deadline).map_err(|_| {
        if Instant::now() >= deadline {
            RootIdError::Deadline
        } else {
            RootIdError::Entropy
        }
    })?;
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
    if root.parent().is_none() {
        return Err(RootIdError::Custody);
    }
    place_staged(staged, root).map_err(|unplaced| RootIdError::Io(unplaced.kind()))
}

/// Why [`place_staged`] did not finish: the rename failed, so nothing was placed; or the root was
/// placed and its parent could not be synced.
enum Unplaced {
    Rename(std::io::Error),
    ParentSync(std::io::Error),
}

impl Unplaced {
    fn kind(&self) -> std::io::ErrorKind {
        match self {
            Self::Rename(error) | Self::ParentSync(error) => error.kind(),
        }
    }
}

/// Rename `staged` onto `root` only if nothing is there (`RENAME_NOREPLACE`), then sync `root`'s
/// parent: the one rename-into-place door, shared by the attempts root (`place_marked`) and the
/// state root ([`place`]). A root that appeared meanwhile is `AlreadyExists`, never replaced.
///
/// # Errors
/// [`Unplaced::Rename`] when nothing was placed (`InvalidInput` for a root with no parent, checked
/// before the rename); [`Unplaced::ParentSync`] when the root WAS placed and the parent sync failed.
fn place_staged(staged: &Path, root: &Path) -> Result<(), Unplaced> {
    let parent = root
        .parent()
        .ok_or_else(|| Unplaced::Rename(std::io::Error::from(std::io::ErrorKind::InvalidInput)))?;
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        staged,
        rustix::fs::CWD,
        root,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(|error| Unplaced::Rename(error.into()))?;
    File::open(parent)
        .and_then(|parent| parent.sync_all())
        .map_err(Unplaced::ParentSync)
}

/// Why [`commission`] did not complete (OPS-1). Every variant but [`CommissionError::Placed`] is a
/// refusal that left nothing it created in place (its stage, if made, is removed; a spent deadline
/// may leave missing ancestors of the root, which RC02 does not constrain). [`CommissionError::Placed`]
/// is the one that did: the state root stands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommissionError {
    /// Something already stands at the state root: a directory (even an empty one), a file or a
    /// link. It is never altered; the operator removes it by hand (R1.4).
    Exists(PathBuf),
    /// The state root is relative, or has no parent, or its parent is not its own canonical path —
    /// judged at the deepest ancestor that exists before anything is created (block R L5), and at
    /// the parent again once its missing ancestors are made.
    NotCanonical(PathBuf),
    /// The operator's deadline was spent before the stage was verified: nothing was placed.
    Deadline,
    /// The store refused to create the ledger in the stage or to re-open it there, by its error's
    /// `Debug` rendering (`store::Error` holds I/O and SQLite errors, which are not comparable).
    Store(String),
    /// A directory, the manifest, the stage's read-back of a mode, or the rename failed, by its
    /// kind: nothing was placed.
    Io(std::io::ErrorKind),
    /// The stage read back differs from what was written, by what differs: nothing was placed.
    ReadBack(&'static str),
    /// The state root WAS placed — renamed from a stage verified complete before the rename (the
    /// manifest selecting `generation` and `epoch`, 0600; the ledger opening at the current
    /// migration; the root 0700) — and a step after the rename failed (block R F2/M2/F-L1/L8). The
    /// root stands as verified: `serve` reconciles it; a second `commission` refuses it by name.
    Placed {
        root: PathBuf,
        generation: String,
        epoch: String,
        why: PostPlacement,
    },
}

/// What failed after the state root was placed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PostPlacement {
    /// The root's parent could not be synced after the rename, by its kind.
    ParentSync(std::io::ErrorKind),
    /// The operator's deadline was spent before the placed root was read back.
    Deadline,
    /// The store refused to open the ledger at the placed root.
    Store(String),
    /// A mode of the placed root could not be read, by its kind.
    Io(std::io::ErrorKind),
    /// The placed root read back differs from its stage, by what differs.
    ReadBack(&'static str),
}

impl PostPlacement {
    fn line(&self) -> String {
        match self {
            Self::ParentSync(kind) => format!("parent sync: {kind:?}"),
            Self::Deadline => "deadline".to_owned(),
            Self::Store(why) => format!("store refused: {why}"),
            Self::Io(kind) => format!("io: {kind:?}"),
            Self::ReadBack(which) => format!("read-back differs: {which}"),
        }
    }
}

impl CommissionError {
    /// The one line `habitat-engine commission` says for this error (after `habitat-engine: `): a
    /// refusal as `commission refused: ...`, and a placed root as what stands and what failed.
    #[must_use]
    pub fn line(&self) -> String {
        match self {
            Self::Exists(path) => format!(
                "commission refused: state root exists at {}",
                path.display()
            ),
            Self::NotCanonical(path) => format!(
                "commission refused: state root is not its own canonical path ({})",
                path.display()
            ),
            Self::Deadline => "commission refused: deadline".to_owned(),
            Self::Store(why) => format!("commission refused: store refused ({why})"),
            Self::Io(kind) => format!("commission refused: io ({kind:?})"),
            Self::ReadBack(which) => format!("commission refused: read-back differs ({which})"),
            Self::Placed {
                root,
                generation,
                epoch,
                why,
            } => format!(
                "commission: root placed; post-placement verification failed ({}): {} stands \
                 as verified in its stage before the rename (manifest 0600 selecting \
                 generation={generation} epoch={epoch}, its ledger at the current migration, root \
                 0700); serve reconciles it, and a second commission refuses it until it is \
                 removed by hand",
                why.line(),
                root.display()
            ),
        }
    }
}

/// A commissioned state root, every value read back from it after it was placed (OPS-1 step 7):
/// none is copied from the input or written as a literal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Commissioned {
    /// The state root.
    pub root: PathBuf,
    /// The generation the manifest selects, read back through [`read_manifest`].
    pub generation: String,
    /// The epoch the manifest selects, read back through [`read_manifest`].
    pub epoch: String,
    /// The ledger's migration version, read back through [`Store::open_inspection`].
    pub user_version: u32,
    /// The root's permission bits, read back.
    pub root_mode: u32,
    /// The manifest's permission bits, read back.
    pub manifest_mode: u32,
}

impl Commissioned {
    /// The one line `habitat-engine commission` prints on success.
    #[must_use]
    pub fn line(&self) -> String {
        format!(
            "commissioned {} generation={} epoch={} user_version={} root_mode={:04o} \
             manifest_mode={:04o}",
            self.root.display(),
            self.generation,
            self.epoch,
            self.user_version,
            self.root_mode,
            self.manifest_mode
        )
    }
}

/// A state root [`place`] renamed into place from a stage verified complete before the rename:
/// what [`Placed::verify`] reads back at the root.
#[derive(Debug)]
#[must_use = "a placed root is verified at its own path by `Placed::verify`"]
pub struct Placed {
    root: PathBuf,
    generation: String,
    epoch: String,
    staged: Commissioned,
}

impl Placed {
    /// What the stage read back BEFORE the rename, at the stage's own path (which no longer exists
    /// once the root is placed): the proof that the verification preceded the placement.
    #[must_use]
    pub const fn staged(&self) -> &Commissioned {
        &self.staged
    }

    /// OPS-1 step 7: the placed root read back through the readers `serve` uses — the manifest,
    /// the ledger re-opened AT THE ROOT'S OWN PATH (the proof that nothing in the stage recorded
    /// the stage's path, I1's premise), the modes — every value [`Commissioned`] carries taken from
    /// it.
    ///
    /// # Errors
    /// [`CommissionError::Placed`], naming what failed: the root stands.
    pub fn verify(self, deadline: Instant) -> Result<Commissioned, CommissionError> {
        read_back(&self.root, (&self.generation, &self.epoch), deadline).map_err(|unread| {
            CommissionError::Placed {
                why: match unread {
                    Unread::Deadline => PostPlacement::Deadline,
                    Unread::Store(why) => PostPlacement::Store(why),
                    Unread::Io(kind) => PostPlacement::Io(kind),
                    Unread::Differs(which) => PostPlacement::ReadBack(which),
                },
                root: self.root,
                generation: self.generation,
                epoch: self.epoch,
            }
        })
    }
}

/// Commission the state root at `state_root` for `active` (OPS-1; RC02; HO-03): [`place`] it, then
/// [`Placed::verify`] it at its own path. `serve` never creates the root, its manifest or its
/// ledger; this is the operator's door (`habitat-engine commission`). The one `deadline` is the
/// operator's; nothing here adds a bound (R22-4).
///
/// # Errors
/// [`place`]'s, then [`Placed::verify`]'s.
pub fn commission(
    state_root: &Path,
    active: Active<'_>,
    deadline: Instant,
) -> Result<Commissioned, CommissionError> {
    place(state_root, active, deadline)?.verify(deadline)
}

/// Place the state root at `state_root` for `active` (OPS-1 steps 1-6): the root is born complete. A
/// 0700 directory is staged beside it, the ledger created in it through the store's one create door
/// ([`Store::open`] with `create`, migrations applied to `CURRENT`), the manifest written LAST
/// through the custody door, and the stage verified complete — the manifest through
/// [`read_manifest`], the ledger through [`Store::open_inspection`], the modes — BEFORE it is
/// renamed onto the root only if nothing stands there (`place_staged`, `RENAME_NOREPLACE`; block R
/// F2/M2). A crash leaves only an inert `.staged` directory; a refusal before the rename removes
/// the stage.
///
/// The caller holds IPC01 custody (`control_socket::prepare`) across the call, so no engine serves
/// while a root is commissioned.
///
/// # Errors
/// [`CommissionError::Exists`] when anything stands at `state_root`; [`CommissionError::NotCanonical`]
/// for a relative root, one with no parent, or a path through a link; [`CommissionError::Deadline`]
/// for a spent deadline; [`CommissionError::Store`] for the store's refusal;
/// [`CommissionError::Io`] for a directory, manifest or rename that failed;
/// [`CommissionError::ReadBack`] when the stage does not read back as written — each with nothing
/// placed; [`CommissionError::Placed`] when the root was placed and its parent sync failed.
pub fn place(
    state_root: &Path,
    active: Active<'_>,
    deadline: Instant,
) -> Result<Placed, CommissionError> {
    let root = state_root.to_path_buf();
    let (Some(parent), Some(name)) = (
        state_root.parent(),
        state_root.file_name().and_then(|name| name.to_str()),
    ) else {
        return Err(CommissionError::NotCanonical(root));
    };
    if !state_root.is_absolute() {
        return Err(CommissionError::NotCanonical(root));
    }
    // 1. The parent must be its own canonical path. Its deepest ancestor that exists is judged
    //    BEFORE anything is created, so a link on the way refuses with nothing made through it
    //    (block R L5); the missing ancestors are then created (at the umask's default: RC02
    //    constrains the root, not `~/.local`) and the parent judged again.
    let existing = parent
        .ancestors()
        .find(|ancestor| std::fs::symlink_metadata(ancestor).is_ok())
        .ok_or_else(|| CommissionError::NotCanonical(root.clone()))?;
    for judged in [Some(existing), None] {
        if judged.is_none() {
            std::fs::create_dir_all(parent).map_err(|error| CommissionError::Io(error.kind()))?;
        }
        let judged = judged.unwrap_or(parent);
        let resolved = judged
            .canonicalize()
            .map_err(|error| CommissionError::Io(error.kind()))?;
        if resolved != judged {
            return Err(CommissionError::NotCanonical(root));
        }
    }
    // 2. Nothing may stand at the root: `read_manifest` cannot decide this (it says `Absent` for a
    //    root with no manifest), so the custody door's own open does. A directory, a file or a link
    //    (even a dangling one: `NOFOLLOW`) is `Exists`.
    match PrivateDirectory::open(state_root) {
        Err(DirectoryError::NotFound) => {}
        Err(DirectoryError::Io(error)) => return Err(CommissionError::Io(error.kind())),
        Ok(_) | Err(DirectoryError::Custody) => return Err(CommissionError::Exists(root)),
    }
    // 3. The stage beside the root, private, named by the generation it will hold.
    let staged = parent.join(format!(".{name}.{}.staged", active.generation.as_str()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&staged)
        .map_err(|error| CommissionError::Io(error.kind()))?;
    // 4-6. Filled, verified, and only then placed.
    let ids = (active.generation.as_str(), active.epoch.as_str());
    let verified = stage(&staged, active, deadline)
        .and_then(|()| read_back(&staged, ids, deadline).map_err(CommissionError::from));
    let placed = verified.and_then(|verified| match place_staged(&verified.root, state_root) {
        Ok(()) => Ok(Ok(verified)),
        Err(Unplaced::Rename(error)) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(CommissionError::Exists(root.clone()))
        }
        Err(Unplaced::Rename(error)) => Err(CommissionError::Io(error.kind())),
        Err(Unplaced::ParentSync(error)) => Ok(Err(CommissionError::Placed {
            root: root.clone(),
            generation: active.generation.as_str().to_owned(),
            epoch: active.epoch.as_str().to_owned(),
            why: PostPlacement::ParentSync(error.kind()),
        })),
    });
    match placed {
        Ok(Ok(staged)) => Ok(Placed {
            root,
            generation: active.generation.as_str().to_owned(),
            epoch: active.epoch.as_str().to_owned(),
            staged,
        }),
        // Placed: the stage is the root now, and nothing is removed.
        Ok(Err(placed)) => Err(placed),
        Err(refused) => {
            // Best effort: the stage is never read or renamed once refused, so a leftover is inert.
            let _ = std::fs::remove_dir_all(&staged);
            Err(refused)
        }
    }
}

/// OPS-1 steps 4-5: the ledger created in the stage through the store's one create door, and the
/// manifest written LAST through the custody door.
fn stage(staged: &Path, active: Active<'_>, deadline: Instant) -> Result<(), CommissionError> {
    drop(
        Store::open(staged, active.generation, active.epoch, true, deadline)
            .map_err(store_refused)?,
    );
    let manifest = render_manifest(active)
        .map_err(|error| CommissionError::Io(std::io::Error::from(error).kind()))?;
    PrivateDirectory::open(staged)
        .map_err(|error| match error {
            DirectoryError::NotFound => CommissionError::Io(std::io::ErrorKind::NotFound),
            DirectoryError::Custody => CommissionError::ReadBack("stage custody"),
            DirectoryError::Io(error) => CommissionError::Io(error.kind()),
        })?
        .create_new(ACTIVE_MANIFEST, &manifest)
        .map_err(|error| match error {
            FileError::Io(error) => CommissionError::Io(error.kind()),
            FileError::NotFound => CommissionError::Io(std::io::ErrorKind::NotFound),
            FileError::Custody | FileError::TooLarge => {
                CommissionError::Io(std::io::ErrorKind::AlreadyExists)
            }
        })
}

/// Why a root did not read back as written.
enum Unread {
    /// The operator's deadline was spent before the ledger re-opened.
    Deadline,
    /// The store refused to re-open the ledger, by its error's `Debug` rendering.
    Store(String),
    /// A mode could not be read, by its kind.
    Io(std::io::ErrorKind),
    /// What was read differs from what was written, by what differs.
    Differs(&'static str),
}

impl From<Unread> for CommissionError {
    /// A stage that did not read back, before its rename: nothing was placed.
    fn from(unread: Unread) -> Self {
        match unread {
            Unread::Deadline => Self::Deadline,
            Unread::Store(why) => Self::Store(why),
            Unread::Io(kind) => Self::Io(kind),
            Unread::Differs(which) => Self::ReadBack(which),
        }
    }
}

/// A root read back through the readers `serve` uses — the stage before its rename, the placed root
/// after — against the `(generation, epoch)` written, and every value it reports taken from it: the
/// ids from [`read_manifest`], the version from a store open at `root`'s own path, the modes from
/// `root`'s paths (the reader has already refused any but 0700 and 0600).
fn read_back(
    root: &Path,
    (generation, epoch): (&str, &str),
    deadline: Instant,
) -> Result<Commissioned, Unread> {
    let manifest = read_manifest(root).map_err(|unselected| match unselected {
        Unselected::Absent => Unread::Differs("manifest absent"),
        Unselected::Custody => Unread::Differs("manifest custody"),
        Unselected::Malformed => Unread::Differs("manifest malformed"),
    })?;
    let selected = manifest
        .active()
        .map_err(|_| Unread::Differs("manifest malformed"))?;
    if (selected.generation.as_str(), selected.epoch.as_str()) != (generation, epoch) {
        return Err(Unread::Differs("manifest ids"));
    }
    let user_version = Store::open_inspection(root, selected.generation, selected.epoch, deadline)
        .map_err(|error| match error {
            crate::store::Error::Deadline => Unread::Deadline,
            other => Unread::Store(format!("{other:?}")),
        })?
        .schema_version();
    let mode = |path: &Path| {
        std::fs::symlink_metadata(path)
            .map(|meta| meta.mode() & 0o777)
            .map_err(|error| Unread::Io(error.kind()))
    };
    Ok(Commissioned {
        root: root.to_path_buf(),
        generation: selected.generation.as_str().to_owned(),
        epoch: selected.epoch.as_str().to_owned(),
        user_version,
        root_mode: mode(root)?,
        manifest_mode: mode(&root.join(ACTIVE_MANIFEST))?,
    })
}

/// A store refusal as commissioning names it: a spent deadline by its own name.
fn store_refused(error: crate::store::Error) -> CommissionError {
    match error {
        crate::store::Error::Deadline => CommissionError::Deadline,
        other => CommissionError::Store(format!("{other:?}")),
    }
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
