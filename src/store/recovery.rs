//! Bounded readback of durable task obligations, without reconciliation authority.

use super::{
    Error, Result, Store, TaskHead, read_number, read_optional_number, remaining, schema, task_row,
};
use crate::contracts::{
    Generation, Sha256Digest, UuidV4,
    roster::{Instance, Pin},
};
use rusqlite::{Connection, Row, TransactionBehavior, types::ValueRef};
use std::time::Instant;

/// Aggregate rows and raw SQLite payload bytes, not a serialized JSON limit.
/// Fixed Rust row overhead is additionally bounded by the 1024-row hard cap.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryLimits {
    pub rows: usize,
    pub bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableTask {
    pub head: TaskHead,
    pub principal_uid: u32,
    pub principal_role: String,
    pub limit_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableAttempt {
    pub id: String,
    pub task: String,
    pub generation: String,
    pub state: String,
    pub effect: String,
    pub cleanup: String,
    pub used_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableVerification {
    pub attempt: String,
    pub event: String,
    pub subject: String,
    pub evidence: String,
    pub verdict: String,
    pub used_ms: Option<u64>,
    pub cleanup_settled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableAcceptance {
    pub event: String,
    pub task: String,
    pub attempt: String,
    pub generation: String,
    pub criteria: String,
    pub manifest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableStop {
    pub task: String,
    pub event: String,
    pub evidence: String,
    pub reason: String,
    pub state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingDelivery {
    pub event: String,
    pub recipient: String,
    pub sequence: u64,
    pub task: Option<String>,
    pub roster: Option<String>,
}

/// Complete within these explicitly named tables, or no result is returned.
/// Includes settled and terminal histories: these are not instructions to dispatch.
/// OS process identity, Pi queues, external effects and temporary artifact custody
/// are not persisted by this schema and cannot be inferred from this inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryInventory {
    pub epoch: String,
    pub generation: String,
    pub mode: String,
    pub event_high_water: u64,
    pub tasks: Vec<DurableTask>,
    pub attempts: Vec<DurableAttempt>,
    pub verifications: Vec<DurableVerification>,
    pub acceptances: Vec<DurableAcceptance>,
    pub stops: Vec<DurableStop>,
    pub pending_delivery: Vec<PendingDelivery>,
    pub instances: Vec<Instance>,
    pub pins: Vec<Pin>,
    pub rows: usize,
    pub payload_bytes: usize,
}

impl RecoveryInventory {
    /// Compare a supplied cursor with this same complete transaction's epoch/cutoff.
    /// Equal epochs still confer no visibility/filter/retention or replay authority.
    /// `now_unix_ms` must be the caller's current clock observation, not cursor data.
    /// # Errors
    /// Refuses malformed, expired, future-issued or same-epoch future-sequence cursors.
    pub fn cursor_disposition(
        &self,
        cursor: &crate::contracts::events::EventCursorV1,
        now_unix_ms: u64,
    ) -> Result<crate::contracts::events::CursorDisposition> {
        use crate::contracts::{events::CursorDisposition, parse_u64_decimal};
        cursor.validate().map_err(|_| Error::Invalid)?;
        let issued = parse_u64_decimal(&cursor.issued_unix_ms).map_err(|_| Error::Invalid)?;
        let expires = parse_u64_decimal(&cursor.expires_unix_ms).map_err(|_| Error::Invalid)?;
        if issued > now_unix_ms || expires <= now_unix_ms || issued >= expires {
            return Err(Error::Invalid);
        }
        if cursor.epoch != self.epoch {
            return Ok(CursorDisposition::ResyncRequired);
        }
        if parse_u64_decimal(&cursor.sequence).map_err(|_| Error::Invalid)? > self.event_high_water
        {
            return Err(Error::Invalid);
        }
        Ok(CursorDisposition::SnapshotOnly)
    }
}

struct Budget {
    limits: RecoveryLimits,
    rows: usize,
    bytes: usize,
    deadline: Instant,
}
impl Budget {
    fn charge(&mut self, row: &Row<'_>) -> Result<()> {
        remaining(self.deadline)?;
        self.rows = self.rows.checked_add(1).ok_or(Error::Bound)?;
        if self.rows > self.limits.rows {
            return Err(Error::Bound);
        }
        for index in 0..row.as_ref().column_count() {
            let length = match row.get_ref(index)? {
                ValueRef::Null => 0,
                ValueRef::Integer(_) | ValueRef::Real(_) => 8,
                ValueRef::Text(bytes) | ValueRef::Blob(bytes) => bytes.len(),
            };
            self.bytes = self.bytes.checked_add(length).ok_or(Error::Bound)?;
            if self.bytes > self.limits.bytes {
                return Err(Error::Bound);
            }
        }
        Ok(())
    }
    fn read<T>(
        &mut self,
        db: &Connection,
        sql: &str,
        mut parse: impl FnMut(&Row<'_>) -> Result<T>,
    ) -> Result<Vec<T>> {
        let limit = self.limits.rows - self.rows + 1;
        let mut statement = db.prepare(sql)?;
        let mut rows = statement.query([u32::try_from(limit).map_err(|_| Error::Bound)?])?;
        let mut values = Vec::new();
        while let Some(row) = rows.next()? {
            self.charge(row)?; // Check payload before any owned row allocation/decode.
            values.push(parse(row)?);
        }
        remaining(self.deadline)?;
        Ok(values)
    }
}

impl Store {
    /// Read one consistent bounded snapshot using this Store's existing connection.
    /// This internal owner surface does not authorize reconciliation or dispatch.
    /// All durable tasks/attempts/checks/stops/acceptances, current instances/pins and
    /// pending outbox rows are included. Artifact availability, operation replay and
    /// historical roster events require their existing owners' separate readback.
    ///
    /// # Errors
    /// Refuses stale epoch, poisoned custody, malformed rows, deadline expiry, or
    /// aggregate bounds outside 1..=1024 rows and 1..=1048576 payload bytes.
    /// Oversized snapshots never return a successful partial page. Read-transaction
    /// rollback errors poison this Store and preserve an earlier read error.
    pub fn recovery_inventory(
        &mut self,
        expected_epoch: UuidV4<'_>,
        limits: RecoveryLimits,
        deadline: Instant,
    ) -> Result<RecoveryInventory> {
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        if expected_epoch.as_str() != self.epoch {
            return Err(Error::Conflict);
        }
        if !(1..=1024).contains(&limits.rows) || !(1..=1_048_576).contains(&limits.bytes) {
            return Err(Error::Bound);
        }
        schema::bound(&self.connection, deadline)?;
        let generation = self
            .generation
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(Error::Custody)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        let result = collect(&tx, expected_epoch.as_str(), generation, limits, deadline);
        match (result, tx.rollback()) {
            (result, Ok(())) => result,
            (Err(original), Err(failure)) => {
                self.poisoned = true;
                Err(Error::Rollback {
                    original: Box::new(original),
                    failure,
                })
            }
            (Ok(_), Err(failure)) => {
                self.poisoned = true;
                Err(Error::Sqlite(failure))
            }
        }
    }
}

fn uuid(value: &str) -> Result<()> {
    UuidV4::parse(value).map(|_| ()).map_err(|_| Error::Corrupt)
}
fn revision(value: &str) -> Result<()> {
    value
        .parse::<Generation>()
        .map(|_| ())
        .map_err(|_| Error::Corrupt)
}
fn digest(value: &str) -> Result<()> {
    Sha256Digest::parse(value)
        .map(|_| ())
        .map_err(|_| Error::Corrupt)
}
fn one_of(value: &str, names: &[&str]) -> Result<()> {
    if names.contains(&value) {
        Ok(())
    } else {
        Err(Error::Corrupt)
    }
}
fn flag(row: &Row<'_>, index: usize) -> Result<bool> {
    match row.get::<_, i64>(index)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Error::Corrupt),
    }
}

fn collect(
    db: &Connection,
    epoch: &str,
    generation: &str,
    limits: RecoveryLimits,
    deadline: Instant,
) -> Result<RecoveryInventory> {
    let mut budget = Budget {
        limits,
        rows: 0,
        bytes: 0,
        deadline,
    };
    let mut metadata = budget.read(db, "SELECT epoch,generation,mode,(SELECT coalesce(max(sequence),0) FROM events) FROM ledger_meta WHERE singleton=1 LIMIT ?", |row| {
        let e: String=row.get(0)?; let g: String=row.get(1)?; let mode: String=row.get(2)?;
        uuid(&e)?; uuid(&g)?; one_of(&mode,&["normal","reconciliation"])?;
        if e!=epoch || g!=generation { return Err(Error::Conflict); }
        Ok((e,g,mode,read_number(row,3)?))
    })?;
    let (epoch, generation, mode, event_high_water) = metadata.pop().ok_or(Error::Corrupt)?;
    let tasks=budget.read(db,"SELECT id,generation,state,cancellation,accepted_event,criteria_digest,spent_ms,reserved_work_ms,reserved_verify_ms,principal_uid,principal_role,limit_ms FROM tasks ORDER BY id LIMIT ?",|row| {
        let head=task_row(row)?; uuid(&head.id)?; revision(&head.generation)?; digest(&head.criteria)?; flag(row,3)?;
        one_of(&head.state,&["admitted","queued","running","verifying","repair_pending","cancellation_requested","blocked","accepted","failed","cancelled","abandoned","effect_unknown"])?;
        if let Some(event)=&head.accepted_event { uuid(event)?; }
        if (head.state=="accepted") != head.accepted_event.is_some() { return Err(Error::Corrupt); }
        let principal_uid=row.get(9)?; let principal_role:String=row.get(10)?;
        super::Principal::new(principal_uid,&principal_role).map_err(|_| Error::Corrupt)?;
        let limit_ms=read_number(row,11)?;
        if limit_ms==0 || head.spent_ms.checked_add(head.reserved_work_ms).and_then(|n|n.checked_add(head.reserved_verify_ms)).is_none_or(|n|n>limit_ms) { return Err(Error::Corrupt); }
        Ok(DurableTask{head,principal_uid,principal_role,limit_ms})
    })?;
    let attempts=budget.read(db,"SELECT id,task_id,generation,state,effect,cleanup,used_ms FROM attempts ORDER BY task_id,id LIMIT ?",|row| {
        let r=DurableAttempt{id:row.get(0)?,task:row.get(1)?,generation:row.get(2)?,state:row.get(3)?,effect:row.get(4)?,cleanup:row.get(5)?,used_ms:read_optional_number(row,6)?};
        uuid(&r.id)?;uuid(&r.task)?;revision(&r.generation)?;
        one_of(&r.state,&["queued","running","settled","unknown"])?;one_of(&r.effect,&["none","committed","pending","unknown"])?;one_of(&r.cleanup,&["none","pending","settled","unknown"])?;
        if r.state=="settled" && (!matches!(r.effect.as_str(),"none"|"committed") || r.cleanup!="settled" || r.used_ms.is_none()) { return Err(Error::Corrupt); }
        Ok(r)
    })?;
    let verifications=budget.read(db,"SELECT attempt_id,event_id,subject_digest,evidence_digest,verdict,used_ms,cleanup_settled FROM verifications ORDER BY attempt_id LIMIT ?",|row| {
        let r=DurableVerification{attempt:row.get(0)?,event:row.get(1)?,subject:row.get(2)?,evidence:row.get(3)?,verdict:row.get(4)?,used_ms:read_optional_number(row,5)?,cleanup_settled:flag(row,6)?};
        uuid(&r.attempt)?;uuid(&r.event)?;digest(&r.subject)?;digest(&r.evidence)?;one_of(&r.verdict,&["passed","failed","invalid","error","timeout","cancelled"])?;Ok(r)
    })?;
    let acceptances=budget.read(db,"SELECT event_id,task_id,attempt_id,generation,criteria_digest,manifest_digest FROM acceptances ORDER BY task_id LIMIT ?",|row| {
        let r=DurableAcceptance{event:row.get(0)?,task:row.get(1)?,attempt:row.get(2)?,generation:row.get(3)?,criteria:row.get(4)?,manifest:row.get(5)?};
        uuid(&r.event)?;uuid(&r.task)?;uuid(&r.attempt)?;revision(&r.generation)?;digest(&r.criteria)?;digest(&r.manifest)?;Ok(r)
    })?;
    let stops=budget.read(db,"SELECT task_id,event_id,evidence_digest,reason,state FROM task_stops ORDER BY task_id LIMIT ?",|row| {
        let r=DurableStop{task:row.get(0)?,event:row.get(1)?,evidence:row.get(2)?,reason:row.get(3)?,state:row.get(4)?};
        uuid(&r.task)?;uuid(&r.event)?;digest(&r.evidence)?;crate::contracts::receipt::Name::new(r.reason.clone()).map_err(|_|Error::Corrupt)?;one_of(&r.state,&["failed","cancelled"])?;Ok(r)
    })?;
    let pending_delivery=budget.read(db,"SELECT o.event_id,o.recipient,e.sequence,e.task_id,e.roster_id FROM outbox o JOIN events e ON e.id=o.event_id WHERE o.delivered=0 ORDER BY e.sequence,o.recipient LIMIT ?",|row| {
        let r=PendingDelivery{event:row.get(0)?,recipient:row.get(1)?,sequence:read_number(row,2)?,task:row.get(3)?,roster:row.get(4)?};
        uuid(&r.event)?;for id in [&r.task,&r.roster].into_iter().flatten(){uuid(id)?;}
        if r.task.is_some()==r.roster.is_some() || r.sequence==0 || r.sequence>event_high_water || r.recipient.is_empty() {return Err(Error::Corrupt);}
        Ok(r)
    })?;
    let instances=budget.read(db,"SELECT id,task_id,attempt_id,agent_record_id,agent_record_version,revision,body FROM roster_instances ORDER BY id LIMIT ?",|row| {
        let r:Instance=serde_json::from_slice(row.get_ref(6)?.as_blob().map_err(|_|Error::Corrupt)?)?;
        for id in [&r.id,&r.task_id,&r.attempt_id,&r.agent_record_id,&r.session_id,&r.workspace_ref,&r.started.epoch]{uuid(id)?;}
        for v in [&r.generation,&r.revision,&r.agent_record_version,&r.attempt_generation]{revision(v)?;}
        for (index,value) in [&r.id,&r.task_id,&r.attempt_id,&r.agent_record_id,&r.agent_record_version,&r.revision].into_iter().enumerate(){if row.get_ref(index)?.as_str().map_err(|_|Error::Corrupt)?!=value {return Err(Error::Corrupt);}}
        Ok(r)
    })?;
    let pins=budget.read(db,"SELECT attempt_id,record_id,record_version,body FROM roster_pins ORDER BY attempt_id,record_id LIMIT ?",|row| {
        let r:Pin=serde_json::from_slice(row.get_ref(3)?.as_blob().map_err(|_|Error::Corrupt)?)?;
        uuid(&r.attempt_id)?;uuid(&r.record.head.record_id)?;revision(&r.record.head.record_version)?;uuid(&r.selected_at.epoch)?;
        for (index,value) in [&r.attempt_id,&r.record.head.record_id,&r.record.head.record_version].into_iter().enumerate(){if row.get_ref(index)?.as_str().map_err(|_|Error::Corrupt)?!=value {return Err(Error::Corrupt);}}
        Ok(r)
    })?;
    validate_roster_bindings(&instances, &pins, &attempts, event_high_water, deadline)?;
    remaining(deadline)?;
    Ok(RecoveryInventory {
        epoch,
        generation,
        mode,
        event_high_water,
        tasks,
        attempts,
        verifications,
        acceptances,
        stops,
        pending_delivery,
        instances,
        pins,
        rows: budget.rows,
        payload_bytes: budget.bytes,
    })
}

fn validate_roster_bindings(
    instances: &[Instance],
    pins: &[Pin],
    attempts: &[DurableAttempt],
    event_high_water: u64,
    deadline: Instant,
) -> Result<()> {
    for instance in instances {
        remaining(deadline)?;
        let attempt = attempts
            .iter()
            .find(|a| a.id == instance.attempt_id)
            .ok_or(Error::Corrupt)?;
        if attempt.task != instance.task_id || attempt.generation != instance.attempt_generation {
            return Err(Error::Corrupt);
        }
    }
    for pin in pins {
        remaining(deadline)?;
        pin.selection.validate().map_err(|_| Error::Corrupt)?;
        pin.record
            .head
            .definition
            .validate()
            .map_err(|_| Error::Corrupt)?;
        // Validate the immutable observation at admission's captured clock, not
        // the new receiver clock on reopen. Expiry now grants no cleanup authority.
        let observed = pin.record.observation.as_ref().ok_or(Error::Corrupt)?;
        observed.input.validate().map_err(|_| Error::Corrupt)?;
        uuid(&observed.id)?;
        uuid(&observed.received.epoch)?;
        let cutoff = pin
            .record
            .head
            .observation_cutoff_unix_ms
            .as_deref()
            .map(crate::contracts::parse_u64_decimal)
            .transpose()
            .map_err(|_| Error::Corrupt)?;
        if observed.sequence == 0
            || observed.sequence > event_high_water
            || cutoff != Some(observed.received.unix_ms)
            || observed.input.instance_id.is_some()
            || !pin
                .selection
                .permits(&pin.record.head, Some(observed), &pin.selected_at)
        {
            return Err(Error::Corrupt);
        }
        if pin.selection.record_id != pin.record.head.record_id
            || pin.selection.expected_revision != pin.record.head.record_version
            || !attempts.iter().any(|a| a.id == pin.attempt_id)
        {
            return Err(Error::Corrupt);
        }
    }
    Ok(())
}
