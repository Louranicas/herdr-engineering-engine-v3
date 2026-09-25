//! The catalogue's half of HEE3-Control/1: from an admitted envelope to one reply.
//!
//! `contracts::control` decides what the wire allows. This module decides what the catalogue
//! requires, in RC03 §5's order: the action and its version, what the action requires of the
//! envelope (an idempotency key for every effect that changes state; the precondition rule the
//! catalogue entry declares), then the grant, then visibility and the effect grant through
//! [`Catalogue::validate`] — the one door that produces a [`super::Dispatch`] — and only then the
//! body, which the owning action materializes.
//!
//! **The principal is a parameter.** RC03 §5: JSON never supplies it; the transport that
//! authenticated the peer does. [`serve`] cannot read one from the request because it has no
//! argument that could carry it there.
//!
//! **Served, and not.** `tools.list` and `tools.inspect` are served: the catalogue and its
//! published schema digests are in this process. Every other
//! action is refused `unavailable` with a named reason until its owner is composed behind this
//! receiver, so an unbuilt action is a legible refusal rather than a silent success.

use super::{
    Action, CATALOGUE_REVISION, Caller, Catalogue, ERROR_SCHEMA_SHA256, MAX_PAGE, PreconditionRule,
    Refusal,
};
use crate::contracts::Principal;
use crate::contracts::control::{
    self as wire, Envelope, ErrorCode, Fault, FrameFault, Health, MAX_DEADLINE_AHEAD_MS,
    MAX_FRAME_BYTES, Outcome, PageCursor, PageIn, Precondition, Received, Retry, Socket,
    result_frame,
};
use crate::task::control::{self as task_body, Selector, Spec};
use serde_json::{Map, Value, json};
use std::sync::atomic::{AtomicBool, Ordering};

/// How long a `tools.list` continuation cursor stays valid after it is issued.
pub const CURSOR_LIFETIME_MS: u64 = 300_000;
/// The longest `tools.list` query, in UTF-8 bytes (RC03 §4: `UTF8[0..256]`).
pub const MAX_QUERY_BYTES: usize = 256;
/// The widest page a caller may ask for (`PageInV1.limit`), the wire's own.
pub use crate::contracts::control::MAX_PAGE_LIMIT;

/// The server-side grant store: what a grant lets an authenticated principal see and do.
pub trait Grants {
    /// The caller `grant_id` makes of `principal` at `now_unix_ms`, or `None` when the grant does
    /// not exist, has expired, belongs to another principal, or was reviewed under a scope other
    /// than `scope_sha256`. A grant is never a bearer token: the principal is part of the key.
    fn resolve(
        &self,
        principal: &Principal,
        grant_id: &str,
        scope_sha256: &str,
        now_unix_ms: u64,
    ) -> Option<Caller>;
}

/// No grant store is configured, so no grant resolves and every request is refused `forbidden`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoGrants;

impl Grants for NoGrants {
    fn resolve(&self, _: &Principal, _: &str, _: &str, _: u64) -> Option<Caller> {
        None
    }
}

/// What the transport does with one received frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reply {
    /// Write nothing and close the connection (RC03 §3).
    Close(FrameFault),
    /// Write this LF-terminated record.
    Frame(Vec<u8>),
}

/// One admitted `task.submit` or `task.cancel`, as the task owner receives it.
#[derive(Clone, Copy, Debug)]
pub struct TaskRequest<'a> {
    /// The transport's principal.
    pub principal: &'a Principal,
    /// The request's idempotency key (the envelope requires one for both actions).
    pub idempotency_key: &'a str,
    /// The request's exact bytes: the durable replay digest names these (RC03 §6).
    pub payload: &'a [u8],
    /// The request deadline, already inside the admitted window. For the readback of an expired
    /// request's record ([`Tasks::replay`]) it is the widest window the wire admits, from receipt.
    pub deadline_unix_ms: u64,
    /// Receiver wall time at receipt.
    pub now_unix_ms: u64,
}

/// The actions whose owner records a result under the idempotency key, so that an exact replay can
/// be answered from the record after the request's deadline (RC03 §6).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Recorded {
    /// `task.submit`.
    Submit,
    /// `task.cancel`.
    Cancel,
}

impl Recorded {
    /// The recorded kind of catalogue action `id`, or `None` for an action that records nothing.
    #[must_use]
    pub fn of(id: &str) -> Option<Self> {
        match id {
            "task.submit" => Some(Self::Submit),
            "task.cancel" => Some(Self::Cancel),
            _ => None,
        }
    }
}

/// The task owner behind the receiver: admission and readback through the ledger.
pub trait Tasks {
    /// Admit `spec` durably, or return the stored result of an exact replay.
    ///
    /// # Errors
    ///
    /// A typed refusal: `conflict` for a reused key with other bytes, `unavailable` when the
    /// ledger cannot be written.
    fn submit(&self, request: &TaskRequest<'_>, spec: &Spec) -> Result<Outcome, Fault>;

    /// Read one visible task.
    ///
    /// # Errors
    ///
    /// `not_found` for a task this principal cannot see; `resource_exhausted` past the read bound.
    fn get(
        &self,
        principal: &Principal,
        selector: &Selector,
        deadline_unix_ms: u64,
        now_unix_ms: u64,
    ) -> Result<Outcome, Fault>;

    /// One page of the principal's tasks (B06), `list`'s cursor already proved current in its
    /// lifetime and filter; its snapshot is the ledger's to judge.
    ///
    /// # Errors
    ///
    /// `resync_required` for a snapshot the ledger never reached; `invalid_argument` for an
    /// `after_key` this listing could not have issued; `resource_exhausted` past a task's read bound;
    /// `unavailable` when the ledger cannot be read.
    fn list(
        &self,
        principal: &Principal,
        list: &task_body::List,
        deadline_unix_ms: u64,
        now_unix_ms: u64,
    ) -> Result<Outcome, Fault>;

    /// Record the intent to cancel `target` (the precondition's task, at its expected generation)
    /// durably, or return the stored result of an exact replay (B05, RC03 §6).
    ///
    /// # Errors
    ///
    /// `stale_generation` naming the current generation; `not_found` for a task this principal
    /// cannot see; `conflict` for a reused key with other bytes or a task that already stopped;
    /// `effect_unknown` after an uncertain commit.
    fn cancel(
        &self,
        request: &TaskRequest<'_>,
        target: &Precondition,
        body: &task_body::Cancel,
    ) -> Result<Outcome, Fault>;

    /// RC03 §6 readback for an expired request: the stored result, with `replayed: true`, when
    /// `request` is an exact replay of a recorded `of` request -- the same principal, key and exact
    /// bytes -- or `None` when its key is unseen. Nothing is written.
    ///
    /// # Errors
    ///
    /// `conflict` for other bytes under a recorded key (its recorded disposition, as inside the
    /// deadline); `unavailable` when the ledger cannot be read or its last commit is uncertain.
    fn replay(&self, request: &TaskRequest<'_>, of: Recorded) -> Result<Option<Outcome>, Fault>;
}

/// What the coordinator has composed behind the receiver: the grant store, the health it observed
/// at start, and the task owner. An action whose owner is absent is refused `unavailable`.
#[derive(Clone, Copy)]
pub struct Composed<'a> {
    /// The grant store.
    pub grants: &'a dyn Grants,
    /// The coordinator's health observation, when composed.
    pub health: Option<&'a Health>,
    /// The task owner, when a writable ledger is composed.
    pub tasks: Option<&'a dyn Tasks>,
    /// Whether the engine is draining (APP-01), read at each `health`: a drain begun while a
    /// connection is open is reported from that frame on, as `socket: draining` (never ready).
    pub draining: Option<&'a AtomicBool>,
}

/// Everything one dispatch may read.
struct Context<'a> {
    envelope: &'a Envelope,
    payload: &'a [u8],
    principal: &'a Principal,
    now_unix_ms: u64,
    composed: Composed<'a>,
}

/// Serve one frame payload (without its LF) for `principal`, at receiver wall time `now_unix_ms`,
/// with only a grant store composed.
#[must_use]
pub fn serve(
    payload: &[u8],
    now_unix_ms: u64,
    principal: &Principal,
    grants: &(impl Grants + Sized),
) -> Reply {
    serve_composed(
        payload,
        now_unix_ms,
        principal,
        Composed {
            grants,
            health: None,
            tasks: None,
            draining: None,
        },
    )
}

/// Serve one frame payload with everything the coordinator composed.
#[must_use]
pub fn serve_composed(
    payload: &[u8],
    now_unix_ms: u64,
    principal: &Principal,
    composed: Composed<'_>,
) -> Reply {
    let grants = composed.grants;
    let envelope = match wire::receive(payload, now_unix_ms) {
        Received::Closed(fault) => return Reply::Close(fault),
        Received::Refused {
            request_id,
            request_sha256,
            fault,
        } => return Reply::Frame(fault.frame(&request_id, &request_sha256)),
        Received::Admitted(envelope) => envelope,
        Received::Expired(envelope) => {
            return Reply::Frame(
                match replay_expired(&envelope, payload, now_unix_ms, principal, composed) {
                    Ok(outcome) => {
                        result_frame(&envelope.request_id, &envelope.request_sha256, &outcome)
                    }
                    Err(fault) => fault.frame(&envelope.request_id, &envelope.request_sha256),
                },
            );
        }
    };
    let context = Context {
        envelope: &envelope,
        payload,
        principal,
        now_unix_ms,
        composed,
    };
    let outcome = admit(&envelope, now_unix_ms, principal, grants)
        .and_then(|(action, caller)| dispatch(action, &caller, &context));
    Reply::Frame(match outcome {
        Ok(outcome) => result_frame(&envelope.request_id, &envelope.request_sha256, &outcome),
        Err(fault) => fault.frame(&envelope.request_id, &envelope.request_sha256),
    })
}

/// An envelope whose deadline passed before receipt (RC03 §6: "For an already-recorded key, return
/// its stored disposition even if the original deadline has since passed"). A recorded key answers
/// from its record: the stored result for the exact bytes, `conflict` for other bytes. Everything
/// else -- an unseen key, an action that records nothing, a request the catalogue or the grant
/// refuses -- is refused [`Fault::expired`] before dispatch, as the wire refused it before. The
/// grant is still resolved: a record is read only for the principal and grant that may read it.
/// The record's read is bounded by the widest window the wire admits.
fn replay_expired(
    envelope: &Envelope,
    payload: &[u8],
    now_unix_ms: u64,
    principal: &Principal,
    composed: Composed<'_>,
) -> Result<Outcome, Fault> {
    let (action, _) =
        admit(envelope, now_unix_ms, principal, composed.grants).map_err(|_| Fault::expired())?;
    let (Some(of), Some(tasks), Some(key)) = (
        Recorded::of(action.id),
        composed.tasks,
        envelope.idempotency_key.as_deref(),
    ) else {
        return Err(Fault::expired());
    };
    tasks
        .replay(
            &TaskRequest {
                principal,
                idempotency_key: key,
                payload,
                deadline_unix_ms: now_unix_ms.saturating_add(MAX_DEADLINE_AHEAD_MS),
                now_unix_ms,
            },
            of,
        )?
        .ok_or_else(Fault::expired)
}

fn admit(
    envelope: &Envelope,
    now_unix_ms: u64,
    principal: &Principal,
    grants: &(impl Grants + ?Sized),
) -> Result<(Action, Caller), Fault> {
    let action = Catalogue::find(&envelope.action).map_err(|_| unknown_action())?;
    if action.wire_version() != u32::try_from(envelope.action_version).ok() {
        return Err(Fault::of(
            ErrorCode::UnsupportedActionVersion,
            Retry::Never,
            "this receiver serves action version 1 only",
        )
        .at("/action_version"));
    }
    if action.effect.mutates() && envelope.idempotency_key.is_none() {
        return Err(Fault::invalid(
            "/idempotency_key",
            "required for an action that changes state",
        ));
    }
    match (action.precondition, &envelope.precondition) {
        (PreconditionRule::Forbidden, Some(_)) => {
            return Err(Fault::invalid("/precondition", "null for this action"));
        }
        (PreconditionRule::Required(_), None) => {
            return Err(Fault::invalid("/precondition", "required for this action"));
        }
        (
            PreconditionRule::Optional(kind) | PreconditionRule::Required(kind),
            Some(precondition),
        ) if precondition.resource != kind => {
            return Err(Fault::invalid(
                "/precondition/resource",
                "the resource kind this action owns",
            ));
        }
        _ => {}
    }
    let caller = grants
        .resolve(
            principal,
            &envelope.grant_id,
            &envelope.scope_sha256,
            now_unix_ms,
        )
        .ok_or(
            Fault::of(
                ErrorCode::Forbidden,
                Retry::AfterCondition,
                "no grant of this principal matches this grant and scope",
            )
            .at("/authority/grant_id"),
        )?;
    let dispatch = Catalogue::validate(&caller, action.id, action.version).map_err(|refusal| {
        match refusal {
            // A hidden action reads as absent: the refusal must not confirm what the caller
            // may not see.
            Refusal::UnknownAction => unknown_action(),
            Refusal::UngrantedEffect => Fault::of(
                ErrorCode::Forbidden,
                Retry::AfterCondition,
                "the grant does not cover this action's effect",
            )
            .at("/action"),
            Refusal::UnknownVersion | Refusal::PageTooWide | Refusal::PageOutOfRange => internal(),
        }
    })?;
    Ok((dispatch.action(), caller))
}

fn dispatch(action: Action, caller: &Caller, context: &Context<'_>) -> Result<Outcome, Fault> {
    let body = &context.envelope.body;
    let owner_absent = || {
        Fault::of(
            ErrorCode::Unavailable,
            Retry::AfterCondition,
            "this action's owner is not composed behind this receiver",
        )
        .because("owner not composed")
    };
    match action.id {
        "tools.list" => tools_list(caller, body, context.now_unix_ms).map(Outcome::read),
        "task.submit" => {
            let spec = task_body::submission(body)?;
            let tasks = context.composed.tasks.ok_or_else(owner_absent)?;
            let key = context
                .envelope
                .idempotency_key
                .as_deref()
                .ok_or_else(internal)?;
            tasks.submit(
                &TaskRequest {
                    principal: context.principal,
                    idempotency_key: key,
                    payload: context.payload,
                    deadline_unix_ms: context.envelope.deadline_unix_ms,
                    now_unix_ms: context.now_unix_ms,
                },
                &spec,
            )
        }
        "task.cancel" => {
            let cancel = task_body::cancel(body)?;
            let tasks = context.composed.tasks.ok_or_else(owner_absent)?;
            let key = context
                .envelope
                .idempotency_key
                .as_deref()
                .ok_or_else(internal)?;
            // `admit` has already required a task precondition for this action.
            let target = context
                .envelope
                .precondition
                .as_ref()
                .ok_or_else(internal)?;
            tasks.cancel(
                &TaskRequest {
                    principal: context.principal,
                    idempotency_key: key,
                    payload: context.payload,
                    deadline_unix_ms: context.envelope.deadline_unix_ms,
                    now_unix_ms: context.now_unix_ms,
                },
                target,
                &cancel,
            )
        }
        "task.list" => {
            // Shape, then owner, then the listing's own semantics (the cursor's lifetime and
            // filter), in the order every task action keeps.
            let list = task_body::list(body)?;
            let tasks = context.composed.tasks.ok_or_else(owner_absent)?;
            if let Some(cursor) = &list.page.cursor {
                cursor.resumes(&list.filter_sha256(), context.now_unix_ms)?;
            }
            tasks.list(
                context.principal,
                &list,
                context.envelope.deadline_unix_ms,
                context.now_unix_ms,
            )
        }
        "task.get" => {
            let selector = task_body::get(body)?;
            let tasks = context.composed.tasks.ok_or_else(owner_absent)?;
            tasks.get(
                context.principal,
                &selector,
                context.envelope.deadline_unix_ms,
                context.now_unix_ms,
            )
        }
        "tools.inspect" => tools_inspect(caller, body).map(Outcome::read),
        "health" => health(context),
        _ => Err(owner_absent()),
    }
}

/// `health`: the coordinator's observation, as composed, with the drain read at each frame (APP-01).
fn health(context: &Context<'_>) -> Result<Outcome, Fault> {
    if !context.envelope.body.is_empty() {
        return Err(Fault::invalid("/body", "an empty object"));
    }
    context
        .composed
        .health
        .map(|health| {
            let draining = context
                .composed
                .draining
                .is_some_and(|flag| flag.load(Ordering::Acquire));
            let health = if draining {
                Health {
                    socket: Socket::Draining,
                    ..*health
                }
            } else {
                *health
            };
            Outcome::read(health.body())
        })
        .ok_or(
            Fault::of(
                ErrorCode::Unavailable,
                Retry::AfterCondition,
                "health needs the coordinator's recovery, database and socket state",
            )
            .because("coordinator state is not composed behind this receiver"),
        )
}

fn tools_list(
    caller: &Caller,
    body: &Map<String, Value>,
    now_unix_ms: u64,
) -> Result<Value, Fault> {
    if body.len() != 2 {
        return Err(Fault::invalid("/body", "exactly query and page"));
    }
    let query = match body.get("query") {
        Some(Value::Null) => None,
        Some(Value::String(query)) if query.len() <= MAX_QUERY_BYTES => Some(query.as_str()),
        _ => {
            return Err(Fault::invalid(
                "/body/query",
                "null or UTF-8 of at most 256 bytes",
            ));
        }
    };
    let PageIn { limit, cursor } = PageIn::parse(body.get("page"))?;
    let filter = filter_sha256(query);
    let after = cursor
        .map(|cursor| resume(cursor, &filter, now_unix_ms))
        .transpose()?;
    // Never wider than the catalogue's own page bound, whatever the caller asked for; a
    // continuation cursor carries the rest, so nothing is silently dropped.
    let width = usize::try_from(limit).map_or(MAX_PAGE, |limit| limit.min(MAX_PAGE));
    let listed =
        Catalogue::search(caller, query, after.as_deref(), width).map_err(
            |refusal| match refusal {
                Refusal::PageOutOfRange => {
                    Fault::invalid("/body/page/cursor/after_key", "a key this listing issued")
                }
                _ => internal(),
            },
        )?;
    let revision = CATALOGUE_REVISION.to_string();
    let mut items = Vec::with_capacity(listed.entries.len());
    for action in &listed.entries {
        let version = action.wire_version().ok_or_else(internal)?;
        items.push(json!({
            "id": action.id,
            "version": version,
            "purpose": action.purpose,
            "effect": action.effect.wire_class(),
        }));
    }
    let next_cursor = match (listed.next, listed.entries.last()) {
        (Some(_), Some(last)) => json!({
            "snapshot_revision": revision,
            "after_key": last.id,
            "filter_sha256": filter,
            "expires_unix_ms": now_unix_ms.saturating_add(CURSOR_LIFETIME_MS).to_string(),
        }),
        _ => Value::Null,
    };
    Ok(json!({
        "catalogue_revision": revision,
        "page": {
            "items": items,
            "next_cursor": next_cursor,
            "snapshot_revision": revision,
        },
    }))
}

/// `tools.inspect`: one visible action's compatibility tuple (RC03 §4), its bounds and its
/// readback route. The digests name the published schema files (decision D-4).
fn tools_inspect(caller: &Caller, body: &Map<String, Value>) -> Result<Value, Fault> {
    if body.len() != 2 {
        return Err(Fault::invalid("/body", "exactly action and version"));
    }
    let id = body
        .get("action")
        .and_then(Value::as_str)
        .ok_or(Fault::invalid("/body/action", "ActionId"))?;
    let version = body
        .get("version")
        .and_then(Value::as_u64)
        .ok_or(Fault::invalid("/body/version", "integer"))?;
    let action = Catalogue::inspect_wire(caller, id, version).map_err(|refusal| match refusal {
        // Hidden and undeclared are one answer: the refusal must not confirm what is unseen.
        Refusal::UnknownAction => unknown_action().at("/body/action"),
        Refusal::UnknownVersion => Fault::of(
            ErrorCode::UnsupportedActionVersion,
            Retry::Never,
            "this receiver serves action version 1 only",
        )
        .at("/body/version"),
        Refusal::UngrantedEffect | Refusal::PageTooWide | Refusal::PageOutOfRange => internal(),
    })?;
    let version = action.wire_version().ok_or_else(internal)?;
    Ok(json!({
        "action": action.id,
        "version": version,
        "purpose": action.purpose,
        "effect": action.effect.wire_class(),
        "request_schema_sha256": action.request_schema_sha256,
        "result_schema_sha256": action.result_schema_sha256,
        "error_schema_sha256": ERROR_SCHEMA_SHA256,
        "max_request_bytes": MAX_FRAME_BYTES,
        "max_deadline_ms": MAX_DEADLINE_AHEAD_MS,
        "readback_action": action.readback_action,
    }))
}

/// Where a continuation cursor resumes, once it is proved to belong to this listing: the
/// catalogue's own revision, then the cursor's lifetime and filter.
fn resume(cursor: PageCursor, filter: &str, now_unix_ms: u64) -> Result<String, Fault> {
    if u64::try_from(CATALOGUE_REVISION).ok() != Some(cursor.snapshot_revision) {
        return Err(Fault::resync("/body/page/cursor/snapshot_revision"));
    }
    cursor.resumes(filter, now_unix_ms)?;
    Ok(cursor.after_key)
}

/// The digest a `tools.list` cursor binds its filter by: SHA-256 over the server-produced
/// compact JSON `{"query":<query or null>}`, no LF — derived from the validated value, never
/// from caller-retained text (the form RC03 §4 fixes for event filters, applied to this one).
#[must_use]
pub fn filter_sha256(query: Option<&str>) -> String {
    wire::request_sha256(json!({ "query": query }).to_string().as_bytes())
}

fn unknown_action() -> Fault {
    Fault::of(
        ErrorCode::UnknownAction,
        Retry::Never,
        "no such action is visible",
    )
    .at("/action")
}

fn internal() -> Fault {
    Fault::of(
        ErrorCode::Internal,
        Retry::Never,
        "the catalogue broke an invariant it states",
    )
}
