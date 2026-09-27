//! The operator's backup target (OPS-2; RC01 "Backup freshness", RC02 "Backups"; HO-03 as amended):
//! where `serve` writes the backups RC01 requires before dispatch, read once at start from the
//! operator's private record `<config root>/backup/backup.json`, and the door through which the
//! free space of the two filesystems RC01 reserves is measured.
//!
//! The record has no defaults: its schema, destination and deadline are each required, an unknown
//! field is refused, and a zero deadline is malformed (HO-03: "the backup deadline is a required
//! config field with no default"; the R22-4 precedent). The destination must be the operator's
//! private 0700 directory, its own canonical path, and on another device than the state root
//! (RC02: "separate local device") — which also refuses an unmounted `/var/mnt/STORAGE-10TB`
//! (`nofail` in fstab), whose empty mount point would otherwise hold backups on the state root's
//! own filesystem. The state root's device is passed in by the caller, so which device the rule
//! compares against is chosen by an argument, never by arranging the machine (F95).

use crate::app::custody::{DirectoryError, FileError, PrivateDirectory};
use serde::Deserialize;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The directory under the configuration root holding the record.
pub const BACKUP_DIRECTORY: &str = "backup";
/// The record's file name.
pub const BACKUP_FILE: &str = "backup.json";
/// The schema the record declares.
pub const BACKUP_SCHEMA: &str = "hee3.backup-target/1";
/// The largest record read.
pub const MAX_BACKUP_BYTES: u64 = 4096;

/// Where `serve`'s backups go, and the operator's deadline for each.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupTarget {
    /// The operator's private directory each backup is a fresh `<backup-id>/` child of.
    pub destination: PathBuf,
    /// Each backup's deadline, from the record (no default).
    pub deadline: Duration,
    /// The state root whose filesystem's headroom is checked beside the destination's.
    pub state_root: PathBuf,
}

/// Why there is no backup target: each refusal by its own name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackupUnready {
    /// No record, nor its directory: nothing is configured.
    Absent,
    /// The record's directory or file is not the operator's private one, or could not be read.
    Custody,
    /// The record is not one `hee3.backup-target/1` record with every field present and a
    /// positive deadline, or is larger than [`MAX_BACKUP_BYTES`].
    Malformed,
    /// The destination is relative, or not its own canonical path.
    NotCanonical,
    /// Nothing is at the destination.
    DestinationAbsent,
    /// The destination is not the operator's private 0700 directory.
    DestinationCustody,
    /// The destination is on the state root's own device (RC02: a separate local device).
    SameDevice { state: u64, destination: u64 },
}

impl BackupUnready {
    /// The refusal's name, whole.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Custody => "custody",
            Self::Malformed => "malformed",
            Self::NotCanonical => "not canonical",
            Self::DestinationAbsent => "destination absent",
            Self::DestinationCustody => "destination custody",
            Self::SameDevice { .. } => "same device",
        }
    }

    /// The refusal as a line says it: its name, and both devices for [`BackupUnready::SameDevice`].
    #[must_use]
    pub fn line(self) -> String {
        match self {
            Self::SameDevice { state, destination } => {
                format!("{} (state={state} destination={destination})", self.name())
            }
            other => other.name().to_owned(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record<'r> {
    schema: &'r str,
    destination: &'r str,
    deadline_seconds: u64,
}

/// Read the operator's backup target from `directory` (`<config root>/backup`), for the state root
/// at `state_root` on device `state_dev`.
///
/// # Errors
/// Each [`BackupUnready`] variant, as named there.
pub fn read_target(
    directory: &Path,
    state_root: &Path,
    state_dev: u64,
) -> Result<BackupTarget, BackupUnready> {
    let held = match PrivateDirectory::open(directory) {
        Ok(held) => held,
        Err(DirectoryError::NotFound) => return Err(BackupUnready::Absent),
        Err(DirectoryError::Custody | DirectoryError::Io(_)) => return Err(BackupUnready::Custody),
    };
    let bytes = match held.read(BACKUP_FILE, MAX_BACKUP_BYTES) {
        Ok(bytes) => bytes,
        Err(FileError::NotFound) => return Err(BackupUnready::Absent),
        Err(FileError::TooLarge) => return Err(BackupUnready::Malformed),
        Err(FileError::Custody | FileError::Io(_)) => return Err(BackupUnready::Custody),
    };
    let record: Record<'_> =
        serde_json::from_slice(&bytes).map_err(|_| BackupUnready::Malformed)?;
    if record.schema != BACKUP_SCHEMA || record.deadline_seconds == 0 {
        return Err(BackupUnready::Malformed);
    }
    let destination = Path::new(record.destination);
    if !destination.is_absolute() {
        return Err(BackupUnready::NotCanonical);
    }
    match destination.canonicalize() {
        Ok(resolved) if resolved == destination => {}
        Ok(_) => return Err(BackupUnready::NotCanonical),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(BackupUnready::DestinationAbsent);
        }
        Err(_) => return Err(BackupUnready::DestinationCustody),
    }
    let meta = match PrivateDirectory::open(destination) {
        Ok(_) => std::fs::metadata(destination).map_err(|_| BackupUnready::DestinationCustody)?,
        Err(DirectoryError::NotFound) => return Err(BackupUnready::DestinationAbsent),
        Err(DirectoryError::Custody | DirectoryError::Io(_)) => {
            return Err(BackupUnready::DestinationCustody);
        }
    };
    if meta.dev() == state_dev {
        return Err(BackupUnready::SameDevice {
            state: state_dev,
            destination: meta.dev(),
        });
    }
    Ok(BackupTarget {
        destination: destination.to_path_buf(),
        deadline: Duration::from_secs(record.deadline_seconds),
        state_root: state_root.to_path_buf(),
    })
}

/// How the free space of a filesystem is measured: the seam the headroom rule is reached through,
/// so the rule is provable by choosing the numbers (F95) — no test world here holds RC01's 96 GiB
/// and 256 GiB reserves on two devices.
pub trait FreeSpace {
    /// The bytes available to this user on the filesystem holding `path`.
    ///
    /// # Errors
    /// When the filesystem cannot be measured.
    fn free(&self, path: &Path) -> std::io::Result<u64>;
}

/// The production measurement: `fstatvfs` on the path opened as a directory, `f_bavail` blocks of
/// `f_frsize` bytes — the space an unprivileged writer can use.
#[derive(Clone, Copy, Debug, Default)]
pub struct Statvfs;

impl FreeSpace for Statvfs {
    fn free(&self, path: &Path) -> std::io::Result<u64> {
        let directory = std::fs::File::open(path)?;
        let stat = rustix::fs::fstatvfs(&directory)?;
        Ok(stat.f_bavail.saturating_mul(stat.f_frsize))
    }
}
