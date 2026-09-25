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
const ID_C: &str = "07b90000-0000-4000-8000-0000000000c3";

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

/// Task 1 (effect unknown) acknowledged with `[A]`, then abandoned with `[B, C]`: the stop's evidence
/// is the abandonment's first reference, `B`. Three distinct objects, so the order of the
/// dispositions is observable in `refs`.
fn abandoned(scratch: &Scratch) -> Result<(StoreTasks, String, Vec<Object>), Box<dyn Error>> {
    let (tasks, ids, objects) = ledger(
        scratch,
        &[Stage::Unknown],
        &[
            b"operator's reconciliation note",
            b"worker log excerpt, 2 KiB",
            b"repository diff at the lost reply",
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
        &json!([b, reference(&objects[2], ID_C, "text/x-diff")]),
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
/// disposition's references in disposition order, an exact repeat once (`A` from the
/// acknowledgement; the abandonment's `B`, already listed, and `C`). `none` stays empty.
#[test]
fn an_abandoned_task_names_its_evidence_in_both_views() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, task, objects) = abandoned(&scratch)?;
    let (a, b, c) = (
        reference(&objects[0], ID_A, "text/plain"),
        reference(&objects[1], ID_B, "text/x-log"),
        reference(&objects[2], ID_C, "text/x-diff"),
    );
    let none = get(&tasks, &task, "none", 3)?;
    let summary = get(&tasks, &task, "summary", 4)?;
    let refs = get(&tasks, &task, "refs", 5)?;
    assert_eq!(none["body"]["evidence"], json!([]), "{none}");
    assert_eq!(summary["body"]["evidence"], json!([b]), "{summary}");
    // The stop, then the acknowledgement's A, then the abandonment's B (a repeat) and C: oldest first.
    assert_eq!(refs["body"]["evidence"], json!([b, a, c]), "{refs}");
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

/// Resolve keys beyond the three `t28_tasks` shares, one per request.
const KEYS: [&str; 6] = [
    "07b90000-0000-4000-8000-0000000007e1",
    "07b90000-0000-4000-8000-0000000007e2",
    "07b90000-0000-4000-8000-0000000007e3",
    "07b90000-0000-4000-8000-0000000007e4",
    "07b90000-0000-4000-8000-0000000007e5",
    "07b90000-0000-4000-8000-0000000007e6",
];

/// `count` references to `object`, each its own identity (`base` + index).
fn cited(object: &Object, base: u32, count: u32) -> Vec<Value> {
    (0..count)
        .map(|index| {
            reference(
                object,
                &format!("07b90000-0000-4000-8000-{:012x}", base + index),
                "text/plain",
            )
        })
        .collect()
}

/// B09-E3 · the contract's bound (`evidence arrays ≤64 … truncation is never silent`), off both
/// edges. Task 1: an evidence-free acknowledgement, then an abandonment citing 64 identities — the
/// stop is the first of them, counted once (review D1): `refs` answers all 64. Task 2: 64 in the
/// acknowledgement and one more in the abandonment — 65, `resource_exhausted`, the narrower route
/// the readback (`summary`, `retry: after_readback`, effect `none`); `summary` answers. The count
/// is SQL's, before any blob is read: with the acknowledgement's blob replaced by 65 references no
/// reader would accept, the answer is still `resource_exhausted`, never the `internal` a read would
/// have produced (review G8).
#[test]
fn more_than_64_references_route_to_the_summary() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, ids, objects) = ledger(
        &scratch,
        &[Stage::Unknown, Stage::Unknown],
        &[b"one note, cited often"],
    )?;
    let operator = operator()?;
    let resolve = |no: u8,
                   key: &str,
                   task: &str,
                   generation: &str,
                   attempt: u16,
                   disposition: &str,
                   evidence: &Value| {
        resolve_with(
            &tasks,
            &operator,
            (no, key, task, generation),
            &nth(0x05b2, attempt),
            disposition,
            evidence,
        )
    };
    let sixty_four = cited(&objects[0], 0x100, 64);
    resolve(
        1,
        KEYS[0],
        &ids[0],
        "3",
        1,
        "acknowledge_external_effect",
        &json!([]),
    )?;
    let stopped = resolve(2, KEYS[1], &ids[0], "4", 1, "abandon", &json!(sixty_four))?;
    assert_eq!(
        stopped["body"]["task"]["state"],
        json!("abandoned"),
        "{stopped}"
    );
    let full = get(&tasks, &ids[0], "refs", 3)?;
    assert_eq!(
        full["body"]["evidence"],
        json!(sixty_four),
        "64 are admitted: {full}"
    );
    resolve(
        4,
        KEYS[2],
        &ids[1],
        "3",
        2,
        "acknowledge_external_effect",
        &json!(cited(&objects[0], 0x200, 64)),
    )?;
    let last = reference(&objects[0], ID_B, "text/plain");
    let over = resolve(5, KEYS[3], &ids[1], "4", 2, "abandon", &json!([last]))?;
    assert_eq!(over["body"]["task"]["state"], json!("abandoned"), "{over}");
    let refs = get(&tasks, &ids[1], "refs", 6)?;
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
               "body": {"selector": {"task_id": ids[1]}, "evidence": "summary"}})
    );
    let summary = get(&tasks, &ids[1], "summary", 7)?;
    assert_eq!(summary["body"]["evidence"], json!([last]), "{summary}");
    let junk = json!(vec![json!({"not": "a reference"}); 65]).to_string();
    let changed = tamper_two(
        &scratch,
        "UPDATE task_dispositions SET evidence=CAST(? AS BLOB) WHERE task_id=? AND disposition='acknowledge_external_effect'",
        &junk,
        &ids[1],
    )?;
    assert_eq!(changed, 1);
    let counted = get(&tasks, &ids[1], "refs", 8)?;
    assert_eq!(
        counted["code"],
        json!("resource_exhausted"),
        "counted before read: {counted}"
    );
    conforms(&[
        ("task.get", &full),
        ("task.get", &refs),
        ("task.get", &summary),
        ("task.get", &counted),
    ])
}

/// B09-E4 · a task holding a verification or an acceptance has evidence whose identity the ledger
/// did not record (B09b, with B14, records it): its views are refused `unavailable`, `retry: never`,
/// never guessed; `none` answers. Each clause alone: the failed task holds only a verification; the
/// accepted task, its verification removed from the ledger file, only an acceptance (review G3). The
/// principal door comes first: another operator's view of the same task is `not_found`, which says
/// nothing of what the task holds (review G6).
#[test]
fn identity_the_ledger_did_not_record_is_refused_not_guessed() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, ids, _) = ledger(&scratch, &[Stage::Failed, Stage::Accepted], &[])?;
    let removed = tamper(
        &scratch,
        "DELETE FROM verifications WHERE attempt_id=?",
        &nth(0x05b2, 2),
    )?;
    assert_eq!(removed, 1, "the accepted task keeps only its acceptance");
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
        let stranger = Principal::new(1001, "operator").map_err(|error| format!("{error:?}"))?;
        let hidden = serve(
            &tasks,
            &stranger,
            &request(
                "task.get",
                0x30 + u8::try_from(index)?,
                None,
                &json!({"selector": {"task_id": task}, "evidence": "summary"}),
            ),
        )?;
        assert_eq!(hidden["code"], json!("not_found"), "{hidden}");
        replies.push(hidden);
    }
    let rows: Vec<(&str, &Value)> = replies.iter().map(|reply| ("task.get", reply)).collect();
    conforms(&rows)
}

/// Run `sql` against the ledger file itself (a tampering the engine never performs); the rows it
/// changed, so a tampering that matched nothing cannot pass for one that did (review N7).
fn tamper(scratch: &Scratch, sql: &str, param: &str) -> Result<usize, Box<dyn Error>> {
    tamper_with(scratch, sql, &[param])
}

/// [`tamper`] with two parameters.
fn tamper_two(
    scratch: &Scratch,
    sql: &str,
    first: &str,
    second: &str,
) -> Result<usize, Box<dyn Error>> {
    tamper_with(scratch, sql, &[first, second])
}

fn tamper_with(scratch: &Scratch, sql: &str, params: &[&str]) -> Result<usize, Box<dyn Error>> {
    let file = scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("ledger.sqlite3");
    let db = rusqlite::Connection::open(file)?;
    Ok(db.execute(sql, rusqlite::params_from_iter(params))?)
}

/// The id of task `task`'s disposition of kind `disposition`, read from the ledger file.
fn disposition_id(
    scratch: &Scratch,
    task: &str,
    disposition: &str,
) -> Result<String, Box<dyn Error>> {
    let sql =
        format!("SELECT id FROM task_dispositions WHERE task_id=? AND disposition='{disposition}'");
    Ok(ledger_value(scratch, &sql, task)?
        .as_str()
        .ok_or("disposition id")?
        .to_owned())
}

// B09-E5 · a stop's identity comes only from its disposition (R2.1): five tamperings of the ledger
// file, one test each, each caught by exactly one rule of the join (review G1).

/// B09-E5 · the stop names the acknowledgement, whose first reference was made to match the stop: only the rule "an abandonment" refuses it (`internal`).
#[test]
fn a_stop_naming_another_kind_of_disposition_is_corrupt() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, task, objects) = abandoned(&scratch)?;
    let acknowledgement = disposition_id(&scratch, &task, "acknowledge_external_effect")?;
    let matching = json!([reference(&objects[1], ID_B, "text/x-log")]).to_string();
    assert_eq!(
        tamper_two(
            &scratch,
            "UPDATE task_dispositions SET evidence=CAST(? AS BLOB) WHERE id=?",
            &matching,
            &acknowledgement,
        )?,
        1
    );
    assert_eq!(
        tamper(
            &scratch,
            "UPDATE events SET body=CAST(json_set(CAST(body AS TEXT),'$.disposition_id',?) AS BLOB) \
             WHERE kind='task_stopped'",
            &acknowledgement,
        )?,
        1
    );
    let kind = get(&tasks, &task, "summary", 3)?;
    assert_eq!(
        kind["code"],
        json!("internal"),
        "another kind of disposition: {kind}"
    );
    conforms(&[("task.get", &kind)])
}

/// B09-E5 · the abandonment's first reference no longer names the stop's object: only the digest rule refuses it (`internal`).
#[test]
fn a_stop_whose_disposition_names_another_object_is_corrupt() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, task, objects) = abandoned(&scratch)?;
    let abandonment = disposition_id(&scratch, &task, "abandon")?;
    let elsewhere = json!([reference(&objects[0], ID_A, "text/plain")]).to_string();
    assert_eq!(
        tamper_two(
            &scratch,
            "UPDATE task_dispositions SET evidence=CAST(? AS BLOB) WHERE id=?",
            &elsewhere,
            &abandonment,
        )?,
        1
    );
    let digest = get(&tasks, &task, "summary", 4)?;
    assert_eq!(
        digest["code"],
        json!("internal"),
        "another object: {digest}"
    );
    conforms(&[("task.get", &digest)])
}

/// B09-E5 · the stop names no disposition: unrecoverable, `unavailable`, `never`.
#[test]
fn a_stop_naming_no_disposition_has_no_recorded_identity() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, task, _) = abandoned(&scratch)?;
    assert_eq!(
        tamper(
            &scratch,
            "UPDATE events SET body=CAST(json_remove(CAST(body AS TEXT),'$.disposition_id') AS BLOB) \
             WHERE kind='task_stopped' AND ?<>''",
            "x",
        )?,
        1
    );
    let unnamed = get(&tasks, &task, "summary", 5)?;
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
    conforms(&[("task.get", &unnamed)])
}

/// B09-E5 · the abandonment's first reference names the stop's object with another size: only the size rule refuses it (`internal`).
#[test]
fn a_stop_whose_disposition_names_another_size_is_corrupt() -> Outcome {
    // The abandonment's first reference names the stop's object with another size: only the
    // size clause refuses it.
    let scratch = Scratch::new()?;
    let (tasks, task, objects) = abandoned(&scratch)?;
    let abandonment = disposition_id(&scratch, &task, "abandon")?;
    let mut resized = reference(&objects[1], ID_B, "text/x-log");
    resized["byte_length"] = json!(objects[1].size() + 1);
    assert_eq!(
        tamper_two(
            &scratch,
            "UPDATE task_dispositions SET evidence=CAST(? AS BLOB) WHERE id=?",
            &json!([resized]).to_string(),
            &abandonment,
        )?,
        1
    );
    let size = get(&tasks, &task, "summary", 6)?;
    assert_eq!(size["code"], json!("internal"), "another size: {size}");
    conforms(&[("task.get", &size)])
}

/// B09-E5 · two tasks abandoned with the same first reference, task 1's stop pointed at task 2's abandonment: only "of that task" refuses it (`internal`).
#[test]
fn a_stop_naming_another_tasks_abandonment_is_corrupt() -> Outcome {
    // Two tasks abandoned with the same first reference; task 1's stop is pointed at task 2's
    // abandonment: kind, digest and size all match, so only "of that task" refuses it.
    let scratch = Scratch::new()?;
    let (tasks, ids, objects) = ledger(
        &scratch,
        &[Stage::Unknown, Stage::Unknown],
        &[b"shared note"],
    )?;
    let operator = operator()?;
    let shared = json!([reference(&objects[0], ID_A, "text/plain")]);
    for (index, task) in ids.iter().enumerate() {
        let attempt = nth(0x05b2, u16::try_from(index + 1)?);
        let key = |offset: usize| KEYS[index * 2 + offset];
        resolve_with(
            &tasks,
            &operator,
            (u8::try_from(index * 2 + 1)?, key(0), task, "3"),
            &attempt,
            "acknowledge_external_effect",
            &json!([]),
        )?;
        let stopped = resolve_with(
            &tasks,
            &operator,
            (u8::try_from(index * 2 + 2)?, key(1), task, "4"),
            &attempt,
            "abandon",
            &shared,
        )?;
        assert_eq!(
            stopped["body"]["task"]["state"],
            json!("abandoned"),
            "{stopped}"
        );
    }
    let other = disposition_id(&scratch, &ids[1], "abandon")?;
    let first_stop = ledger_value(
        &scratch,
        "SELECT event_id FROM task_stops WHERE task_id=?",
        &ids[0],
    )?;
    assert_eq!(
        tamper_two(
            &scratch,
            "UPDATE events SET body=CAST(json_set(CAST(body AS TEXT),'$.disposition_id',?) AS BLOB) WHERE id=?",
            &other,
            first_stop.as_str().ok_or("stop event")?,
        )?,
        1
    );
    let foreign = get(&tasks, &ids[0], "summary", 7)?;
    assert_eq!(
        foreign["code"],
        json!("internal"),
        "another task's abandonment: {foreign}"
    );
    conforms(&[("task.get", &foreign)])
}

/// B09-E8 · the byte bound (R1.4): five distinct 13 MiB objects (65 MiB) are more than one read may
/// hash (64 MiB), refused before any is read with the summary as the route; the summary (no stop
/// here) answers. An operator of another uid sees neither view: the principal door comes first.
#[test]
fn a_view_past_its_byte_bound_routes_to_the_summary() -> Outcome {
    let scratch = Scratch::new()?;
    let bodies: Vec<Vec<u8>> = (0..5_u8)
        .map(|index| vec![b'a' + index; 13 * 1024 * 1024])
        .collect();
    let borrowed: Vec<&[u8]> = bodies.iter().map(Vec::as_slice).collect();
    let (tasks, ids, objects) = ledger(&scratch, &[Stage::Unknown], &borrowed)?;
    let heavy: Vec<Value> = objects
        .iter()
        .enumerate()
        .map(|(index, object)| {
            reference(
                object,
                &format!("07b90000-0000-4000-8000-0000000001{index:02x}"),
                "application/octet-stream",
            )
        })
        .collect();
    let quarantined = resolve_with(
        &tasks,
        &operator()?,
        (1, RESOLVE_KEY, &ids[0], "3"),
        &nth(0x05b2, 1),
        "quarantine",
        &json!(heavy),
    )?;
    assert_eq!(quarantined["kind"], json!("result"), "{quarantined}");
    let refs = get(&tasks, &ids[0], "refs", 2)?;
    assert_eq!(
        (&refs["code"], &refs["retry"], &refs["effect"]),
        (
            &json!("resource_exhausted"),
            &json!("after_readback"),
            &json!("none")
        ),
        "{refs}"
    );
    assert_eq!(refs["readback"]["body"]["evidence"], json!("summary"));
    let summary = get(&tasks, &ids[0], "summary", 3)?;
    assert_eq!(summary["body"]["evidence"], json!([]), "{summary}");
    let stranger = Principal::new(1001, "operator").map_err(|error| format!("{error:?}"))?;
    let hidden = serve(
        &tasks,
        &stranger,
        &request(
            "task.get",
            4,
            None,
            &json!({"selector": {"task_id": ids[0]}, "evidence": "refs"}),
        ),
    )?;
    assert_eq!(hidden["code"], json!("not_found"), "{hidden}");
    conforms(&[
        ("task.get", &refs),
        ("task.get", &summary),
        ("task.get", &hidden),
    ])
}

/// B09-E6 · every disposition's evidence is registered in `artifacts` in the resolve's own
/// transaction (R1.5, a B08 repair: backups copy only registered objects), and never past the
/// backup's 4096-object bound (R2.4), off both edges: filled to 4095, a disposition adding one object
/// is admitted (4096); one adding another is refused `resource_exhausted`, `retry: never`, nothing
/// written; and one adding nothing — no evidence, or objects already registered — is admitted at the
/// full inventory (review D2), so an operator can always clear an obligation.
#[test]
fn a_resolve_registers_its_evidence_within_the_backup_bound() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, ids, objects) = ledger(
        &scratch,
        &[
            Stage::Unknown,
            Stage::Unknown,
            Stage::Unknown,
            Stage::Unknown,
        ],
        &[b"first note", b"second note", b"third note"],
    )?;
    let operator = operator()?;
    let quarantine = |no: u8, key: &str, index: usize, evidence: &Value| {
        resolve_with(
            &tasks,
            &operator,
            (no, key, &ids[index], "3"),
            &nth(0x05b2, u16::try_from(index + 1).unwrap_or(0)),
            "quarantine",
            evidence,
        )
    };
    let count = |sql: &str| ledger_value(&scratch, sql, "x");
    let first = quarantine(
        1,
        KEYS[0],
        0,
        &json!([reference(&objects[0], ID_A, "text/plain")]),
    )?;
    assert_eq!(first["kind"], json!("result"), "{first}");
    assert_eq!(
        ledger_value(
            &scratch,
            "SELECT count(*) FROM artifacts WHERE digest=?",
            objects[0].digest()
        )?,
        json!(1),
        "the disposition's evidence is registered"
    );
    let registered = count("SELECT count(*) FROM artifacts WHERE ?<>''")?
        .as_i64()
        .ok_or("count")?;
    for index in 0..(4095 - registered) {
        assert_eq!(
            tamper(
                &scratch,
                "INSERT INTO artifacts(digest,size) VALUES(?,0)",
                &format!("sha256:{index:064x}")
            )?,
            1
        );
    }
    let at_bound = quarantine(
        2,
        KEYS[1],
        1,
        &json!([reference(&objects[1], ID_B, "text/plain")]),
    )?;
    assert_eq!(
        at_bound["kind"],
        json!("result"),
        "4096 is admitted: {at_bound}"
    );
    assert_eq!(
        count("SELECT count(*) FROM artifacts WHERE ?<>''")?,
        json!(4096)
    );
    let dispositions = count("SELECT count(*) FROM task_dispositions WHERE ?<>''")?;
    let over = quarantine(
        3,
        KEYS[2],
        2,
        &json!([reference(&objects[2], ID_C, "text/plain")]),
    )?;
    assert_eq!(
        (&over["code"], &over["details"]["field"], &over["retry"]),
        (
            &json!("resource_exhausted"),
            &json!("/body/evidence"),
            &json!("never")
        ),
        "{over}"
    );
    assert_eq!(
        count("SELECT count(*) FROM task_dispositions WHERE ?<>''")?,
        dispositions,
        "nothing written"
    );
    assert_eq!(
        count("SELECT count(*) FROM artifacts WHERE ?<>''")?,
        json!(4096)
    );
    conforms(&[
        ("task.resolve", &first),
        ("task.resolve", &at_bound),
        ("task.resolve", &over),
    ])
}

/// B09-E6b · a disposition that adds nothing to the object inventory — no evidence, or an object
/// already registered — is never refused for the inventory's size, even when it is full: an
/// operator can always clear an obligation whoever else grew the inventory (review D2).
#[test]
fn a_resolve_adding_nothing_is_admitted_at_a_full_inventory() -> Outcome {
    let scratch = Scratch::new()?;
    let (tasks, ids, objects) = ledger(
        &scratch,
        &[Stage::Unknown, Stage::Unknown, Stage::Unknown],
        &[b"the one registered note"],
    )?;
    let operator = operator()?;
    let registered = json!([reference(&objects[0], ID_A, "text/plain")]);
    let quarantine = |no: u8, key: &str, index: usize, evidence: &Value| {
        resolve_with(
            &tasks,
            &operator,
            (no, key, &ids[index], "3"),
            &nth(0x05b2, u16::try_from(index + 1).unwrap_or(0)),
            "quarantine",
            evidence,
        )
    };
    let first = quarantine(1, KEYS[0], 0, &registered)?;
    assert_eq!(first["kind"], json!("result"), "{first}");
    let held = ledger_value(&scratch, "SELECT count(*) FROM artifacts WHERE ?<>''", "x")?;
    for index in 0..(4096 - held.as_i64().ok_or("count")?) {
        assert_eq!(
            tamper(
                &scratch,
                "INSERT INTO artifacts(digest,size) VALUES(?,0)",
                &format!("sha256:{index:064x}")
            )?,
            1
        );
    }
    let nothing = quarantine(2, KEYS[1], 1, &json!([]))?;
    assert_eq!(
        nothing["kind"],
        json!("result"),
        "no evidence at a full inventory: {nothing}"
    );
    let again = quarantine(3, KEYS[2], 2, &registered)?;
    assert_eq!(
        again["kind"],
        json!("result"),
        "a registered object at a full inventory: {again}"
    );
    assert_eq!(
        ledger_value(&scratch, "SELECT count(*) FROM artifacts WHERE ?<>''", "x")?,
        json!(4096)
    );
    conforms(&[
        ("task.resolve", &first),
        ("task.resolve", &nothing),
        ("task.resolve", &again),
    ])
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
