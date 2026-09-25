//! Durable verification outcomes and conservative reservation settlement.

use super::{
    Error, Expected, Object, PublishedAcceptance, Result, Store, event, head, next, number,
    require_attempt, same_generation,
};
use crate::contracts::{Sha256Digest, UuidV4};
use rusqlite::params;
use std::time::Instant;

/// Verifier disposition supplied by the trusted check owner, never worker output.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationVerdict {
    Passed,
    Failed,
    Invalid,
    Error,
    Timeout,
    Cancelled,
}

/// Exact subject and immutable evidence for one completed verifier invocation.
/// Unknown cost or cleanup retains the reservation and blocks new work.
#[derive(Clone, Debug)]
pub struct Verification<'a> {
    pub verdict: VerificationVerdict,
    pub subject: Sha256Digest<'a>,
    pub evidence: Object,
    pub used_ms: Option<u64>,
    pub cleanup_settled: bool,
}

#[derive(serde::Serialize)]
struct Observation<'a> {
    attempt: &'a str,
    attempt_generation: String,
    subject: &'a str,
    evidence: &'a Object,
    verdict: VerificationVerdict,
    used_ms: Option<u64>,
    cleanup_settled: bool,
}

impl Store {
    /// Prepare acceptance only for the exact subject and evidence already recorded
    /// as a reconciled pass by the trusted collector. Verification cost was charged
    /// by `record_verification`; call `accept` with zero additional verification cost.
    /// The complete evidence inventory permits at most 4096 distinct CAS objects
    /// totalling 64 MiB, with the existing 16-MiB limit on each object. Typed graph
    /// resolution and completeness remain the collector's preceding responsibility.
    /// Its prospective union with already registered objects and the new acceptance
    /// manifest must fit the snapshot's 4096-object cap. This is also rechecked in
    /// the acceptance transaction; unrelated registrations may exhaust capacity.
    ///
    /// # Errors
    /// Refuses absent/nonpass/unsettled verification, changed subject or evidence,
    /// cancellation, stale generations, and an evidence list omitting the receipt.
    pub fn prepare_verified_acceptance(
        &self,
        expected: &Expected<'_>,
        event_id: UuidV4<'_>,
        subject: Sha256Digest<'_>,
        evidence: &Object,
        objects: &[Object],
        deadline: Instant,
    ) -> Result<PublishedAcceptance> {
        super::schema::bound(&self.connection, deadline)?;
        let current = head(&self.connection, expected.task.as_str())?;
        // Cancellation before the compare-and-set: a cancel bumps the generation, so the older
        // order named the cause `Conflict` (B14a-R1.2).
        if current.cancellation {
            return Err(Error::Cancelled);
        }
        same_generation(&current, expected.task_generation)?;
        require_attempt(&self.connection, expected, true)?;
        if current.state != "verifying" {
            return Err(Error::Outstanding);
        }
        let verified = VerifiedSubject {
            subject: subject.as_str().to_owned(),
            evidence: evidence.clone(),
        };
        require_verified(&self.connection, expected.attempt.as_str(), &verified)?;
        if !objects.iter().any(|object| object == evidence) {
            return Err(Error::Invalid);
        }
        self.prepare_acceptance_inner(expected, event_id, objects, Some(verified), deadline)
    }

    /// Record an actual verifier return, including failures, before choosing repair.
    /// A pass leaves the task verifying; only separate acceptance can close it.
    /// Failed checks spend their measured reservation too. Unknown cost/cleanup
    /// blocks dispatch and acceptance while preserving the entire verification reserve.
    ///
    /// # Errors
    /// Refuses stale or repeated observations, unsettled workers, unavailable evidence,
    /// impossible cancellation and costs above the reserved bound.
    pub fn record_verification(
        &mut self,
        expected: &Expected<'_>,
        observation: &Verification<'_>,
        event_id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<String> {
        self.read_object(&observation.evidence, deadline)?;
        let body = serde_json::to_vec(&Observation {
            attempt: expected.attempt.as_str(),
            attempt_generation: expected.attempt_generation.to_string(),
            subject: observation.subject.as_str(),
            evidence: &observation.evidence,
            verdict: observation.verdict,
            used_ms: observation.used_ms,
            cleanup_settled: observation.cleanup_settled,
        })?;
        self.transaction(deadline, |tx| {
            let current = head(tx, expected.task.as_str())?;
            same_generation(&current, expected.task_generation)?;
            require_attempt(tx, expected, true)?;
            if !matches!(current.state.as_str(), "verifying" | "cancellation_requested")
                || current.accepted_event.is_some()
            {
                return Err(Error::Outstanding);
            }
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM verifications WHERE attempt_id=?)",
                [expected.attempt.as_str()], |row| row.get(0),
            )?;
            if exists { return Err(Error::Conflict); }
            if observation.used_ms.is_some_and(|used| used > current.reserved_verify_ms) {
                return Err(Error::Budget);
            }
            if observation.verdict == VerificationVerdict::Cancelled && !current.cancellation {
                return Err(Error::Conflict);
            }
            let reconciled = observation.used_ms.is_some() && observation.cleanup_settled;
            let state = if !reconciled {
                "effect_unknown"
            } else if current.cancellation {
                "cancellation_requested"
            } else {
                match observation.verdict {
                    VerificationVerdict::Passed => "verifying",
                    VerificationVerdict::Failed => "repair_pending",
                    VerificationVerdict::Invalid | VerificationVerdict::Error
                    | VerificationVerdict::Timeout => "failed",
                    VerificationVerdict::Cancelled => return Err(Error::Conflict),
                }
            };
            let charged = if reconciled { observation.used_ms.unwrap_or(0) } else { 0 };
            let generation = next(expected.task_generation)?;
            tx.execute(
                "INSERT INTO artifacts(digest,size) VALUES(?,?) ON CONFLICT(digest) DO NOTHING",
                params![observation.evidence.digest, number(observation.evidence.size)?],
            )?;
            let size: u64 = tx.query_row("SELECT size FROM artifacts WHERE digest=?",
                [&observation.evidence.digest], |row| super::read_number(row, 0))?;
            if size != observation.evidence.size { return Err(Error::Corrupt); }
            event(tx, event_id.as_str(), expected.task.as_str(), &generation, "verification_observed")?;
            tx.execute("UPDATE events SET body=? WHERE id=?", params![body, event_id.as_str()])?;
            tx.execute(
                "INSERT INTO verifications(attempt_id,event_id,subject_digest,evidence_digest,verdict,used_ms,cleanup_settled) VALUES(?,?,?,?,?,?,?)",
                params![expected.attempt.as_str(), event_id.as_str(), observation.subject.as_str(),
                    observation.evidence.digest, verdict_name(observation.verdict),
                    observation.used_ms.map(number).transpose()?, observation.cleanup_settled],
            )?;
            tx.execute(
                "UPDATE tasks SET generation=?,state=?,spent_ms=spent_ms+?,reserved_verify_ms=reserved_verify_ms-? WHERE id=?",
                params![generation,state,number(charged)?,number(charged)?,expected.task.as_str()],
            )?;
            Ok(generation)
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct VerifiedSubject {
    pub(super) subject: String,
    pub(super) evidence: Object,
}

pub(super) fn require_verified(
    connection: &rusqlite::Connection,
    attempt: &str,
    subject: &VerifiedSubject,
) -> Result<()> {
    let matches: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM verifications v JOIN artifacts a ON a.digest=v.evidence_digest WHERE v.attempt_id=? AND v.subject_digest=? AND v.evidence_digest=? AND a.size=? AND v.verdict='passed' AND v.used_ms IS NOT NULL AND v.cleanup_settled=1)",
        params![attempt, subject.subject, subject.evidence.digest, number(subject.evidence.size)?],
        |row| row.get(0),
    )?;
    if !matches {
        return Err(Error::Outstanding);
    }
    Ok(())
}

fn verdict_name(verdict: VerificationVerdict) -> &'static str {
    match verdict {
        VerificationVerdict::Passed => "passed",
        VerificationVerdict::Failed => "failed",
        VerificationVerdict::Invalid => "invalid",
        VerificationVerdict::Error => "error",
        VerificationVerdict::Timeout => "timeout",
        VerificationVerdict::Cancelled => "cancelled",
    }
}
