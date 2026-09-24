//! T28 APP-07 (a unit-test module of `app::tasks`): a lost commit driven end to end through the
//! real ledger. The store's `AfterCommit` cut point exists only in a test build of this crate, so
//! these cases live inside it: `task.submit` goes through the control receiver into
//! `StoreTasks` over a store that commits the admission and then reports the commit uncertain,
//! exactly as a failed COMMIT does. No double stands in for the store: what the ledger does after
//! an uncertain commit (it refuses every later use of that connection) is the real store's own.
use super::{StoreTasks, submit_readback};
use crate::actions::control::{self, Composed, Grants, Reply};
use crate::actions::{Caller, Effect, Owner};
use crate::contracts::UuidV4;
use crate::store::{CutPoint, Principal, Store};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::Write as _;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

type Outcome = Result<(), Box<dyn Error>>;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const GENERATION: &str = "28e00000-0000-4000-8000-000000000001";
const EPOCH: &str = "28e00000-0000-4000-8000-000000000002";
const KEY: &str = "28e00000-0000-4000-8000-0000000000aa";
const NOW: u64 = 1_790_000_000_000;

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Result<Self, Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!(
            "hee3-t28u-{}-{}",
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

/// Sees and may do everything: these cases are about the ledger, not grants.
struct Open;

impl Grants for Open {
    fn resolve(&self, _: &Principal, _: &str, _: &str, _: u64) -> Option<Caller> {
        let caller = Owner::ALL.into_iter().fold(Caller::new(), Caller::seeing);
        Some(Effect::ALL.into_iter().fold(caller, Caller::granted))
    }
}

/// `sha256:` hex of `bytes`, computed with `sha2` directly, not through the engine.
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::from("sha256:"), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

fn open(scratch: &Scratch, point: Option<CutPoint>) -> Result<StoreTasks, Box<dyn Error>> {
    let root = scratch.0.join("state");
    if !root.exists() {
        DirBuilder::new().mode(0o700).create(&root)?;
    }
    let (generation, epoch) = (UuidV4::parse(GENERATION)?, UuidV4::parse(EPOCH)?);
    let deadline = Instant::now() + Duration::from_secs(10);
    let store = match point {
        Some(point) => Store::open_faulted(&root, generation, epoch, true, deadline, point),
        None => Store::open(&root, generation, epoch, false, deadline),
    }
    .map_err(|error| format!("{error:?}"))?;
    Ok(StoreTasks::new(store, EPOCH.to_owned()))
}

fn request(action: &str, request: u8, key: Option<&str>, body: &Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "protocol": "hee3.control", "version": 1, "kind": "request",
        "request_id": format!("28e00000-0000-4000-8000-0000000001{request:02x}"),
        "action": action, "action_version": 1, "idempotency_key": key,
        "deadline_unix_ms": (NOW + 5_000).to_string(),
        "authority": {"grant_id": KEY, "scope_sha256": format!("sha256:{}", "4".repeat(64))},
        "precondition": null, "body": body,
    }))
    .unwrap_or_default()
}

fn submission() -> Vec<u8> {
    request(
        "task.submit",
        1,
        Some(KEY),
        &json!({"spec": {
            "task_class": "rust-library-change/1",
            "intent": "Add a strict decimal parser.",
            "criteria": ["rejects a leading zero", "round-trips the maximum"],
            "privacy": "local_only",
            "workspace_id": "28e00000-0000-4000-8000-0000000000bb",
            "budget": {"mode": "hard", "wall_ms": "600000", "tokens": "0", "currency_microunits": "0"},
            "parent": null,
        }}),
    )
}

fn readback(request_id: u8) -> Vec<u8> {
    request(
        "task.get",
        request_id,
        None,
        &json!({"selector": {"source_action": "task.submit", "idempotency_key": KEY}, "evidence": "none"}),
    )
}

fn serve(tasks: &StoreTasks, payload: &[u8]) -> Result<Value, Box<dyn Error>> {
    let operator = Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?;
    let composed = Composed {
        grants: &Open,
        health: None,
        tasks: Some(tasks),
    };
    match control::serve_composed(payload, NOW, &operator, composed) {
        Reply::Frame(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Reply::Close(fault) => Err(format!("closed: {}", fault.name()).into()),
    }
}

#[test]
fn a_lost_commit_is_effect_unknown_and_its_readback_finds_the_admission() -> Outcome {
    let scratch = Scratch::new()?;
    let submit = submission();
    {
        let tasks = open(&scratch, Some(CutPoint::AfterCommit))?;
        let lost = serve(&tasks, &submit)?;
        // The whole record: RC03 §4's effect_unknown, carrying the read that settles it, built
        // from values the caller held before sending.
        assert_eq!(
            lost,
            json!({
                "protocol": "hee3.control", "version": 1, "kind": "error",
                "request_id": "28e00000-0000-4000-8000-000000000101",
                "request_sha256": digest(&submit),
                "code": "effect_unknown", "effect": "unknown", "retry": "after_readback",
                "readback": {"action": "task.get", "action_version": 1, "body": {
                    "selector": {"source_action": "task.submit", "idempotency_key": KEY},
                    "evidence": "none"}},
                "message": "the commit may or may not have happened; read back before any retry",
                "details": {"field": null, "constraint": null, "current_generation": null},
            })
        );
        assert_eq!(lost["readback"], submit_readback(KEY));
        // The same engine cannot answer that read: the ledger refuses every use of a connection
        // whose commit is uncertain until it is reopened. That is a condition to wait on, named,
        // never `internal` with `retry: never`, which would forbid the read effect_unknown asked for.
        let unsettled = serve(&tasks, &readback(2))?;
        assert_eq!(
            unsettled,
            json!({
                "protocol": "hee3.control", "version": 1, "kind": "error",
                "request_id": "28e00000-0000-4000-8000-000000000102",
                "request_sha256": digest(&readback(2)),
                "code": "unavailable", "effect": "none", "retry": "after_condition",
                "readback": null,
                "message": "the ledger's last commit is uncertain; it must be reopened",
                "details": {"field": null, "constraint": null, "current_generation": null},
            })
        );
    }
    // Reopened, as an engine restart reopens it: the readback finds the admission the uncertain
    // commit made, and the caller's exact retry is that admission replayed, never a second one.
    let tasks = open(&scratch, None)?;
    let found = serve(&tasks, &readback(3))?;
    let task = found["body"]["task"]["task_id"]
        .as_str()
        .ok_or("the readback found no task")?
        .to_owned();
    assert_eq!(
        found["body"]["task"],
        json!({"task_id": task, "generation": "1", "state": "admitted", "current_attempt_id": null, "unresolved_obligations": 0})
    );
    let retried = serve(&tasks, &submit)?;
    assert_eq!(
        (
            &retried["effect"],
            &retried["replayed"],
            &retried["body"]["task"]["task_id"]
        ),
        (&json!("committed"), &json!(true), &json!(task)),
        "{retried}"
    );
    Ok(())
}
