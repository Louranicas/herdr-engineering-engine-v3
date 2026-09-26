//! Run records: what the settle of an attempt commits about the run, keyed by the observation
//! that committed it (DS2; decision record `~/hee3-evidence/T00-plan-20260926/DS1-DS2.md` §2).
//!
//! The ledger cannot certify that a record is *true*. It certifies **who committed it and when**:
//! a record is committed only by the one call that observed attempt A settle, in that call's own
//! transaction, keyed by that call's event. Nothing earlier, later, or belonging to another attempt
//! can stand in for it, and a settled attempt admits no further observation
//! (`require_attempt(.., false)`), so the settling event's record set is final. A composer that
//! takes a [`CommittedRun`] cannot be handed a digest, so it cannot re-acquire a record from free
//! bytes. The residual, named: an in-crate caller of the settle door handing in objects it made up
//! is narrowed to one door and one production constructor per record type, not refused.

use super::{Error, Object, Result, number, read_number, register_evidence};
use crate::contracts::{Generation, Principal, UuidV4};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use std::collections::BTreeMap;

/// The kinds of run record one observation may commit. Closed: a new kind is a migration (the
/// `kind` CHECK) and a schema id here, together, so a kind/schema mismatch is unrepresentable.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RunRecordKind {
    /// When the run began and ended, by the runtime's monotonic clock.
    RunClock,
    /// How the run ended: exit, signal, timeout, cancellation.
    RunOutcome,
    /// What the teardown settled and what it could not.
    RunCleanup,
    /// The readbacks the runtime performed after the run.
    Readbacks,
    /// The bounded capture of the run's output.
    Capture,
}

impl RunRecordKind {
    /// Every kind, in the order the CHECK spells them.
    pub const ALL: [Self; 5] = [
        Self::RunClock,
        Self::RunOutcome,
        Self::RunCleanup,
        Self::Readbacks,
        Self::Capture,
    ];

    /// The stored spelling: the `kind` CHECK's vocabulary in `migrations/006.sql`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::RunClock => "run_clock",
            Self::RunOutcome => "run_outcome",
            Self::RunCleanup => "run_cleanup",
            Self::Readbacks => "readbacks",
            Self::Capture => "capture",
        }
    }

    /// The schema every object of this kind is decoded under (one home; never stored).
    #[must_use]
    pub const fn schema_id(self) -> &'static str {
        match self {
            Self::RunClock => "hee3.run-clock/1",
            Self::RunOutcome => "hee3.run-outcome/1",
            Self::RunCleanup => "hee3.run-cleanup/1",
            Self::Readbacks => "hee3.readbacks/1",
            Self::Capture => "hee3.capture/1",
        }
    }

    /// The kind a stored spelling names; `Corrupt` for any other, since the CHECK admits no other.
    fn parse(text: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.name() == text)
            .ok_or(Error::Corrupt)
    }
}

/// The media type every run record is published under.
pub const RECORD_MEDIA_TYPE: &str = "application/json";

/// One record a settle commits: the published object and the identity it is named by.
#[derive(Clone, Copy, Debug)]
pub struct RunRecord<'a> {
    /// What the object is.
    pub kind: RunRecordKind,
    /// The identity a receipt names it by.
    pub artifact_id: UuidV4<'a>,
    /// The object, as [`super::Store::publish`] returned it.
    pub object: &'a Object,
}

/// A record the ledger committed, read back from the ledger: never constructed from caller bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Committed {
    artifact_id: String,
    object: Object,
    kind: RunRecordKind,
}

impl Committed {
    /// The identity the record was committed under.
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    /// The object the record was committed as (digest and registered size).
    #[must_use]
    pub const fn object(&self) -> &Object {
        &self.object
    }

    /// The schema the object decodes under, derived from its kind.
    #[must_use]
    pub const fn schema_id(&self) -> &'static str {
        self.kind.schema_id()
    }
}

/// The record set of the observation that settled an attempt: exactly that event's set, never a
/// union over earlier observations. The only constructor is [`super::Store::committed_run`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedRun {
    task: String,
    attempt: String,
    attempt_generation: Generation,
    settled_event: String,
    settled_sequence: u64,
    records: BTreeMap<RunRecordKind, Committed>,
}

impl CommittedRun {
    /// The task the attempt belongs to.
    #[must_use]
    pub fn task(&self) -> &str {
        &self.task
    }

    /// The attempt whose settle committed these records.
    #[must_use]
    pub fn attempt(&self) -> &str {
        &self.attempt
    }

    /// The attempt's generation.
    #[must_use]
    pub const fn attempt_generation(&self) -> Generation {
        self.attempt_generation
    }

    /// The observation event that settled the attempt.
    #[must_use]
    pub fn settled_event(&self) -> &str {
        &self.settled_event
    }

    /// That event's ledger sequence.
    #[must_use]
    pub const fn settled_sequence(&self) -> u64 {
        self.settled_sequence
    }

    /// The committed record of `kind`, if the settling observation committed one.
    #[must_use]
    pub fn record(&self, kind: RunRecordKind) -> Option<&Committed> {
        self.records.get(&kind)
    }

    /// How many records the settling observation committed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the settling observation committed no record.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Whether `artifact_id` already names another digest anywhere identity is recorded — the write-time
/// half of migration 6's validate clause, one query for every identity-writing door (F-g).
pub(super) fn identity_bound_elsewhere(
    tx: &Connection,
    artifact_id: &str,
    digest: &str,
) -> Result<bool> {
    tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM (\
           SELECT artifact_id i, digest d FROM attempt_records \
           UNION ALL SELECT evidence_artifact_id, evidence_digest FROM verifications WHERE evidence_artifact_id IS NOT NULL \
           UNION ALL SELECT evidence_artifact_id, evidence_digest FROM task_stops WHERE evidence_artifact_id IS NOT NULL \
           UNION ALL SELECT artifact_id, digest FROM acceptance_objects WHERE artifact_id IS NOT NULL \
           UNION ALL SELECT manifest_artifact_id, manifest_digest FROM acceptances WHERE manifest_artifact_id IS NOT NULL\
         ) WHERE i=?1 AND d!=?2)",
        params![artifact_id, digest],
        |row| row.get(0),
    )
    .map_err(Error::from)
}

/// Commit `records` as observation `event_id`'s set for `attempt`, inside the settle's own
/// transaction, after the attempt's row was updated; mark the settling event when `settled`.
/// # Errors
/// `Invalid` for a repeated kind (refused before any write); `Conflict` for an artifact id already
/// bound to another digest, by the ledger or by another record of this observation;
/// `register_evidence`'s `Corrupt` and inventory refusals.
pub(super) fn commit(
    tx: &Transaction<'_>,
    attempt: &str,
    event_id: &str,
    records: &[RunRecord<'_>],
    settled: bool,
) -> Result<()> {
    let mut kinds = records.iter().map(|record| record.kind).collect::<Vec<_>>();
    kinds.sort_unstable();
    kinds.dedup();
    if kinds.len() != records.len() {
        return Err(Error::Invalid);
    }
    for (index, record) in records.iter().enumerate() {
        // Bound elsewhere in the ledger, or by an earlier record of this same observation: the
        // one rule, whichever door or slot holds the other binding (review of d60df83, gap 2).
        let within = records[..index].iter().any(|earlier| {
            earlier.artifact_id.as_str() == record.artifact_id.as_str()
                && earlier.object.digest() != record.object.digest()
        });
        if within
            || identity_bound_elsewhere(tx, record.artifact_id.as_str(), record.object.digest())?
        {
            return Err(Error::Conflict);
        }
    }
    let objects = records
        .iter()
        .map(|record| record.object.clone())
        .collect::<Vec<_>>();
    register_evidence(tx, &objects)?;
    for record in records {
        tx.execute(
            "INSERT INTO attempt_records(event_id,kind,attempt_id,digest,artifact_id) VALUES(?,?,?,?,?)",
            params![
                event_id,
                record.kind.name(),
                attempt,
                record.object.digest(),
                record.artifact_id.as_str()
            ],
        )?;
    }
    if settled {
        tx.execute(
            "UPDATE attempts SET settled_event=? WHERE id=?",
            params![event_id, attempt],
        )?;
    }
    Ok(())
}

/// The settling observation's record set for `attempt`, visible to `principal` only.
/// # Errors
/// `NotFound` when the attempt is not one of the principal's tasks'; `Outstanding` when it is not
/// settled; `EvidenceIdentity` when it settled before migration 6 (no settling event recorded);
/// `Corrupt` for a row the rules refuse.
pub(super) fn committed_run(
    db: &Connection,
    principal: &Principal,
    attempt: &str,
) -> Result<CommittedRun> {
    let row: Option<(String, String, String, Option<String>)> = db
        .query_row(
            "SELECT a.task_id,a.generation,a.state,a.settled_event FROM attempts a \
             JOIN tasks t ON t.id=a.task_id \
             WHERE a.id=? AND t.principal_uid=? AND t.principal_role=?",
            params![attempt, principal.uid(), principal.role()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?;
    let (task, generation, state, settled_event) = row.ok_or(Error::NotFound)?;
    if state != "settled" {
        return Err(Error::Outstanding);
    }
    let settled_event = settled_event.ok_or(Error::EvidenceIdentity)?;
    let attempt_generation: Generation = generation.parse().map_err(|_| Error::Corrupt)?;
    let settled_sequence: u64 = db.query_row(
        "SELECT sequence FROM events WHERE id=? AND task_id=? AND kind='attempt_observed'",
        params![settled_event, task],
        |row| read_number(row, 0),
    )?;
    let mut statement = db.prepare(
        "SELECT r.kind,r.artifact_id,r.digest,f.size FROM attempt_records r \
         JOIN artifacts f ON f.digest=r.digest \
         WHERE r.event_id=? AND r.attempt_id=? ORDER BY r.kind LIMIT ?",
    )?;
    let bound = number(RunRecordKind::ALL.len() as u64 + 1)?;
    let rows = statement
        .query_map(params![settled_event, attempt, bound], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                read_number(row, 3)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut records = BTreeMap::new();
    for (kind, artifact_id, digest, size) in rows {
        let kind = RunRecordKind::parse(&kind)?;
        let committed = Committed {
            artifact_id,
            object: Object { digest, size },
            kind,
        };
        if records.insert(kind, committed).is_some() {
            return Err(Error::Corrupt);
        }
    }
    Ok(CommittedRun {
        task,
        attempt: attempt.to_owned(),
        attempt_generation,
        settled_event,
        settled_sequence,
        records,
    })
}
