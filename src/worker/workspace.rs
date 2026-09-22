//! Bounded source capture and fresh copies under the trusted worker owner.
//! These local filesystem observations do not establish hostile same-UID isolation.

use rustix::fs::{Dir, Mode, OFlags, fcntl_getfl, fstatfs, mkdirat, open, openat};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, Metadata, Permissions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Instant;

const MAX_ENTRIES: usize = 4096;
const MAX_DEPTH: usize = 32;
const MAX_FILE: u64 = 16 * 1024 * 1024;
const MAX_BYTES: u64 = 64 * 1024 * 1024;
const READ_FLAGS: OFlags = OFlags::RDONLY
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK)
    .union(OFlags::CLOEXEC);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FileIdentity {
    pub device: u64,
    pub inode: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Stamp {
    identity: FileIdentity,
    size: u64,
    mode: u32,
    links: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}
impl Stamp {
    fn new(meta: &Metadata) -> Self {
        Self {
            identity: FileIdentity {
                device: meta.dev(),
                inode: meta.ino(),
            },
            size: meta.len(),
            mode: meta.mode(),
            links: meta.nlink(),
            modified: (meta.mtime(), meta.mtime_nsec()),
            changed: (meta.ctime(), meta.ctime_nsec()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Content {
    Directory,
    File {
        bytes: Vec<u8>,
        sha256: [u8; 32],
        executable: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub path: String,
    pub content: Content,
    source: Stamp,
}
impl Entry {
    #[must_use]
    pub const fn source_identity(&self) -> FileIdentity {
        self.source.identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    root: PathBuf,
    root_stamp: Stamp,
    entries: BTreeMap<String, Entry>,
    bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Path,
    Custody,
    Type,
    Alias,
    Changed,
    Bound,
    Deadline,
    Io,
    Policy,
}

/// An incomplete copy is retained at its owned path for explicit reconciliation.
#[derive(Debug)]
pub struct MaterializeError {
    pub error: Error,
    pub partial_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Materialized {
    pub path: PathBuf,
}

fn budget(deadline: Instant) -> Result<(), Error> {
    if Instant::now() >= deadline {
        Err(Error::Deadline)
    } else {
        Ok(())
    }
}
fn component(value: &str) -> bool {
    !value.is_empty() && value != "." && value != ".." && !value.contains(['/', '\0'])
}
fn relative(value: &str) -> bool {
    value.len() <= 4096 && value.split('/').all(component)
}
fn private_root(path: &Path) -> Result<File, Error> {
    if !path.is_absolute()
        || path.canonicalize().map_err(|_| Error::Path)?.as_os_str() != path.as_os_str()
    {
        return Err(Error::Path);
    }
    let file = File::from(
        open(path, READ_FLAGS | OFlags::DIRECTORY, Mode::empty()).map_err(|_| Error::Io)?,
    );
    let meta = file.metadata().map_err(|_| Error::Io)?;
    if meta.uid() != rustix::process::geteuid().as_raw() || meta.mode() & 0o777 != 0o700 {
        return Err(Error::Custody);
    }
    Ok(file)
}

impl Snapshot {
    /// Capture every entry through retained parent descriptors, with bounded reads.
    /// The source root must be canonical, owned by this UID and mode 0700.
    ///
    /// # Errors
    /// Refuses links, special files, aliases, invalid paths, concurrent changes,
    /// unsafe ownership, exhausted size/depth/time limits, or filesystem errors.
    pub fn capture(
        root: &Path,
        protected: &[FileIdentity],
        deadline: Instant,
    ) -> Result<Self, Error> {
        budget(deadline)?;
        if protected.len() > MAX_ENTRIES + 1 {
            return Err(Error::Bound);
        }
        let directory = private_root(root)?;
        let root_stamp = Stamp::new(&directory.metadata().map_err(|_| Error::Io)?);
        let forbidden: BTreeSet<_> = protected.iter().copied().collect();
        if forbidden.contains(&root_stamp.identity) {
            return Err(Error::Alias);
        }
        let mut snapshot = Self {
            root: root.to_path_buf(),
            root_stamp,
            entries: BTreeMap::new(),
            bytes: 0,
        };
        snapshot.walk(&directory, "", &forbidden, None, deadline)?;
        if Stamp::new(&directory.metadata().map_err(|_| Error::Io)?) != snapshot.root_stamp {
            return Err(Error::Changed);
        }
        let reopened = private_root(root)?;
        if Stamp::new(&reopened.metadata().map_err(|_| Error::Io)?) != snapshot.root_stamp {
            return Err(Error::Changed);
        }
        budget(deadline)?;
        Ok(snapshot)
    }

    /// Captured source location; descriptive access alone does not grant custody.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.bytes
    }
    #[must_use]
    pub fn source_identities(&self) -> Vec<FileIdentity> {
        std::iter::once(self.root_stamp.identity)
            .chain(self.entries.values().map(Entry::source_identity))
            .collect()
    }

    /// Require unchanged bytes, complete inventory, inode identities and metadata.
    ///
    /// # Errors
    /// Refuses any changed or unavailable source or a capture-limit failure.
    pub fn readback_source(&self, deadline: Instant) -> Result<(), Error> {
        let current = Self::capture(&self.root, &[], deadline)?;
        if &current == self {
            Ok(())
        } else {
            Err(Error::Changed)
        }
    }

    fn walk(
        &mut self,
        directory: &File,
        prefix: &str,
        protected: &BTreeSet<FileIdentity>,
        required_device: Option<u64>,
        deadline: Instant,
    ) -> Result<(), Error> {
        budget(deadline)?;
        if prefix.split('/').filter(|part| !part.is_empty()).count() > MAX_DEPTH {
            return Err(Error::Bound);
        }
        let before = Stamp::new(&directory.metadata().map_err(|_| Error::Io)?);
        let mut names = BTreeSet::new();
        let mut listing = Dir::read_from(directory).map_err(|_| Error::Io)?;
        while let Some(entry) = listing.read() {
            budget(deadline)?;
            let entry = entry.map_err(|_| Error::Io)?;
            let name = entry.file_name().to_str().map_err(|_| Error::Path)?;
            if matches!(name, "." | "..") {
                continue;
            }
            if !component(name) || !names.insert(name.to_owned()) {
                return Err(Error::Path);
            }
            if self.entries.len() >= MAX_ENTRIES {
                return Err(Error::Bound);
            }
            let path = if prefix.is_empty() {
                name.to_owned()
            } else {
                format!("{prefix}/{name}")
            };
            if !relative(&path) {
                return Err(Error::Path);
            }
            if path.split('/').count() > MAX_DEPTH {
                return Err(Error::Bound);
            }
            let mut file = File::from(
                openat(directory, name, READ_FLAGS, Mode::empty()).map_err(|_| Error::Type)?,
            );
            let meta = file.metadata().map_err(|_| Error::Io)?;
            let source = Stamp::new(&meta);
            if required_device.is_some_and(|device| device != source.identity.device) {
                return Err(Error::Custody);
            }
            if required_device.is_some()
                && (source.identity == self.root_stamp.identity
                    || self
                        .entries
                        .values()
                        .any(|entry| entry.source.identity == source.identity))
            {
                return Err(Error::Alias);
            }
            if meta.uid() != rustix::process::geteuid().as_raw() || meta.mode() & 0o022 != 0 {
                return Err(Error::Custody);
            }
            if protected.contains(&source.identity) {
                return Err(Error::Alias);
            }
            let content = if meta.is_dir() {
                Content::Directory
            } else if meta.is_file() {
                if meta.nlink() != 1 {
                    return Err(Error::Alias);
                }
                self.capture_file(&mut file, &meta, deadline)?
            } else {
                return Err(Error::Type);
            };
            self.entries.insert(
                path.clone(),
                Entry {
                    path: path.clone(),
                    content,
                    source: source.clone(),
                },
            );
            if meta.is_dir() {
                self.walk(&file, &path, protected, required_device, deadline)?;
            }
            if Stamp::new(&file.metadata().map_err(|_| Error::Io)?) != source {
                return Err(Error::Changed);
            }
            let reopened = File::from(
                openat(directory, name, READ_FLAGS, Mode::empty()).map_err(|_| Error::Changed)?,
            );
            if Stamp::new(&reopened.metadata().map_err(|_| Error::Io)?) != source {
                return Err(Error::Changed);
            }
        }
        if Stamp::new(&directory.metadata().map_err(|_| Error::Io)?) != before {
            return Err(Error::Changed);
        }
        budget(deadline)
    }

    fn capture_file(
        &mut self,
        file: &mut File,
        meta: &Metadata,
        deadline: Instant,
    ) -> Result<Content, Error> {
        if meta.len() > MAX_FILE
            || self
                .bytes
                .checked_add(meta.len())
                .is_none_or(|sum| sum > MAX_BYTES)
        {
            return Err(Error::Bound);
        }
        let mut bytes = Vec::new();
        loop {
            budget(deadline)?;
            let mut buffer = [0_u8; 8192];
            let count = file.read(&mut buffer).map_err(|_| Error::Io)?;
            if count == 0 {
                break;
            }
            if bytes.len() as u64 + count as u64 > meta.len() {
                return Err(Error::Changed);
            }
            bytes.extend_from_slice(&buffer[..count]);
        }
        if bytes.len() as u64 != meta.len() {
            return Err(Error::Changed);
        }
        self.bytes += meta.len();
        Ok(Content::File {
            sha256: Sha256::digest(&bytes).into(),
            bytes,
            executable: meta.mode() & 0o111 != 0,
        })
    }

    /// Make an exclusive fresh copy. Only the explicitly named files are writable.
    /// A partial copy survives errors and is never presented as frozen evidence.
    ///
    /// # Errors
    /// Refuses unsafe parent/name/allowlist, existing destination, exhausted deadline
    /// or filesystem failure. Returns the partial owned path if creation occurred.
    pub fn materialize(
        &self,
        parent: &Path,
        name: &str,
        editable: &[String],
        deadline: Instant,
    ) -> Result<Materialized, MaterializeError> {
        let mut partial_path = None;
        let result = (|| {
            budget(deadline)?;
            if !component(name) || name.len() > 255 {
                return Err(Error::Path);
            }
            let editable = self.allowed(editable)?;
            let parent_fd = private_root(parent)?;
            if parent.starts_with(&self.root) {
                return Err(Error::Policy);
            }
            mkdirat(&parent_fd, name, Mode::RWXU).map_err(|_| Error::Io)?;
            let path = parent.join(name);
            partial_path = Some(path.clone());
            let root = File::from(
                openat(
                    &parent_fd,
                    name,
                    READ_FLAGS | OFlags::DIRECTORY,
                    Mode::empty(),
                )
                .map_err(|_| Error::Io)?,
            );
            root.set_permissions(Permissions::from_mode(0o700))
                .map_err(|_| Error::Io)?;
            self.write_copy(&root, &editable, deadline)?;
            root.sync_all().map_err(|_| Error::Io)?;
            parent_fd.sync_all().map_err(|_| Error::Io)?;
            budget(deadline)?;
            Ok(Materialized { path })
        })();
        result.map_err(|error| MaterializeError {
            error,
            partial_path,
        })
    }

    fn allowed<'a>(&self, editable: &'a [String]) -> Result<BTreeSet<&'a str>, Error> {
        if editable.len() > MAX_ENTRIES {
            return Err(Error::Bound);
        }
        let mut allowed = BTreeSet::new();
        for path in editable {
            if !relative(path)
                || !allowed.insert(path.as_str())
                || !self
                    .entries
                    .get(path)
                    .is_some_and(|entry| matches!(entry.content, Content::File { .. }))
            {
                return Err(Error::Policy);
            }
        }
        Ok(allowed)
    }

    fn write_copy(
        &self,
        root: &File,
        editable: &BTreeSet<&str>,
        deadline: Instant,
    ) -> Result<(), Error> {
        for entry in self.entries.values() {
            budget(deadline)?;
            let (prefix, name) = entry.path.rsplit_once('/').unwrap_or(("", &entry.path));
            let parent = open_directory(root, prefix)?;
            match &entry.content {
                Content::Directory => {
                    mkdirat(&parent, name, Mode::RWXU).map_err(|_| Error::Io)?;
                    let created = File::from(
                        openat(&parent, name, READ_FLAGS | OFlags::DIRECTORY, Mode::empty())
                            .map_err(|_| Error::Io)?,
                    );
                    created
                        .set_permissions(Permissions::from_mode(0o700))
                        .map_err(|_| Error::Io)?;
                }
                Content::File {
                    bytes, executable, ..
                } => {
                    let mut file = File::from(
                        openat(
                            &parent,
                            name,
                            OFlags::WRONLY
                                | OFlags::CREATE
                                | OFlags::EXCL
                                | OFlags::NOFOLLOW
                                | OFlags::CLOEXEC,
                            Mode::RUSR | Mode::WUSR,
                        )
                        .map_err(|_| Error::Io)?,
                    );
                    for chunk in bytes.chunks(8192) {
                        budget(deadline)?;
                        file.write_all(chunk).map_err(|_| Error::Io)?;
                    }
                    let mode = 0o400
                        | if *executable { 0o100 } else { 0 }
                        | if editable.contains(entry.path.as_str()) {
                            0o200
                        } else {
                            0
                        };
                    file.set_permissions(Permissions::from_mode(mode))
                        .map_err(|_| Error::Io)?;
                    file.sync_all().map_err(|_| Error::Io)?;
                }
            }
            parent.sync_all().map_err(|_| Error::Io)?;
        }
        Ok(())
    }

    /// Capture a settled edit and enforce its predeclared file-change scope.
    /// Inode identities differ by design; protected seed aliases remain forbidden.
    ///
    /// # Errors
    /// Refuses missing/new paths, changed file types or undeclared content/mode
    /// changes, and all ordinary capture failures.
    pub fn validate_result(
        &self,
        root: &Path,
        allowed_changes: &[String],
        deadline: Instant,
    ) -> Result<Self, Error> {
        let allowed = self.allowed(allowed_changes)?;
        let result = Self::capture(root, &self.source_identities(), deadline)?;
        if self.entries.len() != result.entries.len() {
            return Err(Error::Policy);
        }
        for (path, seed) in &self.entries {
            let actual = result.entries.get(path).ok_or(Error::Policy)?;
            let same_kind = matches!(
                (&seed.content, &actual.content),
                (Content::Directory, Content::Directory)
                    | (Content::File { .. }, Content::File { .. })
            );
            if !same_kind || !allowed.contains(path.as_str()) && seed.content != actual.content {
                return Err(Error::Policy);
            }
        }
        Ok(result)
    }
}

fn open_directory(root: &File, path: &str) -> Result<File, Error> {
    let mut current = File::from(
        openat(root, ".", READ_FLAGS | OFlags::DIRECTORY, Mode::empty()).map_err(|_| Error::Io)?,
    );
    if !path.is_empty() {
        for name in path.split('/') {
            current = File::from(
                openat(
                    &current,
                    name,
                    READ_FLAGS | OFlags::DIRECTORY,
                    Mode::empty(),
                )
                .map_err(|_| Error::Io)?,
            );
        }
    }
    Ok(current)
}

// Traversal accumulator only: this wrapper never exposes its internal Snapshot,
// calls path-based readback, or invents a public source path for a detached mount.
struct DescriptorTree {
    tree: Snapshot,
}
impl DescriptorTree {
    fn capture(
        root: &File,
        protected: &BTreeSet<FileIdentity>,
        deadline: Instant,
    ) -> Result<Self, Error> {
        budget(deadline)?;
        let meta = root.metadata().map_err(|_| Error::Io)?;
        if !meta.is_dir() {
            return Err(Error::Type);
        }
        if meta.uid() != rustix::process::geteuid().as_raw() || meta.mode() & 0o7777 != 0o700 {
            return Err(Error::Custody);
        }
        let stamp = Stamp::new(&meta);
        if protected.contains(&stamp.identity) {
            return Err(Error::Alias);
        }
        let mut tree = Snapshot {
            root: PathBuf::new(),
            root_stamp: stamp.clone(),
            entries: BTreeMap::new(),
            bytes: 0,
        };
        tree.walk(root, "", protected, Some(stamp.identity.device), deadline)?;
        if Stamp::new(&root.metadata().map_err(|_| Error::Io)?) != stamp {
            return Err(Error::Changed);
        }
        budget(deadline)?;
        Ok(Self { tree })
    }

    fn same_source(&self, other: &Self) -> Result<(), Error> {
        if self.tree == other.tree {
            Ok(())
        } else {
            Err(Error::Changed)
        }
    }

    fn verify_copy(&self, other: &Self) -> Result<(), Error> {
        if self.tree.entries.len() != other.tree.entries.len()
            || self.tree.bytes != other.tree.bytes
        {
            return Err(Error::Changed);
        }
        for (path, source) in &self.tree.entries {
            let copied = other.tree.entries.get(path).ok_or(Error::Changed)?;
            if copied.content != source.content {
                return Err(Error::Changed);
            }
            let mode = match &source.content {
                Content::Directory => 0o700,
                Content::File { executable, .. } => {
                    if *executable {
                        0o500
                    } else {
                        0o400
                    }
                }
            };
            if copied.source.mode & 0o7777 != mode {
                return Err(Error::Custody);
            }
        }
        Ok(())
    }
}

fn export_source(root: &File) -> Result<(), Error> {
    let flags = fcntl_getfl(root).map_err(|_| Error::Io)?;
    if flags.intersects(OFlags::PATH) || flags.intersection(OFlags::RWMODE) != OFlags::RDONLY {
        return Err(Error::Custody);
    }
    // Linux UAPI linux/magic.h TMPFS_MAGIC; no ambient path is used to infer it.
    if fstatfs(root).map_err(|_| Error::Io)?.f_type != 0x0102_1994 {
        return Err(Error::Custody);
    }
    Ok(())
}

/// Copy a post-writer tmpfs directory through the retained read-only descriptor.
/// The caller must independently prove writers terminal before calling this function.
/// No source path is reopened and no synthetic public Snapshot is returned.
///
/// # Errors
/// Refuses unsafe types/ownership/devices, aliases, source drift, size/time limits,
/// existing output or failed copy/readback. Created output is retained on every error.
pub fn export_directory(
    root: &File,
    parent: &Path,
    name: &str,
    protected: &[FileIdentity],
    deadline: Instant,
) -> Result<Materialized, MaterializeError> {
    let mut partial_path = None;
    let result = (|| {
        budget(deadline)?;
        if !component(name) || name.len() > 255 {
            return Err(Error::Path);
        }
        if protected.len() > MAX_ENTRIES + 1 {
            return Err(Error::Bound);
        }
        export_source(root)?;
        let forbidden: BTreeSet<_> = protected.iter().copied().collect();
        let source = DescriptorTree::capture(root, &forbidden, deadline)?;
        let parent_fd = private_root(parent)?;
        let parent_identity = Stamp::new(&parent_fd.metadata().map_err(|_| Error::Io)?).identity;
        let identities: BTreeSet<_> = source.tree.source_identities().into_iter().collect();
        if identities.contains(&parent_identity) {
            return Err(Error::Policy);
        }
        if forbidden.contains(&parent_identity) {
            return Err(Error::Alias);
        }
        mkdirat(&parent_fd, name, Mode::RWXU).map_err(|_| Error::Io)?;
        let path = parent.join(name);
        partial_path = Some(path.clone());
        let copied_root = File::from(
            openat(
                &parent_fd,
                name,
                READ_FLAGS | OFlags::DIRECTORY,
                Mode::empty(),
            )
            .map_err(|_| Error::Io)?,
        );
        copied_root
            .set_permissions(Permissions::from_mode(0o700))
            .map_err(|_| Error::Io)?;
        source
            .tree
            .write_copy(&copied_root, &BTreeSet::new(), deadline)?;
        copied_root.sync_all().map_err(|_| Error::Io)?;
        parent_fd.sync_all().map_err(|_| Error::Io)?;
        export_source(root)?;
        source.same_source(&DescriptorTree::capture(root, &forbidden, deadline)?)?;
        let copied_forbidden: BTreeSet<_> = forbidden.union(&identities).copied().collect();
        let copied = DescriptorTree::capture(&copied_root, &copied_forbidden, deadline)?;
        source.verify_copy(&copied)?;
        let named = private_root(&path)?;
        if Stamp::new(&named.metadata().map_err(|_| Error::Io)?) != copied.tree.root_stamp {
            return Err(Error::Changed);
        }
        let current_parent = private_root(parent)?;
        if Stamp::new(&current_parent.metadata().map_err(|_| Error::Io)?).identity
            != parent_identity
        {
            return Err(Error::Changed);
        }
        budget(deadline)?;
        Ok(Materialized { path })
    })();
    result.map_err(|error| MaterializeError {
        error,
        partial_path,
    })
}
