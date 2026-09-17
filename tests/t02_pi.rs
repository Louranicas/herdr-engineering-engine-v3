use habitat_engine::worker::pi::{Frame, Framer, Refusal};

#[test]
fn rejects_duplicate_correlation_keys() {
    assert!(matches!(
        Frame::parse(br#"{"id":"a","id":"b"}"#),
        Err(Refusal::Json)
    ));
}

#[test]
fn eof_fragment_is_not_dispatched() {
    let mut framer = Framer::default();
    assert!(framer.push(b"{}").unwrap().is_empty());
    assert_eq!(framer.finish(), Err(Refusal::Truncated));
}

use habitat_engine::contracts::{Generation, UuidV4};
use habitat_engine::worker::pi::{
    Binding, Command, ModelIdentity, Observation, RunState, Session, Thinking, validate_record,
};
use serde_json::{Value, json};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pi")
}
fn fixture(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join(format!("records/{name}.jsonl"))).unwrap())
        .unwrap()
}
fn frame(value: &Value) -> Frame {
    Frame::parse(&serde_json::to_vec(value).unwrap()).unwrap()
}
fn session() -> Session<'static> {
    Session::new(Binding {
        task: UuidV4::parse("123e4567-e89b-42d3-a456-000000000001").unwrap(),
        attempt: UuidV4::parse("123e4567-e89b-42d3-a456-000000000002").unwrap(),
        generation: "7".parse::<Generation>().unwrap(),
    })
}
fn receive(s: &mut Session<'_>, name: &str, elapsed: u64) -> Observation {
    let mut value = fixture(name);
    if value.get("id").is_some() {
        value["id"] = json!(s.pending_id().unwrap());
    }
    s.observe(&frame(&value), elapsed).unwrap()
}
fn identity() -> ModelIdentity {
    ModelIdentity {
        provider: "fixture-provider".into(),
        id: "fixture-model".into(),
    }
}
fn ready() -> Session<'static> {
    let mut s = session();
    s.issue(Command::State, 0).unwrap();
    receive(&mut s, "state-empty", 0);
    s.issue(Command::SetModel(identity()), 0).unwrap();
    receive(&mut s, "model-selected", 0);
    s.issue(Command::SetThinking(Thinking::High), 0).unwrap();
    receive(&mut s, "thinking-ack", 0);
    s.issue(Command::State, 0).unwrap();
    receive(&mut s, "state-model", 0);
    s
}
fn running() -> Session<'static> {
    let mut s = ready();
    s.issue(Command::Prompt("fictional task".into()), 0)
        .unwrap();
    receive(&mut s, "prompt-ack", 0);
    receive(&mut s, "agent-start", 0);
    receive(&mut s, "turn-start", 0);
    s
}
fn end_messages(s: &mut Session<'_>, names: &[&str], retry: bool, elapsed: u64) {
    let messages: Vec<Value> = names
        .iter()
        .map(|name| fixture(name)["message"].clone())
        .collect();
    let last = messages.last().unwrap();
    s.observe(
        &frame(&json!({"type":"turn_end","message":last,"toolResults":[]})),
        elapsed,
    )
    .unwrap();
    s.observe(
        &frame(&json!({"type":"agent_end","messages":messages,"willRetry":retry})),
        elapsed,
    )
    .unwrap();
}
fn complete_turn(s: &mut Session<'_>, error: bool, elapsed: u64) {
    receive(s, "assistant-start", elapsed);
    let name = if error {
        "assistant-error-end"
    } else {
        "assistant-end"
    };
    receive(s, name, elapsed);
    end_messages(s, &[name], error, elapsed);
}
fn record_bytes(case: &Value) -> Vec<u8> {
    std::fs::read(root().join(case["path"].as_str().unwrap())).unwrap()
}

#[test]
fn independent_record_specimens_match_declared_structural_expectations() {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root().join("manifest.json")).unwrap()).unwrap();
    let mut checked = 0;
    for case in manifest["record_cases"].as_array().unwrap() {
        let layer = case["layer"].as_str().unwrap();
        if !["record", "profile", "framing", "json"].contains(&layer) {
            continue;
        }
        let bytes = record_bytes(case);
        let mut decoder = Framer::default();
        let observed = decoder.push(&bytes).and_then(|frames| {
            decoder.finish()?;
            if ["record", "profile"].contains(&layer) {
                for frame in frames {
                    validate_record(&frame)?;
                }
            }
            Ok(())
        });
        assert_eq!(
            observed.is_ok(),
            case["expected_accept"].as_bool().unwrap(),
            "{}: {observed:?}",
            case["id"]
        );
        checked += 1;
    }
    assert_eq!(checked, 100);
}

#[test]
fn independent_exact_limits_and_next_byte_refusals() {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root().join("manifest.json")).unwrap()).unwrap();
    for case in manifest["boundary_recipes"].as_array().unwrap() {
        let mut bytes = Vec::new();
        for part in case["parts"].as_array().unwrap() {
            let raw = if let Some(ascii) = part.get("ascii") {
                ascii.as_str().unwrap().as_bytes().to_vec()
            } else {
                let row = manifest["record_cases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["id"] == part["fixture_id"])
                    .unwrap();
                record_bytes(row)
            };
            for _ in 0..part["repeat"].as_u64().unwrap_or(1) {
                bytes.extend_from_slice(&raw);
            }
        }
        assert_eq!(
            u64::try_from(bytes.len()).unwrap(),
            case["byte_length"].as_u64().unwrap()
        );
        let mut decoder = Framer::default();
        let observed = decoder.push(&bytes).and_then(|_| decoder.finish());
        assert_eq!(
            observed.is_ok(),
            case["expected_accept"].as_bool().unwrap(),
            "{}: {observed:?}",
            case["id"]
        );
    }
}

#[test]
fn every_byte_split_preserves_unicode_separators() {
    let raw = std::fs::read(root().join("records/unicode-delta.jsonl")).unwrap();
    for split in 0..=raw.len() {
        let mut decoder = Framer::default();
        let mut frames = decoder.push(&raw[..split]).unwrap();
        frames.extend(decoder.push(&raw[split..]).unwrap());
        decoder.finish().unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].bytes(), &raw[..raw.len() - 1]);
        validate_record(&frames[0]).unwrap();
    }
}
#[test]
fn coalesced_records_stay_separate_and_ordered() {
    let mut decoder = Framer::default();
    let frames = decoder
        .push(b"{\"type\":\"agent_start\"}\n{\"type\":\"agent_settled\"}\n")
        .unwrap();
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].bytes(), br#"{"type":"agent_start"}"#);
    decoder.finish().unwrap();
}
#[test]
fn decoder_cannot_resume_after_clean_eof() {
    let mut d = Framer::default();
    d.finish().unwrap();
    assert!(matches!(d.push(b"{}\n"), Err(Refusal::Poisoned)));
}
#[test]
fn decoder_does_not_resynchronize_after_a_bad_line() {
    let mut d = Framer::default();
    assert!(d.push(b"garbage\n").is_err());
    assert!(matches!(d.push(b"{}\n"), Err(Refusal::Poisoned)));
}
#[test]
fn all_state_flags_are_validated_even_when_streaming() {
    let mut value = fixture("state-model");
    value["data"]["isStreaming"] = json!(true);
    value["data"]["isCompacting"] = json!(1);
    assert_eq!(validate_record(&frame(&value)), Err(Refusal::Shape));
}
#[test]
fn floating_and_exponent_counter_tokens_are_refused() {
    let raw = std::fs::read_to_string(root().join("records/state-empty.jsonl")).unwrap();
    for token in ["0.0", "0e0", "-0"] {
        let changed = raw.replace("\"messageCount\":0", &format!("\"messageCount\":{token}"));
        assert_eq!(
            validate_record(&Frame::parse(changed.trim_end().as_bytes()).unwrap()),
            Err(Refusal::Number),
            "{token}"
        );
    }
}
#[test]
fn actual_sdk_sentinel_is_unavailable_metadata() {
    let raw = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("evidence/implementation/T02/sdk-smoke/metadata-benign.stdout"),
    )
    .unwrap();
    let mut decoder = Framer::default();
    let frames = decoder.push(&raw).unwrap();
    decoder.finish().unwrap();
    assert_eq!(frames.len(), 2);
    for f in frames {
        validate_record(&f).unwrap();
    }
    let mut s = session();
    s.issue(Command::State, 0).unwrap();
    match receive(&mut s, "state-unavailable-sentinel", 0) {
        Observation::State(state) => assert!(state.model.is_none()),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        s.issue(Command::Prompt("task".into()), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn prompt_acknowledgement_is_not_execution() {
    let mut s = ready();
    s.issue(Command::Prompt("task".into()), 0).unwrap();
    assert_eq!(receive(&mut s, "prompt-ack", 0), Observation::Acknowledged);
    assert_eq!(s.run_state(), RunState::AwaitingStart);
}
#[test]
fn prompt_requires_final_readback_after_configuration() {
    let mut s = ready();
    s.issue(Command::SetThinking(Thinking::High), 0).unwrap();
    receive(&mut s, "thinking-ack", 0);
    assert_eq!(
        s.issue(Command::Prompt("task".into()), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn response_id_mismatch_permanently_refuses_the_attempt() {
    let mut s = session();
    s.issue(Command::State, 0).unwrap();
    assert_eq!(
        s.observe(&frame(&fixture("response-wrong-id")), 0),
        Err(Refusal::Correlation)
    );
    assert_eq!(s.run_state(), RunState::Refused);
    assert_eq!(s.issue(Command::State, 0), Err(Refusal::Poisoned));
}
#[test]
fn exact_id_with_wrong_command_is_not_correlated() {
    let mut s = session();
    s.issue(Command::State, 0).unwrap();
    let value =
        json!({"id":s.pending_id().unwrap(),"type":"response","command":"abort","success":true});
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Correlation));
}
#[test]
fn unsolicited_response_has_no_owner() {
    let mut s = session();
    assert_eq!(
        s.observe(&frame(&fixture("state-empty")), 0),
        Err(Refusal::Correlation)
    );
}
#[test]
fn command_ids_do_not_repeat_after_readback() {
    let mut s = session();
    let first = s.issue(Command::State, 0).unwrap();
    receive(&mut s, "state-empty", 0);
    let second = s.issue(Command::State, 0).unwrap();
    assert_ne!(first, second);
    let mut next_generation = Session::new(Binding {
        generation: "8".parse().unwrap(),
        ..s.binding()
    });
    assert_ne!(first, next_generation.issue(Command::State, 0).unwrap());
    let mut other_task = Session::new(Binding {
        task: UuidV4::parse("123e4567-e89b-42d3-a456-000000000003").unwrap(),
        ..s.binding()
    });
    assert_ne!(first, other_task.issue(Command::State, 0).unwrap());
}
#[test]
fn changed_session_cannot_inherit_the_attempt() {
    let mut s = ready();
    s.issue(Command::State, 0).unwrap();
    let mut value = fixture("state-model");
    value["id"] = json!(s.pending_id().unwrap());
    value["data"]["sessionId"] = json!("foreign");
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Session));
}
#[test]
fn initial_owned_session_must_be_empty_and_idle() {
    let mut s = session();
    s.issue(Command::State, 0).unwrap();
    let mut value = fixture("state-after-run");
    value["id"] = json!(s.pending_id().unwrap());
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Session));
}
#[test]
fn model_readback_mismatch_refuses_prompt_readiness() {
    let mut s = ready();
    s.issue(Command::State, 0).unwrap();
    let mut value = fixture("state-foreign-model");
    value["id"] = json!(s.pending_id().unwrap());
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Model));
}
#[test]
fn thinking_clamp_is_detected_by_readback() {
    let mut s = ready();
    s.issue(Command::State, 0).unwrap();
    let mut value = fixture("state-model");
    value["id"] = json!(s.pending_id().unwrap());
    value["data"]["thinkingLevel"] = json!("off");
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Model));
}
#[test]
fn preflight_refusal_remains_a_refusal() {
    let mut s = ready();
    s.issue(Command::Prompt("task".into()), 0).unwrap();
    assert!(matches!(
        receive(&mut s, "prompt-refused", 0),
        Observation::Rejected(_)
    ));
    assert_eq!(s.run_state(), RunState::Refused);
}
#[test]
fn agent_end_does_not_settle_even_without_retry() {
    let mut s = running();
    complete_turn(&mut s, false, 0);
    assert_eq!(s.run_state(), RunState::Ended);
}
#[test]
fn retry_continues_under_the_original_clock() {
    let mut s = running();
    complete_turn(&mut s, true, 899_000);
    receive(&mut s, "retry-start", 899_100);
    receive(&mut s, "agent-start", 900_000);
    assert_eq!(s.run_state(), RunState::Running);
    assert_eq!(
        s.observe(&frame(&fixture("agent-end-no-retry")), 1_200_000),
        Err(Refusal::Deadline)
    );
}
#[test]
fn unsupported_compaction_after_end_cannot_settle() {
    let mut s = running();
    complete_turn(&mut s, false, 0);
    assert!(
        s.observe(&frame(&fixture("unsupported-compaction")), 0)
            .is_err()
    );
    assert_eq!(s.run_state(), RunState::Refused);
}
#[test]
fn stale_settled_has_no_active_run() {
    let mut s = ready();
    assert_eq!(
        s.observe(&frame(&fixture("agent-settled")), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn cancellation_serializes_clear_abort_and_idle_readback() {
    let mut s = running();
    s.issue(Command::ClearQueue, 0).unwrap();
    assert_eq!(s.issue(Command::Abort, 0), Err(Refusal::Ordering));
    receive(&mut s, "queue-empty", 0);
    receive(&mut s, "clear-removed", 0);
    s.issue(Command::Abort, 0).unwrap();
    complete_turn(&mut s, false, 0);
    receive(&mut s, "agent-settled", 0);
    receive(&mut s, "abort-ack", 0);
    s.issue(Command::State, 0).unwrap();
    assert_eq!(
        receive(&mut s, "state-after-run", 0),
        Observation::CancelledPiIdle
    );
    // This API deliberately has no process/descendant/workspace or task-acceptance result.
}
#[test]
fn abort_cannot_skip_clearing_the_queues() {
    let mut s = running();
    assert_eq!(s.issue(Command::Abort, 0), Err(Refusal::Ordering));
}
#[test]
fn idle_readback_without_settled_does_not_finish_active_cancellation() {
    let mut s = running();
    s.issue(Command::ClearQueue, 0).unwrap();
    receive(&mut s, "clear-empty", 0);
    s.issue(Command::Abort, 0).unwrap();
    receive(&mut s, "abort-ack", 0);
    s.issue(Command::State, 0).unwrap();
    assert!(matches!(
        receive(&mut s, "state-after-run", 0),
        Observation::State(_)
    ));
    assert_eq!(s.run_state(), RunState::Running);
}
#[test]
fn nonempty_current_queue_is_not_the_removed_queue_response() {
    let mut s = running();
    assert_eq!(
        s.observe(&frame(&fixture("queue-nonempty")), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn final_message_overrides_provisional_delta_text() {
    let mut s = running();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "text-delta", 0);
    let Observation::FinalMessage(value) = receive(&mut s, "assistant-end", 0) else {
        panic!("missing final message")
    };
    assert_eq!(value, fixture("assistant-end")["message"]);
    assert_ne!(
        value["content"][0]["text"],
        fixture("text-delta")["assistantMessageEvent"]["delta"]
    );
}
#[test]
fn duplicate_message_end_cannot_double_count_usage() {
    let mut s = running();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-end", 0);
    assert_eq!(
        s.observe(&frame(&fixture("assistant-end")), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn usage_stats_preserve_subset_accounting_and_unknown_context() {
    let mut s = running();
    s.issue(Command::Stats, 0).unwrap();
    let Observation::ProvisionalUsage(usage) = receive(&mut s, "stats-unknown-context", 0) else {
        panic!("missing usage")
    };
    assert_eq!(usage.total, 39);
    assert_eq!(usage.context_tokens, None);
}
#[test]
fn foreign_stats_cannot_be_attributed_to_this_attempt() {
    let mut s = running();
    s.issue(Command::Stats, 0).unwrap();
    let mut value = fixture("stats-wrong-session");
    value["id"] = json!(s.pending_id().unwrap());
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Session));
}
#[test]
fn settled_session_totals_must_reconcile_authoritative_messages() {
    let mut s = running();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-end", 0);
    end_messages(&mut s, &["assistant-end"], false, 0);
    receive(&mut s, "agent-settled", 0);
    s.issue(Command::State, 0).unwrap();
    receive(&mut s, "state-after-run", 0);
    s.issue(Command::Stats, 0).unwrap();
    receive(&mut s, "stats-known", 0);
    s.issue(Command::Stats, 0).unwrap();
    let mut value = fixture("stats-known");
    value["id"] = json!(s.pending_id().unwrap());
    value["data"]["tokens"]["input"] = json!(21);
    value["data"]["tokens"]["total"] = json!(40);
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Usage));
}
#[test]
fn cleanup_reserve_prevents_a_new_prompt() {
    let mut s = ready();
    assert_eq!(
        s.issue(Command::Prompt("task".into()), 900_000),
        Err(Refusal::Deadline)
    );
}
#[test]
fn observed_clock_cannot_rewind_to_reopen_work() {
    let mut s = ready();
    s.issue(Command::State, 1_000).unwrap();
    assert_eq!(
        s.observe(&frame(&fixture("state-model")), 999),
        Err(Refusal::ClockRewind)
    );
}
#[test]
fn process_failure_cannot_be_overridden_by_later_vendor_success() {
    let mut s = ready();
    s.transport_failed();
    assert_eq!(
        s.observe(&frame(&fixture("agent-settled")), 0),
        Err(Refusal::Poisoned)
    );
}

#[test]
fn completion_delta_rejects_unknown_reason() {
    let mut value = fixture("update-done");
    value["assistantMessageEvent"]["reason"] = json!("invented");
    assert_eq!(validate_record(&frame(&value)), Err(Refusal::Unsupported));
}
#[test]
fn completion_delta_reason_must_match_its_message() {
    let mut value = fixture("update-done");
    value["assistantMessageEvent"]["reason"] = json!("length");
    assert_eq!(validate_record(&frame(&value)), Err(Refusal::Shape));
}
#[test]
fn error_delta_rejects_success_reason() {
    let mut value = fixture("update-error");
    value["assistantMessageEvent"]["reason"] = json!("stop");
    assert_eq!(validate_record(&frame(&value)), Err(Refusal::Unsupported));
}
#[test]
fn error_delta_reason_must_match_its_message() {
    let mut value = fixture("update-error");
    value["assistantMessageEvent"]["reason"] = json!("aborted");
    assert_eq!(validate_record(&frame(&value)), Err(Refusal::Shape));
}
#[test]
fn retry_end_attempt_must_fit_the_selected_cap() {
    let mut value = fixture("retry-success");
    value["attempt"] = json!(4);
    assert_eq!(validate_record(&frame(&value)), Err(Refusal::Number));
}
#[test]
fn announced_retry_prevents_early_settlement() {
    let mut s = running();
    complete_turn(&mut s, true, 0);
    assert_eq!(
        s.observe(&frame(&fixture("agent-settled")), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn retry_start_requires_the_preceding_retry_announcement() {
    let mut s = running();
    complete_turn(&mut s, false, 0);
    assert_eq!(
        s.observe(&frame(&fixture("retry-start")), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn agent_continuation_requires_a_scheduled_retry() {
    let mut s = running();
    complete_turn(&mut s, false, 0);
    assert_eq!(
        s.observe(&frame(&fixture("agent-start")), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn first_retry_cannot_skip_attempt_one() {
    let mut s = running();
    complete_turn(&mut s, true, 0);
    let mut value = fixture("retry-start");
    value["attempt"] = json!(2);
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Ordering));
}
fn retrying() -> Session<'static> {
    let mut s = running();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-error-end", 0);
    end_messages(&mut s, &["assistant-error-end"], true, 0);
    receive(&mut s, "retry-start", 0);
    receive(&mut s, "agent-start", 0);
    receive(&mut s, "turn-start", 0);
    s
}
#[test]
fn retry_success_requires_a_completed_assistant_message() {
    let mut s = retrying();
    assert_eq!(
        s.observe(&frame(&fixture("retry-success")), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn retry_success_is_intermediate_before_final_end_and_settled() {
    let mut s = retrying();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-end", 0);
    receive(&mut s, "retry-success", 0);
    assert_eq!(s.run_state(), RunState::Running);
    end_messages(&mut s, &["assistant-end"], false, 0);
    assert_eq!(receive(&mut s, "agent-settled", 0), Observation::PiSettled);
}
#[test]
fn cancellation_during_retry_delay_can_settle_after_failed_retry_end() {
    let mut s = running();
    complete_turn(&mut s, true, 0);
    receive(&mut s, "retry-start", 0);
    receive(&mut s, "retry-cancelled", 0);
    assert_eq!(receive(&mut s, "agent-settled", 0), Observation::PiSettled);
}
#[test]
fn retry_budget_cannot_change_between_attempts() {
    let mut s = retrying();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-error-end", 0);
    end_messages(&mut s, &["assistant-error-end"], true, 0);
    let mut value = fixture("retry-start");
    value["attempt"] = json!(2);
    value["maxAttempts"] = json!(2);
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Ordering));
}
#[test]
fn failed_final_retry_requires_its_end_before_settled() {
    let mut s = retrying();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-error-end", 0);
    end_messages(&mut s, &["assistant-error-end"], false, 0);
    assert_eq!(
        s.observe(&frame(&fixture("agent-settled")), 0),
        Err(Refusal::Ordering)
    );
}

#[test]
fn mismatched_agent_end_cannot_replace_completed_messages() {
    let mut s = running();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-end", 0);
    let message = fixture("assistant-end")["message"].clone();
    s.observe(
        &frame(&json!({"type":"turn_end","message":message,"toolResults":[]})),
        0,
    )
    .unwrap();
    let mut substituted = message;
    substituted["content"][0]["text"] = json!("substituted final");
    let value = json!({"type":"agent_end","messages":[substituted],"willRetry":false});
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Session));
}
#[test]
fn turn_end_must_repeat_the_exact_final_assistant() {
    let mut s = running();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-end", 0);
    let mut message = fixture("assistant-end")["message"].clone();
    message["timestamp"] = json!(1002);
    assert_eq!(
        s.observe(
            &frame(&json!({"type":"turn_end","message":message,"toolResults":[]})),
            0
        ),
        Err(Refusal::Session)
    );
}
#[test]
fn message_events_require_an_open_turn() {
    let mut s = ready();
    s.issue(Command::Prompt("task".into()), 0).unwrap();
    receive(&mut s, "prompt-ack", 0);
    receive(&mut s, "agent-start", 0);
    assert_eq!(
        s.observe(&frame(&fixture("assistant-start")), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn duplicate_turn_start_is_not_new_work() {
    let mut s = running();
    assert_eq!(
        s.observe(&frame(&fixture("turn-start")), 0),
        Err(Refusal::Ordering)
    );
}
#[test]
fn agent_end_requires_a_closed_turn() {
    let mut s = running();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-end", 0);
    let value = json!({"type":"agent_end","messages":[fixture("assistant-end")["message"]],"willRetry":false});
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Ordering));
}
#[test]
fn settled_usage_stays_provisional_until_idle_readback() {
    let mut s = running();
    complete_turn(&mut s, false, 0);
    receive(&mut s, "agent-settled", 0);
    s.issue(Command::Stats, 0).unwrap();
    assert!(matches!(
        receive(&mut s, "stats-known", 0),
        Observation::ProvisionalUsage(_)
    ));
    s.issue(Command::State, 0).unwrap();
    receive(&mut s, "state-after-run", 0);
    s.issue(Command::Stats, 0).unwrap();
    assert!(matches!(
        receive(&mut s, "stats-known", 0),
        Observation::ReconciledUsage(_)
    ));
}
#[test]
fn pending_assistant_cannot_become_a_final_message() {
    let mut s = running();
    receive(&mut s, "assistant-start", 0);
    let mut value = fixture("assistant-end");
    value["message"]["stopReason"] = json!("pending");
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Ordering));
}
#[test]
fn successful_final_message_cannot_announce_an_error_retry() {
    let mut s = running();
    receive(&mut s, "assistant-start", 0);
    receive(&mut s, "assistant-end", 0);
    let message = fixture("assistant-end")["message"].clone();
    s.observe(
        &frame(&json!({"type":"turn_end","message":message,"toolResults":[]})),
        0,
    )
    .unwrap();
    let value = json!({"type":"agent_end","messages":[message],"willRetry":true});
    assert_eq!(s.observe(&frame(&value), 0), Err(Refusal::Ordering));
}
