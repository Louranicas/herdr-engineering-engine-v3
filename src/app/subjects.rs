//! Publish a captured workspace snapshot as a closed RC04 `SubjectV1` graph.
//! Paths and bytes come only from the snapshot; this adapter opens no candidate path.

use crate::app::evidence::{self, Evidence};
use crate::check::collector::{self, Publisher};
use crate::check::graph::{self, Graph};
use crate::contracts::receipt::{
    Id, List, Maybe, Ref, RelPath, Sha, SubjectFilePageV1, SubjectFileV1, SubjectFileV1Kind,
    SubjectFileV1Origin, SubjectV1, Text, TypedRef, decode,
};
use crate::worker::workspace::{self, Content, Snapshot};
use std::time::Instant;

const MAX_ENTRIES: usize = 4096;
const PAGE_ROWS: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Deadline,
    Snapshot(workspace::Error),
    Scalar,
    Publication(collector::Error),
    ExcludedUnsupported,
    Graph(graph::Error),
    Mismatch,
}

/// Failed typed references remain named for reconciliation. Payload objects already
/// published by `Evidence` remain in its registered or pending-publication views.
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub attempted: Vec<Ref>,
}

fn failure(kind: ErrorKind) -> Error {
    Error {
        kind,
        attempted: Vec::new(),
    }
}

fn text(value: &str) -> Result<Text, Error> {
    Text::new(value).map_err(|_| failure(ErrorKind::Scalar))
}

fn unavailable<T>(reason: &str) -> Result<Maybe<T>, Error> {
    Ok(Maybe::unavailable(text(reason)?))
}

fn record<T: crate::contracts::receipt::ReceiptRecord>(
    publisher: &mut Publisher<'_, Evidence<'_>>,
    value: &T,
) -> Result<TypedRef<T>, Error> {
    publisher.record(value).map_err(|error| Error {
        kind: ErrorKind::Publication(error),
        attempted: publisher.attempted_refs().to_vec(),
    })
}

fn pages(
    publisher: &mut Publisher<'_, Evidence<'_>>,
    rows: &[SubjectFileV1],
) -> Result<TypedRef<SubjectFilePageV1>, Error> {
    let page_count = rows.len().div_ceil(PAGE_ROWS).max(1);
    let count = u32::try_from(page_count).map_err(|_| failure(ErrorKind::Scalar))?;
    let total = u32::try_from(rows.len()).map_err(|_| failure(ErrorKind::Scalar))?;
    let mut next = unavailable("end_of_inventory")?;
    for index in (0..page_count).rev() {
        let start = index * PAGE_ROWS;
        let end = (start + PAGE_ROWS).min(rows.len());
        let page = SubjectFilePageV1 {
            page_index: u32::try_from(index).map_err(|_| failure(ErrorKind::Scalar))?,
            page_count: count,
            row_count: u32::try_from(end - start).map_err(|_| failure(ErrorKind::Scalar))?,
            total_rows: total,
            rows: List::new(rows[start..end].to_vec()).map_err(|_| failure(ErrorKind::Scalar))?,
            next,
        };
        next = Maybe::present(record(publisher, &page)?);
    }
    next.value.ok_or_else(|| failure(ErrorKind::Scalar))
}

/// Publish exact snapshot payloads, chained file pages, and the `SubjectV1` root.
/// `origin=Excluded` is refused because this adapter has no exclusion policy or reason.
///
/// # Errors
/// Refuses changed sources, expiry, unsupported excluded origin, scalar/bounds errors,
/// or any publication/readback failure. Published objects are never claimed rolled back.
pub fn publish(
    evidence: &mut Evidence<'_>,
    snapshot: &Snapshot,
    origin: SubjectFileV1Origin,
    deadline: Instant,
) -> Result<TypedRef<SubjectV1>, Error> {
    if Instant::now() >= deadline {
        return Err(failure(ErrorKind::Deadline));
    }
    if origin == SubjectFileV1Origin::Excluded {
        return Err(failure(ErrorKind::ExcludedUnsupported));
    }
    snapshot
        .readback_source(deadline)
        .map_err(|error| failure(ErrorKind::Snapshot(error)))?;
    let entries: Vec<_> = snapshot.entries().collect();
    if entries.len() > MAX_ENTRIES {
        return Err(failure(ErrorKind::Scalar));
    }
    let mut rows = Vec::with_capacity(entries.len());
    for entry in entries {
        if Instant::now() >= deadline {
            return Err(failure(ErrorKind::Deadline));
        }
        let (kind, content, executable) = match &entry.content {
            Content::Directory => (
                SubjectFileV1Kind::Directory,
                unavailable("directory_has_no_content")?,
                false,
            ),
            Content::File {
                bytes, executable, ..
            } => (
                SubjectFileV1Kind::File,
                Maybe::present(
                    evidence
                        .payload(bytes, "application/octet-stream")
                        .map_err(|error| {
                            failure(ErrorKind::Publication(collector::Error::Sink(error)))
                        })?,
                ),
                *executable,
            ),
        };
        rows.push(SubjectFileV1 {
            path: RelPath::new(entry.path.clone()).map_err(|_| failure(ErrorKind::Scalar))?,
            kind,
            content,
            executable,
            link_target: unavailable("not_a_symlink")?,
            origin,
            exclusion_reason: unavailable("not_excluded")?,
        });
    }
    snapshot
        .readback_source(deadline)
        .map_err(|error| failure(ErrorKind::Snapshot(error)))?;
    let subject_id: Id = evidence::fresh_id(deadline)
        .map_err(|error| failure(ErrorKind::Publication(collector::Error::Sink(error))))?;
    let mut publisher = Publisher::new(evidence);
    let files = pages(&mut publisher, &rows)?;
    let subject = SubjectV1 {
        subject_id,
        tree_sha256: Sha::new(files.as_ref().sha256.as_str().to_owned()).map_err(|_| Error {
            kind: ErrorKind::Scalar,
            attempted: publisher.attempted_refs().to_vec(),
        })?,
        files,
        dirty_patch: unavailable("dirty_patch_unavailable")?,
    };
    record(&mut publisher, &subject)
}

/// Bind a previously published subject to the actual frozen source before dispatch
/// and after collection. Random publication IDs do not replace file comparisons.
/// The complete retained graph, path/type/origin/executable fields and exact file
/// bytes must agree. This fixed workspace profile has no dirty-patch metadata.
///
/// # Errors
/// Refuses missing/corrupt graph objects, source changes, expired observation,
/// unsupported origin, or any inventory/content mismatch. No objects are written.
pub fn verify(
    evidence: &Evidence<'_>,
    snapshot: &Snapshot,
    origin: SubjectFileV1Origin,
    reference: &TypedRef<SubjectV1>,
    deadline: Instant,
) -> Result<(), Error> {
    if origin == SubjectFileV1Origin::Excluded {
        return Err(failure(ErrorKind::ExcludedUnsupported));
    }
    snapshot
        .readback_source(deadline)
        .map_err(|error| failure(ErrorKind::Snapshot(error)))?;
    let graph = Graph::resolve(evidence, reference.as_ref())
        .map_err(|error| failure(ErrorKind::Graph(error)))?;
    let root: SubjectV1 = decode(
        graph
            .get(reference.as_ref())
            .map_err(|error| failure(ErrorKind::Graph(error)))?
            .bytes(),
    )
    .map_err(|_| failure(ErrorKind::Scalar))?;
    if root.dirty_patch.value.is_some() {
        return Err(failure(ErrorKind::Mismatch));
    }
    let mut entries = snapshot.entries();
    let mut next = Some(root.files);
    while let Some(reference) = next {
        if Instant::now() >= deadline {
            return Err(failure(ErrorKind::Deadline));
        }
        let page: SubjectFilePageV1 = decode(
            graph
                .get(reference.as_ref())
                .map_err(|error| failure(ErrorKind::Graph(error)))?
                .bytes(),
        )
        .map_err(|_| failure(ErrorKind::Scalar))?;
        for row in page.rows.as_slice() {
            let entry = entries.next().ok_or_else(|| failure(ErrorKind::Mismatch))?;
            if row.path.as_str() != entry.path || row.origin != origin {
                return Err(failure(ErrorKind::Mismatch));
            }
            match &entry.content {
                Content::Directory if row.kind == SubjectFileV1Kind::Directory => {}
                Content::File {
                    bytes, executable, ..
                } if row.kind == SubjectFileV1Kind::File && row.executable == *executable => {
                    let payload = row
                        .content
                        .value
                        .as_ref()
                        .ok_or_else(|| failure(ErrorKind::Mismatch))?;
                    if graph
                        .get(payload.as_ref())
                        .map_err(|error| failure(ErrorKind::Graph(error)))?
                        .bytes()
                        != bytes
                    {
                        return Err(failure(ErrorKind::Mismatch));
                    }
                }
                _ => return Err(failure(ErrorKind::Mismatch)),
            }
        }
        next = page.next.value;
    }
    if entries.next().is_some() {
        return Err(failure(ErrorKind::Mismatch));
    }
    snapshot
        .readback_source(deadline)
        .map_err(|error| failure(ErrorKind::Snapshot(error)))
}
