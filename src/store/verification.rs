//! Durable verification outcomes and conservative reservation settlement.

use super::run_records::{self, RunRecord, identity_bound_elsewhere};
use super::{
    Error, Expected, Object, PublishedAcceptance, Result, Store, event, head, next, number,
    register_evidence, require_attempt, same_generation,
};
use crate::contracts::control::{EvidenceRef, MAX_EVIDENCE_NAME_BYTES};
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

/// The identity a receipt names an evidence object by (B09b; `EvidenceRefV1`'s three members that
/// the object itself does not carry). The ledger stores it beside the object's digest, so a view
/// can return the reference it was recorded with — never one invented at read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvidenceIdentity<'a> {
    /// The artifact's identity as the receipt names it.
    pub artifact_id: UuidV4<'a>,
    /// Its media type (ASCII, 1..=128 bytes).
    pub media_type: &'a str,
    /// Its schema (ASCII, 1..=128 bytes).
    pub schema_id: &'a str,
}

impl<'a> EvidenceIdentity<'a> {
    /// The identity a wire reference carries for `object`: the one door where a reference and an
    /// object meet, refusing a reference whose digest or size is not the object's (B09 E1).
    /// # Errors
    /// `Invalid` when `reference` does not name `object`, or its names are not well formed.
    pub fn of(reference: &'a EvidenceRef, object: &Object) -> Result<Self> {
        if reference.sha256 != object.digest || reference.byte_length != object.size {
            return Err(Error::Invalid);
        }
        let identity = Self {
            artifact_id: UuidV4::parse(&reference.artifact_id).map_err(|_| Error::Invalid)?,
            media_type: &reference.media_type,
            schema_id: &reference.schema_id,
        };
        identity.check()?;
        Ok(identity)
    }

    /// The wire reference this identity and `object` make together.
    #[must_use]
    pub fn reference(&self, object: &Object) -> EvidenceRef {
        EvidenceRef {
            artifact_id: self.artifact_id.as_str().to_owned(),
            sha256: object.digest.clone(),
            byte_length: object.size,
            media_type: self.media_type.to_owned(),
            schema_id: self.schema_id.to_owned(),
        }
    }

    /// The names are what the wire admits: ASCII, 1..=128 bytes each.
    pub(super) fn check(&self) -> Result<()> {
        let well_formed =
            |name: &str| (1..=MAX_EVIDENCE_NAME_BYTES).contains(&name.len()) && name.is_ascii();
        if well_formed(self.media_type) && well_formed(self.schema_id) {
            Ok(())
        } else {
            Err(Error::Invalid)
        }
    }
}

/// An object together with the identity a receipt names it by (B09b): what an acceptance is handed.
#[derive(Clone, Debug)]
pub struct Identified<'a> {
    /// The object, as [`super::Store::publish`] returned it.
    pub object: Object,
    /// The identity it is named by.
    pub identity: EvidenceIdentity<'a>,
}

/// The 64 criteria a check satisfied, stored as sixteen lower-hex digits (B17; a `u64` above
/// `i64::MAX` cannot be an INTEGER column).
pub(super) fn criteria_text(criteria: Option<u64>) -> Option<String> {
    criteria.map(|bits| format!("{bits:016x}"))
}

/// Exact subject and immutable evidence for one completed verifier invocation.
/// Unknown cost or cleanup retains the reservation and blocks new work.
#[derive(Clone, Debug)]
pub struct Verification<'a> {
    pub verdict: VerificationVerdict,
    pub subject: Sha256Digest<'a>,
    pub evidence: Object,
    /// The identity the receipt names `evidence` by (B09b).
    pub identity: EvidenceIdentity<'a>,
    /// The criteria the check satisfied, as the driver's bit pattern; `None` when the verdict
    /// carries no criteria (B17 reads it back to measure loop progress).
    pub satisfied_criteria: Option<u64>,
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
    /// cancellation, stale generations, and an evidence list omitting the receipt; `Invalid` for
    /// an object identity name the wire would refuse (B09b).
    pub fn prepare_verified_acceptance(
        &self,
        expected: &Expected<'_>,
        event_id: UuidV4<'_>,
        subject: Sha256Digest<'_>,
        evidence: &Object,
        objects: &[Identified<'_>],
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
        if !objects.iter().any(|each| each.object == *evidence) {
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
    /// impossible cancellation and costs above the reserved bound; `Invalid` for an identity
    /// name the wire would refuse; `Conflict` for an artifact id already bound to another digest;
    /// the inventory bound past what a backup copies (the evidence registers through
    /// `register_evidence`, as every other committed object does).
    pub fn record_verification(
        &mut self,
        expected: &Expected<'_>,
        observation: &Verification<'_>,
        event_id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<String> {
        self.record_verification_with_records(expected, observation, &[], &[], event_id, deadline)
    }

    /// [`Store::record_verification`], committing the check's run records in the same transaction,
    /// keyed by this verification's event (R13): the verdict, the receipt and the records commit or
    /// vanish together. The verification row is written first, so a record whose artifact id is
    /// the receipt's under another digest is refused `Conflict` by the same rule as any rebinding.
    /// `cited` are the objects the records name (a step's captures, an output's readback identity):
    /// registered here so the inventory bound counts them and a backup copies them (B14a-3c review).
    /// # Errors
    /// As [`Store::record_verification`]; `Invalid` for a repeated kind (nothing written);
    /// `Conflict` for an artifact id bound to another digest; `Corrupt` for an object registered
    /// with another size; the inventory bound past what a backup copies.
    pub fn record_verification_with_records(
        &mut self,
        expected: &Expected<'_>,
        observation: &Verification<'_>,
        records: &[RunRecord<'_>],
        cited: &[Object],
        event_id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<String> {
        observation.identity.check()?;
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
            // Registered through the one door that also keeps the inventory bound (R13 review).
            register_evidence(tx, std::slice::from_ref(&observation.evidence))?;
            if identity_bound_elsewhere(tx, observation.identity.artifact_id.as_str(), &observation.evidence.digest)? {
                return Err(Error::Conflict);
            }
            event(tx, event_id.as_str(), expected.task.as_str(), &generation, "verification_observed")?;
            tx.execute("UPDATE events SET body=? WHERE id=?", params![body, event_id.as_str()])?;
            tx.execute(
                "INSERT INTO verifications(attempt_id,event_id,subject_digest,evidence_digest,verdict,used_ms,cleanup_settled,\
                 evidence_artifact_id,evidence_media_type,evidence_schema_id,satisfied_criteria) VALUES(?,?,?,?,?,?,?,?,?,?,?)",
                params![expected.attempt.as_str(), event_id.as_str(), observation.subject.as_str(),
                    observation.evidence.digest, verdict_name(observation.verdict),
                    observation.used_ms.map(number).transpose()?, observation.cleanup_settled,
                    observation.identity.artifact_id.as_str(), observation.identity.media_type,
                    observation.identity.schema_id, criteria_text(observation.satisfied_criteria)],
            )?;
            tx.execute(
                "UPDATE tasks SET generation=?,state=?,spent_ms=spent_ms+?,reserved_verify_ms=reserved_verify_ms-? WHERE id=?",
                params![generation,state,number(charged)?,number(charged)?,expected.task.as_str()],
            )?;
            // The check's records, keyed by this verification's event; never the settling event.
            run_records::commit(tx, expected.attempt.as_str(), event_id.as_str(), records, false)?;
            register_evidence(tx, cited)?;
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

/// Every verdict, so a stored spelling is read back through the one table that wrote it.
const VERDICTS: [VerificationVerdict; 6] = [
    VerificationVerdict::Passed,
    VerificationVerdict::Failed,
    VerificationVerdict::Invalid,
    VerificationVerdict::Error,
    VerificationVerdict::Timeout,
    VerificationVerdict::Cancelled,
];

/// The verdict a stored spelling names; `Corrupt` for any other, since only `verdict_name` writes it.
pub(super) fn parse_verdict(text: &str) -> Result<VerificationVerdict> {
    VERDICTS
        .into_iter()
        .find(|verdict| verdict_name(*verdict) == text)
        .ok_or(Error::Corrupt)
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
