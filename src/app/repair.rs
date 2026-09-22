//! One predeclared fixed-file repair over a fresh immutable source snapshot.

use crate::worker::workspace::{self, Content, Snapshot};
use rustix::fs::{Mode, OFlags, open, openat};
use std::fs::{File, Permissions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Instant;

const MAX_REPLACEMENT: usize = 16 * 1024 * 1024;
const OPEN_DIRECTORY: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK)
    .union(OFlags::CLOEXEC);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Path,
    Bound,
    Deadline,
    Workspace(workspace::Error),
    Changed,
    Io,
}

/// A created directory remains named until its trusted owner explicitly removes it.
#[derive(Debug)]
pub struct Failure {
    pub error: Error,
    pub partial_path: Option<PathBuf>,
}

fn budget(deadline: Instant) -> Result<(), Error> {
    if Instant::now() >= deadline {
        Err(Error::Deadline)
    } else {
        Ok(())
    }
}

fn valid_relative(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4096
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != ".." && !part.contains('\0'))
}

fn root(path: &Path) -> Result<File, Error> {
    if !path.is_absolute()
        || path.canonicalize().map_err(|_| Error::Path)?.as_os_str() != path.as_os_str()
    {
        return Err(Error::Path);
    }
    Ok(File::from(
        open(path, OPEN_DIRECTORY, Mode::empty()).map_err(|_| Error::Io)?,
    ))
}

fn parent(root: &File, path: &str) -> Result<(File, String), Error> {
    let mut parts = path.split('/').collect::<Vec<_>>();
    let name = parts.pop().ok_or(Error::Path)?.to_owned();
    let mut directory =
        File::from(openat(root, ".", OPEN_DIRECTORY, Mode::empty()).map_err(|_| Error::Io)?);
    for part in parts {
        directory = File::from(
            openat(&directory, part, OPEN_DIRECTORY, Mode::empty()).map_err(|_| Error::Changed)?,
        );
    }
    Ok((directory, name))
}

fn replace(
    root: &File,
    path: &str,
    bytes: &[u8],
    executable: bool,
    deadline: Instant,
) -> Result<(), Error> {
    let (directory, name) = parent(root, path)?;
    let mut file = File::from(
        openat(
            &directory,
            name,
            OFlags::WRONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| Error::Changed)?,
    );
    let before = file.metadata().map_err(|_| Error::Io)?;
    let expected_mode = if executable { 0o700 } else { 0o600 };
    if !before.file_type().is_file()
        || before.nlink() != 1
        || before.uid() != rustix::process::geteuid().as_raw()
        || before.mode() & 0o777 != expected_mode
    {
        return Err(Error::Changed);
    }
    file.set_len(0).map_err(|_| Error::Io)?;
    for chunk in bytes.chunks(8192) {
        budget(deadline)?;
        file.write_all(chunk).map_err(|_| Error::Io)?;
    }
    file.set_len(bytes.len().try_into().map_err(|_| Error::Bound)?)
        .map_err(|_| Error::Io)?;
    file.set_permissions(Permissions::from_mode(if executable {
        0o500
    } else {
        0o400
    }))
    .map_err(|_| Error::Io)?;
    file.sync_all().map_err(|_| Error::Io)?;
    let after = file.metadata().map_err(|_| Error::Io)?;
    if !after.file_type().is_file()
        || after.nlink() != 1
        || after.dev() != before.dev()
        || after.ino() != before.ino()
        || after.len() != bytes.len() as u64
    {
        return Err(Error::Changed);
    }
    directory.sync_all().map_err(|_| Error::Io)?;
    budget(deadline)
}

fn expected_file(expected: &Snapshot, path: &str, bytes: &[u8]) -> Result<bool, Error> {
    let entry = expected
        .entries()
        .find(|entry| entry.path == path)
        .ok_or(Error::Changed)?;
    match &entry.content {
        Content::File {
            bytes: wanted,
            executable,
            ..
        } if wanted.as_slice() == bytes => Ok(*executable),
        _ => Err(Error::Changed),
    }
}

fn same_inventory(actual: &Snapshot, expected: &Snapshot) -> bool {
    let mut actual = actual.entries();
    let mut expected = expected.entries();
    loop {
        match (actual.next(), expected.next()) {
            (Some(left), Some(right))
                if left.path == right.path && left.content == right.content => {}
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn freeze(root_file: &File, deadline: Instant) -> Result<(), Error> {
    // Snapshot custody requires private directories to remain 0700. Materialize
    // already made every non-editable file read-only; replace seals the sole edit.
    root_file.sync_all().map_err(|_| Error::Io)?;
    budget(deadline)
}

/// Materialize `baseline`, apply the sole predeclared replacement, and require the
/// complete recaptured result to equal `expected` in path, kind, bytes and executable bit.
///
/// # Errors
/// Retains the fresh owned path on every failure after creation.
pub fn apply(
    baseline: &Snapshot,
    expected: &Snapshot,
    editable_path: &str,
    replacement: &[u8],
    fresh_parent: &Path,
    fresh_name: &str,
    deadline: Instant,
) -> Result<Snapshot, Failure> {
    let mut partial_path = None;
    let result = (|| {
        budget(deadline)?;
        if !valid_relative(editable_path) || replacement.len() > MAX_REPLACEMENT {
            return Err(if replacement.len() > MAX_REPLACEMENT {
                Error::Bound
            } else {
                Error::Path
            });
        }
        baseline
            .readback_source(deadline)
            .map_err(Error::Workspace)?;
        expected
            .readback_source(deadline)
            .map_err(Error::Workspace)?;
        let editable = vec![editable_path.to_owned()];
        let materialized = baseline
            .materialize(fresh_parent, fresh_name, &editable, deadline)
            .map_err(|failure| {
                partial_path = failure.partial_path;
                Error::Workspace(failure.error)
            })?;
        partial_path = Some(materialized.path.clone());
        let executable = expected_file(expected, editable_path, replacement)?;
        let root_file = root(&materialized.path)?;
        replace(&root_file, editable_path, replacement, executable, deadline)?;
        let mutable = baseline
            .validate_result(&materialized.path, &editable, deadline)
            .map_err(Error::Workspace)?;
        if !same_inventory(&mutable, expected) {
            return Err(Error::Changed);
        }
        freeze(&root_file, deadline)?;
        let frozen = Snapshot::capture(&materialized.path, &baseline.source_identities(), deadline)
            .map_err(Error::Workspace)?;
        if !same_inventory(&frozen, expected) {
            return Err(Error::Changed);
        }
        baseline
            .readback_source(deadline)
            .map_err(Error::Workspace)?;
        expected
            .readback_source(deadline)
            .map_err(Error::Workspace)?;
        Ok(frozen)
    })();
    result.map_err(|error| Failure {
        error,
        partial_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    use std::time::{Duration, SystemTime};

    fn private_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "hee3-fixed-repair-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        path
    }

    #[test]
    fn hardlink_is_rejected_before_original_bytes_change() {
        let path = private_root("hardlink");
        let original = path.join("subject");
        fs::write(&original, b"preserve-me").unwrap();
        fs::set_permissions(&original, Permissions::from_mode(0o600)).unwrap();
        fs::hard_link(&original, path.join("alias")).unwrap();
        let directory = root(&path).unwrap();
        assert_eq!(
            replace(
                &directory,
                "subject",
                b"damage",
                false,
                Instant::now() + Duration::from_secs(1)
            ),
            Err(Error::Changed)
        );
        assert_eq!(fs::read(&original).unwrap(), b"preserve-me");
        fs::remove_file(path.join("alias")).unwrap();
        fs::remove_file(original).unwrap();
        fs::remove_dir(path).unwrap();
    }

    #[test]
    fn ordinary_owned_materialized_mode_is_replaced() {
        let path = private_root("ordinary");
        let subject = path.join("subject");
        fs::write(&subject, b"before").unwrap();
        fs::set_permissions(&subject, Permissions::from_mode(0o600)).unwrap();
        let directory = root(&path).unwrap();
        replace(
            &directory,
            "subject",
            b"after",
            false,
            Instant::now() + Duration::from_secs(1),
        )
        .unwrap();
        assert_eq!(fs::read(&subject).unwrap(), b"after");
        assert_eq!(
            fs::metadata(&subject).unwrap().permissions().mode() & 0o777,
            0o400
        );
        fs::remove_file(subject).unwrap();
        fs::remove_dir(path).unwrap();
    }
}
