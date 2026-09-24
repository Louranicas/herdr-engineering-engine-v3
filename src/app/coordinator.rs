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

use crate::app::startup::{self, Host, LedgerAccess, Pass};
use crate::contracts::UuidV4;
use crate::contracts::control::{Database, Health, Recovery, Socket};
use crate::recovery::{Mode, Reconciliation};
use crate::store::{RecoveryLimits, Store};
use rustix::fs::{Mode as FileMode, OFlags};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The operator's state root, under their home (RC02).
pub const STATE_DIRECTORY: &str = ".local/state/herdr-engineering-engine-v3";
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
            let line = format!(
                "generation {} reconciled: attempts={} writes={} recovery={} database={}",
                pass.generation,
                pass.attempts.len(),
                pass.writes,
                health.recovery.name(),
                health.database.name()
            );
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
