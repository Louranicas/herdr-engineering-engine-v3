//! T28 task-owner cases (a module of `t28_actions`): `task.submit` and `task.get` composed behind
//! the control receiver over a real ledger (review D-C3 step 3). Every ledger is a scratch root.
use habitat_engine::actions::control::{
    self, Composed, Grants, Recorded, Reply, TaskRequest, Tasks,
};
use habitat_engine::actions::{Caller, Effect, Owner};
use habitat_engine::app::tasks::{StoreTasks, cleanup_of, delivery_of, submit_readback};
use habitat_engine::contracts::UuidV4;
use habitat_engine::contracts::control::{ErrorCode, request_sha256};
use habitat_engine::store::{Principal, Store};
use habitat_engine::task::control::{Selector, get, submission};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::Write as _;
use std::fs::{self, DirBuilder};
use std::io::Write;
use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

type Outcome = Result<(), Box<dyn Error>>;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const GENERATION: &str = "28d00000-0000-4000-8000-000000000001";
const EPOCH: &str = "28d00000-0000-4000-8000-000000000002";
pub(super) const KEY: &str = "28d00000-0000-4000-8000-0000000000aa";
const NOW: u64 = 1_790_000_000_000;

/// `sha256:` hex of a digest, computed here from `sha2` directly, not through the engine.
pub(super) fn digest(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .fold(String::from("sha256:"), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Result<Self, Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!(
            "hee3-t28t-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path)?;
        Ok(Self(path))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(super) fn spec() -> Value {
    json!({
        "task_class": "rust-library-change/1",
        "intent": "Add a strict decimal parser.",
        "criteria": ["rejects a leading zero", "round-trips the maximum"],
        "privacy": "local_only",
        "workspace_id": "28d00000-0000-4000-8000-0000000000bb",
        "budget": {"mode": "hard", "wall_ms": "600000", "tokens": "0", "currency_microunits": "0"},
        "parent": null,
    })
}

fn body(spec: Value) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("spec".into(), spec);
    map
}

fn with(path: &[&str], value: Value) -> Value {
    let mut spec = spec();
    let mut target = &mut spec;
    for step in &path[..path.len() - 1] {
        target = &mut target[*step];
    }
    target[path[path.len() - 1]] = value;
    spec
}

#[test]
fn a_spec_is_admitted_only_under_the_rc01_profile() -> Outcome {
    let admitted = submission(&body(spec())).map_err(|fault| format!("{fault:?}"))?;
    // Off the origin: 600 000 ms splits into 300 000 of work and the 300 000 reserve.
    assert_eq!(
        (
            admitted.task_class,
            admitted.limit_ms,
            admitted.work_ms,
            admitted.verify_ms
        ),
        ("rust-library-change/1", 600_000, 300_000, 300_000)
    );
    assert_eq!(
        admitted.criteria,
        ["rejects a leading zero", "round-trips the maximum"]
    );
    // No limit is created here: any budget passes through, and the verification share is the task
    // loop's reserve, never more than the budget itself.
    for (wall, work, verify) in [
        (100_000_u64, 0_u64, 100_000_u64),
        (2_000_000, 1_700_000, 300_000),
    ] {
        let spec = submission(&body(with(&["budget", "wall_ms"], json!(wall.to_string()))))
            .map_err(|fault| format!("{fault:?}"))?;
        assert_eq!(
            (spec.limit_ms, spec.work_ms, spec.verify_ms),
            (wall, work, verify),
            "wall_ms {wall}"
        );
    }
    Ok(())
}

#[test]
fn each_refusal_names_its_member_and_code() -> Outcome {
    for (case, spec, code, field) in [
        (
            "unknown class",
            with(&["task_class"], json!("free-text/1")),
            ErrorCode::InvalidArgument,
            "/body/spec/task_class",
        ),
        (
            "remote",
            with(&["privacy"], json!("remote_allowed")),
            ErrorCode::Unavailable,
            "/body/spec/privacy",
        ),
        (
            "currency",
            with(&["budget", "currency_microunits"], json!("1")),
            ErrorCode::Unavailable,
            "/body/spec/budget/currency_microunits",
        ),
        (
            "tokens",
            with(&["budget", "tokens"], json!("5")),
            ErrorCode::Unavailable,
            "/body/spec/budget/tokens",
        ),
        (
            "parent",
            with(
                &["parent"],
                json!({"task_id": KEY, "allocation_id": KEY, "brief_revision": "1"}),
            ),
            ErrorCode::Unavailable,
            "/body/spec/parent",
        ),
        (
            "no criteria",
            with(&["criteria"], json!([])),
            ErrorCode::InvalidArgument,
            "/body/spec/criteria",
        ),
        (
            "65 criteria",
            with(&["criteria"], json!(vec!["c"; 65])),
            ErrorCode::InvalidArgument,
            "/body/spec/criteria",
        ),
        (
            "long criterion",
            with(&["criteria"], json!(["x".repeat(1025)])),
            ErrorCode::InvalidArgument,
            "/body/spec/criteria",
        ),
        (
            "empty intent",
            with(&["intent"], json!("")),
            ErrorCode::InvalidArgument,
            "/body/spec/intent",
        ),
        (
            "bad workspace",
            with(&["workspace_id"], json!("nope")),
            ErrorCode::InvalidArgument,
            "/body/spec/workspace_id",
        ),
        (
            "bad mode",
            with(&["budget", "mode"], json!("soft")),
            ErrorCode::InvalidArgument,
            "/body/spec/budget/mode",
        ),
        (
            "extra member",
            with(&["extra"], json!(1)),
            ErrorCode::InvalidArgument,
            "/body/spec",
        ),
    ] {
        let fault = submission(&body(spec)).err().ok_or(case)?;
        assert_eq!((fault.code, fault.field), (code, Some(field)), "{case}");
    }
    Ok(())
}

#[test]
fn a_get_names_its_task_by_identity_or_by_the_submit_key() -> Outcome {
    let parse = |value: Value| {
        get(value.as_object().ok_or("object")?).map_err(|fault| format!("{:?}", fault.code).into())
    };
    let by_id: Result<Selector, Box<dyn Error>> =
        parse(json!({"selector": {"task_id": KEY}, "evidence": "none"}));
    assert_eq!(by_id?, Selector::Task(KEY.into()));
    let by_key: Result<Selector, Box<dyn Error>> = parse(
        json!({"selector": {"source_action": "task.submit", "idempotency_key": KEY}, "evidence": "none"}),
    );
    assert_eq!(by_key?, Selector::SubmitKey(KEY.into()));
    for (case, value, code) in [
        (
            "summary",
            json!({"selector": {"task_id": KEY}, "evidence": "summary"}),
            "Unavailable",
        ),
        (
            "other source",
            json!({"selector": {"source_action": "task.cancel", "idempotency_key": KEY}, "evidence": "none"}),
            "InvalidArgument",
        ),
        (
            "both selectors",
            json!({"selector": {"task_id": KEY, "source_action": "task.submit", "idempotency_key": KEY}, "evidence": "none"}),
            "InvalidArgument",
        ),
        (
            "bad id",
            json!({"selector": {"task_id": "x"}, "evidence": "none"}),
            "InvalidArgument",
        ),
    ] {
        let refused: Result<Selector, Box<dyn Error>> = parse(value);
        assert_eq!(
            refused.err().map(|error| error.to_string()),
            Some(code.to_owned()),
            "{case}"
        );
    }
    Ok(())
}

/// Sees and may do everything task-owned: these cases are about the ledger, not grants.
struct Open;

impl Grants for Open {
    fn resolve(&self, _: &Principal, _: &str, _: &str, _: u64) -> Option<Caller> {
        let caller = Owner::ALL.into_iter().fold(Caller::new(), Caller::seeing);
        Some(Effect::ALL.into_iter().fold(caller, Caller::granted))
    }
}

fn ledger(scratch: &Scratch) -> Result<StoreTasks, Box<dyn Error>> {
    let root = scratch.0.join("state");
    DirBuilder::new().mode(0o700).create(&root)?;
    let store = Store::open(
        &root,
        UuidV4::parse(GENERATION)?,
        UuidV4::parse(EPOCH)?,
        true,
        Instant::now() + Duration::from_secs(10),
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(StoreTasks::new(store, EPOCH.to_owned()))
}

pub(super) fn request(action: &str, request: u8, key: Option<&str>, body: &Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "protocol": "hee3.control", "version": 1, "kind": "request",
        "request_id": format!("28d00000-0000-4000-8000-0000000001{request:02x}"),
        "action": action, "action_version": 1, "idempotency_key": key,
        "deadline_unix_ms": (NOW + 5_000).to_string(),
        "authority": {"grant_id": KEY, "scope_sha256": format!("sha256:{}", "4".repeat(64))},
        "precondition": null, "body": body,
    }))
    .unwrap_or_default()
}

pub(super) fn serve(
    tasks: &StoreTasks,
    principal: &Principal,
    payload: &[u8],
) -> Result<Value, Box<dyn Error>> {
    serve_at(tasks, principal, payload, NOW)
}

/// Serve `payload` at receiver wall time `now_unix_ms`.
fn serve_at(
    tasks: &StoreTasks,
    principal: &Principal,
    payload: &[u8],
    now_unix_ms: u64,
) -> Result<Value, Box<dyn Error>> {
    serve_composed_at(tasks, &Open, principal, payload, now_unix_ms)
}

/// Serve `payload` at `now_unix_ms` with `tasks` and `grants` composed.
fn serve_composed_at(
    tasks: &dyn Tasks,
    grants: &dyn Grants,
    principal: &Principal,
    payload: &[u8],
    now_unix_ms: u64,
) -> Result<Value, Box<dyn Error>> {
    let composed = Composed {
        grants,
        health: None,
        tasks: Some(tasks),
        draining: None,
    };
    match control::serve_composed(payload, now_unix_ms, principal, composed) {
        Reply::Frame(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Reply::Close(fault) => Err(format!("closed: {}", fault.name()).into()),
    }
}

/// Every reply validated against the published schema by the independent oracle.
fn conforms(replies: &[(&str, &Value)]) -> Outcome {
    let rows: Vec<Value> = replies
        .iter()
        .enumerate()
        .map(|(index, (action, reply))| json!({"name": index.to_string(), "action": action, "reply": reply.to_string()}))
        .collect();
    let mut child = Command::new("python3")
        .args([
            "-W",
            "error",
            "tests/fixtures/native/control-v1/receiver-oracle.py",
            "validate",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env_remove("FORCE_COLOR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("stdin")?
        .write_all(&serde_json::to_vec(&rows)?)?;
    let output = child.wait_with_output()?;
    let checked: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(checked, json!({"checked": rows.len(), "invalid": []}));
    Ok(())
}

#[test]
fn a_submission_commits_once_and_replays_exactly() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let submit = request("task.submit", 1, Some(KEY), &json!({"spec": spec()}));
    let first = serve(&tasks, &operator, &submit)?;
    assert_eq!(
        (
            &first["kind"],
            &first["effect"],
            &first["replayed"],
            &first["observed_generation"]
        ),
        (
            &json!("result"),
            &json!("committed"),
            &json!(false),
            &json!("1")
        ),
        "{first}"
    );
    assert_eq!(first["readback"], submit_readback(KEY));
    let task = first["body"]["task"]["task_id"]
        .as_str()
        .ok_or("task id")?
        .to_owned();
    assert_eq!(
        first["body"]["task"],
        json!({"task_id": task, "generation": "1", "state": "admitted", "current_attempt_id": null, "unresolved_obligations": 0})
    );
    let cursor = &first["body"]["engine_cursor"];
    let filter = format!("{{\"resource_ids\":[\"{task}\"],\"topics\":[\"task\"]}}");
    assert_eq!(
        cursor["filter_sha256"],
        json!(digest(Sha256::digest(filter.as_bytes())))
    );
    assert_eq!(
        cursor["expires_unix_ms"],
        json!((NOW + 86_400_000).to_string())
    );
    // The exact same bytes: the stored admission, not a second task.
    let again = serve(&tasks, &operator, &submit)?;
    assert_eq!(
        (&again["replayed"], &again["body"]["task"]["task_id"]),
        (&json!(true), &json!(task))
    );
    // Other bytes under the same key: a conflict, and nothing new admitted.
    let other = request("task.submit", 2, Some(KEY), &json!({"spec": spec()}));
    let conflict = serve(&tasks, &operator, &other)?;
    assert_eq!(
        (&conflict["code"], &conflict["details"]["field"]),
        (&json!("conflict"), &json!("/idempotency_key"))
    );
    conforms(&[
        ("task.submit", &first),
        ("task.submit", &again),
        ("task.submit", &conflict),
    ])?;
    Ok(())
}

#[test]
fn a_submission_reads_back_by_key_and_identity_to_its_principal_only() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let submit = request("task.submit", 1, Some(KEY), &json!({"spec": spec()}));
    let first = serve(&tasks, &operator, &submit)?;
    assert_eq!(
        (
            &first["kind"],
            &first["effect"],
            &first["replayed"],
            &first["observed_generation"]
        ),
        (
            &json!("result"),
            &json!("committed"),
            &json!(false),
            &json!("1")
        ),
        "{first}"
    );
    assert_eq!(first["readback"], submit_readback(KEY));
    let task = first["body"]["task"]["task_id"]
        .as_str()
        .ok_or("task id")?
        .to_owned();
    // Readback by the key the caller held before sending, and by identity.
    let by_key = serve(
        &tasks,
        &operator,
        &request(
            "task.get",
            3,
            None,
            &json!({"selector": {"source_action": "task.submit", "idempotency_key": KEY}, "evidence": "none"}),
        ),
    )?;
    let by_id = serve(
        &tasks,
        &operator,
        &request(
            "task.get",
            4,
            None,
            &json!({"selector": {"task_id": task}, "evidence": "none"}),
        ),
    )?;
    assert_eq!(by_key["body"]["task"], by_id["body"]["task"]);
    let expected_criteria = digest(Sha256::digest(
        br#"["rejects a leading zero","round-trips the maximum"]"#,
    ));
    assert_eq!(
        (
            &by_key["body"]["task"]["state"],
            &by_key["body"]["attempts"],
            &by_key["body"]["cleanup"],
            &by_key["body"]["delivery"],
            &by_key["body"]["criteria_sha256"]
        ),
        (
            &json!("admitted"),
            &json!([]),
            &json!("none"),
            &json!("none"),
            &json!(expected_criteria)
        )
    );
    assert_eq!(
        (&by_key["effect"], &by_key["observed_generation"]),
        (&json!("none"), &json!("1"))
    );
    // Another principal cannot see it, by key or by identity: not found, not forbidden.
    let stranger = Principal::new(1001, "operator").map_err(|error| format!("{error:?}"))?;
    let hidden = serve(
        &tasks,
        &stranger,
        &request(
            "task.get",
            5,
            None,
            &json!({"selector": {"task_id": task}, "evidence": "none"}),
        ),
    )?;
    assert_eq!(hidden["code"], json!("not_found"));
    conforms(&[
        ("task.get", &by_key),
        ("task.get", &by_id),
        ("task.get", &hidden),
    ])?;
    assert_eq!(
        request_sha256(&submit),
        first["request_sha256"].as_str().unwrap_or_default()
    );
    Ok(())
}

/// task-G09 / APP-08: `task.get` reads one task, not the ledger. With more unrelated tasks than
/// the whole-ledger recovery inventory will read (its 1,024-row bound), one task still reads back.
#[test]
fn a_task_reads_back_in_a_ledger_past_the_inventory_bound() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    for index in 0..1_025_u32 {
        let key = format!("28d10000-0000-4000-8000-{index:012x}");
        let admitted = serve(
            &tasks,
            &operator,
            &request("task.submit", 7, Some(&key), &json!({"spec": spec()})),
        )?;
        assert_eq!(
            admitted["effect"],
            json!("committed"),
            "{index}: {admitted}"
        );
    }
    let target = serve(
        &tasks,
        &operator,
        &request("task.submit", 8, Some(KEY), &json!({"spec": spec()})),
    )?;
    let task = target["body"]["task"]["task_id"].clone();
    let read = serve(
        &tasks,
        &operator,
        &request(
            "task.get",
            9,
            None,
            &json!({"selector": {"task_id": task}, "evidence": "none"}),
        ),
    )?;
    assert_eq!(
        (
            &read["kind"],
            &read["body"]["task"]["task_id"],
            &read["body"]["attempts"]
        ),
        (&json!("result"), &task, &json!([])),
        "{read}"
    );
    Ok(())
}

#[test]
fn without_a_composed_ledger_task_actions_are_unavailable() -> Result<(), Box<dyn Error>> {
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let composed = Composed {
        grants: &Open,
        health: None,
        tasks: None,
        draining: None,
    };
    let Reply::Frame(bytes) = control::serve_composed(
        &request("task.submit", 6, Some(KEY), &json!({"spec": spec()})),
        NOW,
        &operator,
        composed,
    ) else {
        return Err("closed".into());
    };
    let reply: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(
        (&reply["code"], &reply["details"]["constraint"]),
        (&json!("unavailable"), &json!("owner not composed"))
    );
    // B05: a schema-valid cancel is refused the same way, before any ledger is consulted.
    let frame = cancel_frame(
        7,
        CANCEL_KEY,
        "28d00000-0000-4000-8000-0000000005ff",
        "1",
        &json!({"reason": "safety", "note": null}),
    )?;
    let Reply::Frame(bytes) = control::serve_composed(&frame, NOW, &operator, composed) else {
        return Err("closed".into());
    };
    let reply: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(
        (&reply["code"], &reply["details"]["constraint"]),
        (&json!("unavailable"), &json!("owner not composed"))
    );
    Ok(())
}

#[test]
fn cleanup_and_delivery_are_read_from_the_ledgers_own_states() {
    for (attempts, expected) in [
        (vec![], "none"),
        (vec!["none"], "none"),
        (vec!["settled", "none"], "settled"),
        (vec!["settled", "pending"], "pending"),
        (vec!["pending", "unknown", "settled"], "unknown"),
    ] {
        assert_eq!(cleanup_of(&attempts), expected, "{attempts:?}");
    }
    for (state, pending, expected) in [
        ("admitted", 0, "none"),
        ("running", 2, "none"),
        ("accepted", 0, "delivered"),
        ("accepted", 1, "pending"),
        ("failed", 0, "delivered"),
        ("cancelled", 3, "pending"),
        ("abandoned", 0, "none"),
    ] {
        assert_eq!(delivery_of(state, pending), expected, "{state} {pending}");
    }
}

#[test]
fn an_uncertain_commit_is_reported_as_effect_unknown_with_its_readback() -> Outcome {
    let fault = habitat_engine::contracts::control::Fault::effect_unknown(submit_readback(KEY));
    let record: Value = serde_json::from_slice(&fault.frame(KEY, &request_sha256(b"x")))?;
    assert_eq!(
        (
            &record["code"],
            &record["effect"],
            &record["retry"],
            &record["readback"]
        ),
        (
            &json!("effect_unknown"),
            &json!("unknown"),
            &json!("after_readback"),
            &json!({"action": "task.get", "action_version": 1, "body": {"selector": {"source_action": "task.submit", "idempotency_key": KEY}, "evidence": "none"}})
        )
    );
    conforms(&[("task.submit", &record)])
}

#[test]
fn the_ledgers_own_allocation_rule_decides_the_budget() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    for (index, (wall, admitted)) in [("1200000", true), ("1200001", false), ("0", false)]
        .into_iter()
        .enumerate()
    {
        let key = format!("28d00000-0000-4000-8000-0000000002{index:02x}");
        let payload = request(
            "task.submit",
            u8::try_from(0x20 + index)?,
            Some(&key),
            &json!({"spec": with(&["budget", "wall_ms"], json!(wall))}),
        );
        let reply = serve(&tasks, &operator, &payload)?;
        if admitted {
            assert_eq!(reply["effect"], json!("committed"), "{wall}: {reply}");
        } else {
            assert_eq!(
                (&reply["code"], &reply["details"]["field"]),
                (
                    &json!("invalid_argument"),
                    &json!("/body/spec/budget/wall_ms")
                ),
                "{wall}: {reply}"
            );
        }
    }
    Ok(())
}

// --- B05 · task.cancel (RC03 §6 "Cancellation"; body and result: contract-decisions.md:343) ------

const CANCEL_KEY: &str = "28d00000-0000-4000-8000-0000000005a1";
const CANCEL_KEY_2: &str = "28d00000-0000-4000-8000-0000000005a2";

/// A `task.cancel` frame: the precondition names the task and the generation the caller expects.
fn cancel_frame(
    request_no: u8,
    key: &str,
    task: &str,
    generation: &str,
    body: &Value,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut frame: Value =
        serde_json::from_slice(&request("task.cancel", request_no, Some(key), body))?;
    frame["precondition"] = json!({"resource": "task", "id": task, "generation": generation});
    Ok(serde_json::to_vec(&frame)?)
}

/// The readback RC03 §6 prescribes for a cancel, written out here rather than taken from the engine.
fn cancel_readback_of(task: &str) -> Value {
    json!({"action": "task.get", "action_version": 1,
           "body": {"selector": {"task_id": task}, "evidence": "none"}})
}

/// The body the ledger holds for event `id`, read from the ledger file itself, not through the engine.
fn event_body(scratch: &Scratch, id: &str) -> Result<Value, Box<dyn Error>> {
    let file = scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("ledger.sqlite3");
    let db =
        rusqlite::Connection::open_with_flags(file, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let (kind, body): (String, Vec<u8>) =
        db.query_row("SELECT kind,body FROM events WHERE id=?", [id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
    assert_eq!(kind, "cancellation_requested");
    Ok(serde_json::from_slice(&body)?)
}

fn submitted(tasks: &StoreTasks, operator: &Principal) -> Result<String, Box<dyn Error>> {
    let first = serve(
        tasks,
        operator,
        &request("task.submit", 1, Some(KEY), &json!({"spec": spec()})),
    )?;
    Ok(first["body"]["task"]["task_id"]
        .as_str()
        .ok_or("task id")?
        .to_owned())
}

fn head_of(tasks: &StoreTasks, operator: &Principal, task: &str) -> Result<Value, Box<dyn Error>> {
    let read = serve(
        tasks,
        operator,
        &request(
            "task.get",
            9,
            None,
            &json!({"selector": {"task_id": task}, "evidence": "none"}),
        ),
    )?;
    Ok(read["body"]["task"].clone())
}

/// B05: a cancel at the expected generation commits the intent once -- a new generation, the
/// `cancellation_requested` state, an obligation identity and the settlement it found -- and an
/// exact replay returns that stored result without a second write. Other bytes under the key
/// conflict. A second cancel under a new key finds the intent already recorded and names the same
/// obligation.
#[test]
fn a_cancel_records_intent_once_and_replays_exactly() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let task = submitted(&tasks, &operator)?;
    let why = json!({"reason": "operator_request", "note": null});
    let frame = cancel_frame(2, CANCEL_KEY, &task, "1", &why)?;
    let first = serve(&tasks, &operator, &frame)?;
    assert_eq!(
        (
            &first["kind"],
            &first["effect"],
            &first["replayed"],
            &first["observed_generation"]
        ),
        (
            &json!("result"),
            &json!("committed"),
            &json!(false),
            &json!("2")
        ),
        "{first}"
    );
    assert_eq!(first["readback"], cancel_readback_of(&task));
    assert_eq!(
        first["body"]["task"],
        json!({"task_id": task, "generation": "2", "state": "cancellation_requested",
               "current_attempt_id": null, "unresolved_obligations": 0})
    );
    assert_eq!(first["body"]["worker_settlement"], json!("not_started"));
    let obligation = first["body"]["cancellation_obligation_id"]
        .as_str()
        .ok_or("obligation id")?
        .to_owned();
    UuidV4::parse(&obligation)?;
    assert_ne!(obligation, task);
    // The obligation is the durable intent, and it holds the reason the caller gave.
    assert_eq!(
        event_body(&scratch, &obligation)?,
        json!({"reason": "operator_request", "note": null})
    );
    // The independent readback agrees with what the cancel reported.
    assert_eq!(head_of(&tasks, &operator, &task)?, first["body"]["task"]);
    let again = serve(&tasks, &operator, &frame)?;
    assert_eq!(
        (
            &again["replayed"],
            &again["body"],
            &again["observed_generation"]
        ),
        (&json!(true), &first["body"], &json!("2"))
    );
    let other = cancel_frame(
        3,
        CANCEL_KEY,
        &task,
        "1",
        &json!({"reason": "superseded", "note": null}),
    )?;
    let conflict = serve(&tasks, &operator, &other)?;
    assert_eq!(
        (&conflict["code"], &conflict["details"]["field"]),
        (&json!("conflict"), &json!("/idempotency_key"))
    );
    let second = serve(
        &tasks,
        &operator,
        &cancel_frame(4, CANCEL_KEY_2, &task, "2", &why)?,
    )?;
    assert_eq!(
        (
            &second["effect"],
            &second["replayed"],
            &second["body"]["task"]["generation"],
            &second["body"]["cancellation_obligation_id"]
        ),
        (
            &json!("committed"),
            &json!(false),
            &json!("2"),
            &json!(obligation)
        ),
        "{second}"
    );
    assert_eq!(head_of(&tasks, &operator, &task)?["generation"], json!("2"));
    conforms(&[
        ("task.cancel", &first),
        ("task.cancel", &again),
        ("task.cancel", &conflict),
        ("task.cancel", &second),
    ])?;
    Ok(())
}

/// B05: a stale expected generation is `stale_generation` naming the current one; a task the
/// principal cannot see is `not_found` and names none. Neither writes anything -- the key is not
/// bound, so the same key then commits a correct request.
#[test]
fn a_stale_or_invisible_cancel_writes_nothing() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let task = submitted(&tasks, &operator)?;
    let why = json!({"reason": "budget", "note": "over the wall"});
    let stale = serve(
        &tasks,
        &operator,
        &cancel_frame(2, CANCEL_KEY, &task, "7", &why)?,
    )?;
    assert_eq!(
        (
            &stale["code"],
            &stale["effect"],
            &stale["details"]["current_generation"],
            &stale["details"]["field"],
            &stale["retry"]
        ),
        (
            &json!("stale_generation"),
            &json!("none"),
            &json!("1"),
            &json!("/precondition/generation"),
            &json!("never")
        ),
        "{stale}"
    );
    let stranger = Principal::new(1001, "operator").map_err(|error| format!("{error:?}"))?;
    let hidden = serve(
        &tasks,
        &stranger,
        &cancel_frame(3, CANCEL_KEY, &task, "1", &why)?,
    )?;
    assert_eq!(
        (&hidden["code"], &hidden["details"]["current_generation"]),
        (&json!("not_found"), &Value::Null),
        "{hidden}"
    );
    // A stranger's STALE generation is still `not_found`: visibility is decided before the
    // generation, so an invisible task's current generation is never named ("where authorized").
    let hidden_stale = serve(
        &tasks,
        &stranger,
        &cancel_frame(3, CANCEL_KEY, &task, "7", &why)?,
    )?;
    assert_eq!(
        (
            &hidden_stale["code"],
            &hidden_stale["details"]["current_generation"]
        ),
        (&json!("not_found"), &Value::Null),
        "{hidden_stale}"
    );
    let absent = serve(
        &tasks,
        &operator,
        &cancel_frame(
            4,
            CANCEL_KEY,
            "28d00000-0000-4000-8000-0000000005ff",
            "1",
            &why,
        )?,
    )?;
    assert_eq!(absent["code"], json!("not_found"));
    assert_eq!(
        head_of(&tasks, &operator, &task)?,
        json!({"task_id": task, "generation": "1", "state": "admitted",
               "current_attempt_id": null, "unresolved_obligations": 0})
    );
    let then = serve(
        &tasks,
        &operator,
        &cancel_frame(5, CANCEL_KEY, &task, "1", &why)?,
    )?;
    assert_eq!(
        (
            &then["effect"],
            &then["replayed"],
            &then["observed_generation"]
        ),
        (&json!("committed"), &json!(false), &json!("2")),
        "{then}"
    );
    let obligation = then["body"]["cancellation_obligation_id"]
        .as_str()
        .ok_or("obligation id")?;
    assert_eq!(
        event_body(&scratch, obligation)?,
        json!({"reason": "budget", "note": "over the wall"})
    );
    conforms(&[
        ("task.cancel", &stale),
        ("task.cancel", &hidden),
        ("task.cancel", &hidden_stale),
        ("task.cancel", &absent),
        ("task.cancel", &then),
    ])?;
    Ok(())
}

/// B05: the body is exactly `{reason, note}` (a missing or extra member names `/body`); `note` is at most 1,024 UTF-8 BYTES (the schema's
/// `maxLength` counts code points, so 342 three-byte characters pass the schema and fail here).
#[test]
fn a_cancel_body_is_checked_member_by_member() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let task = submitted(&tasks, &operator)?;
    let wide = "\u{20ac}".repeat(342);
    let fits = "\u{20ac}".repeat(341);
    for (body, field) in [
        (json!({"reason": "whim", "note": null}), "/body/reason"),
        (json!({"note": null}), "/body"),
        (json!({"reason": "safety"}), "/body"),
        (json!({"reason": "safety", "note": 7}), "/body/note"),
        (json!({"reason": "safety", "note": wide}), "/body/note"),
        (
            json!({"reason": "safety", "note": null, "extra": 1}),
            "/body",
        ),
    ] {
        let reply = serve(
            &tasks,
            &operator,
            &cancel_frame(2, CANCEL_KEY, &task, "1", &body)?,
        )?;
        assert_eq!(
            (&reply["code"], &reply["details"]["field"]),
            (&json!("invalid_argument"), &json!(field)),
            "{body} -> {reply}"
        );
    }
    assert_eq!(head_of(&tasks, &operator, &task)?["generation"], json!("1"));
    let fitted = serve(
        &tasks,
        &operator,
        &cancel_frame(
            3,
            CANCEL_KEY,
            &task,
            "1",
            &json!({"reason": "safety", "note": fits}),
        )?,
    )?;
    assert_eq!(fitted["effect"], json!("committed"), "{fitted}");
    Ok(())
}

/// How far a fixture task is taken through the public ledger API before the cancel.
#[derive(Clone, Copy, PartialEq)]
enum Stage {
    /// One attempt, queued: generation 2.
    Running,
    /// Its attempt settled cleanly, not ready to verify: generation 3.
    Settled,
    /// Its attempt's settlement unknown: `effect_unknown`, generation 3.
    Unknown,
    /// Verified `Invalid`: state `failed` with no stop row yet, generation 4.
    Failed,
    /// Verified and accepted: generation 5.
    Accepted,
}

fn nth(role: u16, index: u16) -> String {
    format!("{role:08x}-0000-4000-8000-{index:012x}")
}

/// A writable ledger in `scratch`, as `ledger` opens it but without the task owner around it.
fn raw_store(scratch: &Scratch) -> Result<Store, Box<dyn Error>> {
    let root = scratch.0.join("state");
    if !root.exists() {
        DirBuilder::new().mode(0o700).create(&root)?;
    }
    Ok(Store::open(
        &root,
        UuidV4::parse(GENERATION)?,
        UuidV4::parse(EPOCH)?,
        true,
        Instant::now() + Duration::from_secs(10),
    )
    .map_err(|error| format!("{error:?}"))?)
}

/// Task `index` submitted and its one attempt begun (queued): generation 2.
fn begun(store: &mut Store, operator: &Principal, index: u16) -> Result<(), Box<dyn Error>> {
    use habitat_engine::store::{Allocation, Submission};
    let until = Instant::now() + Duration::from_secs(10);
    let fault = |error: habitat_engine::store::Error| format!("{index}: {error:?}");
    let criteria_text = format!("sha256:{}", "5".repeat(64));
    let criteria = habitat_engine::contracts::Sha256Digest::parse(&criteria_text)?;
    let (task, attempt) = (nth(0x05b1, index), nth(0x05b2, index));
    store
        .submit(
            Submission {
                principal: operator,
                key: UuidV4::parse(&nth(0x05b0, index))?,
                task: UuidV4::parse(&task)?,
                event: UuidV4::parse(&nth(0x05b3, index))?,
                request_bytes: b"stage fixture",
                criteria,
                allocation: Allocation {
                    limit_ms: 1_200_000,
                    work_ms: 900_000,
                    verify_ms: 300_000,
                },
            },
            until,
        )
        .map_err(fault)?;
    store
        .begin_attempt(
            UuidV4::parse(&task)?,
            "1".parse()?,
            UuidV4::parse(&attempt)?,
            UuidV4::parse(&nth(0x05b4, index))?,
            until,
        )
        .map_err(fault)?;
    Ok(())
}

/// Task `index`, taken to `stage` through the store's own API; returns its identity.
fn staged(
    store: &mut Store,
    operator: &Principal,
    index: u16,
    stage: Stage,
) -> Result<String, Box<dyn Error>> {
    use habitat_engine::store::{Effect, Expected, Settlement, Verification, VerificationVerdict};
    let until = Instant::now() + Duration::from_secs(10);
    let fault = |error: habitat_engine::store::Error| format!("{index}: {error:?}");
    let criteria_text = format!("sha256:{}", "5".repeat(64));
    let criteria = habitat_engine::contracts::Sha256Digest::parse(&criteria_text)?;
    let (task, attempt) = (nth(0x05b1, index), nth(0x05b2, index));
    begun(store, operator, index)?;
    if stage == Stage::Running {
        return Ok(task);
    }
    let expected = |store: &Store| -> Result<Expected<'_>, Box<dyn Error>> {
        let now = store
            .get(operator, UuidV4::parse(&task)?, until)
            .map_err(fault)?
            .generation;
        Ok(Expected {
            task: UuidV4::parse(&task)?,
            task_generation: now.parse()?,
            attempt: UuidV4::parse(&attempt)?,
            attempt_generation: "1".parse()?,
        })
    };
    let known = stage != Stage::Unknown;
    let settle = expected(store)?;
    store
        .settle_attempt(
            &settle,
            Settlement {
                effect: if known { Effect::None } else { Effect::Unknown },
                used_ms: known.then_some(10),
                cleanup_settled: known,
                ready_to_verify: matches!(stage, Stage::Failed | Stage::Accepted),
            },
            UuidV4::parse(&nth(0x05b5, index))?,
            until,
        )
        .map_err(fault)?;
    if matches!(stage, Stage::Settled | Stage::Unknown) {
        return Ok(task);
    }
    let evidence = store
        .publish(b"stage fixture evidence", UuidV4::parse(EPOCH)?, until)
        .map_err(fault)?;
    let verify = expected(store)?;
    store
        .record_verification(
            &verify,
            &Verification {
                verdict: if stage == Stage::Failed {
                    VerificationVerdict::Invalid
                } else {
                    VerificationVerdict::Passed
                },
                subject: criteria,
                evidence: evidence.clone(),
                used_ms: Some(20),
                cleanup_settled: true,
            },
            UuidV4::parse(&nth(0x05b7, index))?,
            until,
        )
        .map_err(fault)?;
    if stage == Stage::Failed {
        return Ok(task);
    }
    let accept = expected(store)?;
    let publication = store
        .prepare_verified_acceptance(
            &accept,
            UuidV4::parse(&nth(0x05b8, index))?,
            criteria,
            &evidence,
            std::slice::from_ref(&evidence),
            until,
        )
        .map_err(fault)?;
    store.accept(&publication, 0, until).map_err(fault)?;
    Ok(task)
}

/// Obligations the ledger file itself holds for `task`: attempts whose effect or cleanup is pending
/// or unknown, plus undelivered outbox rows -- counted in SQL here, not through the engine's view.
fn ledger_obligations(scratch: &Scratch, task: &str) -> Result<u64, Box<dyn Error>> {
    let file = scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("ledger.sqlite3");
    let db =
        rusqlite::Connection::open_with_flags(file, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(db.query_row(
        "SELECT (SELECT count(*) FROM attempts WHERE task_id=?1 AND (effect IN ('pending','unknown') OR cleanup IN ('pending','unknown'))) \
         + (SELECT count(*) FROM outbox o JOIN events e ON e.id=o.event_id WHERE e.task_id=?1 AND o.delivered=0)",
        [task],
        |row| row.get::<_, u32>(0),
    )
    .map(u64::from)?)
}

/// B05: `worker_settlement` and the head are read from the task in the cancel's own transaction,
/// each pinned whole against literals and the ledger file: a live attempt is `pending` (and is the
/// current attempt), a settled one `settled`, one whose settlement is unknown `unknown` -- and an
/// `effect_unknown` task KEEPS that state (the store's precedence: an unknown effect is never
/// masked by a cancellation), while the intent is still recorded at a new generation.
#[test]
fn worker_settlement_is_read_from_the_attempts_the_cancel_found() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let mut store = raw_store(&scratch)?;
    let cases = [
        (
            1_u16,
            Stage::Running,
            "2",
            "3",
            "cancellation_requested",
            "pending",
        ),
        (
            2,
            Stage::Settled,
            "3",
            "4",
            "cancellation_requested",
            "settled",
        ),
        (3, Stage::Unknown, "3", "4", "effect_unknown", "unknown"),
    ];
    for (index, stage, _, _, _, _) in cases {
        staged(&mut store, &operator, index, stage)?;
    }
    let tasks = StoreTasks::new(store, EPOCH.to_owned());
    let why = json!({"reason": "deadline", "note": null});
    let mut replies = Vec::new();
    let mut owed = 0;
    for (index, _, before, after, state, settlement) in cases {
        let task = nth(0x05b1, index);
        assert_eq!(
            head_of(&tasks, &operator, &task)?["generation"],
            json!(before)
        );
        let reply = serve(
            &tasks,
            &operator,
            &cancel_frame(2, &nth(0x05b6, index), &task, before, &why)?,
        )?;
        let obligations = ledger_obligations(&scratch, &task)?;
        owed += obligations;
        let current = if index == 1 {
            json!(nth(0x05b2, 1))
        } else {
            Value::Null
        };
        assert_eq!(
            (
                &reply["observed_generation"],
                &reply["body"]["worker_settlement"]
            ),
            (&json!(after), &json!(settlement)),
            "{index}: {reply}"
        );
        assert_eq!(
            reply["body"]["task"],
            json!({"task_id": task, "generation": after, "state": state,
                   "current_attempt_id": current, "unresolved_obligations": obligations}),
            "{index}: {reply}"
        );
        replies.push(reply);
    }
    assert!(
        owed > 0,
        "some fixture must owe an obligation, or the count is pinned only at 0"
    );
    conforms(
        &replies
            .iter()
            .map(|reply| ("task.cancel", reply))
            .collect::<Vec<_>>(),
    )?;
    Ok(())
}

/// B05: an accepted task's outcome is decided: the cancel is refused `conflict` at `/precondition`
/// and writes nothing. A task `failed` by verification has NOT stopped yet -- T06 pins that a
/// cancellation recorded before the stop wins -- so its cancel commits intent at the next
/// generation, and the stop will record `cancelled`.
#[test]
fn a_cancel_of_an_accepted_task_is_refused_and_of_a_failed_one_wins() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let mut store = raw_store(&scratch)?;
    staged(&mut store, &operator, 4, Stage::Failed)?;
    staged(&mut store, &operator, 5, Stage::Accepted)?;
    let tasks = StoreTasks::new(store, EPOCH.to_owned());
    let why = json!({"reason": "superseded", "note": null});
    let accepted = nth(0x05b1, 5);
    let before = head_of(&tasks, &operator, &accepted)?;
    assert_eq!(
        (&before["state"], &before["generation"]),
        (&json!("accepted"), &json!("5")),
        "{before}"
    );
    let refused = serve(
        &tasks,
        &operator,
        &cancel_frame(2, &nth(0x05b6, 5), &accepted, "5", &why)?,
    )?;
    assert_eq!(
        (
            &refused["code"],
            &refused["effect"],
            &refused["details"]["field"]
        ),
        (&json!("conflict"), &json!("none"), &json!("/precondition")),
        "{refused}"
    );
    assert_eq!(head_of(&tasks, &operator, &accepted)?, before);
    let failed = nth(0x05b1, 4);
    assert_eq!(
        head_of(&tasks, &operator, &failed)?["state"],
        json!("failed")
    );
    let wins = serve(
        &tasks,
        &operator,
        &cancel_frame(3, &nth(0x05b6, 4), &failed, "4", &why)?,
    )?;
    assert_eq!(
        (
            &wins["effect"],
            &wins["body"]["task"]["generation"],
            &wins["body"]["task"]["state"],
            &wins["body"]["worker_settlement"]
        ),
        (
            &json!("committed"),
            &json!("5"),
            &json!("cancellation_requested"),
            &json!("settled")
        ),
        "{wins}"
    );
    conforms(&[("task.cancel", &refused), ("task.cancel", &wins)])?;
    Ok(())
}

/// B05, RC03 §6: an exact replay returns the STORED result even after the task has moved on -- here
/// its attempt settles after the cancel -- never a result recomputed from the task as it now is.
#[test]
fn a_replay_returns_the_stored_result_after_the_task_moves_on() -> Outcome {
    use habitat_engine::store::{Effect, Expected, Settlement};
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let mut store = raw_store(&scratch)?;
    let task = staged(&mut store, &operator, 1, Stage::Running)?;
    let tasks = StoreTasks::new(store, EPOCH.to_owned());
    let frame = cancel_frame(
        2,
        CANCEL_KEY,
        &task,
        "2",
        &json!({"reason": "operator_request", "note": null}),
    )?;
    let first = serve(&tasks, &operator, &frame)?;
    assert_eq!(
        (
            &first["body"]["task"]["generation"],
            &first["body"]["worker_settlement"]
        ),
        (&json!("3"), &json!("pending")),
        "{first}"
    );
    drop(tasks);
    let mut store = raw_store(&scratch)?;
    store
        .settle_attempt(
            &Expected {
                task: UuidV4::parse(&task)?,
                task_generation: "3".parse()?,
                attempt: UuidV4::parse(&nth(0x05b2, 1))?,
                attempt_generation: "1".parse()?,
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(10),
                cleanup_settled: true,
                ready_to_verify: false,
            },
            UuidV4::parse(&nth(0x05b9, 1))?,
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?;
    let tasks = StoreTasks::new(store, EPOCH.to_owned());
    assert_eq!(head_of(&tasks, &operator, &task)?["generation"], json!("4"));
    let again = serve(&tasks, &operator, &frame)?;
    assert_eq!(
        (
            &again["replayed"],
            &again["body"],
            &again["observed_generation"]
        ),
        (&json!(true), &first["body"], &json!("3")),
        "{again}"
    );
    Ok(())
}

/// B05: a task that already stopped without a cancellation (here: failed after verification) has
/// no intent to record -- its outcome stays historical (RC03 §6). The cancel is refused and writes
/// nothing.
#[test]
fn a_cancel_of_a_task_that_already_stopped_is_refused() -> Outcome {
    let scratch = Scratch::new()?;
    let root = scratch.0.join("state");
    DirBuilder::new().mode(0o700).create(&root)?;
    let until = Instant::now() + Duration::from_secs(10);
    let mut store = Store::open(
        &root,
        UuidV4::parse(GENERATION)?,
        UuidV4::parse(EPOCH)?,
        true,
        until,
    )
    .map_err(|error| format!("{error:?}"))?;
    let evidence = store
        .publish(b"retained fixture evidence", UuidV4::parse(EPOCH)?, until)
        .map_err(|error| format!("{error:?}"))?;
    let criteria_text = format!("sha256:{}", "5".repeat(64));
    let criteria = habitat_engine::contracts::Sha256Digest::parse(&criteria_text)?;
    super::socket::fail_task(&mut store, &evidence, criteria, 1)?;
    let operator = super::socket::operator()?;
    let task = format!("{:08x}-0000-4000-8000-{:012x}", 0x28d2, 1);
    let tasks = StoreTasks::new(store, EPOCH.to_owned());
    let before = head_of(&tasks, &operator, &task)?;
    assert_eq!(before["state"], json!("failed"), "{before}");
    let generation = before["generation"].as_str().ok_or("generation")?;
    let refused = serve(
        &tasks,
        &operator,
        &cancel_frame(
            2,
            CANCEL_KEY,
            &task,
            generation,
            &json!({"reason": "operator_request", "note": null}),
        )?,
    )?;
    assert_eq!(
        (
            &refused["code"],
            &refused["effect"],
            &refused["details"]["field"]
        ),
        (&json!("conflict"), &json!("none"), &json!("/precondition")),
        "{refused}"
    );
    assert_eq!(head_of(&tasks, &operator, &task)?, before);
    conforms(&[("task.cancel", &refused)])?;
    Ok(())
}

// --- B05 deferred (a) · RC03 §6: "For an already-recorded key, return its stored disposition even if
// the original deadline has since passed; this is readback, not new execution." ------------------

/// The request frames carry `deadline_unix_ms = NOW + 5_000`: at that instant the deadline has passed
/// (the wire's rule is `deadline <= now`), and an hour later it is far outside any admitted window.
const AT_DEADLINE: u64 = NOW + 5_000;
const AN_HOUR_LATE: u64 = NOW + 3_600_000;

/// B05 (a): an exact replay after its deadline answers the stored result -- the whole frame the first
/// reply carried, with `replayed: true` -- for both actions that record one. A submit's engine cursor
/// is issued at the replay's own receipt, as it is on a replay inside the deadline.
#[test]
fn an_exact_replay_after_its_deadline_returns_the_stored_result() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let submit = request("task.submit", 1, Some(KEY), &json!({"spec": spec()}));
    let first = serve(&tasks, &operator, &submit)?;
    assert_eq!(first["replayed"], json!(false), "{first}");
    let task = first["body"]["task"]["task_id"]
        .as_str()
        .ok_or("task id")?
        .to_owned();
    let cancel = cancel_frame(
        2,
        CANCEL_KEY,
        &task,
        "1",
        &json!({"reason": "deadline", "note": "past its window"}),
    )?;
    let cancelled = serve(&tasks, &operator, &cancel)?;
    assert_eq!(
        (&cancelled["replayed"], &cancelled["observed_generation"]),
        (&json!(false), &json!("2")),
        "{cancelled}"
    );
    let mut replies = vec![
        ("task.submit", first.clone()),
        ("task.cancel", cancelled.clone()),
    ];
    for now in [AT_DEADLINE, AN_HOUR_LATE] {
        let mut expected = first.clone();
        expected["replayed"] = json!(true);
        expected["body"]["engine_cursor"]["issued_unix_ms"] = json!(now.to_string());
        expected["body"]["engine_cursor"]["expires_unix_ms"] =
            json!((now + 86_400_000).to_string());
        let again = serve_at(&tasks, &operator, &submit, now)?;
        assert_eq!(again, expected, "task.submit replayed at {now}");
        replies.push(("task.submit", again));
        let mut expected = cancelled.clone();
        expected["replayed"] = json!(true);
        let again = serve_at(&tasks, &operator, &cancel, now)?;
        assert_eq!(again, expected, "task.cancel replayed at {now}");
        replies.push(("task.cancel", again));
    }
    // A replay is readback, not new execution: the task did not move.
    assert_eq!(
        head_of(&tasks, &operator, &task)?,
        cancelled["body"]["task"]
    );
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|(action, reply)| (*action, reply))
        .collect();
    conforms(&rows)?;
    Ok(())
}

/// Expired requests whose key is not recorded for their principal and action, each with the action it names
/// and the principal that sends it, after `task` was submitted under `KEY` and cancelled under
/// `CANCEL_KEY` to generation 2.
type Case<'a> = (&'static str, &'static str, &'a Principal, Vec<u8>);

fn expired_cases<'a>(
    task: &str,
    operator: &'a Principal,
    other: &'a Principal,
) -> Result<Vec<Case<'a>>, Box<dyn Error>> {
    let why = json!({"reason": "operator_request", "note": null});
    Ok(vec![
        (
            "unseen submit key",
            "task.submit",
            operator,
            request(
                "task.submit",
                3,
                Some(CANCEL_KEY_2),
                &json!({"spec": spec()}),
            ),
        ),
        (
            "another principal's recorded submit",
            "task.submit",
            other,
            request("task.submit", 1, Some(KEY), &json!({"spec": spec()})),
        ),
        (
            "unseen cancel key",
            "task.cancel",
            operator,
            cancel_frame(5, CANCEL_KEY_2, task, "2", &why)?,
        ),
        (
            "another action's recorded key",
            "task.cancel",
            operator,
            cancel_frame(8, KEY, task, "2", &why)?,
        ),
        (
            "no idempotency key",
            "task.submit",
            operator,
            request("task.submit", 9, None, &json!({"spec": spec()})),
        ),
        (
            "a body that is not an object",
            "task.submit",
            operator,
            request("task.submit", 10, Some(CANCEL_KEY_2), &json!(1)),
        ),
        (
            "a read",
            "task.get",
            operator,
            request(
                "task.get",
                7,
                None,
                &json!({"selector": {"task_id": task}, "evidence": "none"}),
            ),
        ),
    ])
}

/// B05 (a), the other half of the rule: an expired request whose key is not recorded for its principal
/// and action is refused `deadline_exceeded` before dispatch, as before -- an unseen key, another
/// principal's recorded key, another action's recorded key, no key, a read, a request the catalogue
/// would refuse and one malformed past its deadline -- and nothing is written. A key is bound per
/// action (RC03 §6), so a live cancel under the submit's key then commits.
#[test]
fn an_expired_request_that_is_not_a_recorded_replay_is_refused_and_writes_nothing() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let other = Principal::new(1001, "operator").map_err(|error| format!("{error:?}"))?;
    let task = submitted(&tasks, &operator)?;
    let why = json!({"reason": "operator_request", "note": null});
    let recorded = cancel_frame(2, CANCEL_KEY, &task, "1", &why)?;
    let cancelled = serve(&tasks, &operator, &recorded)?;
    assert_eq!(cancelled["observed_generation"], json!("2"), "{cancelled}");
    let cases = expired_cases(&task, &operator, &other)?;
    let mut replies = Vec::new();
    for (case, action, principal, frame) in &cases {
        let refused = serve_at(&tasks, principal, frame, AN_HOUR_LATE)?;
        assert_eq!(
            (
                &refused["kind"],
                &refused["code"],
                &refused["effect"],
                &refused["details"]["field"]
            ),
            (
                &json!("error"),
                &json!("deadline_exceeded"),
                &json!("none"),
                &json!("/deadline_unix_ms")
            ),
            "{case}: {refused}"
        );
        replies.push((*action, refused));
    }
    // Nothing was written: the task is where the cancel left it, and the unseen submit key names no
    // task -- read back inside a live window.
    assert_eq!(
        head_of(&tasks, &operator, &task)?,
        cancelled["body"]["task"]
    );
    let by_key = serve(
        &tasks,
        &operator,
        &request(
            "task.get",
            8,
            None,
            &json!({"selector": {"source_action": "task.submit", "idempotency_key": CANCEL_KEY_2},
                    "evidence": "none"}),
        ),
    )?;
    assert_eq!(by_key["code"], json!("not_found"), "{by_key}");
    let live = serve(&tasks, &operator, &cancel_frame(11, KEY, &task, "2", &why)?)?;
    assert_eq!(
        (
            &live["effect"],
            &live["replayed"],
            &live["observed_generation"]
        ),
        (&json!("committed"), &json!(false), &json!("2")),
        "{live}"
    );
    replies.push(("task.cancel", live));
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|(action, reply)| (*action, reply))
        .collect();
    conforms(&rows)?;
    Ok(())
}

/// B05 (a), review F3: a recorded key answers its disposition after the deadline as before it (RC03
/// §6, "For an already-recorded key, return its stored disposition"; "Same selector with another
/// digest returns `conflict` and does not mutate"). Other bytes under a recorded key are `conflict`
/// at `/idempotency_key`, for both actions, and nothing is written.
#[test]
fn a_recorded_key_answers_other_bytes_with_conflict_after_its_deadline() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let task = submitted(&tasks, &operator)?;
    let why = json!({"reason": "operator_request", "note": null});
    let cancelled = serve(
        &tasks,
        &operator,
        &cancel_frame(2, CANCEL_KEY, &task, "1", &why)?,
    )?;
    assert_eq!(cancelled["observed_generation"], json!("2"), "{cancelled}");
    let mut replies = Vec::new();
    for (action, frame) in [
        (
            "task.submit",
            request("task.submit", 4, Some(KEY), &json!({"spec": spec()})),
        ),
        (
            "task.cancel",
            cancel_frame(6, CANCEL_KEY, &task, "2", &why)?,
        ),
    ] {
        let refused = serve_at(&tasks, &operator, &frame, AN_HOUR_LATE)?;
        assert_eq!(
            (
                &refused["code"],
                &refused["effect"],
                &refused["retry"],
                &refused["details"]["field"]
            ),
            (
                &json!("conflict"),
                &json!("none"),
                &json!("never"),
                &json!("/idempotency_key")
            ),
            "{action}: {refused}"
        );
        replies.push((action, refused));
    }
    assert_eq!(
        head_of(&tasks, &operator, &task)?,
        cancelled["body"]["task"]
    );
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|(action, reply)| (*action, reply))
        .collect();
    conforms(&rows)?;
    Ok(())
}

/// A grant store that records every question it is asked, and answers one fixed caller.
struct Asked {
    answer: Option<Caller>,
    asked: std::cell::RefCell<Vec<(String, String, String, u64)>>,
}

impl Grants for Asked {
    fn resolve(
        &self,
        principal: &Principal,
        grant_id: &str,
        scope_sha256: &str,
        now_unix_ms: u64,
    ) -> Option<Caller> {
        self.asked.borrow_mut().push((
            format!("{principal:?}"),
            grant_id.to_owned(),
            scope_sha256.to_owned(),
            now_unix_ms,
        ));
        self.answer.clone()
    }
}

/// B05 (a), review F1: a record is read past its deadline only for a principal whose grant may read
/// it. The same exact replay is refused `deadline_exceeded` when no grant resolves and when the grant
/// lacks the action's effect, and answered from the record when the grant covers it -- through one
/// recording double, which must have been asked with the transport's principal, the request's grant
/// and scope, and the replay's receipt time.
#[test]
fn a_replay_past_its_deadline_needs_the_grant_that_may_read_it() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    // The grant named by the request is distinct from its idempotency key, so a grant store asked
    // with the key (or anything but the request's own grant) is told apart.
    let grant = "28d00000-0000-4000-8000-0000000006a1";
    let mut frame: Value = serde_json::from_slice(&request(
        "task.submit",
        1,
        Some(KEY),
        &json!({"spec": spec()}),
    ))?;
    frame["authority"]["grant_id"] = json!(grant);
    let submit = serde_json::to_vec(&frame)?;
    let first = serve(&tasks, &operator, &submit)?;
    assert_eq!(first["replayed"], json!(false), "{first}");
    let seeing = Owner::ALL.into_iter().fold(Caller::new(), Caller::seeing);
    let covered = Effect::ALL
        .into_iter()
        .fold(seeing.clone(), Caller::granted);
    let uncovered = Effect::ALL
        .into_iter()
        .filter(|effect| *effect != Effect::DurableAdmission)
        .fold(seeing, Caller::granted);
    let scope = format!("sha256:{}", "4".repeat(64));
    let mut replies = Vec::new();
    for (case, answer, code) in [
        ("no grant", None, "deadline_exceeded"),
        (
            "a grant without the effect",
            Some(uncovered),
            "deadline_exceeded",
        ),
        ("a covering grant", Some(covered), "replayed"),
    ] {
        let grants = Asked {
            answer,
            asked: std::cell::RefCell::new(Vec::new()),
        };
        let reply = serve_composed_at(&tasks, &grants, &operator, &submit, AN_HOUR_LATE)?;
        let got = if reply["replayed"] == json!(true) {
            "replayed"
        } else {
            reply["code"].as_str().unwrap_or("?")
        };
        assert_eq!(got, code, "{case}: {reply}");
        assert_eq!(
            *grants.asked.borrow(),
            [(
                format!("{operator:?}"),
                grant.to_owned(),
                scope.clone(),
                AN_HOUR_LATE
            )],
            "{case}"
        );
        replies.push(("task.submit", reply));
    }
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|(action, reply)| (*action, reply))
        .collect();
    conforms(&rows)?;
    Ok(())
}

/// B05 (a), review F4: `Recorded` names the actions whose owner records a result. It is checked
/// against the world, the catalogue, rather than trusted: every action that changes state and is
/// not `Recorded` must be refused live `unavailable / owner not composed`, so nothing is ever
/// recorded that a replay past its deadline would owe; composing a new owner without naming it in
/// `Recorded` turns this red. (That each `Recorded` action is asked of its owner past the deadline
/// is pinned by `the_owner_reads_an_expired_record_within_the_wires_own_window`, through a double
/// that records the kind it was asked for; a `{}` body here is refused before any owner is reached.)
#[test]
fn every_mutating_action_that_records_nothing_has_no_owner_to_record_it() -> Outcome {
    use habitat_engine::actions::{CATALOGUE, PreconditionRule};
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (mut recording, mut unowned) = (Vec::new(), Vec::new());
    for (index, action) in CATALOGUE
        .iter()
        .filter(|action| action.effect.mutates())
        .enumerate()
    {
        let mut frame: Value = serde_json::from_slice(&request(
            action.id,
            0x40 + u8::try_from(index)?,
            Some(KEY),
            &json!({}),
        ))?;
        frame["action_version"] = json!(action.wire_version().ok_or("action version")?);
        frame["precondition"] = match action.precondition {
            PreconditionRule::Forbidden => Value::Null,
            PreconditionRule::Optional(kind) | PreconditionRule::Required(kind) => json!({
                "resource": kind.name(), "id": "28d00000-0000-4000-8000-0000000000cc",
                "generation": "1"}),
        };
        let reply = serve(&tasks, &operator, &serde_json::to_vec(&frame)?)?;
        let owner_absent = (&reply["code"], &reply["details"]["constraint"])
            == (&json!("unavailable"), &json!("owner not composed"));
        if Recorded::of(action.id).is_some() {
            assert!(
                !owner_absent,
                "{} records but has no owner: {reply}",
                action.id
            );
            recording.push(action.id);
        } else {
            assert!(
                owner_absent,
                "{} may record, but is not Recorded: {reply}",
                action.id
            );
            unowned.push(action.id);
        }
    }
    recording.sort_unstable();
    assert_eq!(recording, ["task.cancel", "task.submit"]);
    assert!(
        unowned.len() >= 5,
        "the world is the catalogue's mutating actions: {unowned:?}"
    );
    Ok(())
}

/// A task owner that records what each call was handed and answers nothing (review F5; F101: a
/// double that discards its arguments pins no value).
#[derive(Default)]
struct Handed(std::cell::RefCell<Vec<String>>);

impl Tasks for Handed {
    fn submit(
        &self,
        request: &TaskRequest<'_>,
        spec: &habitat_engine::task::control::Spec,
    ) -> Result<
        habitat_engine::contracts::control::Outcome,
        habitat_engine::contracts::control::Fault,
    > {
        self.0.borrow_mut().push(format!(
            "submit {} {}",
            request.idempotency_key, spec.limit_ms
        ));
        Err(habitat_engine::contracts::control::Fault::expired())
    }

    fn get(
        &self,
        principal: &Principal,
        selector: &Selector,
        deadline_unix_ms: u64,
        now_unix_ms: u64,
    ) -> Result<
        habitat_engine::contracts::control::Outcome,
        habitat_engine::contracts::control::Fault,
    > {
        self.0.borrow_mut().push(format!(
            "get {principal:?} {selector:?} {deadline_unix_ms} {now_unix_ms}"
        ));
        Err(habitat_engine::contracts::control::Fault::expired())
    }

    fn cancel(
        &self,
        request: &TaskRequest<'_>,
        target: &habitat_engine::contracts::control::Precondition,
        body: &habitat_engine::task::control::Cancel,
    ) -> Result<
        habitat_engine::contracts::control::Outcome,
        habitat_engine::contracts::control::Fault,
    > {
        self.0.borrow_mut().push(format!(
            "cancel {} {} {}",
            request.idempotency_key,
            target.id,
            body.reason.name()
        ));
        Err(habitat_engine::contracts::control::Fault::expired())
    }

    fn replay(
        &self,
        request: &TaskRequest<'_>,
        of: Recorded,
    ) -> Result<
        Option<habitat_engine::contracts::control::Outcome>,
        habitat_engine::contracts::control::Fault,
    > {
        self.0.borrow_mut().push(format!(
            "replay {of:?} {:?} {} {} {} {}",
            request.principal,
            request.idempotency_key,
            digest(Sha256::digest(request.payload)),
            request.deadline_unix_ms,
            request.now_unix_ms
        ));
        Ok(None)
    }
}

/// B05 (a), review F5: the owner is handed exactly the expired request -- its principal, key and
/// exact bytes -- with a read window of the wire's own maximum, 60 000 ms from receipt (RC03 §6
/// "at most 60,000 ms ahead"); a read past its deadline is never handed to the owner at all; and the
/// store's read of the record is bounded by the deadline it is given.
#[test]
fn the_owner_reads_an_expired_record_within_the_wires_own_window() -> Outcome {
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let submit = request("task.submit", 1, Some(KEY), &json!({"spec": spec()}));
    let cancel = cancel_frame(
        2,
        CANCEL_KEY,
        "28d00000-0000-4000-8000-0000000000cc",
        "1",
        &json!({"reason": "operator_request", "note": null}),
    )?;
    let read = request(
        "task.get",
        3,
        None,
        &json!({"selector": {"task_id": "28d00000-0000-4000-8000-0000000000cc"}, "evidence": "none"}),
    );
    let handed = Handed::default();
    for frame in [&submit, &cancel, &read] {
        let reply = serve_composed_at(&handed, &Open, &operator, frame, AN_HOUR_LATE)?;
        assert_eq!(reply["code"], json!("deadline_exceeded"), "{reply}");
    }
    let window = AN_HOUR_LATE + 60_000;
    assert_eq!(
        *handed.0.borrow(),
        [
            format!(
                "replay Submit {operator:?} {KEY} {} {window} {AN_HOUR_LATE}",
                digest(Sha256::digest(&submit))
            ),
            format!(
                "replay Cancel {operator:?} {CANCEL_KEY} {} {window} {AN_HOUR_LATE}",
                digest(Sha256::digest(&cancel))
            ),
        ]
    );
    // The store's own read is bounded: a deadline already passed is refused before any read, and a
    // live one reads an unseen key as nothing.
    let scratch = Scratch::new()?;
    let store = raw_store(&scratch)?;
    let passed = Instant::now()
        .checked_sub(Duration::from_millis(1))
        .ok_or("clock")?;
    let refused = store.replayed_submit(&operator, UuidV4::parse(KEY)?, &submit, passed);
    assert!(
        matches!(refused, Err(habitat_engine::store::Error::Deadline)),
        "{refused:?}"
    );
    let unseen = store
        .replayed_submit(
            &operator,
            UuidV4::parse(KEY)?,
            &submit,
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!(unseen, None);
    Ok(())
}
