//! One predeclared fixed-file repair over a fresh immutable source snapshot ([`apply`]), and one
//! untrusted candidate's replacement of a class's single editable file ([`apply_candidate`], B14-P3).

use crate::check::patch;
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

/// The most changed lines any class may admit: the ceiling a caller's bound is clamped to, so the
/// shortest-edit search is `O((n+m)·MAX_CHANGED_LINES)` whatever a caller asks (review P3-1). The
/// check lane owns it with the search, which it bounds (B14-P4).
pub use crate::check::patch::MAX_CHANGED_LINES;

/// What a task class admits of one candidate (B14-P3): at most `bytes` of new text for its editable
/// file, changing at most `changed_lines` of the baseline's lines. The class profile (B14-P2)
/// supplies both, with the editable path; this function trusts its caller for that binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CandidateBounds {
    /// The largest candidate, in bytes: the bound on volume.
    pub bytes: usize,
    /// The most changed lines: line insertions plus deletions in a shortest edit, every
    /// newline-delimited line counted, blank or not, a final line's newline not counted — the
    /// derived patch (B14-P4) counts it, so its edits exceed this by at most two — clamped to
    /// [`MAX_CHANGED_LINES`]. The bound on edit scope, not on volume.
    pub changed_lines: usize,
}

/// The newline-delimited lines of `text` (a final line needs no newline; a final newline adds no
/// empty line): the unit a class counts changes in (design B14 R2, review P3-3).
fn lines(text: &[u8]) -> Vec<&[u8]> {
    let mut parts: Vec<&[u8]> = text.split(|byte| *byte == b'\n').collect();
    if parts.last().is_some_and(|last| last.is_empty()) {
        parts.pop();
    }
    parts
}

/// How many lines [`lines`] would yield, counted without allocating (review P3-2).
fn line_count(text: &[u8]) -> usize {
    // As many parts as `split` yields, less the one empty part a final newline (or no text) leaves.
    text.split(|byte| *byte == b'\n').count()
        - usize::from(text.is_empty() || text.ends_with(b"\n"))
}

/// The shortest edit distance (line insertions plus deletions) between `before` and `after`, if it
/// is at most `limit` — a bounded Myers search, so a large rewrite is refused in `O((n+m)·limit)`
/// without ever computing the whole diff; a distance is at least the length difference, so that is
/// refused first; the deadline is checked every round. The search is the check lane's, the one
/// the receipt's patch binding also runs (B14-P4).
fn changed_lines(
    before: &[&[u8]],
    after: &[&[u8]],
    limit: usize,
    deadline: Instant,
) -> Result<Option<usize>, Error> {
    patch::distance(before, after, limit, Some(deadline)).map_err(|patch::Expired| Error::Deadline)
}

/// Materialize `baseline` (create-new), replace its one `editable_path` with an untrusted
/// `candidate`, and require every other entry to equal the baseline's and the edited entry to hold
/// exactly `candidate` with the baseline's executable bit (B14-P3). Unlike [`apply`] there is no
/// predeclared expected result: the candidate is data, bounded by its class before any file is
/// created, and the only filesystem value is the one this function materializes and freezes.
///
/// The caller owns three bindings this function cannot check: `editable_path` and `bounds` are the
/// class profile's (B14-P2), and `fresh_parent` is the runtime's one attempts directory, never a
/// prior attempt's workspace (B14a; `materialize` refuses only a parent inside the baseline).
///
/// # Errors
/// `Path` for an invalid editable path or one the baseline holds no file at; `Bound` past
/// `bounds.bytes`; `Encoding` for text that is not UTF-8; `Changes` past `bounds.changed_lines`;
/// `Deadline` — all before anything is created. Then [`apply`]'s, including a `Workspace` bound when
/// the recaptured result exceeds the workspace's total. Retains the fresh owned path on every
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
        let limit = bounds.changed_lines.min(MAX_CHANGED_LINES);
        if line_count(seed).abs_diff(line_count(candidate)) > limit {
            return Err(Error::Changes);
        }
        changed_lines(&lines(seed), &lines(candidate), limit, deadline)?.ok_or(Error::Changes)?;
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
    /// A captured baseline of `files` (path, bytes, mode) under a fresh private root, and a fresh
    /// private parent for the candidate's materialization. Both are removed when the guard drops.
    struct Fixture {
        base: Snapshot,
        fresh: PathBuf,
        roots: [PathBuf; 2],
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            for root in &self.roots {
                let _ = fs::remove_dir_all(root);
            }
        }
    }

    fn fixture(label: &str, files: &[(&str, &[u8], u32)]) -> Fixture {
        let source = private_root(&format!("{label}-source"));
        for (path, bytes, mode) in files {
            let file = source.join(path);
            if let Some(parent) = file.parent() {
                fs::DirBuilder::new()
                    .mode(0o700)
                    .recursive(true)
                    .create(parent)
                    .unwrap();
            }
            fs::write(&file, bytes).unwrap();
            fs::set_permissions(&file, Permissions::from_mode(*mode)).unwrap();
        }
        let base = Snapshot::capture(&source, &[], deadline()).unwrap();
        let fresh = private_root(&format!("{label}-fresh"));
        Fixture {
            base,
            fresh: fresh.clone(),
            roots: [source, fresh],
        }
    }

    const WIDE: CandidateBounds = CandidateBounds {
        bytes: 1 << 20,
        changed_lines: 200,
    };
    const BASE: &[u8] = b"pub fn parse(text: &str) -> u64 {\n    0\n}\n";
    const CANDIDATE: &[u8] = b"pub fn parse(text: &str) -> u64 {\n    text.len() as u64\n}\n";
    const CARGO: &[u8] = b"[package]\nname = \"x\"\n";

    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(5)
    }

    /// B14-P3 · the line distances, against an independent Python LCS (`len(a) + len(b) −
    /// 2·LCS(a, b)` over newline-delimited lines, a final newline adding no empty line; the script
    /// and its output are in `~/hee3-evidence/T28/B14-store-runtime-20260926/`): even and odd,
    /// blank lines counted, deletion-only, either side empty, a final line without a newline; each
    /// admitted at its limit and refused one below it.
    #[test]
    fn changed_lines_are_a_bounded_shortest_edit() {
        for (before, after, expected) in [
            (&b"a\nb\nc\n"[..], &b"a\nb\nc\n"[..], 0),
            (b"fn a() {}\nfn b() {}\n", b"fn a() {}\nfn c() {}\n", 2),
            (b"a\n\nb\n", b"a\n\n\n   \nb\n", 2),
            (b"x\ny\n", b"x\np\nq\ny\n", 2),
            (b"1\n2\n3\n4\n", b"4\n3\n2\n1\n", 6),
            (b"a\nb\nc\n", b"a\nc\n", 1),
            (b"", b"a\nb\n", 2),
            (b"a\n", b"", 1),
            (b"a\nb\nc\nd\n", b"b\nd\n", 2),
            (b"a\nb", b"a\nb\n", 0),
        ] {
            let (a, b) = (lines(before), lines(after));
            assert_eq!(
                (line_count(before), line_count(after)),
                (a.len(), b.len()),
                "the count is the list's: {before:?} {after:?}"
            );
            assert_eq!(
                changed_lines(&a, &b, 200, deadline()),
                Ok(Some(expected)),
                "{before:?}"
            );
            assert_eq!(
                changed_lines(&a, &b, expected, deadline()),
                Ok(Some(expected)),
                "at the limit"
            );
            if expected > 0 {
                assert_eq!(
                    changed_lines(&a, &b, expected - 1, deadline()),
                    Ok(None),
                    "past the limit"
                );
            }
        }
        for (text, count) in [
            (&b""[..], 0),
            (b"\n", 1),
            (b"a", 1),
            (b"a\n", 1),
            (b"a\n\n", 2),
            (b"\n\nb", 3),
        ] {
            assert_eq!(
                (lines(text).len(), line_count(text)),
                (count, count),
                "{text:?}"
            );
        }
        // A caller's bound is clamped to the ceiling, and an expired deadline stops the search.
        let long: Vec<&[u8]> = vec![&b"x"[..]; MAX_CHANGED_LINES + 10];
        assert_eq!(changed_lines(&long, &[], usize::MAX, deadline()), Ok(None));
        let passed = Instant::now()
            .checked_sub(Duration::from_millis(1))
            .unwrap();
        assert_eq!(
            changed_lines(&[b"a"], &[b"b"], 10, passed),
            Err(Error::Deadline)
        );
    }

    /// B14-P3 · a candidate replaces exactly the one editable file: the frozen result holds its
    /// bytes read-only at the editable path — keeping the baseline's executable bit, here 0700 →
    /// 0500 — and every other entry as the baseline has it, in a directory this call created.
    #[test]
    fn a_candidate_replaces_exactly_the_editable_file() {
        for (mode, sealed, executable) in [(0o600, 0o400, false), (0o700, 0o500, true)] {
            let f = fixture(
                "apply",
                &[("src/lib.rs", BASE, mode), ("Cargo.toml", CARGO, 0o600)],
            );
            let result = apply_candidate(
                &f.base,
                "src/lib.rs",
                CANDIDATE,
                WIDE,
                &f.fresh,
                "attempt",
                deadline(),
            )
            .unwrap();
            let written = f.fresh.join("attempt/src/lib.rs");
            assert_eq!(fs::read(&written).unwrap(), CANDIDATE);
            assert_eq!(
                fs::metadata(&written).unwrap().permissions().mode() & 0o777,
                sealed
            );
            assert_eq!(fs::read(f.fresh.join("attempt/Cargo.toml")).unwrap(), CARGO);
            let edited = result
                .entries()
                .find(|entry| entry.path == "src/lib.rs")
                .unwrap();
            assert!(
                matches!(&edited.content, Content::File { bytes, executable: is, .. }
                    if bytes.as_slice() == CANDIDATE && *is == executable),
                "{mode:o}"
            );
            assert!(same_but_candidate(
                &result,
                &f.base,
                "src/lib.rs",
                CANDIDATE,
                executable
            ));
            assert!(!same_but_candidate(
                &result,
                &f.base,
                "src/lib.rs",
                BASE,
                executable
            ));
            assert!(!same_but_candidate(
                &result,
                &f.base,
                "src/lib.rs",
                CANDIDATE,
                !executable
            ));
        }
    }

    /// B14-P3 · every entry but the edited one must be the baseline's: a differing file, an extra
    /// trailing entry, a missing trailing entry and a directory where the baseline has a file are
    /// each refused, however the edit looks.
    #[test]
    fn every_other_entry_must_be_the_baselines() {
        let base = fixture(
            "entries",
            &[("src/lib.rs", BASE, 0o600), ("zz.txt", b"z\n", 0o600)],
        );
        let candidate_at = |label: &str, extra: &[(&str, &[u8], u32)]| {
            let mut files = vec![("src/lib.rs", CANDIDATE, 0o600)];
            files.extend_from_slice(extra);
            fixture(label, &files)
        };
        let same = candidate_at("entries-same", &[("zz.txt", b"z\n", 0o600)]);
        assert!(same_but_candidate(
            &same.base,
            &base.base,
            "src/lib.rs",
            CANDIDATE,
            false
        ));
        for (label, extra) in [
            ("entries-differs", vec![("zz.txt", &b"y\n"[..], 0o600)]),
            (
                "entries-extra",
                vec![("zz.txt", b"z\n", 0o600), ("zzz.txt", b"extra\n", 0o600)],
            ),
            ("entries-missing", vec![]),
        ] {
            let other = candidate_at(label, &extra);
            assert!(
                !same_but_candidate(&other.base, &base.base, "src/lib.rs", CANDIDATE, false),
                "{label}"
            );
        }
        let kind = candidate_at("entries-kind", &[("zz.txt/inner", b"z\n", 0o600)]);
        assert!(
            !same_but_candidate(&kind.base, &base.base, "src/lib.rs", CANDIDATE, false),
            "a directory for a file"
        );
    }

    /// B14-P3 · every class bound refuses before anything is created (no partial path): bytes one
    /// past the bound (the bound itself admitted), text that is not UTF-8, changes one past the
    /// bound (at it admitted), a length difference past the bound, and an editable path the
    /// baseline holds no file at (absent, or a directory).
    #[test]
    fn class_bounds_refuse_before_anything_is_created() {
        let f = fixture(
            "bounds",
            &[("src/lib.rs", BASE, 0o600), ("src/other.rs", b"x\n", 0o600)],
        );
        let refused = |candidate: &[u8], bounds: CandidateBounds, path: &str, name: &str| {
            let failure =
                apply_candidate(&f.base, path, candidate, bounds, &f.fresh, name, deadline())
                    .unwrap_err();
            assert!(failure.partial_path.is_none(), "{name}");
            assert!(!f.fresh.join(name).exists(), "{name}");
            failure.error
        };
        let exact = CandidateBounds {
            bytes: CANDIDATE.len(),
            changed_lines: 2,
        };
        assert!(
            apply_candidate(
                &f.base,
                "src/lib.rs",
                CANDIDATE,
                exact,
                &f.fresh,
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
        let longer = [BASE, b"a\nb\nc\n"].concat();
        assert_eq!(
            refused(&longer, lines, "src/lib.rs", "length"),
            Error::Changes
        );
        assert_eq!(
            refused(CANDIDATE, WIDE, "src/absent.rs", "absent"),
            Error::Path
        );
        assert_eq!(refused(CANDIDATE, WIDE, "src", "directory"), Error::Path);
    }

    /// B14-P3 · never reopen: a destination that exists is refused by `materialize`'s create-new
    /// (`Io`), reports no partial path (so a caller's cleanup cannot remove what it did not create),
    /// and is left exactly as it was.
    #[test]
    fn an_existing_destination_is_never_reopened() {
        let f = fixture("reopen", &[("src/lib.rs", BASE, 0o600)]);
        let taken = f.fresh.join("attempt");
        fs::DirBuilder::new().mode(0o700).create(&taken).unwrap();
        fs::write(taken.join("marker"), b"prior").unwrap();
        let failure = apply_candidate(
            &f.base,
            "src/lib.rs",
            CANDIDATE,
            WIDE,
            &f.fresh,
            "attempt",
            deadline(),
        )
        .unwrap_err();
        assert_eq!(failure.error, Error::Workspace(workspace::Error::Io));
        assert!(failure.partial_path.is_none());
        assert_eq!(fs::read(taken.join("marker")).unwrap(), b"prior");
        assert!(!taken.join("src").exists());
    }
}
