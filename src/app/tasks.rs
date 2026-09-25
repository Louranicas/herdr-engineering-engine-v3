//! The task owner composed behind the control receiver: `task.submit`, `task.get` and `task.cancel`
//! through the one ledger (review D-C3 step 3; RC03 §6).
//!
//! The ledger already owns the durable half. `Store::submit` binds (principal, `task.submit`, v1,
//! idempotency key) to the digest of the request's exact bytes, atomically with the task row and its
//! event; an exact replay returns the stored admission; a reused key with other bytes is a
//! conflict. This module only translates: the validated spec into a `Submission`, and the ledger's
//! rows into the wire records. It keeps no second record of anything.
//!
//! * **Replay is observed, not assumed.** Whether a submit is a replay is read (`get_by_key`) before
//!   the write, under the store's single-writer lock with connections served one at a time, so the
//!   read and the write see the same ledger.
//! * **A lost commit is `effect_unknown`, never `internal`.** An uncertain commit carries the exact
//!   readback RC03 §6 prescribes: `task.get` by the submit's own idempotency key.
//! * **`task.get` reads one task.** `Store::task_view` returns the head (through the store's
//!   principal-scoped door), that task's attempts and its undelivered obligations from one read
//!   snapshot, scoped by `task_id` (B03): the ledger's size never decides whether a task can be
//!   read. A task holding more attempts than the ledger's own bound is refused
//!   `resource_exhausted`; nothing is silently truncated. The ledger's state vocabularies are the
//!   wire's, value for value.
//! * **`task.cancel` records intent, never settlement (B05).** `Store::cancel_intent` binds the key to
//!   the request digest and records the intent in one transaction, reading the task through the
//!   principal's own door before its generation (an invisible task names none). The result names
//!   the intent's obligation and what the cancel found of the worker; a lost commit reads back by
//!   `task.get` on the precondition's task.
//! * **`task.resolve` records an operator's disposition, never an observation (B08).**
//!   `Store::resolve_intent` names the obligation (an attempt in state `unknown`, or an undelivered
//!   delivery), applies the design's table (retry / abandon / acknowledge / quarantine), and writes
//!   the disposition as a row of its own; an abandonment stops the task only through the one stop
//!   door, once every unknown effect is acknowledged, retaining reservations when usage is unknown.
//! * **A recorded request answers after its deadline (RC03 §6).** An expired envelope reaches this
//!   owner only through [`Tasks::replay`], which reads the record under its key and writes
//!   nothing: the stored result for the exact bytes, `conflict` for other bytes, and for an unseen
//!   key nothing, which the catalogue refuses `deadline_exceeded`. After an uncertain commit it
//!   answers `unavailable`, as every use of that connection does. The first reply and every replay
//!   are rendered by one function per action.

use crate::actions::control::{CURSOR_LIFETIME_MS, Recorded, TaskRequest, Tasks};
use crate::app::evidence::fresh_id;
use crate::app::routing::{self, Routing, Unready};
use crate::contracts::control::{
    ErrorCode, EvidenceRef, EvidenceView, Fault, Outcome, PageCursor, Precondition, ResultEffect,
    Retry, request_sha256,
};
use crate::contracts::parse_u64_decimal;
use crate::contracts::{Sha256Digest, UuidV4};
use crate::store::{
    Admission, Allocation, CancelIntent, Cancellation, Error as StoreError, ObjectReader,
    Principal, Resolution, ResolveIntent, ResolveRefusal, Store, Submission, TaskFilter, TaskHead,
};
use crate::task::control::{Cancel, List, Preview, Resolve, Selector, Spec};
use serde_json::{Value, json};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long an issued engine cursor names a resumable position.
pub const ENGINE_CURSOR_LIFETIME_MS: u64 = 86_400_000;
/// The visibility policy revision cursors carry. File grants are immutable reviewed records: a
/// change of visibility is a new grant with a new scope digest, so the policy itself has one
/// revision until a revision mechanism exists.
pub const VISIBILITY_REVISION: &str = "0";

/// The ledger, held writable for the life of the coordinator.
pub struct StoreTasks {
    store: Mutex<Store>,
    epoch: String,
    routing: Result<Routing, Unready>,
}

impl StoreTasks {
    /// Compose the ledger `store`, opened for `epoch`, with no route configuration: `task.preview`
    /// answers "route configuration not installed" until [`StoreTasks::with_routing`].
    #[must_use]
    pub fn new(store: Store, epoch: String) -> Self {
        Self {
            store: Mutex::new(store),
            epoch,
            routing: Err(Unready::NotInstalled),
        }
    }

    /// The route configuration `task.preview` screens against, composed once (B07), or why there
    /// is none.
    #[must_use]
    pub fn with_routing(self, routing: Result<Routing, Unready>) -> Self {
        Self { routing, ..self }
    }

    fn cursor(&self, sequence: u64, task: &str, now_unix_ms: u64) -> Value {
        // RC03 §4: the server-derived compact {"resource_ids":[...],"topics":[...]}, keys in that
        // order, arrays sorted and duplicate-free, no LF.
        let filter = format!("{{\"resource_ids\":[\"{task}\"],\"topics\":[\"task\"]}}");
        json!({
            "epoch": self.epoch,
            "sequence": sequence.to_string(),
            "filter_sha256": request_sha256(filter.as_bytes()),
            "visibility_revision": VISIBILITY_REVISION,
            "issued_unix_ms": now_unix_ms.to_string(),
            "expires_unix_ms": now_unix_ms.saturating_add(ENGINE_CURSOR_LIFETIME_MS).to_string(),
        })
    }

    /// Where a `task.list` cursor resumes. The key this listing issues is `<ledger epoch>.<admission
    /// sequence>`: one from another ledger epoch (a restore) is `resync_required`, and one malformed,
    /// naming sequence 0 or past its own snapshot -- which this listing never issues -- is
    /// `invalid_argument`. A well-formed key that is not an admission sequence resumes after it; it
    /// cannot widen what the principal sees.
    fn resume_key(&self, cursor: &PageCursor) -> Result<u64, Fault> {
        let issued = || Fault::invalid("/body/page/cursor/after_key", "a key this listing issued");
        let (epoch, sequence) = cursor.after_key.split_once('.').ok_or_else(issued)?;
        UuidV4::parse(epoch).map_err(|_| issued())?;
        let sequence = parse_u64_decimal(sequence).map_err(|_| issued())?;
        if epoch != self.epoch {
            return Err(Fault::resync("/body/page/cursor/after_key"));
        }
        if sequence == 0 || sequence > cursor.snapshot_revision {
            return Err(issued());
        }
        Ok(sequence)
    }

    /// The `task.submit` result for a stored admission: the one rendering of a first reply and of
    /// every replay of it.
    fn admitted(
        &self,
        admission: &Admission,
        idempotency_key: &str,
        now_unix_ms: u64,
        replayed: bool,
    ) -> Result<Outcome, Fault> {
        UuidV4::parse(&admission.task).map_err(|_| internal())?;
        Ok(Outcome {
            effect: ResultEffect::Committed,
            replayed,
            observed_generation: Some(admission.generation.clone()),
            readback: Some(submit_readback(idempotency_key)),
            body: json!({
                "task": {
                    "task_id": admission.task,
                    "generation": admission.generation,
                    "state": "admitted",
                    "current_attempt_id": null,
                    "unresolved_obligations": 0,
                },
                "engine_cursor": self.cursor(admission.sequence, &admission.task, now_unix_ms),
            }),
        })
    }
}

/// The `task.resolve` result for a stored disposition: the one rendering of a first reply and of
/// every replay of it.
fn resolved(record: &Resolution, replayed: bool) -> Result<Outcome, Fault> {
    UuidV4::parse(&record.task).map_err(|_| internal())?;
    UuidV4::parse(&record.disposition).map_err(|_| internal())?;
    let obligations = u32::try_from(record.unresolved_obligations).map_err(|_| internal())?;
    Ok(Outcome {
        effect: ResultEffect::Committed,
        replayed,
        observed_generation: Some(record.generation.clone()),
        readback: Some(cancel_readback(&record.task)),
        body: json!({
            "task": {
                "task_id": record.task,
                "generation": record.generation,
                "state": record.state,
                "current_attempt_id": record.current_attempt,
                "unresolved_obligations": obligations,
            },
            "disposition_id": record.disposition,
            "obligation_state": if record.resolved { "resolved" } else { "pending" },
        }),
    })
}

/// A `task.resolve` refusal on the wire: each names the member that decided it.
fn resolve_fault(error: StoreError, task: &str) -> Fault {
    match error {
        StoreError::UncertainCommit => Fault::effect_unknown(cancel_readback(task)),
        StoreError::Forbidden => Fault::of(
            ErrorCode::Forbidden,
            Retry::Never,
            "task.resolve is the operator's alone",
        ),
        StoreError::StaleGeneration { current } => {
            Fault::stale("/precondition/generation", current)
        }
        StoreError::NotFound => {
            Fault::of(ErrorCode::NotFound, Retry::Never, "no such visible task")
                .at("/precondition/id")
        }
        StoreError::AlreadyStopped => Fault::of(
            ErrorCode::Conflict,
            Retry::Never,
            "the task already stopped; its outcome stays historical",
        )
        .at("/precondition"),
        StoreError::Disposition(refusal) => match refusal {
            ResolveRefusal::NoObligation => Fault::of(
                ErrorCode::NotFound,
                Retry::Never,
                "the task holds no open obligation by that id",
            )
            .at("/body/obligation_id"),
            ResolveRefusal::LiveAttempt => Fault::of(
                ErrorCode::Conflict,
                Retry::Never,
                "a live attempt is not an operator's to resolve",
            )
            .at("/body/obligation_id")
            .because("use task.cancel"),
            ResolveRefusal::Resolved => Fault::of(
                ErrorCode::Conflict,
                Retry::Never,
                "a disposition already closed this obligation",
            )
            .at("/body/obligation_id"),
            ResolveRefusal::Refused(why) => Fault::of(
                ErrorCode::Conflict,
                Retry::Never,
                "the disposition is refused",
            )
            .at("/body/disposition")
            .because(why),
            ResolveRefusal::Inapplicable(why) => Fault::invalid("/body/disposition", why),
            ResolveRefusal::NoEvidence => Fault::invalid(
                "/body/evidence",
                "an abandonment names at least one artifact the ledger holds",
            ),
            ResolveRefusal::UnknownEvidence => Fault::of(
                ErrorCode::NotFound,
                Retry::Never,
                "an evidence reference names no artifact the ledger holds",
            )
            .at("/body/evidence"),
            // Objects are never deleted, so no condition clears it (review N5).
            ResolveRefusal::Inventory => Fault::of(
                ErrorCode::ResourceExhausted,
                Retry::Never,
                "the ledger's object inventory would exceed the 4096 objects a backup copies",
            )
            .at("/body/evidence"),
        },
        other => store_fault(&other),
    }
}

/// The `task.cancel` result for a stored cancellation: the one rendering of a first reply and of
/// every replay of it.
fn cancelled(record: &Cancellation, replayed: bool) -> Result<Outcome, Fault> {
    UuidV4::parse(&record.task).map_err(|_| internal())?;
    UuidV4::parse(&record.obligation).map_err(|_| internal())?;
    let obligations = u32::try_from(record.unresolved_obligations).map_err(|_| internal())?;
    Ok(Outcome {
        effect: ResultEffect::Committed,
        replayed,
        observed_generation: Some(record.generation.clone()),
        readback: Some(cancel_readback(&record.task)),
        body: json!({
            "task": {
                "task_id": record.task,
                "generation": record.generation,
                "state": record.state,
                "current_attempt_id": record.current_attempt,
                "unresolved_obligations": obligations,
            },
            "cancellation_obligation_id": record.obligation,
            "worker_settlement": record.worker_settlement,
        }),
    })
}

/// The receiver's remaining deadline as a monotonic instant.
fn deadline(deadline_unix_ms: u64, now_unix_ms: u64) -> Instant {
    Instant::now() + Duration::from_millis(deadline_unix_ms.saturating_sub(now_unix_ms))
}

/// The readback that finds an admission from values the caller held before sending.
#[must_use]
pub fn submit_readback(idempotency_key: &str) -> Value {
    json!({
        "action": "task.get",
        "action_version": 1,
        "body": {
            "selector": {"source_action": "task.submit", "idempotency_key": idempotency_key},
            "evidence": "none",
        },
    })
}

/// The readback RC03 §6 prescribes for a cancel: `task.get` by the task the precondition named.
#[must_use]
pub fn cancel_readback(task_id: &str) -> Value {
    json!({
        "action": "task.get",
        "action_version": 1,
        "body": {"selector": {"task_id": task_id}, "evidence": "none"},
    })
}

/// The most bytes a view's distinct objects may sum to before any is read (B09 R1.4): 64 objects of
/// the ledger's 16 MiB limit would be 1 GiB hashed per read.
pub const MAX_VIEW_BYTES: u64 = 64 * 1024 * 1024;

/// The narrower route a bounded view names when it overflows: the same task's `summary`, bounded by
/// construction (contract-decisions.md: "Overflow returns `resource_exhausted` with a narrower
/// query/readback route").
#[must_use]
pub fn summary_readback(task_id: &str) -> Value {
    json!({
        "action": "task.get",
        "action_version": 1,
        "body": {"selector": {"task_id": task_id}, "evidence": "summary"},
    })
}

/// A view's refusal past its bound: `resource_exhausted`, the summary as its readback.
fn view_exhausted(task_id: &str, message: &'static str) -> Fault {
    let mut fault =
        Fault::of(ErrorCode::ResourceExhausted, Retry::AfterReadback, message).at("/body/evidence");
    fault.readback = Some(summary_readback(task_id));
    fault
}

/// The ledger's refusals of an evidence view, and every other through [`store_fault`].
fn evidence_fault(error: &StoreError, task_id: &str) -> Fault {
    match error {
        StoreError::EvidenceIdentity => Fault::of(
            ErrorCode::Unavailable,
            Retry::Never,
            "this task's evidence was recorded without the identity a reference needs",
        )
        .at("/body/evidence")
        .because("evidence identity not recorded for this task"),
        StoreError::EvidenceBound { .. } => view_exhausted(
            task_id,
            "the task holds more evidence references than a view may carry (64)",
        ),
        other => store_fault(other),
    }
}

/// The refusal for evidence that is gone or corrupt now (T17): the history is kept.
fn missing_now() -> Fault {
    Fault::of(
        ErrorCode::Unavailable,
        Retry::AfterCondition,
        "an evidence object is missing or corrupt now; its history is kept",
    )
    .at("/body/evidence")
    .because("evidence object missing or corrupt")
}

/// T17 at readback: every object a view names is present and whole now, or the view is refused
/// `unavailable` (the history is untouched and `evidence: none` still answers). Bounded before any
/// read by [`MAX_VIEW_BYTES`] over the distinct objects.
fn available_now(
    reader: &ObjectReader,
    references: &[EvidenceRef],
    task_id: &str,
    until: Instant,
) -> Result<(), Fault> {
    let mut objects: Vec<&EvidenceRef> = Vec::new();
    for reference in references {
        if !objects.iter().any(|seen| seen.sha256 == reference.sha256) {
            objects.push(reference);
        }
    }
    let bytes = objects.iter().try_fold(0_u64, |sum, reference| {
        sum.checked_add(reference.byte_length)
    });
    if bytes.is_none_or(|bytes| bytes > MAX_VIEW_BYTES) {
        return Err(view_exhausted(
            task_id,
            "the view's objects exceed the bytes one read may check (64 MiB)",
        ));
    }
    for reference in objects {
        match reader.verify(reference, until) {
            Ok(()) => {}
            Err(StoreError::Deadline) => return Err(store_fault(&StoreError::Deadline)),
            // Absent or other bytes: the evidence is gone or corrupt now (T17).
            Err(
                StoreError::NotFound
                | StoreError::Corrupt
                | StoreError::Invalid
                | StoreError::Bound
                | StoreError::Os(rustix::io::Errno::NOENT),
            ) => {
                return Err(missing_now());
            }
            Err(StoreError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(missing_now());
            }
            // Anything else is not a fact about the evidence: the object could not be read now
            // (review N6), and a later read may succeed.
            Err(_) => {
                return Err(Fault::of(
                    ErrorCode::Unavailable,
                    Retry::AfterCondition,
                    "an evidence object could not be read now",
                )
                .at("/body/evidence")
                .because("evidence object unreadable"));
            }
        }
    }
    Ok(())
}

fn unavailable(message: &'static str) -> Fault {
    Fault::of(ErrorCode::Unavailable, Retry::AfterCondition, message)
}

fn internal() -> Fault {
    Fault::of(
        ErrorCode::Internal,
        Retry::Never,
        "the ledger returned a value outside its own contract",
    )
}

fn store_fault(error: &StoreError) -> Fault {
    match error {
        StoreError::NotFound => {
            Fault::of(ErrorCode::NotFound, Retry::Never, "no such visible task")
                .at("/body/selector")
        }
        StoreError::Conflict => Fault::of(
            ErrorCode::Conflict,
            Retry::Never,
            "this idempotency key already admitted a different request",
        )
        .at("/idempotency_key"),
        StoreError::Deadline => Fault::of(
            ErrorCode::DeadlineExceeded,
            Retry::Never,
            "the request deadline passed inside the ledger",
        ),
        StoreError::Bound => Fault::of(
            ErrorCode::ResourceExhausted,
            Retry::Never,
            "the ledger read exceeds its bound; use a narrower readback",
        ),
        StoreError::TaskViewBound { .. } => Fault::of(
            ErrorCode::ResourceExhausted,
            Retry::Never,
            "the task holds more attempts than the ledger's attempt bound",
        ),
        StoreError::Locked | StoreError::InspectionOnly | StoreError::RecoveryRequired => {
            unavailable("the ledger is not open for this operation")
        }
        // After an uncertain commit the store refuses every later use of that connection; the
        // readback `effect_unknown` prescribed waits on the ledger being reopened. Never
        // `internal`, whose `retry: never` would forbid that read.
        StoreError::UncertainCommit => {
            unavailable("the ledger's last commit is uncertain; it must be reopened")
        }
        _ => internal(),
    }
}

fn head_record(
    head: &TaskHead,
    current_attempt: Option<&str>,
    obligations: u32,
) -> Result<Value, Fault> {
    UuidV4::parse(&head.id).map_err(|_| internal())?;
    Ok(json!({
        "task_id": head.id,
        "generation": head.generation,
        "state": head.state,
        "current_attempt_id": current_attempt,
        "unresolved_obligations": obligations,
    }))
}

impl Tasks for StoreTasks {
    fn submit(&self, request: &TaskRequest<'_>, spec: &Spec) -> Result<Outcome, Fault> {
        let until = deadline(request.deadline_unix_ms, request.now_unix_ms);
        let key = UuidV4::parse(request.idempotency_key)
            .map_err(|_| Fault::invalid("/idempotency_key", "UuidV4"))?;
        let criteria_text = serde_json::to_string(&spec.criteria).map_err(|_| internal())?;
        let criteria_digest = request_sha256(criteria_text.as_bytes());
        let criteria = Sha256Digest::parse(&criteria_digest).map_err(|_| internal())?;
        let task_id = fresh_id(until).map_err(|_| unavailable("no entropy for a task identity"))?;
        let event_id =
            fresh_id(until).map_err(|_| unavailable("no entropy for an event identity"))?;
        let task = UuidV4::parse(task_id.as_str()).map_err(|_| internal())?;
        let event = UuidV4::parse(event_id.as_str()).map_err(|_| internal())?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| unavailable("the ledger's owner panicked"))?;
        let replayed = match store.get_by_key(request.principal, key, until) {
            Ok(_) => true,
            Err(StoreError::NotFound) => false,
            Err(error) => return Err(store_fault(&error)),
        };
        let admission = store
            .submit(
                Submission {
                    principal: request.principal,
                    key,
                    task,
                    event,
                    request_bytes: request.payload,
                    criteria,
                    allocation: Allocation {
                        limit_ms: spec.limit_ms,
                        work_ms: spec.work_ms,
                        verify_ms: spec.verify_ms,
                    },
                },
                until,
            )
            .map_err(|error| match error {
                StoreError::UncertainCommit => {
                    Fault::effect_unknown(submit_readback(request.idempotency_key))
                }
                // The ledger's own allocation rule, not one made here, decided the budget.
                StoreError::Bound => Fault::invalid(
                    "/body/spec/budget/wall_ms",
                    "within the ledger's task allocation rule",
                ),
                other => store_fault(&other),
            })?;
        drop(store);
        self.admitted(
            &admission,
            request.idempotency_key,
            request.now_unix_ms,
            replayed,
        )
    }

    fn get(
        &self,
        principal: &Principal,
        selector: &Selector,
        evidence: Option<EvidenceView>,
        deadline_unix_ms: u64,
        now_unix_ms: u64,
    ) -> Result<Outcome, Fault> {
        let until = deadline(deadline_unix_ms, now_unix_ms);
        let mut store = self
            .store
            .lock()
            .map_err(|_| unavailable("the ledger's owner panicked"))?;
        // A submit key names a task through the admission record; the view then reads that task.
        let id = match selector {
            Selector::Task(id) => id.clone(),
            Selector::SubmitKey(key) => {
                store
                    .get_by_key(
                        principal,
                        UuidV4::parse(key).map_err(|_| internal())?,
                        until,
                    )
                    .map_err(|error| store_fault(&error))?
                    .id
            }
        };
        let task = UuidV4::parse(&id).map_err(|_| internal())?;
        // The rows under the ledger's lock, in one snapshot; the objects after it, outside (B09).
        let (view, references, reader) = match evidence {
            None => (
                store
                    .task_view(principal, task, until)
                    .map_err(|error| store_fault(&error))?,
                Vec::new(),
                None,
            ),
            Some(named) => {
                let (view, references) = store
                    .task_evidence(principal, task, named, until)
                    .map_err(|error| evidence_fault(&error, &id))?;
                let reader = store.object_reader().map_err(|error| store_fault(&error))?;
                (view, references, Some(reader))
            }
        };
        drop(store);
        if let Some(reader) = reader {
            available_now(&reader, &references, &id, until)?;
        }
        let head = &view.head;
        let attempts: Vec<_> = view.attempts.iter().collect();
        let deliveries = view.pending_deliveries;
        let current = view.current_attempt();
        let obligations = u32::try_from(view.unresolved_obligations()).map_err(|_| internal())?;
        let cleanups: Vec<&str> = attempts
            .iter()
            .map(|attempt| attempt.cleanup.as_str())
            .collect();
        let cleanup = cleanup_of(&cleanups);
        let delivery = delivery_of(&head.state, deliveries, view.given_up_deliveries);
        Ok(Outcome {
            effect: ResultEffect::None,
            replayed: false,
            observed_generation: Some(head.generation.clone()),
            readback: None,
            body: json!({
                "task": head_record(head, current, obligations)?,
                "criteria_sha256": head.criteria,
                "attempts": attempts.iter().map(|attempt| json!({
                    "attempt_id": attempt.id,
                    "generation": attempt.generation,
                    "state": attempt.state,
                    "effect": attempt.effect,
                })).collect::<Vec<_>>(),
                "cleanup": cleanup,
                "delivery": delivery,
                "evidence": references.iter().map(|reference| json!({
                    "artifact_id": reference.artifact_id,
                    "sha256": reference.sha256,
                    "byte_length": reference.byte_length,
                    "media_type": reference.media_type,
                    "schema_id": reference.schema_id,
                })).collect::<Vec<_>>(),
                "cursor": self.cursor(view.event_high_water, &head.id, now_unix_ms),
            }),
        })
    }

    fn list(
        &self,
        principal: &Principal,
        list: &List,
        deadline_unix_ms: u64,
        now_unix_ms: u64,
    ) -> Result<Outcome, Fault> {
        let until = deadline(deadline_unix_ms, now_unix_ms);
        let cursor = list.page.cursor.as_ref();
        let after = cursor
            .map(|cursor| self.resume_key(cursor))
            .transpose()?
            .unwrap_or(0);
        let states: Vec<&str> = list.states.iter().map(|state| state.name()).collect();
        let filter = TaskFilter {
            states: &states,
            task_class: list.task_class.as_deref(),
            parent_task_id: list.parent_task_id.as_deref(),
        };
        let limit = usize::try_from(list.page.limit).map_err(|_| internal())?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| unavailable("the ledger's owner panicked"))?;
        let listing = store
            .task_list(
                principal,
                &filter,
                cursor.map(|cursor| cursor.snapshot_revision),
                after,
                limit,
                until,
            )
            .map_err(|error| match error {
                StoreError::SnapshotAhead { .. } | StoreError::SnapshotMoved { .. } => {
                    Fault::resync("/body/page/cursor/snapshot_revision")
                }
                other => store_fault(&other),
            })?;
        drop(store);
        let mut items = Vec::with_capacity(listing.tasks.len());
        for (_, view) in &listing.tasks {
            let obligations =
                u32::try_from(view.unresolved_obligations()).map_err(|_| internal())?;
            items.push(head_record(
                &view.head,
                view.current_attempt(),
                obligations,
            )?);
        }
        let snapshot = listing.snapshot.to_string();
        let next_cursor = match listing.tasks.last() {
            Some((sequence, _)) if listing.more => json!({
                "snapshot_revision": snapshot,
                "after_key": format!("{}.{sequence}", self.epoch),
                "filter_sha256": list.filter_sha256(),
                "expires_unix_ms": now_unix_ms.saturating_add(CURSOR_LIFETIME_MS).to_string(),
            }),
            _ => Value::Null,
        };
        Ok(Outcome::read(json!({
            "page": {"items": items, "next_cursor": next_cursor, "snapshot_revision": snapshot},
        })))
    }

    fn cancel(
        &self,
        request: &TaskRequest<'_>,
        target: &Precondition,
        body: &Cancel,
    ) -> Result<Outcome, Fault> {
        let until = deadline(request.deadline_unix_ms, request.now_unix_ms);
        let key = UuidV4::parse(request.idempotency_key)
            .map_err(|_| Fault::invalid("/idempotency_key", "UuidV4"))?;
        let task =
            UuidV4::parse(&target.id).map_err(|_| Fault::invalid("/precondition/id", "UuidV4"))?;
        let event_id =
            fresh_id(until).map_err(|_| unavailable("no entropy for an event identity"))?;
        let event = UuidV4::parse(event_id.as_str()).map_err(|_| internal())?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| unavailable("the ledger's owner panicked"))?;
        let (record, replayed) = store
            .cancel_intent(
                CancelIntent {
                    principal: request.principal,
                    key,
                    task,
                    expected: target.generation,
                    event,
                    request_bytes: request.payload,
                    reason: body.reason,
                    note: body.note.as_deref(),
                },
                until,
            )
            .map_err(|error| match error {
                StoreError::UncertainCommit => Fault::effect_unknown(cancel_readback(&target.id)),
                StoreError::StaleGeneration { current } => {
                    Fault::stale("/precondition/generation", current)
                }
                StoreError::NotFound => {
                    Fault::of(ErrorCode::NotFound, Retry::Never, "no such visible task")
                        .at("/precondition/id")
                }
                StoreError::AlreadyStopped => Fault::of(
                    ErrorCode::Conflict,
                    Retry::Never,
                    "the task already stopped; its outcome stays historical",
                )
                .at("/precondition"),
                other => store_fault(&other),
            })?;
        drop(store);
        cancelled(&record, replayed)
    }

    fn resolve(
        &self,
        request: &TaskRequest<'_>,
        target: &Precondition,
        body: &Resolve,
    ) -> Result<Outcome, Fault> {
        let until = deadline(request.deadline_unix_ms, request.now_unix_ms);
        let key = UuidV4::parse(request.idempotency_key)
            .map_err(|_| Fault::invalid("/idempotency_key", "UuidV4"))?;
        let task =
            UuidV4::parse(&target.id).map_err(|_| Fault::invalid("/precondition/id", "UuidV4"))?;
        let obligation = UuidV4::parse(&body.obligation_id)
            .map_err(|_| Fault::invalid("/body/obligation_id", "UuidV4"))?;
        let identities = [(); 3].map(|()| fresh_id(until));
        let [disposition_id, event_id, stop_id] = identities;
        let entropy = |id: Result<_, _>| id.map_err(|_| unavailable("no entropy for an identity"));
        let (disposition_id, event_id, stop_id) = (
            entropy(disposition_id)?,
            entropy(event_id)?,
            entropy(stop_id)?,
        );
        let mut store = self
            .store
            .lock()
            .map_err(|_| unavailable("the ledger's owner panicked"))?;
        let (record, replayed) = store
            .resolve_intent(
                ResolveIntent {
                    principal: request.principal,
                    key,
                    task,
                    expected: target.generation,
                    obligation,
                    disposition: body.disposition,
                    reason: &body.reason,
                    evidence: &body.evidence,
                    disposition_id: UuidV4::parse(disposition_id.as_str())
                        .map_err(|_| internal())?,
                    event: UuidV4::parse(event_id.as_str()).map_err(|_| internal())?,
                    stop_event: UuidV4::parse(stop_id.as_str()).map_err(|_| internal())?,
                    request_bytes: request.payload,
                },
                until,
            )
            .map_err(|error| resolve_fault(error, &target.id))?;
        drop(store);
        resolved(&record, replayed)
    }

    fn preview(
        &self,
        principal: &Principal,
        preview: &Preview,
        deadline_unix_ms: u64,
        now_unix_ms: u64,
    ) -> Result<Outcome, Fault> {
        let composed = self.routing.as_ref().map_err(|why| why.fault())?;
        let until = deadline(deadline_unix_ms, now_unix_ms);
        let snapshot = self
            .store
            .lock()
            .map_err(|_| unavailable("the ledger's owner panicked"))?
            .roster_snapshot(principal, until)
            .map_err(|error| store_fault(&error))?;
        routing::preview(composed, &snapshot, &preview.spec).map(Outcome::read)
    }

    fn replay(&self, request: &TaskRequest<'_>, of: Recorded) -> Result<Option<Outcome>, Fault> {
        let until = deadline(request.deadline_unix_ms, request.now_unix_ms);
        let key = UuidV4::parse(request.idempotency_key)
            .map_err(|_| Fault::invalid("/idempotency_key", "UuidV4"))?;
        let store = self
            .store
            .lock()
            .map_err(|_| unavailable("the ledger's owner panicked"))?;
        match of {
            Recorded::Submit => store
                .replayed_submit(request.principal, key, request.payload, until)
                .map_err(|error| store_fault(&error))?
                .map(|admission| {
                    self.admitted(
                        &admission,
                        request.idempotency_key,
                        request.now_unix_ms,
                        true,
                    )
                })
                .transpose(),
            Recorded::Cancel => store
                .replayed_cancel(request.principal, key, request.payload, until)
                .map_err(|error| store_fault(&error))?
                .map(|record| cancelled(&record, true))
                .transpose(),
            Recorded::Resolve => store
                .replayed_resolve(request.principal, key, request.payload, until)
                .map_err(|error| store_fault(&error))?
                .map(|record| resolved(&record, true))
                .transpose(),
        }
    }
}

/// The task's cleanup, from its attempts' own cleanup states: the least settled one wins.
#[must_use]
pub fn cleanup_of(attempts: &[&str]) -> &'static str {
    if attempts.contains(&"unknown") {
        "unknown"
    } else if attempts.contains(&"pending") {
        "pending"
    } else if attempts.contains(&"settled") {
        "settled"
    } else {
        "none"
    }
}

/// The task's delivery. Delivery obligations are created only by terminal events (acceptance and
/// a stop: `store.rs`'s acceptance and `store/terminal.rs`, including an operator's abandonment),
/// so before one nothing is owed; after one it is pending until its outbox row is delivered, and a
/// delivery an operator gave up (B08) is `unknown` -- never `delivered`, which it was not.
#[must_use]
pub fn delivery_of(state: &str, pending: usize, given_up: usize) -> &'static str {
    match state {
        "accepted" | "failed" | "cancelled" | "abandoned" if pending > 0 => "pending",
        "accepted" | "failed" | "cancelled" | "abandoned" if given_up > 0 => "unknown",
        "accepted" | "failed" | "cancelled" | "abandoned" => "delivered",
        _ => "none",
    }
}

#[cfg(test)]
#[path = "../../tests/t28_uncertain.rs"]
mod uncertain;
