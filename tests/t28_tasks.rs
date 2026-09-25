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
pub(super) const GENERATION: &str = "28d00000-0000-4000-8000-000000000001";
const EPOCH: &str = "28d00000-0000-4000-8000-000000000002";
pub(super) const KEY: &str = "28d00000-0000-4000-8000-0000000000aa";
pub(super) const NOW: u64 = 1_790_000_000_000;

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

pub(super) struct Scratch(pub(super) PathBuf);

impl Scratch {
    pub(super) fn new() -> Result<Self, Box<dyn Error>> {
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
pub(super) struct Open;

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
pub(super) fn serve_composed_at(
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
pub(super) fn conforms(replies: &[(&str, &Value)]) -> Outcome {
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
    // B06: a valid list is refused the same way; an invalid one names its body member first.
    for (body, code, constraint) in [
        (
            unfiltered(10, &Value::Null),
            "unavailable",
            json!("owner not composed"),
        ),
        (
            json!({}),
            "invalid_argument",
            json!("exactly its declared members"),
        ),
    ] {
        let Reply::Frame(bytes) =
            control::serve_composed(&list_frame(8, &body), NOW, &operator, composed)
        else {
            return Err("closed".into());
        };
        let reply: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(
            (&reply["code"], &reply["details"]["constraint"]),
            (&json!(code), &constraint),
            "{reply}"
        );
    }
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
    // B08: `abandoned` is a stop like the others (an operator's abandonment through the stop door,
    // with its notification), and a delivery given up is `unknown`, never `delivered`.
    for (state, pending, given_up, expected) in [
        ("admitted", 0, 0, "none"),
        ("running", 2, 0, "none"),
        ("accepted", 0, 0, "delivered"),
        ("accepted", 1, 0, "pending"),
        ("accepted", 0, 1, "unknown"),
        ("failed", 0, 0, "delivered"),
        ("cancelled", 3, 1, "pending"),
        ("abandoned", 0, 0, "delivered"),
        ("abandoned", 1, 0, "pending"),
        ("blocked", 0, 1, "none"),
    ] {
        assert_eq!(
            delivery_of(state, pending, given_up),
            expected,
            "{state} {pending} {given_up}"
        );
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
    /// Its attempt's effect unknown, but cleanup settled and usage known: `effect_unknown`, gen 3.
    UnknownEffectOnly,
    /// Its attempt's effect observed `none`, but usage unknown: the attempt stays `unknown`, gen 3.
    UsageUnknown,
    /// Its attempt's effect unknown and its usage unknown, cleanup settled: `effect_unknown`, gen 3.
    UnknownEffectAndUsage,
    /// Verified `Invalid`: state `failed` with no stop row yet, generation 4.
    Failed,
    /// Verified and accepted: generation 5.
    Accepted,
}

fn nth(role: u16, index: u16) -> String {
    format!("{role:08x}-0000-4000-8000-{index:012x}")
}

/// A writable ledger in `scratch`, as `ledger` opens it but without the task owner around it.
pub(super) fn raw_store(scratch: &Scratch) -> Result<Store, Box<dyn Error>> {
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
                effect: if known
                    && !matches!(
                        stage,
                        Stage::UnknownEffectOnly | Stage::UnknownEffectAndUsage
                    ) {
                    Effect::None
                } else {
                    Effect::Unknown
                },
                used_ms: (known
                    && !matches!(stage, Stage::UsageUnknown | Stage::UnknownEffectAndUsage))
                .then_some(10),
                cleanup_settled: known,
                ready_to_verify: matches!(stage, Stage::Failed | Stage::Accepted),
            },
            UuidV4::parse(&nth(0x05b5, index))?,
            until,
        )
        .map_err(fault)?;
    if matches!(
        stage,
        Stage::Settled
            | Stage::Unknown
            | Stage::UnknownEffectOnly
            | Stage::UsageUnknown
            | Stage::UnknownEffectAndUsage
    ) {
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
    assert_eq!(recording, ["task.cancel", "task.resolve", "task.submit"]);
    assert!(
        unowned.len() >= 5,
        "the world is the catalogue's mutating actions: {unowned:?}"
    );
    Ok(())
}

/// A task owner that records what each call was handed and answers nothing (review F5; F101: a
/// double that discards its arguments pins no value).
#[derive(Default)]
pub(super) struct Handed(pub(super) std::cell::RefCell<Vec<String>>);

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

    fn resolve(
        &self,
        request: &TaskRequest<'_>,
        target: &habitat_engine::contracts::control::Precondition,
        body: &habitat_engine::task::control::Resolve,
    ) -> Result<
        habitat_engine::contracts::control::Outcome,
        habitat_engine::contracts::control::Fault,
    > {
        self.0.borrow_mut().push(format!(
            "resolve {} {} {} {}",
            request.idempotency_key,
            target.id,
            body.obligation_id,
            body.disposition.name()
        ));
        Err(habitat_engine::contracts::control::Fault::expired())
    }

    fn list(
        &self,
        principal: &Principal,
        list: &habitat_engine::task::control::List,
        deadline_unix_ms: u64,
        now_unix_ms: u64,
    ) -> Result<
        habitat_engine::contracts::control::Outcome,
        habitat_engine::contracts::control::Fault,
    > {
        self.0.borrow_mut().push(format!(
            "list {principal:?} {} {deadline_unix_ms} {now_unix_ms}",
            list.filter_sha256()
        ));
        Err(habitat_engine::contracts::control::Fault::expired())
    }

    fn preview(
        &self,
        principal: &Principal,
        preview: &habitat_engine::task::control::Preview,
        deadline_unix_ms: u64,
        now_unix_ms: u64,
    ) -> Result<
        habitat_engine::contracts::control::Outcome,
        habitat_engine::contracts::control::Fault,
    > {
        self.0.borrow_mut().push(format!(
            "preview {principal:?} {} {} {deadline_unix_ms} {now_unix_ms}",
            preview.spec.task_class, preview.spec.limit_ms
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

// --- B06 · task.list (task-G05; body and result: contract-decisions.md:342; pages: :319-321) ---------

/// A `task.submit` frame under the `n`th list key, so each admission is a distinct task.
fn submit_nth(n: u8) -> Vec<u8> {
    request(
        "task.submit",
        0x60 + n,
        Some(&format!("28d00000-0000-4000-8000-0000000007{n:02x}")),
        &json!({"spec": spec()}),
    )
}

/// A `task.list` frame: `states`, `task_class`, `parent_task_id` and the page, as given.
fn list_frame(request_no: u8, body: &Value) -> Vec<u8> {
    request("task.list", request_no, None, body)
}

/// A `task.list` frame whose own deadline is live at receiver time `now_unix_ms`, for cases served
/// later than `NOW`: the request's deadline and the cursor's lifetime are different clocks.
fn list_frame_at(
    request_no: u8,
    body: &Value,
    now_unix_ms: u64,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut frame: Value = serde_json::from_slice(&list_frame(request_no, body))?;
    frame["deadline_unix_ms"] = json!((now_unix_ms + 5_000).to_string());
    Ok(serde_json::to_vec(&frame)?)
}

/// The ledger's global event high-water, read from the ledger file itself, not through the engine.
fn high_water(scratch: &Scratch) -> Result<u64, Box<dyn Error>> {
    let file = scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("ledger.sqlite3");
    let db =
        rusqlite::Connection::open_with_flags(file, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(db
        .query_row("SELECT max(sequence) FROM events", [], |row| {
            row.get::<_, i64>(0)
        })
        .map(u64::try_from)??)
}

/// The filter digest a `task.list` cursor binds, computed here with `sha2` over the compact form this
/// test writes by hand (keys in byte order, `states` sorted, no LF), not through the engine.
fn list_filter(states: &[&str], class: Option<&str>, parent: Option<&str>) -> String {
    let mut sorted = states.to_vec();
    sorted.sort_unstable();
    let quoted = |text: Option<&str>| text.map_or("null".to_owned(), |text| format!("\"{text}\""));
    let states: Vec<String> = sorted.iter().map(|state| format!("\"{state}\"")).collect();
    let form = format!(
        "{{\"parent_task_id\":{},\"states\":[{}],\"task_class\":{}}}",
        quoted(parent),
        states.join(","),
        quoted(class)
    );
    digest(Sha256::digest(form.as_bytes()))
}

fn unfiltered(limit: u64, cursor: &Value) -> Value {
    json!({"states": [], "task_class": null, "parent_task_id": null,
           "page": {"limit": limit, "cursor": cursor}})
}

/// B06: a principal's tasks page in admission order under one snapshot. Each page is asserted whole
/// -- off the origin: the second and third pages, not only the first -- and every item equals the
/// head `task.get` reads for that task. The snapshot revision is the ledger's event high-water when
/// the listing began, read from the ledger file; a task admitted after it is not a member of this
/// listing. Another principal's task is never listed.
#[test]
fn a_list_pages_the_principals_tasks_in_admission_order() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let other = Principal::new(1001, "operator").map_err(|error| format!("{error:?}"))?;
    let mut ids = Vec::new();
    for n in 1..=3 {
        let reply = serve(&tasks, &operator, &submit_nth(n))?;
        ids.push(
            reply["body"]["task"]["task_id"]
                .as_str()
                .ok_or("task id")?
                .to_owned(),
        );
    }
    serve(&tasks, &other, &submit_nth(9))?;
    for n in 4..=5 {
        let reply = serve(&tasks, &operator, &submit_nth(n))?;
        ids.push(
            reply["body"]["task"]["task_id"]
                .as_str()
                .ok_or("task id")?
                .to_owned(),
        );
    }
    let snapshot = high_water(&scratch)?.to_string();
    assert_eq!(snapshot, "6", "six admissions, one event each");
    let heads = ids
        .iter()
        .map(|id| head_of(&tasks, &operator, id))
        .collect::<Result<Vec<_>, _>>()?;
    let filter = list_filter(&[], None, None);
    let first = serve(
        &tasks,
        &operator,
        &list_frame(1, &unfiltered(2, &Value::Null)),
    )?;
    // The after_key names the ledger epoch and the admission sequence, so a cursor from another
    // ledger (a restore) can be told apart.
    let cursor = |after: &str| {
        json!({"snapshot_revision": snapshot, "after_key": format!("{EPOCH}.{after}"), "filter_sha256": filter,
               "expires_unix_ms": (NOW + 300_000).to_string()})
    };
    assert_eq!(
        first["body"],
        json!({"page": {"items": [heads[0], heads[1]], "next_cursor": cursor("2"),
                        "snapshot_revision": snapshot}}),
        "{first}"
    );
    // A task admitted after the listing began is not a member of it. The other principal's admission
    // is the fourth event, so this principal's fourth task is the fifth.
    serve(&tasks, &operator, &submit_nth(6))?;
    let second = serve(
        &tasks,
        &operator,
        &list_frame(2, &unfiltered(2, &cursor("2"))),
    )?;
    assert_eq!(
        second["body"],
        json!({"page": {"items": [heads[2], heads[3]], "next_cursor": cursor("5"),
                        "snapshot_revision": snapshot}}),
        "{second}"
    );
    let third = serve(
        &tasks,
        &operator,
        &list_frame(3, &unfiltered(2, &cursor("5"))),
    )?;
    assert_eq!(
        third["body"],
        json!({"page": {"items": [heads[4]], "next_cursor": null, "snapshot_revision": snapshot}}),
        "{third}"
    );
    assert_eq!(
        (&third["kind"], &third["effect"], &third["replayed"]),
        (&json!("result"), &json!("none"), &json!(false))
    );
    // Another principal sees its own one task, and none of these.
    let theirs = serve(
        &tasks,
        &other,
        &list_frame(4, &unfiltered(100, &Value::Null)),
    )?;
    let items = theirs["body"]["page"]["items"].as_array().ok_or("items")?;
    assert_eq!(items.len(), 1, "{theirs}");
    assert!(
        !ids.iter().any(|id| items[0]["task_id"] == json!(id)),
        "{theirs}"
    );
    conforms(&[
        ("task.list", &first),
        ("task.list", &second),
        ("task.list", &third),
        ("task.list", &theirs),
    ])?;
    Ok(())
}

/// B06: the filters. `states` selects by the task's current state (empty selects every state);
/// `task_class` by the class the task was admitted under, read from its admitted request; a
/// `parent_task_id` selects the tasks admitted under that parent, which no task has yet.
#[test]
fn a_list_filters_by_state_class_and_parent() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let mut ids = Vec::new();
    for n in 1..=3 {
        let reply = serve(&tasks, &operator, &submit_nth(n))?;
        ids.push(
            reply["body"]["task"]["task_id"]
                .as_str()
                .ok_or("task id")?
                .to_owned(),
        );
    }
    let cancelled = serve(
        &tasks,
        &operator,
        &cancel_frame(
            4,
            CANCEL_KEY,
            &ids[1],
            "1",
            &json!({"reason": "superseded", "note": null}),
        )?,
    )?;
    assert_eq!(cancelled["observed_generation"], json!("2"), "{cancelled}");
    let listed = |no: u8, states: Value, class: Value, parent: Value| {
        let reply = serve(
            &tasks,
            &operator,
            &list_frame(
                no,
                &json!({"states": states, "task_class": class, "parent_task_id": parent,
                        "page": {"limit": 100, "cursor": null}}),
            ),
        )?;
        let ids: Vec<String> = reply["body"]["page"]["items"]
            .as_array()
            .ok_or("items")?
            .iter()
            .map(|item| item["task_id"].as_str().unwrap_or("?").to_owned())
            .collect();
        Ok::<_, Box<dyn Error>>((ids, reply))
    };
    let all = ids.clone();
    let mut replies = Vec::new();
    for (no, states, class, parent, expected) in [
        (10, json!([]), Value::Null, Value::Null, all.clone()),
        (
            11,
            json!(["cancellation_requested"]),
            Value::Null,
            Value::Null,
            vec![ids[1].clone()],
        ),
        (
            12,
            json!(["admitted"]),
            Value::Null,
            Value::Null,
            vec![ids[0].clone(), ids[2].clone()],
        ),
        (
            13,
            json!(["admitted", "cancellation_requested"]),
            Value::Null,
            Value::Null,
            all.clone(),
        ),
        (14, json!(["accepted"]), Value::Null, Value::Null, vec![]),
        (
            15,
            json!([]),
            json!("rust-library-change/1"),
            Value::Null,
            all.clone(),
        ),
        (16, json!([]), json!("other-class/1"), Value::Null, vec![]),
        (17, json!([]), Value::Null, json!(ids[0]), vec![]),
    ] {
        let (got, reply) = listed(no, states.clone(), class.clone(), parent.clone())?;
        assert_eq!(
            got, expected,
            "states {states} class {class} parent {parent}: {reply}"
        );
        replies.push(reply);
    }
    let rows: Vec<(&str, &Value)> = replies.iter().map(|reply| ("task.list", reply)).collect();
    conforms(&rows)?;
    Ok(())
}

/// B06: a cursor is a selector over one listing, never a grant (RC03 §4). It resumes only the filter
/// it was issued for (`invalid_argument` at `filter_sha256` otherwise), only before it expires, and
/// only over the ledger it was issued from: an expired cursor and one naming a snapshot beyond the
/// ledger's high-water are `resync_required`. A malformed one is refused by member.
#[test]
fn a_list_cursor_resumes_only_its_own_filter_snapshot_and_lifetime() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    for n in 1..=3 {
        serve(&tasks, &operator, &submit_nth(n))?;
    }
    let first = serve(
        &tasks,
        &operator,
        &list_frame(1, &unfiltered(1, &Value::Null)),
    )?;
    let cursor = first["body"]["page"]["next_cursor"].clone();
    assert_eq!(cursor["after_key"], json!(format!("{EPOCH}.1")), "{first}");
    let with = |edit: &dyn Fn(&mut Value)| {
        let mut cursor = cursor.clone();
        edit(&mut cursor);
        cursor
    };
    let mut replies = vec![first.clone()];
    for (case, body, now, code, field) in [
        (
            "another filter",
            json!({"states": ["admitted"], "task_class": null, "parent_task_id": null,
                   "page": {"limit": 1, "cursor": cursor}}),
            NOW,
            "invalid_argument",
            "/body/page/cursor/filter_sha256",
        ),
        (
            "expired",
            unfiltered(1, &cursor),
            NOW + 300_000,
            "resync_required",
            "/body/page/cursor/expires_unix_ms",
        ),
        (
            "a snapshot the ledger never reached",
            unfiltered(1, &with(&|c| c["snapshot_revision"] = json!("4"))),
            NOW,
            "resync_required",
            "/body/page/cursor/snapshot_revision",
        ),
        (
            "an unknown member",
            unfiltered(1, &with(&|c| c["extra"] = json!(1))),
            NOW,
            "invalid_argument",
            "/body/page/cursor",
        ),
    ] {
        let reply = serve_at(&tasks, &operator, &list_frame_at(2, &body, now)?, now)?;
        assert_eq!(
            (&reply["code"], &reply["details"]["field"]),
            (&json!(code), &json!(field)),
            "{case}: {reply}"
        );
        replies.push(reply);
    }
    // The cursor still resumes its own listing, one millisecond before it expires.
    let resumed = serve_at(
        &tasks,
        &operator,
        &list_frame_at(3, &unfiltered(1, &cursor), NOW + 299_999)?,
        NOW + 299_999,
    )?;
    assert_eq!(
        resumed["body"]["page"]["next_cursor"]["after_key"],
        json!(format!("{EPOCH}.2")),
        "{resumed}"
    );
    replies.push(resumed);
    let rows: Vec<(&str, &Value)> = replies.iter().map(|reply| ("task.list", reply)).collect();
    conforms(&rows)?;
    Ok(())
}

/// `task.list` bodies, each one member away from a valid body, with the member each must be
/// refused at (`None`: valid).
/// Every task state, in the contract's order (`TaskStateV1`), written here from the contract.
const TASK_STATES: [&str; 12] = [
    "admitted",
    "queued",
    "running",
    "verifying",
    "repair_pending",
    "cancellation_requested",
    "blocked",
    "accepted",
    "failed",
    "cancelled",
    "abandoned",
    "effect_unknown",
];

/// A valid unfiltered `task.list` body with the member at `path` replaced by `value`.
fn list_body_with(path: &[&str], value: Value) -> Value {
    let mut body = unfiltered(100, &Value::Null);
    let mut target = &mut body;
    for step in &path[..path.len() - 1] {
        target = &mut target[*step];
    }
    target[path[path.len() - 1]] = value;
    body
}

fn list_body_cases() -> Vec<(u8, &'static str, Value, Option<&'static str>)> {
    let with = list_body_with;
    vec![
        (
            1,
            "an extra member",
            with(&["extra"], json!(1)),
            Some("/body"),
        ),
        (
            2,
            "states not an array",
            with(&["states"], json!("admitted")),
            Some("/body/states"),
        ),
        (
            3,
            "an unknown state",
            with(&["states"], json!(["done"])),
            Some("/body/states"),
        ),
        (
            4,
            "a repeated state",
            with(&["states"], json!(["queued", "queued"])),
            Some("/body/states"),
        ),
        (
            5,
            "every state",
            with(&["states"], json!(TASK_STATES)),
            None,
        ),
        (
            6,
            "an empty class",
            with(&["task_class"], json!("")),
            Some("/body/task_class"),
        ),
        (
            7,
            "a 65-byte class",
            with(&["task_class"], json!("c".repeat(65))),
            Some("/body/task_class"),
        ),
        (
            8,
            "a 64-byte class",
            with(&["task_class"], json!("c".repeat(64))),
            None,
        ),
        (
            9,
            "a non-ASCII class",
            with(&["task_class"], json!("é")),
            Some("/body/task_class"),
        ),
        (
            10,
            "a parent that is not a UuidV4",
            with(&["parent_task_id"], json!("p")),
            Some("/body/parent_task_id"),
        ),
        (
            11,
            "a zero limit",
            with(&["page", "limit"], json!(0)),
            Some("/body/page/limit"),
        ),
        (
            12,
            "a limit of 101",
            with(&["page", "limit"], json!(101)),
            Some("/body/page/limit"),
        ),
        (13, "a limit of 1", with(&["page", "limit"], json!(1)), None),
        (
            14,
            "a page member too many",
            with(&["page", "extra"], json!(1)),
            Some("/body/page"),
        ),
        (
            15,
            "a cursor that is not an object",
            with(&["page", "cursor"], json!("c")),
            Some("/body/page/cursor"),
        ),
    ]
}

/// B06: the body, member by member (contract-decisions.md:342): `states` a duplicate-free array of
/// at most 12 task states; `task_class` null or ASCII of 1..64 bytes; `parent_task_id` null or a
/// `UuidV4`; `page` exactly a limit of 1..100 and a cursor.
#[test]
fn a_list_body_is_checked_member_by_member() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let mut replies = Vec::new();
    for (no, case, body, field) in list_body_cases() {
        let reply = serve(&tasks, &operator, &list_frame(no, &body))?;
        match field {
            Some(field) => assert_eq!(
                (&reply["code"], &reply["details"]["field"]),
                (&json!("invalid_argument"), &json!(field)),
                "{case}: {reply}"
            ),
            None => assert_eq!(reply["kind"], json!("result"), "{case}: {reply}"),
        }
        replies.push(reply);
    }
    let rows: Vec<(&str, &Value)> = replies.iter().map(|reply| ("task.list", reply)).collect();
    conforms(&rows)?;
    Ok(())
}

/// B06: a task whose admitted bytes are not a JSON request (one staged through the store's own API,
/// never the wire) is listed like any other -- with the head `task.get` reads, here generation 2
/// with its queued attempt current -- and simply has no class or parent to select it by: a class
/// filter passes over it rather than failing the listing. A last page that is exactly full issues
/// no cursor.
#[test]
fn a_list_passes_over_a_task_whose_admitted_bytes_name_no_class() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let mut store = raw_store(&scratch)?;
    begun(&mut store, &operator, 1)?;
    let tasks = StoreTasks::new(store, EPOCH.to_owned());
    let admitted = serve(&tasks, &operator, &submit_nth(1))?;
    let wire_task = admitted["body"]["task"]["task_id"]
        .as_str()
        .ok_or("task id")?
        .to_owned();
    let staged = nth(0x05b1, 1);
    let staged_head = head_of(&tasks, &operator, &staged)?;
    assert_eq!(
        (
            &staged_head["generation"],
            &staged_head["current_attempt_id"]
        ),
        (&json!("2"), &json!(nth(0x05b2, 1))),
        "{staged_head}"
    );
    // Exactly a full page with nothing after it: no cursor to an empty page.
    let all = serve(
        &tasks,
        &operator,
        &list_frame(1, &unfiltered(2, &Value::Null)),
    )?;
    assert_eq!(
        all["body"]["page"],
        json!({"items": [staged_head, head_of(&tasks, &operator, &wire_task)?],
               "next_cursor": null, "snapshot_revision": "3"}),
        "{all}"
    );
    let classed = serve(
        &tasks,
        &operator,
        &list_frame(
            2,
            &json!({"states": [], "task_class": "rust-library-change/1", "parent_task_id": null,
                    "page": {"limit": 100, "cursor": null}}),
        ),
    )?;
    let ids: Vec<&Value> = classed["body"]["page"]["items"]
        .as_array()
        .ok_or("items")?
        .iter()
        .map(|item| &item["task_id"])
        .collect();
    assert_eq!(ids, [&json!(wire_task)], "{classed}");
    conforms(&[("task.list", &all), ("task.list", &classed)])?;
    Ok(())
}

/// B06: a filter is one filter whatever order its states are named in. A cursor issued for
/// `[cancellation_requested, admitted]` binds the digest computed here independently (the states
/// sorted), and resumes the same listing named `[admitted, cancellation_requested]` -- but not
/// under another class or another parent, which are the filter too.
#[test]
fn a_list_filter_is_one_filter_whatever_the_order_of_its_states() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    for n in 1..=3 {
        serve(&tasks, &operator, &submit_nth(n))?;
    }
    let both = |no: u8, states: Value, cursor: &Value| {
        serve(
            &tasks,
            &operator,
            &list_frame(
                no,
                &json!({"states": states, "task_class": null, "parent_task_id": null,
                        "page": {"limit": 1, "cursor": cursor}}),
            ),
        )
    };
    let named = both(
        4,
        json!(["cancellation_requested", "admitted"]),
        &Value::Null,
    )?;
    let issued = named["body"]["page"]["next_cursor"].clone();
    assert_eq!(
        issued["filter_sha256"],
        json!(list_filter(
            &["cancellation_requested", "admitted"],
            None,
            None
        )),
        "{named}"
    );
    let reordered = both(5, json!(["admitted", "cancellation_requested"]), &issued)?;
    assert_eq!(
        reordered["body"]["page"]["next_cursor"]["after_key"],
        json!(format!("{EPOCH}.2")),
        "{reordered}"
    );
    // The class and the parent are the filter too: the same cursor presented under another class or
    // another parent is refused at its filter digest.
    let mut refused = Vec::new();
    for (no, class, parent) in [
        (6, json!("rust-library-change/1"), Value::Null),
        (
            7,
            Value::Null,
            json!("28d00000-0000-4000-8000-0000000000cc"),
        ),
    ] {
        let reply = serve(
            &tasks,
            &operator,
            &list_frame(
                no,
                &json!({"states": ["admitted", "cancellation_requested"], "task_class": class,
                        "parent_task_id": parent, "page": {"limit": 1, "cursor": issued}}),
            ),
        )?;
        assert_eq!(
            (&reply["code"], &reply["details"]["field"]),
            (
                &json!("invalid_argument"),
                &json!("/body/page/cursor/filter_sha256")
            ),
            "class {class} parent {parent}: {reply}"
        );
        refused.push(reply);
    }
    conforms(&[
        ("task.list", &named),
        ("task.list", &reordered),
        ("task.list", &refused[0]),
        ("task.list", &refused[1]),
    ])?;
    Ok(())
}

/// B06 (review 2): a listing is consistent with its snapshot. A continuation is refused
/// `resync_required` once any member it has yet to list has changed after the snapshot, so every
/// item a listing shows is its state at the snapshot and every filter is applied at it. A change to
/// a member already listed, or a task admitted after the snapshot (not a member), does not refuse.
#[test]
fn a_list_continuation_refuses_a_snapshot_its_members_moved_past() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let mut ids = Vec::new();
    for n in 1..=3 {
        let reply = serve(&tasks, &operator, &submit_nth(n))?;
        ids.push(
            reply["body"]["task"]["task_id"]
                .as_str()
                .ok_or("task id")?
                .to_owned(),
        );
    }
    let why = json!({"reason": "superseded", "note": null});
    let other = Principal::new(1001, "operator").map_err(|error| format!("{error:?}"))?;
    let admitted = serve(&tasks, &other, &submit_nth(9))?;
    let theirs = admitted["body"]["task"]["task_id"]
        .as_str()
        .ok_or("task id")?
        .to_owned();
    let first = serve(
        &tasks,
        &operator,
        &list_frame(1, &unfiltered(1, &Value::Null)),
    )?;
    let cursor = first["body"]["page"]["next_cursor"].clone();
    // The listed member moves, another principal's task moves, and a non-member is admitted: the
    // listing still continues.
    serve(
        &tasks,
        &other,
        &cancel_frame(6, CANCEL_KEY, &theirs, "1", &why)?,
    )?;
    serve(
        &tasks,
        &operator,
        &cancel_frame(2, CANCEL_KEY, &ids[0], "1", &why)?,
    )?;
    serve(&tasks, &operator, &submit_nth(4))?;
    let second = serve(&tasks, &operator, &list_frame(3, &unfiltered(1, &cursor)))?;
    assert_eq!(
        second["body"]["page"]["items"][0]["task_id"],
        json!(ids[1]),
        "{second}"
    );
    // A member not yet listed moves: the snapshot no longer describes it.
    let later = second["body"]["page"]["next_cursor"].clone();
    serve(
        &tasks,
        &operator,
        &cancel_frame(4, CANCEL_KEY_2, &ids[2], "1", &why)?,
    )?;
    let refused = serve(&tasks, &operator, &list_frame(5, &unfiltered(1, &later)))?;
    assert_eq!(
        (&refused["code"], &refused["details"]["field"]),
        (
            &json!("resync_required"),
            &json!("/body/page/cursor/snapshot_revision")
        ),
        "{refused}"
    );
    conforms(&[
        ("task.list", &first),
        ("task.list", &second),
        ("task.list", &refused),
    ])?;
    Ok(())
}

/// B06 (review 2): the parent selects the tasks whose admitted request names it (staged through
/// the store's own API, since the wire does not yet admit a parent), and a listing is scoped by the
/// whole principal: the same uid under another role lists none of these tasks.
#[test]
fn a_list_selects_by_parent_and_scopes_by_the_whole_principal() -> Outcome {
    use habitat_engine::store::{Allocation, Submission};
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let reviewer = Principal::new(1000, "reviewer").map_err(|error| format!("{error:?}"))?;
    let parent = "28d00000-0000-4000-8000-0000000008aa";
    let mut store = raw_store(&scratch)?;
    let until = Instant::now() + Duration::from_secs(10);
    let criteria_text = format!("sha256:{}", "5".repeat(64));
    let criteria = habitat_engine::contracts::Sha256Digest::parse(&criteria_text)?;
    for (index, bytes) in [
        (
            1_u16,
            json!({"body": {"spec": {"task_class": "rust-library-change/1",
                                         "parent": {"task_id": parent}}}}),
        ),
        (
            2,
            json!({"body": {"spec": {"task_class": "rust-library-change/1", "parent": null}}}),
        ),
    ] {
        let request_bytes = serde_json::to_vec(&bytes)?;
        store
            .submit(
                Submission {
                    principal: &operator,
                    key: UuidV4::parse(&nth(0x08b0, index))?,
                    task: UuidV4::parse(&nth(0x08b1, index))?,
                    event: UuidV4::parse(&nth(0x08b2, index))?,
                    request_bytes: &request_bytes,
                    criteria,
                    allocation: Allocation {
                        limit_ms: 1_200_000,
                        work_ms: 900_000,
                        verify_ms: 300_000,
                    },
                },
                until,
            )
            .map_err(|error| format!("{error:?}"))?;
    }
    let tasks = StoreTasks::new(store, EPOCH.to_owned());
    let listed = |principal: &Principal, no: u8, parent: Value| {
        let reply = serve(
            &tasks,
            principal,
            &list_frame(
                no,
                &json!({"states": [], "task_class": null, "parent_task_id": parent,
                        "page": {"limit": 100, "cursor": null}}),
            ),
        )?;
        let ids: Vec<String> = reply["body"]["page"]["items"]
            .as_array()
            .ok_or("items")?
            .iter()
            .map(|item| item["task_id"].as_str().unwrap_or("?").to_owned())
            .collect();
        Ok::<_, Box<dyn Error>>((ids, reply))
    };
    let (children, by_parent) = listed(&operator, 1, json!(parent))?;
    assert_eq!(children, [nth(0x08b1, 1)], "{by_parent}");
    let (all, unfiltered_reply) = listed(&operator, 2, Value::Null)?;
    assert_eq!(all, [nth(0x08b1, 1), nth(0x08b1, 2)], "{unfiltered_reply}");
    let (theirs, other_role) = listed(&reviewer, 3, Value::Null)?;
    assert!(theirs.is_empty(), "{other_role}");
    conforms(&[
        ("task.list", &by_parent),
        ("task.list", &unfiltered_reply),
        ("task.list", &other_role),
    ])?;
    Ok(())
}

/// B06: the `after_key` a listing issues names its ledger epoch and an admission sequence. One from
/// another epoch (a restore) is `resync_required`; sequence 0, a key past the cursor's own snapshot and
/// one that is not `<epoch>.<sequence>` are `invalid_argument`, all at `/body/page/cursor/after_key`.
#[test]
fn a_list_after_key_names_its_epoch_and_an_issued_sequence() -> Outcome {
    let scratch = Scratch::new()?;
    let tasks = ledger(&scratch)?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    for n in 1..=3 {
        serve(&tasks, &operator, &submit_nth(n))?;
    }
    let first = serve(
        &tasks,
        &operator,
        &list_frame(1, &unfiltered(1, &Value::Null)),
    )?;
    let cursor = first["body"]["page"]["next_cursor"].clone();
    let with = |edit: &dyn Fn(&mut Value)| {
        let mut cursor = cursor.clone();
        edit(&mut cursor);
        cursor
    };
    let mut replies = Vec::new();
    for (case, body, now, code, field) in [
        (
            "another ledger's epoch",
            unfiltered(
                1,
                &with(&|c| c["after_key"] = json!("28d00000-0000-4000-8000-00000000ffff.1")),
            ),
            NOW,
            "resync_required",
            "/body/page/cursor/after_key",
        ),
        (
            "sequence 0, which no listing issues",
            unfiltered(1, &with(&|c| c["after_key"] = json!(format!("{EPOCH}.0")))),
            NOW,
            "invalid_argument",
            "/body/page/cursor/after_key",
        ),
        (
            "a key above its own snapshot",
            unfiltered(1, &with(&|c| c["after_key"] = json!(format!("{EPOCH}.4")))),
            NOW,
            "invalid_argument",
            "/body/page/cursor/after_key",
        ),
        (
            "an after_key that is not a sequence",
            unfiltered(1, &with(&|c| c["after_key"] = json!("task-1"))),
            NOW,
            "invalid_argument",
            "/body/page/cursor/after_key",
        ),
    ] {
        let reply = serve_at(&tasks, &operator, &list_frame_at(2, &body, now)?, now)?;
        assert_eq!(
            (&reply["code"], &reply["details"]["field"]),
            (&json!(code), &json!(field)),
            "{case}: {reply}"
        );
        replies.push(reply);
    }
    let rows: Vec<(&str, &Value)> = replies.iter().map(|reply| ("task.list", reply)).collect();
    conforms(&rows)?;
    Ok(())
}

// --- B08 · task.resolve (contract-decisions.md:344; design ~/hee3-evidence/T28/B08-task-resolve-*/DESIGN.md) ---

const RESOLVE_KEY: &str = "28d00000-0000-4000-8000-0000000009a1";
const RESOLVE_KEY_2: &str = "28d00000-0000-4000-8000-0000000009a2";
const RESOLVE_KEY_3: &str = "28d00000-0000-4000-8000-0000000009a3";

/// A `task.resolve` frame: the precondition names the task and the generation the operator expects.
fn resolve_frame(
    request_no: u8,
    key: &str,
    task: &str,
    generation: &str,
    body: &Value,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut frame: Value =
        serde_json::from_slice(&request("task.resolve", request_no, Some(key), body))?;
    frame["precondition"] = json!({"resource": "task", "id": task, "generation": generation});
    Ok(serde_json::to_vec(&frame)?)
}

fn resolve_body(obligation: &str, disposition: &str, evidence: &Value) -> Value {
    json!({"obligation_id": obligation, "disposition": disposition,
           "reason": "operator reviewed the worker's lost reply", "evidence": evidence})
}

/// An `EvidenceRefV1` naming `object`, stored in the ledger.
fn evidence_of(object: &habitat_engine::store::Object) -> Value {
    json!({"artifact_id": "28d00000-0000-4000-8000-0000000009e1", "sha256": object.digest(),
           "byte_length": object.size(), "media_type": "text/plain", "schema_id": "hee3.evidence/1"})
}

/// A ledger query answered from the ledger file itself, not through the engine.
fn ledger_value(scratch: &Scratch, sql: &str, task: &str) -> Result<Value, Box<dyn Error>> {
    let file = scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("ledger.sqlite3");
    let db =
        rusqlite::Connection::open_with_flags(file, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let value: rusqlite::types::Value = db.query_row(sql, [task], |row| row.get(0))?;
    Ok(match value {
        rusqlite::types::Value::Null => Value::Null,
        rusqlite::types::Value::Integer(n) => json!(n),
        rusqlite::types::Value::Text(t) => json!(t),
        other => json!(format!("{other:?}")),
    })
}

/// A ledger of staged tasks (`stages[i]` is task `i + 1`) and a published evidence object.
fn resolve_ledger(
    scratch: &Scratch,
    operator: &Principal,
    stages: &[Stage],
) -> Result<(StoreTasks, Vec<String>, habitat_engine::store::Object), Box<dyn Error>> {
    let mut store = raw_store(scratch)?;
    let mut tasks = Vec::new();
    for (index, stage) in stages.iter().enumerate() {
        tasks.push(staged(
            &mut store,
            operator,
            u16::try_from(index + 1)?,
            *stage,
        )?);
    }
    let evidence = store
        .publish(
            b"operator's reconciliation note",
            UuidV4::parse(EPOCH)?,
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?;
    Ok((StoreTasks::new(store, EPOCH.to_owned()), tasks, evidence))
}

/// Serve one `task.resolve` as `principal`: `(request number, key, task, expected generation)`,
/// then the obligation, the disposition and the evidence references.
fn resolve_with(
    tasks: &StoreTasks,
    principal: &Principal,
    (no, key, task, generation): (u8, &str, &str, &str),
    obligation: &str,
    disposition: &str,
    evidence: &Value,
) -> Result<Value, Box<dyn Error>> {
    let body = resolve_body(obligation, disposition, evidence);
    serve(
        tasks,
        principal,
        &resolve_frame(no, key, task, generation, &body)?,
    )
}

fn code_at(reply: &Value) -> (&Value, &Value) {
    (&reply["code"], &reply["details"]["field"])
}

/// One refusal case: its name, principal, frame, code and field.
type Refusal<'a> = (&'static str, &'a Principal, Vec<u8>, &'static str, Value);

/// The refusals of `a_resolve_needs_...`: each case's principal, frame, code and field.
fn resolve_refusals<'a>(
    operator: &'a Principal,
    reviewer: &'a Principal,
    stranger: &'a Principal,
    ids: &[String],
    intent: &str,
) -> Result<Vec<Refusal<'a>>, Box<dyn Error>> {
    let body = |obligation: &str| resolve_body(obligation, "quarantine", &json!([]));
    let unknown = body(&nth(0x05b2, 1));
    Ok(vec![
        (
            "another role",
            reviewer,
            resolve_frame(1, RESOLVE_KEY, &ids[0], "3", &unknown)?,
            "forbidden",
            Value::Null,
        ),
        (
            "another operator's task",
            stranger,
            resolve_frame(8, RESOLVE_KEY, &ids[0], "3", &unknown)?,
            "not_found",
            json!("/precondition/id"),
        ),
        // Before anything else about the task: an invisible task names no obligation either.
        (
            "another operator's task, a bogus obligation",
            stranger,
            resolve_frame(
                10,
                RESOLVE_KEY,
                &ids[0],
                "3",
                &body("28d00000-0000-4000-8000-0000000009fd"),
            )?,
            "not_found",
            json!("/precondition/id"),
        ),
        (
            "an unknown task",
            operator,
            resolve_frame(
                2,
                RESOLVE_KEY,
                "28d00000-0000-4000-8000-0000000009ff",
                "3",
                &unknown,
            )?,
            "not_found",
            json!("/precondition/id"),
        ),
        (
            "a stale generation",
            operator,
            resolve_frame(3, RESOLVE_KEY, &ids[0], "2", &unknown)?,
            "stale_generation",
            json!("/precondition/generation"),
        ),
        (
            "an id the task does not hold",
            operator,
            resolve_frame(
                4,
                RESOLVE_KEY,
                &ids[0],
                "3",
                &body("28d00000-0000-4000-8000-0000000009fe"),
            )?,
            "not_found",
            json!("/body/obligation_id"),
        ),
        (
            "a live attempt",
            operator,
            resolve_frame(5, RESOLVE_KEY, &ids[1], "3", &body(&nth(0x05b2, 2)))?,
            "conflict",
            json!("/body/obligation_id"),
        ),
        (
            "a cancellation intent",
            operator,
            resolve_frame(6, RESOLVE_KEY, &ids[1], "3", &body(intent))?,
            "not_found",
            json!("/body/obligation_id"),
        ),
        (
            "a settled attempt",
            operator,
            resolve_frame(7, RESOLVE_KEY, &ids[2], "3", &body(&nth(0x05b2, 3)))?,
            "not_found",
            json!("/body/obligation_id"),
        ),
    ])
}

/// B08: who may resolve what. Only the operator role; only a task it can see, at the generation it
/// expects; only an obligation the task holds -- a live attempt is the cancel's (`conflict`), and a
/// settled attempt, a cancellation intent or an unknown id is `not_found` at the obligation.
#[test]
fn a_resolve_needs_the_operator_the_task_its_generation_and_an_open_obligation() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let reviewer = Principal::new(1000, "reviewer").map_err(|error| format!("{error:?}"))?;
    let stranger = Principal::new(1001, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, _) = resolve_ledger(
        &scratch,
        &operator,
        &[Stage::Unknown, Stage::Running, Stage::Settled],
    )?;
    let cancelled = serve(
        &tasks,
        &operator,
        &cancel_frame(
            9,
            CANCEL_KEY,
            &ids[1],
            "2",
            &json!({"reason": "safety", "note": null}),
        )?,
    )?;
    let intent = cancelled["body"]["cancellation_obligation_id"]
        .as_str()
        .ok_or("obligation")?;
    let mut replies = Vec::new();
    for (case, principal, frame, code, field) in
        resolve_refusals(&operator, &reviewer, &stranger, &ids, intent)?
    {
        let reply = serve(&tasks, principal, &frame)?;
        assert_eq!(code_at(&reply), (&json!(code), &field), "{case}: {reply}");
        replies.push(reply);
    }
    // Nothing was written: the unknown-effect task is where its settlement left it.
    assert_eq!(
        head_of(&tasks, &operator, &ids[0])?["generation"],
        json!("3")
    );
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|reply| ("task.resolve", reply))
        .collect();
    conforms(&rows)?;
    Ok(())
}

/// B08: an unknown effect is quarantined or acknowledged, never replayed. `retry` is refused while
/// the effect is unknown; `abandon` is refused until the effect is acknowledged. Quarantine holds
/// the task `blocked`, closing nothing; acknowledgement over an unknown cleanup leaves the
/// obligation pending. Each disposition is a durable row of its own, naming what was decided.
#[test]
fn an_unknown_effect_is_quarantined_or_acknowledged_never_replayed() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(&scratch, &operator, &[Stage::Unknown])?;
    let (task, attempt) = (ids[0].as_str(), nth(0x05b2, 1));
    let refs = json!([evidence_of(&evidence)]);
    let mut replies = Vec::new();
    for (no, disposition) in [(1, "retry"), (2, "abandon")] {
        let refused = resolve_with(
            &tasks,
            &operator,
            (no, RESOLVE_KEY, task, "3"),
            &attempt,
            disposition,
            &refs,
        )?;
        assert_eq!(
            code_at(&refused),
            (&json!("conflict"), &json!("/body/disposition")),
            "{disposition}: {refused}"
        );
        replies.push(refused);
    }
    let quarantined = resolve_with(
        &tasks,
        &operator,
        (3, RESOLVE_KEY, task, "3"),
        &attempt,
        "quarantine",
        &json!([]),
    )?;
    let head = &quarantined["body"]["task"];
    assert_eq!(
        (
            &head["state"],
            &head["generation"],
            &quarantined["body"]["obligation_state"],
            &head["unresolved_obligations"]
        ),
        (&json!("blocked"), &json!("4"), &json!("pending"), &json!(1)),
        "{quarantined}"
    );
    let disposition_id = quarantined["body"]["disposition_id"]
        .as_str()
        .ok_or("disposition id")?;
    UuidV4::parse(disposition_id)?;
    assert_eq!(&head_of(&tasks, &operator, task)?, head);
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT disposition||'/'||obligation_kind||'/'||resolves FROM task_dispositions WHERE id=?",
            disposition_id
        )?,
        json!("quarantine/attempt/0")
    );
    let acknowledged = resolve_with(
        &tasks,
        &operator,
        (4, RESOLVE_KEY_2, task, "4"),
        &attempt,
        "acknowledge_external_effect",
        &refs,
    )?;
    let head = &acknowledged["body"]["task"];
    assert_eq!(
        (
            &head["state"],
            &acknowledged["body"]["obligation_state"],
            &head["unresolved_obligations"]
        ),
        (&json!("blocked"), &json!("pending"), &json!(1)),
        "cleanup is still unknown: {acknowledged}"
    );
    replies.extend([quarantined, acknowledged]);
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|reply| ("task.resolve", reply))
        .collect();
    conforms(&rows)?;
    Ok(())
}

/// B08: once every unknown effect of the task is acknowledged, abandonment stops it through the one
/// stop door -- `abandoned`, a stop row naming the operator's stored evidence -- keeping the
/// reservation, because usage is unknown; and a cancel after it is refused (the outcome is decided).
#[test]
fn an_acknowledged_task_is_abandoned_through_the_stop_door() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(&scratch, &operator, &[Stage::Unknown])?;
    let (task, attempt) = (ids[0].as_str(), nth(0x05b2, 1));
    let refs = json!([evidence_of(&evidence)]);
    let reserved = ledger_value(
        &scratch,
        "SELECT reserved_work_ms FROM tasks WHERE id=?",
        task,
    )?;
    resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, task, "3"),
        &attempt,
        "acknowledge_external_effect",
        &refs,
    )?;
    let abandoned = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, task, "4"),
        &attempt,
        "abandon",
        &refs,
    )?;
    assert_eq!(
        (
            &abandoned["body"]["task"]["state"],
            &abandoned["body"]["obligation_state"],
            &abandoned["body"]["task"]["generation"]
        ),
        (&json!("abandoned"), &json!("pending"), &json!("6")),
        "the disposition, then the stop: {abandoned}"
    );
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT state||'/'||evidence_digest FROM task_stops WHERE task_id=?",
            task
        )?,
        json!(format!("abandoned/{}", evidence.digest()))
    );
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT reserved_work_ms FROM tasks WHERE id=?",
            task
        )?,
        reserved,
        "usage is unknown, so the reservation is retained"
    );
    let too_late = serve(
        &tasks,
        &operator,
        &cancel_frame(
            3,
            CANCEL_KEY,
            task,
            "6",
            &json!({"reason": "operator_request", "note": null}),
        )?,
    )?;
    assert_eq!(
        code_at(&too_late),
        (&json!("conflict"), &json!("/precondition")),
        "{too_late}"
    );
    // Its attempts are no longer an operator's to dispose of: the outcome is decided.
    let decided = resolve_with(
        &tasks,
        &operator,
        (4, RESOLVE_KEY_3, task, "6"),
        &attempt,
        "quarantine",
        &json!([]),
    )?;
    assert_eq!(
        code_at(&decided),
        (&json!("conflict"), &json!("/precondition")),
        "{decided}"
    );
    conforms(&[
        ("task.resolve", &abandoned),
        ("task.cancel", &too_late),
        ("task.resolve", &decided),
    ])?;
    Ok(())
}

/// B08: acknowledging an unknown effect whose cleanup settled and usage is known closes that
/// obligation (the head counts 0; the attempt row still records what was observed); a second
/// resolution of it is `conflict` at the obligation. An attempt whose effect was observed but whose
/// usage is unknown has no effect to acknowledge (`invalid_argument`); `retry` records a request.
#[test]
fn an_acknowledged_effect_with_settled_cleanup_closes_its_obligation() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(
        &scratch,
        &operator,
        &[Stage::UnknownEffectOnly, Stage::UsageUnknown],
    )?;
    let refs = json!([evidence_of(&evidence)]);
    let (first, second) = (nth(0x05b2, 1), nth(0x05b2, 2));
    let acknowledge = "acknowledge_external_effect";
    let acknowledged = resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &ids[0], "3"),
        &first,
        acknowledge,
        &refs,
    )?;
    let head = &acknowledged["body"]["task"];
    assert_eq!(
        (
            &acknowledged["body"]["obligation_state"],
            &head["unresolved_obligations"],
            &head["state"],
            &head["generation"]
        ),
        (
            &json!("resolved"),
            &json!(0),
            &json!("effect_unknown"),
            &json!("4")
        ),
        "{acknowledged}"
    );
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT effect FROM attempts WHERE task_id=?",
            &ids[0]
        )?,
        json!("unknown"),
        "the observation is never rewritten"
    );
    let again = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, &ids[0], "4"),
        &first,
        acknowledge,
        &refs,
    )?;
    assert_eq!(
        code_at(&again),
        (&json!("conflict"), &json!("/body/obligation_id")),
        "{again}"
    );
    let nothing = resolve_with(
        &tasks,
        &operator,
        (3, RESOLVE_KEY_2, &ids[1], "3"),
        &second,
        acknowledge,
        &refs,
    )?;
    assert_eq!(
        code_at(&nothing),
        (&json!("invalid_argument"), &json!("/body/disposition")),
        "{nothing}"
    );
    let retried = resolve_with(
        &tasks,
        &operator,
        (4, RESOLVE_KEY_3, &ids[1], "3"),
        &second,
        "retry",
        &json!([]),
    )?;
    assert_eq!(
        (
            &retried["body"]["obligation_state"],
            &retried["body"]["task"]["generation"]
        ),
        (&json!("pending"), &json!("4")),
        "{retried}"
    );
    conforms(&[
        ("task.resolve", &acknowledged),
        ("task.resolve", &again),
        ("task.resolve", &nothing),
        ("task.resolve", &retried),
    ])?;
    Ok(())
}

/// B08: a delivery obligation (an accepted task's undelivered notification, named by its event) is
/// retried or given up. Quarantine and acknowledgement do not apply. Giving it up closes it for the
/// head's count AND the notifier's work list alike; a second disposition of it is `conflict`.
/// The task stays accepted.
#[test]
fn a_delivery_obligation_is_retried_or_given_up_for_every_reader() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, _) = resolve_ledger(&scratch, &operator, &[Stage::Accepted])?;
    let task = &ids[0];
    let event = ledger_value(
        &scratch,
        "SELECT o.event_id FROM outbox o JOIN events e ON e.id=o.event_id WHERE e.task_id=? AND o.delivered=0",
        task,
    )?;
    let event = event.as_str().ok_or("delivery event")?.to_owned();
    let mut replies = Vec::new();
    for (no, disposition) in [(1, "quarantine"), (2, "acknowledge_external_effect")] {
        let reply = serve(
            &tasks,
            &operator,
            &resolve_frame(
                no,
                RESOLVE_KEY,
                task,
                "5",
                &resolve_body(&event, disposition, &json!([])),
            )?,
        )?;
        assert_eq!(
            code_at(&reply),
            (&json!("invalid_argument"), &json!("/body/disposition")),
            "{reply}"
        );
        replies.push(reply);
    }
    let retried = resolve_with(
        &tasks,
        &operator,
        (3, RESOLVE_KEY, task, "5"),
        &event,
        "retry",
        &json!([]),
    )?;
    assert_eq!(
        (
            &retried["body"]["obligation_state"],
            &retried["body"]["task"]["unresolved_obligations"],
            &retried["body"]["task"]["state"]
        ),
        (&json!("pending"), &json!(1), &json!("accepted")),
        "{retried}"
    );
    let given_up = resolve_with(
        &tasks,
        &operator,
        (4, RESOLVE_KEY_2, task, "6"),
        &event,
        "abandon",
        &json!([]),
    )?;
    assert_eq!(
        (
            &given_up["body"]["obligation_state"],
            &given_up["body"]["task"]["unresolved_obligations"],
            &given_up["body"]["task"]["state"]
        ),
        (&json!("resolved"), &json!(0), &json!("accepted")),
        "{given_up}"
    );
    let again = resolve_with(
        &tasks,
        &operator,
        (5, RESOLVE_KEY_3, task, "7"),
        &event,
        "retry",
        &json!([]),
    )?;
    assert_eq!(
        code_at(&again),
        (&json!("conflict"), &json!("/body/obligation_id")),
        "{again}"
    );
    drop(tasks);
    let store = raw_store(&scratch)?;
    let listed = store
        .pending_delivery(256, Instant::now() + Duration::from_secs(10))
        .map_err(|error| format!("{error:?}"))?;
    assert!(
        listed.iter().all(|(id, _, _)| *id != event),
        "the notifier's list: {listed:?}"
    );
    replies.extend([retried, given_up, again]);
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|reply| ("task.resolve", reply))
        .collect();
    conforms(&rows)?;
    Ok(())
}

/// B08: an exact replay returns the stored disposition (inside and after its deadline); other bytes
/// under the key conflict. And an abandonment names at least one artifact the ledger holds.
#[test]
fn a_resolve_replays_exactly_and_abandonment_names_stored_evidence() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(&scratch, &operator, &[Stage::UsageUnknown])?;
    let (task, attempt) = (&ids[0], nth(0x05b2, 1));
    let unstored = json!([{"artifact_id": "28d00000-0000-4000-8000-0000000009e2",
        "sha256": format!("sha256:{}", "7".repeat(64)), "byte_length": 3,
        "media_type": "text/plain", "schema_id": "hee3.evidence/1"}]);
    let mut replies = Vec::new();
    for (no, refs, code, field) in [
        (1, json!([]), "invalid_argument", "/body/evidence"),
        (2, unstored, "not_found", "/body/evidence"),
    ] {
        let reply = serve(
            &tasks,
            &operator,
            &resolve_frame(
                no,
                RESOLVE_KEY,
                task,
                "3",
                &resolve_body(&attempt, "abandon", &refs),
            )?,
        )?;
        assert_eq!(code_at(&reply), (&json!(code), &json!(field)), "{reply}");
        replies.push(reply);
    }
    let frame = resolve_frame(
        3,
        RESOLVE_KEY,
        task,
        "3",
        &resolve_body(&attempt, "abandon", &json!([evidence_of(&evidence)])),
    )?;
    let first = serve(&tasks, &operator, &frame)?;
    assert_eq!(
        (&first["body"]["task"]["state"], &first["replayed"]),
        (&json!("abandoned"), &json!(false)),
        "{first}"
    );
    // The evidence object is gone now: an exact replay answers from its record, never re-reading it.
    let hex = &evidence.digest()[7..];
    let object = scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("objects/sha256")
        .join(&hex[..2]);
    let mut permissions = fs::metadata(&object)?.permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o700);
    fs::set_permissions(&object, permissions)?;
    fs::remove_file(object.join(hex))?;
    for now in [NOW, AN_HOUR_LATE] {
        let mut expected = first.clone();
        expected["replayed"] = json!(true);
        assert_eq!(
            serve_at(&tasks, &operator, &frame, now)?,
            expected,
            "replayed at {now}"
        );
    }
    let other = resolve_with(
        &tasks,
        &operator,
        (4, RESOLVE_KEY, task, "3"),
        &attempt,
        "retry",
        &json!([]),
    )?;
    assert_eq!(
        code_at(&other),
        (&json!("conflict"), &json!("/idempotency_key")),
        "{other}"
    );
    replies.extend([first, other]);
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|reply| ("task.resolve", reply))
        .collect();
    conforms(&rows)?;
    Ok(())
}

/// B08: `blocked` is a waiting state, as `effect_unknown` is. A cancel of a quarantined task keeps
/// it `blocked` (it sets only the intent), so the quarantine is not lifted by renaming it; the
/// intent then decides the stop: abandoning the acknowledged task stops it `cancelled`.
#[test]
fn a_quarantine_survives_a_cancel_and_the_cancel_decides_the_stop() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(&scratch, &operator, &[Stage::Unknown])?;
    let (task, attempt) = (&ids[0], nth(0x05b2, 1));
    let quarantined = resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, task, "3"),
        &attempt,
        "quarantine",
        &json!([]),
    )?;
    assert_eq!(
        quarantined["body"]["task"]["state"],
        json!("blocked"),
        "{quarantined}"
    );
    let cancelled = serve(
        &tasks,
        &operator,
        &cancel_frame(
            2,
            CANCEL_KEY,
            task,
            "4",
            &json!({"reason": "safety", "note": null}),
        )?,
    )?;
    assert_eq!(
        (
            &cancelled["body"]["task"]["state"],
            &cancelled["body"]["task"]["generation"]
        ),
        (&json!("blocked"), &json!("5")),
        "the cancel keeps the quarantine: {cancelled}"
    );
    assert_eq!(
        ledger_value(&scratch, "SELECT cancellation FROM tasks WHERE id=?", task)?,
        json!(1)
    );
    let refs = json!([evidence_of(&evidence)]);
    resolve_with(
        &tasks,
        &operator,
        (3, RESOLVE_KEY_2, task, "5"),
        &attempt,
        "acknowledge_external_effect",
        &refs,
    )?;
    let stopped = resolve_with(
        &tasks,
        &operator,
        (4, RESOLVE_KEY_3, task, "6"),
        &attempt,
        "abandon",
        &refs,
    )?;
    assert_eq!(
        stopped["body"]["task"]["state"],
        json!("cancelled"),
        "{stopped}"
    );
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT state||'/'||reason FROM task_stops WHERE task_id=?",
            task
        )?,
        json!("cancelled/abandoned_by_operator")
    );
    conforms(&[
        ("task.resolve", &quarantined),
        ("task.cancel", &cancelled),
        ("task.resolve", &stopped),
    ])?;
    Ok(())
}

/// B08: the edges of the table. A cancel pending before a quarantine keeps `effect_unknown` (the
/// quarantine never renames a liability away); an abandonment whose every usage is known releases
/// the reservations and charges the unsettled attempt's measured usage.
#[test]
fn a_resolve_keeps_a_pending_cancel_and_releases_known_usage() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(
        &scratch,
        &operator,
        &[Stage::Unknown, Stage::UnknownEffectOnly],
    )?;
    let refs = json!([evidence_of(&evidence)]);
    let cancelled = serve(
        &tasks,
        &operator,
        &cancel_frame(
            1,
            CANCEL_KEY,
            &ids[0],
            "3",
            &json!({"reason": "safety", "note": null}),
        )?,
    )?;
    assert_eq!(
        cancelled["body"]["task"]["state"],
        json!("effect_unknown"),
        "{cancelled}"
    );
    let quarantined = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY, &ids[0], "4"),
        &nth(0x05b2, 1),
        "quarantine",
        &json!([]),
    )?;
    assert_eq!(
        quarantined["body"]["task"]["state"],
        json!("effect_unknown"),
        "{quarantined}"
    );
    let (second, acknowledge) = (nth(0x05b2, 2), "acknowledge_external_effect");
    resolve_with(
        &tasks,
        &operator,
        (3, RESOLVE_KEY_2, &ids[1], "3"),
        &second,
        acknowledge,
        &refs,
    )?;
    let spent = ledger_value(&scratch, "SELECT spent_ms FROM tasks WHERE id=?", &ids[1])?;
    let abandoned = resolve_with(
        &tasks,
        &operator,
        (4, RESOLVE_KEY_3, &ids[1], "4"),
        &second,
        "abandon",
        &refs,
    )?;
    assert_eq!(
        abandoned["body"]["task"]["state"],
        json!("abandoned"),
        "{abandoned}"
    );
    // The attempt measured 10 ms (the fixture's own literal) but never settled, so the spend had not
    // counted it: the stop charges it, and releases the reservations.
    let charged = json!(spent.as_i64().ok_or("spent")? + 10);
    assert_eq!(
        (
            ledger_value(
                &scratch,
                "SELECT reserved_work_ms+reserved_verify_ms FROM tasks WHERE id=?",
                &ids[1]
            )?,
            ledger_value(&scratch, "SELECT spent_ms FROM tasks WHERE id=?", &ids[1])?
        ),
        (json!(0), charged),
        "every usage is known: reservations released, measured usage charged"
    );
    conforms(&[
        ("task.cancel", &cancelled),
        ("task.resolve", &quarantined),
        ("task.resolve", &abandoned),
    ])?;
    Ok(())
}

/// B08: a delivery already acknowledged is no obligation: naming it is `not_found`.
#[test]
fn a_delivered_notification_is_no_obligation() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, _) = resolve_ledger(&scratch, &operator, &[Stage::Accepted])?;
    let event = ledger_value(
        &scratch,
        "SELECT o.event_id FROM outbox o JOIN events e ON e.id=o.event_id WHERE e.task_id=? AND o.delivered=0",
        &ids[0],
    )?;
    let event = event.as_str().ok_or("delivery event")?.to_owned();
    drop(tasks);
    let mut store = raw_store(&scratch)?;
    // The recipient key the store addresses this principal's events to (`<uid>:<role>`).
    store
        .acknowledge_delivery(
            UuidV4::parse(&event)?,
            "1000:operator",
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?;
    let tasks = StoreTasks::new(store, EPOCH.to_owned());
    let delivered = resolve_with(
        &tasks,
        &operator,
        (5, "28d00000-0000-4000-8000-0000000009a4", &ids[0], "5"),
        &event,
        "abandon",
        &json!([]),
    )?;
    assert_eq!(
        code_at(&delivered),
        (&json!("not_found"), &json!("/body/obligation_id")),
        "{delivered}"
    );
    conforms(&[("task.resolve", &delivered)])?;
    Ok(())
}

/// `task.resolve` bodies, each one member away from valid, with the member each is refused at.
fn resolve_body_cases(object: &Value) -> Vec<(&'static str, Value, Option<&'static str>)> {
    let good = resolve_body(&nth(0x05b2, 1), "quarantine", &json!([object]));
    let with = |edit: &dyn Fn(&mut Value)| {
        let mut body = good.clone();
        edit(&mut body);
        body
    };
    vec![
        (
            "an extra member",
            with(&|b| b["extra"] = json!(1)),
            Some("/body"),
        ),
        (
            "an obligation that is not a UuidV4",
            with(&|b| b["obligation_id"] = json!("o")),
            Some("/body/obligation_id"),
        ),
        (
            "an unknown disposition",
            with(&|b| b["disposition"] = json!("forget")),
            Some("/body/disposition"),
        ),
        (
            "an empty reason",
            with(&|b| b["reason"] = json!("")),
            Some("/body/reason"),
        ),
        (
            "a 2049-byte reason",
            with(&|b| b["reason"] = json!("r".repeat(2049))),
            Some("/body/reason"),
        ),
        (
            "a 2048-byte reason",
            with(&|b| b["reason"] = json!("é".repeat(1024))),
            None,
        ),
        // Bytes, not code points: 1,025 two-byte characters are 2,050 bytes.
        (
            "a 2050-byte reason of 1025 characters",
            with(&|b| b["reason"] = json!("é".repeat(1025))),
            Some("/body/reason"),
        ),
        (
            "evidence that is not an array",
            with(&|b| b["evidence"] = json!({})),
            Some("/body/evidence"),
        ),
        (
            "65 references",
            with(&|b| b["evidence"] = json!(vec![object.clone(); 65])),
            Some("/body/evidence"),
        ),
        (
            "64 references",
            with(&|b| b["evidence"] = json!(vec![object.clone(); 64])),
            None,
        ),
        (
            "a reference missing a member",
            with(&|b| {
                b["evidence"][0]
                    .as_object_mut()
                    .map(|o| o.remove("schema_id"));
            }),
            Some("/body/evidence"),
        ),
        (
            "a reference with a bad digest",
            with(&|b| b["evidence"][0]["sha256"] = json!("sha256:zz")),
            Some("/body/evidence"),
        ),
        (
            "a length past u32",
            with(&|b| b["evidence"][0]["byte_length"] = json!(4_294_967_296_u64)),
            Some("/body/evidence"),
        ),
        (
            "a non-ASCII media type",
            with(&|b| b["evidence"][0]["media_type"] = json!("é")),
            Some("/body/evidence"),
        ),
    ]
}

/// B08: the body, member by member (contract-decisions.md:344; `EvidenceRefV1` at :322).
#[test]
fn a_resolve_body_is_checked_member_by_member() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(&scratch, &operator, &[Stage::Unknown])?;
    let mut replies = Vec::new();
    // A valid body commits a quarantine, so each later case names the generation it left.
    let mut generation = "3".to_owned();
    for (index, (case, body, field)) in resolve_body_cases(&evidence_of(&evidence))
        .into_iter()
        .enumerate()
    {
        let key = format!("28d00000-0000-4000-8000-0000000009{:02x}", 0xb0 + index);
        let reply = serve(
            &tasks,
            &operator,
            &resolve_frame(u8::try_from(index)?, &key, &ids[0], &generation, &body)?,
        )?;
        if let Some(field) = field {
            assert_eq!(
                code_at(&reply),
                (&json!("invalid_argument"), &json!(field)),
                "{case}: {reply}"
            );
        } else {
            assert_eq!(reply["kind"], json!("result"), "{case}: {reply}");
            generation = reply["body"]["task"]["generation"]
                .as_str()
                .ok_or("generation")?
                .to_owned();
        }
        replies.push(reply);
    }
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|reply| ("task.resolve", reply))
        .collect();
    conforms(&rows)?;
    Ok(())
}

/// `id` as a `UuidV4`, or why not, as text.
fn uuid_of(id: &str) -> Result<UuidV4<'_>, String> {
    UuidV4::parse(id).map_err(|error| format!("{error:?}"))
}

/// Settle `attempt` (generation 1) of `task` at `generation` as observed: no effect, 10 ms, cleanup
/// settled, not ready to verify.
fn settle_observed(
    store: &mut Store,
    task: &str,
    generation: &str,
    attempt: &str,
    event: u16,
) -> Result<String, String> {
    use habitat_engine::store::{Effect, Expected, Settlement};
    store
        .settle_attempt(
            &Expected {
                task: uuid_of(task)?,
                task_generation: generation.parse().map_err(|error| format!("{error:?}"))?,
                attempt: uuid_of(attempt)?,
                attempt_generation: "1".parse().map_err(|error| format!("{error:?}"))?,
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(10),
                cleanup_settled: true,
                ready_to_verify: false,
            },
            UuidV4::parse(&nth(0x08c0, event)).map_err(|error| format!("{error:?}"))?,
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))
}

/// B08 (code review): the observation door and a disposition. A stopped task's attempt cannot be
/// settled back into life (`AlreadyStopped`); an observation that settles a quarantined task's
/// attempt lifts the quarantine, because it settles the ambiguity the quarantine held apart.
#[test]
fn an_observation_never_revives_a_stopped_task_and_lifts_a_quarantine() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) =
        resolve_ledger(&scratch, &operator, &[Stage::Unknown, Stage::Unknown])?;
    let refs = json!([evidence_of(&evidence)]);
    let (first, second) = (nth(0x05b2, 1), nth(0x05b2, 2));
    resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &ids[0], "3"),
        &first,
        "acknowledge_external_effect",
        &refs,
    )?;
    let abandoned = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, &ids[0], "4"),
        &first,
        "abandon",
        &refs,
    )?;
    let generation = abandoned["body"]["task"]["generation"]
        .as_str()
        .ok_or("gen")?
        .to_owned();
    resolve_with(
        &tasks,
        &operator,
        (3, RESOLVE_KEY_3, &ids[1], "3"),
        &second,
        "quarantine",
        &json!([]),
    )?;
    drop(tasks);
    let mut store = raw_store(&scratch)?;
    assert_eq!(
        settle_observed(&mut store, &ids[0], &generation, &first, 1),
        Err("AlreadyStopped".to_owned())
    );
    settle_observed(&mut store, &ids[1], "4", &second, 2)?;
    let lifted = store
        .get(
            &operator,
            UuidV4::parse(&ids[1])?,
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!(
        lifted.state, "repair_pending",
        "the observation lifts the quarantine"
    );
    Ok(())
}

/// B08 (code review): every reader of a delivery sees a disposition. `task.get` reads an abandoned
/// task's stop notification as `pending`, and a delivery given up as `unknown`, never `delivered`;
/// the recovery inventory's delivery list drops what was given up.
#[test]
fn every_delivery_reader_sees_an_abandonment_and_a_delivery_given_up() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(
        &scratch,
        &operator,
        &[Stage::UnknownEffectOnly, Stage::Accepted],
    )?;
    let refs = json!([evidence_of(&evidence)]);
    let first = nth(0x05b2, 1);
    resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &ids[0], "3"),
        &first,
        "acknowledge_external_effect",
        &refs,
    )?;
    resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, &ids[0], "4"),
        &first,
        "abandon",
        &refs,
    )?;
    let read = |no: u8, task: &str| {
        serve(
            &tasks,
            &operator,
            &request(
                "task.get",
                no,
                None,
                &json!({"selector": {"task_id": task}, "evidence": "none"}),
            ),
        )
    };
    assert_eq!(
        read(3, &ids[0])?["body"]["delivery"],
        json!("pending"),
        "the stop notification is owed"
    );
    let event = ledger_value(
        &scratch,
        "SELECT o.event_id FROM outbox o JOIN events e ON e.id=o.event_id WHERE e.task_id=? AND o.delivered=0",
        &ids[1],
    )?;
    let event = event.as_str().ok_or("delivery event")?.to_owned();
    resolve_with(
        &tasks,
        &operator,
        (4, RESOLVE_KEY_3, &ids[1], "5"),
        &event,
        "abandon",
        &json!([]),
    )?;
    let given_up = read(5, &ids[1])?;
    assert_eq!(
        given_up["body"]["delivery"],
        json!("unknown"),
        "given up is not delivered: {given_up}"
    );
    drop(tasks);
    let mut store = raw_store(&scratch)?;
    let inventory = store
        .recovery_inventory(
            UuidV4::parse(EPOCH)?,
            habitat_engine::store::RecoveryLimits {
                rows: 1024,
                bytes: 1_048_576,
            },
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?;
    assert!(
        inventory
            .pending_delivery
            .iter()
            .all(|row| row.event != event),
        "the recovery inventory's list"
    );
    conforms(&[("task.get", &given_up)])?;
    Ok(())
}

/// B08 (code review): a quarantine of an obligation already closed is `conflict`; an unknown effect
/// whose usage is unknown stays pending when acknowledged, though its cleanup settled.
#[test]
fn a_closed_obligation_takes_no_quarantine_and_unknown_usage_keeps_one_open() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(
        &scratch,
        &operator,
        &[Stage::UnknownEffectOnly, Stage::UnknownEffectAndUsage],
    )?;
    let refs = json!([evidence_of(&evidence)]);
    let acknowledge = "acknowledge_external_effect";
    let (first, second) = (nth(0x05b2, 1), nth(0x05b2, 2));
    resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &ids[0], "3"),
        &first,
        acknowledge,
        &refs,
    )?;
    let closed = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, &ids[0], "4"),
        &first,
        "quarantine",
        &json!([]),
    )?;
    assert_eq!(
        code_at(&closed),
        (&json!("conflict"), &json!("/body/obligation_id")),
        "{closed}"
    );
    let usage = resolve_with(
        &tasks,
        &operator,
        (3, RESOLVE_KEY_3, &ids[1], "3"),
        &second,
        acknowledge,
        &refs,
    )?;
    assert_eq!(
        (
            &usage["body"]["obligation_state"],
            &usage["body"]["task"]["unresolved_obligations"]
        ),
        (&json!("pending"), &json!(1)),
        "usage unknown: {usage}"
    );
    conforms(&[("task.resolve", &closed), ("task.resolve", &usage)])?;
    Ok(())
}

/// B08 (code review): the stop door re-reads the evidence's registered size, for every stop. An
/// artifact row registered with another size than the object is corruption: the abandonment is
/// refused and nothing stops.
#[test]
fn a_stop_refuses_evidence_registered_with_another_size() -> Outcome {
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, evidence) = resolve_ledger(&scratch, &operator, &[Stage::UnknownEffectOnly])?;
    drop(tasks);
    let file = scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("ledger.sqlite3");
    let db = rusqlite::Connection::open(&file)?;
    db.execute(
        "INSERT INTO artifacts(digest,size) VALUES(?,?)",
        rusqlite::params![evidence.digest(), i64::try_from(evidence.size())? + 1],
    )?;
    drop(db);
    let tasks = StoreTasks::new(raw_store(&scratch)?, EPOCH.to_owned());
    let refs = json!([evidence_of(&evidence)]);
    let first = nth(0x05b2, 1);
    resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &ids[0], "3"),
        &first,
        "acknowledge_external_effect",
        &refs,
    )?;
    let refused = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, &ids[0], "4"),
        &first,
        "abandon",
        &refs,
    )?;
    assert_eq!(
        (&refused["kind"], &refused["code"]),
        (&json!("error"), &json!("internal")),
        "{refused}"
    );
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT count(*) FROM task_stops WHERE task_id=?",
            &ids[0]
        )?,
        json!(0),
        "nothing stopped"
    );
    conforms(&[("task.resolve", &refused)])?;
    Ok(())
}

/// B08 (second review): the stop charges exactly the unsettled attempts' measured usage, on top of
/// what settled attempts already charged -- off the origin: the first attempt settled at 10 ms (the
/// fixture's literal), the second is unsettled at 7 ms, so the spend is 17 after the stop.
#[test]
fn an_abandonment_charges_only_what_no_settlement_charged() -> Outcome {
    use habitat_engine::store::{Effect, Expected, Settlement};
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let mut store = raw_store(&scratch)?;
    let task = staged(&mut store, &operator, 1, Stage::Settled)?;
    let until = Instant::now() + Duration::from_secs(10);
    let second = nth(0x08d2, 1);
    let begun = store
        .begin_attempt(
            UuidV4::parse(&task)?,
            "3".parse()?,
            UuidV4::parse(&second)?,
            UuidV4::parse(&nth(0x08d3, 1))?,
            until,
        )
        .map_err(|error| format!("{error:?}"))?;
    store
        .settle_attempt(
            &Expected {
                task: UuidV4::parse(&task)?,
                task_generation: "4".parse()?,
                attempt: UuidV4::parse(&second)?,
                attempt_generation: begun.generation.parse()?,
            },
            Settlement {
                effect: Effect::Unknown,
                used_ms: Some(7),
                cleanup_settled: true,
                ready_to_verify: false,
            },
            UuidV4::parse(&nth(0x08d4, 1))?,
            until,
        )
        .map_err(|error| format!("{error:?}"))?;
    let evidence = store
        .publish(
            b"operator's reconciliation note",
            UuidV4::parse(EPOCH)?,
            until,
        )
        .map_err(|error| format!("{error:?}"))?;
    let tasks = StoreTasks::new(store, EPOCH.to_owned());
    assert_eq!(
        ledger_value(&scratch, "SELECT spent_ms FROM tasks WHERE id=?", &task)?,
        json!(10)
    );
    let refs = json!([evidence_of(&evidence)]);
    resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &task, "5"),
        &second,
        "acknowledge_external_effect",
        &refs,
    )?;
    let abandoned = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, &task, "6"),
        &second,
        "abandon",
        &refs,
    )?;
    assert_eq!(
        abandoned["body"]["task"]["state"],
        json!("abandoned"),
        "{abandoned}"
    );
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT spent_ms||'/'||reserved_work_ms FROM tasks WHERE id=?",
            &task
        )?,
        json!("17/0")
    );
    Ok(())
}

/// B08 (second review): only an observation that settles lifts a quarantine. A re-observation that
/// is still unsettled leaves the task `blocked`: the ambiguity the quarantine holds apart remains.
#[test]
fn an_unsettled_observation_keeps_a_quarantine() -> Outcome {
    use habitat_engine::store::{Effect, Expected, Settlement};
    let scratch = Scratch::new()?;
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let (tasks, ids, _) = resolve_ledger(&scratch, &operator, &[Stage::Unknown])?;
    let attempt = nth(0x05b2, 1);
    resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &ids[0], "3"),
        &attempt,
        "quarantine",
        &json!([]),
    )?;
    drop(tasks);
    let mut store = raw_store(&scratch)?;
    let until = Instant::now() + Duration::from_secs(10);
    store
        .settle_attempt(
            &Expected {
                task: UuidV4::parse(&ids[0])?,
                task_generation: "4".parse()?,
                attempt: UuidV4::parse(&attempt)?,
                attempt_generation: "1".parse()?,
            },
            Settlement {
                effect: Effect::Unknown,
                used_ms: None,
                cleanup_settled: false,
                ready_to_verify: false,
            },
            UuidV4::parse(&nth(0x08d5, 1))?,
            until,
        )
        .map_err(|error| format!("{error:?}"))?;
    let head = store
        .get(&operator, UuidV4::parse(&ids[0])?, until)
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!(
        (head.state.as_str(), head.generation.as_str()),
        ("blocked", "5")
    );
    Ok(())
}
