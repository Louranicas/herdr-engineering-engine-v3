//! The task module's control bodies: `task.submit`'s `TaskSpecV1`, `task.get`'s request and
//! `task.cancel`'s reason, read under the RC01 offline, zero-external-spend profile (RC03 §4; RC01
//! numeric policy).
//!
//! RC03 §1: "The receiving module owns its action body." These are the rules only the task module
//! can decide; the envelope, the grant and the catalogue door have already run. Every refusal names
//! its member and the rule, in static text.
//!
//! The RC01 overlay, stated once here and nowhere else:
//! * one admitted class, `rust-library-change/1` ("The first release supports
//!   `rust-library-change/1`"); free text cannot create a class;
//! * `privacy` must be `local_only` and `currency_microunits` `"0"`: the remote and paid profiles
//!   are reserved in the vocabulary and `unavailable` under RC01;
//! * no time limit is created here: the caller's `wall_ms` budget passes through, its verification
//!   share is [`crate::task::CLEANUP_RESERVE`] (RC01's policy, owned by the task loop guard, never
//!   more than the budget itself), and the ledger's own allocation rule decides what it admits;
//! * `tokens` must be `"0"`: the ledger has no token column yet (review D-C1), and an allocation the
//!   engine cannot enforce is not admitted;
//! * `parent` must be `null`: child allocations belong to cohort composition, not composed here.

use crate::contracts::control::{ErrorCode, Fault, Retry};
use crate::contracts::{UuidV4, parse_u64_decimal};
use serde_json::{Map, Value};

/// The task classes the RC01 profile admits.
pub const ADMITTED_CLASSES: [&str; 1] = ["rust-library-change/1"];
const MAX_INTENT_BYTES: usize = 8192;
const MAX_CRITERION_BYTES: usize = 1024;
const MAX_CRITERIA: usize = 64;

/// The admission a valid `TaskSpecV1` asks for, in the ledger's terms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Spec {
    /// The admitted class.
    pub task_class: &'static str,
    /// The acceptance criteria, in order.
    pub criteria: Vec<String>,
    /// The whole wall-time allocation.
    pub limit_ms: u64,
    /// The share before the verification reserve.
    pub work_ms: u64,
    /// The verification and cleanup reserve.
    pub verify_ms: u64,
}

fn text<'a>(
    members: &'a Map<String, Value>,
    name: &str,
    field: &'static str,
    most: usize,
) -> Result<&'a str, Fault> {
    members
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| (1..=most).contains(&value.len()))
        .ok_or(Fault::invalid(field, "UTF-8 within its byte bound"))
}

fn object<'a>(
    value: Option<&'a Value>,
    field: &'static str,
) -> Result<&'a Map<String, Value>, Fault> {
    match value {
        Some(Value::Object(members)) => Ok(members),
        _ => Err(Fault::invalid(field, "object")),
    }
}

fn exactly(members: &Map<String, Value>, names: &[&str], field: &'static str) -> Result<(), Fault> {
    if members.len() == names.len() && names.iter().all(|name| members.contains_key(*name)) {
        Ok(())
    } else {
        Err(Fault::invalid(field, "exactly its declared members"))
    }
}

fn unavailable(field: &'static str, constraint: &'static str) -> Fault {
    Fault::of(
        ErrorCode::Unavailable,
        Retry::AfterCondition,
        "the RC01 offline, zero-external-spend profile does not admit this",
    )
    .at(field)
    .because(constraint)
}

/// Read `task.submit`'s body.
///
/// # Errors
///
/// `invalid_argument` naming the member for a malformed spec; `unavailable` for a profile RC01
/// reserves but does not admit (remote privacy, currency, tokens, a parent allocation).
pub fn submission(body: &Map<String, Value>) -> Result<Spec, Fault> {
    exactly(body, &["spec"], "/body")?;
    let spec = object(body.get("spec"), "/body/spec")?;
    exactly(
        spec,
        &[
            "task_class",
            "intent",
            "criteria",
            "privacy",
            "workspace_id",
            "budget",
            "parent",
        ],
        "/body/spec",
    )?;
    let class = spec
        .get("task_class")
        .and_then(Value::as_str)
        .filter(|class| (1..=64).contains(&class.len()) && class.is_ascii())
        .ok_or(Fault::invalid(
            "/body/spec/task_class",
            "ASCII of 1..64 bytes",
        ))?;
    let task_class = ADMITTED_CLASSES
        .into_iter()
        .find(|admitted| *admitted == class)
        .ok_or(Fault::invalid(
            "/body/spec/task_class",
            "a class in the admitted catalogue",
        ))?;
    text(spec, "intent", "/body/spec/intent", MAX_INTENT_BYTES)?;
    let Some(Value::Array(rows)) = spec.get("criteria") else {
        return Err(Fault::invalid(
            "/body/spec/criteria",
            "array of 1..64 criteria",
        ));
    };
    if !(1..=MAX_CRITERIA).contains(&rows.len()) {
        return Err(Fault::invalid(
            "/body/spec/criteria",
            "array of 1..64 criteria",
        ));
    }
    let criteria = rows
        .iter()
        .map(|row| {
            row.as_str()
                .filter(|criterion| (1..=MAX_CRITERION_BYTES).contains(&criterion.len()))
                .map(str::to_owned)
                .ok_or(Fault::invalid(
                    "/body/spec/criteria",
                    "each criterion UTF-8 of 1..1024 bytes",
                ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    match spec.get("privacy").and_then(Value::as_str) {
        Some("local_only") => {}
        Some("remote_allowed") => {
            return Err(unavailable("/body/spec/privacy", "local_only under RC01"));
        }
        _ => {
            return Err(Fault::invalid(
                "/body/spec/privacy",
                "local_only or remote_allowed",
            ));
        }
    }
    spec.get("workspace_id")
        .and_then(Value::as_str)
        .filter(|id| UuidV4::parse(id).is_ok())
        .ok_or(Fault::invalid("/body/spec/workspace_id", "UuidV4"))?;
    let (limit_ms, work_ms, verify_ms) = budget(object(spec.get("budget"), "/body/spec/budget")?)?;
    match spec.get("parent") {
        Some(Value::Null) => {}
        Some(Value::Object(_)) => {
            return Err(unavailable(
                "/body/spec/parent",
                "child allocations are not composed behind this receiver",
            ));
        }
        _ => {
            return Err(Fault::invalid(
                "/body/spec/parent",
                "null or a parent allocation",
            ));
        }
    }
    Ok(Spec {
        task_class,
        criteria,
        limit_ms,
        work_ms,
        verify_ms,
    })
}

fn budget(budget: &Map<String, Value>) -> Result<(u64, u64, u64), Fault> {
    exactly(
        budget,
        &["mode", "wall_ms", "tokens", "currency_microunits"],
        "/body/spec/budget",
    )?;
    if !matches!(
        budget.get("mode").and_then(Value::as_str),
        Some("hard" | "conservative")
    ) {
        return Err(Fault::invalid(
            "/body/spec/budget/mode",
            "hard or conservative",
        ));
    }
    let decimal = |name: &str, field: &'static str| {
        budget
            .get(name)
            .and_then(Value::as_str)
            .and_then(|text| parse_u64_decimal(text).ok())
            .ok_or(Fault::invalid(field, "U64Decimal"))
    };
    let wall = decimal("wall_ms", "/body/spec/budget/wall_ms")?;
    let tokens = decimal("tokens", "/body/spec/budget/tokens")?;
    let currency = decimal(
        "currency_microunits",
        "/body/spec/budget/currency_microunits",
    )?;
    if currency != 0 {
        return Err(unavailable(
            "/body/spec/budget/currency_microunits",
            "zero external spend under RC01",
        ));
    }
    if tokens != 0 {
        return Err(unavailable(
            "/body/spec/budget/tokens",
            "token budgets are not yet enforced by the ledger (D-C1)",
        ));
    }
    // The verification share is the task loop's existing reserve, never more than the budget.
    let reserve = u64::try_from(crate::task::CLEANUP_RESERVE.as_millis())
        .unwrap_or(u64::MAX)
        .min(wall);
    Ok((wall, wall - reserve, reserve))
}

/// Why a cancel is asked for: `task.cancel`'s closed reason set (RC03 §6; contract-decisions.md:343).
pub const CANCEL_REASONS: [&str; 5] = [
    "operator_request",
    "superseded",
    "budget",
    "deadline",
    "safety",
];
/// `note` is bounded in UTF-8 bytes; the schema's `maxLength` counts code points, so the byte rule
/// is decided here.
const MAX_NOTE_BYTES: usize = 1024;

/// A valid `task.cancel` body. The task and the generation it expects are the precondition's, not
/// the body's.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cancel {
    /// One of [`CANCEL_REASONS`].
    pub reason: &'static str,
    /// The caller's note, at most 1,024 UTF-8 bytes.
    pub note: Option<String>,
}

/// Read `task.cancel`'s body.
///
/// # Errors
///
/// `invalid_argument` naming the member: `/body` unless the members are exactly `reason` and
/// `note`; `/body/reason` outside [`CANCEL_REASONS`]; `/body/note` unless null or a string of at
/// most 1,024 bytes.
pub fn cancel(body: &Map<String, Value>) -> Result<Cancel, Fault> {
    exactly(body, &["reason", "note"], "/body")?;
    let reason = body
        .get("reason")
        .and_then(Value::as_str)
        .and_then(|reason| CANCEL_REASONS.into_iter().find(|known| *known == reason))
        .ok_or(Fault::invalid(
            "/body/reason",
            "operator_request, superseded, budget, deadline or safety",
        ))?;
    let note = match body.get("note") {
        Some(Value::Null) => None,
        Some(Value::String(note)) if note.len() <= MAX_NOTE_BYTES => Some(note.clone()),
        _ => {
            return Err(Fault::invalid(
                "/body/note",
                "null or UTF-8 within 1,024 bytes",
            ));
        }
    };
    Ok(Cancel { reason, note })
}

/// How `task.get` names its task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Selector {
    /// By identity.
    Task(String),
    /// By the admission's idempotency key: the readback a lost `task.submit` reply needs.
    SubmitKey(String),
}

/// Read `task.get`'s body.
///
/// # Errors
///
/// `invalid_argument` naming the member; `unavailable` for an evidence view this receiver does not
/// compose (`summary`, `refs`).
pub fn get(body: &Map<String, Value>) -> Result<Selector, Fault> {
    exactly(body, &["selector", "evidence"], "/body")?;
    match body.get("evidence").and_then(Value::as_str) {
        Some("none") => {}
        Some("summary" | "refs") => {
            return Err(Fault::of(
                ErrorCode::Unavailable,
                Retry::AfterCondition,
                "evidence views are not composed behind this receiver",
            )
            .at("/body/evidence"));
        }
        _ => return Err(Fault::invalid("/body/evidence", "none, summary or refs")),
    }
    let selector = object(body.get("selector"), "/body/selector")?;
    let uuid = |name: &str, field: &'static str| {
        selector
            .get(name)
            .and_then(Value::as_str)
            .filter(|id| UuidV4::parse(id).is_ok())
            .map(str::to_owned)
            .ok_or(Fault::invalid(field, "UuidV4"))
    };
    if selector.len() == 1 && selector.contains_key("task_id") {
        return Ok(Selector::Task(uuid("task_id", "/body/selector/task_id")?));
    }
    exactly(
        selector,
        &["source_action", "idempotency_key"],
        "/body/selector",
    )?;
    if selector.get("source_action").and_then(Value::as_str) != Some("task.submit") {
        return Err(Fault::invalid(
            "/body/selector/source_action",
            "const task.submit",
        ));
    }
    Ok(Selector::SubmitKey(uuid(
        "idempotency_key",
        "/body/selector/idempotency_key",
    )?))
}
