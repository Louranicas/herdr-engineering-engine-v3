//! Custody of the operator's private configuration (RC03 §5): the one door through which the
//! grant store and the route configuration are read.
//!
//! **Custody before content.** A directory must be the operator's, mode 0700, opened without
//! following a final link; a file in it must be a regular 0600 file of the operator's, opened
//! without following a link and read under a caller-named bound, `take(bound + 1)` so an oversize
//! file is refused without being read whole. A record a second user could have written is not a
//! record.
//!
//! Every failure is typed, so a caller can tell an absence (nothing installed) from a refusal
//! (something is there and is not trusted) and name each honestly.

use rustix::fs::{Mode, OFlags, openat};
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// Why a private directory could not be opened.
#[derive(Debug)]
pub enum DirectoryError {
    /// Nothing is at the path.
    NotFound,
    /// Something is there, and it is not the operator's private 0700 directory.
    Custody,
    /// It could not be opened for another reason.
    Io(io::Error),
}

/// Why a file in a private directory could not be read.
#[derive(Debug)]
pub enum FileError {
    /// No such file.
    NotFound,
    /// Something is there, and it is not the operator's regular 0600 file (or the name is not a
    /// plain file name).
    Custody,
    /// The file is larger than the caller's bound.
    TooLarge,
    /// It could not be read for another reason.
    Io(io::Error),
}

fn io_error(errno: rustix::io::Errno) -> io::Error {
    errno.into()
}

/// A directory held open under custody.
#[derive(Debug)]
pub struct PrivateDirectory {
    directory: File,
}

impl PrivateDirectory {
    /// Open `path` under custody.
    ///
    /// # Errors
    ///
    /// [`DirectoryError::NotFound`] when nothing is there; [`DirectoryError::Custody`] unless it is
    /// a directory owned by this process's effective user with mode 0700, reached without
    /// following a final symlink (a symlink there is refused, not followed); [`DirectoryError::Io`]
    /// otherwise.
    pub fn open(path: &Path) -> Result<Self, DirectoryError> {
        let directory = match rustix::fs::open(
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(fd) => File::from(fd),
            Err(rustix::io::Errno::NOENT) => return Err(DirectoryError::NotFound),
            Err(rustix::io::Errno::LOOP | rustix::io::Errno::NOTDIR) => {
                return Err(DirectoryError::Custody);
            }
            Err(errno) => return Err(DirectoryError::Io(io_error(errno))),
        };
        let meta = directory.metadata().map_err(DirectoryError::Io)?;
        if !meta.is_dir() || !owned_private(&meta, 0o700) {
            return Err(DirectoryError::Custody);
        }
        Ok(Self { directory })
    }

    /// Read the file `name` in this directory, at most `bound` bytes.
    ///
    /// # Errors
    ///
    /// [`FileError::Custody`] for a name that is not a plain file name, a symlink, or anything but
    /// a regular file owned by this process's effective user with mode 0600;
    /// [`FileError::NotFound`] when there is no such file; [`FileError::TooLarge`] over `bound`;
    /// [`FileError::Io`] otherwise.
    pub fn read(&self, name: &str, bound: u64) -> Result<Vec<u8>, FileError> {
        if name.is_empty() || name.contains('/') || name == "." || name == ".." {
            return Err(FileError::Custody);
        }
        let file = match openat(
            &self.directory,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        ) {
            Ok(fd) => File::from(fd),
            Err(rustix::io::Errno::NOENT) => return Err(FileError::NotFound),
            Err(rustix::io::Errno::LOOP) => return Err(FileError::Custody),
            Err(errno) => return Err(FileError::Io(io_error(errno))),
        };
        let meta = file.metadata().map_err(FileError::Io)?;
        if !meta.is_file() || !owned_private(&meta, 0o600) {
            return Err(FileError::Custody);
        }
        if meta.len() > bound {
            return Err(FileError::TooLarge);
        }
        let mut bytes = Vec::new();
        file.take(bound.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(FileError::Io)?;
        if u64::try_from(bytes.len()).map_or(true, |length| length > bound) {
            return Err(FileError::TooLarge);
        }
        Ok(bytes)
    }
}

fn owned_private(meta: &std::fs::Metadata, mode: u32) -> bool {
    meta.uid() == rustix::process::geteuid().as_raw() && meta.mode() & 0o777 == mode
}
