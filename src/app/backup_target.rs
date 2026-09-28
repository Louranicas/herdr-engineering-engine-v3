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
//! no mount resolves are each refused by name. RC02's separate device exists to survive a disk
//! failure (`contract-decisions.md` RC02), so the rule then resolves each side's mount source to the
//! top-level physical disks under it through the block topology sysfs states (`/sys/class/block`:
//! a partition to its parent disk, a device-mapper, LVM or md device through its `slaves`) and
//! refuses two sides sharing a disk, and a source the topology cannot carry to a physical disk (a
//! loop or zram device, an unlisted parent), by name (OPS12 round 2, R2-1). The source is the key,
//! never the table's `major:minor`: a btrfs mount states an anonymous `0:N` that
//! `/sys/dev/block` does not list (measured on this host, 2026-09-29: `0:35`). A multi-device
//! btrfs resolves only the member its source names (`/sys/fs/btrfs/<uuid>/devices` is not read);
//! two mounts of one such filesystem are still refused as one filesystem. The table and the
//! topology are arguments, so which mounts and disks the rule decides over is chosen by the caller,
//! never by arranging the machine (F95). An unmounted
//! `/var/mnt/STORAGE-10TB` (`nofail` in fstab) is refused by `DestinationAbsent` (its subdirectory
//! is missing) and, were the subdirectory present on the bare mount point, by the device rule: the
//! bare mount point's containing mount is then the state root's own device.

use crate::app::custody::{DirectoryError, FileError, PrivateDirectory};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
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
/// Where the block topology is read in production: sysfs, whose `class/block` lists every block
/// device.
pub const SYSFS: &str = "/sys";
/// The most block devices the topology lists, and the most `slaves` one device lists, before it is
/// refused by name: a bound at acquisition (this host lists 16, measured 2026-09-29).
pub const MAX_BLOCK_DEVICES: u64 = 4096;
/// The largest block topology text read (a test build's declared topology).
pub const MAX_TOPOLOGY_BYTES: u64 = 1 << 20;
/// The largest device-mapper name read from `dm/name` (the kernel's `DM_NAME_LEN` is 128).
const MAX_DM_NAME_BYTES: u64 = 256;

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

/// Why the block topology the device rule reads could not be used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopologyWhy {
    /// A sysfs entry or the declared file could not be opened or read, by the I/O kind
    /// (`InvalidData` for a device name that is not UTF-8 or a device-mapper name over its bound).
    Unreadable(std::io::ErrorKind),
    /// More block devices, or more `slaves` of one, than the reader's bound.
    TooMany { bound: u64 },
    /// The declared text is larger than [`MAX_TOPOLOGY_BYTES`].
    TooLarge,
    /// The line (1-based) is not a topology line, or names a device a second time.
    Malformed { line: usize },
}

impl TopologyWhy {
    const fn name(self) -> &'static str {
        match self {
            Self::Unreadable(_) => "block topology unreadable",
            Self::TooMany { .. } | Self::TooLarge => "block topology too large",
            Self::Malformed { .. } => "block topology malformed",
        }
    }

    fn line(self) -> String {
        let name = self.name();
        match self {
            Self::Unreadable(kind) => format!("{name} ({kind:?})"),
            Self::TooMany { bound } => format!("{name} (over {bound} devices)"),
            Self::TooLarge => format!("{name} (over {MAX_TOPOLOGY_BYTES} bytes)"),
            Self::Malformed { line } => format!("{name} (line {line})"),
        }
    }
}

/// Why a mount source could not be carried to the physical disks under it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unresolvable {
    /// The source names no block device the topology lists (`/dev/<name>` or
    /// `/dev/mapper/<dm name>`, nothing else).
    Source,
    /// A device with no physical disk under it that sysfs names: a `devices/virtual` device with no
    /// `slaves` (loop, zram, ram, nbd, a device-mapper target over nothing).
    Virtual,
    /// A partition's disk or a stacked device's slave that the topology does not list.
    Unlisted,
    /// A partition whose link names no parent directory, so no disk.
    NoParent,
    /// A chain of stacked devices that reaches no disk (a cycle).
    NoDisk,
}

impl Unresolvable {
    const fn name(self) -> &'static str {
        match self {
            Self::Source => "no block device",
            Self::Virtual => "virtual device",
            Self::Unlisted => "unlisted device",
            Self::NoParent => "partition without a disk",
            Self::NoDisk => "no disk",
        }
    }
}

/// Why RC02's separate-device rule refused (block R F1/F-H1/H1), each by its own name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceWhy {
    /// The mount table could not be used.
    Table(TableWhy),
    /// The block topology could not be used.
    Topology(TopologyWhy),
    /// The side's mount source could not be resolved to a physical disk (OPS12 round 2, R2-1).
    DiskUnresolved { side: Side, why: Unresolvable },
    /// Both sides' mount sources rest on one physical disk (OPS12 round 2, R2-1): the table's two
    /// mount ids.
    SameDisk { state: u64, destination: u64 },
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
            Self::Device(DeviceWhy::Topology(why)) => why.name(),
            Self::Device(DeviceWhy::Unresolved(_)) => "device unresolved",
            Self::Device(DeviceWhy::DiskUnresolved { .. }) => "disk unresolved",
            Self::Device(DeviceWhy::NoDevice { .. }) => "not a block device",
            Self::Device(
                DeviceWhy::SameSource { .. }
                | DeviceWhy::SameFilesystem { .. }
                | DeviceWhy::SameDisk { .. },
            ) => "same device",
        }
    }

    /// The refusal as a line says it: its name, and for a device refusal what decided it — the I/O
    /// kind, the bound, the line, the side, the filesystem or why its disk is unresolved, or both
    /// mount ids of the table.
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
            Self::Device(DeviceWhy::Topology(why)) => why.line(),
            Self::Device(DeviceWhy::Unresolved(side)) => format!("{name} ({})", side.name()),
            Self::Device(DeviceWhy::DiskUnresolved { side, why }) => {
                format!("{name} ({}: {})", side.name(), why.name())
            }
            Self::Device(DeviceWhy::SameDisk { state, destination }) => {
                format!("{name} (one disk: state mount {state}, destination mount {destination})")
            }
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

/// One block device as sysfs states it under `class/block/<name>`: facts only, read as they are, so
/// every judgement over them is the pure resolver's (F95).
#[derive(Clone, Debug, Eq, PartialEq)]
struct BlockDevice {
    /// The class link's target as `readlink` returns it (`../../devices/virtual/block/dm-0`): a
    /// partition's parent directory is its disk, and a `devices/virtual` path has no hardware.
    link: PathBuf,
    /// Whether it has a `partition` attribute.
    partition: bool,
    /// Its device-mapper name (`dm/name`), which a `/dev/mapper/<name>` source names.
    dm_name: Option<OsString>,
    /// Its `slaves`: the devices it is stacked on (dm-crypt, LVM, md).
    slaves: Vec<String>,
}

/// The block topology RC02's disk rule reads: every block device sysfs lists, by its kernel name.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Topology {
    devices: BTreeMap<String, BlockDevice>,
}

impl Topology {
    /// Parse `bytes` as a declared topology, one device per line as sysfs states it:
    /// `<name> <link> <whole|part> [dm:<dm name>] [slave:<name>]...`, fields separated by one space
    /// and each `\ooo`-escaped as the mount table's are. At most `bound` devices, and a name given
    /// twice is malformed. A final newline ends the last line.
    ///
    /// # Errors
    /// [`TopologyWhy::Malformed`] naming the first line that is not one; [`TopologyWhy::TooMany`]
    /// past `bound`.
    pub fn parse(bytes: &[u8], bound: u64) -> Result<Self, TopologyWhy> {
        let text = bytes.strip_suffix(b"\n").unwrap_or(bytes);
        let mut devices = BTreeMap::new();
        if text.is_empty() {
            return Ok(Self { devices });
        }
        for (index, line) in text.split(|byte| *byte == b'\n').enumerate() {
            if u64::try_from(devices.len()).unwrap_or(u64::MAX) >= bound {
                return Err(TopologyWhy::TooMany { bound });
            }
            let malformed = TopologyWhy::Malformed { line: index + 1 };
            let (name, device) = parse_device(line).ok_or(malformed)?;
            if devices.insert(name, device).is_some() {
                return Err(malformed);
            }
        }
        Ok(Self { devices })
    }

    /// Read and parse the declared topology at `path` under [`MAX_TOPOLOGY_BYTES`] and
    /// [`MAX_BLOCK_DEVICES`], each refused by name past it.
    ///
    /// # Errors
    /// [`TopologyWhy::Unreadable`], [`TopologyWhy::TooLarge`], or [`Topology::parse`]'s.
    pub fn read_text(path: &Path) -> Result<Self, TopologyWhy> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .and_then(|file| file.take(MAX_TOPOLOGY_BYTES + 1).read_to_end(&mut bytes))
            .map_err(|error| TopologyWhy::Unreadable(error.kind()))?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_TOPOLOGY_BYTES {
            return Err(TopologyWhy::TooLarge);
        }
        Self::parse(&bytes, MAX_BLOCK_DEVICES)
    }

    /// Read the topology from sysfs at `sysfs` (production: [`SYSFS`]): each entry of
    /// `class/block`, its link as `readlink` returns it, whether it has a `partition` attribute,
    /// its `dm/name` and its `slaves` — at most `bound` devices, and `bound` slaves of one,
    /// refused by name past it before the next is acquired. Nothing here judges a device.
    ///
    /// # Errors
    /// [`TopologyWhy::Unreadable`] by the I/O kind; [`TopologyWhy::TooMany`] past `bound`.
    pub fn read(sysfs: &Path, bound: u64) -> Result<Self, TopologyWhy> {
        let io = |error: std::io::Error| TopologyWhy::Unreadable(error.kind());
        let class = sysfs.join("class/block");
        let mut devices = BTreeMap::new();
        for entry in std::fs::read_dir(&class).map_err(io)? {
            let entry = entry.map_err(io)?;
            if u64::try_from(devices.len()).unwrap_or(u64::MAX) >= bound {
                return Err(TopologyWhy::TooMany { bound });
            }
            let name = utf8(entry.file_name())?;
            let path = entry.path();
            let device = BlockDevice {
                link: std::fs::read_link(&path).map_err(io)?,
                partition: present(&path.join("partition"))?,
                dm_name: dm_name(&path.join("dm/name"))?,
                slaves: slaves(&path.join("slaves"), bound)?,
            };
            devices.insert(name, device);
        }
        Ok(Self { devices })
    }

    /// The kernel name of the block device a mount `source` names: `/dev/mapper/<dm name>` by its
    /// device-mapper name (one device only), `/dev/<name>` by its own name; nothing else.
    fn device_of(&self, source: &OsStr) -> Option<&str> {
        let bytes = source.as_encoded_bytes();
        if let Some(mapped) = bytes.strip_prefix(b"/dev/mapper/") {
            let mut named = self.devices.iter().filter(|(_, device)| {
                device
                    .dm_name
                    .as_ref()
                    .is_some_and(|name| name.as_encoded_bytes() == mapped)
            });
            let (name, _) = named.next()?;
            return named.next().is_none().then_some(name.as_str());
        }
        let name = std::str::from_utf8(bytes.strip_prefix(b"/dev/")?).ok()?;
        self.devices
            .get_key_value(name)
            .map(|(name, _)| name.as_str())
    }

    /// The top-level physical disks under the block device `source` names, pure (F95): a device
    /// with `slaves` rests on each of them, a partition on the disk its link's parent names, a
    /// `devices/virtual` device with no slaves on no disk, and any other device is a disk. Each
    /// device is visited once, so the walk is bounded by the topology's own size.
    fn disks(&self, source: &OsStr) -> Result<BTreeSet<&str>, Unresolvable> {
        let mut pending = vec![self.device_of(source).ok_or(Unresolvable::Source)?];
        let (mut seen, mut disks) = (BTreeSet::new(), BTreeSet::new());
        while let Some(name) = pending.pop() {
            if !seen.insert(name) {
                continue;
            }
            let device = self.devices.get(name).ok_or(Unresolvable::Unlisted)?;
            if !device.slaves.is_empty() {
                pending.extend(device.slaves.iter().map(String::as_str));
            } else if device.partition {
                let parent = device
                    .link
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(OsStr::to_str)
                    .ok_or(Unresolvable::NoParent)?;
                pending.push(parent);
            } else if is_virtual(&device.link) {
                return Err(Unresolvable::Virtual);
            } else {
                disks.insert(name);
            }
        }
        if disks.is_empty() {
            return Err(Unresolvable::NoDisk);
        }
        Ok(disks)
    }
}

/// One declared topology line, or `None` when it is not one.
fn parse_device(text: &[u8]) -> Option<(String, BlockDevice)> {
    let mut fields = text.split(|byte| *byte == b' ');
    let name = String::from_utf8(unescape(fields.next()?)?).ok()?;
    let link = PathBuf::from(OsString::from_vec(unescape(fields.next()?)?));
    let partition = match fields.next()? {
        b"part" => true,
        b"whole" => false,
        _ => return None,
    };
    if name.is_empty() || name.contains('/') || link.as_os_str().is_empty() {
        return None;
    }
    let (mut dm_name, mut slaves) = (None, Vec::new());
    for field in fields {
        if let Some(mapped) = field.strip_prefix(b"dm:") {
            let mapped = unescape(mapped)?;
            if dm_name.is_some() || mapped.is_empty() {
                return None;
            }
            dm_name = Some(OsString::from_vec(mapped));
        } else {
            let slave = String::from_utf8(unescape(field.strip_prefix(b"slave:")?)?).ok()?;
            if slave.is_empty() {
                return None;
            }
            slaves.push(slave);
        }
    }
    Some((
        name,
        BlockDevice {
            link,
            partition,
            dm_name,
            slaves,
        },
    ))
}

/// Whether a class link names a `devices/virtual` device: one with no hardware under it.
fn is_virtual(link: &Path) -> bool {
    let mut previous = None;
    link.components().any(|component| {
        let virtual_after_devices =
            previous == Some(OsStr::new("devices")) && component.as_os_str() == "virtual";
        previous = Some(component.as_os_str());
        virtual_after_devices
    })
}

/// A sysfs entry name as UTF-8, or `InvalidData`.
fn utf8(name: OsString) -> Result<String, TopologyWhy> {
    name.into_string()
        .map_err(|_| TopologyWhy::Unreadable(std::io::ErrorKind::InvalidData))
}

/// Whether sysfs has `path` (a `partition` attribute): absent is `false`, any other failure refuses.
fn present(path: &Path) -> Result<bool, TopologyWhy> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(TopologyWhy::Unreadable(error.kind())),
    }
}

/// A device's `dm/name`, its final newline removed: `None` when it is not a device-mapper device;
/// read under [`MAX_DM_NAME_BYTES`], `InvalidData` past it.
fn dm_name(path: &Path) -> Result<Option<OsString>, TopologyWhy> {
    let mut bytes = Vec::new();
    match std::fs::File::open(path)
        .and_then(|file| file.take(MAX_DM_NAME_BYTES + 1).read_to_end(&mut bytes))
    {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(TopologyWhy::Unreadable(error.kind())),
    }
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_DM_NAME_BYTES {
        return Err(TopologyWhy::Unreadable(std::io::ErrorKind::InvalidData));
    }
    let name = bytes.strip_suffix(b"\n").unwrap_or(&bytes).to_vec();
    Ok(Some(OsString::from_vec(name)))
}

/// A device's `slaves` entries (none when it has no `slaves` directory), at most `bound`.
fn slaves(path: &Path, bound: u64) -> Result<Vec<String>, TopologyWhy> {
    let io = |error: std::io::Error| TopologyWhy::Unreadable(error.kind());
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(io(error)),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(io)?;
        if u64::try_from(names.len()).unwrap_or(u64::MAX) >= bound {
            return Err(TopologyWhy::TooMany { bound });
        }
        names.push(utf8(entry.file_name())?);
    }
    names.sort();
    Ok(names)
}

/// What RC02's device rule is decided over: the mount table and the block topology, each as read
/// or why it could not be.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Devices {
    /// The mount table.
    pub mounts: Result<MountTable, TableWhy>,
    /// The block topology.
    pub topology: Result<Topology, TopologyWhy>,
}

/// RC02's separate-device rule, pure (F95): the state root's and the destination's containing
/// mounts (`None`: no mount resolves the path) over the block `topology`. Each must resolve, each
/// must have a backing device, they must name neither one source nor one filesystem, and — RC02's
/// separate device surviving a disk failure (R2-1) — each source must resolve to its physical
/// disks and the two sets must share none; the first that does not hold refuses, in that order, the
/// state root before the destination. The topology is consulted only at the disk step, so a
/// refusal that needs no topology is never masked by one that could not be read.
///
/// # Errors
/// [`DeviceWhy::Unresolved`], [`DeviceWhy::NoDevice`], [`DeviceWhy::SameSource`],
/// [`DeviceWhy::SameFilesystem`], [`DeviceWhy::Topology`], [`DeviceWhy::DiskUnresolved`] or
/// [`DeviceWhy::SameDisk`].
pub fn device_decision(
    state: Option<&Mount>,
    destination: Option<&Mount>,
    topology: &Result<Topology, TopologyWhy>,
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
    let topology = topology.as_ref().map_err(|why| DeviceWhy::Topology(*why))?;
    let disks = |side, mount: &Mount| {
        topology
            .disks(&mount.source)
            .map_err(|why| DeviceWhy::DiskUnresolved { side, why })
    };
    let (state_disks, destination_disks) = (
        disks(Side::State, state)?,
        disks(Side::Destination, destination)?,
    );
    if !state_disks.is_disjoint(&destination_disks) {
        return Err(DeviceWhy::SameDisk {
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
/// at `state_root`, deciding RC02's device rule over `devices` (the mount table and the block
/// topology, each or why it could not be read). The state root is resolved to its canonical path before it is looked up: the table
/// names canonical mount points, and on Kinoite `/home/<user>` is a link to `/var/home/<user>`.
///
/// # Errors
/// Each [`BackupUnready`] variant, as named there.
pub fn read_target(
    directory: &Path,
    state_root: &Path,
    devices: &Devices,
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
    let table = devices
        .mounts
        .as_ref()
        .map_err(|why| BackupUnready::Device(DeviceWhy::Table(*why)))?;
    let state = state_root.canonicalize().ok();
    device_decision(
        state.as_deref().and_then(|state| table.containing(state)),
        table.containing(destination),
        &devices.topology,
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
