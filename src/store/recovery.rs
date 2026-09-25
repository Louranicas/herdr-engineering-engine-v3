//! Bounded readback of durable task obligations, without reconciliation authority.

use super::{
    Error, Principal, Result, Store, TaskHead, read_number, read_optional_number, remaining,
    schema, task_row, visible_head,
};
use crate::contracts::rc01::MAX_ATTEMPTS;
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
        match (result, super::roll_back(tx)) {
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

const TASK_COLUMNS: &str = "SELECT id,generation,state,cancellation,accepted_event,criteria_digest,spent_ms,reserved_work_ms,reserved_verify_ms,principal_uid,principal_role,limit_ms FROM tasks";
const ATTEMPT_COLUMNS: &str =
    "SELECT id,task_id,generation,state,effect,cleanup,used_ms FROM attempts";
const VERIFICATION_COLUMNS: &str = "SELECT attempt_id,event_id,subject_digest,evidence_digest,verdict,used_ms,cleanup_settled FROM verifications";
const INSTANCE_COLUMNS: &str = "SELECT id,task_id,attempt_id,agent_record_id,agent_record_version,revision,body FROM roster_instances";
const PIN_COLUMNS: &str = "SELECT attempt_id,record_id,record_version,body FROM roster_pins";

/// One task row, validated: the one reader of `tasks` for both inventories.
fn durable_task_row(row: &Row<'_>) -> Result<DurableTask> {
    let head = task_row(row)?;
    uuid(&head.id)?;
    revision(&head.generation)?;
    digest(&head.criteria)?;
    flag(row, 3)?;
    one_of(
        &head.state,
        &[
            "admitted",
            "queued",
            "running",
            "verifying",
            "repair_pending",
            "cancellation_requested",
            "blocked",
            "accepted",
            "failed",
            "cancelled",
            "abandoned",
            "effect_unknown",
        ],
    )?;
    if let Some(event) = &head.accepted_event {
        uuid(event)?;
    }
    if (head.state == "accepted") != head.accepted_event.is_some() {
        return Err(Error::Corrupt);
    }
    let principal_uid = row.get(9)?;
    let principal_role: String = row.get(10)?;
    super::Principal::new(principal_uid, &principal_role).map_err(|_| Error::Corrupt)?;
    let limit_ms = read_number(row, 11)?;
    if limit_ms == 0
        || head
            .spent_ms
            .checked_add(head.reserved_work_ms)
            .and_then(|n| n.checked_add(head.reserved_verify_ms))
            .is_none_or(|n| n > limit_ms)
    {
        return Err(Error::Corrupt);
    }
    Ok(DurableTask {
        head,
        principal_uid,
        principal_role,
        limit_ms,
    })
}

/// One verification row, validated.
fn verification_row(row: &Row<'_>) -> Result<DurableVerification> {
    let r = DurableVerification {
        attempt: row.get(0)?,
        event: row.get(1)?,
        subject: row.get(2)?,
        evidence: row.get(3)?,
        verdict: row.get(4)?,
        used_ms: read_optional_number(row, 5)?,
        cleanup_settled: flag(row, 6)?,
    };
    uuid(&r.attempt)?;
    uuid(&r.event)?;
    digest(&r.subject)?;
    digest(&r.evidence)?;
    one_of(
        &r.verdict,
        &[
            "passed",
            "failed",
            "invalid",
            "error",
            "timeout",
            "cancelled",
        ],
    )?;
    Ok(r)
}

/// One roster instance row, validated against its own columns.
fn instance_row(row: &Row<'_>) -> Result<Instance> {
    let r: Instance =
        serde_json::from_slice(row.get_ref(6)?.as_blob().map_err(|_| Error::Corrupt)?)?;
    for id in [
        &r.id,
        &r.task_id,
        &r.attempt_id,
        &r.agent_record_id,
        &r.session_id,
        &r.workspace_ref,
        &r.started.epoch,
    ] {
        uuid(id)?;
    }
    for v in [
        &r.generation,
        &r.revision,
        &r.agent_record_version,
        &r.attempt_generation,
    ] {
        revision(v)?;
    }
    for (index, value) in [
        &r.id,
        &r.task_id,
        &r.attempt_id,
        &r.agent_record_id,
        &r.agent_record_version,
        &r.revision,
    ]
    .into_iter()
    .enumerate()
    {
        if row.get_ref(index)?.as_str().map_err(|_| Error::Corrupt)? != value {
            return Err(Error::Corrupt);
        }
    }
    Ok(r)
}

/// One roster pin row, validated against its own columns.
fn pin_row(row: &Row<'_>) -> Result<Pin> {
    let r: Pin = serde_json::from_slice(row.get_ref(3)?.as_blob().map_err(|_| Error::Corrupt)?)?;
    uuid(&r.attempt_id)?;
    uuid(&r.record.head.record_id)?;
    revision(&r.record.head.record_version)?;
    uuid(&r.selected_at.epoch)?;
    for (index, value) in [
        &r.attempt_id,
        &r.record.head.record_id,
        &r.record.head.record_version,
    ]
    .into_iter()
    .enumerate()
    {
        if row.get_ref(index)?.as_str().map_err(|_| Error::Corrupt)? != value {
            return Err(Error::Corrupt);
        }
    }
    Ok(r)
}

/// One attempt row, validated. The one reader of `attempts`, for the inventory and a task's view.
fn attempt_row(row: &Row<'_>) -> Result<DurableAttempt> {
    let r = DurableAttempt {
        id: row.get(0)?,
        task: row.get(1)?,
        generation: row.get(2)?,
        state: row.get(3)?,
        effect: row.get(4)?,
        cleanup: row.get(5)?,
        used_ms: read_optional_number(row, 6)?,
    };
    uuid(&r.id)?;
    uuid(&r.task)?;
    revision(&r.generation)?;
    one_of(&r.state, &["queued", "running", "settled", "unknown"])?;
    one_of(&r.effect, &["none", "committed", "pending", "unknown"])?;
    one_of(&r.cleanup, &["none", "pending", "settled", "unknown"])?;
    if r.state == "settled"
        && (!matches!(r.effect.as_str(), "none" | "committed")
            || r.cleanup != "settled"
            || r.used_ms.is_none())
    {
        return Err(Error::Corrupt);
    }
    Ok(r)
}

/// What `task.list` selects (B06): task states (empty selects every state), and the class and the
/// parent a task was admitted under (`None` selects any).
#[derive(Clone, Copy, Debug)]
pub struct TaskFilter<'a> {
    pub states: &'a [&'a str],
    pub task_class: Option<&'a str>,
    pub parent_task_id: Option<&'a str>,
}

/// One page of a principal's tasks (B06): the snapshot it was read under, each listed task with
/// its admission sequence (the listing's ordering key), and whether more remain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskListing {
    pub snapshot: u64,
    pub tasks: Vec<(u64, TaskView)>,
    pub more: bool,
}

/// A class or a parent as the task's admitted request names it; `NULL` for bytes that are not a
/// JSON request (a ledger fixture never admitted through the wire).
const ADMITTED_CLASS: &str = "CASE WHEN json_valid(CAST(t.spec AS TEXT)) THEN json_extract(CAST(t.spec AS TEXT),'$.body.spec.task_class') END";
const ADMITTED_PARENT: &str = "CASE WHEN json_valid(CAST(t.spec AS TEXT)) THEN json_extract(CAST(t.spec AS TEXT),'$.body.spec.parent.task_id') END";

fn list_in(
    db: &Connection,
    principal: &Principal,
    filter: &TaskFilter<'_>,
    snapshot: Option<u64>,
    after: u64,
    limit: usize,
    deadline: Instant,
) -> Result<TaskListing> {
    let high_water = db.query_row("SELECT coalesce(max(sequence),0) FROM events", [], |row| {
        read_number(row, 0)
    })?;
    let continuing = snapshot.is_some();
    let snapshot = snapshot.unwrap_or(high_water);
    if snapshot > high_water {
        return Err(Error::SnapshotAhead {
            snapshot,
            high_water,
        });
    }
    let (after_at, snapshot_at) = (
        i64::try_from(after).map_err(|_| Error::Bound)?,
        i64::try_from(snapshot).map_err(|_| Error::Bound)?,
    );
    // A continuation shows each member as it was at the snapshot: refused once a member it has
    // yet to list has an event after it. A member already listed, or a task admitted after the
    // snapshot (not a member), does not move it.
    if continuing {
        let moved: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM events n JOIN tasks t ON t.id=n.task_id \
             JOIN events a ON a.task_id=t.id AND a.kind='admitted' \
             WHERE n.sequence>?4 AND a.sequence>?3 AND a.sequence<=?4 \
             AND t.principal_uid=?1 AND t.principal_role=?2)",
            rusqlite::params![principal.uid(), principal.role(), after_at, snapshot_at],
            |row| row.get(0),
        )?;
        if moved {
            return Err(Error::SnapshotMoved { snapshot });
        }
    }
    let states = serde_json::to_string(filter.states)?;
    let sql = format!(
        "SELECT t.id,e.sequence FROM tasks t JOIN events e ON e.task_id=t.id AND e.kind='admitted' \
         WHERE t.principal_uid=?1 AND t.principal_role=?2 AND e.sequence>?3 AND e.sequence<=?4 \
         AND (?5='[]' OR t.state IN (SELECT value FROM json_each(?5))) \
         AND (?6 IS NULL OR ({ADMITTED_CLASS})=?6) AND (?7 IS NULL OR ({ADMITTED_PARENT})=?7) \
         ORDER BY e.sequence LIMIT ?8"
    );
    let wanted = i64::try_from(limit).map_err(|_| Error::Bound)?;
    let mut statement = db.prepare(&sql)?;
    let rows = statement
        .query_map(
            rusqlite::params![
                principal.uid(),
                principal.role(),
                after_at,
                snapshot_at,
                states,
                filter.task_class,
                filter.parent_task_id,
                wanted.checked_add(1).ok_or(Error::Bound)?,
            ],
            |row| Ok((row.get::<_, String>(0)?, read_number(row, 1)?)),
        )?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let more = rows.len() > limit;
    let mut tasks = Vec::with_capacity(limit.min(rows.len()));
    for (id, sequence) in rows.into_iter().take(limit) {
        remaining(deadline)?;
        let task = UuidV4::parse(&id).map_err(|_| Error::Corrupt)?;
        tasks.push((sequence, read_view(db, principal, task, deadline)?));
    }
    Ok(TaskListing {
        snapshot,
        tasks,
        more,
    })
}

/// One task as its principal may read it (B03): its head, its attempts and how many of its
/// delivery obligations are still owed, from one read snapshot, with the ledger's read point.
/// Scoped by `task_id`, so a ledger's size never decides whether one task can be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskView {
    pub head: TaskHead,
    pub attempts: Vec<DurableAttempt>,
    pub pending_deliveries: usize,
    /// The ledger's event high-water when the view was read: the same read point the recovery
    /// inventory reports, not the sequence at which this task last changed.
    pub event_high_water: u64,
}

impl TaskView {
    /// The attempt a worker holds now: the one queued or running, if any.
    #[must_use]
    pub fn current_attempt(&self) -> Option<&str> {
        self.attempts
            .iter()
            .find(|attempt| matches!(attempt.state.as_str(), "queued" | "running"))
            .map(|attempt| attempt.id.as_str())
    }

    /// Obligations still owed: every attempt whose effect or cleanup is pending or unknown, plus
    /// every undelivered outbox row.
    #[must_use]
    pub fn unresolved_obligations(&self) -> usize {
        self.attempts
            .iter()
            .filter(|attempt| {
                matches!(attempt.effect.as_str(), "pending" | "unknown")
                    || matches!(attempt.cleanup.as_str(), "pending" | "unknown")
            })
            .count()
            + self.pending_deliveries
    }

    /// What a cancel finds of the task's worker (B05): `not_started` with no attempt, `pending`
    /// while one is queued or running, `unknown` when one's settlement is unknown, and `settled`
    /// only when every attempt settled.
    #[must_use]
    pub fn worker_settlement(&self) -> &'static str {
        let states = || self.attempts.iter().map(|attempt| attempt.state.as_str());
        if self.attempts.is_empty() {
            "not_started"
        } else if states().any(|state| matches!(state, "queued" | "running")) {
            "pending"
        } else if states().any(|state| state == "unknown") {
            "unknown"
        } else {
            "settled"
        }
    }
}

impl Store {
    /// Read one task, scoped to `task` and visible to `principal` only.
    ///
    /// Refuses a poisoned store, as the recovery inventory does (the B03 unification; the
    /// admission-recovery readback `get_by_key` keeps its pinned exception). A task holding more
    /// attempts than `MAX_ATTEMPTS` is refused as `TaskViewBound` with both numbers.
    /// # Errors
    /// `UncertainCommit` when poisoned; `NotFound` for a task this principal cannot see;
    /// `TaskViewBound`; read and rollback failures, a rollback failure poisoning the store.
    pub fn task_view(
        &mut self,
        principal: &Principal,
        task: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<TaskView> {
        self.read_snapshot(deadline, |db| read_view(db, principal, task, deadline))
    }

    /// B06 `task.list`: one page of `principal`'s tasks in admission order, each read as
    /// [`Store::task_view`] reads it, from one read snapshot. Membership is fixed by `snapshot` (the
    /// event high-water when the listing began; `None` begins one): a task admitted after it is not
    /// a member. A continuation is refused (`SnapshotMoved`) once a member it has yet to list has an
    /// event after the snapshot, so every page shows its members as they were at it -- as far as the
    /// ledger's events record: a delivery acknowledgement (`acknowledge_delivery`, not yet reached from
    /// the wire) writes none, so it can lower a later member's unresolved obligations unseen. `states`, the admitted class and the parent select; the page starts after the
    /// admission sequence `after` and holds at most `limit` tasks, `more` saying whether any remain.
    /// A class and a parent are read from the task's admitted request, where they are persisted.
    /// # Errors
    /// `UncertainCommit` when poisoned; `SnapshotAhead` for a snapshot beyond the high-water;
    /// `SnapshotMoved`;
    /// `TaskViewBound` for a listed task past the attempt bound; `Bound`; read failures.
    pub fn task_list(
        &mut self,
        principal: &Principal,
        filter: &TaskFilter<'_>,
        snapshot: Option<u64>,
        after: u64,
        limit: usize,
        deadline: Instant,
    ) -> Result<TaskListing> {
        self.read_snapshot(deadline, |db| {
            list_in(db, principal, filter, snapshot, after, limit, deadline)
        })
    }

    /// Run `read` in one deferred read transaction, rolled back after: the one door for a
    /// principal's read snapshot. Refuses a poisoned store; a rollback failure poisons it.
    fn read_snapshot<T>(
        &mut self,
        deadline: Instant,
        read: impl FnOnce(&Connection) -> Result<T>,
    ) -> Result<T> {
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        schema::bound(&self.connection, deadline)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        let result = read(&tx);
        match (result, super::roll_back(tx)) {
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

pub(super) fn read_view(
    db: &Connection,
    principal: &Principal,
    task: UuidV4<'_>,
    deadline: Instant,
) -> Result<TaskView> {
    let head = visible_head(db, principal, task)?;
    let attempts_found = db.query_row(
        "SELECT count(*) FROM attempts WHERE task_id=?",
        [task.as_str()],
        |row| read_number(row, 0),
    )?;
    let limit = u64::from(MAX_ATTEMPTS);
    if attempts_found > limit {
        return Err(Error::TaskViewBound {
            attempts: attempts_found,
            limit,
        });
    }
    remaining(deadline)?;
    let mut attempts = Vec::new();
    let mut statement = db.prepare(
        "SELECT id,task_id,generation,state,effect,cleanup,used_ms FROM attempts WHERE task_id=? ORDER BY id",
    )?;
    let mut rows = statement.query([task.as_str()])?;
    while let Some(row) = rows.next()? {
        attempts.push(attempt_row(row)?);
    }
    let pending = db.query_row(
        "SELECT count(*) FROM outbox o JOIN events e ON e.id=o.event_id WHERE e.task_id=? AND o.delivered=0",
        [task.as_str()],
        |row| read_number(row, 0),
    )?;
    let event_high_water =
        db.query_row("SELECT coalesce(max(sequence),0) FROM events", [], |row| {
            read_number(row, 0)
        })?;
    Ok(TaskView {
        head,
        attempts,
        pending_deliveries: usize::try_from(pending).map_err(|_| Error::Bound)?,
        event_high_water,
    })
}

/// B03b · the ONE definition of a terminal attempt's confirmed-clean closure, as an SQL condition
/// over an `attempts` row aliased `a`. Closed means a startup record for the attempt carries the
/// ENGINE'S OWN physical readback showing it clean or its workspace deliberately retained — never a
/// worker's `cleanup_settled` claim (T07-AP-42 falsified that). Two record kinds can show it:
///
/// * `reconciliation_decided`: `handed` is what the shell observed and handed the policy; closed
///   when its cleanup readback is `complete` AND its process custody is positively not a live
///   holder. Custody is an allow-list — `absent` (nothing holds it), `pid_reused` (the PID is now
///   another process: not ours, R07), `unobserved` (a settled attempt's observation was never a
///   local process, RC-24). `live_same_identity`, `unreadable` and any other or future spelling
///   keep the attempt open: unknown is never closed. ALSO closed, with the disposition "workspace
///   retained", when the decision is a standing one (`acceptance_stands` / `cancellation_stands`),
///   custody is in the same allow-list and the workspace readback shows it still `writable`:
///   startup never deletes a standing task's workspace (it may be the retained evidence), so
///   without this clause such attempts refill every batch and starve the cleanable ones behind
///   them. Their retention and collection belong to T18 (route: "retained-workspace
///   retention/GC"), not to startup.
/// * `reconciliation_readback` of the `cleanup` effect whose readback is `complete`.
///
/// The bodies are the engine's own serialized records (`app::startup`), an owned contract; a
/// rename there fails `t07_startup`'s real-writer closure case rather than silently reopening all
/// history.
const CLOSED_BY_ENGINE_READBACK: &str = "EXISTS (SELECT 1 FROM events e WHERE e.task_id=a.task_id \
    AND json_extract(CAST(e.body AS TEXT),'$.attempt')=a.id AND ( \
    (e.kind='reconciliation_decided' \
     AND json_extract(CAST(e.body AS TEXT),'$.handed.cleanup.cleanup_readback')='complete' \
     AND json_extract(CAST(e.body AS TEXT),'$.handed.process.custody') IN ('absent','pid_reused','unobserved')) \
    OR (e.kind='reconciliation_decided' \
     AND json_extract(CAST(e.body AS TEXT),'$.decision.decision') IN ('acceptance_stands','cancellation_stands') \
     AND json_extract(CAST(e.body AS TEXT),'$.handed.workspace.workspace')='writable' \
     AND json_extract(CAST(e.body AS TEXT),'$.handed.process.custody') IN ('absent','pid_reused','unobserved')) \
    OR (e.kind='reconciliation_readback' \
     AND json_extract(CAST(e.body AS TEXT),'$.effect')='cleanup' \
     AND json_extract(CAST(e.body AS TEXT),'$.readback.cleanup_readback')='complete')))";

/// The task states after which nothing more is dispatched for a task.
const TERMINAL: &str = "('accepted','failed','cancelled','abandoned')";

/// Startup's two bounds (B03b). `open` bounds the effect-bearing open attempts, which readiness
/// requires reconciled, and refuses above it by name; `cleanup_batch` is how many terminal
/// attempts not yet confirmed clean one boot takes, oldest first. Neither counts closed history.
#[derive(Clone, Copy, Debug)]
pub struct StartupLimits {
    pub open: usize,
    pub cleanup_batch: usize,
    /// The row and byte budget for everything read about the selected attempts.
    pub read: RecoveryLimits,
}

/// What startup reads (B03b): the selected attempts — every effect-bearing open attempt, plus
/// this boot's batch of terminal attempts not yet confirmed clean — with ONLY their tasks,
/// verifications, instances and pins. The `inventory` is therefore NOT the whole ledger's; its
/// acceptances, stops and pending deliveries are empty because startup reads none of them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupInventory {
    pub inventory: RecoveryInventory,
    /// Which of the inventory's attempts are this boot's terminal-cleanup batch rather than
    /// effect-bearing open work: decided and acted on, but outside readiness.
    pub cleanup_attempts: Vec<String>,
    /// Terminal attempts still unconfirmed after this boot's batch: reported, never silent.
    pub cleanup_backlog: usize,
}

impl Store {
    /// Whether `attempt` is closed history by `CLOSED_BY_ENGINE_READBACK`: the one door.
    /// # Errors
    /// A poisoned store, an expired deadline or a read failure.
    pub fn attempt_closed(&self, attempt: UuidV4<'_>, deadline: Instant) -> Result<bool> {
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        schema::bound(&self.connection, deadline)?;
        Ok(self.connection.query_row(
            &format!("SELECT {CLOSED_BY_ENGINE_READBACK} FROM attempts a WHERE a.id=?"),
            [attempt.as_str()],
            |row| row.get::<_, bool>(0),
        )?)
    }

    /// Startup's read (B03b): open obligations only, in one read snapshot.
    /// # Errors
    /// `StartupBound { open, limit }` above the open bound; otherwise as
    /// [`Store::recovery_inventory`].
    pub fn startup_inventory(
        &mut self,
        expected_epoch: UuidV4<'_>,
        limits: StartupLimits,
        deadline: Instant,
    ) -> Result<StartupInventory> {
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        if expected_epoch.as_str() != self.epoch {
            return Err(Error::Conflict);
        }
        schema::bound(&self.connection, deadline)?;
        let generation = self
            .generation
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(Error::Custody)?
            .to_owned();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        let result = collect_open(&tx, expected_epoch.as_str(), &generation, limits, deadline);
        match (result, super::roll_back(tx)) {
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

/// Effect-bearing open: an unsettled attempt, or any attempt of a task not yet terminal.
fn open_attempts_predicate() -> String {
    format!("(a.state!='settled' OR t.state NOT IN {TERMINAL})")
}

/// The terminal tail: a settled attempt of a terminal task not yet confirmed clean.
fn tail_predicate() -> String {
    format!("(a.state='settled' AND t.state IN {TERMINAL} AND NOT {CLOSED_BY_ENGINE_READBACK})")
}

fn collect_open(
    db: &Connection,
    epoch: &str,
    generation: &str,
    limits: StartupLimits,
    deadline: Instant,
) -> Result<StartupInventory> {
    let open = db.query_row(
        &format!(
            "SELECT count(*) FROM attempts a JOIN tasks t ON t.id=a.task_id WHERE {}",
            open_attempts_predicate()
        ),
        [],
        |row| read_number(row, 0),
    )?;
    let limit = u64::try_from(limits.open).map_err(|_| Error::Bound)?;
    if open > limit {
        return Err(Error::StartupBound { open, limit });
    }
    let tail = db.query_row(
        &format!(
            "SELECT count(*) FROM attempts a JOIN tasks t ON t.id=a.task_id WHERE {}",
            tail_predicate()
        ),
        [],
        |row| read_number(row, 0),
    )?;
    let batch = u64::try_from(limits.cleanup_batch).map_err(|_| Error::Bound)?;
    remaining(deadline)?;
    // One definition of the selected set, prefixed to every read below.
    let selected = format!(
        "WITH selected(id) AS (SELECT a.id FROM attempts a JOIN tasks t ON t.id=a.task_id WHERE {open} \
         UNION ALL SELECT id FROM (SELECT a.id AS id FROM attempts a JOIN tasks t ON t.id=a.task_id \
         WHERE {tail} ORDER BY a.rowid LIMIT {batch})) ",
        open = open_attempts_predicate(),
        tail = tail_predicate(),
    );
    let mut budget = Budget {
        limits: limits.read,
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
    let attempts = budget.read(db, &format!("{selected}{ATTEMPT_COLUMNS} WHERE id IN (SELECT id FROM selected) ORDER BY task_id,id LIMIT ?"), attempt_row)?;
    let tasks = budget.read(db, &format!("{selected}{TASK_COLUMNS} WHERE id IN (SELECT a.task_id FROM attempts a WHERE a.id IN (SELECT id FROM selected)) ORDER BY id LIMIT ?"), durable_task_row)?;
    let verifications = budget.read(db, &format!("{selected}{VERIFICATION_COLUMNS} WHERE attempt_id IN (SELECT id FROM selected) ORDER BY attempt_id LIMIT ?"), verification_row)?;
    let instances = budget.read(db, &format!("{selected}{INSTANCE_COLUMNS} WHERE attempt_id IN (SELECT id FROM selected) ORDER BY id LIMIT ?"), instance_row)?;
    let pins = budget.read(db, &format!("{selected}{PIN_COLUMNS} WHERE attempt_id IN (SELECT id FROM selected) ORDER BY attempt_id,record_id LIMIT ?"), pin_row)?;
    validate_roster_bindings(&instances, &pins, &attempts, event_high_water, deadline)?;
    let mut statement = db.prepare(&format!(
        "SELECT a.id FROM attempts a JOIN tasks t ON t.id=a.task_id WHERE {} ORDER BY a.rowid LIMIT {batch}",
        tail_predicate()
    ))?;
    let mut rows = statement.query([])?;
    let mut cleanup_attempts = Vec::new();
    while let Some(row) = rows.next()? {
        cleanup_attempts.push(row.get::<_, String>(0)?);
    }
    remaining(deadline)?;
    Ok(StartupInventory {
        cleanup_attempts,
        cleanup_backlog: usize::try_from(tail.saturating_sub(batch)).map_err(|_| Error::Bound)?,
        inventory: RecoveryInventory {
            epoch,
            generation,
            mode,
            event_high_water,
            tasks,
            attempts,
            verifications,
            acceptances: Vec::new(),
            stops: Vec::new(),
            pending_delivery: Vec::new(),
            instances,
            pins,
            rows: budget.rows,
            payload_bytes: budget.bytes,
        },
    })
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
    let tasks = budget.read(
        db,
        &format!("{TASK_COLUMNS} ORDER BY id LIMIT ?"),
        durable_task_row,
    )?;
    let attempts = budget.read(
        db,
        &format!("{ATTEMPT_COLUMNS} ORDER BY task_id,id LIMIT ?"),
        attempt_row,
    )?;
    let verifications = budget.read(
        db,
        &format!("{VERIFICATION_COLUMNS} ORDER BY attempt_id LIMIT ?"),
        verification_row,
    )?;
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
    let instances = budget.read(
        db,
        &format!("{INSTANCE_COLUMNS} ORDER BY id LIMIT ?"),
        instance_row,
    )?;
    let pins = budget.read(
        db,
        &format!("{PIN_COLUMNS} ORDER BY attempt_id,record_id LIMIT ?"),
        pin_row,
    )?;
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
