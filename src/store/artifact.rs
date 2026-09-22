//! Fixed, fd-relative immutable object publication for the store owner.

use super::{Error, Result, digest};
use crate::contracts::UuidV4;
use rustix::fs::{Mode, OFlags, RenameFlags, mkdirat, open, openat, renameat_with, unlinkat};
use std::fs::{File, Permissions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub(super) const OBJECT_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct Directory {
    pub file: File,
    pub path: PathBuf,
}

impl Directory {
    pub fn root(path: &Path) -> Result<Self> {
        // The configured absolute root must already be private and canonical.
        // This is cooperative owner custody, not resistance to a hostile same UID.
        if !path.is_absolute() || path.canonicalize()? != path {
            return Err(Error::Custody);
        }
        let file = File::from(open(
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        Self::checked(file, path.to_path_buf())
    }

    fn checked(file: File, path: PathBuf) -> Result<Self> {
        let meta = file.metadata()?;
        if !meta.is_dir()
            || meta.uid() != rustix::process::geteuid().as_raw()
            || meta.mode() & 0o777 != 0o700
        {
            return Err(Error::Custody);
        }
        Ok(Self { file, path })
    }

    // Duplicate held custody; never resolve a path or acquire a new lock.
    pub fn try_clone(&self) -> Result<Self> {
        Self::checked(self.file.try_clone()?, self.path.clone())
    }

    pub fn child(&self, name: &str, create: bool) -> Result<Self> {
        if name.is_empty() || matches!(name, "." | "..") || name.contains(['/', '\0']) {
            return Err(Error::Invalid);
        }
        if create {
            match mkdirat(&self.file, name, Mode::RWXU) {
                Ok(()) => self.file.sync_all()?,
                Err(rustix::io::Errno::EXIST) => (),
                Err(error) => return Err(error.into()),
            }
        }
        let file = File::from(openat(
            &self.file,
            name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        Self::checked(file, self.path.join(name))
    }

    pub fn regular(&self, name: &str) -> Result<File> {
        let file = File::from(openat(
            &self.file,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        )?);
        let meta = file.metadata()?;
        if !meta.is_file()
            || meta.nlink() != 1
            || meta.uid() != rustix::process::geteuid().as_raw()
            || meta.mode() & 0o077 != 0
        {
            return Err(Error::Custody);
        }
        Ok(file)
    }

    pub fn create_file(&self, name: &str) -> Result<File> {
        Ok(File::from(openat(
            &self.file,
            name,
            OFlags::RDWR | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        )?))
    }

    pub fn lock(&self) -> Result<File> {
        let file = match self.create_file("store.lock") {
            Ok(file) => {
                file.sync_all()?;
                self.file.sync_all()?;
                file
            }
            Err(Error::Os(error)) if error == rustix::io::Errno::EXIST => {
                self.regular("store.lock")?
            }
            Err(error) => return Err(error),
        };
        file.try_lock().map_err(|_| Error::Locked)?;
        let named = self.regular("store.lock")?;
        let held = file.metadata()?;
        let current = named.metadata()?;
        if (held.dev(), held.ino()) != (current.dev(), current.ino()) {
            return Err(Error::Custody);
        }
        Ok(file)
    }
}

/// An object identity, never evidence of a trusted producer or successful check.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Object {
    pub(super) digest: String,
    pub(super) size: u64,
}

impl Object {
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }
}

pub(super) fn verify(root: &Directory, object: &Object) -> Result<Vec<u8>> {
    crate::contracts::Sha256Digest::parse(&object.digest).map_err(|_| Error::Invalid)?;
    if object.size > OBJECT_LIMIT as u64 {
        return Err(Error::Bound);
    }
    let hex = &object.digest[7..];
    let shard = root.child(&hex[..2], false)?;
    let file = shard.regular(hex)?;
    if file.metadata()?.len() != object.size {
        return Err(Error::Corrupt);
    }
    let mut bytes = Vec::new();
    file.take(object.size + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != object.size || digest(&bytes) != object.digest {
        return Err(Error::Corrupt);
    }
    Ok(bytes)
}

pub(super) fn publish(
    root: &Directory,
    bytes: &[u8],
    staging_id: UuidV4<'_>,
    mut point: impl FnMut(super::CutPoint) -> Result<()>,
) -> Result<Object> {
    if bytes.len() > OBJECT_LIMIT {
        return Err(Error::Bound);
    }
    let object = Object {
        digest: digest(bytes),
        size: bytes.len() as u64,
    };
    let hex = &object.digest[7..];
    let shard = root.child(&hex[..2], true)?;
    let temp = format!(".stage-{}", staging_id.as_str());
    let mut file = shard.create_file(&temp)?;
    let held = file.metadata()?;
    let result = (|| {
        file.write_all(bytes)?;
        point(super::CutPoint::ObjectWrite)?;
        file.set_permissions(Permissions::from_mode(0o400))?;
        file.sync_all()?;
        point(super::CutPoint::ObjectSync)?;
        // The only writable descriptor is closed before publication.
        drop(file);
        match renameat_with(&shard.file, &temp, &shard.file, hex, RenameFlags::NOREPLACE) {
            Ok(()) => (),
            Err(rustix::io::Errno::EXIST) => {
                verify(root, &object)?;
                unlinkat(&shard.file, &temp, rustix::fs::AtFlags::empty())?;
            }
            Err(error) => return Err(error.into()),
        }
        point(super::CutPoint::ObjectRename)?;
        shard.file.sync_all()?;
        point(super::CutPoint::ObjectDirectorySync)?;
        Ok(())
    })();
    let cleanup = || -> Result<()> {
        // Clean only this invocation's exclusive staging inode. A published CAS
        // object is retained, as are any substituted/uncertain files.
        match shard.regular(&temp) {
            Ok(file) => {
                let current = file.metadata()?;
                if (current.dev(), current.ino()) != (held.dev(), held.ino()) {
                    return Err(Error::Custody);
                }
                unlinkat(&shard.file, &temp, rustix::fs::AtFlags::empty())?;
                shard.file.sync_all()?;
            }
            Err(Error::Os(error)) if error == rustix::io::Errno::NOENT => (),
            Err(error) => return Err(error),
        }
        Ok(())
    };
    match result {
        Ok(()) => Ok(object),
        Err(original) => match cleanup() {
            Ok(()) => Err(original),
            Err(failure) => Err(Error::Cleanup {
                original: Box::new(original),
                failure: Box::new(failure),
            }),
        },
    }
}
