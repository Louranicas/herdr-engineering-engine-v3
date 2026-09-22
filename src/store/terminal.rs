//! Terminal nonacceptance after existing work and verification obligations settle.

use super::{
    Effect, Error, Object, Principal, Result, Settlement, Store, event, head, next, number,
    same_generation,
};
use crate::contracts::{Generation, UuidV4, receipt::Name};
use rusqlite::params;
use std::time::Instant;

/// A stop request cannot choose success or erase cancellation intent.
#[derive(Clone, Copy)]
pub struct Stop<'a> {
    pub task: UuidV4<'a>,
    pub generation: Generation,
    pub reason: &'a Name,
    pub evidence: &'a Object,
    pub event: UuidV4<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stopped {
    pub generation: String,
    pub cancelled: bool,
}

impl Store {
    /// Close an unaccepted task only after its durable attempts and checks settled.
    /// The exact stop evidence, terminal state and notification commit together.
    /// Reservations are released only here; measured spent cost is preserved.
    /// Cancellation intent takes precedence over any supplied failure reason.
    ///
    /// # Errors
    /// Refuses invisible/stale/already accepted or stopped tasks, corrupt evidence,
    /// outstanding work/usage/cleanup, or an unreturned current verification.
    pub fn finish_unaccepted(
        &mut self,
        principal: &Principal,
        stop: Stop<'_>,
        deadline: Instant,
    ) -> Result<Stopped> {
        self.finish_with_preparation(principal, stop, None, deadline)
    }

    /// Stop failed preparation before any attempt, charging its measured wall cost.
    /// `Settlement` describes preparation only: `effect` must be `None`, cleanup
    /// must be observed settled, usage must be known, and `ready_to_verify` false.
    /// The caller retains its exact error, accounting and cleanup observations in
    /// `stop.evidence`. Unknown facts never release reservations or imply zero cost.
    /// Cancellation takes precedence; cost, stop evidence and outbox commit together.
    ///
    /// # Errors
    /// Refuses any existing attempt, stale/nonadmitted/closed task, unknown or
    /// unsettled preparation, usage above reserved work, and ordinary Store failures.
    pub fn finish_preparation(
        &mut self,
        principal: &Principal,
        stop: Stop<'_>,
        preparation: Settlement,
        deadline: Instant,
    ) -> Result<Stopped> {
        self.finish_with_preparation(principal, stop, Some(preparation), deadline)
    }

    fn finish_with_preparation(
        &mut self,
        principal: &Principal,
        stop: Stop<'_>,
        preparation: Option<Settlement>,
        deadline: Instant,
    ) -> Result<Stopped> {
        self.get(principal, stop.task, deadline)?;
        self.read_object(stop.evidence, deadline)?;
        let mut body = serde_json::json!({
            "reason": stop.reason.as_str(), "evidence": stop.evidence,
        });
        if let Some(preparation) = preparation {
            body["preparation"] = serde_json::json!({
                "used_ms": preparation.used_ms,
                "effect": preparation.effect.name(),
                "cleanup_settled": preparation.cleanup_settled,
                "ready_to_verify": preparation.ready_to_verify,
            });
        }
        let body = serde_json::to_vec(&body)?;
        self.transaction(deadline, |tx| {
            let current = head(tx, stop.task.as_str())?;
            same_generation(&current, stop.generation)?;
            let closed: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM task_stops WHERE task_id=?)",
                [stop.task.as_str()], |row| row.get(0))?;
            if current.accepted_event.is_some() || closed { return Err(Error::Conflict); }
            let preparation_ms = if let Some(preparation) = preparation {
                let attempts: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM attempts WHERE task_id=?)",
                    [stop.task.as_str()], |row| row.get(0),
                )?;
                if attempts
                    || !matches!(current.state.as_str(), "admitted" | "cancellation_requested")
                    || preparation.effect != Effect::None
                    || !preparation.cleanup_settled
                    || preparation.ready_to_verify
                {
                    return Err(Error::Outstanding);
                }
                let used = preparation.used_ms.ok_or(Error::Outstanding)?;
                if used > current.reserved_work_ms { return Err(Error::Budget); }
                used
            } else { 0 };
            let spent = current.spent_ms.checked_add(preparation_ms).ok_or(Error::Budget)?;
            let outstanding: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM attempts a LEFT JOIN verifications v ON v.attempt_id=a.id WHERE a.task_id=? AND (a.state!='settled' OR a.effect NOT IN ('none','committed') OR a.cleanup!='settled' OR a.used_ms IS NULL OR (v.attempt_id IS NOT NULL AND (v.used_ms IS NULL OR v.cleanup_settled!=1))))",
                [stop.task.as_str()], |row| row.get(0),
            )?;
            if outstanding { return Err(Error::Outstanding); }
            // Cancellation may have interrupted a check before it returned. A
            // retained zero-cost/not-started observation is required in that case;
            // this operation never infers verifier idleness from worker settlement.
            if matches!(current.state.as_str(), "verifying" | "cancellation_requested") {
                let missing: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM attempts a WHERE a.task_id=? AND a.generation=(SELECT CAST(MAX(CAST(generation AS INTEGER)) AS TEXT) FROM attempts WHERE task_id=?) AND NOT EXISTS(SELECT 1 FROM verifications v WHERE v.attempt_id=a.id))",
                    params![stop.task.as_str(), stop.task.as_str()], |row| row.get(0),
                )?;
                if missing { return Err(Error::Outstanding); }
            }
            if !matches!(current.state.as_str(), "admitted" | "queued" | "repair_pending" | "failed" | "verifying" | "cancellation_requested") {
                return Err(Error::Outstanding);
            }
            tx.execute("INSERT INTO artifacts(digest,size) VALUES(?,?) ON CONFLICT(digest) DO NOTHING",
                params![stop.evidence.digest, number(stop.evidence.size)?])?;
            let stored_size: u64 = tx.query_row("SELECT size FROM artifacts WHERE digest=?",
                [&stop.evidence.digest], |row| super::read_number(row, 0))?;
            if stored_size != stop.evidence.size { return Err(Error::Corrupt); }
            let generation = next(stop.generation)?;
            let state = if current.cancellation { "cancelled" } else { "failed" };
            event(tx, stop.event.as_str(), stop.task.as_str(), &generation, "task_stopped")?;
            tx.execute("UPDATE events SET body=? WHERE id=?", params![body, stop.event.as_str()])?;
            tx.execute("INSERT INTO task_stops(task_id,event_id,evidence_digest,reason,state) VALUES(?,?,?,?,?)",
                params![stop.task.as_str(),stop.event.as_str(),stop.evidence.digest,stop.reason.as_str(),state])?;
            tx.execute("UPDATE tasks SET generation=?,state=?,spent_ms=?,reserved_work_ms=0,reserved_verify_ms=0 WHERE id=?",
                params![generation,state,number(spent)?,stop.task.as_str()])?;
            tx.execute("INSERT INTO outbox(event_id,recipient) VALUES(?,?)", params![stop.event.as_str(),principal.recipient()])?;
            Ok(Stopped { generation, cancelled: current.cancellation })
        })
    }
}
