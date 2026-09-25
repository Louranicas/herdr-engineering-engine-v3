//! One text file's change as a GNU unified diff (B14-P4): the bounded shortest-edit search that
//! both the candidate bound (`app::repair`) and the receipt's seed → result patch binding
//! (`consistency`) use — one core, so the two can never count a change differently — and its
//! `diff -u --label a/<path> --label b/<path>` rendering.
//!
//! Allocations, each bounded before it is made: a text past [`MAX_TEXT`] is refused before it is
//! split; the line views are one slice per line (at most `MAX_TEXT` lines a side); the search's
//! trace holds `2·d + 1` cells for each round `d` actually run, `d ≤` the caller's limit, itself at
//! most [`MAX_EDITS`]; the rendering is at most both texts plus two bytes and one marker a line,
//! plus one header a hunk. A deadline, when given, is read every search round — the only
//! superlinear work; one round's diagonal slides are not individually bounded (inherited from P3),
//! and the rendering after the search is linear in its output.

use std::time::Instant;

/// The most changed lines any class may admit (B14-P3's ceiling on `CandidateBounds`).
pub const MAX_CHANGED_LINES: usize = 4096;
/// The most edits a derivation searches: a class's changed lines plus the at most one deletion and
/// one insertion a final line's missing newline adds when the terminator counts (B14-P4 P4-R1.4).
pub const MAX_EDITS: usize = MAX_CHANGED_LINES + 2;
/// The largest text either side may be, in bytes (the candidate replacement's bound).
pub const MAX_TEXT: usize = 16 * 1024 * 1024;
/// Unchanged lines a GNU unified diff shows around each change.
const CONTEXT: usize = 3;
const NO_NEWLINE: &[u8] = b"\n\\ No newline at end of file\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The deadline passed.
    Deadline,
    /// The shortest edit needs more than the limit.
    Changes,
    /// A text is not UTF-8, or holds a NUL (GNU would call it binary).
    Encoding,
    /// A text is larger than [`MAX_TEXT`], or a path is not one line.
    Bound,
}

/// The one way a search stops short of an answer: its deadline passed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Expired;

impl From<Expired> for Error {
    fn from(_: Expired) -> Self {
        Self::Deadline
    }
}

fn budget(deadline: Option<Instant>) -> Result<(), Expired> {
    match deadline {
        Some(deadline) if Instant::now() >= deadline => Err(Expired),
        _ => Ok(()),
    }
}

/// The shortest edit distance (line insertions plus deletions) between `before` and `after`, if it
/// is at most `limit` — a bounded Myers search, so a large rewrite is refused in `O((n+m)·limit)`
/// without ever computing the whole diff; a distance is at least the length difference, so that is
/// refused first.
///
/// # Errors
/// `Expired` when `deadline` passes between rounds.
pub fn distance(
    before: &[&[u8]],
    after: &[&[u8]],
    limit: usize,
    deadline: Option<Instant>,
) -> Result<Option<usize>, Expired> {
    search(before, after, limit, deadline, None)
}

/// The bounded Myers search: the distance, if at most `limit` (clamped to [`MAX_EDITS`]), and —
/// when `trace` is given — the `furthest` vector's live span after every round, which is what a
/// backtrack reads. The trace grows one round at a time, only as rounds are actually run.
fn search(
    before: &[&[u8]],
    after: &[&[u8]],
    limit: usize,
    deadline: Option<Instant>,
    mut trace: Option<&mut Vec<Vec<usize>>>,
) -> Result<Option<usize>, Expired> {
    let (n, m) = (before.len(), after.len());
    if n.abs_diff(m) > limit {
        return Ok(None);
    }
    let limit = limit.min(n + m).min(MAX_EDITS);
    // The furthest `x` reached on each diagonal `k = x - y`, indexed `k + limit + 1`.
    let mut furthest = vec![0_usize; 2 * limit + 3];
    for distance in 0..=limit {
        budget(deadline)?;
        let mut done = false;
        for step in 0..=distance {
            // k runs -distance, -distance + 2, ..., distance: index = k + limit + 1.
            let index = limit + 1 + 2 * step - distance;
            let down = step == 0 || (step != distance && furthest[index - 1] < furthest[index + 1]);
            let mut x = if down {
                furthest[index + 1]
            } else {
                furthest[index - 1] + 1
            };
            // y = x - k, with k = 2·step - distance. Never negative (a down move adds one to a
            // non-negative y, a right move keeps one); were it ever, refuse rather than guess.
            let Some(mut y) = (x + distance).checked_sub(2 * step) else {
                return Ok(None);
            };
            while x < n && y < m && before[x] == after[y] {
                x += 1;
                y += 1;
            }
            furthest[index] = x;
            if x >= n && y >= m {
                done = true;
                break;
            }
        }
        if let Some(trace) = trace.as_deref_mut() {
            trace.push(furthest[limit + 1 - distance..=limit + 1 + distance].to_vec());
        }
        if done {
            return Ok(Some(distance));
        }
    }
    Ok(None)
}

/// One line of a shortest edit script: kept (its index in `before`), deleted from `before` (its
/// index there), or inserted from `after` (its index there).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Edit {
    Keep(usize),
    Delete(usize),
    Insert(usize),
}

/// The shortest edit script [`search`] found at `distance`, read back from its `trace`: at each
/// round the move taken is the one the forward pass took (down from `k + 1` when that diagonal
/// reached further, right from `k - 1` otherwise), so the script is the forward pass's own path.
/// `None` when the trace does not describe a path from the origin to `(n, m)`.
fn script(
    old_len: usize,
    new_len: usize,
    trace: &[Vec<usize>],
    distance: usize,
) -> Option<Vec<Edit>> {
    // In round `round`'s stored span, diagonal `diagonal` sits at `diagonal + round`.
    let furthest = |round: usize, diagonal: isize| -> Option<usize> {
        let slot = usize::try_from(diagonal.checked_add(isize::try_from(round).ok()?)?).ok()?;
        trace.get(round)?.get(slot).copied()
    };
    let (mut old_at, mut new_at) = (old_len, new_len);
    let mut edits = Vec::with_capacity(old_len + new_len);
    for round in (1..=distance).rev() {
        let diagonal = isize::try_from(old_at).ok()? - isize::try_from(new_at).ok()?;
        let reach = isize::try_from(round).ok()?;
        let down = diagonal == -reach
            || (diagonal != reach
                && furthest(round - 1, diagonal - 1)? < furthest(round - 1, diagonal + 1)?);
        let previous = if down { diagonal + 1 } else { diagonal - 1 };
        let previous_old = furthest(round - 1, previous)?;
        let previous_new = usize::try_from(isize::try_from(previous_old).ok()? - previous).ok()?;
        let (from_old, from_new) = if down {
            (previous_old, previous_new + 1)
        } else {
            (previous_old + 1, previous_new)
        };
        while old_at > from_old && new_at > from_new {
            old_at -= 1;
            new_at -= 1;
            edits.push(Edit::Keep(old_at));
        }
        if (old_at, new_at) != (from_old, from_new) {
            return None;
        }
        edits.push(if down {
            Edit::Insert(previous_new)
        } else {
            Edit::Delete(previous_old)
        });
        (old_at, new_at) = (previous_old, previous_new);
    }
    while old_at > 0 && new_at > 0 {
        old_at -= 1;
        new_at -= 1;
        edits.push(Edit::Keep(old_at));
    }
    if (old_at, new_at) != (0, 0) {
        return None;
    }
    edits.reverse();
    Some(edits)
}

/// The lines of `text` with their terminators: a final line without a newline is its own,
/// different line, so a change to the final newline is a change (B14-P4 P4-2).
fn terminated(text: &[u8]) -> Vec<&[u8]> {
    text.split_inclusive(|byte| *byte == b'\n').collect()
}

fn admitted(text: &[u8]) -> Result<(), Error> {
    if text.len() > MAX_TEXT {
        return Err(Error::Bound);
    }
    if text.contains(&0) {
        return Err(Error::Encoding);
    }
    std::str::from_utf8(text).map_err(|_| Error::Encoding)?;
    Ok(())
}

/// The `-` and `+` lines of a unified diff of one file: its edit count. Its first two lines are the
/// headers; a hunk header starts `@`, a context line ` `, a marker `\`.
#[must_use]
pub fn edit_count(patch: &[u8]) -> usize {
    patch
        .split_inclusive(|byte| *byte == b'\n')
        .skip(2)
        .filter(|row| matches!(row.first(), Some(b'-' | b'+')))
        .count()
}

/// The GNU `diff -u --label a/<path> --label b/<path>` text taking `before` to `after` (B14-P4),
/// or no bytes when they are equal: every change's deletions before its insertions, `CONTEXT`
/// lines around it, changes at most `2·CONTEXT` unchanged lines apart in one hunk, a hunk side of
/// one line written without its count and an empty side at the line before, and `\ No newline at
/// end of file` after any written line that is its side's unterminated last. Among equally short
/// scripts the choice is this search's, which may differ from GNU's (P4-R1.2).
///
/// # Errors
/// `Bound` for a text past [`MAX_TEXT`] or a path holding a newline, `Encoding` for text that is
/// not UTF-8 or holds a NUL — both before any split — then `Changes` past `limit` edits (clamped to
/// [`MAX_EDITS`]) and `Deadline` between search rounds.
pub fn unified(
    before: &[u8],
    after: &[u8],
    path: &str,
    limit: usize,
    deadline: Option<Instant>,
) -> Result<Vec<u8>, Error> {
    admitted(before)?;
    admitted(after)?;
    if path.contains('\n') {
        return Err(Error::Bound);
    }
    if before == after {
        return Ok(Vec::new());
    }
    let (old, new) = (terminated(before), terminated(after));
    let mut trace = Vec::new();
    let distance = search(&old, &new, limit, deadline, Some(&mut trace))?.ok_or(Error::Changes)?;
    let edits = script(old.len(), new.len(), &trace, distance).ok_or(Error::Changes)?;
    drop(trace);
    // Each change: the half-open run of script positions that are not `Keep`.
    let mut changes = Vec::new();
    let mut position = 0;
    while position < edits.len() {
        if matches!(edits[position], Edit::Keep(_)) {
            position += 1;
            continue;
        }
        let start = position;
        while position < edits.len() && !matches!(edits[position], Edit::Keep(_)) {
            position += 1;
        }
        changes.push(start..position);
    }
    let mut text = format!("--- a/{path}\n+++ b/{path}\n").into_bytes();
    let mut index = 0;
    while index < changes.len() {
        // Merge every following change at most 2·CONTEXT kept lines after this hunk's last.
        let mut last = index;
        while last + 1 < changes.len() && changes[last + 1].start - changes[last].end <= 2 * CONTEXT
        {
            last += 1;
        }
        let from = changes[index].start.saturating_sub(CONTEXT);
        let to = (changes[last].end + CONTEXT).min(edits.len());
        let lines = Lines {
            old: &old,
            new: &new,
        };
        lines.hunk(&mut text, consumed(&edits[..from]), &edits[from..to]);
        index = last + 1;
    }
    Ok(text)
}

/// How many old and new lines `edits` consume.
fn consumed(edits: &[Edit]) -> (usize, usize) {
    edits.iter().fold((0, 0), |(old, new), edit| match edit {
        Edit::Keep(_) => (old + 1, new + 1),
        Edit::Delete(_) => (old + 1, new),
        Edit::Insert(_) => (old, new + 1),
    })
}

struct Lines<'a> {
    old: &'a [&'a [u8]],
    new: &'a [&'a [u8]],
}

impl Lines<'_> {
    /// Write one hunk starting at `(old_start, new_start)` (0-based lines already consumed): its
    /// header, then its script — within each change, deletions before insertions.
    fn hunk(&self, text: &mut Vec<u8>, (old_start, new_start): (usize, usize), span: &[Edit]) {
        let (old_count, new_count) = consumed(span);
        let side = |start: usize, count: usize| match count {
            0 => format!("{start},0"),
            1 => format!("{}", start + 1),
            _ => format!("{},{count}", start + 1),
        };
        text.extend_from_slice(
            format!(
                "@@ -{} +{} @@\n",
                side(old_start, old_count),
                side(new_start, new_count)
            )
            .as_bytes(),
        );
        let line = |text: &mut Vec<u8>, prefix: u8, bytes: &[u8]| {
            text.push(prefix);
            text.extend_from_slice(bytes);
            if !bytes.ends_with(b"\n") {
                text.extend_from_slice(NO_NEWLINE);
            }
        };
        let mut position = 0;
        while position < span.len() {
            if let Edit::Keep(old) = span[position] {
                line(text, b' ', self.old[old]);
                position += 1;
                continue;
            }
            let end = span[position..]
                .iter()
                .position(|edit| matches!(edit, Edit::Keep(_)))
                .map_or(span.len(), |offset| position + offset);
            for edit in &span[position..end] {
                if let Edit::Delete(old) = edit {
                    line(text, b'-', self.old[*old]);
                }
            }
            for edit in &span[position..end] {
                if let Edit::Insert(new) = edit {
                    line(text, b'+', self.new[*new]);
                }
            }
            position = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(10)
    }

    /// Apply a unified diff of one file to `before`, strictly: every hunk's header counts must
    /// equal the lines it consumes and produces, every context and deleted line must be `before`'s
    /// at that place, and hunks must advance. `None` for any patch that does not apply exactly.
    /// Test-only and written by the module's author, so it is first proven on GNU's own patches.
    fn apply_patch(before: &[u8], patch: &[u8], name: &str) -> Option<Vec<u8>> {
        if patch.is_empty() {
            return Some(before.to_vec());
        }
        let old = terminated(before);
        let mut rows = terminated(patch).into_iter().peekable();
        let header = format!("--- a/{name}\n+++ b/{name}\n");
        let first_two = [rows.next()?, rows.next()?].concat();
        if first_two != header.as_bytes() {
            return None;
        }
        let (mut out, mut cursor) = (Vec::new(), 0_usize);
        let number = |text: &str| text.parse::<usize>().ok();
        while let Some(row) = rows.next() {
            let row = std::str::from_utf8(row).ok()?;
            let ranges = row.strip_prefix("@@ -")?.strip_suffix(" @@\n")?;
            let (minus, plus) = ranges.split_once(" +")?;
            let side = |text: &str| match text.split_once(',') {
                Some((start, count)) => Some((number(start)?, number(count)?)),
                None => Some((number(text)?, 1)),
            };
            let ((old_start, old_count), (_, new_count)) = (side(minus)?, side(plus)?);
            let begin = if old_count == 0 {
                old_start
            } else {
                old_start.checked_sub(1)?
            };
            if begin < cursor || begin > old.len() {
                return None;
            }
            out.extend(old[cursor..begin].iter().copied().flatten());
            cursor = begin;
            let (mut consumed, mut produced) = (0, 0);
            while consumed < old_count || produced < new_count {
                let line = rows.next()?;
                let (prefix, body) = line.split_first()?;
                let mut body = body.to_vec();
                if rows.peek() == Some(&&b"\\ No newline at end of file\n"[..]) {
                    rows.next();
                    body.pop()?;
                }
                match prefix {
                    b' ' | b'-' => {
                        if old.get(cursor)? != &body.as_slice() {
                            return None;
                        }
                        cursor += 1;
                        consumed += 1;
                        if *prefix == b' ' {
                            out.extend_from_slice(&body);
                            produced += 1;
                        }
                    }
                    b'+' => {
                        out.extend_from_slice(&body);
                        produced += 1;
                    }
                    _ => return None,
                }
            }
            if (consumed, produced) != (old_count, new_count) {
                return None;
            }
        }
        out.extend(old[cursor..].iter().copied().flatten());
        Some(out)
    }

    /// B14-P4 · every derived patch against the world: each case's GNU `diff -u --label a/P
    /// --label b/P` text and its minimal distance (a textbook LCS program in Python — neither is
    /// this module's reasoning; the generator and diff version are recorded in the file).
    /// Where the shortest edit is unique enough that GNU's choice is every tool's — every named
    /// case — the patch is GNU's byte for byte; for the random cases, where GNU's heuristics
    /// (`discard_confusing_lines`, `shift_boundaries`) pick among equally short scripts, the patch
    /// has exactly the minimal edit count and applies to give `after` exactly. The applier is
    /// first shown to apply GNU's own patch, so it is not only this module's reading of the format.
    #[test]
    fn derived_patches_are_minimal_unified_diffs_that_apply()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixtures: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/patch/cases.json"))?;
        let path = fixtures["path"].as_str().ok_or("path")?;
        let cases = fixtures["cases"].as_array().ok_or("cases")?;
        assert_eq!(cases.len(), 85, "the generated set, whole");
        let (mut named, mut differing) = (0, Vec::new());
        for case in cases {
            let text = |key: &str| case[key].as_str().ok_or_else(|| key.to_owned());
            let (name, before, after) = (text("name")?, text("before")?, text("after")?);
            let (gnu, distance) = (
                text("expected")?,
                case["distance"].as_u64().ok_or("distance")?,
            );
            let (before, after) = (before.as_bytes(), after.as_bytes());
            assert_eq!(
                apply_patch(before, gnu.as_bytes(), path).as_deref(),
                Some(after),
                "{name}: the applier applies GNU's patch"
            );
            let derived = unified(before, after, path, 202, Some(deadline()))
                .map_err(|e| format!("{name}: {e:?}"))?;
            assert_eq!(
                (
                    apply_patch(before, &derived, path).as_deref(),
                    edit_count(&derived)
                ),
                (Some(after), usize::try_from(distance)?),
                "{name}: {}",
                String::from_utf8_lossy(&derived)
            );
            if derived != gnu.as_bytes() {
                differing.push(name);
            }
            if !name.starts_with("random_") {
                named += 1;
                assert_eq!(derived, gnu.as_bytes(), "{name}: GNU's bytes");
            }
        }
        assert_eq!(named, 25, "every named case is held to GNU's bytes");
        assert!(differing.iter().all(|name| name.starts_with("random_")));
        Ok(())
    }

    /// B14-P4 · the class's own frozen pair: `reference.patch` is the manifest's object (its digest
    /// asserted from `manifest.json`), and deriving base → reference reproduces it byte for byte.
    #[test]
    fn the_reference_pair_derives_the_frozen_reference_patch()
    -> Result<(), Box<dyn std::error::Error>> {
        let task = "../../evaluation/tasks/WL-U64-PARSE-001/v1";
        let manifest: serde_json::Value = serde_json::from_str(include_str!(
            "../../evaluation/tasks/WL-U64-PARSE-001/v1/manifest.json"
        ))?;
        let patch = include_bytes!("../../evaluation/tasks/WL-U64-PARSE-001/v1/reference.patch");
        assert_eq!(
            manifest["files_sha256"]["reference.patch"].as_str(),
            Some(sha2_hex(patch).as_str()),
            "{task}/reference.patch is the manifest's"
        );
        let derived = unified(
            include_bytes!("../../evaluation/tasks/WL-U64-PARSE-001/v1/base/src/lib.rs"),
            include_bytes!("../../evaluation/tasks/WL-U64-PARSE-001/v1/reference/src/lib.rs"),
            "src/lib.rs",
            202,
            Some(deadline()),
        );
        assert_eq!(derived.as_deref(), Ok(&patch[..]));
        Ok(())
    }

    fn sha2_hex(bytes: &[u8]) -> String {
        use sha2::Digest;
        use std::fmt::Write as _;
        sha2::Sha256::digest(bytes)
            .iter()
            .fold(String::new(), |mut hex, byte| {
                // Writing to a `String` cannot fail.
                let _ = write!(hex, "{byte:02x}");
                hex
            })
    }

    /// B14-P4 P4-R1.4 · the terminator's +2 is exact: `x\na` → `y\na\n` changes one line when the
    /// terminator is ignored (P3's count, 2 edits) and two when it counts (4 edits). The derivation
    /// admits exactly 4 and refuses 3; `a\nb` → `a\nb\n` is 0 edits for P3 and exactly 2 here.
    #[test]
    fn a_final_newline_adds_exactly_two_edits() {
        let blind = |text: &'static [u8]| -> Vec<&'static [u8]> {
            text.split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty())
                .collect()
        };
        for (before, after, p3, p4) in [
            (&b"x\na"[..], &b"y\na\n"[..], 2, 4),
            (b"a\nb", b"a\nb\n", 0, 2),
            (b"q", b"q\nr", 1, 3),
        ] {
            assert_eq!(
                distance(&blind(before), &blind(after), 10, Some(deadline())),
                Ok(Some(p3)),
                "the terminator-blind count"
            );
            let patch = unified(before, after, "f", p4, Some(deadline()));
            assert_eq!(patch.map(|patch| edit_count(&patch)), Ok(p4), "{before:?}");
            assert_eq!(
                unified(before, after, "f", p4 - 1, Some(deadline())),
                Err(Error::Changes),
                "{before:?} one below"
            );
        }
    }

    /// B14-P4 · each refusal by its own cause, and each before any work: a text one byte past
    /// `MAX_TEXT` (either side), a NUL, text that is not UTF-8, a path of two lines, a passed
    /// deadline; and a limit of `usize::MAX` is clamped rather than overflowed.
    #[test]
    fn refusals_name_their_cause() {
        let big = vec![b'a'; MAX_TEXT + 1];
        let fits = vec![b'a'; MAX_TEXT];
        assert_eq!(
            unified(&big, b"", "f", 10, Some(deadline())),
            Err(Error::Bound)
        );
        assert_eq!(
            unified(b"", &big, "f", 10, Some(deadline())),
            Err(Error::Bound)
        );
        assert_eq!(
            unified(&fits, &fits, "f", 10, Some(deadline())),
            Ok(Vec::new())
        );
        assert_eq!(
            unified(b"a\0\n", b"a\n", "f", 10, Some(deadline())),
            Err(Error::Encoding)
        );
        assert_eq!(
            unified(b"a\n", b"\0", "f", 10, Some(deadline())),
            Err(Error::Encoding)
        );
        assert_eq!(
            unified(b"\xff\n", b"a\n", "f", 10, Some(deadline())),
            Err(Error::Encoding)
        );
        assert_eq!(
            unified(b"a\n", b"\xc3", "f", 10, Some(deadline())),
            Err(Error::Encoding)
        );
        assert_eq!(
            unified(b"a\n", b"b\n", "f\ng", 10, Some(deadline())),
            Err(Error::Bound)
        );
        let past = Some(Instant::now());
        assert_eq!(unified(b"a\n", b"b\n", "f", 10, past), Err(Error::Deadline));
        assert_eq!(distance(&[b"a"], &[b"b"], 10, past), Err(Expired));
        assert_eq!(
            unified(b"a\n", b"b\n", "f", usize::MAX, None),
            Ok(b"--- a/f\n+++ b/f\n@@ -1 +1 @@\n-a\n+b\n".to_vec())
        );
        assert_eq!(distance(&[b"a"], &[b"b"], usize::MAX, None), Ok(Some(2)));
    }

    /// The search clamps to `MAX_EDITS`: a pair `MAX_EDITS + 2` edits apart is refused at any
    /// larger limit, and one exactly `MAX_EDITS` apart is found. The two constants are pinned to
    /// their sources, not read through their own names: 4096 is B14-P3's reviewed ceiling (review
    /// P3-1), and the 2 is what `a_final_newline_adds_exactly_two_edits` measures.
    #[test]
    fn the_search_never_exceeds_its_ceiling() {
        assert_eq!((MAX_CHANGED_LINES, MAX_EDITS), (4096, 4098));
        let lines: Vec<Vec<u8>> = (0..MAX_EDITS + 2)
            .map(|i| format!("{i}\n").into_bytes())
            .collect();
        let all: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
        assert_eq!(distance(&all, &[], usize::MAX, None), Ok(None));
        assert_eq!(
            distance(&all[..MAX_EDITS], &[], usize::MAX, None),
            Ok(Some(MAX_EDITS))
        );
    }
}
