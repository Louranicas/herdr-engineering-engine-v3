//! T16 herdr client-surface cases (`T16-HD-nn`). Every case builds its world as values.
//! There is no pane, socket, process or clock here.
//!
//! The contract's required proof is *"pane loss and reconnect; duplicate UI submission;
//! stale event; explicit cancellation; truthful failure/unknown presentation and proof
//! navigation"*.

use std::error::Error;

use habitat_engine::herdr::{
    Admitted, EngineReceipt, IntentKey, MAX_BUFFERED_EVENTS, MAX_RECONNECTS, Refusal,
    SCHEMA_VERSION, Snapshot, Status, View,
};

type Outcome = Result<(), Box<dyn Error>>;

fn id(index: usize) -> String {
    format!("{index:08x}-0000-4000-8000-000000000000")
}

fn snapshot(task: &str, status: Status, sequence: u64) -> Snapshot {
    Snapshot {
        task: task.to_owned(),
        status,
        route_explanation: None,
        evidence: Vec::new(),
        gaps: Vec::new(),
        sequence,
    }
}

// ------------------------------------------- acceptance cannot be minted by a client

/// T16-HD-01 · an acceptance exists only via an engine receipt. This case pins the one
/// legitimate path; the absence of any other is what the rest of the family rests on.
#[test]
fn acceptance_comes_only_from_an_engine_receipt() -> Outcome {
    let task = id(1);
    let receipt = EngineReceipt::issue(&task, 1, 7)?;
    let admitted = Admitted::from_engine(receipt);
    let acceptance = admitted.acceptance().ok_or("expected an acceptance")?;
    assert_eq!(acceptance.task().as_str(), task);
    assert_eq!(acceptance.receipt().sequence(), 7);
    assert_eq!(acceptance.receipt().epoch(), 1);
    assert!(admitted.is_durable());
    Ok(())
}

/// T16-HD-02 · a denial carries no acceptance at all, so a client rendering it has nothing
/// to mistake for one.
#[test]
fn a_denial_carries_no_acceptance() {
    let denied: Admitted<'_> = Admitted::Denied;
    assert!(denied.acceptance().is_none());
    assert!(!denied.is_durable());
}

/// T16-HD-03 · a malformed task identity is refused when the receipt is issued, before any
/// client sees it.
#[test]
fn a_malformed_task_identity_is_refused_at_issue() {
    use habitat_engine::contracts::ScalarError;
    assert_eq!(
        EngineReceipt::issue("not-a-uuid", 1, 1).map(|_| ()),
        Err(Refusal::MalformedIdentity(ScalarError::InvalidUuid))
    );
}

/// T16-HD-04 · the four presented statuses that are not settled cannot be read as settled.
/// `Unknown` and `CancellationPending` are explicitly unsettled: a client that renders either
/// as an outcome is inventing one.
#[test]
fn only_passed_and_failed_are_settled() {
    assert_eq!(Status::ALL.len(), 5);
    let settled: Vec<&str> = Status::ALL
        .into_iter()
        .filter(|status| status.is_settled())
        .map(Status::name)
        .collect();
    assert_eq!(settled, vec!["passed", "failed"]);
    for status in Status::ALL {
        assert_eq!(status.to_string(), status.name());
    }
}

/// T16-HD-05 · an unknown status presents as unknown and never as a verdict, in a view.
#[test]
fn an_unknown_status_presents_as_unknown() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(snapshot(&task, Status::Unknown, 1))?;
    let presented = view.task(&task)?;
    assert_eq!(presented.status, Status::Unknown);
    assert!(!presented.status.is_settled());
    Ok(())
}

// ------------------------------------------- duplicate submission

/// T16-HD-06 · a repeated submission under the same intent key reports `Duplicate`, carrying
/// the same acceptance — one durable task, not two.
#[test]
fn a_repeated_submission_is_a_duplicate_not_a_second_task() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    let key = IntentKey::new("operator-enter-1");
    let first = view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 1)?),
    )?;
    assert!(matches!(first, Admitted::Accepted(_)));
    let second = view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 2)?),
    )?;
    assert!(matches!(second, Admitted::Duplicate(_)));
    assert!(
        second.is_durable(),
        "the task exists, it was just not created twice"
    );
    assert_eq!(view.submitted(&key), Some(task.as_str()));
    Ok(())
}

/// T16-HD-07 · two different intent keys are two submissions, even for the same task text.
#[test]
fn different_intent_keys_are_different_submissions() -> Outcome {
    let mut view = View::new(1);
    let (first, second) = (id(1), id(2));
    let a = view.submit(
        &IntentKey::new("first"),
        Admitted::from_engine(EngineReceipt::issue(&first, 1, 1)?),
    )?;
    let b = view.submit(
        &IntentKey::new("second"),
        Admitted::from_engine(EngineReceipt::issue(&second, 1, 2)?),
    )?;
    assert!(matches!(a, Admitted::Accepted(_)));
    assert!(matches!(b, Admitted::Accepted(_)));
    Ok(())
}

/// T16-HD-08 · a denial is not recorded as a submission, so a retry after a denial is a
/// fresh attempt rather than a duplicate.
#[test]
fn a_denial_does_not_register_the_key() -> Outcome {
    let mut view = View::new(1);
    let key = IntentKey::new("k");
    let denied = view.submit(&key, Admitted::Denied)?;
    assert!(matches!(denied, Admitted::Denied));
    assert_eq!(view.submitted(&key), None);
    let task = id(1);
    let retried = view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 1)?),
    )?;
    assert!(
        matches!(retried, Admitted::Accepted(_)),
        "a retry after denial is fresh"
    );
    Ok(())
}

/// T16-HD-09 · a receipt from another epoch is refused at submission: a view cannot record
/// an acceptance that belongs to a world it is not attached to.
#[test]
fn a_receipt_from_another_epoch_is_refused() -> Outcome {
    let mut view = View::new(1);
    assert_eq!(
        view.submit(
            &IntentKey::new("k"),
            Admitted::from_engine(EngineReceipt::issue(&id(1), 2, 1)?)
        )
        .map(|_| ()),
        Err(Refusal::EpochMismatch)
    );
    Ok(())
}

/// T16-HD-10 · an intent key round-trips its text.
#[test]
fn an_intent_key_round_trips() {
    let key = IntentKey::new("pane-3:enter:1712");
    assert_eq!(key.as_str(), "pane-3:enter:1712");
    assert_eq!(IntentKey::new("a"), IntentKey::new("a"));
    assert_ne!(IntentKey::new("a"), IntentKey::new("b"));
}

// ------------------------------------------- stale events

/// T16-HD-11 · an event at or before the cursor is refused as stale, so a replayed event is
/// not rendered as new.
#[test]
fn an_event_at_or_before_the_cursor_is_stale() -> Outcome {
    let mut view = View::new(1);
    view.observe(1, 5, "five")?;
    assert_eq!(view.cursor(), 5);
    assert_eq!(view.observe(1, 5, "five again"), Err(Refusal::StaleEvent));
    assert_eq!(view.observe(1, 4, "four"), Err(Refusal::StaleEvent));
    assert_eq!(view.buffered(), 1, "neither stale event was buffered");
    Ok(())
}

/// T16-HD-12 · the staleness boundary is asserted from both sides: cursor + 1 is accepted.
#[test]
fn the_staleness_boundary_admits_the_next_sequence() -> Outcome {
    let mut view = View::new(1);
    view.observe(1, 5, "five")?;
    view.observe(1, 6, "six")?;
    assert_eq!(view.cursor(), 6);
    assert_eq!(view.buffered(), 2);
    Ok(())
}

/// T16-HD-13 · an event from another epoch is refused; the client must reconnect.
#[test]
fn an_event_from_another_epoch_is_refused() {
    let mut view = View::new(1);
    assert_eq!(view.observe(2, 1, "x"), Err(Refusal::EpochMismatch));
    assert_eq!(view.buffered(), 0);
}

/// T16-HD-14 · draining takes the buffer and leaves it empty, without moving the cursor —
/// rendering is not acknowledgement.
#[test]
fn draining_empties_the_buffer_without_moving_the_cursor() -> Outcome {
    let mut view = View::new(1);
    view.observe(1, 1, "a")?;
    view.observe(1, 2, "b")?;
    let drained = view.drain();
    assert_eq!(drained, vec![(1, "a".to_owned()), (2, "b".to_owned())]);
    assert_eq!(view.buffered(), 0);
    assert_eq!(view.cursor(), 2, "the cursor is where the client read to");
    Ok(())
}

/// T16-HD-15 · the buffer bound refuses before the event is stored, so a slow renderer
/// cannot make the client grow without bound.
#[test]
fn the_buffer_bound_refuses_before_storing() -> Outcome {
    let mut view = View::new(1);
    for index in 1..=MAX_BUFFERED_EVENTS {
        view.observe(1, u64::try_from(index)?, "x")?;
    }
    assert_eq!(view.buffered(), MAX_BUFFERED_EVENTS);
    assert_eq!(
        view.observe(1, u64::try_from(MAX_BUFFERED_EVENTS + 1)?, "one too many"),
        Err(Refusal::BufferFull)
    );
    assert_eq!(view.buffered(), MAX_BUFFERED_EVENTS);
    assert_eq!(
        view.cursor(),
        u64::try_from(MAX_BUFFERED_EVENTS)?,
        "and the refused event did not advance the cursor"
    );
    Ok(())
}

// ------------------------------------------- pane loss and reconnect

/// T16-HD-16 · reconnecting in the same epoch takes the engine's cursor and discards the
/// buffer: anything the client retained across the loss is what it cannot vouch for.
#[test]
fn reconnecting_takes_the_engine_cursor_and_drops_the_buffer() -> Outcome {
    let mut view = View::new(1);
    view.observe(1, 1, "a")?;
    view.observe(1, 2, "b")?;
    assert_eq!(view.buffered(), 2);
    view.reconnect(1, 7)?;
    assert_eq!(view.cursor(), 7, "the engine decides where we are");
    assert_eq!(view.buffered(), 0);
    assert_eq!(view.reconnects(), 1);
    Ok(())
}

/// T16-HD-17 · reconnecting in the same epoch keeps presented snapshots: the tasks did not
/// stop existing because the pane did.
#[test]
fn reconnecting_in_epoch_keeps_presented_tasks() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(snapshot(&task, Status::Running, 1))?;
    view.reconnect(1, 5)?;
    assert_eq!(view.tasks(), 1);
    assert_eq!(view.task(&task)?.status, Status::Running);
    Ok(())
}

/// T16-HD-18 · reconnecting into a NEW epoch clears the view entirely: a snapshot from a
/// previous epoch describes a world that no longer exists.
#[test]
fn reconnecting_into_a_new_epoch_clears_the_view() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(snapshot(&task, Status::Passed, 1))?;
    view.submit(
        &IntentKey::new("k"),
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 1)?),
    )?;
    view.reconnect(2, 0)?;
    assert_eq!(view.epoch(), 2);
    assert_eq!(view.tasks(), 0);
    assert_eq!(view.submitted(&IntentKey::new("k")), None);
    assert_eq!(view.task(&task).map(|_| ()), Err(Refusal::UnknownTask));
    Ok(())
}

/// T16-HD-19 · a reconnect that reports a cursor behind the client's is refused: the client
/// would have seen something the engine never emitted.
#[test]
fn a_backwards_reconnect_is_refused() -> Outcome {
    let mut view = View::new(1);
    view.observe(1, 9, "nine")?;
    assert_eq!(view.reconnect(1, 8), Err(Refusal::CursorAhead));
    assert_eq!(view.cursor(), 9, "and the view did not move");
    assert_eq!(view.reconnects(), 0);
    Ok(())
}

/// T16-HD-20 · a reconnect at exactly the client's cursor is admitted; the boundary is
/// asserted from both sides.
#[test]
fn reconnecting_at_the_same_cursor_is_admitted() -> Outcome {
    let mut view = View::new(1);
    view.observe(1, 9, "nine")?;
    view.reconnect(1, 9)?;
    assert_eq!(view.cursor(), 9);
    assert_eq!(view.reconnects(), 1);
    Ok(())
}

/// T16-HD-21 · a backwards cursor in a DIFFERENT epoch is fine: sequences restart, so the
/// comparison only applies within one epoch.
#[test]
fn a_backwards_cursor_in_a_new_epoch_is_admitted() -> Outcome {
    let mut view = View::new(1);
    view.observe(1, 100, "hundred")?;
    view.reconnect(2, 0)?;
    assert_eq!(view.cursor(), 0);
    assert_eq!(view.epoch(), 2);
    Ok(())
}

/// T16-HD-22 · the reconnect bound refuses at its limit.
#[test]
fn the_reconnect_bound_refuses_at_its_limit() -> Outcome {
    let mut view = View::new(1);
    for _ in 0..MAX_RECONNECTS {
        view.reconnect(1, 0)?;
    }
    assert_eq!(view.reconnects(), MAX_RECONNECTS);
    assert_eq!(view.reconnect(1, 0), Err(Refusal::ReconnectLimit));
    Ok(())
}

/// T16-HD-23 · a full loss-and-recover cycle: submit, observe, lose, reconnect, and the
/// durable task is still presented from the engine's own state. This is the contract's
/// integrated proof scenario.
#[test]
fn the_loss_and_recover_cycle_preserves_the_durable_task() -> Outcome {
    let task = id(1);
    let key = IntentKey::new("operator-intent");
    let mut view = View::new(1);
    let admitted = view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 1)?),
    )?;
    assert!(admitted.is_durable());
    view.observe(1, 1, "dispatched")?;
    view.present(snapshot(&task, Status::Running, 1))?;

    // the pane is lost; the client keeps nothing it cannot vouch for
    view.reconnect(1, 4)?;
    assert_eq!(view.buffered(), 0);

    // the engine re-presents, and the view carries the outcome it was told
    view.present(Snapshot {
        route_explanation: Some("chose the local adapter on the cost ceiling".to_owned()),
        evidence: vec!["evidence/release/T16-01.json".to_owned()],
        ..snapshot(&task, Status::Passed, 5)
    })?;
    let presented = view.task(&task)?;
    assert_eq!(presented.status, Status::Passed);
    assert!(presented.status.is_settled());
    assert_eq!(presented.evidence.len(), 1, "proof stays navigable");
    assert_eq!(view.submitted(&key), Some(task.as_str()));
    Ok(())
}

// ------------------------------------------- presentation is truthful

/// T16-HD-24 · an out-of-order snapshot does not move a task's presentation backwards.
#[test]
fn an_out_of_order_snapshot_is_ignored() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    assert!(view.present(snapshot(&task, Status::Passed, 5))?);
    assert!(
        !view.present(snapshot(&task, Status::Running, 3))?,
        "an older snapshot is not applied"
    );
    assert_eq!(view.task(&task)?.status, Status::Passed);
    Ok(())
}

/// T16-HD-25 · a snapshot at the same sequence is also ignored, so a re-delivered snapshot
/// cannot flip a presentation.
#[test]
fn a_same_sequence_snapshot_is_ignored() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(snapshot(&task, Status::Passed, 5))?;
    assert!(!view.present(snapshot(&task, Status::Failed, 5))?);
    assert_eq!(view.task(&task)?.status, Status::Passed);
    Ok(())
}

/// T16-HD-26 · a newer snapshot is applied and reported as applied.
#[test]
fn a_newer_snapshot_is_applied() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(snapshot(&task, Status::Running, 1))?;
    assert!(view.present(snapshot(&task, Status::Failed, 2))?);
    assert_eq!(view.task(&task)?.status, Status::Failed);
    Ok(())
}

/// T16-HD-27 · gaps the engine reported are preserved, not omitted: a client that drops them
/// presents a more complete picture than it has.
#[test]
fn reported_gaps_are_preserved() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(Snapshot {
        gaps: vec![
            "worker cleanup unread".to_owned(),
            "cost unmeasured".to_owned(),
        ],
        ..snapshot(&task, Status::Unknown, 1)
    })?;
    assert_eq!(view.task(&task)?.gaps.len(), 2);
    Ok(())
}

/// T16-HD-28 · a route explanation is presented verbatim when supplied, and its absence is
/// `None` rather than an invented sentence.
#[test]
fn a_route_explanation_is_verbatim_or_absent() -> Outcome {
    let (a, b) = (id(1), id(2));
    let mut view = View::new(1);
    view.present(Snapshot {
        route_explanation: Some("excluded remote: privacy local_only".to_owned()),
        ..snapshot(&a, Status::Running, 1)
    })?;
    view.present(snapshot(&b, Status::Running, 1))?;
    assert_eq!(
        view.task(&a)?.route_explanation.as_deref(),
        Some("excluded remote: privacy local_only")
    );
    assert_eq!(view.task(&b)?.route_explanation, None);
    Ok(())
}

/// T16-HD-29 · cancellation presents as pending rather than as a settled outcome, because an
/// obligation that is recorded is not an obligation that is discharged.
#[test]
fn cancellation_presents_as_pending_not_settled() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(snapshot(&task, Status::CancellationPending, 1))?;
    let presented = view.task(&task)?;
    assert_eq!(presented.status, Status::CancellationPending);
    assert!(!presented.status.is_settled());
    assert_eq!(presented.status.name(), "cancellation-pending");
    Ok(())
}

/// T16-HD-30 · a malformed task in a snapshot is refused, so a view cannot present a task
/// whose identity it could not parse.
#[test]
fn a_malformed_snapshot_task_is_refused() {
    use habitat_engine::contracts::ScalarError;
    let mut view = View::new(1);
    assert_eq!(
        view.present(snapshot("not-a-uuid", Status::Running, 1)),
        Err(Refusal::MalformedIdentity(ScalarError::InvalidUuid))
    );
    assert_eq!(view.tasks(), 0);
}

/// T16-HD-31 · an unknown task is refused by name rather than returning an empty snapshot.
#[test]
fn an_unknown_task_is_refused() {
    let view = View::new(1);
    assert_eq!(view.task(&id(9)).map(|_| ()), Err(Refusal::UnknownTask));
}

/// T16-HD-32 · several tasks are presented independently.
#[test]
fn several_tasks_are_presented_independently() -> Outcome {
    let mut view = View::new(1);
    for index in 1..=4 {
        view.present(snapshot(&id(index), Status::Running, 1))?;
    }
    assert_eq!(view.tasks(), 4);
    view.present(snapshot(&id(2), Status::Failed, 2))?;
    assert_eq!(view.task(&id(2))?.status, Status::Failed);
    assert_eq!(view.task(&id(3))?.status, Status::Running);
    Ok(())
}

/// T16-HD-33 · a fresh view holds nothing and sits at genesis.
#[test]
fn a_fresh_view_holds_nothing() {
    let view = View::new(3);
    assert_eq!(view.epoch(), 3);
    assert_eq!(view.cursor(), 0);
    assert_eq!(view.buffered(), 0);
    assert_eq!(view.tasks(), 0);
    assert_eq!(view.reconnects(), 0);
}

/// T16-HD-34 · every refusal has a distinct name and none is a substring of another.
#[test]
fn refusal_names_are_distinct_and_non_overlapping() {
    use habitat_engine::contracts::ScalarError;
    let all = [
        Refusal::MalformedIdentity(ScalarError::InvalidUuid),
        Refusal::EpochMismatch,
        Refusal::StaleEvent,
        Refusal::BufferFull,
        Refusal::ReconnectLimit,
        Refusal::UnknownTask,
        Refusal::CursorAhead,
    ];
    for (i, a) in all.iter().enumerate() {
        assert!(!a.name().is_empty());
        for (j, b) in all.iter().enumerate() {
            if i != j {
                assert!(
                    !a.name().contains(b.name()),
                    "{} contains {}",
                    a.name(),
                    b.name()
                );
            }
        }
    }
}

/// T16-HD-35 · a refusal carrying a scalar error shows both parts.
#[test]
fn refusal_display_shows_the_carried_error() {
    use habitat_engine::contracts::ScalarError;
    assert_eq!(
        Refusal::MalformedIdentity(ScalarError::InvalidUuid).to_string(),
        "malformed client identity: expected a lowercase hyphenated UUIDv4"
    );
    assert_eq!(
        Refusal::StaleEvent.to_string(),
        "event precedes the client cursor"
    );
}

/// T16-HD-36 · the declared bounds are the values enforced.
#[test]
fn declared_bounds_are_the_enforced_bounds() {
    assert_eq!(MAX_BUFFERED_EVENTS, 1024);
    assert_eq!(MAX_RECONNECTS, 1024);
    assert_eq!(SCHEMA_VERSION, 1);
}

// ------------------------------------------- the client owns no task state

/// T16-HD-37 · the view has no method that transitions a task. Presenting a `Failed`
/// snapshot and then a `Passed` one at a higher sequence changes the presentation, and that
/// is the only way a status ever changes here: the engine said so.
#[test]
fn only_the_engine_changes_a_status() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(snapshot(&task, Status::Failed, 1))?;
    assert_eq!(view.task(&task)?.status, Status::Failed);
    view.present(snapshot(&task, Status::Passed, 2))?;
    assert_eq!(view.task(&task)?.status, Status::Passed);
    Ok(())
}

/// T16-HD-38 · losing the client does not change any task: reconnecting in-epoch leaves
/// every presented status exactly as the engine last reported it.
#[test]
fn client_loss_leaves_engine_state_untouched() -> Outcome {
    let mut view = View::new(1);
    for index in 1..=3 {
        view.present(snapshot(&id(index), Status::Running, 1))?;
    }
    view.reconnect(1, 99)?;
    for index in 1..=3 {
        assert_eq!(view.task(&id(index))?.status, Status::Running);
    }
    Ok(())
}

/// T16-HD-39 · a view that never observed anything still presents whatever the engine gave
/// it, because presentation and the event stream are independent paths.
#[test]
fn presentation_does_not_depend_on_the_event_stream() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(snapshot(&task, Status::Passed, 9))?;
    assert_eq!(view.cursor(), 0, "no events were observed");
    assert_eq!(view.task(&task)?.status, Status::Passed);
    Ok(())
}

/// T16-HD-40 · evidence references are carried verbatim for navigation and are never
/// interpreted; an entry that looks like an instruction is just a string.
#[test]
fn evidence_references_are_carried_verbatim() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    let refs = vec![
        "evidence/release/a.json".to_owned(),
        "; rm -rf / # not a command, just text".to_owned(),
    ];
    view.present(Snapshot {
        evidence: refs.clone(),
        ..snapshot(&task, Status::Passed, 1)
    })?;
    assert_eq!(view.task(&task)?.evidence, refs);
    Ok(())
}

/// T16-HD-41 · a snapshot with no evidence presents an empty list rather than a placeholder,
/// so "no proof retained" is visibly different from "proof not looked for".
#[test]
fn absent_evidence_is_an_empty_list() -> Outcome {
    let task = id(1);
    let mut view = View::new(1);
    view.present(snapshot(&task, Status::Passed, 1))?;
    assert!(view.task(&task)?.evidence.is_empty());
    assert!(view.task(&task)?.gaps.is_empty());
    Ok(())
}

/// T16-HD-42 · the submission record survives an in-epoch reconnect, so a reconnecting
/// client does not resubmit an intent the engine already admitted.
#[test]
fn the_submission_record_survives_an_in_epoch_reconnect() -> Outcome {
    let task = id(1);
    let key = IntentKey::new("k");
    let mut view = View::new(1);
    view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 1)?),
    )?;
    view.reconnect(1, 3)?;
    assert_eq!(view.submitted(&key), Some(task.as_str()));
    let again = view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 4)?),
    )?;
    assert!(matches!(again, Admitted::Duplicate(_)));
    Ok(())
}

/// T16-HD-43 · after an epoch change the submission record is gone, so the same intent may
/// legitimately be submitted again into the new world.
#[test]
fn an_epoch_change_releases_the_submission_record() -> Outcome {
    let task = id(1);
    let key = IntentKey::new("k");
    let mut view = View::new(1);
    view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 1)?),
    )?;
    view.reconnect(2, 0)?;
    let fresh = view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 2, 1)?),
    )?;
    assert!(matches!(fresh, Admitted::Accepted(_)));
    Ok(())
}

/// T16-HD-44 · observing, draining and observing again keeps every event exactly once, in
/// order, across the drain boundary.
#[test]
fn events_survive_a_drain_boundary_exactly_once() -> Outcome {
    let mut view = View::new(1);
    let mut seen = Vec::new();
    for index in 1..=6_u64 {
        view.observe(1, index, &format!("e{index}"))?;
        if index % 3 == 0 {
            seen.extend(view.drain());
        }
    }
    let sequences: Vec<u64> = seen.iter().map(|(s, _)| *s).collect();
    assert_eq!(sequences, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(view.buffered(), 0);
    Ok(())
}

/// T16-HD-45 · a gap in the engine's sequence is accepted, because the client is not the
/// authority on what the engine emitted; the cursor jumps to what it was told.
#[test]
fn a_sequence_gap_is_accepted_and_moves_the_cursor() -> Outcome {
    let mut view = View::new(1);
    view.observe(1, 1, "a")?;
    view.observe(1, 50, "later")?;
    assert_eq!(view.cursor(), 50);
    assert_eq!(view.buffered(), 2);
    Ok(())
}

/// T16-HD-46 · reconnecting twice counts twice, so an operator can see a flapping client.
#[test]
fn reconnects_are_counted() -> Outcome {
    let mut view = View::new(1);
    for expected in 1..=5 {
        view.reconnect(1, 0)?;
        assert_eq!(view.reconnects(), expected);
    }
    Ok(())
}

/// T16-HD-47 · an admitted submission reports the durable identity the engine chose, not one
/// the client invented.
#[test]
fn the_durable_identity_comes_from_the_engine() -> Outcome {
    let engine_task = id(42);
    let mut view = View::new(1);
    let admitted = view.submit(
        &IntentKey::new("client-guessed-something-else"),
        Admitted::from_engine(EngineReceipt::issue(&engine_task, 1, 1)?),
    )?;
    let acceptance = admitted.acceptance().ok_or("expected acceptance")?;
    assert_eq!(acceptance.task().as_str(), engine_task);
    Ok(())
}

/// T16-HD-48 · a duplicate carries the acceptance from the current call, so the caller
/// always has a receipt to navigate from even on the duplicate path.
#[test]
fn a_duplicate_still_carries_an_acceptance() -> Outcome {
    let task = id(1);
    let key = IntentKey::new("k");
    let mut view = View::new(1);
    view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 1)?),
    )?;
    let duplicate = view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 2)?),
    )?;
    let acceptance = duplicate.acceptance().ok_or("expected acceptance")?;
    assert_eq!(acceptance.task().as_str(), task);
    Ok(())
}

/// T16-HD-49 · a denial submitted under a key that already succeeded returns the denial
/// itself, not a fabricated duplicate: the engine's answer stands.
#[test]
fn a_denial_after_a_success_reports_the_denial() -> Outcome {
    let task = id(1);
    let key = IntentKey::new("k");
    let mut view = View::new(1);
    view.submit(
        &key,
        Admitted::from_engine(EngineReceipt::issue(&task, 1, 1)?),
    )?;
    let denied = view.submit(&key, Admitted::Denied)?;
    assert!(matches!(denied, Admitted::Denied));
    assert!(!denied.is_durable());
    Ok(())
}

/// T16-HD-50 · status names are stable wire strings, asserted individually so a rename is a
/// visible change rather than a silent one.
#[test]
fn status_names_are_stable() {
    assert_eq!(Status::Running.name(), "running");
    assert_eq!(Status::Passed.name(), "passed");
    assert_eq!(Status::Failed.name(), "failed");
    assert_eq!(Status::CancellationPending.name(), "cancellation-pending");
    assert_eq!(Status::Unknown.name(), "unknown");
}

/// T16-HD-51 · a view attached to a high epoch behaves identically to one at epoch 1, so no
/// behaviour is pinned to the identity element.
#[test]
fn behaviour_does_not_depend_on_the_epoch_value() -> Outcome {
    let task = id(1);
    let mut view = View::new(9_999);
    view.submit(
        &IntentKey::new("k"),
        Admitted::from_engine(EngineReceipt::issue(&task, 9_999, 1)?),
    )?;
    view.observe(9_999, 1, "a")?;
    view.present(snapshot(&task, Status::Running, 1))?;
    assert_eq!(view.epoch(), 9_999);
    assert_eq!(view.cursor(), 1);
    assert_eq!(view.tasks(), 1);
    assert_eq!(
        view.observe(1, 2, "wrong epoch"),
        Err(Refusal::EpochMismatch)
    );
    Ok(())
}

/// T16-HD-52 · the whole client cycle over two epochs: work in one, restore, and the view
/// carries nothing across that it cannot vouch for while the engine's new state presents
/// normally.
#[test]
fn the_cycle_across_an_epoch_boundary_carries_nothing_stale() -> Outcome {
    let old_task = id(1);
    let new_task = id(2);
    let mut view = View::new(1);
    view.submit(
        &IntentKey::new("first"),
        Admitted::from_engine(EngineReceipt::issue(&old_task, 1, 1)?),
    )?;
    view.observe(1, 1, "old")?;
    view.present(snapshot(&old_task, Status::Passed, 1))?;

    view.reconnect(2, 0)?;
    assert_eq!(view.tasks(), 0);
    assert_eq!(view.buffered(), 0);
    assert_eq!(view.cursor(), 0);

    view.submit(
        &IntentKey::new("second"),
        Admitted::from_engine(EngineReceipt::issue(&new_task, 2, 1)?),
    )?;
    view.present(snapshot(&new_task, Status::Running, 1))?;
    assert_eq!(view.tasks(), 1);
    assert_eq!(view.task(&new_task)?.status, Status::Running);
    assert_eq!(view.task(&old_task).map(|_| ()), Err(Refusal::UnknownTask));
    Ok(())
}
