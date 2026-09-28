//! The operator's backup target (OPS-2; RC01 "Backup freshness", RC01 "Persistent capacity", RC02
//! "Backups"; HO-03 as amended): where `serve` writes the backups RC01 requires before dispatch, read
//! once at start from the operator's private record `<config root>/backup/backup.json`, and the door
//! through which the free space of the two filesystems RC01 reserves, and the destination's existing
//! backup usage RC01 budgets, are measured.
//!
//! The record has no defaults: its schema, destination and deadline are each required, an unknown
//! field is refused, and a zero deadline is malformed (HO-03: "the backup deadline is a required
//! config field with no default"; the R22-4 precedent). The destination must be the operator's
//! private 0700 directory, its own canonical path, and on another device than the state root (RC02:
//! "separate local device"). The device rule is decided over the mount SOURCE of each path's
//! containing mount in the mount table (`/proc/self/mountinfo`: the longest mount point that is a
//! component-wise prefix of the canonical path), never over `st_dev`, which names a btrfs subvolume
//! rather than a device: on this host `/var` and `/var/home` are two subvolumes of one LUKS device
//! with two `st_dev` values (block R F1/F-H1/H1). Equal sources, an equal filesystem, a filesystem
//! with no backing device (tmpfs, ramfs, overlay, or a source that is not a device path) and a path
//! no mount resolves are each refused by name. The table is an argument, so which mounts the rule
//! decides over is chosen by the caller, never by arranging the machine (F95). An unmounted
//! `/var/mnt/STORAGE-10TB` (`nofail` in fstab) is refused by `DestinationAbsent` (its subdirectory
//! is missing) and, were the subdirectory present on the bare mount point, by the device rule: the
//! bare mount point's containing mount is then the state root's own device.

use crate::app::custody::{DirectoryError, FileError, PrivateDirectory};
use serde::Deserialize;
use std::ffi::OsString;
use std::io::Read;
use std::os::unix::ffi::OsStringExt;
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
/// The mount table the device rule reads in production.
pub const MOUNT_TABLE: &str = "/proc/self/mountinfo";
/// The largest mount table read: read up to this many bytes and refused by name past it, a limit at
/// the point of acquisition (this toolbox's table is 49,035 bytes, the host's 4,504; measured
/// 2026-09-28).
pub const MAX_MOUNT_TABLE_BYTES: u64 = 1 << 20;
/// The most directory entries the usage walk visits under the destination before it refuses by
/// name (the walk's acquisition bound: one /1 backup holds at most 4096 objects and their fan-out
/// directories, so this is some 250 backups of the largest shape).
pub const USAGE_ENTRY_BOUND: u64 = 1 << 20;

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
    /// RC02's separate-device rule refused, by why.
    Device(DeviceWhy),
}

/// Which of RC02's two paths a device refusal names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    /// The state root.
    State,
    /// The backup destination.
    Destination,
}

impl Side {
    const fn name(self) -> &'static str {
        match self {
            Self::State => "state",
            Self::Destination => "destination",
        }
    }
}

/// Why a filesystem has no backing device RC02 can count as one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoDevice {
    /// `tmpfs`: memory, gone at reboot.
    Tmpfs,
    /// `ramfs`: memory, gone at reboot.
    Ramfs,
    /// `overlay`: a union over other filesystems, none of them named.
    Overlay,
    /// Any other filesystem whose source is not a device path (`none`, `portal`, `host:/export`).
    Source,
}

impl NoDevice {
    const fn name(self) -> &'static str {
        match self {
            Self::Tmpfs => "tmpfs",
            Self::Ramfs => "ramfs",
            Self::Overlay => "overlay",
            Self::Source => "no device source",
        }
    }
}

/// Why the mount table the device rule reads could not be used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableWhy {
    /// It could not be opened or read, by the I/O kind.
    Unreadable(std::io::ErrorKind),
    /// It is larger than [`MAX_MOUNT_TABLE_BYTES`].
    TooLarge,
    /// The line (1-based) is not a mountinfo line.
    Malformed { line: usize },
}

/// Why RC02's separate-device rule refused (block R F1/F-H1/H1), each by its own name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceWhy {
    /// The mount table could not be used.
    Table(TableWhy),
    /// No mount contains the side's path, or the state root could not be resolved to a canonical
    /// path to look up.
    Unresolved(Side),
    /// The side's filesystem has no backing device.
    NoDevice { side: Side, kind: NoDevice },
    /// Both sides' containing mounts name one source: the table's two mount ids.
    SameSource { state: u64, destination: u64 },
    /// Both sides' containing mounts are one filesystem (`major:minor`) under two source names —
    /// a multi-device btrfs mounted by two of its members: the table's two mount ids.
    SameFilesystem { state: u64, destination: u64 },
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
            Self::Device(DeviceWhy::Table(TableWhy::Unreadable(_))) => "mount table unreadable",
            Self::Device(DeviceWhy::Table(TableWhy::TooLarge)) => "mount table too large",
            Self::Device(DeviceWhy::Table(TableWhy::Malformed { .. })) => "mount table malformed",
            Self::Device(DeviceWhy::Unresolved(_)) => "device unresolved",
            Self::Device(DeviceWhy::NoDevice { .. }) => "not a block device",
            Self::Device(DeviceWhy::SameSource { .. } | DeviceWhy::SameFilesystem { .. }) => {
                "same device"
            }
        }
    }

    /// The refusal as a line says it: its name, and for a device refusal what decided it — the I/O
    /// kind, the bound, the line, the side, the filesystem, or both mount ids of the table.
    #[must_use]
    pub fn line(self) -> String {
        let name = self.name();
        match self {
            Self::Device(DeviceWhy::Table(TableWhy::Unreadable(kind))) => {
                format!("{name} ({kind:?})")
            }
            Self::Device(DeviceWhy::Table(TableWhy::TooLarge)) => {
                format!("{name} (over {MAX_MOUNT_TABLE_BYTES} bytes)")
            }
            Self::Device(DeviceWhy::Table(TableWhy::Malformed { line })) => {
                format!("{name} (line {line})")
            }
            Self::Device(DeviceWhy::Unresolved(side)) => format!("{name} ({})", side.name()),
            Self::Device(DeviceWhy::NoDevice { side, kind }) => {
                format!("{name} ({}: {})", side.name(), kind.name())
            }
            Self::Device(DeviceWhy::SameSource { state, destination }) => {
                format!("{name} (one source: state mount {state}, destination mount {destination})")
            }
            Self::Device(DeviceWhy::SameFilesystem { state, destination }) => format!(
                "{name} (one filesystem: state mount {state}, destination mount {destination})"
            ),
            Self::Absent
            | Self::Custody
            | Self::Malformed
            | Self::NotCanonical
            | Self::DestinationAbsent
            | Self::DestinationCustody => name.to_owned(),
        }
    }
}

/// One mount, as the mount table states it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mount {
    /// The table's mount id (its first field): what a refusal names.
    pub id: u64,
    /// The filesystem's `major:minor`.
    pub filesystem: (u32, u32),
    /// Where it is mounted, its escapes decoded.
    pub mount_point: PathBuf,
    /// The filesystem type.
    pub fstype: String,
    /// The mount source, its escapes decoded.
    pub source: OsString,
}

/// The mount table, parsed: every line one [`Mount`], in the table's order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountTable {
    mounts: Vec<Mount>,
}

impl MountTable {
    /// Parse `bytes` as `/proc/self/mountinfo` text: per line, the mount id, the parent id,
    /// `major:minor`, the root, the mount point, the options, any optional fields, a lone `-`, the
    /// filesystem type and the source. The mount point and the source have `\ooo` escapes decoded;
    /// the mount point must be absolute. A final newline ends the last line.
    ///
    /// # Errors
    /// [`TableWhy::Malformed`] naming the first line that is not one.
    pub fn parse(bytes: &[u8]) -> Result<Self, TableWhy> {
        let text = bytes.strip_suffix(b"\n").unwrap_or(bytes);
        let mut mounts = Vec::new();
        if text.is_empty() {
            return Ok(Self { mounts });
        }
        for (index, line) in text.split(|byte| *byte == b'\n').enumerate() {
            mounts.push(parse_line(line).ok_or(TableWhy::Malformed { line: index + 1 })?);
        }
        Ok(Self { mounts })
    }

    /// Read and parse the table at `path`, reading at most [`MAX_MOUNT_TABLE_BYTES`] and refusing
    /// past it (a `/proc` file states no length, so the bound is on the read itself).
    ///
    /// # Errors
    /// [`TableWhy::Unreadable`], [`TableWhy::TooLarge`], or [`MountTable::parse`]'s.
    pub fn read(path: &Path) -> Result<Self, TableWhy> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .and_then(|file| file.take(MAX_MOUNT_TABLE_BYTES + 1).read_to_end(&mut bytes))
            .map_err(|error| TableWhy::Unreadable(error.kind()))?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_MOUNT_TABLE_BYTES {
            return Err(TableWhy::TooLarge);
        }
        Self::parse(&bytes)
    }

    /// The mount `path` is on: the one whose mount point is the longest component-wise prefix of
    /// `path` (`/var/mnt` never contains `/var/mntx`), the later line when two share a mount point
    /// (the one stacked on top is visible). `None` for a relative path or one no mount contains.
    #[must_use]
    pub fn containing(&self, path: &Path) -> Option<&Mount> {
        if !path.is_absolute() {
            return None;
        }
        let mut best: Option<(usize, &Mount)> = None;
        for mount in &self.mounts {
            if path.starts_with(&mount.mount_point) {
                let depth = mount.mount_point.components().count();
                if best.is_none_or(|(deepest, _)| depth >= deepest) {
                    best = Some((depth, mount));
                }
            }
        }
        best.map(|(_, mount)| mount)
    }
}

/// One mountinfo line, or `None` when it is not one.
fn parse_line(line: &[u8]) -> Option<Mount> {
    let fields: Vec<&[u8]> = line.split(|byte| *byte == b' ').collect();
    let separator = fields.iter().skip(6).position(|field| *field == b"-")? + 6;
    let id = std::str::from_utf8(fields.first()?).ok()?;
    let (major, minor) = std::str::from_utf8(fields.get(2)?).ok()?.split_once(':')?;
    let mount_point = PathBuf::from(OsString::from_vec(unescape(fields.get(4)?)?));
    let fstype = String::from_utf8(unescape(fields.get(separator + 1)?)?).ok()?;
    let source = OsString::from_vec(unescape(fields.get(separator + 2)?)?);
    if !mount_point.is_absolute() || fstype.is_empty() || source.is_empty() {
        return None;
    }
    Some(Mount {
        id: decimal(id)?,
        filesystem: (
            u32::try_from(decimal(major)?).ok()?,
            u32::try_from(decimal(minor)?).ok()?,
        ),
        mount_point,
        fstype,
        source,
    })
}

/// A plain decimal: digits only, at least one.
fn decimal(text: &str) -> Option<u64> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// A mountinfo field with its `\ooo` escapes (space, tab, newline, backslash) decoded; `None` for a
/// backslash not followed by three octal digits naming one byte.
fn unescape(field: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Vec::with_capacity(field.len());
    let mut rest = field;
    while let Some((&byte, tail)) = rest.split_first() {
        if byte == b'\\' {
            let digits = tail.get(..3)?;
            if !digits.iter().all(|digit| (b'0'..=b'7').contains(digit)) {
                return None;
            }
            let value = digits
                .iter()
                .fold(0_u32, |value, digit| value * 8 + u32::from(digit - b'0'));
            decoded.push(u8::try_from(value).ok()?);
            rest = tail.get(3..)?;
        } else {
            decoded.push(byte);
            rest = tail;
        }
    }
    Some(decoded)
}

/// What a filesystem's mount says of its device: `None` for a block device RC02 can count.
fn no_device(mount: &Mount) -> Option<NoDevice> {
    match mount.fstype.as_str() {
        "tmpfs" => Some(NoDevice::Tmpfs),
        "ramfs" => Some(NoDevice::Ramfs),
        "overlay" => Some(NoDevice::Overlay),
        _ if !mount.source.as_encoded_bytes().starts_with(b"/") => Some(NoDevice::Source),
        _ => None,
    }
}

/// RC02's separate-device rule, pure (F95): the state root's and the destination's containing
/// mounts (`None`: no mount resolves the path). Each must resolve, each must have a backing device,
/// and they must name neither one source nor one filesystem; the first that does not hold refuses,
/// in that order, the state root before the destination.
///
/// # Errors
/// [`DeviceWhy::Unresolved`], [`DeviceWhy::NoDevice`], [`DeviceWhy::SameSource`] or
/// [`DeviceWhy::SameFilesystem`].
pub fn device_decision(
    state: Option<&Mount>,
    destination: Option<&Mount>,
) -> Result<(), DeviceWhy> {
    let state = state.ok_or(DeviceWhy::Unresolved(Side::State))?;
    let destination = destination.ok_or(DeviceWhy::Unresolved(Side::Destination))?;
    for (side, mount) in [(Side::State, state), (Side::Destination, destination)] {
        if let Some(kind) = no_device(mount) {
            return Err(DeviceWhy::NoDevice { side, kind });
        }
    }
    let mounts = (state.id, destination.id);
    if state.source == destination.source {
        return Err(DeviceWhy::SameSource {
            state: mounts.0,
            destination: mounts.1,
        });
    }
    if state.filesystem == destination.filesystem {
        return Err(DeviceWhy::SameFilesystem {
            state: mounts.0,
            destination: mounts.1,
        });
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record<'r> {
    schema: &'r str,
    destination: &'r str,
    deadline_seconds: u64,
}

/// Read the operator's backup target from `directory` (`<config root>/backup`), for the state root
/// at `state_root`, deciding RC02's device rule over `mounts` (the mount table, or why it could not
/// be read). The state root is resolved to its canonical path before it is looked up: the table
/// names canonical mount points, and on Kinoite `/home/<user>` is a link to `/var/home/<user>`.
///
/// # Errors
/// Each [`BackupUnready`] variant, as named there.
pub fn read_target(
    directory: &Path,
    state_root: &Path,
    mounts: &Result<MountTable, TableWhy>,
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
    match PrivateDirectory::open(destination) {
        Ok(_) => {}
        Err(DirectoryError::NotFound) => return Err(BackupUnready::DestinationAbsent),
        Err(DirectoryError::Custody | DirectoryError::Io(_)) => {
            return Err(BackupUnready::DestinationCustody);
        }
    }
    let table = mounts
        .as_ref()
        .map_err(|why| BackupUnready::Device(DeviceWhy::Table(*why)))?;
    let state = state_root.canonicalize().ok();
    device_decision(
        state.as_deref().and_then(|state| table.containing(state)),
        table.containing(destination),
    )
    .map_err(BackupUnready::Device)?;
    Ok(BackupTarget {
        destination: destination.to_path_buf(),
        deadline: Duration::from_secs(record.deadline_seconds),
        state_root: state_root.to_path_buf(),
    })
}

/// Why the destination's existing backup usage could not be measured (RC01 "128-GiB backup
/// budget").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Usage {
    /// A directory or an entry under it could not be read, by the I/O kind.
    Io(std::io::ErrorKind),
    /// More than `bound` entries lie under the destination: the walk stopped at its bound.
    Bound { bound: u64 },
}

/// How RC01's space is measured: the seam the headroom and budget rules are reached through, so each
/// rule is provable by choosing the numbers (F95) — no test world here holds RC01's 96 GiB and
/// 256 GiB reserves on two devices, or 128 GiB of backups.
pub trait FreeSpace {
    /// The bytes available to this user on the filesystem holding `path`.
    ///
    /// # Errors
    /// When the filesystem cannot be measured.
    fn free(&self, path: &Path) -> std::io::Result<u64>;

    /// The bytes the backups already under `destination` hold (RC01's backup budget).
    ///
    /// # Errors
    /// [`Usage`], by why the usage could not be measured.
    fn used(&self, destination: &Path) -> Result<u64, Usage>;
}

/// The production measurement: `fstatvfs` on the path opened as a directory, `f_bavail` blocks of
/// `f_frsize` bytes — the space an unprivileged writer can use; and the destination's usage by
/// [`backup_usage`] under [`USAGE_ENTRY_BOUND`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Statvfs;

impl FreeSpace for Statvfs {
    fn free(&self, path: &Path) -> std::io::Result<u64> {
        let directory = std::fs::File::open(path)?;
        let stat = rustix::fs::fstatvfs(&directory)?;
        Ok(stat.f_bavail.saturating_mul(stat.f_frsize))
    }

    fn used(&self, destination: &Path) -> Result<u64, Usage> {
        backup_usage(destination, USAGE_ENTRY_BOUND)
    }
}

/// The bytes the regular files under `destination` hold, by their length, walked without following
/// a link: the usage RC01's backup budget is measured in (the store's backup door writes only
/// regular files and directories). The walk visits at most `bound` entries and refuses past it,
/// naming the bound, so no directory the operator filled can make it unbounded.
///
/// # Errors
/// [`Usage::Io`] for a directory or entry that could not be read; [`Usage::Bound`] past `bound`.
pub fn backup_usage(destination: &Path, bound: u64) -> Result<u64, Usage> {
    let io = |error: std::io::Error| Usage::Io(error.kind());
    let mut pending = vec![destination.to_path_buf()];
    let (mut entries, mut bytes) = (0_u64, 0_u64);
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).map_err(io)? {
            let entry = entry.map_err(io)?;
            entries = entries.saturating_add(1);
            if entries > bound {
                return Err(Usage::Bound { bound });
            }
            // `DirEntry::metadata` does not follow a link: a link is neither walked nor counted.
            let meta = entry.metadata().map_err(io)?;
            if meta.is_dir() {
                pending.push(entry.path());
            } else if meta.is_file() {
                bytes = bytes.saturating_add(meta.len());
            }
        }
    }
    Ok(bytes)
}

/// Free space declared, not measured (RA1 b): the state root's filesystem answers `state`, the
/// destination's answers `backup`, and any other path is refused by name, so no verdict can rest on
/// a path nothing declared (F101). Only a test build's `serve` composes one, from
/// `HEE3_TEST_HEADROOM` (feature `headroom-seam`, which only the package's dev-dependency on itself
/// turns on); a release build of `habitat-engine` has no door to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declared {
    state_root: PathBuf,
    destination: PathBuf,
    state: u64,
    backup: u64,
}

impl Declared {
    /// `value` as exactly `state=<bytes> backup=<bytes>` (each a decimal with no leading zero), for
    /// `target`'s two filesystems; `None` for anything else.
    #[must_use]
    pub fn parse(value: &str, target: &BackupTarget) -> Option<Self> {
        let (state, backup) = value.split_once(' ')?;
        let number = |text: &str, name: &str| {
            text.strip_prefix(name)
                .and_then(|digits| crate::contracts::parse_u64_decimal(digits).ok())
        };
        Some(Self {
            state_root: target.state_root.clone(),
            destination: target.destination.clone(),
            state: number(state, "state=")?,
            backup: number(backup, "backup=")?,
        })
    }
}

impl FreeSpace for Declared {
    /// The destination's usage is measured, never declared: a test world's destination is a real,
    /// small directory, walked as `serve` walks it. Any other path is refused like [`Declared::free`].
    fn used(&self, destination: &Path) -> Result<u64, Usage> {
        if destination == self.destination {
            backup_usage(destination, USAGE_ENTRY_BOUND)
        } else {
            Err(Usage::Io(std::io::ErrorKind::NotFound))
        }
    }

    fn free(&self, path: &Path) -> std::io::Result<u64> {
        if path == self.state_root {
            Ok(self.state)
        } else if path == self.destination {
            Ok(self.backup)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no free space is declared for {}", path.display()),
            ))
        }
    }
}
