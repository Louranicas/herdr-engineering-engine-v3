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
//! **Served, and not.** `tools.list` is served: the catalogue is in this process. Every other
//! action is refused `unavailable` with a named reason until its owner is composed behind this
//! receiver, so an unbuilt action is a legible refusal rather than a silent success.

use super::{Action, CATALOGUE_REVISION, Caller, Catalogue, MAX_PAGE, PreconditionRule, Refusal};
use crate::contracts::control::{
    self as wire, Envelope, ErrorCode, Fault, FrameFault, Health, Received, Retry,
    read_result_frame,
};
use crate::contracts::{Sha256Digest, parse_u64_decimal};
use crate::store::Principal;
use serde_json::{Map, Value, json};

/// How long a `tools.list` continuation cursor stays valid after it is issued.
pub const CURSOR_LIFETIME_MS: u64 = 300_000;
/// The longest `tools.list` query, in UTF-8 bytes (RC03 §4: `UTF8[0..256]`).
pub const MAX_QUERY_BYTES: usize = 256;
/// The widest `tools.list` page a caller may ask for (`PageInV1.limit`).
pub const MAX_PAGE_LIMIT: u64 = 100;
const MAX_AFTER_KEY_BYTES: usize = 256;

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

/// What the coordinator has composed behind the receiver: the grant store, and the health it
/// observed at start when it has one. An action whose state is absent is refused `unavailable`.
#[derive(Clone, Copy)]
pub struct Composed<'a> {
    /// The grant store.
    pub grants: &'a dyn Grants,
    /// The coordinator's health observation, when composed.
    pub health: Option<&'a Health>,
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
    };
    let outcome = admit(&envelope, now_unix_ms, principal, grants).and_then(|(action, caller)| {
        dispatch(
            action,
            &caller,
            &envelope.body,
            now_unix_ms,
            composed.health,
        )
    });
    Reply::Frame(match outcome {
        Ok(body) => read_result_frame(&envelope.request_id, &envelope.request_sha256, &body),
        Err(fault) => fault.frame(&envelope.request_id, &envelope.request_sha256),
    })
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
            Refusal::UnknownAction | Refusal::NotVisible => unknown_action(),
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

fn dispatch(
    action: Action,
    caller: &Caller,
    body: &Map<String, Value>,
    now_unix_ms: u64,
    health: Option<&Health>,
) -> Result<Value, Fault> {
    match action.id {
        "tools.list" => tools_list(caller, body, now_unix_ms),
        "tools.inspect" => Err(Fault::of(
            ErrorCode::Unavailable,
            Retry::AfterCondition,
            "per-action schema digests await their publication convention",
        )
        .because("RC03 schema: tuple digests bind published artifact bytes")),
        "health" if !body.is_empty() => Err(Fault::invalid("/body", "an empty object")),
        "health" => health.map(Health::body).ok_or(
            Fault::of(
                ErrorCode::Unavailable,
                Retry::AfterCondition,
                "health needs the coordinator's recovery, database and socket state",
            )
            .because("coordinator state is not composed behind this receiver"),
        ),
        _ => Err(Fault::of(
            ErrorCode::Unavailable,
            Retry::AfterCondition,
            "this action's owner is not composed behind this receiver",
        )
        .because("owner not composed")),
    }
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
    let Some(Value::Object(page)) = body.get("page") else {
        return Err(Fault::invalid("/body/page", "PageInV1"));
    };
    if page.len() != 2 {
        return Err(Fault::invalid("/body/page", "exactly limit and cursor"));
    }
    let limit = page
        .get("limit")
        .and_then(Value::as_u64)
        .filter(|limit| (1..=MAX_PAGE_LIMIT).contains(limit))
        .ok_or(Fault::invalid("/body/page/limit", "integer 1..100"))?;
    let filter = filter_sha256(query);
    let after = match page.get("cursor") {
        Some(Value::Null) => None,
        Some(Value::Object(cursor)) => Some(resume(cursor, &filter, now_unix_ms)?),
        _ => {
            return Err(Fault::invalid("/body/page/cursor", "null or PageCursorV1"));
        }
    };
    // Never wider than the catalogue's own page bound, whatever the caller asked for; a
    // continuation cursor carries the rest, so nothing is silently dropped.
    let width = usize::try_from(limit).map_or(MAX_PAGE, |limit| limit.min(MAX_PAGE));
    let listed =
        Catalogue::search(caller, query, after, width).map_err(|refusal| match refusal {
            Refusal::PageOutOfRange => {
                Fault::invalid("/body/page/cursor/after_key", "a key this listing issued")
            }
            _ => internal(),
        })?;
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

/// Where a continuation cursor resumes, once it is proved to belong to this listing.
fn resume<'a>(
    cursor: &'a Map<String, Value>,
    filter: &str,
    now_unix_ms: u64,
) -> Result<&'a str, Fault> {
    let text = |name: &str| cursor.get(name).and_then(Value::as_str);
    if cursor.len() != 4 {
        return Err(Fault::invalid(
            "/body/page/cursor",
            "exactly snapshot_revision, after_key, filter_sha256 and expires_unix_ms",
        ));
    }
    let snapshot = text("snapshot_revision")
        .and_then(|value| parse_u64_decimal(value).ok())
        .ok_or(Fault::invalid(
            "/body/page/cursor/snapshot_revision",
            "U64Decimal",
        ))?;
    let after_key = text("after_key")
        .filter(|key| (1..=MAX_AFTER_KEY_BYTES).contains(&key.len()) && key.is_ascii())
        .ok_or(Fault::invalid(
            "/body/page/cursor/after_key",
            "ASCII of 1..256 bytes",
        ))?;
    let issued_for = text("filter_sha256")
        .filter(|digest| Sha256Digest::parse(digest).is_ok())
        .ok_or(Fault::invalid("/body/page/cursor/filter_sha256", "Sha256"))?;
    let expires = text("expires_unix_ms")
        .and_then(|value| parse_u64_decimal(value).ok())
        .ok_or(Fault::invalid(
            "/body/page/cursor/expires_unix_ms",
            "U64Decimal",
        ))?;
    if u64::try_from(CATALOGUE_REVISION).ok() != Some(snapshot) {
        return Err(resync("/body/page/cursor/snapshot_revision"));
    }
    if expires <= now_unix_ms {
        return Err(resync("/body/page/cursor/expires_unix_ms"));
    }
    if issued_for != filter {
        return Err(Fault::invalid(
            "/body/page/cursor/filter_sha256",
            "the filter this cursor was issued for",
        ));
    }
    Ok(after_key)
}

/// The digest a `tools.list` cursor binds its filter by: SHA-256 over the server-produced
/// compact JSON `{"query":<query or null>}`, no LF — derived from the validated value, never
/// from caller-retained text (the form RC03 §4 fixes for event filters, applied to this one).
#[must_use]
pub fn filter_sha256(query: Option<&str>) -> String {
    wire::request_sha256(json!({ "query": query }).to_string().as_bytes())
}

fn resync(field: &'static str) -> Fault {
    Fault::of(
        ErrorCode::ResyncRequired,
        Retry::Never,
        "the cursor no longer names this listing; start again without one",
    )
    .at(field)
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
