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
use crate::store::RecoveryLimits;
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
#[derive(Debug, Eq, PartialEq)]
pub enum Unselected {
    /// There is no manifest: nothing has been commissioned.
    Absent,
    /// The state root or the manifest is not the operator's private one.
    Custody,
    /// The manifest is not one `hee3.active-generation/1` record.
    Malformed,
}

/// The generation and epoch the manifest selects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Active {
    /// The ledger generation directory under `generations/`.
    pub generation: String,
    /// The ledger epoch it must carry.
    pub epoch: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    generation: String,
    epoch: String,
}

/// Read the active-generation manifest under `state_root`.
///
/// # Errors
///
/// [`Unselected::Absent`] when there is no state root or manifest; [`Unselected::Custody`] for a
/// root that is not the operator's 0700 directory or a manifest that is not their 0600 regular
/// file (a link is not followed); [`Unselected::Malformed`] for anything else.
pub fn active(state_root: &Path) -> Result<Active, Unselected> {
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
    let record: Manifest = serde_json::from_slice(&bytes).map_err(|_| Unselected::Malformed)?;
    if record.schema != ACTIVE_SCHEMA
        || UuidV4::parse(&record.generation).is_err()
        || UuidV4::parse(&record.epoch).is_err()
    {
        return Err(Unselected::Malformed);
    }
    Ok(Active {
        generation: record.generation,
        epoch: record.epoch,
    })
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

/// Select the active generation under `state_root` and reconcile it; the observation `health`
/// serves, and a line naming why when it is not ready.
#[must_use]
pub fn observe_at_start(
    state_root: &Path,
    checked_unix_ms: u64,
    deadline: Instant,
) -> (Health, String) {
    let selected = match active(state_root) {
        Ok(selected) => selected,
        Err(unselected) => {
            let why = format!(
                "no active generation at {} ({unselected:?})",
                state_root.display()
            );
            return (health_of(Err(&why), checked_unix_ms), why);
        }
    };
    let (Ok(generation), Ok(epoch)) = (
        UuidV4::parse(&selected.generation),
        UuidV4::parse(&selected.epoch),
    ) else {
        let why = "the manifest's identities changed after validation".to_owned();
        return (health_of(Err(&why), checked_unix_ms), why);
    };
    let startup = startup::Startup {
        root: state_root,
        generation,
        epoch,
        limits: START_LIMITS,
        restored_from: None,
        cursors: &[],
        claims: &[],
        deadline,
    };
    let mut host = Host::new(deadline);
    match startup::run(&startup, &mut host) {
        Ok(pass) => {
            let health = health_of(Ok(&pass), checked_unix_ms);
            let line = format!(
                "generation {} reconciled: attempts={} writes={} recovery={} database={}",
                pass.generation,
                pass.attempts.len(),
                pass.writes,
                health.recovery.name(),
                health.database.name()
            );
            (health, line)
        }
        Err(error) => {
            let why = format!("startup refused: {error:?}");
            (health_of(Err(&why), checked_unix_ms), why)
        }
    }
}

/// The state root under `home`.
#[must_use]
pub fn state_root(home: &Path) -> PathBuf {
    home.join(STATE_DIRECTORY)
}

/// Open the active generation's ledger writable for the task owner, after startup reconciled it.
///
/// # Errors
///
/// A line naming why no writable ledger could be composed; the caller serves task actions as
/// `unavailable` and says so.
pub fn compose_tasks(
    state_root: &Path,
    deadline: Instant,
) -> Result<crate::app::tasks::StoreTasks, String> {
    let selected = active(state_root)
        .map_err(|unselected| format!("no active generation ({unselected:?})"))?;
    let generation = UuidV4::parse(&selected.generation)
        .map_err(|_| "the manifest generation changed".to_owned())?;
    let epoch =
        UuidV4::parse(&selected.epoch).map_err(|_| "the manifest epoch changed".to_owned())?;
    let store = crate::store::Store::open(state_root, generation, epoch, false, deadline)
        .map_err(|error| format!("the ledger did not open writable: {error:?}"))?;
    Ok(crate::app::tasks::StoreTasks::new(store, selected.epoch))
}
