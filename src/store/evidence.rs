//! `task.get`'s evidence views (B09a): each reference the ledger holds for a task, with the
//! identity it was recorded with — never invented at read.
//!
//! Design: `~/hee3-evidence/T28/B09-task-get-evidence-20260925/DESIGN.md` (R1, R2), after two
//! independent design reviews.
//!
//! * **Identity comes from where it was recorded.** An operator's disposition stores its
//!   `EvidenceRefV1`s as given (B08). An abandonment's stop is recoverable only by the join its
//!   event records — `disposition_id` → an abandonment of that task's attempt whose first reference
//!   names the stop's digest and size; a stop naming no disposition has no recorded identity, and
//!   one naming another kind of disposition is corruption. A verification or an acceptance has no
//!   recorded identity until the ledger keeps it (B09b, with B14, its first production writer):
//!   its views are refused, never guessed.
//! * **Bounded before it is read.** `refs` counts the dispositions' references in SQL
//!   (`json_array_length`) and refuses past the contract's 64 before any blob is fetched.
//! * **Rows here, objects after.** Rows are read in one snapshot; each object is verified after
//!   it, outside the ledger's lock, through an [`ObjectReader`] that holds only the generation's
//!   content-addressed object directory (objects are never deleted, so the check is sound).

use super::recovery::{TaskView, read_view};
use super::{Error, Object, Principal, Result, Store, artifact, read_number, remaining};
use crate::contracts::UuidV4;
use crate::contracts::control::{EvidenceRef, EvidenceView};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;
use std::time::Instant;

/// The contract's bound on an evidence array (`docs/contract-decisions.md`: "evidence arrays ≤64").
pub const MAX_VIEW_REFS: u64 = 64;

/// A read-only handle on one generation's object directory: no lock, no root, so it may outlive
/// the ledger's lock while it verifies (B09 R2.3).
#[derive(Debug)]
pub struct ObjectReader {
    objects: artifact::Directory,
}

impl ObjectReader {
    /// Verify the object `reference` names: present, of its size, of its SHA-256.
    ///
    /// # Errors
    /// `NotFound`/`Os`/`Io` for an absent object, `Corrupt` for other bytes, `Bound` past the
    /// object limit, `Invalid` for a malformed digest, `Deadline`.
    pub fn verify(&self, reference: &EvidenceRef, deadline: Instant) -> Result<()> {
        remaining(deadline)?;
        artifact::verify(
            &self.objects,
            &Object {
                digest: reference.sha256.clone(),
                size: reference.byte_length,
            },
        )?;
        remaining(deadline).map(drop)
    }
}

impl Store {
    /// A read-only handle on this generation's object directory (B09 R2.3).
    ///
    /// # Errors
    /// The directory could not be duplicated.
    pub fn object_reader(&self) -> Result<ObjectReader> {
        Ok(ObjectReader {
            objects: self.objects.try_clone()?,
        })
    }

    /// One task, scoped to `task` and visible to `principal` only, with the references `view`
    /// names — both from one read snapshot (B09).
    ///
    /// # Errors
    /// [`Store::task_view`]'s; `EvidenceIdentity` for evidence whose identity the ledger did not
    /// record; `EvidenceBound` past [`MAX_VIEW_REFS`]; `Corrupt` for a stored reference the wire
    /// would refuse or a stop whose disposition does not name it.
    pub fn task_evidence(
        &mut self,
        principal: &Principal,
        task: UuidV4<'_>,
        view: EvidenceView,
        deadline: Instant,
    ) -> Result<(TaskView, Vec<EvidenceRef>)> {
        self.read_snapshot(deadline, |db| {
            let read = read_view(db, principal, task, deadline)?;
            let references = read_evidence(db, task.as_str(), view, deadline)?;
            Ok((read, references))
        })
    }
}

fn read_evidence(
    db: &Connection,
    task: &str,
    view: EvidenceView,
    deadline: Instant,
) -> Result<Vec<EvidenceRef>> {
    let unrecorded: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM verifications v JOIN attempts a ON a.id=v.attempt_id \
         WHERE a.task_id=?1) OR EXISTS(SELECT 1 FROM acceptances WHERE task_id=?1)",
        [task],
        |row| row.get(0),
    )?;
    if unrecorded {
        return Err(Error::EvidenceIdentity);
    }
    let mut references: Vec<EvidenceRef> = stop_reference(db, task)?.into_iter().collect();
    if view == EvidenceView::Refs {
        let dispositions: u64 = db.query_row(
            "SELECT coalesce(sum(json_array_length(CAST(evidence AS TEXT))),0) \
             FROM task_dispositions WHERE task_id=?",
            [task],
            |row| read_number(row, 0),
        )?;
        let found = dispositions
            .checked_add(u64::try_from(references.len()).map_err(|_| Error::Bound)?)
            .ok_or(Error::Bound)?;
        if found > MAX_VIEW_REFS {
            return Err(Error::EvidenceBound {
                found,
                limit: MAX_VIEW_REFS,
            });
        }
        remaining(deadline)?;
        let mut statement = db.prepare(
            "SELECT d.evidence FROM task_dispositions d JOIN events e ON e.id=d.event_id \
             WHERE d.task_id=? ORDER BY e.sequence",
        )?;
        let rows = statement.query_map([task], |row| row.get::<_, Vec<u8>>(0))?;
        for row in rows {
            remaining(deadline)?;
            for reference in stored(&row?)? {
                // Collapse only an exact repeat: two references sharing an identity with other
                // members are both what was attested, and both are shown.
                if !references.contains(&reference) {
                    references.push(reference);
                }
            }
        }
    }
    Ok(references)
}

/// The references a disposition stored, each re-read by the wire's one reader; one it would
/// refuse is corruption.
fn stored(bytes: &[u8]) -> Result<Vec<EvidenceRef>> {
    let Value::Array(items) = serde_json::from_slice(bytes).map_err(|_| Error::Corrupt)? else {
        return Err(Error::Corrupt);
    };
    items
        .iter()
        .map(|item| EvidenceRef::parse(item).ok_or(Error::Corrupt))
        .collect()
}

/// The stop's evidence, recovered only through the disposition its event names (R2.1).
fn stop_reference(db: &Connection, task: &str) -> Result<Option<EvidenceRef>> {
    let Some((event, digest)) = db
        .query_row(
            "SELECT event_id,evidence_digest FROM task_stops WHERE task_id=?",
            [task],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?
    else {
        return Ok(None);
    };
    let disposition: Option<String> = db.query_row(
        "SELECT json_extract(CAST(body AS TEXT),'$.disposition_id') FROM events WHERE id=?",
        [&event],
        |row| row.get(0),
    )?;
    let Some(disposition) = disposition else {
        return Err(Error::EvidenceIdentity);
    };
    let evidence: Vec<u8> = db
        .query_row(
            "SELECT evidence FROM task_dispositions WHERE id=? AND task_id=? \
             AND disposition='abandon' AND obligation_kind='attempt'",
            params![disposition, task],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(Error::Corrupt)?;
    let first = stored(&evidence)?
        .into_iter()
        .next()
        .ok_or(Error::Corrupt)?;
    let size: u64 = db.query_row(
        "SELECT size FROM artifacts WHERE digest=?",
        [&digest],
        |row| read_number(row, 0),
    )?;
    if first.sha256 != digest || first.byte_length != size {
        return Err(Error::Corrupt);
    }
    Ok(Some(first))
}
