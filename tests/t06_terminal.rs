//! Independent terminal nonacceptance oracles, designed before bodies.
//! Fixtures reuse prior independent T06 Store API setup, not terminal logic.
//! Supplied settlement/evidence records do not prove an actual checker ran or
//! cleanup occurred. Read-only SQL observes durable rows; no fixture writes SQL.

use habitat_engine::contracts::receipt::Name;
use habitat_engine::contracts::{Generation, Sha256Digest, UuidV4};
use habitat_engine::store::{
    Allocation, Effect, Expected, Object, Principal, Settlement, Stop, Stopped, Store, Submission,
    TaskHead, Verification, VerificationVerdict,
};
use rusqlite::{Connection, OpenFlags, types::Value};
use std::collections::BTreeMap;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const GEN: &str = "06600000-0000-4000-8000-000000000001";
const EPOCH: &str = "06600000-0000-4000-8000-000000000002";
const TASK: &str = "06600000-0000-4000-8000-000000000003";
const KEY: &str = "06600000-0000-4000-8000-000000000004";
const ADMITTED: &str = "06600000-0000-4000-8000-000000000005";
const ATTEMPT: &str = "06600000-0000-4000-8000-000000000006";
const STARTED: &str = "06600000-0000-4000-8000-000000000007";
const SETTLED: &str = "06600000-0000-4000-8000-000000000008";
const VERIFIED: &str = "06600000-0000-4000-8000-000000000009";
const STOPPED: &str = "06600000-0000-4000-8000-00000000000a";
const CANCELLED: &str = "06600000-0000-4000-8000-00000000000b";
const STAGE: &str = "06600000-0000-4000-8000-00000000000c";
const OTHER: &str = "06600000-0000-4000-8000-00000000000d";
const SECOND_ATTEMPT: &str = "06600000-0000-4000-8000-00000000000e";
const SECOND_START: &str = "06600000-0000-4000-8000-00000000000f";
const SECOND_SETTLE: &str = "06600000-0000-4000-8000-000000000010";
const SECOND_VERIFY: &str = "06600000-0000-4000-8000-000000000011";
const ACCEPTED: &str = "06600000-0000-4000-8000-000000000012";
const CRITERIA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SUBJECT: &str = "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
const EVIDENCE: &[u8] = b"independent terminal owner fixture: all supplied effects settled\n";
const REASON: &str = "development_stop";
static NEXT_AREA: AtomicU64 = AtomicU64::new(0);

type Ledger = BTreeMap<String, Vec<Vec<Value>>>;

struct Area {
    path: PathBuf,
    device: u64,
    inode: u64,
}
impl Area {
    fn new() -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "hee3-t06-terminal-{}-{}",
            std::process::id(),
            NEXT_AREA.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        let metadata = fs::symlink_metadata(&path).unwrap();
        Self {
            path,
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
    fn open(&self, create: bool) -> Store {
        Store::open(&self.path, id(GEN), id(EPOCH), create, deadline()).unwrap()
    }
    fn inspect(&self) -> Connection {
        Connection::open_with_flags(
            self.path
                .join("generations")
                .join(GEN)
                .join("ledger.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap()
    }
    fn object_path(&self, object: &Object) -> PathBuf {
        let hex = &object.digest()[7..];
        self.path
            .join("generations")
            .join(GEN)
            .join("objects/sha256")
            .join(&hex[..2])
            .join(hex)
    }
    fn ledger(&self) -> Ledger {
        let db = self.inspect();
        let names: Vec<String> = {
            let mut statement = db.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap();
            statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .map(Result::unwrap)
                .collect()
        };
        let mut result = BTreeMap::new();
        for name in names {
            let escaped = name.replace('"', "\"\"");
            let columns = db
                .prepare(&format!("SELECT * FROM \"{escaped}\""))
                .unwrap()
                .column_count();
            let order = (1..=columns)
                .map(|index| index.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let mut statement = db
                .prepare(&format!("SELECT * FROM \"{escaped}\" ORDER BY {order}"))
                .unwrap();
            let rows = statement
                .query_map([], |row| {
                    (0..columns)
                        .map(|column| row.get(column))
                        .collect::<rusqlite::Result<Vec<Value>>>()
                })
                .unwrap()
                .map(Result::unwrap)
                .collect();
            result.insert(name, rows);
        }
        db.close().unwrap();
        result
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        let metadata = fs::symlink_metadata(&self.path).unwrap();
        assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
        assert_eq!((metadata.dev(), metadata.ino()), (self.device, self.inode));
        fs::remove_dir_all(&self.path).unwrap();
    }
}

// Field order ensures the Store closes before Area removes its owned directory.
struct Rig {
    store: Store,
    area: Area,
    evidence: Object,
}
impl Rig {
    fn admitted() -> Self {
        let area = Area::new();
        let mut store = area.open(true);
        store
            .submit(
                Submission {
                    principal: &principal(),
                    key: id(KEY),
                    task: id(TASK),
                    event: id(ADMITTED),
                    request_bytes: b"terminal nonacceptance fixture",
                    criteria: Sha256Digest::parse(CRITERIA).unwrap(),
                    allocation: Allocation {
                        limit_ms: 1000,
                        work_ms: 800,
                        verify_ms: 200,
                    },
                },
                deadline(),
            )
            .unwrap();
        let evidence = store.publish(EVIDENCE, id(STAGE), deadline()).unwrap();
        Self {
            store,
            area,
            evidence,
        }
    }
    fn running() -> Self {
        let mut rig = Self::admitted();
        let attempt = rig
            .store
            .begin_attempt(
                id(TASK),
                generation("1"),
                id(ATTEMPT),
                id(STARTED),
                deadline(),
            )
            .unwrap();
        assert_eq!(
            (
                attempt.task_generation.as_str(),
                attempt.generation.as_str()
            ),
            ("2", "1")
        );
        rig
    }
    fn ready() -> Self {
        let mut rig = Self::running();
        rig.worker(Settlement {
            effect: Effect::None,
            used_ms: Some(30),
            cleanup_settled: true,
            ready_to_verify: true,
        });
        assert_eq!(rig.head().state, "verifying");
        rig
    }
    fn checked(verdict: VerificationVerdict) -> Self {
        let mut rig = Self::ready();
        rig.verify(verdict, Some(20), true);
        rig
    }
    fn head(&self) -> TaskHead {
        self.store.get(&principal(), id(TASK), deadline()).unwrap()
    }
    fn worker(&mut self, settlement: Settlement) {
        let current = self.head().generation;
        self.store
            .settle_attempt(
                &expected(&current, ATTEMPT, "1"),
                settlement,
                id(SETTLED),
                deadline(),
            )
            .unwrap();
    }
    fn verify(
        &mut self,
        verdict: VerificationVerdict,
        used_ms: Option<u64>,
        cleanup_settled: bool,
    ) {
        let current = self.head().generation;
        self.store
            .record_verification(
                &expected(&current, ATTEMPT, "1"),
                &Verification {
                    verdict,
                    subject: Sha256Digest::parse(SUBJECT).unwrap(),
                    evidence: self.evidence.clone(),
                    used_ms,
                    cleanup_settled,
                },
                id(VERIFIED),
                deadline(),
            )
            .unwrap();
    }
    fn cancel(&mut self) {
        let current = self.head().generation;
        self.store
            .cancel(id(TASK), generation(&current), id(CANCELLED), deadline())
            .unwrap();
    }
    fn stop(&mut self) -> Stopped {
        let current = self.head().generation;
        finish(
            &mut self.store,
            &self.evidence,
            &principal(),
            TASK,
            &current,
            STOPPED,
            deadline(),
        )
        .unwrap()
    }
    fn refused_unchanged(&mut self) {
        let before = self.head();
        let ledger = self.area.ledger();
        assert!(
            finish(
                &mut self.store,
                &self.evidence,
                &principal(),
                TASK,
                &before.generation,
                STOPPED,
                deadline()
            )
            .is_err()
        );
        assert_eq!(self.head(), before);
        assert_eq!(self.area.ledger(), ledger);
    }
    fn accounting(&self, spent: u64, work: u64, verify: u64) {
        let head = self.head();
        assert_eq!(
            (
                head.spent_ms,
                head.reserved_work_ms,
                head.reserved_verify_ms
            ),
            (spent, work, verify)
        );
    }
    fn stopped_rows(&self, state: &str) {
        let db = self.area.inspect();
        let row: (String, String, String, String, String) = db
            .query_row(
                "SELECT task_id,event_id,evidence_digest,reason,state FROM task_stops",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(
            row,
            (
                TASK.to_owned(),
                STOPPED.to_owned(),
                self.evidence.digest().to_owned(),
                REASON.to_owned(),
                state.to_owned()
            )
        );
        let (kind, body, sequence): (String, Vec<u8>, i64) = db
            .query_row(
                "SELECT kind,body,sequence FROM events WHERE id=?",
                [STOPPED],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(kind, "task_stopped");
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"reason":REASON,"evidence":{"digest":self.evidence.digest(),"size":EVIDENCE.len()}})
        );
        let artifact_size: String = db
            .query_row(
                "SELECT CAST(size AS TEXT) FROM artifacts WHERE digest=?",
                [self.evidence.digest()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(artifact_size, EVIDENCE.len().to_string());
        for table in ["acceptances", "acceptance_objects"] {
            let count: i64 = db
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(count, 0);
        }
        let stops: i64 = db
            .query_row("SELECT count(*) FROM task_stops", [], |row| row.get(0))
            .unwrap();
        let outbox: i64 = db
            .query_row("SELECT count(*) FROM outbox", [], |row| row.get(0))
            .unwrap();
        assert_eq!((stops, outbox), (1, 1));
        assert_eq!(
            self.store.pending_delivery(256, deadline()).unwrap(),
            vec![(
                STOPPED.to_owned(),
                "1000:operator".to_owned(),
                u64::try_from(sequence).unwrap()
            )]
        );
        assert!(self.head().accepted_event.is_none());
        db.close().unwrap();
    }
}

fn id(value: &str) -> UuidV4<'_> {
    UuidV4::parse(value).unwrap()
}
fn generation(value: &str) -> Generation {
    value.parse().unwrap()
}
fn principal() -> Principal {
    Principal::new(1000, "operator").unwrap()
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn expected(current: &str, attempt: &'static str, ordinal: &str) -> Expected<'static> {
    Expected {
        task: id(TASK),
        task_generation: generation(current),
        attempt: id(attempt),
        attempt_generation: generation(ordinal),
    }
}
fn finish(
    store: &mut Store,
    evidence: &Object,
    principal: &Principal,
    task: &str,
    current: &str,
    event: &str,
    until: Instant,
) -> Result<Stopped, habitat_engine::store::Error> {
    store.finish_unaccepted(
        principal,
        Stop {
            task: id(task),
            generation: generation(current),
            reason: &Name::new(REASON).unwrap(),
            evidence,
            event: id(event),
        },
        until,
    )
}

#[test]
fn no_attempt_failure_atomically_releases_reservations_and_records_exact_stop() {
    let mut rig = Rig::admitted();
    rig.accounting(0, 800, 200);
    let stopped = rig.stop();
    assert_eq!(stopped.generation, "2");
    assert!(!stopped.cancelled);
    assert_eq!(rig.head().state, "failed");
    rig.accounting(0, 0, 0);
    rig.stopped_rows("failed");
}

#[test]
fn no_attempt_cancellation_has_cancelled_terminal_outcome_and_one_notification() {
    let mut rig = Rig::admitted();
    rig.cancel();
    rig.accounting(0, 800, 200);
    let stopped = rig.stop();
    assert_eq!(stopped.generation, "3");
    assert!(stopped.cancelled);
    assert_eq!(rig.head().state, "cancelled");
    assert!(rig.head().cancellation);
    rig.accounting(0, 0, 0);
    rig.stopped_rows("cancelled");
}

#[test]
fn settled_worker_failure_can_stop_without_a_checker_that_never_launched() {
    let mut rig = Rig::running();
    rig.worker(Settlement {
        effect: Effect::None,
        used_ms: Some(30),
        cleanup_settled: true,
        ready_to_verify: false,
    });
    assert_eq!(rig.head().state, "repair_pending");
    rig.accounting(30, 770, 200);
    assert_eq!(rig.stop().generation, "4");
    rig.accounting(30, 0, 0);
    rig.stopped_rows("failed");
}

#[test]
fn known_committed_effect_is_settled_and_does_not_prevent_nonacceptance() {
    let mut rig = Rig::running();
    rig.worker(Settlement {
        effect: Effect::Committed,
        used_ms: Some(30),
        cleanup_settled: true,
        ready_to_verify: false,
    });
    rig.stop();
    rig.accounting(30, 0, 0);
    rig.stopped_rows("failed");
}

#[test]
fn a_recorded_checker_pass_can_stop_without_becoming_acceptance() {
    let mut rig = Rig::checked(VerificationVerdict::Passed);
    assert_eq!(rig.head().state, "verifying");
    rig.accounting(50, 770, 180);
    assert_eq!(rig.stop().generation, "5");
    assert_eq!(rig.head().state, "failed");
    rig.accounting(50, 0, 0);
    rig.stopped_rows("failed");
}

#[test]
fn failed_checker_can_stop_from_repair_pending_with_known_spend_preserved() {
    let mut rig = Rig::checked(VerificationVerdict::Failed);
    assert_eq!(rig.head().state, "repair_pending");
    rig.stop();
    rig.accounting(50, 0, 0);
    rig.stopped_rows("failed");
}

#[test]
fn settled_invalid_error_and_timeout_returns_can_be_finalized_truthfully() {
    for verdict in [
        VerificationVerdict::Invalid,
        VerificationVerdict::Error,
        VerificationVerdict::Timeout,
    ] {
        let mut rig = Rig::checked(verdict);
        assert_eq!(rig.head().state, "failed");
        rig.accounting(50, 770, 180);
        rig.stop();
        rig.accounting(50, 0, 0);
        rig.stopped_rows("failed");
    }
}

#[test]
fn cancellation_after_failed_verification_wins_before_terminal_commit() {
    let mut rig = Rig::checked(VerificationVerdict::Error);
    rig.cancel();
    assert_eq!(rig.head().state, "cancellation_requested");
    let stopped = rig.stop();
    assert_eq!(stopped.generation, "6");
    assert!(stopped.cancelled);
    rig.accounting(50, 0, 0);
    rig.stopped_rows("cancelled");
}

#[test]
fn cancellation_after_pass_does_not_turn_a_stop_into_acceptance() {
    let mut rig = Rig::checked(VerificationVerdict::Passed);
    rig.cancel();
    assert!(rig.stop().cancelled);
    rig.accounting(50, 0, 0);
    rig.stopped_rows("cancelled");
}

#[test]
fn running_worker_refuses_terminal_release_and_all_ledger_rows_stay_unchanged() {
    let mut rig = Rig::running();
    rig.refused_unchanged();
    rig.accounting(0, 800, 200);
}

#[test]
fn pending_or_unknown_effect_keeps_reservations_even_with_known_cost_and_cleanup() {
    for effect in [Effect::Pending, Effect::Unknown] {
        let mut rig = Rig::running();
        rig.worker(Settlement {
            effect,
            used_ms: Some(30),
            cleanup_settled: true,
            ready_to_verify: false,
        });
        let before = rig.head();
        rig.refused_unchanged();
        assert_eq!(rig.head(), before);
        assert!(before.reserved_work_ms > 0);
    }
}

#[test]
fn missing_worker_cost_is_not_zero_and_cannot_release_the_reservation() {
    let mut rig = Rig::running();
    rig.worker(Settlement {
        effect: Effect::None,
        used_ms: None,
        cleanup_settled: true,
        ready_to_verify: false,
    });
    rig.refused_unchanged();
    rig.accounting(0, 800, 200);
}

#[test]
fn known_worker_cost_without_settled_cleanup_still_cannot_stop() {
    let mut rig = Rig::running();
    rig.worker(Settlement {
        effect: Effect::None,
        used_ms: Some(30),
        cleanup_settled: false,
        ready_to_verify: false,
    });
    rig.refused_unchanged();
    rig.accounting(0, 800, 200);
}

#[test]
fn unknown_verifier_cost_keeps_the_entire_remaining_verification_reserve() {
    let mut rig = Rig::ready();
    rig.verify(VerificationVerdict::Passed, None, true);
    rig.refused_unchanged();
    rig.accounting(30, 770, 200);
}

#[test]
fn known_verifier_measurement_with_unsettled_cleanup_is_not_terminal_spend() {
    let mut rig = Rig::ready();
    rig.verify(VerificationVerdict::Failed, Some(20), false);
    rig.refused_unchanged();
    rig.accounting(30, 770, 200);
}

#[test]
fn cancellation_before_checker_dispatch_needs_an_actual_zero_cost_cancelled_return() {
    let mut rig = Rig::ready();
    rig.refused_unchanged();
    rig.cancel();
    rig.refused_unchanged();
    rig.accounting(30, 770, 200);
    rig.verify(VerificationVerdict::Cancelled, Some(0), true);
    assert!(rig.stop().cancelled);
    rig.accounting(30, 0, 0);
    rig.stopped_rows("cancelled");
}

#[test]
fn a_previous_attempt_verifier_return_cannot_settle_the_current_attempt() {
    let mut rig = Rig::checked(VerificationVerdict::Failed);
    let started = rig
        .store
        .begin_attempt(
            id(TASK),
            generation("4"),
            id(SECOND_ATTEMPT),
            id(SECOND_START),
            deadline(),
        )
        .unwrap();
    assert_eq!(
        (
            started.task_generation.as_str(),
            started.generation.as_str()
        ),
        ("5", "2")
    );
    rig.store
        .settle_attempt(
            &expected("5", SECOND_ATTEMPT, "2"),
            Settlement {
                effect: Effect::None,
                used_ms: Some(10),
                cleanup_settled: true,
                ready_to_verify: true,
            },
            id(SECOND_SETTLE),
            deadline(),
        )
        .unwrap();
    rig.refused_unchanged();
    rig.accounting(60, 760, 180);
    rig.cancel();
    rig.refused_unchanged();
    let current = rig.head().generation;
    rig.store
        .record_verification(
            &expected(&current, SECOND_ATTEMPT, "2"),
            &Verification {
                verdict: VerificationVerdict::Cancelled,
                subject: Sha256Digest::parse(SUBJECT).unwrap(),
                evidence: rig.evidence.clone(),
                used_ms: Some(0),
                cleanup_settled: true,
            },
            id(SECOND_VERIFY),
            deadline(),
        )
        .unwrap();
    assert!(rig.stop().cancelled);
    rig.accounting(60, 0, 0);
    rig.stopped_rows("cancelled");
}

#[test]
fn a_repeated_stop_cannot_append_an_event_or_change_the_historical_result() {
    let mut rig = Rig::admitted();
    rig.stop();
    let before = rig.head();
    let ledger = rig.area.ledger();
    for event in [STOPPED, OTHER] {
        assert!(
            finish(
                &mut rig.store,
                &rig.evidence,
                &principal(),
                TASK,
                &before.generation,
                event,
                deadline()
            )
            .is_err()
        );
        assert_eq!(rig.head(), before);
        assert_eq!(rig.area.ledger(), ledger);
    }
}

#[test]
fn stale_stop_generation_refuses_without_losing_a_valid_current_stop() {
    let mut rig = Rig::checked(VerificationVerdict::Passed);
    let before = rig.head();
    let ledger = rig.area.ledger();
    assert!(
        finish(
            &mut rig.store,
            &rig.evidence,
            &principal(),
            TASK,
            "3",
            STOPPED,
            deadline()
        )
        .is_err()
    );
    assert_eq!(rig.head(), before);
    assert_eq!(rig.area.ledger(), ledger);
    rig.stop();
    rig.stopped_rows("failed");
}

#[test]
fn invisible_uid_and_role_cannot_stop_the_owners_task() {
    let mut rig = Rig::admitted();
    let before = rig.head();
    let ledger = rig.area.ledger();
    for foreign in [
        Principal::new(1001, "operator").unwrap(),
        Principal::new(1000, "viewer").unwrap(),
    ] {
        assert!(
            finish(
                &mut rig.store,
                &rig.evidence,
                &foreign,
                TASK,
                "1",
                STOPPED,
                deadline()
            )
            .is_err()
        );
        assert_eq!(rig.head(), before);
        assert_eq!(rig.area.ledger(), ledger);
    }
    rig.stop();
    rig.stopped_rows("failed");
}

#[test]
fn unknown_task_cannot_consume_a_real_event_or_evidence() {
    let mut rig = Rig::admitted();
    let before = rig.head();
    let ledger = rig.area.ledger();
    assert!(
        finish(
            &mut rig.store,
            &rig.evidence,
            &principal(),
            OTHER,
            "1",
            STOPPED,
            deadline()
        )
        .is_err()
    );
    assert_eq!(rig.head(), before);
    assert_eq!(rig.area.ledger(), ledger);
    rig.stop();
    rig.stopped_rows("failed");
}

#[test]
fn accepted_task_cannot_be_reclassified_as_an_unaccepted_stop() {
    let mut rig = Rig::checked(VerificationVerdict::Passed);
    let proof = rig
        .store
        .prepare_verified_acceptance(
            &expected("4", ATTEMPT, "1"),
            id(ACCEPTED),
            Sha256Digest::parse(SUBJECT).unwrap(),
            &rig.evidence,
            std::slice::from_ref(&rig.evidence),
            deadline(),
        )
        .unwrap();
    rig.store.accept(&proof, 0, deadline()).unwrap();
    rig.refused_unchanged();
    assert_eq!(rig.head().state, "accepted");
    assert_eq!(rig.head().accepted_event.as_deref(), Some(ACCEPTED));
    assert_eq!(
        rig.store.pending_delivery(256, deadline()).unwrap().len(),
        1
    );
}

#[test]
fn an_object_reference_from_a_different_store_is_not_retained_stop_evidence() {
    let mut rig = Rig::admitted();
    let other = Area::new();
    let other_store = other.open(true);
    let object = other_store
        .publish(b"only present in the foreign store", id(STAGE), deadline())
        .unwrap();
    let before = rig.head();
    let ledger = rig.area.ledger();
    assert!(
        finish(
            &mut rig.store,
            &object,
            &principal(),
            TASK,
            "1",
            STOPPED,
            deadline()
        )
        .is_err()
    );
    assert_eq!(rig.head(), before);
    assert_eq!(rig.area.ledger(), ledger);
    rig.stop();
    rig.stopped_rows("failed");
}

#[test]
fn same_length_corruption_is_rehashed_before_any_terminal_state_or_release() {
    let mut rig = Rig::admitted();
    let path = rig.area.object_path(&rig.evidence);
    let metadata = fs::symlink_metadata(&path).unwrap();
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    let original = fs::read(&path).unwrap();
    let mut changed = original.clone();
    changed[0] ^= 1;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, &changed).unwrap();
    fs::set_permissions(&path, metadata.permissions()).unwrap();
    let current = rig.head().generation;
    assert!(matches!(
        finish(
            &mut rig.store,
            &rig.evidence,
            &principal(),
            TASK,
            &current,
            STOPPED,
            deadline()
        ),
        Err(habitat_engine::store::Error::Corrupt)
    ));
    rig.refused_unchanged();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, &original).unwrap();
    fs::set_permissions(&path, metadata.permissions()).unwrap();
    rig.stop();
    rig.stopped_rows("failed");
}

#[test]
fn event_collision_rolls_back_stop_artifact_binding_release_and_outbox_together() {
    let mut rig = Rig::admitted();
    let before = rig.head();
    let ledger = rig.area.ledger();
    assert!(
        finish(
            &mut rig.store,
            &rig.evidence,
            &principal(),
            TASK,
            &before.generation,
            ADMITTED,
            deadline()
        )
        .is_err()
    );
    assert_eq!(rig.head(), before);
    assert_eq!(rig.area.ledger(), ledger);
    rig.stop();
    rig.accounting(0, 0, 0);
    rig.stopped_rows("failed");
}

#[test]
fn expired_deadline_cannot_commit_a_stop_or_consume_its_event_identity() {
    let mut rig = Rig::admitted();
    let before = rig.head();
    let ledger = rig.area.ledger();
    assert!(
        finish(
            &mut rig.store,
            &rig.evidence,
            &principal(),
            TASK,
            "1",
            STOPPED,
            Instant::now().checked_sub(Duration::from_secs(1)).unwrap()
        )
        .is_err()
    );
    assert_eq!(rig.head(), before);
    assert_eq!(rig.area.ledger(), ledger);
    rig.stop();
    rig.stopped_rows("failed");
}

#[test]
fn durable_stop_survives_reopen_and_later_cancel_preserves_historical_failure() {
    let mut rig = Rig::checked(VerificationVerdict::Failed);
    rig.stop();
    let stopped = rig.head();
    let ledger = rig.area.ledger();
    let Rig {
        store,
        area,
        evidence,
    } = rig;
    drop(store);
    let mut store = area.open(false);
    assert_eq!(
        store.get(&principal(), id(TASK), deadline()).unwrap(),
        stopped
    );
    assert_eq!(
        store
            .cancel(
                id(TASK),
                generation(&stopped.generation),
                id(CANCELLED),
                deadline()
            )
            .unwrap(),
        stopped.generation
    );
    assert_eq!(
        store.get(&principal(), id(TASK), deadline()).unwrap(),
        stopped
    );
    assert_eq!(area.ledger(), ledger);
    let reopened = Rig {
        store,
        area,
        evidence,
    };
    reopened.stopped_rows("failed");
}

#[test]
fn stop_notification_acknowledgement_is_exact_recipient_scoped_and_does_not_accept() {
    let mut rig = Rig::admitted();
    rig.stop();
    let stopped = rig.head();
    let ledger = rig.area.ledger();
    assert!(
        rig.store
            .acknowledge_delivery(id(STOPPED), "1001:operator", deadline())
            .is_err()
    );
    assert_eq!(rig.area.ledger(), ledger);
    rig.store
        .acknowledge_delivery(id(STOPPED), "1000:operator", deadline())
        .unwrap();
    assert!(
        rig.store
            .pending_delivery(256, deadline())
            .unwrap()
            .is_empty()
    );
    assert_eq!(rig.head(), stopped);
    rig.store
        .acknowledge_delivery(id(STOPPED), "1000:operator", deadline())
        .unwrap();
    assert_eq!(rig.head(), stopped);
    assert!(rig.head().accepted_event.is_none());
}

#[test]
fn spending_the_exact_verification_reserve_can_still_finalize_without_refund() {
    let mut rig = Rig::ready();
    rig.verify(VerificationVerdict::Failed, Some(200), true);
    rig.accounting(230, 770, 0);
    rig.stop();
    rig.accounting(230, 0, 0);
    rig.stopped_rows("failed");
}

#[test]
fn prepared_acceptance_and_new_work_cannot_reopen_a_committed_unaccepted_stop() {
    let mut rig = Rig::checked(VerificationVerdict::Passed);
    let proof = rig
        .store
        .prepare_verified_acceptance(
            &expected("4", ATTEMPT, "1"),
            id(ACCEPTED),
            Sha256Digest::parse(SUBJECT).unwrap(),
            &rig.evidence,
            std::slice::from_ref(&rig.evidence),
            deadline(),
        )
        .unwrap();
    rig.stop();
    let stopped = rig.head();
    let ledger = rig.area.ledger();
    assert!(rig.store.accept(&proof, 0, deadline()).is_err());
    assert!(
        rig.store
            .begin_attempt(
                id(TASK),
                generation(&stopped.generation),
                id(SECOND_ATTEMPT),
                id(SECOND_START),
                deadline()
            )
            .is_err()
    );
    assert_eq!(rig.head(), stopped);
    assert_eq!(rig.area.ledger(), ledger);
    rig.stopped_rows("failed");
}

fn preparation(used_ms: Option<u64>) -> Settlement {
    Settlement {
        effect: Effect::None,
        used_ms,
        cleanup_settled: true,
        ready_to_verify: false,
    }
}
fn finish_preparation(
    rig: &mut Rig,
    current: &str,
    event: &str,
    facts: Settlement,
) -> Result<Stopped, habitat_engine::store::Error> {
    rig.store.finish_preparation(
        &principal(),
        Stop {
            task: id(TASK),
            generation: generation(current),
            reason: &Name::new(REASON).unwrap(),
            evidence: &rig.evidence,
            event: id(event),
        },
        facts,
        deadline(),
    )
}

#[test]
fn preparation_failure_charges_measured_usage_and_atomically_commits_stop_outbox() {
    let mut rig = Rig::admitted();
    let stopped = finish_preparation(&mut rig, "1", STOPPED, preparation(Some(37))).unwrap();
    assert_eq!(stopped.generation, "2");
    assert!(!stopped.cancelled);
    assert_eq!(rig.head().state, "failed");
    rig.accounting(37, 0, 0);
    assert!(rig.head().accepted_event.is_none());
    let db = rig.area.inspect();
    for (table, expected) in [
        ("attempts", 0),
        ("acceptances", 0),
        ("task_stops", 1),
        ("outbox", 1),
    ] {
        let count: i64 = db
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, expected);
    }
    let (kind, body, digest): (String, Vec<u8>, String) = db.query_row(
        "SELECT e.kind,e.body,s.evidence_digest FROM events e JOIN task_stops s ON s.event_id=e.id WHERE e.id=?",
        [STOPPED], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)),
    ).unwrap();
    assert_eq!(kind, "task_stopped");
    assert_eq!(digest, rig.evidence.digest());
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "reason":REASON,
            "evidence":{"digest":rig.evidence.digest(),"size":EVIDENCE.len()},
            "preparation":{"used_ms":37,"effect":"none","cleanup_settled":true,"ready_to_verify":false}
        })
    );
    assert_eq!(
        rig.store.pending_delivery(256, deadline()).unwrap().len(),
        1
    );
}

#[test]
fn preparation_unknown_or_unsettled_observations_preserve_all_durable_rows() {
    let mut rig = Rig::admitted();
    let before = rig.area.ledger();
    for facts in [
        preparation(None),
        Settlement {
            effect: Effect::Unknown,
            ..preparation(Some(2))
        },
        Settlement {
            effect: Effect::Pending,
            ..preparation(Some(2))
        },
        Settlement {
            effect: Effect::Committed,
            ..preparation(Some(2))
        },
        Settlement {
            cleanup_settled: false,
            ..preparation(Some(2))
        },
        Settlement {
            ready_to_verify: true,
            ..preparation(Some(2))
        },
    ] {
        assert!(matches!(
            finish_preparation(&mut rig, "1", STOPPED, facts),
            Err(habitat_engine::store::Error::Outstanding)
        ));
        assert_eq!(rig.area.ledger(), before);
        rig.accounting(0, 800, 200);
    }
    finish_preparation(&mut rig, "1", STOPPED, preparation(Some(2))).unwrap();
    rig.accounting(2, 0, 0);
}

#[test]
fn preparation_exact_work_reserve_is_charged_but_adjacent_overrun_refuses() {
    let mut rig = Rig::admitted();
    let before = rig.area.ledger();
    for used in [801, u64::MAX] {
        assert!(matches!(
            finish_preparation(&mut rig, "1", STOPPED, preparation(Some(used))),
            Err(habitat_engine::store::Error::Budget)
        ));
        assert_eq!(rig.area.ledger(), before);
    }
    finish_preparation(&mut rig, "1", STOPPED, preparation(Some(800))).unwrap();
    rig.accounting(800, 0, 0);
}

#[test]
fn preparation_zero_measured_cost_is_distinct_from_unknown() {
    let mut rig = Rig::admitted();
    finish_preparation(&mut rig, "1", STOPPED, preparation(Some(0))).unwrap();
    rig.accounting(0, 0, 0);
    assert_eq!(rig.head().state, "failed");
}

#[test]
fn preparation_cannot_charge_an_existing_attempt_even_after_settlement() {
    for mut rig in [Rig::running(), Rig::checked(VerificationVerdict::Failed)] {
        let before = rig.area.ledger();
        let current = rig.head().generation;
        assert!(matches!(
            finish_preparation(&mut rig, &current, STOPPED, preparation(Some(1))),
            Err(habitat_engine::store::Error::Outstanding)
        ));
        assert_eq!(rig.area.ledger(), before);
    }
}

#[test]
fn preparation_cancel_precedence_requires_current_generation_and_keeps_cost() {
    let mut rig = Rig::admitted();
    rig.store
        .cancel(id(TASK), generation("1"), id(CANCELLED), deadline())
        .unwrap();
    let before = rig.area.ledger();
    assert!(matches!(
        finish_preparation(&mut rig, "1", STOPPED, preparation(Some(5))),
        Err(habitat_engine::store::Error::Conflict)
    ));
    assert_eq!(rig.area.ledger(), before);
    let current = rig.head().generation;
    let stopped = finish_preparation(&mut rig, &current, STOPPED, preparation(Some(5))).unwrap();
    assert!(stopped.cancelled);
    assert_eq!(rig.head().state, "cancelled");
    rig.accounting(5, 0, 0);
    assert_eq!(
        rig.store.pending_delivery(256, deadline()).unwrap().len(),
        1
    );
}

#[test]
fn preparation_stop_event_collision_rolls_back_charge_and_reserve_release() {
    let mut rig = Rig::admitted();
    let before = rig.area.ledger();
    assert!(finish_preparation(&mut rig, "1", ADMITTED, preparation(Some(4))).is_err());
    assert_eq!(rig.area.ledger(), before);
    rig.accounting(0, 800, 200);
    finish_preparation(&mut rig, "1", STOPPED, preparation(Some(4))).unwrap();
    rig.accounting(4, 0, 0);
}

#[test]
fn preparation_without_failure_keeps_admission_and_reserves_for_first_attempt() {
    let mut rig = Rig::admitted();
    rig.accounting(0, 800, 200);
    assert_eq!(rig.head().state, "admitted");
    assert!(
        rig.store
            .pending_delivery(256, deadline())
            .unwrap()
            .is_empty()
    );
    let attempt = rig
        .store
        .begin_attempt(
            id(TASK),
            generation("1"),
            id(ATTEMPT),
            id(STARTED),
            deadline(),
        )
        .unwrap();
    assert_eq!(attempt.task_generation, "2");
    rig.accounting(0, 800, 200);
}

#[test]
fn preparation_charge_survives_reopen_and_cannot_be_applied_twice() {
    let mut rig = Rig::admitted();
    finish_preparation(&mut rig, "1", STOPPED, preparation(Some(9))).unwrap();
    let before = rig.area.ledger();
    let current = rig.head().generation;
    assert!(matches!(
        finish_preparation(&mut rig, &current, OTHER, preparation(Some(9))),
        Err(habitat_engine::store::Error::Conflict)
    ));
    assert_eq!(rig.area.ledger(), before);
    let Rig {
        store,
        area,
        evidence,
    } = rig;
    drop(store);
    let store = area.open(false);
    let reopened = Rig {
        store,
        area,
        evidence,
    };
    reopened.accounting(9, 0, 0);
    assert_eq!(reopened.area.ledger(), before);
}
