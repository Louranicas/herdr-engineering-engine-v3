//! B14a-4 · the native candidate source through the offline contract Rig (R18 proofs (b) and (c)).
//! A child module of `t08_contract.rs`: it borrows the Rig, the fake client and the daemon stand-in,
//! and never opens a network connection.

use super::{LOADED, MODEL, Rig, digest, rendered};
use habitat_engine::app::candidates::{
    ClassPrompt, ClassPromptError, FilePins, NativeCandidates, Outcome, Poll, Refusal, Settle,
    render, settle_children,
};
use habitat_engine::app::runtime::{Ask, Candidate, CandidateSource, Custody, Previous, Readiness};
use habitat_engine::contracts::{Sha256Digest, UuidV4};
use habitat_engine::store::VerificationVerdict;
use habitat_engine::worker::Finish;
use habitat_engine::worker::native::{self, FULL_FILE, MAX_RUN, ProviderState};
use habitat_engine::worker::process::{CleanupPoll, GroupState, WaitOwnership};
use serde_json::json;
use std::cell::Cell;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

const TASK: &[u8] = include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/TASK.md");
const CARGO: &[u8] = include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/base/Cargo.toml");
const BASE: &[u8] = include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/base/src/lib.rs");
const REFERENCE: &str =
    include_str!("../evaluation/tasks/WL-U64-PARSE-001/v1/reference/src/lib.rs");
/// The reviewed closure's workload record: the one pin the prompt's inputs are read against
/// (`files_sha256`), an independent source — the review's, not this test's hashing.
const WORKLOAD_RECORD: &[u8] = include_bytes!(
    "fixtures/reviewed-003/a87e5ba9f699168556ef0859c0690113f0e1186592109dd797745593aff99121"
);
const TASK_ID: &str = "08c40000-0000-4000-8000-000000000001";
const ATTEMPT_ID: &str = "08c40000-0000-4000-8000-000000000002";
const INVOCATION_ID: &str = "08c40000-0000-4000-8000-000000000003";
const RECIPE: &str = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const WORKSPACE: &str = "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const INPUT_TOKENS: u64 = 552;
const OUTPUT_TOKENS: u64 = 258;

/// The closure's pins for the three candidate inputs, read from the workload record.
fn pins() -> FilePins {
    let record: serde_json::Value = serde_json::from_slice(WORKLOAD_RECORD).unwrap();
    let files = &record["files_sha256"];
    let pin = |name: &str| format!("sha256:{}", files[name].as_str().unwrap());
    FilePins {
        task: pin("TASK.md"),
        cargo: pin("base/Cargo.toml"),
        base: pin("base/src/lib.rs"),
    }
}

fn class_prompt() -> ClassPrompt {
    ClassPrompt::new(TASK, CARGO, BASE, &pins()).unwrap()
}

fn ask<'a>(
    previous: Option<&'a Previous>,
    charged_from: Instant,
    work_until: Instant,
    cancelled: &'a AtomicBool,
) -> Ask<'a> {
    Ask {
        task: UuidV4::parse(TASK_ID).unwrap(),
        attempt: UuidV4::parse(ATTEMPT_ID).unwrap(),
        generation: "1".parse().unwrap(),
        invocation: UuidV4::parse(INVOCATION_ID).unwrap(),
        recipe: Sha256Digest::parse(RECIPE).unwrap(),
        workspace: Sha256Digest::parse(WORKSPACE).unwrap(),
        charged_from,
        work_until,
        cancelled,
        previous,
    }
}

/// The Rig under the full-file profile: the fake expects a templated request with the `/2` row's
/// options (literals, never the table), the loaded model reports the `/2` context, and the
/// scenario's prompt is the rendering the source will send.
fn full_file_rig(previous: Option<&Previous>) -> Rig {
    let mut rig = Rig::new();
    rig.scenario["expect"] =
        json!({"raw": false, "options": {"num_ctx": 4096, "num_predict": 1024}});
    rig.scenario["ps"]["models"][0]["context_length"] = json!(4096);
    rig.prompt(&render(&class_prompt(), previous).unwrap());
    rig.scenario["generated"]["prompt_eval_count"] = json!(INPUT_TOKENS);
    rig.scenario["generated"]["eval_count"] = json!(OUTPUT_TOKENS);
    rig
}

fn respond(rig: &mut Rig, response: &str, done_reason: &str) {
    rig.scenario["generated"]["response"] = json!(response);
    rig.scenario["generated"]["done_reason"] = json!(done_reason);
    rig.save();
}

fn captured(rig: &Rig) -> serde_json::Value {
    serde_json::from_slice(&fs::read(rig.root.join("captured-request.json")).unwrap()).unwrap()
}

/// R18 (b) · the source over the fake: the reference file bare is the replacement byte for byte and
/// the request the fake captured is the templated `/2` request whole (raw false, options 4096/1024,
/// the rendered prompt); fenced, the body; `length`, an empty answer, two fences and a second bare
/// call under a previous refused-candidate record each come back by name — and every call's settle
/// is asserted whole (tokens, finish, identity and raw digests, the outcome).
#[test]
fn t08n_01_the_source_hands_the_runtime_the_reference_file_and_names_each_refusal() {
    let cancelled = AtomicBool::new(false);
    let mut rig = full_file_rig(None);
    let mut source = NativeCandidates::new(rig.profile.clone(), FULL_FILE, class_prompt());
    respond(&mut rig, REFERENCE, "stop");
    let work_until = rig.origin + Duration::from_secs(60);
    let answer = source.next(&ask(None, rig.origin, work_until, &cancelled));
    assert_eq!(
        answer.candidate,
        Candidate::Replacement(REFERENCE.as_bytes().to_vec())
    );
    assert_eq!(
        captured(&rig),
        json!({"model": MODEL, "prompt": render(&class_prompt(), None).unwrap(), "stream": false,
               "raw": false, "truncate": false, "shift": false, "keep_alive": 60,
               "options": {"num_ctx": 4096, "num_predict": 1024}})
    );
    let expected_identity_raw = digest(&rendered(&rig.scenario["ps"]));
    let settle = |outcome: Outcome, finish: Option<Finish>, raw: Option<String>| Settle {
        attempt: ATTEMPT_ID.to_owned(),
        adapter: FULL_FILE.id,
        input_tokens: Some(INPUT_TOKENS),
        output_tokens: Some(OUTPUT_TOKENS),
        wall_ms: 0,
        finish,
        identity_sha256: Some(expected_identity_raw.clone()),
        raw_sha256: raw,
        outcome,
    };
    let whole = |s: &Settle| Settle {
        wall_ms: 0,
        ..s.clone()
    };
    // The settle travels with the candidate (R19.3): asserted whole from the answer.
    assert_eq!(
        whole(answer.settle.as_ref().unwrap()),
        settle(
            Outcome::Replacement(REFERENCE.len()),
            Some(Finish::Stop),
            Some(digest(&rendered(&rig.scenario["generated"])))
        )
    );
    // Fenced: the body.
    respond(&mut rig, &format!("```rust\n{REFERENCE}```\n"), "stop");
    assert_eq!(
        source
            .next(&ask(None, rig.origin, work_until, &cancelled))
            .candidate,
        Candidate::Replacement(REFERENCE.as_bytes().to_vec())
    );
    // Truncated: the whole text is refused under its name, the text kept for the record.
    let cut = &REFERENCE[..200];
    respond(&mut rig, cut, "length");
    let truncated = source.next(&ask(None, rig.origin, work_until, &cancelled));
    assert_eq!(
        truncated.candidate,
        Candidate::Refused {
            refusal: Refusal::Truncated,
            text: cut.as_bytes().to_vec()
        }
    );
    assert_eq!(
        whole(truncated.settle.as_ref().unwrap()),
        settle(
            Outcome::Refused(Refusal::Truncated),
            Some(Finish::Length),
            Some(digest(&rendered(&rig.scenario["generated"])))
        )
    );
    respond(&mut rig, "  \n", "stop");
    assert_eq!(
        source
            .next(&ask(None, rig.origin, work_until, &cancelled))
            .candidate,
        Candidate::Refused {
            refusal: Refusal::Empty,
            text: b"  \n".to_vec()
        }
    );
    respond(&mut rig, "```rust\na\n```\nand\n```rust\nb\n```\n", "stop");
    assert!(matches!(
        source
            .next(&ask(None, rig.origin, work_until, &cancelled))
            .candidate,
        Candidate::Refused {
            refusal: Refusal::NotOneFile,
            ..
        }
    ));
}

/// R18 (b) · a second attempt after the runtime's own refused-candidate record: the prompt the fake
/// captured names the refusal (F6), begins with the task text and carries the current editable.
#[test]
fn t08n_03_a_second_attempt_renders_the_previous_refusal_by_name() {
    let cancelled = AtomicBool::new(false);
    let previous = Previous {
        verdict: VerificationVerdict::Failed,
        criteria: 0,
        evidence: br#"{"kind":"refused_candidate","refusal":"candidate_truncated","candidate_sha256":"sha256:00"}"#
            .to_vec(),
        schema_id: "hee3.refused-candidate/1".to_owned(),
    };
    let mut second = full_file_rig(Some(&previous));
    let mut source = NativeCandidates::new(second.profile.clone(), FULL_FILE, class_prompt());
    respond(&mut second, REFERENCE, "stop");
    let work_until = second.origin + Duration::from_secs(60);
    let answer = source.next(&ask(Some(&previous), second.origin, work_until, &cancelled));
    assert_eq!(
        answer.candidate,
        Candidate::Replacement(REFERENCE.as_bytes().to_vec())
    );
    let prompt = captured(&second)["prompt"].as_str().unwrap().to_owned();
    assert!(
        prompt.contains("\n\nPrevious attempt: failed, 0 criteria satisfied, refused as candidate_truncated.\n\n"),
        "{prompt}"
    );
    assert!(prompt.starts_with(std::str::from_utf8(TASK).unwrap()));
    assert!(prompt.contains("Current src/lib.rs:\n"));
    let settle = answer.settle.as_ref().unwrap();
    assert_eq!(settle.outcome, Outcome::Replacement(REFERENCE.len()));
    assert_eq!(settle.adapter, FULL_FILE.id);
    assert_eq!(settle.finish, Some(Finish::Stop));
    assert_eq!(settle.attempt, ATTEMPT_ID);
    assert_eq!(source.retained(), 0);
}

/// R18 (b), (c) · provider failures stop the attempt without a second generate: a loaded model at the
/// qualified `/1` context under the `/2` profile is `Identity` at the readback before generate (A9 —
/// the resident instance is not ours; the fake's calls show no generate ran); a work window past the
/// adapter's own `MAX_RUN` is `Deadline` at the door with no exchange at all (A2, named not
/// clamped); a class prompt whose inputs are not the closure's pinned bytes is refused at
/// construction; every settle carries the failure.
#[test]
fn t08n_02_provider_failures_stop_the_attempt_by_name_without_a_second_generate() {
    let cancelled = AtomicBool::new(false);
    // The loaded model resident at 512 under the /2 profile.
    let mut rig = full_file_rig(None);
    rig.scenario["ps"]["models"][0]["context_length"] = json!(512);
    respond(&mut rig, REFERENCE, "stop");
    let mut source = NativeCandidates::new(rig.profile.clone(), FULL_FILE, class_prompt());
    let work_until = rig.origin + Duration::from_secs(60);
    let failed = source.next(&ask(None, rig.origin, work_until, &cancelled));
    assert_eq!(
        failed.candidate,
        Candidate::Provider {
            error: native::Error::Identity,
            state: ProviderState::NotDispatched,
            cleanup_settled: true,
            retained: 0,
        }
    );
    assert_eq!(
        rig.calls(),
        vec!["version", "tags", "ps"],
        "the readback refused before any generate"
    );
    let settle = failed.settle.as_ref().unwrap();
    assert_eq!(settle.outcome, Outcome::Provider(native::Error::Identity));
    assert_eq!(settle.finish, None);
    assert_eq!(settle.adapter, FULL_FILE.id);
    // A work window past the adapter's cap: refused at the door, no exchange.
    let mut far = full_file_rig(None);
    respond(&mut far, REFERENCE, "stop");
    let mut source = NativeCandidates::new(far.profile.clone(), FULL_FILE, class_prompt());
    let past = far.origin + MAX_RUN + Duration::from_secs(1);
    assert_eq!(
        source
            .next(&ask(None, far.origin, past, &cancelled))
            .candidate,
        Candidate::Provider {
            error: native::Error::Deadline,
            state: ProviderState::NotDispatched,
            cleanup_settled: true,
            retained: 0,
        }
    );
    assert!(far.calls().is_empty(), "{:?}", far.calls());
    let within = far.origin + MAX_RUN;
    assert_eq!(
        source
            .next(&ask(None, far.origin, within, &cancelled))
            .candidate,
        Candidate::Replacement(REFERENCE.as_bytes().to_vec()),
        "the cap itself is inside the door"
    );
    // A class prompt that is not the closure's pinned bytes never reaches a request.
    let mut wrong = pins();
    wrong.base = digest(b"another base");
    assert_eq!(
        ClassPrompt::new(TASK, CARGO, BASE, &wrong).err(),
        Some(ClassPromptError::Base)
    );
    assert_eq!(LOADED, "hee3-t08-contract-loaded:qualification");
}

/// R18 decision 1 · an adapter id the table does not hold is refused at the door as `Profile`, before
/// any exchange — never admitted as the qualified row.
#[test]
fn t08n_04_an_unknown_adapter_id_is_refused_at_the_door() {
    let rig = full_file_rig(None);
    rig.save();
    let mut request = super::full();
    request.adapter_profile = "ollama-fc44-12ff8654/9".into();
    let run = native::execute(
        &request,
        &rig.profile,
        rig.origin,
        rig.origin + Duration::from_secs(60),
        &AtomicBool::new(false),
    );
    assert!(matches!(run, Err(native::Error::Profile)), "{run:?}");
    assert!(rig.calls().is_empty(), "{:?}", rig.calls());
}

/// The fake's control: asked past its scripted answers it exits 3 with a diagnostic, so a second
/// generate the runtime should never make is a `Provider { Process }` failure, never a silent repeat
/// of the last answer.
#[test]
fn t08n_05_the_fake_refuses_a_generate_past_its_scripted_answers() {
    let cancelled = AtomicBool::new(false);
    let mut rig = full_file_rig(None);
    rig.scenario["generated"] = json!([rig.scenario["generated"].clone()]);
    rig.scenario["generated"][0]["response"] = json!(REFERENCE);
    rig.save();
    let mut source = NativeCandidates::new(rig.profile.clone(), FULL_FILE, class_prompt());
    let work_until = rig.origin + Duration::from_secs(60);
    assert_eq!(
        source
            .next(&ask(None, rig.origin, work_until, &cancelled))
            .candidate,
        Candidate::Replacement(REFERENCE.as_bytes().to_vec())
    );
    assert!(matches!(
        source
            .next(&ask(None, rig.origin, work_until, &cancelled))
            .candidate,
        Candidate::Provider {
            error: native::Error::Process,
            ..
        }
    ));
    // The refusal itself, as the double recorded it: the second ask of a one-answer script. Any
    // other process failure leaves no such record.
    let refused: serde_json::Value =
        serde_json::from_slice(&fs::read(rig.root.join("past-list.json")).unwrap()).unwrap();
    assert_eq!(refused, json!({"asked": 2, "scripted": 1}));
    assert_eq!(
        rig.calls()
            .iter()
            .filter(|call| *call == "generate")
            .count(),
        2,
        "{:?}",
        rig.calls()
    );
}

/// The readiness the source must report over a rig: the daemon incarnation the stand-in reader
/// (`DaemonStandIn::daemon`) pinned, the manifest the rig pinned, the one capability, and the
/// catalogue bytes the fake printed.
fn readiness(rig: &Rig) -> Readiness {
    let daemon = &rig.profile.daemon;
    Readiness {
        actual_identity: format!("{}:{}:{}", daemon.boot_id, daemon.pid, daemon.start_ticks),
        immutable_revision: Some(rig.profile.manifest.sha256.clone()),
        capabilities: vec!["text".to_owned()],
        evidence: rendered(&rig.scenario["tags"]),
    }
}

/// R21 N4, N18 · `ready` before an attempt: a model resident at the row's context is ready on the
/// readback alone (no generate); an absent one (DS18's idle `/api/ps`) is loaded — an empty prompt
/// under the row's options — and decided by the `/api/ps` readback after it, with the load's own
/// catalogue as the evidence; a daemon still idle after the load is `Identity`; the caller's
/// deadline and cancellation stop it before any exchange. The two ready rigs differ in the daemon
/// incarnation and the catalogue row, so each readiness is pinned whole against its own fixture.
/// Nothing in the fake leaves a child pending, so `settle_retained` settles nothing (a real
/// retained child is Tier-3, F95; the loop itself is pinned by argument, closure C5 —
/// `the_settle_loop_counts_settled_and_pending_children_by_argument`).
#[test]
fn ready_loads_an_absent_model_and_names_the_catalogue_as_its_evidence() {
    let running = AtomicBool::new(false);
    // Resident at the /2 context already.
    let resident = full_file_rig(None);
    resident.save();
    let mut source = NativeCandidates::new(resident.profile.clone(), FULL_FILE, class_prompt());
    let deadline = resident.origin + Duration::from_secs(60);
    assert_eq!(source.ready(deadline, &running), Ok(readiness(&resident)));
    assert_eq!(resident.calls(), ["version", "tags", "ps"]);
    assert_eq!(
        source.settle_retained(deadline, &running),
        Custody::default()
    );
    // Absent: loaded, then read back.
    let absent = |post: serde_json::Value| {
        let mut rig = full_file_rig(None);
        rig.prompt("");
        rig.scenario["tags"]["models"][0]["modified_at"] = json!("2026-09-22T07:54:00Z");
        rig.scenario["post_ps"] = post;
        rig.scenario["ps"] = json!({"models": []});
        rig.save();
        rig
    };
    let loads = absent(resident.scenario["ps"].clone());
    let mut source = NativeCandidates::new(loads.profile.clone(), FULL_FILE, class_prompt());
    let deadline = loads.origin + Duration::from_secs(60);
    let ready = source.ready(deadline, &running);
    assert_eq!(ready, Ok(readiness(&loads)));
    assert_ne!(ready, Ok(readiness(&resident)), "the fixtures differ");
    assert_eq!(
        loads.calls(),
        ["version", "tags", "ps", "version", "tags", "generate", "ps"]
    );
    assert_eq!(captured(&loads)["prompt"], json!(""));
    assert_eq!(source.retained(), 0);
    assert_eq!(
        source.settle_retained(deadline, &running),
        Custody::default()
    );
    // Still idle after the load: the readback decides.
    let idle = absent(json!({"models": []}));
    let mut source = NativeCandidates::new(idle.profile.clone(), FULL_FILE, class_prompt());
    assert_eq!(
        source.ready(idle.origin + Duration::from_secs(60), &running),
        Err(native::Error::Identity)
    );
    assert_eq!(
        idle.calls(),
        ["version", "tags", "ps", "version", "tags", "generate", "ps"]
    );
    // The caller's deadline and cancellation, before any exchange.
    let stopped = full_file_rig(None);
    stopped.save();
    let mut source = NativeCandidates::new(stopped.profile.clone(), FULL_FILE, class_prompt());
    assert_eq!(
        source.ready(Instant::now(), &running),
        Err(native::Error::Deadline)
    );
    assert_eq!(
        source.ready(
            stopped.origin + Duration::from_secs(60),
            &AtomicBool::new(true)
        ),
        Err(native::Error::Cancelled)
    );
    // The incarnation the readiness would claim is checked first: a stale start is refused by
    // name before any exchange, never reported as the running instance.
    let mut stale = stopped.profile.clone();
    stale.daemon.start_ticks += 1;
    let mut source = NativeCandidates::new(stale, FULL_FILE, class_prompt());
    assert_eq!(
        source.ready(stopped.origin + Duration::from_secs(60), &running),
        Err(native::Error::Identity)
    );
    assert!(stopped.calls().is_empty(), "{:?}", stopped.calls());
}

/// Past this many polls a model child reports its wait lost, so a settle loop that never ends
/// fails the poll-count assertions by name instead of hanging the proof (F102).
const POLL_BUDGET: usize = 1_000;

/// Every poll a model child was handed: its id and the deadline (F101).
type Polls = Arc<Mutex<Vec<(usize, Instant)>>>;

/// A retained child as a model (F101, closure C5): the turn it settles on (`None`: never), whether
/// its wait is lost, a cancel flag it raises when polled, and the shared log of every poll.
struct Child {
    id: usize,
    settles_on: Option<usize>,
    lost: bool,
    raises: Option<Arc<AtomicBool>>,
    polls: Polls,
    turns: usize,
}

impl Poll for Child {
    fn poll_cleanup(&mut self, deadline: Instant) -> CleanupPoll {
        self.turns += 1;
        self.polls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((self.id, deadline));
        if let Some(flag) = &self.raises {
            flag.store(true, Ordering::SeqCst);
        }
        let settled = self.settles_on.is_some_and(|turn| self.turns >= turn);
        CleanupPoll {
            ownership: if self.lost || self.turns > POLL_BUDGET {
                WaitOwnership::Lost
            } else {
                WaitOwnership::Waitable
            },
            leader_terminal: settled,
            exit_code: settled.then_some(0),
            signal: None,
            group: if settled {
                GroupState::Empty
            } else {
                GroupState::Live
            },
        }
    }
}

/// Closure C5 (M5, F129) · the settle loop over retained children, reached by argument: each child
/// polled in order under the caller's deadline, a stepping clock read once per turn, and a pause
/// between turns. Two fixtures differing in every count. (A) three children: one settles on its
/// second turn, one's wait is lost, one never settles and is refused when the clock reaches the
/// deadline (clock 3..=10 ms against a 10 ms deadline: 8 polls) — settled 1, pending 2, kept
/// `[1, 2]`, polls `[2, 1, 8]`, 8 pauses, every poll handed the deadline. (B) four children that each
/// settle on their first turn, the third raising the cancellation as it is polled — settled 3,
/// pending 1, kept `[3]`, the fourth never polled, no pause.
#[test]
fn the_settle_loop_counts_settled_and_pending_children_by_argument() {
    let base = Instant::now();
    let child = |id, settles_on, lost, raises, polls: &Polls| Child {
        id,
        settles_on,
        lost,
        raises,
        polls: Arc::clone(polls),
        turns: 0,
    };
    let per_child = |polls: &Polls, children: usize| {
        let polls = polls.lock().unwrap_or_else(PoisonError::into_inner);
        (0..children)
            .map(|id| polls.iter().filter(|poll| poll.0 == id).count())
            .collect::<Vec<_>>()
    };
    // (A) the stepping clock: 1 ms per read, from `base`.
    let polls: Polls = Arc::new(Mutex::new(Vec::new()));
    let deadline = base + Duration::from_millis(10);
    let tick = Cell::new(0_u64);
    let pauses = Cell::new(0_usize);
    let (custody, kept) = settle_children(
        vec![
            child(0, Some(2), false, None, &polls),
            child(1, None, true, None, &polls),
            child(2, None, false, None, &polls),
        ],
        deadline,
        &AtomicBool::new(false),
        || {
            let now = base + Duration::from_millis(tick.get());
            tick.set(tick.get() + 1);
            now
        },
        || pauses.set(pauses.get() + 1),
    );
    assert_eq!(
        custody,
        Custody {
            settled: 1,
            pending: 2
        }
    );
    assert_eq!(kept.iter().map(|c| c.id).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(per_child(&polls, 3), [2, 1, 8]);
    assert_eq!(pauses.get(), 8);
    assert!(
        polls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .all(|poll| poll.1 == deadline),
        "every poll handed the caller's deadline"
    );
    // (B) a cancellation raised by the third child as it is polled.
    let polls: Polls = Arc::new(Mutex::new(Vec::new()));
    let cancelled = Arc::new(AtomicBool::new(false));
    let pauses = Cell::new(0_usize);
    let (custody, kept) = settle_children(
        vec![
            child(0, Some(1), false, None, &polls),
            child(1, Some(1), false, None, &polls),
            child(2, Some(1), false, Some(Arc::clone(&cancelled)), &polls),
            child(3, Some(1), false, None, &polls),
        ],
        base + Duration::from_secs(60),
        &cancelled,
        || base,
        || pauses.set(pauses.get() + 1),
    );
    assert_eq!(
        custody,
        Custody {
            settled: 3,
            pending: 1
        }
    );
    assert_eq!(kept.iter().map(|c| c.id).collect::<Vec<_>>(), [3]);
    assert_eq!(per_child(&polls, 4), [1, 1, 1, 0]);
    assert_eq!(pauses.get(), 0);
}
