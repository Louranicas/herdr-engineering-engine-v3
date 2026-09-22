//! Finalize only an already committed cancellation with durable settled returns.
//! No execution, cleanup, delivery, grant or original task-clock resume occurs.
use crate::{Result, checked, manifest, recovery};
use habitat_engine::contracts::{Generation, UuidV4, receipt::Name};
use habitat_engine::store::{
    Object, Principal, RecoveryInventory, RecoveryLimits, Stop, Store, TaskHead,
};
use serde_json::{Value, json};
use std::{
    fs::File,
    path::Path,
    time::{Duration, Instant},
};

const LIMIT: Duration = Duration::from_secs(60);
const REASON: &str = "recovery_cancelled";

/// Internal operator request. These fields confer no authority on their own.
#[derive(Clone, Copy)]
pub struct Request<'a> {
    pub root: &'a Path,
    pub store_generation: &'a str,
    pub epoch: &'a str,
    pub task: &'a str,
    pub expected_generation: &'a str,
    pub event: &'a str,
}
impl Request<'_> {
    fn validate(self) -> Result<()> {
        for value in [self.store_generation, self.epoch, self.task, self.event] {
            checked(UuidV4::parse(value))?;
        }
        checked(self.expected_generation.parse::<Generation>())?;
        Ok(())
    }
}

/// Open one existing Store and commit a terminal cancellation, or read back that
/// exact earlier operation. The existing Store still owns the transaction.
/// # Errors
/// Refuses unknown/unsettled work, absent cancellation/latest verifier, foreign
/// ownership, stale identity, conflicting history and any evidence/report error.
pub fn finish(request: Request<'_>, output: &Path) -> Result<()> {
    finish_until(request, output, Instant::now() + LIMIT)
}

/// Use an unchanged bounded administrative deadline, never a new task budget.
/// # Errors
/// Same refusals as [`finish`], plus expired/overlong administrative deadlines.
pub fn finish_until(request: Request<'_>, output: &Path, deadline: Instant) -> Result<()> {
    let origin = Instant::now();
    if deadline <= origin || deadline.duration_since(origin) > LIMIT {
        return Err("cancellation finalization deadline".into());
    }
    request.validate()?;
    recovery::paths(request.root, output)?;
    manifest::private_dir(output)?;
    sync(output.parent().ok_or("report parent")?)?;
    let opened = checked(Store::open(
        request.root,
        checked(UuidV4::parse(request.store_generation))?,
        checked(UuidV4::parse(request.epoch))?,
        false,
        deadline,
    ));
    let (operation, readback) = match opened {
        Ok(mut store) => {
            let operation = finalize(&mut store, request, deadline);
            // Even an uncertain commit or finalization error must attempt readback.
            let readback = readback(&mut store, request, deadline);
            (operation, readback)
        }
        Err(error) => (
            Err(format!("Store open: {error}")),
            Err("Store not opened".into()),
        ),
    };
    let report = json!({
        "kind":"hee3-development-cancellation-finalization/1",
        "operation":observed(&operation),"readback":observed(&readback),
        "recovery_complete":false,"execution_resumed":false,"delivery_performed":false,
        "new_worker_or_verifier_charge":false,
        "administrative_elapsed_ms":origin.elapsed().as_millis().to_string()
    });
    let publication = publish_report(output, &report, deadline);
    // Retain actual outcome separately from report failures. A committed result
    // does not become a rollback just because its diagnostic publication failed.
    let mut errors = Vec::new();
    if let Err(error) = &operation {
        errors.push(format!("operation: {error}"));
    }
    if let Err(error) = &readback {
        errors.push(format!("readback: {error}"));
    }
    if let Err(error) = publication {
        errors.push(format!(
            "report: {error}; operation outcome: {}",
            observed(&operation)
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
fn observed(value: &Result<Value>) -> Value {
    match value {
        Ok(value) => json!({"status":"ok","value":value}),
        Err(error) => json!({"status":"error","error":error}),
    }
}
fn sync(path: &Path) -> Result<()> {
    checked(File::open(path).and_then(|file| file.sync_all()))
}
fn publish_report(output: &Path, report: &Value, deadline: Instant) -> Result<()> {
    recovery::tick(deadline)?;
    let bytes = recovery::bounded_json(report)?;
    manifest::write(&output.join("report.json"), &bytes)?;
    sync(output)?;
    recovery::tick(deadline)?;
    manifest::json(
        &output.join("complete.json"),
        &json!({
            "kind":"hee3-development-cancellation-report-complete/1",
            "report_sha256":manifest::digest(&bytes),"report_bytes":bytes.len(),
            "meaning":"complete diagnostic report, not necessarily successful operation"
        }),
    )?;
    sync(output)
}
fn inventory(store: &mut Store, epoch: &str, deadline: Instant) -> Result<RecoveryInventory> {
    checked(store.recovery_inventory(
        checked(UuidV4::parse(epoch))?,
        RecoveryLimits {
            rows: 1024,
            bytes: 1_048_576,
        },
        deadline,
    ))
}
fn readback(store: &mut Store, request: Request<'_>, deadline: Instant) -> Result<Value> {
    let principal = checked(Principal::new(
        rustix::process::geteuid().as_raw(),
        "operator",
    ))?;
    checked(store.get(&principal, checked(UuidV4::parse(request.task))?, deadline))?;
    let mut snapshot = inventory(store, request.epoch, deadline)?;
    snapshot.tasks.retain(|row| row.head.id == request.task);
    snapshot.attempts.retain(|row| row.task == request.task);
    let attempts: std::collections::BTreeSet<_> =
        snapshot.attempts.iter().map(|row| row.id.clone()).collect();
    snapshot
        .verifications
        .retain(|row| attempts.contains(&row.attempt));
    snapshot.acceptances.retain(|row| row.task == request.task);
    snapshot.stops.retain(|row| row.task == request.task);
    snapshot
        .pending_delivery
        .retain(|row| row.task.as_deref() == Some(request.task));
    snapshot.instances.retain(|row| row.task_id == request.task);
    snapshot
        .pins
        .retain(|row| attempts.contains(&row.attempt_id));
    let mut value = recovery::inventory_json(&snapshot);
    value["scope"] = json!("authorized target task projection of complete bounded Store inventory");
    value["rows_and_payload_accounting"] = json!("whole Store snapshot, before target projection");
    Ok(value)
}
fn finalize(store: &mut Store, request: Request<'_>, deadline: Instant) -> Result<Value> {
    let principal = checked(Principal::new(
        rustix::process::geteuid().as_raw(),
        "operator",
    ))?;
    let task = checked(UuidV4::parse(request.task))?;
    let head = checked(store.get(&principal, task, deadline))?;
    if head.accepted_event.is_some() {
        return Err(format!(
            "historical acceptance remains {:?}",
            head.accepted_event
        ));
    }
    if !head.cancellation {
        return Err("committed cancellation required; failure/exhaustion is distinct".into());
    }
    let expected = checked(request.expected_generation.parse::<Generation>())?;
    let snapshot = inventory(store, request.epoch, deadline)?;
    let proof = proof(request, &head, &snapshot)?;
    let evidence: Object = checked(serde_json::from_value(
        json!({"digest":manifest::digest(&proof),"size":checked(u64::try_from(proof.len()))?}),
    ))?;
    if let Some(stop) = snapshot.stops.iter().find(|row| row.task == request.task) {
        if head.state != "cancelled"
            || head.generation != checked(expected.next())?.to_string()
            || stop.event != request.event
            || stop.state != "cancelled"
            || stop.reason != REASON
            || stop.evidence != evidence.digest()
        {
            return Err("conflicting historical stop; no replay".into());
        }
        if checked(store.read_object(&evidence, deadline))? != proof {
            return Err("replay evidence differs".into());
        }
        return Ok(outcome(request, &head.generation, true, &evidence));
    }
    if head.generation != expected.to_string() || head.state != "cancellation_requested" {
        return Err("stale generation or nonpending cancellation".into());
    }
    let published =
        checked(store.publish(&proof, checked(UuidV4::parse(request.event))?, deadline))?;
    if published != evidence {
        return Err("published request evidence differs".into());
    }
    let reason = checked(Name::new(REASON))?;
    let stopped = checked(store.finish_unaccepted(
        &principal,
        Stop {
            task,
            generation: expected,
            reason: &reason,
            evidence: &evidence,
            event: checked(UuidV4::parse(request.event))?,
        },
        deadline,
    ))?;
    if !stopped.cancelled {
        return Err("terminal owner returned noncancelled outcome".into());
    }
    Ok(outcome(request, &stopped.generation, false, &evidence))
}
fn outcome(request: Request<'_>, generation: &str, replayed: bool, evidence: &Object) -> Value {
    json!({"task":request.task,"event":request.event,"epoch":request.epoch,
        "store_generation":request.store_generation,"generation":generation,
        "state":"cancelled","replayed":replayed,"evidence":evidence,
        "terminal_operation_committed":true,
        "current_invocation_wrote_terminal":!replayed,"delivery":"readback_only"})
}
fn proof(request: Request<'_>, head: &TaskHead, snapshot: &RecoveryInventory) -> Result<Vec<u8>> {
    let attempts: Vec<_> = snapshot
        .attempts
        .iter()
        .filter(|row| row.task == request.task)
        .collect();
    let mut latest: Option<(u64, &str)> = None;
    for attempt in &attempts {
        if attempt.state != "settled"
            || !matches!(attempt.effect.as_str(), "none" | "committed")
            || attempt.cleanup != "settled"
            || attempt.used_ms.is_none()
        {
            return Err("unsettled attempt/effect/usage/cleanup".into());
        }
        let generation = checked(attempt.generation.parse::<Generation>())?.value();
        if latest.is_none_or(|(prior, _)| generation > prior) {
            latest = Some((generation, &attempt.id));
        }
    }
    let (_, latest) = latest.ok_or("preparation custody unavailable: no attempts")?;
    let checks: Vec<_> = snapshot
        .verifications
        .iter()
        .filter(|row| attempts.iter().any(|a| a.id == row.attempt))
        .collect();
    if !checks.iter().any(|row| row.attempt == latest) {
        return Err("latest verifier return absent".into());
    }
    if checks
        .iter()
        .any(|row| row.used_ms.is_none() || !row.cleanup_settled)
    {
        return Err("verifier usage/cleanup unresolved".into());
    }
    recovery::bounded_json(&json!({
        "kind":"hee3-development-cancellation-request/1","reason":REASON,
        "store_generation":request.store_generation,"epoch":request.epoch,
        "principal":{"uid":rustix::process::geteuid().as_raw(),"role":"operator"},
        "task":request.task,"expected_generation":request.expected_generation,"event":request.event,
        "criteria":head.criteria,"spent_ms":head.spent_ms,
        "attempts":attempts.iter().map(|x|json!({"id":x.id,"generation":x.generation,
            "effect":x.effect,"cleanup":x.cleanup,"used_ms":x.used_ms})).collect::<Vec<_>>(),
        "verifications":checks.iter().map(|x|json!({"attempt":x.attempt,"event":x.event,
            "subject":x.subject,"evidence":x.evidence,"verdict":x.verdict,
            "used_ms":x.used_ms,"cleanup_settled":x.cleanup_settled})).collect::<Vec<_>>()
    }))
}
