//! B14a-4 · the native candidate source through the offline contract Rig (R18 proofs (b) and (c)).
//! A child module of `t08_contract.rs`: it borrows the Rig, the fake client and the daemon stand-in,
//! and never opens a network connection.

use super::{LOADED, MODEL, Rig, digest, rendered};
use habitat_engine::app::candidates::{
    ClassPrompt, FilePins, NativeCandidates, Outcome, Refusal, Settle, render,
};
use habitat_engine::app::runtime::{Ask, Candidate, CandidateSource, Previous};
use habitat_engine::contracts::{Sha256Digest, UuidV4};
use habitat_engine::store::VerificationVerdict;
use habitat_engine::worker::Finish;
use habitat_engine::worker::native::{self, FULL_FILE, MAX_RUN, ProviderState};
use serde_json::json;
use std::fs;
use std::sync::atomic::AtomicBool;
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
    origin: Instant,
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
        origin,
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
    let candidate = source.next(&ask(None, rig.origin, work_until, &cancelled));
    assert_eq!(
        candidate,
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
    assert_eq!(
        whole(&source.settles()[0]),
        settle(
            Outcome::Replacement(REFERENCE.len()),
            Some(Finish::Stop),
            Some(digest(&rendered(&rig.scenario["generated"])))
        )
    );
    // Fenced: the body.
    respond(&mut rig, &format!("```rust\n{REFERENCE}```\n"), "stop");
    assert_eq!(
        source.next(&ask(None, rig.origin, work_until, &cancelled)),
        Candidate::Replacement(REFERENCE.as_bytes().to_vec())
    );
    // Truncated: the whole text is refused under its name, the text kept for the record.
    let cut = &REFERENCE[..200];
    respond(&mut rig, cut, "length");
    assert_eq!(
        source.next(&ask(None, rig.origin, work_until, &cancelled)),
        Candidate::Refused {
            refusal: Refusal::Truncated,
            text: cut.as_bytes().to_vec()
        }
    );
    assert_eq!(
        whole(&source.settles()[2]),
        settle(
            Outcome::Refused(Refusal::Truncated),
            Some(Finish::Length),
            Some(digest(&rendered(&rig.scenario["generated"])))
        )
    );
    respond(&mut rig, "  \n", "stop");
    assert_eq!(
        source.next(&ask(None, rig.origin, work_until, &cancelled)),
        Candidate::Refused {
            refusal: Refusal::Empty,
            text: b"  \n".to_vec()
        }
    );
    respond(&mut rig, "```rust\na\n```\nand\n```rust\nb\n```\n", "stop");
    assert!(matches!(
        source.next(&ask(None, rig.origin, work_until, &cancelled)),
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
    assert_eq!(
        source.next(&ask(Some(&previous), second.origin, work_until, &cancelled)),
        Candidate::Replacement(REFERENCE.as_bytes().to_vec())
    );
    let prompt = captured(&second)["prompt"].as_str().unwrap().to_owned();
    assert!(
        prompt.contains("\n\nPrevious attempt: failed, 0 criteria satisfied, refused as candidate_truncated.\n\n"),
        "{prompt}"
    );
    assert!(prompt.starts_with(std::str::from_utf8(TASK).unwrap()));
    assert!(prompt.contains("Current src/lib.rs:\n"));
    assert_eq!(source.settles().len(), 1);
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
    assert_eq!(
        source.next(&ask(None, rig.origin, work_until, &cancelled)),
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
    assert_eq!(
        source.settles()[0].outcome,
        Outcome::Provider(native::Error::Identity)
    );
    assert_eq!(source.settles()[0].finish, None);
    // A work window past the adapter's cap: refused at the door, no exchange.
    let mut far = full_file_rig(None);
    respond(&mut far, REFERENCE, "stop");
    let mut source = NativeCandidates::new(far.profile.clone(), FULL_FILE, class_prompt());
    let past = far.origin + MAX_RUN + Duration::from_secs(1);
    assert_eq!(
        source.next(&ask(None, far.origin, past, &cancelled)),
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
        source.next(&ask(None, far.origin, within, &cancelled)),
        Candidate::Replacement(REFERENCE.as_bytes().to_vec()),
        "the cap itself is inside the door"
    );
    // A class prompt that is not the closure's pinned bytes never reaches a request.
    let mut wrong = pins();
    wrong.base = digest(b"another base");
    assert_eq!(
        ClassPrompt::new(TASK, CARGO, BASE, &wrong).err(),
        Some(Refusal::Prompt("base"))
    );
    assert_eq!(LOADED, "hee3-t08-contract-loaded:qualification");
}
