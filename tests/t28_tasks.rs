//! T28 task-owner cases (a module of `t28_actions`): `task.submit` and `task.get` composed behind
//! the control receiver over a real ledger (review D-C3 step 3). Every ledger is a scratch root.
use habitat_engine::actions::control::{self, Composed, Grants, Reply};
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
fn digest(bytes: impl AsRef<[u8]>) -> String {
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
    let composed = Composed {
        grants: &Open,
        health: None,
        tasks: Some(tasks),
    };
    match control::serve_composed(payload, NOW, principal, composed) {
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

#[test]
fn without_a_composed_ledger_task_actions_are_unavailable() -> Result<(), Box<dyn Error>> {
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let composed = Composed {
        grants: &Open,
        health: None,
        tasks: None,
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
