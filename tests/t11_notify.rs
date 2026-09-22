//! T11 notify cases (`T11-NT-nn`). Every case builds its inputs as values and asserts the
//! whole observable result. There is no clock, socket, file or task-state write here: the
//! module under test owns delivery bookkeeping and nothing else.
//!
//! The contract's required proof is *"commit-before-delivery; lost ack, replay dedup, cursor
//! expiry, restored epoch, slow subscriber and visibility filtering"*, and each family below
//! names which it covers.

use std::error::Error;

use habitat_engine::notify::{
    Commit, Committed, Cursor, Delivery, FailureCategory, MAX_BACKLOG, MAX_RECIPIENTS, MAX_REPLAY,
    Outbox, Quiet, Refusal, SCHEMA_VERSION, Visibility, Wake, WakeMark,
};

type Outcome = Result<(), Box<dyn Error>>;

fn id(index: usize) -> String {
    format!("{index:08x}-0000-4000-8000-000000000000")
}

/// An outbox at epoch 1 holding `count` public events for no recipients.
fn filled(count: usize) -> Result<Outbox, Box<dyn Error>> {
    let mut outbox = Outbox::new(1);
    for index in 1..=count {
        let identity = id(index);
        let event = Committed::new(
            &identity,
            Visibility::Public,
            Commit::witness(1, u64::try_from(index)?),
        )?;
        outbox.enqueue(event, &[])?;
    }
    Ok(outbox)
}

// ------------------------------------------------------- commit before delivery

/// T11-NT-01 · an event can only be enqueued once it carries a commit witness. There is no
/// constructor from an uncommitted event, so pre-commit delivery is unrepresentable rather
/// than refused; this case pins that the committed path works end to end.
#[test]
fn a_committed_event_is_enqueued_with_its_sequence() -> Outcome {
    let mut outbox = Outbox::new(1);
    let identity = id(1);
    let event = Committed::new(&identity, Visibility::Public, Commit::witness(1, 7))?;
    assert_eq!(
        outbox.enqueue(event, &[])?,
        1,
        "outbox sequence, not ledger"
    );
    assert_eq!(outbox.retained(), 1);
    assert_eq!(outbox.next_sequence(), 2);
    Ok(())
}

/// T11-NT-02 · the commit witness carries the ledger's own epoch and sequence, and reports
/// them back unchanged.
#[test]
fn a_commit_witness_reports_its_epoch_and_sequence() {
    let commit = Commit::witness(4, 99);
    assert_eq!(commit.epoch(), 4);
    assert_eq!(commit.sequence(), 99);
}

/// T11-NT-03 · a commit from another epoch cannot be enqueued; a restored epoch must resync
/// rather than interleave with the previous one.
#[test]
fn a_commit_from_another_epoch_is_refused() -> Outcome {
    let mut outbox = Outbox::new(1);
    let identity = id(1);
    let event = Committed::new(&identity, Visibility::Public, Commit::witness(2, 1))?;
    assert_eq!(outbox.enqueue(event, &[]), Err(Refusal::EpochMismatch));
    assert_eq!(outbox.retained(), 0);
    Ok(())
}

/// T11-NT-04 · a malformed event identity is refused at construction, before an outbox is
/// involved at all.
#[test]
fn a_malformed_event_identity_is_refused_at_construction() {
    use habitat_engine::contracts::ScalarError;
    assert_eq!(
        Committed::new("nope", Visibility::Public, Commit::witness(1, 1)).map(|_| ()),
        Err(Refusal::MalformedIdentity(ScalarError::InvalidUuid))
    );
}

/// T11-NT-05 · a committed event reports its identity, visibility and commit unchanged.
#[test]
fn a_committed_event_reports_its_parts() -> Outcome {
    let identity = id(3);
    let event = Committed::new(&identity, Visibility::Operator, Commit::witness(1, 5))?;
    assert_eq!(event.identity().as_str(), identity);
    assert_eq!(event.visibility(), Visibility::Operator);
    assert_eq!(event.commit().sequence(), 5);
    Ok(())
}

/// T11-NT-06 · a duplicate event identity is refused rather than enqueued twice.
#[test]
fn a_duplicate_event_identity_is_refused() -> Outcome {
    let mut outbox = Outbox::new(1);
    let identity = id(1);
    outbox.enqueue(
        Committed::new(&identity, Visibility::Public, Commit::witness(1, 1))?,
        &[],
    )?;
    assert_eq!(
        outbox.enqueue(
            Committed::new(&identity, Visibility::Public, Commit::witness(1, 2))?,
            &[]
        ),
        Err(Refusal::DuplicateEvent)
    );
    assert_eq!(outbox.retained(), 1);
    Ok(())
}

/// T11-NT-07 · sequences are assigned in enqueue order and are contiguous.
#[test]
fn sequences_are_contiguous_in_enqueue_order() -> Outcome {
    let outbox = filled(5)?;
    let stream = outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY)?;
    let sequences: Vec<u64> = stream.events.iter().map(|e| e.sequence).collect();
    assert_eq!(sequences, vec![1, 2, 3, 4, 5]);
    Ok(())
}

/// T11-NT-08 · the persisted schema version is pinned.
#[test]
fn schema_version_is_pinned() {
    assert_eq!(SCHEMA_VERSION, 1);
}

// ------------------------------------------------------------ visibility filtering

/// T11-NT-09 · the visibility order is a total order on breadth, asserted for all nine
/// caller/event pairs rather than for a convenient few.
#[test]
fn visibility_admission_is_a_total_order_on_breadth() {
    let expected = [
        (Visibility::Owner, Visibility::Owner, true),
        (Visibility::Owner, Visibility::Operator, true),
        (Visibility::Owner, Visibility::Public, true),
        (Visibility::Operator, Visibility::Owner, false),
        (Visibility::Operator, Visibility::Operator, true),
        (Visibility::Operator, Visibility::Public, true),
        (Visibility::Public, Visibility::Owner, false),
        (Visibility::Public, Visibility::Operator, false),
        (Visibility::Public, Visibility::Public, true),
    ];
    for (caller, event, admits) in expected {
        assert_eq!(
            caller.admits(event),
            admits,
            "{} sees {}",
            caller.name(),
            event.name()
        );
    }
}

/// T11-NT-10 · a subscriber sees only the events its visibility admits, and the events it
/// cannot see do not appear as blanks or shift the others' sequences.
#[test]
fn a_subscriber_sees_only_what_its_visibility_admits() -> Outcome {
    let mut outbox = Outbox::new(1);
    for (index, visibility) in Visibility::ALL.into_iter().enumerate() {
        let identity = id(index + 1);
        outbox.enqueue(
            Committed::new(
                &identity,
                visibility,
                Commit::witness(1, u64::try_from(index + 1)?),
            )?,
            &[],
        )?;
    }
    let public = outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY)?;
    assert_eq!(public.events.len(), 1);
    assert_eq!(public.events[0].visibility, Visibility::Public);
    assert_eq!(public.events[0].sequence, 3, "sequences are not renumbered");
    let operator = outbox.subscribe(Visibility::Operator, Cursor::genesis(1), MAX_REPLAY)?;
    assert_eq!(operator.events.len(), 2);
    let owner = outbox.subscribe(Visibility::Owner, Cursor::genesis(1), MAX_REPLAY)?;
    assert_eq!(owner.events.len(), 3);
    Ok(())
}

/// T11-NT-11 · a filtered subscriber's cursor still advances past the events it could not
/// see, so it does not re-scan them forever.
#[test]
fn a_filtered_cursor_advances_past_invisible_events() -> Outcome {
    let mut outbox = Outbox::new(1);
    for index in 1..=3 {
        let identity = id(index);
        outbox.enqueue(
            Committed::new(
                &identity,
                Visibility::Owner,
                Commit::witness(1, u64::try_from(index)?),
            )?,
            &[],
        )?;
    }
    let stream = outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY)?;
    assert!(stream.events.is_empty(), "nothing is visible");
    assert_eq!(stream.cursor.sequence(), 3, "but the cursor moved");
    Ok(())
}

/// T11-NT-12 · every visibility round-trips its wire name and the enumeration is the world.
#[test]
fn visibility_names_round_trip() -> Outcome {
    assert_eq!(Visibility::ALL.len(), 3);
    for visibility in Visibility::ALL {
        assert_eq!(Visibility::parse(visibility.name())?, visibility);
        assert_eq!(visibility.to_string(), visibility.name());
    }
    assert_eq!(Visibility::parse("root"), Err(Refusal::UnknownVisibility));
    Ok(())
}

// ------------------------------------------------- cursors, gaps and slow subscribers

/// T11-NT-13 · a cursor carries its epoch, so an old sequence cannot silently name a new
/// event after a restore.
#[test]
fn a_cursor_carries_its_epoch() {
    let cursor = Cursor::after(3, 17);
    assert_eq!(cursor.epoch(), 3);
    assert_eq!(cursor.sequence(), 17);
    assert_eq!(Cursor::genesis(3).sequence(), 0);
}

/// T11-NT-14 · a cursor from another epoch is refused; the subscriber must resync.
#[test]
fn a_cursor_from_another_epoch_is_refused() -> Outcome {
    let outbox = filled(3)?;
    assert_eq!(
        outbox.subscribe(Visibility::Public, Cursor::genesis(2), MAX_REPLAY),
        Err(Refusal::EpochMismatch)
    );
    Ok(())
}

/// T11-NT-15 · the caught-up boundary, asserted from both sides. A cursor **at** the last
/// committed sequence is caught up and receives an empty stream; only a cursor naming a
/// sequence that was never committed is `CursorAhead`.
///
/// The first version of this case asserted that a caught-up cursor was an error. It is not,
/// and it must not be: the steady-state poll reaches exactly this state every time it drains
/// the outbox, so erroring there would make the normal path the failing one. The code was
/// right and the case was wrong.
#[test]
fn the_caught_up_boundary_is_empty_and_only_beyond_it_is_ahead() -> Outcome {
    let outbox = filled(3)?;
    let caught_up = outbox.subscribe(Visibility::Public, Cursor::after(1, 3), MAX_REPLAY)?;
    assert!(caught_up.events.is_empty(), "caught up, not an error");
    assert_eq!(caught_up.cursor.sequence(), 3);
    assert_eq!(
        outbox.subscribe(Visibility::Public, Cursor::after(1, 4), MAX_REPLAY),
        Err(Refusal::CursorAhead),
        "sequence 4 was never committed"
    );
    let behind = outbox.subscribe(Visibility::Public, Cursor::after(1, 2), MAX_REPLAY)?;
    assert_eq!(behind.events.len(), 1);
    Ok(())
}

/// T11-NT-16 · resuming from a returned cursor yields the next events and nothing already
/// seen, across three successive reads.
#[test]
fn resuming_from_a_returned_cursor_never_repeats() -> Outcome {
    let outbox = filled(9)?;
    let mut cursor = Cursor::genesis(1);
    let mut seen = Vec::new();
    for _ in 0..3 {
        let stream = outbox.subscribe(Visibility::Public, cursor, 3)?;
        seen.extend(stream.events.iter().map(|e| e.sequence));
        cursor = stream.cursor;
    }
    assert_eq!(seen, (1..=9).collect::<Vec<u64>>());
    Ok(())
}

/// T11-NT-17 · a slow subscriber gets backpressure by limit, not truncation: the stream is
/// capped and the cursor says exactly where to resume.
#[test]
fn a_slow_subscriber_is_paced_by_the_limit() -> Outcome {
    let outbox = filled(20)?;
    let stream = outbox.subscribe(Visibility::Public, Cursor::genesis(1), 4)?;
    assert_eq!(stream.events.len(), 4);
    assert_eq!(stream.cursor.sequence(), 4);
    assert!(stream.gap.is_none(), "paced is not the same as gapped");
    Ok(())
}

/// T11-NT-18 · a replay wider than the bound is refused at the point of acquisition, before
/// any event is copied.
#[test]
fn a_replay_wider_than_the_bound_is_refused() -> Outcome {
    let outbox = filled(3)?;
    assert_eq!(
        outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY + 1),
        Err(Refusal::ReplayTooWide)
    );
    let ok = outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY)?;
    assert_eq!(ok.events.len(), 3);
    Ok(())
}

/// T11-NT-19 · a limit of zero returns nothing and does not advance the cursor, so a caller
/// polling with zero cannot silently skip events.
#[test]
fn a_zero_limit_returns_nothing_and_holds_the_cursor() -> Outcome {
    let outbox = filled(3)?;
    let stream = outbox.subscribe(Visibility::Public, Cursor::genesis(1), 0)?;
    assert!(stream.events.is_empty());
    assert_eq!(stream.cursor.sequence(), 0);
    Ok(())
}

/// T11-NT-20 · a cursor that fell behind the retained backlog returns a gap naming both
/// numbers, alongside whatever is still retained. This is cursor expiry, reported rather
/// than papered over.
#[test]
fn a_cursor_behind_the_backlog_returns_a_gap_with_both_numbers() -> Outcome {
    let mut outbox = filled(6)?;
    for index in 1..=3 {
        outbox
            .record(&id(index), &id(900), Delivery::Delivered)
            .ok();
    }
    assert_eq!(outbox.compact(3), 3);
    assert_eq!(outbox.retained_from(), 4);
    let stream = outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY)?;
    let gap = stream.gap.ok_or("expected a gap")?;
    assert_eq!(gap.retained_from, 4);
    assert_eq!(gap.requested_after, 0);
    assert_eq!(stream.events.len(), 3, "what survived is still served");
    Ok(())
}

/// T11-NT-21 · a cursor exactly at the retained boundary is not a gap; the boundary is
/// asserted from both sides.
#[test]
fn the_retained_boundary_is_not_a_gap() -> Outcome {
    let mut outbox = filled(6)?;
    assert_eq!(outbox.compact(3), 3);
    let at = outbox.subscribe(Visibility::Public, Cursor::after(1, 3), MAX_REPLAY)?;
    assert!(at.gap.is_none(), "a cursor at the boundary missed nothing");
    let behind = outbox.subscribe(Visibility::Public, Cursor::after(1, 2), MAX_REPLAY)?;
    assert!(behind.gap.is_some(), "one before the boundary did miss");
    Ok(())
}

// ------------------------------------------------- delivery, lost ack and replay dedup

/// T11-NT-22 · a fresh obligation is `Unknown`, which is distinct from pending: "we have not
/// tried" and "we tried and cannot tell" are different facts.
#[test]
fn a_fresh_obligation_is_unknown_not_pending() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&recipient],
    )?;
    assert_eq!(outbox.obligation(&event, &recipient)?, Delivery::Unknown);
    assert_ne!(
        Delivery::Unknown,
        Delivery::Pending(FailureCategory::Indeterminate)
    );
    Ok(())
}

/// T11-NT-23 · recording a delivery settles the obligation, and re-recording it is
/// idempotent — this is the lost-acknowledgement retry.
#[test]
fn a_repeated_acknowledgement_is_idempotent() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&recipient],
    )?;
    assert!(outbox.record(&event, &recipient, Delivery::Delivered)?);
    assert!(
        !outbox.record(&event, &recipient, Delivery::Delivered)?,
        "the retry changes nothing"
    );
    assert_eq!(outbox.obligation(&event, &recipient)?, Delivery::Delivered);
    Ok(())
}

/// T11-NT-24 · a delivered obligation is terminal: a later failure does not reopen it, or a
/// flapping transport would invite a redelivery the contract forbids.
#[test]
fn a_delivered_obligation_is_not_reopened_by_a_later_failure() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&recipient],
    )?;
    outbox.record(&event, &recipient, Delivery::Delivered)?;
    assert!(!outbox.record(
        &event,
        &recipient,
        Delivery::Pending(FailureCategory::Unreachable)
    )?);
    assert_eq!(outbox.obligation(&event, &recipient)?, Delivery::Delivered);
    Ok(())
}

/// T11-NT-25 · dedup is keyed on the `(event, recipient)` pair, so the same event to two
/// recipients carries two independent obligations.
#[test]
fn dedup_is_keyed_on_the_event_and_recipient_pair() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let (a, b) = (id(900), id(901));
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&a, &b],
    )?;
    outbox.record(&event, &a, Delivery::Delivered)?;
    assert_eq!(outbox.obligation(&event, &a)?, Delivery::Delivered);
    assert_eq!(
        outbox.obligation(&event, &b)?,
        Delivery::Unknown,
        "the other recipient is untouched"
    );
    Ok(())
}

/// T11-NT-26 · every failure category round-trips its name, and exactly one is not
/// retryable.
#[test]
fn failure_categories_name_themselves_and_one_is_terminal() {
    assert_eq!(FailureCategory::ALL.len(), 3);
    let terminal: Vec<&str> = FailureCategory::ALL
        .into_iter()
        .filter(|c| !c.retryable())
        .map(FailureCategory::name)
        .collect();
    assert_eq!(terminal, vec!["rejected"]);
}

/// T11-NT-27 · an indeterminate failure stays retryable **and** unsettled: the engine cannot
/// prove the event did not arrive, so the obligation is not discharged.
#[test]
fn an_indeterminate_failure_stays_an_open_obligation() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&recipient],
    )?;
    outbox.record(
        &event,
        &recipient,
        Delivery::Pending(FailureCategory::Indeterminate),
    )?;
    let state = outbox.obligation(&event, &recipient)?;
    assert!(!state.is_settled());
    assert_eq!(state.name(), "pending");
    assert_eq!(outbox.outstanding(MAX_REPLAY).len(), 1);
    Ok(())
}

/// T11-NT-28 · recording the same non-terminal outcome twice is idempotent too, so a retry
/// loop does not churn the record.
#[test]
fn recording_the_same_failure_twice_changes_nothing() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&recipient],
    )?;
    let pending = Delivery::Pending(FailureCategory::Unreachable);
    assert!(outbox.record(&event, &recipient, pending)?);
    assert!(!outbox.record(&event, &recipient, pending)?);
    Ok(())
}

/// T11-NT-29 · an unknown event or an unknown recipient is refused by name.
#[test]
fn an_unknown_event_or_recipient_is_refused() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&recipient],
    )?;
    assert_eq!(
        outbox.record(&id(2), &recipient, Delivery::Delivered),
        Err(Refusal::UnknownEvent)
    );
    assert_eq!(
        outbox.record(&event, &id(999), Delivery::Delivered),
        Err(Refusal::UnknownEvent)
    );
    assert_eq!(
        outbox.obligation(&id(2), &recipient),
        Err(Refusal::UnknownEvent)
    );
    Ok(())
}

/// T11-NT-30 · `outstanding` lists exactly the undischarged obligations and is bounded.
#[test]
fn outstanding_lists_only_undischarged_obligations() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let (a, b, c) = (id(900), id(901), id(902));
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&a, &b, &c],
    )?;
    outbox.record(&event, &a, Delivery::Delivered)?;
    let out = outbox.outstanding(MAX_REPLAY);
    assert_eq!(out.len(), 2);
    assert!(out.iter().all(|(_, recipient, _)| *recipient != a));
    assert_eq!(outbox.outstanding(1).len(), 1, "the caller's bound holds");
    assert_eq!(
        outbox.outstanding(usize::MAX).len(),
        2,
        "and the module's own bound caps an unbounded ask"
    );
    Ok(())
}

// ------------------------------------------------- compaction, restore and bounds

/// T11-NT-31 · compaction discards settled events up to the requested sequence.
#[test]
fn compaction_discards_settled_events() -> Outcome {
    let outbox_len = 6;
    let mut outbox = filled(outbox_len)?;
    assert_eq!(outbox.compact(4), 4);
    assert_eq!(outbox.retained(), 2);
    assert_eq!(outbox.retained_from(), 5);
    Ok(())
}

/// T11-NT-32 · an event with an outstanding obligation is KEPT through compaction, whatever
/// the caller asked for — dropping it would discharge a delivery by forgetting it.
#[test]
fn compaction_keeps_events_with_outstanding_obligations() -> Outcome {
    let mut outbox = Outbox::new(1);
    let recipient = id(900);
    for index in 1..=3 {
        let event = id(index);
        outbox.enqueue(
            Committed::new(
                &event,
                Visibility::Public,
                Commit::witness(1, u64::try_from(index)?),
            )?,
            &[&recipient],
        )?;
    }
    outbox.record(&id(1), &recipient, Delivery::Delivered)?;
    outbox.record(&id(3), &recipient, Delivery::Delivered)?;
    assert_eq!(outbox.compact(3), 2, "only the settled two go");
    assert_eq!(outbox.retained(), 1);
    assert_eq!(outbox.obligation(&id(2), &recipient)?, Delivery::Unknown);
    Ok(())
}

/// T11-NT-33 · compacting an empty outbox is a no-op and leaves the retained boundary at the
/// next sequence, so a later subscribe does not report a spurious gap.
#[test]
fn compacting_everything_leaves_a_coherent_boundary() -> Outcome {
    let mut outbox = filled(3)?;
    assert_eq!(outbox.compact(3), 3);
    assert_eq!(outbox.retained(), 0);
    assert_eq!(outbox.retained_from(), outbox.next_sequence());
    assert_eq!(outbox.compact(3), 0, "a second compaction discards nothing");
    Ok(())
}

/// T11-NT-34 · restore begins a new epoch, discards the backlog and restarts the sequence;
/// a cursor from the old epoch is then refused.
#[test]
fn restore_starts_a_new_epoch_and_forces_resync() -> Outcome {
    let mut outbox = filled(5)?;
    let stale = Cursor::after(1, 2);
    outbox.restore(2);
    assert_eq!(outbox.epoch(), 2);
    assert_eq!(outbox.retained(), 0);
    assert_eq!(outbox.next_sequence(), 1);
    assert_eq!(
        outbox.subscribe(Visibility::Public, stale, MAX_REPLAY),
        Err(Refusal::EpochMismatch),
        "the old cursor must resync"
    );
    Ok(())
}

/// T11-NT-35 · after a restore the same event identity may be enqueued again — the old
/// epoch's record is gone, and identity uniqueness is per epoch.
#[test]
fn an_identity_may_be_reused_in_a_new_epoch() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[],
    )?;
    outbox.restore(2);
    assert_eq!(
        outbox.enqueue(
            Committed::new(&event, Visibility::Public, Commit::witness(2, 1))?,
            &[]
        )?,
        1
    );
    Ok(())
}

/// T11-NT-36 · a cursor in the new epoch reads the new events from genesis.
#[test]
fn a_new_epoch_reads_from_genesis() -> Outcome {
    let mut outbox = filled(3)?;
    outbox.restore(7);
    let event = id(50);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(7, 1))?,
        &[],
    )?;
    let stream = outbox.subscribe(Visibility::Public, Cursor::genesis(7), MAX_REPLAY)?;
    assert_eq!(stream.events.len(), 1);
    assert_eq!(stream.events[0].sequence, 1);
    assert!(stream.gap.is_none());
    Ok(())
}

/// T11-NT-37 · the recipient bound refuses before the record is built.
#[test]
fn the_recipient_bound_refuses_before_building() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let owned: Vec<String> = (0..=MAX_RECIPIENTS).map(|i| id(1000 + i)).collect();
    let refs: Vec<&str> = owned.iter().map(String::as_str).collect();
    assert_eq!(
        outbox.enqueue(
            Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
            &refs
        ),
        Err(Refusal::RecipientLimit)
    );
    assert_eq!(outbox.retained(), 0);
    assert_eq!(
        outbox.enqueue(
            Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
            &refs[..MAX_RECIPIENTS]
        )?,
        1,
        "exactly the bound is admitted"
    );
    Ok(())
}

/// T11-NT-38 · a malformed recipient identity is refused.
#[test]
fn a_malformed_recipient_is_refused() -> Outcome {
    use habitat_engine::contracts::ScalarError;
    let mut outbox = Outbox::new(1);
    let event = id(1);
    assert_eq!(
        outbox.enqueue(
            Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
            &["not-a-uuid"]
        ),
        Err(Refusal::MalformedIdentity(ScalarError::InvalidUuid))
    );
    Ok(())
}

/// T11-NT-39 · the backlog bound refuses at its limit, and the outbox keeps what it had.
#[test]
fn the_backlog_bound_refuses_at_its_limit() -> Outcome {
    let mut outbox = filled(MAX_BACKLOG)?;
    assert_eq!(outbox.retained(), MAX_BACKLOG);
    let extra = id(MAX_BACKLOG + 1);
    assert_eq!(
        outbox.enqueue(
            Committed::new(
                &extra,
                Visibility::Public,
                Commit::witness(1, u64::try_from(MAX_BACKLOG + 1)?)
            )?,
            &[]
        ),
        Err(Refusal::BacklogFull)
    );
    assert_eq!(outbox.retained(), MAX_BACKLOG);
    Ok(())
}

/// T11-NT-40 · duplicate recipients in one enqueue collapse to one obligation, so a caller
/// that repeats a recipient does not create two deliveries for one logical party.
#[test]
fn duplicate_recipients_collapse_to_one_obligation() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&recipient, &recipient, &recipient],
    )?;
    assert_eq!(outbox.outstanding(MAX_REPLAY).len(), 1);
    Ok(())
}

// ---------------------------------------------------------------- diagnostics

/// T11-NT-41 · every refusal has a distinct name and none is a substring of another.
#[test]
fn refusal_names_are_distinct_and_non_overlapping() {
    use habitat_engine::contracts::ScalarError;
    let all = [
        Refusal::MalformedIdentity(ScalarError::InvalidUuid),
        Refusal::UnknownVisibility,
        Refusal::BacklogFull,
        Refusal::RecipientLimit,
        Refusal::ReplayTooWide,
        Refusal::CursorExpired,
        Refusal::CursorAhead,
        Refusal::EpochMismatch,
        Refusal::UnknownEvent,
        Refusal::DuplicateEvent,
        Refusal::SequenceOverflow,
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

/// T11-NT-42 · a refusal carrying a scalar error shows both parts.
#[test]
fn refusal_display_shows_the_carried_error() {
    use habitat_engine::contracts::ScalarError;
    assert_eq!(
        Refusal::MalformedIdentity(ScalarError::InvalidUuid).to_string(),
        "malformed notify identity: expected a lowercase hyphenated UUIDv4"
    );
    assert_eq!(
        Refusal::EpochMismatch.to_string(),
        "epoch differs; resync required"
    );
}

/// T11-NT-43 · delivery states name themselves and only `Delivered` is settled.
#[test]
fn delivery_states_name_themselves() {
    assert_eq!(Delivery::Delivered.name(), "delivered");
    assert_eq!(Delivery::Unknown.name(), "unknown");
    assert_eq!(
        Delivery::Pending(FailureCategory::Rejected).name(),
        "pending"
    );
    assert!(Delivery::Delivered.is_settled());
    assert!(!Delivery::Unknown.is_settled());
    assert!(!Delivery::Pending(FailureCategory::Rejected).is_settled());
}

/// T11-NT-44 · an outbox reports its own bounds, so a caller can size its loop without
/// hardcoding a number that could drift from the module's.
#[test]
fn an_outbox_reports_its_position() -> Outcome {
    let outbox = filled(4)?;
    assert_eq!(outbox.epoch(), 1);
    assert_eq!(outbox.retained(), 4);
    assert_eq!(outbox.retained_from(), 1);
    assert_eq!(outbox.next_sequence(), 5);
    Ok(())
}

/// T11-NT-45 · an empty outbox serves an empty stream from genesis without a gap, and a
/// cursor at genesis is not "ahead" of an empty outbox only because nothing is committed.
#[test]
fn an_empty_outbox_serves_an_empty_stream() -> Outcome {
    let outbox = Outbox::new(1);
    let stream = outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY)?;
    assert!(stream.events.is_empty());
    assert!(stream.gap.is_none());
    assert_eq!(stream.cursor.sequence(), 0);
    Ok(())
}

/// T11-NT-46 · the module's declared bounds are the values the code uses, asserted here so a
/// change to one without the other is visible.
#[test]
fn declared_bounds_are_the_enforced_bounds() {
    assert_eq!(MAX_BACKLOG, 8192);
    assert_eq!(MAX_REPLAY, 512);
    assert_eq!(MAX_RECIPIENTS, 256);
}

/// T11-NT-47 · a subscriber that reads to the end, then reads again after a new event, sees
/// exactly the new one. This is the steady-state loop the daemon will run.
#[test]
fn the_steady_state_loop_sees_exactly_the_new_event() -> Outcome {
    let mut outbox = filled(3)?;
    // `Stream` borrows the outbox (its events hold borrowed identities), so the cursor is
    // copied out before the next enqueue. The borrow checker refusing the other order is the
    // API working: a stream read before a write must not be read as though it were after.
    let resume = {
        let caught_up = outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY)?;
        assert_eq!(caught_up.events.len(), 3);
        caught_up.cursor
    };
    let fresh = id(4);
    outbox.enqueue(
        Committed::new(&fresh, Visibility::Public, Commit::witness(1, 4))?,
        &[],
    )?;
    let next = outbox.subscribe(Visibility::Public, resume, MAX_REPLAY)?;
    assert_eq!(next.events.len(), 1);
    assert_eq!(next.events[0].identity.as_str(), fresh);
    Ok(())
}

/// T11-NT-48 · an event with no recipients carries no obligation and is compactable at once;
/// a broadcast with nobody listening must not pin the backlog forever.
#[test]
fn an_event_with_no_recipients_is_immediately_compactable() -> Outcome {
    let mut outbox = filled(2)?;
    assert!(outbox.outstanding(MAX_REPLAY).is_empty());
    assert_eq!(outbox.compact(2), 2);
    Ok(())
}

/// T11-NT-49 · obligations survive compaction attempts across several events and recipients,
/// and the retained boundary tracks the oldest surviving event rather than the request.
#[test]
fn the_retained_boundary_tracks_the_oldest_survivor() -> Outcome {
    let mut outbox = Outbox::new(1);
    let recipient = id(900);
    for index in 1..=5 {
        let event = id(index);
        outbox.enqueue(
            Committed::new(
                &event,
                Visibility::Public,
                Commit::witness(1, u64::try_from(index)?),
            )?,
            &[&recipient],
        )?;
    }
    for index in [1, 2, 4] {
        outbox.record(&id(index), &recipient, Delivery::Delivered)?;
    }
    assert_eq!(outbox.compact(5), 3);
    assert_eq!(outbox.retained_from(), 3, "event 3 is the oldest survivor");
    assert_eq!(outbox.retained(), 2);
    Ok(())
}

/// T11-NT-50 · a full lifecycle: commit, enqueue, fail, retry, deliver, compact — asserted
/// as a sequence so an intermediate state that breaks the invariant is caught where it
/// happens rather than at the end.
#[test]
fn a_full_delivery_lifecycle_holds_at_every_step() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&recipient],
    )?;
    assert_eq!(outbox.obligation(&event, &recipient)?, Delivery::Unknown);
    outbox.record(
        &event,
        &recipient,
        Delivery::Pending(FailureCategory::Unreachable),
    )?;
    assert_eq!(outbox.outstanding(MAX_REPLAY).len(), 1);
    assert_eq!(outbox.compact(1), 0, "an open obligation pins the event");
    outbox.record(&event, &recipient, Delivery::Delivered)?;
    assert!(outbox.outstanding(MAX_REPLAY).is_empty());
    assert_eq!(outbox.compact(1), 1, "and a settled one releases it");
    assert_eq!(outbox.retained(), 0);
    Ok(())
}

/// T11-NT-51 · a rejected delivery is not retryable but is still an open obligation, so it
/// stays visible for an operator rather than disappearing as "done".
#[test]
fn a_rejected_delivery_stays_visible_though_not_retryable() -> Outcome {
    let mut outbox = Outbox::new(1);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(1, 1))?,
        &[&recipient],
    )?;
    outbox.record(
        &event,
        &recipient,
        Delivery::Pending(FailureCategory::Rejected),
    )?;
    assert!(!FailureCategory::Rejected.retryable());
    assert_eq!(
        outbox.outstanding(MAX_REPLAY).len(),
        1,
        "still an obligation"
    );
    Ok(())
}

/// T11-NT-52 · visibility filtering and the gap interact correctly: a subscriber that cannot
/// see anything still learns it missed a window.
#[test]
fn a_filtered_subscriber_still_learns_of_a_gap() -> Outcome {
    let mut outbox = Outbox::new(1);
    for index in 1..=4 {
        let event = id(index);
        outbox.enqueue(
            Committed::new(
                &event,
                Visibility::Owner,
                Commit::witness(1, u64::try_from(index)?),
            )?,
            &[],
        )?;
    }
    assert_eq!(outbox.compact(2), 2);
    let stream = outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY)?;
    assert!(stream.events.is_empty(), "none are visible");
    let gap = stream.gap.ok_or("expected a gap")?;
    assert_eq!(gap.retained_from, 3);
    Ok(())
}

/// T11-NT-53 · nothing in this module is pinned to epoch 1. The whole lifecycle is exercised
/// at a high epoch, so a reader that hard-codes the identity element fails here.
#[test]
fn behaviour_does_not_depend_on_the_epoch_value() -> Outcome {
    let epoch = 7_391_u64;
    let mut outbox = Outbox::new(epoch);
    assert_eq!(outbox.epoch(), epoch);
    let event = id(1);
    let recipient = id(900);
    outbox.enqueue(
        Committed::new(&event, Visibility::Public, Commit::witness(epoch, 1))?,
        &[&recipient],
    )?;
    let stream = outbox.subscribe(Visibility::Public, Cursor::genesis(epoch), MAX_REPLAY)?;
    assert_eq!(stream.events.len(), 1);
    assert_eq!(stream.cursor.epoch(), epoch);
    assert_eq!(
        outbox.subscribe(Visibility::Public, Cursor::genesis(1), MAX_REPLAY),
        Err(Refusal::EpochMismatch),
        "epoch 1 is just another epoch here"
    );
    outbox.record(&event, &recipient, Delivery::Delivered)?;
    assert_eq!(outbox.compact(1), 1);
    outbox.restore(epoch + 1);
    assert_eq!(outbox.epoch(), epoch + 1);
    Ok(())
}

/// T11-NT-54 · the epoch an outbox reports is the one it was given, checked across several
/// distinct values so a constant cannot satisfy it.
#[test]
fn the_reported_epoch_is_the_one_given() {
    for epoch in [0_u64, 1, 2, 42, u64::MAX] {
        assert_eq!(Outbox::new(epoch).epoch(), epoch);
    }
}

// ---------------------------------------------------------------------------------------
// T11 clause: "actionable wakes are deduplicated/batched; controlled idle and repeated
// unchanged events cause zero model requests."
//
// A wake is what costs a model request, so every case below asserts the WAKE, not a counter
// somebody increments. The three quiet grounds are pinned separately: a suite that only
// checked `!is_actionable()` would pass against an implementation that returned one ground
// for all three, and the ground is what tells an operator whether the system is idle, caught
// up, or deliberately holding.
// ---------------------------------------------------------------------------------------

/// An outbox at epoch 1 holding `count` public events, all addressed to `recipient`.
fn addressed(count: usize, recipient: &str) -> Result<Outbox, Box<dyn Error>> {
    let mut outbox = Outbox::new(1);
    for index in 1..=count {
        let identity = id(index);
        let event = Committed::new(
            &identity,
            Visibility::Public,
            Commit::witness(1, u64::try_from(index)?),
        )?;
        outbox.enqueue(event, &[recipient])?;
    }
    Ok(outbox)
}

/// Recipients are `UUIDv4`, so they come from the same `id` helper as events.
fn recipient(index: usize) -> String {
    id(900 + index)
}

#[test]
fn an_empty_outbox_wakes_nobody_and_says_it_is_idle() {
    let worker = recipient(1);
    let outbox = Outbox::new(1);
    let wake = outbox.wake(&worker, WakeMark::new(), 0);
    assert_eq!(wake, Wake::Quiet(Quiet::Idle));
    assert!(!wake.is_actionable());
    assert!(wake.events().is_empty());
}

#[test]
fn a_recipient_with_nothing_addressed_to_it_is_idle_not_unchanged() -> Outcome {
    let worker = recipient(1);
    let other = recipient(4);
    // Three events exist; none is for this recipient. "Idle" and "unchanged" are different
    // facts and an operator reads them differently.
    let outbox = addressed(3, &worker)?;
    assert_eq!(
        outbox.wake(&other, WakeMark::new(), 0),
        Wake::Quiet(Quiet::Idle)
    );
    Ok(())
}

#[test]
fn one_wake_carries_every_outstanding_event_rather_than_one_each() -> Outcome {
    let worker = recipient(1);
    // The batching claim, stated as a count: five events, one wake.
    let outbox = addressed(5, &worker)?;
    let wake = outbox.wake(&worker, WakeMark::new(), 0);
    match &wake {
        Wake::Actionable { events, through } => {
            assert_eq!(events.len(), 5);
            assert_eq!(events, &(1..=5).map(id).collect::<Vec<_>>());
            assert_eq!(*through, 5);
        }
        Wake::Quiet(ground) => return Err(format!("expected a wake, got {}", ground.name()).into()),
    }
    Ok(())
}

#[test]
fn the_events_accessor_returns_the_batch_on_an_actionable_wake() -> Outcome {
    // Found by mutation testing: `Wake::events -> &[]` survived, because the only assertion
    // on the accessor was `assert!(wake.events().is_empty())` on a QUIET wake -- the identity
    // element -- while every actionable case read the `events` field by destructuring and
    // never went through the accessor at all (F129).
    let worker = recipient(1);
    let outbox = addressed(4, &worker)?;
    let wake = outbox.wake(&worker, WakeMark::new(), 0);
    assert!(wake.is_actionable());
    assert_eq!(wake.events(), &(1..=4).map(id).collect::<Vec<_>>());
    // The accessor and the field must agree; a reader that used one and a writer the other
    // is the shape this case exists to refuse.
    match &wake {
        Wake::Actionable { events, .. } => assert_eq!(wake.events(), events.as_slice()),
        Wake::Quiet(ground) => return Err(format!("expected a wake, got {}", ground.name()).into()),
    }
    Ok(())
}

#[test]
fn a_second_wake_for_the_same_events_is_unchanged() -> Outcome {
    let worker = recipient(1);
    // The dedup claim: waking, then asking again with the advanced mark, costs nothing.
    let outbox = addressed(3, &worker)?;
    let first = outbox.wake(&worker, WakeMark::new(), 0);
    let Wake::Actionable { through, .. } = first else {
        return Err("expected a first wake".into());
    };
    let mark = WakeMark::new().advanced(through, 0, 0);
    assert_eq!(outbox.wake(&worker, mark, 0), Wake::Quiet(Quiet::Unchanged));
    // And again: quiet is stable, not a one-shot.
    assert_eq!(
        outbox.wake(&worker, mark, 100),
        Wake::Quiet(Quiet::Unchanged)
    );
    Ok(())
}

#[test]
fn a_new_event_after_a_wake_wakes_again_with_only_the_new_one() -> Outcome {
    let worker = recipient(1);
    let mut outbox = addressed(2, &worker)?;
    let mark = WakeMark::new().advanced(2, 0, 0);
    let identity = id(3);
    outbox.enqueue(
        Committed::new(&identity, Visibility::Public, Commit::witness(1, 3))?,
        &[&worker],
    )?;
    match outbox.wake(&worker, mark, 0) {
        Wake::Actionable { events, through } => {
            // Only the new one: the first two were already paid for.
            assert_eq!(events, vec![id(3)]);
            assert_eq!(through, 3);
        }
        Wake::Quiet(ground) => return Err(format!("expected a wake, got {}", ground.name()).into()),
    }
    Ok(())
}

#[test]
fn a_settled_delivery_does_not_keep_waking() -> Outcome {
    let worker = recipient(1);
    let mut outbox = addressed(2, &worker)?;
    outbox.record(&id(1), &worker, Delivery::Delivered)?;
    match outbox.wake(&worker, WakeMark::new(), 0) {
        Wake::Actionable { events, .. } => assert_eq!(events, vec![id(2)]),
        Wake::Quiet(ground) => return Err(format!("expected a wake, got {}", ground.name()).into()),
    }
    outbox.record(&id(2), &worker, Delivery::Delivered)?;
    assert_eq!(
        outbox.wake(&worker, WakeMark::new(), 0),
        Wake::Quiet(Quiet::Idle)
    );
    Ok(())
}

#[test]
fn a_quiet_window_holds_a_wake_that_would_otherwise_fire() -> Outcome {
    let worker = recipient(1);
    // Controlled idle: the events ARE actionable and new, and the answer is still no request.
    let outbox = addressed(3, &worker)?;
    let held = WakeMark::new().advanced(0, 1_000, 500);
    assert_eq!(outbox.wake(&worker, held, 1_000), Wake::Quiet(Quiet::Held));
    assert_eq!(outbox.wake(&worker, held, 1_499), Wake::Quiet(Quiet::Held));
    // The boundary from the other side, or only the holding half would be checked.
    assert!(outbox.wake(&worker, held, 1_500).is_actionable());
    Ok(())
}

#[test]
fn a_burst_inside_one_window_becomes_one_wake_not_one_each() -> Outcome {
    let worker = recipient(1);
    // The batching claim under a window: ten events arriving while held cost one request.
    let mut outbox = Outbox::new(1);
    let held = WakeMark::new().advanced(0, 0, 1_000);
    for index in 1..=10_usize {
        let identity = id(index);
        outbox.enqueue(
            Committed::new(
                &identity,
                Visibility::Public,
                Commit::witness(1, u64::try_from(index)?),
            )?,
            &[&worker],
        )?;
        assert_eq!(
            outbox.wake(&worker, held, u64::try_from(index)?),
            Wake::Quiet(Quiet::Held),
            "event {index} must not wake inside the window"
        );
    }
    match outbox.wake(&worker, held, 1_000) {
        Wake::Actionable { events, through } => {
            assert_eq!(events.len(), 10);
            assert_eq!(through, 10);
        }
        Wake::Quiet(ground) => return Err(format!("expected a wake, got {}", ground.name()).into()),
    }
    Ok(())
}

#[test]
fn a_wake_batch_is_bounded_and_the_remainder_is_not_forgotten() -> Outcome {
    let worker = recipient(1);
    let outbox = addressed(MAX_REPLAY + 5, &worker)?;
    let first = outbox.wake(&worker, WakeMark::new(), 0);
    let Wake::Actionable { events, through } = first else {
        return Err("expected a first wake".into());
    };
    assert_eq!(events.len(), MAX_REPLAY);
    assert_eq!(through, u64::try_from(MAX_REPLAY)?);
    // The five beyond the bound come in the next wake rather than being dropped.
    match outbox.wake(&worker, WakeMark::new().advanced(through, 0, 0), 0) {
        Wake::Actionable { events, through } => {
            assert_eq!(events.len(), 5);
            assert_eq!(through, u64::try_from(MAX_REPLAY + 5)?);
        }
        Wake::Quiet(ground) => {
            return Err(format!("the remainder must wake, got {}", ground.name()).into());
        }
    }
    Ok(())
}

#[test]
fn every_quiet_ground_has_a_distinct_name() {
    let names: Vec<&str> = [Quiet::Idle, Quiet::Unchanged, Quiet::Held]
        .into_iter()
        .map(Quiet::name)
        .collect();
    assert_eq!(names, ["idle", "unchanged", "held"]);
    let unique: std::collections::BTreeSet<&str> = names.iter().copied().collect();
    assert_eq!(unique.len(), names.len());
}

#[test]
fn a_wake_mark_records_what_it_was_advanced_with() {
    let fresh = WakeMark::new();
    assert_eq!((fresh.through(), fresh.quiet_until_ms()), (0, 0));
    // Off the origin: three advances asserted whole, so a mark frozen at its first value
    // or one that ignored the window would both fail.
    let first = fresh.advanced(7, 100, 50);
    assert_eq!((first.through(), first.quiet_until_ms()), (7, 150));
    let second = first.advanced(9, 200, 25);
    assert_eq!((second.through(), second.quiet_until_ms()), (9, 225));
    let third = second.advanced(11, 1_000, 0);
    assert_eq!((third.through(), third.quiet_until_ms()), (11, 1_000));
}

#[test]
fn a_wake_mark_window_saturates_rather_than_wrapping() {
    // A wrapping add here would turn a long hold into an immediate wake.
    let mark = WakeMark::new().advanced(1, u64::MAX, 1_000);
    assert_eq!(mark.quiet_until_ms(), u64::MAX);
}

#[test]
fn two_recipients_wake_independently() -> Outcome {
    let alpha = recipient(2);
    let beta = recipient(3);
    let mut outbox = Outbox::new(1);
    let identity = id(1);
    outbox.enqueue(
        Committed::new(&identity, Visibility::Public, Commit::witness(1, 1))?,
        &[&alpha, &beta],
    )?;
    assert!(outbox.wake(&alpha, WakeMark::new(), 0).is_actionable());
    // Advancing alpha's mark says nothing about beta: the mark is the caller's, per recipient.
    let alpha_done = WakeMark::new().advanced(1, 0, 0);
    assert_eq!(
        outbox.wake(&alpha, alpha_done, 0),
        Wake::Quiet(Quiet::Unchanged)
    );
    assert!(outbox.wake(&beta, WakeMark::new(), 0).is_actionable());
    Ok(())
}

#[test]
fn a_pending_failure_still_wakes_because_it_is_not_settled() -> Outcome {
    let worker = recipient(1);
    let mut outbox = addressed(1, &worker)?;
    outbox.record(
        &id(1),
        &worker,
        Delivery::Pending(FailureCategory::Unreachable),
    )?;
    assert!(outbox.wake(&worker, WakeMark::new(), 0).is_actionable());
    Ok(())
}
