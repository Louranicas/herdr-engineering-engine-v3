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
//! * **A recorded request answers after its deadline (RC03 §6).** An expired envelope reaches this
//!   owner only through [`Tasks::replay`], which reads the stored result of an exact replay (same
//!   principal, key and bytes) and writes nothing; anything else is refused `deadline_exceeded` by
//!   the catalogue. The first reply and every replay are rendered by one function per action.

use crate::actions::control::{Recorded, TaskRequest, Tasks};
use crate::app::evidence::fresh_id;
use crate::contracts::control::{
    ErrorCode, Fault, Outcome, Precondition, ResultEffect, Retry, request_sha256,
};
use crate::contracts::{Sha256Digest, UuidV4};
use crate::store::{
    Admission, Allocation, CancelIntent, Cancellation, Error as StoreError, Principal, Store,
    Submission, TaskHead,
};
use crate::task::control::{Cancel, Selector, Spec};
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
}

impl StoreTasks {
    /// Compose the ledger `store`, opened for `epoch`.
    #[must_use]
    pub fn new(store: Store, epoch: String) -> Self {
        Self {
            store: Mutex::new(store),
            epoch,
        }
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
        let view = store
            .task_view(
                principal,
                UuidV4::parse(&id).map_err(|_| internal())?,
                until,
            )
            .map_err(|error| store_fault(&error))?;
        drop(store);
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
        let delivery = delivery_of(&head.state, deliveries);
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
                "evidence": [],
                "cursor": self.cursor(view.event_high_water, &head.id, now_unix_ms),
            }),
        })
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
/// a stop: `store.rs`'s acceptance and `store/terminal.rs`), so before one nothing is owed; after
/// one it is pending until its outbox row is delivered.
#[must_use]
pub fn delivery_of(state: &str, pending: usize) -> &'static str {
    match state {
        "accepted" | "failed" | "cancelled" if pending > 0 => "pending",
        "accepted" | "failed" | "cancelled" => "delivered",
        _ => "none",
    }
}

#[cfg(test)]
#[path = "../../tests/t28_uncertain.rs"]
mod uncertain;
