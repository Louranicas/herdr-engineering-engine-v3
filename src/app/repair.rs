//! One predeclared fixed-file repair over a fresh immutable source snapshot ([`apply`]), and one
//! untrusted candidate's replacement of a class's single editable file ([`apply_candidate`], B14-P3).

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
    /// A candidate is not UTF-8 text.
    Encoding,
    /// A candidate changes more logical lines than its class admits.
    Changes,
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

/// What a task class admits of one candidate (B14-P3): at most `bytes` of new text for its editable
/// file, changing at most `changed_lines` logical lines of the baseline's.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CandidateBounds {
    /// The largest candidate, in bytes.
    pub bytes: usize,
    /// The most changed logical lines: line insertions plus deletions in a shortest edit, blank
    /// (whitespace-only) lines not counted.
    pub changed_lines: usize,
}

/// The non-blank lines of `text`, the unit a class counts changes in.
fn logical_lines(text: &[u8]) -> Vec<&[u8]> {
    text.split(|byte| *byte == b'\n')
        .filter(|line| !line.iter().all(u8::is_ascii_whitespace))
        .collect()
}

/// The shortest edit distance (line insertions plus deletions) between `before` and `after`, if it
/// is at most `limit` — a bounded Myers search, so a large rewrite is refused in `O((n+m)·limit)`
/// without ever computing the whole diff.
fn changed_lines(before: &[&[u8]], after: &[&[u8]], limit: usize) -> Option<usize> {
    let (n, m) = (before.len(), after.len());
    let limit = limit.min(n + m);
    // The furthest `x` reached on each diagonal `k = x - y`, indexed `k + limit + 1`.
    let mut furthest = vec![0_usize; 2 * limit + 3];
    for distance in 0..=limit {
        for step in 0..=distance {
            // k runs -distance, -distance + 2, ..., distance: index = k + limit + 1.
            let index = limit + 1 + 2 * step - distance;
            let down = step == 0 || (step != distance && furthest[index - 1] < furthest[index + 1]);
            let mut x = if down {
                furthest[index + 1]
            } else {
                furthest[index - 1] + 1
            };
            // y = x - k, with k = 2·step - distance.
            let Some(mut y) = (x + distance).checked_sub(2 * step) else {
                continue;
            };
            while x < n && y < m && before[x] == after[y] {
                x += 1;
                y += 1;
            }
            furthest[index] = x;
            if x >= n && y >= m {
                return Some(distance);
            }
        }
    }
    None
}

/// Materialize `baseline` (create-new), replace its one `editable_path` with an untrusted
/// `candidate`, and require every other entry to equal the baseline's and the edited entry to hold
/// exactly `candidate` with the baseline's executable bit (B14-P3). Unlike [`apply`] there is no
/// predeclared expected result: the candidate is data, bounded by its class before any file is
/// created, and the only filesystem value is the one this function materializes and freezes.
///
/// # Errors
/// `Path` for an invalid editable path or one the baseline holds no file at; `Bound` past
/// `bounds.bytes`; `Encoding` for text that is not UTF-8; `Changes` past `bounds.changed_lines`
/// (all before anything is created); then [`apply`]'s. Retains the fresh owned path on every
/// failure after creation.
pub fn apply_candidate(
    baseline: &Snapshot,
    editable_path: &str,
    candidate: &[u8],
    bounds: CandidateBounds,
    fresh_parent: &Path,
    fresh_name: &str,
    deadline: Instant,
) -> Result<Snapshot, Failure> {
    let mut partial_path = None;
    let result = (|| {
        budget(deadline)?;
        if !valid_relative(editable_path) {
            return Err(Error::Path);
        }
        if candidate.len() > bounds.bytes.min(MAX_REPLACEMENT) {
            return Err(Error::Bound);
        }
        std::str::from_utf8(candidate).map_err(|_| Error::Encoding)?;
        let (seed, executable) = baseline
            .entries()
            .find_map(|entry| match &entry.content {
                Content::File {
                    bytes, executable, ..
                } if entry.path == editable_path => Some((bytes.as_slice(), *executable)),
                _ => None,
            })
            .ok_or(Error::Path)?;
        changed_lines(
            &logical_lines(seed),
            &logical_lines(candidate),
            bounds.changed_lines,
        )
        .ok_or(Error::Changes)?;
        baseline
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
        let root_file = root(&materialized.path)?;
        replace(&root_file, editable_path, candidate, executable, deadline)?;
        let mutable = baseline
            .validate_result(&materialized.path, &editable, deadline)
            .map_err(Error::Workspace)?;
        if !same_but_candidate(&mutable, baseline, editable_path, candidate, executable) {
            return Err(Error::Changed);
        }
        freeze(&root_file, deadline)?;
        let frozen = Snapshot::capture(&materialized.path, &baseline.source_identities(), deadline)
            .map_err(Error::Workspace)?;
        if !same_but_candidate(&frozen, baseline, editable_path, candidate, executable) {
            return Err(Error::Changed);
        }
        baseline
            .readback_source(deadline)
            .map_err(Error::Workspace)?;
        Ok(frozen)
    })();
    result.map_err(|error| Failure {
        error,
        partial_path,
    })
}

/// `actual` equals `baseline` entry for entry, except `path`, which is a file holding exactly
/// `candidate` with `executable`.
fn same_but_candidate(
    actual: &Snapshot,
    baseline: &Snapshot,
    path: &str,
    candidate: &[u8],
    executable: bool,
) -> bool {
    let mut actual = actual.entries();
    let mut baseline = baseline.entries();
    loop {
        match (actual.next(), baseline.next()) {
            (Some(left), Some(right)) if left.path == right.path && left.path == path => {
                let Content::File {
                    bytes,
                    executable: is,
                    ..
                } = &left.content
                else {
                    return false;
                };
                if bytes.as_slice() != candidate || *is != executable {
                    return false;
                }
            }
            (Some(left), Some(right))
                if left.path == right.path && left.content == right.content => {}
            (None, None) => return true,
            _ => return false,
        }
    }
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
    /// A captured baseline of `files` (path, bytes) under a fresh private root, and a fresh private
    /// parent for the candidate's materialization.
    fn baseline(label: &str, files: &[(&str, &[u8])]) -> (Snapshot, PathBuf, PathBuf) {
        let source = private_root(&format!("{label}-source"));
        for (path, bytes) in files {
            let file = source.join(path);
            if let Some(parent) = file.parent() {
                fs::DirBuilder::new()
                    .mode(0o700)
                    .recursive(true)
                    .create(parent)
                    .unwrap();
            }
            fs::write(&file, bytes).unwrap();
            fs::set_permissions(&file, Permissions::from_mode(0o600)).unwrap();
        }
        let snapshot =
            Snapshot::capture(&source, &[], Instant::now() + Duration::from_secs(5)).unwrap();
        (snapshot, source, private_root(&format!("{label}-fresh")))
    }

    const WIDE: CandidateBounds = CandidateBounds {
        bytes: 1 << 20,
        changed_lines: 200,
    };
    const BASE: &[u8] = b"pub fn parse(text: &str) -> u64 {\n    0\n}\n";
    const CANDIDATE: &[u8] = b"pub fn parse(text: &str) -> u64 {\n    text.len() as u64\n}\n";

    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(5)
    }

    /// B14-P3 · the line counts, against an independent Python LCS (`len(a) + len(b) - 2·LCS(a, b)`
    /// over non-blank lines): same 0, one changed line 2, blank-only change 0, two inserted 2, a
    /// four-line reversal 6; a limit below the distance is refused, at it admitted.
    #[test]
    fn changed_lines_are_a_bounded_shortest_edit() {
        for (before, after, expected) in [
            (&b"a\nb\nc\n"[..], &b"a\nb\nc\n"[..], 0),
            (b"fn a() {}\nfn b() {}\n", b"fn a() {}\nfn c() {}\n", 2),
            (b"a\n\nb\n", b"a\n\n\n   \nb\n", 0),
            (b"x\ny\n", b"x\np\nq\ny\n", 2),
            (b"1\n2\n3\n4\n", b"4\n3\n2\n1\n", 6),
        ] {
            let (a, b) = (logical_lines(before), logical_lines(after));
            assert_eq!(changed_lines(&a, &b, 200), Some(expected), "{before:?}");
            assert_eq!(
                changed_lines(&a, &b, expected),
                Some(expected),
                "at the limit"
            );
            if expected > 0 {
                assert_eq!(changed_lines(&a, &b, expected - 1), None, "past the limit");
            }
        }
    }

    /// B14-P3 · a candidate replaces exactly the one editable file: the frozen result holds its
    /// bytes read-only at the editable path, every other entry as the baseline has it, in a
    /// directory this call created.
    #[test]
    fn a_candidate_replaces_exactly_the_editable_file() {
        let (base, _, fresh) = baseline(
            "apply",
            &[
                ("src/lib.rs", BASE),
                ("Cargo.toml", b"[package]\nname = \"x\"\n"),
            ],
        );
        let result = apply_candidate(
            &base,
            "src/lib.rs",
            CANDIDATE,
            WIDE,
            &fresh,
            "attempt",
            deadline(),
        )
        .unwrap();
        let written = fresh.join("attempt/src/lib.rs");
        assert_eq!(fs::read(&written).unwrap(), CANDIDATE);
        assert_eq!(
            fs::metadata(&written).unwrap().permissions().mode() & 0o777,
            0o400
        );
        assert_eq!(
            fs::read(fresh.join("attempt/Cargo.toml")).unwrap(),
            b"[package]\nname = \"x\"\n"
        );
        assert!(same_but_candidate(
            &result,
            &base,
            "src/lib.rs",
            CANDIDATE,
            false
        ));
        assert!(!same_but_candidate(
            &result,
            &base,
            "src/lib.rs",
            BASE,
            false
        ));
    }

    /// B14-P3 · every class bound refuses before anything is created (no partial path): bytes one
    /// past the bound (the bound itself admitted), text that is not UTF-8, changes one past the
    /// bound (at it admitted), and an editable path the baseline holds no file at.
    #[test]
    fn class_bounds_refuse_before_anything_is_created() {
        let (base, _, fresh) =
            baseline("bounds", &[("src/lib.rs", BASE), ("src/other.rs", b"x\n")]);
        let refused = |candidate: &[u8], bounds: CandidateBounds, path: &str, name: &str| {
            let failure = apply_candidate(&base, path, candidate, bounds, &fresh, name, deadline())
                .unwrap_err();
            assert!(failure.partial_path.is_none(), "{name}");
            assert!(!fresh.join(name).exists(), "{name}");
            failure.error
        };
        let exact = CandidateBounds {
            bytes: CANDIDATE.len(),
            changed_lines: 2,
        };
        assert!(
            apply_candidate(
                &base,
                "src/lib.rs",
                CANDIDATE,
                exact,
                &fresh,
                "at-bounds",
                deadline()
            )
            .is_ok()
        );
        let bytes = CandidateBounds {
            bytes: CANDIDATE.len() - 1,
            ..exact
        };
        assert_eq!(
            refused(CANDIDATE, bytes, "src/lib.rs", "bytes"),
            Error::Bound
        );
        assert_eq!(
            refused(&[0xff, 0xfe], WIDE, "src/lib.rs", "encoding"),
            Error::Encoding
        );
        let lines = CandidateBounds {
            changed_lines: 1,
            ..exact
        };
        assert_eq!(
            refused(CANDIDATE, lines, "src/lib.rs", "changes"),
            Error::Changes
        );
        assert_eq!(
            refused(CANDIDATE, WIDE, "src/absent.rs", "absent"),
            Error::Path
        );
        assert_eq!(refused(CANDIDATE, WIDE, "src", "directory"), Error::Path);
        assert_eq!(
            refused(CANDIDATE, WIDE, "../src/lib.rs", "escape"),
            Error::Path
        );
    }

    /// B14-P3 · never reopen: a destination that exists is refused and left exactly as it was.
    #[test]
    fn an_existing_destination_is_never_reopened() {
        let (base, _, fresh) = baseline("reopen", &[("src/lib.rs", BASE)]);
        let taken = fresh.join("attempt");
        fs::DirBuilder::new().mode(0o700).create(&taken).unwrap();
        fs::write(taken.join("marker"), b"prior").unwrap();
        let failure = apply_candidate(
            &base,
            "src/lib.rs",
            CANDIDATE,
            WIDE,
            &fresh,
            "attempt",
            deadline(),
        )
        .unwrap_err();
        assert!(
            matches!(failure.error, Error::Workspace(_)),
            "{:?}",
            failure.error
        );
        assert_eq!(fs::read(taken.join("marker")).unwrap(), b"prior");
        assert!(!taken.join("src").exists());
    }
}
