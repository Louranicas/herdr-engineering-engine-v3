//! B09a · `task.get`'s evidence views (task-G08), for what the engine writes in production today:
//! an operator's dispositions (their attested `EvidenceRefV1`s) and an abandonment's stop (its
//! identity recovered through its disposition). Design: `~/hee3-evidence/T28/B09-task-get-evidence-
//! 20260925/DESIGN.md` (R1, R2). Expected values are literals built from the references each case
//! sends; ledger facts are read or tampered by SQL on the ledger file; every reply goes through the
//! independent schema oracle.

use super::tasks::{
    EPOCH, GENERATION, Handed, KEY, NOW, Open, RESOLVE_KEY, RESOLVE_KEY_2, Scratch, Stage,
    conforms, ledger_value, nth, raw_store, request, resolve_with, serve, serve_composed_at,
    staged,
};
use habitat_engine::app::tasks::StoreTasks;
use habitat_engine::contracts::UuidV4;
use habitat_engine::store::{Object, Principal};
use serde_json::{Value, json};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

type Outcome = Result<(), Box<dyn Error>>;

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}

fn operator() -> Result<Principal, Box<dyn Error>> {
    Ok(Principal::new(1000, "operator").map_err(|error| format!("{error:?}"))?)
}

/// A composed ledger, its task identities and its published objects.
type Seeded = (StoreTasks, Vec<String>, Vec<Object>);

/// A ledger holding tasks at `stages` and one published object per entry of `bodies`.
fn ledger(scratch: &Scratch, stages: &[Stage], bodies: &[&[u8]]) -> Result<Seeded, Box<dyn Error>> {
    let operator = operator()?;
    let mut store = raw_store(scratch)?;
    let mut tasks = Vec::new();
    for (index, stage) in stages.iter().enumerate() {
        tasks.push(staged(
            &mut store,
            &operator,
            u16::try_from(index + 1)?,
            *stage,
        )?);
    }
    let mut objects = Vec::new();
    for (index, body) in bodies.iter().enumerate() {
        let staging = format!("07b90000-0000-4000-8000-{:012x}", index + 1);
        objects.push(
            store
                .publish(body, UuidV4::parse(&staging)?, deadline())
                .map_err(|error| format!("{error:?}"))?,
        );
    }
    Ok((StoreTasks::new(store, EPOCH.to_owned()), tasks, objects))
}

/// The `EvidenceRefV1` naming `object` under identity `id` (the caller's, recorded as given).
fn reference(object: &Object, id: &str, media: &str) -> Value {
    json!({"artifact_id": id, "sha256": object.digest(), "byte_length": object.size(),
           "media_type": media, "schema_id": "hee3.evidence/1"})
}

const ID_A: &str = "07b90000-0000-4000-8000-0000000000a1";
const ID_B: &str = "07b90000-0000-4000-8000-0000000000b2";

/// `task.get` of `task` with evidence view `view`.
fn get(tasks: &StoreTasks, task: &str, view: &str, no: u8) -> Result<Value, Box<dyn Error>> {
    serve(
        tasks,
        &operator()?,
        &request(
            "task.get",
            no,
            None,
            &json!({"selector": {"task_id": task}, "evidence": view}),
        ),
    )
}

/// Task 1 (effect unknown) acknowledged with `[A]`, then abandoned with `[B, A]`: the stop's evidence
/// is the abandonment's first reference, `B`.
fn abandoned(scratch: &Scratch) -> Result<(StoreTasks, String, Vec<Object>), Box<dyn Error>> {
    let (tasks, ids, objects) = ledger(
        scratch,
        &[Stage::Unknown],
        &[
            b"operator's reconciliation note",
            b"worker log excerpt, 2 KiB",
        ],
    )?;
    let operator = operator()?;
    let (task, attempt) = (ids[0].clone(), nth(0x05b2, 1));
    let (a, b) = (
        reference(&objects[0], ID_A, "text/plain"),
        reference(&objects[1], ID_B, "text/x-log"),
    );
    let acknowledged = resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &task, "3"),
        &attempt,
        "acknowledge_external_effect",
        &json!([a]),
    )?;
    assert_eq!(acknowledged["kind"], json!("result"), "{acknowledged}");
    let stopped = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, &task, "4"),
        &attempt,
        "abandon",
        &json!([b, a]),
    )?;
    assert_eq!(
        stopped["body"]["task"]["state"],
        json!("abandoned"),
        "{stopped}"
    );
    Ok((tasks, task, objects))
}

/// B09-E1 · the two views of an abandoned task. `summary` is the stop's evidence — the abandonment's
/// first reference, `B`, with the identity the operator gave it. `refs` is the stop, then every
/// disposition's references in disposition order, each identity once (`A` from the
/// acknowledgement; the abandonment's `B` and `A` already listed). `none` stays empty.
#[test]
fn an_abandoned_task_names_its_evidence_in_both_views() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, task, objects) = abandoned(&scratch)?;
    let (a, b) = (
        reference(&objects[0], ID_A, "text/plain"),
        reference(&objects[1], ID_B, "text/x-log"),
    );
    let none = get(&tasks, &task, "none", 3)?;
    let summary = get(&tasks, &task, "summary", 4)?;
    let refs = get(&tasks, &task, "refs", 5)?;
    assert_eq!(none["body"]["evidence"], json!([]), "{none}");
    assert_eq!(summary["body"]["evidence"], json!([b]), "{summary}");
    assert_eq!(refs["body"]["evidence"], json!([b, a]), "{refs}");
    // The head is the same read in every view.
    for reply in [&summary, &refs] {
        assert_eq!(reply["body"]["task"], none["body"]["task"]);
        assert_eq!(reply["effect"], json!("none"));
    }
    conforms(&[
        ("task.get", &none),
        ("task.get", &summary),
        ("task.get", &refs),
    ])
}

/// The object file behind `object` in the ledger's generation.
fn object_file(scratch: &Scratch, object: &Object) -> PathBuf {
    let hex = &object.digest()[7..];
    scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("objects")
        .join("sha256")
        .join(&hex[..2])
        .join(hex)
}

/// B09-E2 · T17: "missing, expired or corrupt evidence at later readback must report current
/// unavailability while preserving the historical event". An object removed after the fact makes the
/// view that names it `unavailable` at `/body/evidence` — never a fresh verification — while a view
/// that does not name it, and `none`, still answer; a same-size object with other bytes is corrupt.
#[test]
fn evidence_is_checked_now_and_history_is_kept() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, task, objects) = abandoned(&scratch)?;
    let a = object_file(&scratch, &objects[0]);
    assert!(a.is_file(), "{}", a.display());
    fs::remove_file(&a)?;
    let refs = get(&tasks, &task, "refs", 6)?;
    assert_eq!(
        (&refs["code"], &refs["details"]["field"], &refs["retry"]),
        (
            &json!("unavailable"),
            &json!("/body/evidence"),
            &json!("after_condition")
        ),
        "{refs}"
    );
    let summary = get(&tasks, &task, "summary", 7)?;
    assert_eq!(summary["kind"], json!("result"), "B is present: {summary}");
    let b = object_file(&scratch, &objects[1]);
    let size = usize::try_from(fs::metadata(&b)?.len())?;
    fs::remove_file(&b)?;
    fs::write(&b, vec![b'x'; size])?;
    let corrupt = get(&tasks, &task, "summary", 8)?;
    assert_eq!(
        (&corrupt["code"], &corrupt["details"]["field"]),
        (&json!("unavailable"), &json!("/body/evidence")),
        "{corrupt}"
    );
    let none = get(&tasks, &task, "none", 9)?;
    assert_eq!(none["body"]["task"]["state"], json!("abandoned"), "{none}");
    conforms(&[
        ("task.get", &refs),
        ("task.get", &summary),
        ("task.get", &corrupt),
        ("task.get", &none),
    ])
}

/// B09-E3 · the contract's bound (`evidence arrays ≤64 … truncation is never silent`): dispositions
/// carrying more than 64 references (counted before identities collapse — conservative, and decided
/// in SQL before a single blob is read) make `refs` `resource_exhausted`, and the narrower route
/// travels as the readback (`summary`, `retry: after_readback`), not in the message; the effect is
/// `none`. `summary` then answers.
#[test]
fn more_than_64_references_route_to_the_summary() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, ids, objects) =
        ledger(&scratch, &[Stage::Unknown], &[b"one note, cited 65 times"])?;
    let operator = operator()?;
    let (task, attempt) = (ids[0].clone(), nth(0x05b2, 1));
    let many: Vec<Value> = (1..=64)
        .map(|index| {
            reference(
                &objects[0],
                &format!("07b90000-0000-4000-8000-{index:012x}"),
                "text/plain",
            )
        })
        .collect();
    let acknowledged = resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &task, "3"),
        &attempt,
        "acknowledge_external_effect",
        &json!(many),
    )?;
    assert_eq!(acknowledged["kind"], json!("result"), "{acknowledged}");
    let last = reference(&objects[0], ID_B, "text/plain");
    let stopped = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, &task, "4"),
        &attempt,
        "abandon",
        &json!([last]),
    )?;
    assert_eq!(
        stopped["body"]["task"]["state"],
        json!("abandoned"),
        "{stopped}"
    );
    let refs = get(&tasks, &task, "refs", 3)?;
    assert_eq!(
        (&refs["code"], &refs["effect"], &refs["retry"]),
        (
            &json!("resource_exhausted"),
            &json!("none"),
            &json!("after_readback")
        ),
        "{refs}"
    );
    assert_eq!(
        refs["readback"],
        json!({"action": "task.get", "action_version": 1,
               "body": {"selector": {"task_id": task}, "evidence": "summary"}})
    );
    let summary = get(&tasks, &task, "summary", 4)?;
    assert_eq!(summary["body"]["evidence"], json!([last]), "{summary}");
    conforms(&[("task.get", &refs), ("task.get", &summary)])
}

/// B09-E4 · a task holding a verification or an acceptance has evidence whose identity the ledger
/// did not record (B09b, with B14, records it): its views are refused `unavailable`, `retry: never`,
/// never guessed; `none` answers.
#[test]
fn identity_the_ledger_did_not_record_is_refused_not_guessed() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, ids, _) = ledger(&scratch, &[Stage::Failed, Stage::Accepted], &[])?;
    let mut replies = Vec::new();
    for (index, task) in ids.iter().enumerate() {
        for (offset, view) in ["summary", "refs"].into_iter().enumerate() {
            let reply = get(&tasks, task, view, u8::try_from(index * 2 + offset + 1)?)?;
            assert_eq!(
                (
                    &reply["code"],
                    &reply["retry"],
                    &reply["details"]["constraint"]
                ),
                (
                    &json!("unavailable"),
                    &json!("never"),
                    &json!("evidence identity not recorded for this task")
                ),
                "{view}: {reply}"
            );
            replies.push(reply);
        }
        let none = get(&tasks, task, "none", 0x20 + u8::try_from(index)?)?;
        assert_eq!(none["kind"], json!("result"), "{none}");
    }
    let rows: Vec<(&str, &Value)> = replies.iter().map(|reply| ("task.get", reply)).collect();
    conforms(&rows)
}

/// Run `sql` against the ledger file itself (a tampering the engine never performs).
fn tamper(scratch: &Scratch, sql: &str, param: &str) -> Outcome {
    let file = scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("ledger.sqlite3");
    let db = rusqlite::Connection::open(file)?;
    db.execute(sql, [param])?;
    Ok(())
}

/// B09-E5 · a stop's identity comes only from its disposition (R2.1): a stop whose event names no
/// disposition is unrecoverable (`unavailable`, `never`); one whose disposition is not an
/// abandonment of that task is corruption (`internal`), never a guessed reference.
#[test]
fn a_stops_identity_comes_only_from_its_disposition() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, task, _) = abandoned(&scratch)?;
    let acknowledgement = ledger_value(
        &scratch,
        "SELECT id FROM task_dispositions WHERE task_id=? AND disposition='acknowledge_external_effect'",
        &task,
    )?;
    let acknowledgement = acknowledgement
        .as_str()
        .ok_or("acknowledgement id")?
        .to_owned();
    tamper(
        &scratch,
        "UPDATE events SET body=CAST(json_set(CAST(body AS TEXT),'$.disposition_id',?) AS BLOB) \
         WHERE kind='task_stopped'",
        &acknowledgement,
    )?;
    let wrong = get(&tasks, &task, "summary", 3)?;
    assert_eq!(wrong["code"], json!("internal"), "{wrong}");
    tamper(
        &scratch,
        "UPDATE events SET body=CAST(json_remove(CAST(body AS TEXT),'$.disposition_id') AS BLOB) \
         WHERE kind='task_stopped' AND ?<>''",
        "x",
    )?;
    let unnamed = get(&tasks, &task, "summary", 4)?;
    assert_eq!(
        (
            &unnamed["code"],
            &unnamed["retry"],
            &unnamed["details"]["constraint"]
        ),
        (
            &json!("unavailable"),
            &json!("never"),
            &json!("evidence identity not recorded for this task")
        ),
        "{unnamed}"
    );
    conforms(&[("task.get", &wrong), ("task.get", &unnamed)])
}

/// B09-E6 · every disposition's evidence is registered in `artifacts` in the resolve's own
/// transaction (R1.5, a B08 repair: backups copy only registered objects), and a resolve that would
/// take the inventory past the backup's 4096-object bound is refused `resource_exhausted` with
/// nothing written (R2.4), so dispositions can never make a ledger un-backup-able.
#[test]
fn a_resolve_registers_its_evidence_within_the_backup_bound() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, ids, objects) = ledger(
        &scratch,
        &[Stage::Unknown, Stage::Unknown],
        &[b"first note", b"second note"],
    )?;
    let operator = operator()?;
    let first = reference(&objects[0], ID_A, "text/plain");
    let quarantined = resolve_with(
        &tasks,
        &operator,
        (1, RESOLVE_KEY, &ids[0], "3"),
        &nth(0x05b2, 1),
        "quarantine",
        &json!([first]),
    )?;
    assert_eq!(quarantined["kind"], json!("result"), "{quarantined}");
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT count(*) FROM artifacts WHERE digest=?",
            objects[0].digest()
        )?,
        json!(1),
        "the disposition's evidence is registered"
    );
    let registered = ledger_value(&scratch, "SELECT count(*) FROM artifacts WHERE ?<>''", "x")?;
    let registered = registered.as_i64().ok_or("count")?;
    for index in 0..(4096 - registered) {
        tamper(
            &scratch,
            "INSERT INTO artifacts(digest,size) VALUES(?,0)",
            &format!("sha256:{index:064x}"),
        )?;
    }
    let dispositions = ledger_value(
        &scratch,
        "SELECT count(*) FROM task_dispositions WHERE ?<>''",
        "x",
    )?;
    let second = reference(&objects[1], ID_B, "text/plain");
    let full = resolve_with(
        &tasks,
        &operator,
        (2, RESOLVE_KEY_2, &ids[1], "3"),
        &nth(0x05b2, 2),
        "quarantine",
        &json!([second]),
    )?;
    assert_eq!(
        (&full["code"], &full["details"]["field"]),
        (&json!("resource_exhausted"), &json!("/body/evidence")),
        "{full}"
    );
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT count(*) FROM task_dispositions WHERE ?<>''",
            "x"
        )?,
        dispositions,
        "nothing was written"
    );
    assert_eq!(
        ledger_value(&scratch, "SELECT count(*) FROM artifacts WHERE ?<>''", "x")?,
        json!(4096)
    );
    conforms(&[("task.resolve", &quarantined), ("task.resolve", &full)])
}

/// B09-E7 · the owner is handed the view it was asked for (F101: a double that records).
#[test]
fn the_get_owner_is_handed_the_view() -> Outcome {
    let handed = Handed::default();
    let operator = operator()?;
    let frame = request(
        "task.get",
        0x30,
        None,
        &json!({"selector": {"task_id": KEY}, "evidence": "refs"}),
    );
    let reply = serve_composed_at(&handed, &Open, &operator, &frame, NOW)?;
    assert_eq!(reply["code"], json!("deadline_exceeded"), "{reply}");
    assert_eq!(
        *handed.0.borrow(),
        [format!(
            "get {operator:?} Task(\"{KEY}\") Some(Refs) {} {NOW}",
            NOW + 5_000
        )]
    );
    Ok(())
}
