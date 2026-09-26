//! Run records: what an observation of an attempt commits about a run, keyed by the observation
//! that committed it (DS2; decision record `~/hee3-evidence/T00-plan-20260926/DS1-DS2.md` §2;
//! R13 in `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`).
//!
//! Two observations commit records: the **settle** of the attempt's work (`attempt_observed`), and
//! the **check** of its applied candidate (`verification_observed`), whose run is the workload in
//! the U64 class. The ledger cannot certify that a record is *true*. It certifies **who committed
//! it and when**: a record is committed only by the one call that observed attempt A settle or be
//! checked, in that call's own transaction, keyed by that call's event. Nothing earlier, later, or
//! belonging to another attempt can stand in for it. A settled attempt admits no further
//! observation (`require_attempt(.., false)`) and an attempt has one verification
//! (`verifications.attempt_id` is its key), so both sets are final. A composer that takes a
//! [`CommittedRun`] or a [`CommittedCheck`] cannot be handed a digest, so it cannot re-acquire a
//! record from free bytes. The residual, named: an in-crate caller of either door handing in
//! objects it made up is narrowed to one door and one production constructor per record type, not
//! refused.

use super::evidence::reference_row;
use super::verification::{VerificationVerdict, parse_verdict};
use super::{Error, Object, Result, number, read_number, register_evidence};
use crate::contracts::control::EvidenceRef;
use crate::contracts::{Generation, Principal, Sha256Digest, UuidV4};
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
    /// What the worker's one call to the model came to: usage, identity, wall, how it ended
    /// (B14a-5, R19). Committed by the attempt's settle, never by a check — a rule `commit` keeps
    /// (`Invalid` from the check door), not a comment.
    WorkerSettle,
}

impl RunRecordKind {
    /// Every kind, in the order the CHECK spells them.
    pub const ALL: [Self; 6] = [
        Self::RunClock,
        Self::RunOutcome,
        Self::RunCleanup,
        Self::Readbacks,
        Self::Capture,
        Self::WorkerSettle,
    ];

    /// The stored spelling: the `kind` CHECK's vocabulary in `migrations/007.sql` (006's, widened).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::RunClock => "run_clock",
            Self::RunOutcome => "run_outcome",
            Self::RunCleanup => "run_cleanup",
            Self::Readbacks => "readbacks",
            Self::Capture => "capture",
            Self::WorkerSettle => "worker_settle",
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
            Self::WorkerSettle => "hee3.worker-settle/1",
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

/// Which observation committed a record: the attempt's settle, or its check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Observation {
    /// The settle of the attempt's work (`attempt_observed`).
    Settle,
    /// The check of its applied candidate (`verification_observed`).
    Check,
}

/// A record the ledger committed, read back from the ledger: never constructed from caller bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Committed {
    artifact_id: String,
    object: Object,
    kind: RunRecordKind,
    observation: Observation,
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

    /// The kind the observation committed the record under.
    #[must_use]
    pub const fn kind(&self) -> RunRecordKind {
        self.kind
    }

    /// Which observation committed it (R13): a settle record never reads as a check record.
    #[must_use]
    pub const fn observation(&self) -> Observation {
        self.observation
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
    observation: Observation,
) -> Result<()> {
    let mut kinds = records.iter().map(|record| record.kind).collect::<Vec<_>>();
    kinds.sort_unstable();
    kinds.dedup();
    if kinds.len() != records.len() {
        return Err(Error::Invalid);
    }
    // The worker's settle is the attempt's settle's to commit (B14a-5, R19 round 2 finding 5): a
    // check that hands one in is refused here, inside the door's transaction, which rolls back the
    // verification row and the task update already written — nothing is committed. The settle door
    // stays the generic run-record door DS2 specified (the five kinds have committed there since
    // B14a-2b-ii).
    if observation == Observation::Check && kinds.contains(&RunRecordKind::WorkerSettle) {
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
            observation: Observation::Settle,
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

/// The record set the check of an attempt committed (R13): exactly the verification's set, with the
/// verification's own facts from the same snapshot. The only constructor is
/// [`super::Store::committed_check`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedCheck {
    task: String,
    attempt: String,
    verification_event: String,
    verification_sequence: u64,
    verdict: VerificationVerdict,
    subject: String,
    evidence: EvidenceRef,
    records: BTreeMap<RunRecordKind, Committed>,
}

impl CommittedCheck {
    /// The task the attempt belongs to.
    #[must_use]
    pub fn task(&self) -> &str {
        &self.task
    }

    /// The attempt whose check committed these records.
    #[must_use]
    pub fn attempt(&self) -> &str {
        &self.attempt
    }

    /// The verification event that committed them.
    #[must_use]
    pub fn verification_event(&self) -> &str {
        &self.verification_event
    }

    /// That event's ledger sequence.
    #[must_use]
    pub const fn verification_sequence(&self) -> u64 {
        self.verification_sequence
    }

    /// The verdict the verification recorded, read back through the table that wrote it.
    #[must_use]
    pub const fn verdict(&self) -> VerificationVerdict {
        self.verdict
    }

    /// The subject digest the verification bound: a `Sha256Digest` rendering, validated at read
    /// (`Corrupt` otherwise); borrowed because the digest type owns no bytes.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The verification's evidence (the receipt) as the reference it was recorded with — its own
    /// artifact id, media type and schema, never a run record's.
    #[must_use]
    pub const fn evidence(&self) -> &EvidenceRef {
        &self.evidence
    }

    /// The committed record of `kind`, if the check committed one.
    #[must_use]
    pub fn record(&self, kind: RunRecordKind) -> Option<&Committed> {
        self.records.get(&kind)
    }

    /// How many records the check committed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the check committed no record.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// One verification row as `committed_check` reads it: the facts the check committed beside its
/// records, and the event's own kind and task so the row is refused rather than trusted.
struct VerificationRow {
    event: String,
    sequence: u64,
    event_kind: String,
    event_task: String,
    verdict: String,
    subject: String,
    /// The receipt's reference, `None` when the row predates migration 6.
    evidence: Option<serde_json::Value>,
}

/// The check's record set for `attempt`, visible to `principal` only (R13).
/// # Errors
/// `NotFound` when the attempt is not one of the principal's tasks'; `Outstanding` when it has no
/// verification; `EvidenceIdentity` when the verification recorded no identity (before migration
/// 6); `Corrupt` for a row the rules refuse.
pub(super) fn committed_check(
    db: &Connection,
    principal: &Principal,
    attempt: &str,
) -> Result<CommittedCheck> {
    let task: String = db
        .query_row(
            "SELECT a.task_id FROM attempts a JOIN tasks t ON t.id=a.task_id \
             WHERE a.id=? AND t.principal_uid=? AND t.principal_role=?",
            params![attempt, principal.uid(), principal.role()],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(Error::NotFound)?;
    let row = db
        .query_row(
            "SELECT v.evidence_artifact_id,v.evidence_digest,f.size,v.evidence_media_type,v.evidence_schema_id, \
                    v.event_id,e.sequence,e.kind,e.task_id,v.verdict,v.subject_digest \
             FROM verifications v JOIN events e ON e.id=v.event_id JOIN artifacts f ON f.digest=v.evidence_digest \
             WHERE v.attempt_id=?",
            [attempt],
            |row| {
                // The receipt's reference, built by the one function every stored reference is
                // read through (`reference_row`); NULL identity is a row from before migration 6.
                let identified: Option<String> = row.get(0)?;
                let evidence = match identified {
                    Some(_) => Some(reference_row(row)?),
                    None => None,
                };
                Ok(VerificationRow {
                    event: row.get(5)?,
                    sequence: read_number(row, 6)?,
                    event_kind: row.get(7)?,
                    event_task: row.get(8)?,
                    verdict: row.get(9)?,
                    subject: row.get(10)?,
                    evidence,
                })
            },
        )
        .optional()?
        .ok_or(Error::Outstanding)?;
    // The row is read back, not trusted: its event is the verification's own, of this task.
    if row.event_kind != "verification_observed" || row.event_task != task {
        return Err(Error::Corrupt);
    }
    let evidence = row.evidence.ok_or(Error::EvidenceIdentity)?;
    let evidence = EvidenceRef::parse(&evidence).ok_or(Error::Corrupt)?;
    let verdict = parse_verdict(&row.verdict)?;
    Sha256Digest::parse(&row.subject).map_err(|_| Error::Corrupt)?;
    let (verification_event, verification_sequence, subject) =
        (row.event, row.sequence, row.subject);
    let mut statement = db.prepare(
        "SELECT r.kind,r.artifact_id,r.digest,f.size FROM attempt_records r \
         JOIN artifacts f ON f.digest=r.digest \
         WHERE r.event_id=? AND r.attempt_id=? ORDER BY r.kind LIMIT ?",
    )?;
    let bound = number(RunRecordKind::ALL.len() as u64 + 1)?;
    let rows = statement
        .query_map(params![verification_event, attempt, bound], |row| {
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
            observation: Observation::Check,
        };
        if records.insert(kind, committed).is_some() {
            return Err(Error::Corrupt);
        }
    }
    Ok(CommittedCheck {
        task,
        attempt: attempt.to_owned(),
        verification_event,
        verification_sequence,
        verdict,
        subject,
        evidence,
        records,
    })
}
