//! Atomic roster selection and task cancellation causes; no worker launch.

use super::{
    BTreeSet, Connection, CutPoint, Error, Generation, Instant, Operation, Principal, ReceiptTime,
    Result, Store, Transaction, UuidV4, capacity, digest, dto, invalid, next, operator, prior,
    random_id, record, retain_operation, retain_revision, roster_event, schema,
};
use crate::contracts::control::CancelReason;
use crate::contracts::roster::{
    ActiveAttemptPolicy, CancellationCause, Disable, Instance, InstanceState, Kind, MAX_HISTORY,
    MAX_INPUT, MAX_PINS, MAX_RECORDS, Outcome, Pin, Selection,
};
use crate::store::{
    AttemptHead, begin_attempt_in, cancellation_body, event, head, outcome_decided,
    request_cancellation,
};
use rusqlite::{OptionalExtension, params};

#[derive(Clone, Copy)]
pub struct RosterStart<'a> {
    pub principal: &'a Principal,
    pub task: UuidV4<'a>,
    pub expected: Generation,
    pub attempt: UuidV4<'a>,
    pub event: UuidV4<'a>,
    pub agent_record_id: &'a str,
    pub session: UuidV4<'a>,
    pub workspace: UuidV4<'a>,
    pub selections: &'a [Selection],
    pub lease_ms: u64,
}

#[derive(Clone, Debug)]
pub struct RosterAttempt {
    pub attempt: AttemptHead,
    pub instance: Instance,
    pub pins: Vec<Pin>,
}

pub(super) fn instance(
    connection: &Connection,
    principal: &Principal,
    id: &str,
) -> Result<Instance> {
    let bytes:Vec<u8>=connection.query_row("SELECT i.body FROM roster_instances i JOIN tasks t ON t.id=i.task_id WHERE i.id=? AND t.principal_uid=? AND t.principal_role=?",params![id,principal.uid(),principal.role()],|row|row.get(0)).optional()?.ok_or(Error::NotFound)?;
    let instance: Instance = serde_json::from_slice(&bytes)?;
    for id in [
        &instance.id,
        &instance.task_id,
        &instance.attempt_id,
        &instance.agent_record_id,
        &instance.session_id,
        &instance.workspace_ref,
        &instance.started.epoch,
    ] {
        dto::uuid(id).map_err(|_| Error::Corrupt)?;
    }
    for revision in [
        &instance.generation,
        &instance.revision,
        &instance.agent_record_version,
        &instance.attempt_generation,
    ] {
        dto::generation(revision).map_err(|_| Error::Corrupt)?;
    }
    if instance.id != id {
        return Err(Error::Corrupt);
    }
    Ok(instance)
}

fn validate_start(input: &RosterStart<'_>) -> Result<()> {
    operator(input.principal)?;
    dto::uuid(input.agent_record_id).map_err(invalid)?;
    if input.selections.is_empty()
        || input.selections.len() > MAX_PINS
        || !(1..=1_200_000).contains(&input.lease_ms)
    {
        return Err(Error::Bound);
    }
    let mut ids = BTreeSet::new();
    for selection in input.selections {
        selection.validate().map_err(invalid)?;
        if !ids.insert(&selection.record_id) {
            return Err(Error::Invalid);
        }
    }
    if !ids.contains(&input.agent_record_id.to_owned()) {
        return Err(Error::Invalid);
    }
    Ok(())
}

fn selected_pins(
    tx: &Transaction<'_>,
    input: &RosterStart<'_>,
    now: &ReceiptTime,
) -> Result<Vec<Pin>> {
    input
        .selections
        .iter()
        .map(|selection| {
            let record = record(tx, input.principal, &selection.record_id)?;
            if record
                .observation
                .as_ref()
                .is_some_and(|observation| observation.input.instance_id.is_some())
                || !selection.permits(&record.head, record.observation.as_ref(), now)
            {
                return Err(Error::Conflict);
            }
            if selection.record_id == input.agent_record_id
                && record.head.definition.kind != Kind::Agent
            {
                return Err(Error::Invalid);
            }
            Ok(Pin {
                attempt_id: input.attempt.as_str().to_owned(),
                selection: selection.clone(),
                record,
                selected_at: now.clone(),
            })
        })
        .collect()
}

fn retain_instance(tx: &Transaction<'_>, instance: &Instance, event_id: &str) -> Result<()> {
    let bytes = serde_json::to_vec(instance)?;
    tx.execute(
        "INSERT INTO roster_instances VALUES(?,?,?,?,?,?,?)",
        params![
            instance.id,
            instance.task_id,
            instance.attempt_id,
            instance.agent_record_id,
            instance.agent_record_version,
            instance.revision,
            bytes
        ],
    )?;
    tx.execute(
        "INSERT INTO roster_instance_history VALUES(?,?,?,?)",
        params![instance.id, instance.revision, bytes, event_id],
    )?;
    Ok(())
}

fn can_transition(old: InstanceState, new: InstanceState) -> bool {
    match old {
        InstanceState::Exited => new == InstanceState::Exited,
        InstanceState::Stopping => matches!(
            new,
            InstanceState::Stopping | InstanceState::Exited | InstanceState::Unknown
        ),
        _ => new != InstanceState::Starting || old == InstanceState::Starting,
    }
}

fn active_attempts(tx: &Transaction<'_>, record_id: &str) -> Result<Vec<(String, String)>> {
    let mut statement=tx.prepare("SELECT a.id,a.task_id FROM roster_pins p JOIN attempts a ON a.id=p.attempt_id JOIN tasks t ON t.id=a.task_id WHERE p.record_id=? AND (a.state!='settled' OR (t.state='verifying' AND a.id=(SELECT latest.id FROM attempts latest WHERE latest.task_id=a.task_id ORDER BY length(latest.generation) DESC,latest.generation DESC LIMIT 1))) ORDER BY a.id LIMIT 257")?;
    let rows = statement
        .query_map([record_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if rows.len() > MAX_RECORDS {
        return Err(Error::Bound);
    }
    Ok(rows)
}

fn cancel_causes(
    tx: &Transaction<'_>,
    active: &[(String, String)],
    record_id: &str,
    disable_event: &str,
    deadline: Instant,
) -> Result<Vec<CancellationCause>> {
    capacity(tx, "roster_cancel_causes", active.len(), MAX_HISTORY)?;
    let mut causes = Vec::new();
    for (attempt, task) in active {
        let cause = CancellationCause {
            disable_event: disable_event.to_owned(),
            record_id: record_id.to_owned(),
            task_id: task.clone(),
            attempt_id: attempt.clone(),
        };
        tx.execute(
            "INSERT INTO roster_cancel_causes VALUES(?,?,?,?)",
            params![disable_event, record_id, task, attempt],
        )?;
        let current = head(tx, task)?;
        // The task's one cancellation transition and its one decided-outcome rule (B05): a disable
        // is an operator's request, and an unknown effect stays unknown under it.
        if !current.cancellation && !outcome_decided(tx, &current)? {
            let event_id = random_id(deadline)?;
            request_cancellation(
                tx,
                &current,
                UuidV4::parse(task).map_err(|_| Error::Corrupt)?,
                current.generation.parse().map_err(|_| Error::Corrupt)?,
                UuidV4::parse(&event_id).map_err(|_| Error::Corrupt)?,
                &cancellation_body(CancelReason::OperatorRequest, None)?,
            )?;
        }
        causes.push(cause);
    }
    Ok(causes)
}

impl Store {
    /// Begin the existing owned attempt and pin all roster facts in one transaction.
    /// This records a starting instance; it does not start a process or prove custody.
    /// # Errors
    /// Disabled/stale/unproven selections or task/budget predicates abort everything.
    pub fn begin_rostered_attempt(
        &mut self,
        input: RosterStart<'_>,
        deadline: Instant,
    ) -> Result<RosterAttempt> {
        validate_start(&input)?;
        let id = fresh_id!(self.clock, deadline)?;
        let clock = self.clock.clone();
        let fault = self.fault();
        self.transaction(deadline,|tx| {
            let visible:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM tasks WHERE id=? AND principal_uid=? AND principal_role=?)",params![input.task.as_str(),input.principal.uid(),input.principal.role()],|row|row.get(0))?;
            if !visible { return Err(Error::NotFound); }
            capacity(tx,"roster_instances",1,MAX_HISTORY)?;
            capacity(tx,"roster_instance_history",1,MAX_HISTORY)?;
            capacity(tx,"roster_pins",input.selections.len(),MAX_HISTORY)?;
            let now=clock.sample()?;
            let lease=now.monotonic_ms.checked_add(input.lease_ms).ok_or(Error::Bound)?;
            let pins=selected_pins(tx,&input,&now)?;
            let agent=pins.iter().find(|pin|pin.record.head.record_id==input.agent_record_id).ok_or(Error::Invalid)?;
            let attempt=begin_attempt_in(tx,input.task,input.expected,input.attempt,input.event,fault)?;
            let instance=Instance { id,generation:"1".to_owned(),revision:"1".to_owned(),agent_record_id:input.agent_record_id.to_owned(),agent_record_version:agent.record.head.record_version.clone(),task_id:input.task.as_str().to_owned(),attempt_id:attempt.id.clone(),attempt_generation:attempt.generation.clone(),session_id:input.session.as_str().to_owned(),workspace_ref:input.workspace.as_str().to_owned(),started:now,lease_expires_monotonic_ms:lease,state:InstanceState::Starting,usage_ms:None };
            for pin in &pins {
                tx.execute("INSERT INTO roster_pins VALUES(?,?,?,?)",params![attempt.id,pin.record.head.record_id,pin.record.head.record_version,serde_json::to_vec(pin)?])?;
            }
            retain_instance(tx,&instance,input.event.as_str())?;
            cut_point!(fault,CutPoint::RosterPin);
            Ok(RosterAttempt { attempt,instance,pins })
        })
    }

    /// # Errors
    /// Missing and invisible instances are indistinguishable.
    pub fn roster_instance(
        &self,
        principal: &Principal,
        id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Instance> {
        schema::bound(&self.connection, deadline)?;
        instance(&self.connection, principal, id.as_str())
    }

    /// List retained instances for a visible profile, including exited history.
    /// # Errors
    /// Refuses invisible profiles or a result exceeding the fixed 256-row bound.
    pub fn roster_instances(
        &self,
        principal: &Principal,
        record_id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Vec<Instance>> {
        schema::bound(&self.connection, deadline)?;
        record(&self.connection, principal, record_id.as_str())?;
        let mut statement = self.connection.prepare(
            "SELECT id FROM roster_instances WHERE agent_record_id=? ORDER BY id LIMIT 257",
        )?;
        let ids = statement
            .query_map([record_id.as_str()], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if ids.len() > MAX_RECORDS {
            return Err(Error::Bound);
        }
        ids.iter()
            .map(|id| instance(&self.connection, principal, id))
            .collect()
    }

    /// Record an instance view. Task usage/effects and cleanup remain separate owners.
    /// # Errors
    /// Refuses stale revision, terminal-state regression and decreasing/forgotten usage.
    pub fn roster_change_instance(
        &mut self,
        principal: &Principal,
        id: UuidV4<'_>,
        expected: Generation,
        state: InstanceState,
        usage_ms: Option<u64>,
        deadline: Instant,
    ) -> Result<Instance> {
        operator(principal)?;
        let event_id = fresh_id!(self.clock, deadline)?;
        self.transaction(deadline, |tx| {
            let mut current = instance(tx, principal, id.as_str())?;
            if current.revision != expected.to_string() {
                return Err(Error::Conflict);
            }
            if !can_transition(current.state, state)
                || current
                    .usage_ms
                    .is_some_and(|old| usage_ms.is_none_or(|new| new < old))
            {
                return Err(Error::Conflict);
            }
            capacity(tx, "roster_instance_history", 1, MAX_HISTORY)?;
            current.revision = next(expected)?;
            current.state = state;
            current.usage_ms = usage_ms;
            let body = serde_json::to_vec(&current)?;
            event(
                tx,
                &event_id,
                &current.task_id,
                &head(tx, &current.task_id)?.generation,
                "roster.instance_observed",
            )?;
            tx.execute(
                "UPDATE roster_instances SET revision=?,body=? WHERE id=?",
                params![current.revision, body, id.as_str()],
            )?;
            tx.execute(
                "INSERT INTO roster_instance_history VALUES(?,?,?,?)",
                params![current.id, current.revision, body, event_id],
            )?;
            Ok(current)
        })
    }

    /// Read immutable admitted facts for an existing owned attempt.
    /// # Errors
    /// Refuses invisible attempts and overlong/invalid pinned data.
    pub fn roster_pins(
        &self,
        principal: &Principal,
        attempt: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Vec<Pin>> {
        schema::bound(&self.connection, deadline)?;
        let visible:bool=self.connection.query_row("SELECT EXISTS(SELECT 1 FROM attempts a JOIN tasks t ON t.id=a.task_id WHERE a.id=? AND t.principal_uid=? AND t.principal_role=?)",params![attempt.as_str(),principal.uid(),principal.role()],|row|row.get(0))?;
        if !visible {
            return Err(Error::NotFound);
        }
        let mut statement = self.connection.prepare(
            "SELECT body FROM roster_pins WHERE attempt_id=? ORDER BY record_id LIMIT 17",
        )?;
        let bytes = statement
            .query_map([attempt.as_str()], |row| row.get::<_, Vec<u8>>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if bytes.len() > MAX_PINS {
            return Err(Error::Bound);
        }
        bytes
            .iter()
            .map(|value| Ok(serde_json::from_slice(value)?))
            .collect()
    }

    /// Commit disable, immutable revision and attributable cancellation causes.
    /// # Errors
    /// Stale/invisible/conflicting requests refuse. Unknown effects retain liability.
    pub fn roster_disable(
        &mut self,
        principal: &Principal,
        input: &Disable,
        request_bytes: &[u8],
        deadline: Instant,
    ) -> Result<Outcome> {
        operator(principal)?;
        input.validate().map_err(invalid)?;
        if request_bytes.is_empty() || request_bytes.len() > MAX_INPUT {
            return Err(Error::Bound);
        }
        std::str::from_utf8(request_bytes).map_err(|_| Error::Invalid)?;
        let temp = fresh_id!(self.clock, deadline)?;
        let object = self.publish(
            request_bytes,
            UuidV4::parse(&temp).map_err(|_| Error::Runtime)?,
            deadline,
        )?;
        let event_id = fresh_id!(self.clock, deadline)?;
        let epoch = self.epoch.clone();
        let request_digest = digest(request_bytes);
        let fault = self.fault();
        self.transaction(deadline, |tx| {
            if let Some(prior) = prior(
                tx,
                principal,
                "roster.disable",
                &input.idempotency_key,
                &request_digest,
            )? {
                return Ok(prior);
            }
            let mut current = record(tx, principal, &input.record_id)?.head;
            if current.record_version != input.expected_revision {
                return Err(Error::Conflict);
            }
            capacity(tx, "roster_revisions", 1, MAX_HISTORY)?;
            let active = active_attempts(tx, &input.record_id)?;
            current.record_version =
                next(current.record_version.parse().map_err(|_| Error::Corrupt)?)?;
            current.disabled = true;
            current.observation_cutoff_unix_ms = None;
            tx.execute(
                "UPDATE roster_records SET disabled=1,revision=?,observation_id=NULL WHERE id=?",
                params![current.record_version, current.record_id],
            )?;
            let sequence = roster_event(
                tx,
                &event_id,
                &input.record_id,
                &current.record_version,
                "roster.disable",
                input.audit_reason.as_bytes(),
            )?;
            retain_revision(tx, &current, &event_id)?;
            let causes = if input.active_attempt_policy == ActiveAttemptPolicy::RequestCancel {
                cancel_causes(tx, &active, &input.record_id, &event_id, deadline)?
            } else {
                Vec::new()
            };
            cut_point!(fault, CutPoint::RosterWrite);
            let outcome = Outcome {
                head: current,
                event_id,
                epoch,
                sequence,
                active_attempts: active.into_iter().map(|(attempt, _)| attempt).collect(),
                cancellation_causes: causes,
            };
            retain_operation(
                tx,
                &Operation {
                    principal,
                    action: "roster.disable",
                    key: &input.idempotency_key,
                    request_digest: &request_digest,
                    source: &object,
                    row: None,
                },
                &outcome,
            )?;
            Ok(outcome)
        })
    }
}
