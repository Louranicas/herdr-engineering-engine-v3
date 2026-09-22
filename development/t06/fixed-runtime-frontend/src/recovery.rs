//! Internal development readback of an existing Store. Never resumes execution.
//! Store custody is logically read-only here; WAL/profile and lock-file effects
//! from the existing Store opener remain possible. This is not forensic access.
use crate::{Result, checked, manifest};
use habitat_engine::contracts::{
    Generation, UuidV4,
    events::{CursorDisposition, EventCursorV1},
    parse_u64_decimal,
};
use habitat_engine::recovery::{
    self as policy, AcceptanceCandidate, Acknowledgement, AttemptFacts, AttemptState, Cleanup,
    CleanupReadback, CursorFacts, Effect, Evidence, Lease, LedgerFacts, Mode, Observations,
    PiQueueCustody, TaskFacts, TaskHistory, TaskState, Verdict, Verification, WorkspaceReadback,
};
pub use habitat_engine::recovery::{Dimension, ProcessCustody as Custody};
use habitat_engine::store::{
    DurableAttempt, DurableTask, DurableVerification, RecoveryInventory, RecoveryLimits, Store,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const REPORT_LIMIT: usize = 8 * 1024 * 1024;
const READ_LIMIT: Duration = Duration::from_secs(60);

/// Reopen one exact existing generation, report complete bounded durable facts,
/// then release custody. The report is not admission or a recovery disposition.
///
/// # Errors
/// Refuses unsafe paths, invalid identities, deadline, unavailable custody,
/// malformed/oversized inventory and publication failures. No task transition,
/// dispatch, cancellation, cleanup, acknowledgement or replay is performed.
pub fn inspect(root: &Path, generation: &str, epoch: &str, output: &Path) -> Result<()> {
    inspect_until(root, generation, epoch, output, Instant::now() + READ_LIMIT)
}

/// Same readback with the caller's original bounded inspection deadline.
/// This deadline does not replace any task execution clock.
/// # Errors
/// Same refusals as [`inspect`], including expired or overlong inspection windows.
pub fn inspect_until(
    root: &Path,
    generation: &str,
    epoch: &str,
    output: &Path,
    deadline: Instant,
) -> Result<()> {
    inspect_cursor_until(root, generation, epoch, output, None, deadline)
}

/// Optional complete RC03 cursor JSON for snapshot comparison, never event replay.
/// # Errors
/// Refuses malformed/overbound cursor, invalid time/sequence or the existing readback failures.
pub fn inspect_cursor(
    root: &Path,
    generation: &str,
    epoch: &str,
    output: &Path,
    cursor: &str,
) -> Result<()> {
    inspect_cursor_until(
        root,
        generation,
        epoch,
        output,
        Some(cursor),
        Instant::now() + READ_LIMIT,
    )
}

fn inspect_cursor_until(
    root: &Path,
    generation: &str,
    epoch: &str,
    output: &Path,
    cursor: Option<&str>,
    deadline: Instant,
) -> Result<()> {
    let origin = Instant::now();
    let cursor: Option<EventCursorV1> = cursor
        .map(|raw| -> Result<EventCursorV1> {
            if raw.len() > 1024 {
                return Err("cursor bound".into());
            }
            let value: EventCursorV1 = checked(serde_json::from_str(raw))?;
            checked(value.validate())?;
            Ok(value)
        })
        .transpose()?;
    if deadline <= origin || deadline.duration_since(origin) > READ_LIMIT {
        return Err("inspection deadline".into());
    }
    let generation = checked(UuidV4::parse(generation))?;
    let epoch = checked(UuidV4::parse(epoch))?;
    paths(root, output)?;
    manifest::private_dir(output)?;
    checked(File::open(output.parent().ok_or("report parent")?).and_then(|f| f.sync_all()))?;
    let observed = read(root, generation, epoch, deadline).and_then(|inventory| {
        let now = checked(SystemTime::now().duration_since(UNIX_EPOCH))?;
        let now = checked(u64::try_from(now.as_millis()))?;
        let disposition = cursor
            .as_ref()
            .map_or(Ok(CursorDisposition::SnapshotOnly), |cursor| {
                checked(inventory.cursor_disposition(cursor, now))
            })?;
        Ok((inventory, disposition, now))
    });
    match observed {
        Ok((inventory, disposition, now)) => {
            let (identities, custody, classified) = physical_custody(&inventory, deadline);
            let (reconciliation, permits_execution) =
                reconciliation(&inventory, &classified, cursor.as_ref());
            if permits_execution {
                return Err("policy permitted execution; report refused".into());
            }
            let report = json!({
                "kind":"hee3-development-recovery-inspection/1",
                "inspection_completed":true,
                "cursor_disposition":disposition,
                "cursor_supplied":cursor.is_some(),
                "cursor":cursor,
                "cursor_checked_unix_ms":now.to_string(),
                "replay_authorized":false,
                "filter_visibility_retention":"unassessed",
                "recovery_complete":false,
                "execution_resumed":false,
                "policy_permits_execution":permits_execution,
                "reconciliation":reconciliation,
                "evidence_availability":"unassessed",
                "physical_process_custody":custody,
                "physical_process_identities":identities,
                "physical_process_scope":"report of one bounded /proc read per running attempt's process identity; no signal, kill, dispatch, resume or custody promotion",
                "pi_queue_custody":"unreconciled",
                "scope":"internal existing-ledger readback; no RC03 action or admission",
                "elapsed_ms":origin.elapsed().as_millis().to_string(),
                "store_root":root,
                "limits":{"rows":1024,"sqlite_payload_bytes":1_048_576,"report_bytes":REPORT_LIMIT},
                "inventory":inventory_json(&inventory)
            });
            tick(deadline)?;
            let bytes = bounded_json(&report)?;
            manifest::write(&output.join("report.json"), &bytes)?;
            checked(File::open(output).and_then(|f| f.sync_all()))?;
            tick(deadline)?;
            manifest::json(
                &output.join("complete.json"),
                &json!({
                    "kind":"hee3-development-recovery-inspection-complete/1",
                    "report_sha256":manifest::digest(&bytes),"report_bytes":bytes.len(),
                    "epoch":inventory.epoch,"generation":inventory.generation,
                    "event_high_water":inventory.event_high_water.to_string()
                }),
            )?;
            checked(File::open(output).and_then(|f| f.sync_all()))
        }
        Err(original) => {
            let error = json!({"kind":"hee3-development-recovery-inspection-error/1",
                "inspection_completed":false,"error":original,
                "generation":generation.as_str(),"epoch":epoch.as_str()});
            let retained = manifest::json(&output.join("error.json"), &error)
                .and_then(|()| checked(File::open(output).and_then(|f| f.sync_all())));
            match retained {
                Ok(()) => Err(original),
                Err(publication) => Err(format!(
                    "{original}; error publication failed: {publication}"
                )),
            }
        }
    }
}

pub(crate) fn tick(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline {
        Err("inspection deadline".into())
    } else {
        Ok(())
    }
}
pub(crate) fn paths(root: &Path, output: &Path) -> Result<()> {
    if root.to_str().is_none()
        || output.to_str().is_none()
        || !root.is_absolute()
        || checked(root.canonicalize())? != root
        || !output.is_absolute()
        || output.file_name().is_none()
        || output.starts_with(root)
        || root.starts_with(output)
    {
        return Err("inspection path custody".into());
    }
    let parent = output.parent().ok_or("report parent")?;
    if checked(parent.canonicalize())? != parent {
        return Err("report parent canonical custody".into());
    }
    let metadata = checked(fs::symlink_metadata(parent))?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err("report parent owner custody".into());
    }
    Ok(())
}
fn read(
    root: &Path,
    generation: UuidV4<'_>,
    epoch: UuidV4<'_>,
    deadline: Instant,
) -> Result<RecoveryInventory> {
    let mut store = checked(Store::open_inspection(root, generation, epoch, deadline))?;
    checked(store.recovery_inventory(
        epoch,
        RecoveryLimits {
            rows: 1024,
            bytes: 1_048_576,
        },
        deadline,
    ))
}

struct Bounded(Vec<u8>);
impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > REPORT_LIMIT - self.0.len() {
            return Err(io::Error::other("inspection report bound"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(crate) fn bounded_json(report: &Value) -> Result<Vec<u8>> {
    let mut out = Bounded(Vec::new());
    checked(serde_json::to_writer(&mut out, report))?;
    Ok(out.0)
}
pub(crate) fn inventory_json(v: &RecoveryInventory) -> Value {
    json!({
        "epoch":v.epoch,"generation":v.generation,"mode":v.mode,
        "event_high_water":v.event_high_water.to_string(),"rows":v.rows,"payload_bytes":v.payload_bytes,
        "tasks":v.tasks.iter().map(|x|json!({"id":x.head.id,"generation":x.head.generation,
            "state":x.head.state,"cancellation":x.head.cancellation,"accepted_event":x.head.accepted_event,
            "criteria":x.head.criteria,"spent_ms":x.head.spent_ms,"reserved_work_ms":x.head.reserved_work_ms,
            "reserved_verify_ms":x.head.reserved_verify_ms,"limit_ms":x.limit_ms,
            "principal_uid":x.principal_uid,"principal_role":x.principal_role})).collect::<Vec<_>>(),
        "attempts":v.attempts.iter().map(|x|json!({"id":x.id,"task":x.task,"generation":x.generation,
            "state":x.state,"effect":x.effect,"cleanup":x.cleanup,"used_ms":x.used_ms})).collect::<Vec<_>>(),
        "verifications":v.verifications.iter().map(|x|json!({"attempt":x.attempt,"event":x.event,
            "subject":x.subject,"evidence":x.evidence,"verdict":x.verdict,
            "used_ms":x.used_ms,"cleanup_settled":x.cleanup_settled})).collect::<Vec<_>>(),
        "acceptances":v.acceptances.iter().map(|x|json!({"event":x.event,"task":x.task,
            "attempt":x.attempt,"generation":x.generation,"criteria":x.criteria,"manifest":x.manifest})).collect::<Vec<_>>(),
        "stops":v.stops.iter().map(|x|json!({"task":x.task,"event":x.event,"evidence":x.evidence,
            "reason":x.reason,"state":x.state})).collect::<Vec<_>>(),
        "pending_delivery":v.pending_delivery.iter().map(|x|json!({"event":x.event,"recipient":x.recipient,
            "sequence":x.sequence.to_string(),"task":x.task,"roster":x.roster})).collect::<Vec<_>>(),
        "instances":v.instances,"pins":v.pins
    })
}

/// Kernel-described identity of one process, as a worker's roster observation
/// records it in `actual_identity`: the PID, field 22 of `/proc/<pid>/stat`
/// (start time in clock ticks) and the PID namespace link. Any other
/// `actual_identity` text (an executable path, a digest) is not a process
/// identity and is never classified.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub start_ticks: u64,
    pub namespace: String,
}

impl ProcessIdentity {
    /// Parse one roster observation's `actual_identity` text. `None` means the
    /// text is not a process identity; it never means the process is absent.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        if text.len() > 256 {
            return None;
        }
        let value: Self = serde_json::from_str(text).ok()?;
        (value.pid >= 1 && namespace_link(&value.namespace)).then_some(value)
    }
}

fn namespace_link(text: &str) -> bool {
    text.strip_prefix("pid:[")
        .and_then(|rest| rest.strip_suffix(']'))
        .is_some_and(|digits| {
            !digits.is_empty() && digits.len() <= 20 && digits.bytes().all(|b| b.is_ascii_digit())
        })
}

/// What `/proc/<pid>` says about the PID an observation named. It carries no
/// PID of its own: the read was made for the observed PID, so the policy cannot
/// compare two different PIDs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LiveIdentity {
    pub start_ticks: u64,
    pub namespace: String,
    /// Field 3 of `/proc/<pid>/stat`, retained for the reader; the policy does
    /// not classify on it (a zombie holding the same identity reads as live).
    pub state: char,
}

/// The outcome of one bounded `/proc` read: the policy's only world input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveRead {
    Present(LiveIdentity),
    /// No such PID (`ENOENT`/`ESRCH`).
    Absent,
    /// Read refused or malformed; the text is kept, never interpreted.
    Unreadable(String),
}

/// The pure policy: from what the ledger observed and what `/proc` now says to
/// exactly one custody arm. No I/O, no clock, no authority.
#[must_use]
pub fn classify(observed: &ProcessIdentity, live: &LiveRead) -> Custody {
    match live {
        LiveRead::Unreadable(error) => Custody::Unreadable {
            error: error.clone(),
        },
        LiveRead::Absent => Custody::Absent,
        LiveRead::Present(identity) => {
            let mut differs = Vec::new();
            if identity.start_ticks != observed.start_ticks {
                differs.push(Dimension::StartTicks);
            }
            if identity.namespace != observed.namespace {
                differs.push(Dimension::Namespace);
            }
            if differs.is_empty() {
                Custody::LiveSameIdentity
            } else {
                Custody::PidReused { differs }
            }
        }
    }
}

/// The report's one-line `physical_process_custody`: `unreconciled` when no
/// running attempt carries a process identity (every other adapter identity),
/// the shared arm when every classified attempt agrees, else `mixed`.
#[must_use]
pub fn summary(classified: &[Custody]) -> &'static str {
    let mut names = classified.iter().map(Custody::name);
    match names.next() {
        None => "unreconciled",
        Some(first) => {
            if names.all(|name| name == first) {
                first
            } else {
                "mixed"
            }
        }
    }
}

const PROC_STAT_LIMIT: u64 = 4096;

/// Thin I/O: one bounded read of `/proc/<pid>/stat` and one of `/proc/<pid>/ns/pid`.
fn live_read(pid: u32, deadline: Instant) -> LiveRead {
    if tick(deadline).is_err() {
        return LiveRead::Unreadable("inspection deadline".into());
    }
    let stat = match read_stat(pid) {
        Ok(text) => text,
        Err(error) if gone(&error) => return LiveRead::Absent,
        Err(error) => return LiveRead::Unreadable(format!("stat: {error}")),
    };
    let Some((state, start_ticks)) = parse_stat(&stat) else {
        return LiveRead::Unreadable("malformed stat".into());
    };
    let namespace = match fs::read_link(format!("/proc/{pid}/ns/pid")) {
        Ok(link) => link,
        Err(error) if gone(&error) => return LiveRead::Absent,
        Err(error) => return LiveRead::Unreadable(format!("ns: {error}")),
    };
    let Some(namespace) = namespace.to_str() else {
        return LiveRead::Unreadable("non-UTF-8 namespace link".into());
    };
    LiveRead::Present(LiveIdentity {
        start_ticks,
        namespace: namespace.into(),
        state,
    })
}

fn gone(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::NotFound
        || error.raw_os_error() == Some(rustix::io::Errno::SRCH.raw_os_error())
}

fn read_stat(pid: u32) -> io::Result<String> {
    let mut text = String::new();
    File::open(format!("/proc/{pid}/stat"))?
        .take(PROC_STAT_LIMIT)
        .read_to_string(&mut text)?;
    if u64::try_from(text.len())
        .ok()
        .is_none_or(|length| length >= PROC_STAT_LIMIT)
    {
        return Err(io::Error::other("stat bound"));
    }
    Ok(text)
}

/// proc(5): field 2 `comm` may hold spaces and parentheses, so fields are
/// counted after the last `)`: field 3 is the state, field 22 the start ticks.
fn parse_stat(stat: &str) -> Option<(char, u64)> {
    let close = stat.rfind(')')?;
    let mut fields = stat.get(close + 1..)?.split_whitespace();
    let mut state = fields.next()?.chars();
    let (first, second) = (state.next()?, state.next());
    if second.is_some() {
        return None;
    }
    let start_ticks = fields.nth(18)?.parse().ok()?;
    Some((first, start_ticks))
}

/// Every running attempt's pinned roster observation whose `actual_identity`
/// is a process identity, read once against `/proc` and classified. Entries
/// are ordered as the inventory orders pins (attempt, record).
fn physical_custody(
    inventory: &RecoveryInventory,
    deadline: Instant,
) -> (Vec<Value>, &'static str, Vec<(String, Custody)>) {
    let mut entries = Vec::new();
    let mut classified = Vec::new();
    let mut per_attempt: Vec<(String, Custody)> = Vec::new();
    for attempt in inventory.attempts.iter().filter(|a| a.state == "running") {
        for pin in inventory.pins.iter().filter(|p| p.attempt_id == attempt.id) {
            let Some(text) = pin
                .record
                .observation
                .as_ref()
                .and_then(|o| o.input.actual_identity.as_deref())
            else {
                continue;
            };
            let Some(observed) = ProcessIdentity::parse(text) else {
                continue;
            };
            let live = live_read(observed.pid, deadline);
            let custody = classify(&observed, &live);
            let live_json = match &live {
                LiveRead::Present(identity) => json!({
                    "pid":observed.pid,"start_ticks":identity.start_ticks,
                    "namespace":identity.namespace,"state":identity.state
                }),
                LiveRead::Absent | LiveRead::Unreadable(_) => Value::Null,
            };
            entries.push(json!({
                "attempt":attempt.id,"record_id":pin.record.head.record_id,
                "observed":observed,"live":live_json,"custody":custody.name(),
                "differs":custody.differs(),"error":custody.error()
            }));
            if !per_attempt.iter().any(|(id, _)| *id == attempt.id) {
                per_attempt.push((attempt.id.clone(), custody.clone()));
            }
            classified.push(custody);
        }
    }
    (entries, summary(&classified), per_attempt)
}

/// The engine policy's decision for every attempt in the inventory and for the
/// supplied cursor. The inspector holds no acknowledgement record, Pi readback,
/// cleanup or workspace readback and no receiver clock, and says so in the
/// inputs it passes; the decisions are reports and the second value is whether
/// any of them permits execution, which the report refuses to carry as true.
fn reconciliation(
    inventory: &RecoveryInventory,
    classified: &[(String, Custody)],
    cursor: Option<&EventCursorV1>,
) -> (Value, bool) {
    let mode = Mode::parse(&inventory.mode);
    let ledger = LedgerFacts {
        epoch: &inventory.epoch,
        mode: mode.unwrap_or(Mode::Reconciliation),
        event_high_water: inventory.event_high_water,
        restored_from: None,
    };
    let mut permits = mode.is_none();
    let mut attempts = Vec::new();
    for attempt in &inventory.attempts {
        let entry = match attempt_decision(&ledger, inventory, classified, attempt) {
            Ok(decision) => {
                permits |= decision.reconciliation.permits_execution();
                json!({
                    "attempt":attempt.id,"task":attempt.task,
                    "rule":decision.rule.id(),"decision":decision.reconciliation,
                    "name":decision.reconciliation.name()
                })
            }
            Err(error) => json!({"attempt":attempt.id,"task":attempt.task,"error":error}),
        };
        attempts.push(entry);
    }
    let cursor = cursor.map(|cursor| match parse_u64_decimal(&cursor.sequence) {
        Ok(sequence) => {
            let decision = policy::reconcile_cursor(
                &ledger,
                &CursorFacts {
                    epoch: &cursor.epoch,
                    sequence,
                },
            );
            permits |= decision.reconciliation.permits_execution();
            json!({"rule":decision.rule.id(),"decision":decision.reconciliation,
                "name":decision.reconciliation.name()})
        }
        Err(error) => json!({"error":format!("{error:?}")}),
    });
    (
        json!({
            "kind":"hee3-recovery-policy/1",
            "mode_parsed":mode.is_some(),
            "attempts":attempts,"cursor":cursor,
            "inputs":{"acknowledgement":"unrecorded","pi_queue":"unreconciled",
                "cleanup_readback":"not_read","workspace":"not_read","clock":null,
                "restored_from":null,"claim":null},
            "scope":"engine policy decisions over the durable inventory and the classified process custody; report only, consumed by nothing here"
        }),
        permits,
    )
}

fn attempt_decision(
    ledger: &LedgerFacts<'_>,
    inventory: &RecoveryInventory,
    classified: &[(String, Custody)],
    attempt: &DurableAttempt,
) -> Result<policy::Decision> {
    let task = inventory
        .tasks
        .iter()
        .find(|task| task.head.id == attempt.task)
        .ok_or("task row absent")?;
    let verification = inventory
        .verifications
        .iter()
        .rfind(|row| row.attempt == attempt.id);
    let task_facts = task_facts(task, verification)?;
    let attempt_facts = attempt_facts(inventory, attempt, verification)?;
    let process = classified
        .iter()
        .find(|(id, _)| *id == attempt.id)
        .map_or(Custody::Unobserved, |(_, custody)| custody.clone());
    let observed = Observations {
        claim: None,
        clock: None,
        process,
        pi_queue: PiQueueCustody::Unreconciled,
        cleanup: CleanupReadback::NotRead,
        workspace: WorkspaceReadback::NotRead,
    };
    Ok(policy::reconcile(
        ledger,
        &task_facts,
        &attempt_facts,
        &observed,
    ))
}

fn task_facts<'a>(
    task: &'a DurableTask,
    verification: Option<&'a DurableVerification>,
) -> Result<TaskFacts<'a>> {
    let state = TaskState::parse(&task.head.state).ok_or("task state")?;
    let history = match (task.head.cancellation, task.head.accepted_event.as_deref()) {
        (true, Some(_)) => TaskHistory::Contradictory,
        (false, Some(event)) => TaskHistory::Accepted {
            event,
            ordinal: None,
        },
        (true, None) => TaskHistory::Cancelled { ordinal: None },
        (false, None) => TaskHistory::Open,
    };
    let candidate = match verification {
        Some(row) if row.verdict == "passed" && task.head.accepted_event.is_none() => {
            AcceptanceCandidate::Prepared {
                verification_event: &row.event,
            }
        }
        _ => AcceptanceCandidate::None,
    };
    Ok(TaskFacts {
        generation: checked(task.head.generation.parse::<Generation>())?.value(),
        state,
        history,
        candidate,
    })
}

fn attempt_facts<'a>(
    inventory: &'a RecoveryInventory,
    attempt: &'a DurableAttempt,
    verification: Option<&'a DurableVerification>,
) -> Result<AttemptFacts<'a>> {
    let lease = inventory
        .instances
        .iter()
        .find(|instance| instance.attempt_id == attempt.id)
        .map_or(Lease::NotLeased, |instance| Lease::Leased {
            clock_epoch: &instance.started.epoch,
            expires_monotonic_ms: instance.lease_expires_monotonic_ms,
        });
    let verification = match verification {
        Some(row) => Verification::Recorded {
            verdict: Verdict::parse(&row.verdict).ok_or("verdict")?,
            cleanup_settled: row.cleanup_settled,
        },
        None => Verification::None,
    };
    Ok(AttemptFacts {
        id: &attempt.id,
        generation: checked(attempt.generation.parse::<Generation>())?.value(),
        state: AttemptState::parse(&attempt.state).ok_or("attempt state")?,
        effect: Effect::parse(&attempt.effect).ok_or("effect")?,
        cleanup: Cleanup::parse(&attempt.cleanup).ok_or("cleanup")?,
        acknowledgement: Acknowledgement::Unrecorded,
        lease,
        verification,
        evidence: Evidence::Unassessed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture's own observation text as the ledger retained it
    /// (`pid-reuse-continuation/targeted001/.../observation.json`).
    const RETAINED: &str = r#"{"namespace":"pid:[4026534249]","pid":2,"start_ticks":65353749}"#;
    /// One real `/proc/<pid>/stat` line recorded on 2026-09-21 from a bash
    /// script named `fix) ture`: the comm holds a space and a parenthesis, so
    /// a whitespace split reads field 22 as `0` while the kernel's field 22
    /// (`awk '{print $23}'` on this line, shifted by the space) is 65524797.
    const STAT_SAMPLE: &str = "4127178 (fix) ture) S 4127176 4127178 4127176 0 -1 4194560 213 0 0 0 0 0 0 0 20 0 1 0 65524797 237809664 941 18446744073709551615 94583113252864 94583114338841 140733747969664 0 0 0 65536 4 65538 1 0 0 17 7 0 0 0 0 0 94583114729520 94583114778872 94583760994304 140733747973483 140733747973589 140733747973589 140733747978136 0
";

    fn observed() -> ProcessIdentity {
        ProcessIdentity {
            pid: 4242,
            start_ticks: 65_353_749,
            namespace: "pid:[4026534249]".into(),
        }
    }
    fn present(start_ticks: u64, namespace: &str) -> LiveRead {
        LiveRead::Present(LiveIdentity {
            start_ticks,
            namespace: namespace.into(),
            state: 'S',
        })
    }

    #[test]
    fn same_start_and_namespace_is_live_same_identity() {
        let custody = classify(&observed(), &present(65_353_749, "pid:[4026534249]"));
        assert_eq!(custody, Custody::LiveSameIdentity);
        assert_eq!(custody.name(), "live_same_identity");
        assert_eq!(custody.differs(), &[]);
        assert_eq!(custody.error(), None);
    }
    #[test]
    fn later_start_is_pid_reused_on_start_ticks() {
        let custody = classify(&observed(), &present(65_353_771, "pid:[4026534249]"));
        assert_eq!(
            custody,
            Custody::PidReused {
                differs: vec![Dimension::StartTicks]
            }
        );
        assert_eq!(custody.name(), "pid_reused");
    }
    #[test]
    fn earlier_start_is_also_pid_reused() {
        let custody = classify(&observed(), &present(65_353_748, "pid:[4026534249]"));
        assert_eq!(custody.differs(), &[Dimension::StartTicks]);
    }
    #[test]
    fn different_namespace_is_pid_reused_on_namespace() {
        let custody = classify(&observed(), &present(65_353_749, "pid:[4026531836]"));
        assert_eq!(
            custody,
            Custody::PidReused {
                differs: vec![Dimension::Namespace]
            }
        );
    }
    #[test]
    fn both_dimensions_differ_are_both_named_in_order() {
        let custody = classify(&observed(), &present(65_353_771, "pid:[4026531836]"));
        assert_eq!(
            custody.differs(),
            &[Dimension::StartTicks, Dimension::Namespace]
        );
        assert_eq!(custody.name(), "pid_reused");
    }
    #[test]
    fn no_such_pid_is_absent() {
        let custody = classify(&observed(), &LiveRead::Absent);
        assert_eq!(custody, Custody::Absent);
        assert_eq!(custody.name(), "absent");
    }
    #[test]
    fn refused_read_is_unreadable_and_keeps_the_text() {
        let text = "stat: Permission denied (os error 13)";
        let custody = classify(&observed(), &LiveRead::Unreadable(text.into()));
        assert_eq!(custody, Custody::Unreadable { error: text.into() });
        assert_eq!(custody.name(), "unreadable");
        assert_eq!(custody.error(), Some(text));
    }
    #[test]
    fn summary_is_unreconciled_only_with_nothing_classified() {
        assert_eq!(summary(&[]), "unreconciled");
        let reused = Custody::PidReused {
            differs: vec![Dimension::StartTicks],
        };
        assert_eq!(summary(std::slice::from_ref(&reused)), "pid_reused");
        assert_eq!(
            summary(&[Custody::LiveSameIdentity, Custody::LiveSameIdentity]),
            "live_same_identity"
        );
        assert_eq!(summary(&[Custody::Absent, reused]), "mixed");
        assert_eq!(
            summary(&[Custody::Unreadable {
                error: "ns: x".into()
            }]),
            "unreadable"
        );
    }
    #[test]
    fn retained_fixture_identity_parses_and_other_identities_do_not() {
        assert_eq!(
            ProcessIdentity::parse(RETAINED),
            Some(ProcessIdentity {
                pid: 2,
                start_ticks: 65_353_749,
                namespace: "pid:[4026534249]".into()
            })
        );
        for text in [
            "literal/fixture",
            "native-executor:sha256:aaaa",
            r#"{"namespace":"pid:[4026534249]","pid":0,"start_ticks":65353749}"#,
            r#"{"namespace":"mnt:[4026534249]","pid":2,"start_ticks":65353749}"#,
            r#"{"namespace":"pid:[]","pid":2,"start_ticks":65353749}"#,
            r#"{"namespace":"pid:[4026534249]","pid":2,"start_ticks":65353749,"extra":1}"#,
            r#"{"namespace":"pid:[4026534249]","pid":2}"#,
            r#"{"namespace":"pid:[4026534249]","pid":-2,"start_ticks":1}"#,
        ] {
            assert_eq!(ProcessIdentity::parse(text), None, "{text}");
        }
        let long = format!(
            r#"{{"namespace":"pid:[4026534249]","pid":2,"start_ticks":65353749,"x":"{}"}}"#,
            "y".repeat(300)
        );
        assert_eq!(ProcessIdentity::parse(&long), None);
    }
    #[test]
    fn stat_fields_are_counted_after_the_last_parenthesis() {
        assert_eq!(parse_stat(STAT_SAMPLE), Some(('S', 65_524_797)));
        let naive: u64 = STAT_SAMPLE
            .split_whitespace()
            .nth(21)
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(naive, 0, "the whitespace reading is the wrong one");
        assert_eq!(parse_stat("4127178 (fix) ture) SS 1 2"), None);
        assert_eq!(parse_stat("no parenthesis"), None);
        assert_eq!(parse_stat("1 (a) S 1 2 3"), None, "too few fields");
    }
    #[test]
    fn live_read_of_this_process_and_of_an_impossible_pid() {
        let deadline = Instant::now() + Duration::from_secs(5);
        let LiveRead::Present(me) = live_read(std::process::id(), deadline) else {
            unreachable!("own process is present")
        };
        let namespace = fs::read_link("/proc/self/ns/pid").unwrap();
        assert_eq!(me.namespace, namespace.to_str().unwrap());
        // `/proc/<pid>/stat` describes the thread-group leader, which the test
        // harness parks while this thread reads; only a live state is asserted.
        assert!(!matches!(me.state, 'Z' | 'X'), "state {}", me.state);
        assert!(me.start_ticks > 0);
        assert_eq!(live_read(u32::MAX, deadline), LiveRead::Absent);
        assert_eq!(
            live_read(std::process::id(), Instant::now()),
            LiveRead::Unreadable("inspection deadline".into())
        );
    }
}
