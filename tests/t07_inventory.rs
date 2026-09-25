//! Actual Store close/reopen controls. Expected facts are independent fixed values.
//! Direct SQL is read-only except separately named closed-ledger corruption cases.
use habitat_engine::contracts::{Generation, Sha256Digest, UuidV4, receipt::Name};
use habitat_engine::store::{
    Allocation, Effect, Error, Expected, Object, Principal, RecoveryInventory, RecoveryLimits,
    Settlement, Stop, Store, Submission, Verification, VerificationVerdict,
};
use rusqlite::{Connection, OpenFlags, types::Value};
use std::{
    collections::BTreeMap,
    fs::{self, DirBuilder},
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
const GEN: &str = "07000000-0000-4000-8000-000000000001";
const EPOCH: &str = "07000000-0000-4000-8000-000000000002";
const TASK: &str = "07000000-0000-4000-8000-000000000003";
const KEY: &str = "07000000-0000-4000-8000-000000000004";
const ADMIT: &str = "07000000-0000-4000-8000-000000000005";
const ATTEMPT: &str = "07000000-0000-4000-8000-000000000006";
const START: &str = "07000000-0000-4000-8000-000000000007";
const SETTLE: &str = "07000000-0000-4000-8000-000000000008";
const CHECK: &str = "07000000-0000-4000-8000-000000000009";
const ACCEPT: &str = "07000000-0000-4000-8000-00000000000a";
const CANCEL: &str = "07000000-0000-4000-8000-00000000000b";
const STOP: &str = "07000000-0000-4000-8000-00000000000c";
const STAGE: &str = "07000000-0000-4000-8000-00000000000d";
const OTHER: &str = "07000000-0000-4000-8000-00000000000e";
const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
static NEXT: AtomicU64 = AtomicU64::new(0);
fn id(s: &str) -> UuidV4<'_> {
    UuidV4::parse(s).unwrap()
}
fn generation(s: &str) -> Generation {
    s.parse().unwrap()
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn limits() -> RecoveryLimits {
    RecoveryLimits {
        rows: 1024,
        bytes: 1_048_576,
    }
}
fn principal() -> Principal {
    Principal::new(1000, "operator").unwrap()
}
struct Area {
    path: PathBuf,
    identity: (u64, u64),
}
impl Area {
    fn new() -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "t07-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
        let m = fs::symlink_metadata(&path).unwrap();
        Self {
            path,
            identity: (m.dev(), m.ino()),
        }
    }
    fn db(&self) -> PathBuf {
        self.path
            .join("generations")
            .join(GEN)
            .join("ledger.sqlite3")
    }
    fn open(&self, create: bool) -> Store {
        Store::open(&self.path, id(GEN), id(EPOCH), create, deadline()).unwrap()
    }
    fn ledger(&self) -> BTreeMap<String, Vec<Vec<Value>>> {
        let db = Connection::open_with_flags(
            self.db(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        let names:Vec<String>=db.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap().query_map([],|r|r.get(0)).unwrap().map(Result::unwrap).collect();
        let mut result = BTreeMap::new();
        for name in names {
            let sql = format!("SELECT * FROM \"{}\"", name.replace('"', "\"\""));
            let mut q = db.prepare(&sql).unwrap();
            let width = q.column_count();
            let mut rows: Vec<Vec<Value>> = q
                .query_map([], |r| (0..width).map(|i| r.get(i)).collect())
                .unwrap()
                .map(Result::unwrap)
                .collect();
            rows.sort_by_key(|r| format!("{r:?}"));
            result.insert(name, rows);
        }
        db.close().unwrap();
        result
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        let m = fs::symlink_metadata(&self.path).unwrap();
        assert_eq!((m.dev(), m.ino()), self.identity);
        assert!(m.is_dir() && !m.file_type().is_symlink());
        fs::remove_dir_all(&self.path).unwrap();
    }
}
struct Rig {
    store: Option<Store>,
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
                    event: id(ADMIT),
                    request_bytes: b"recovery inventory fixture",
                    workspace_id: id("28f00000-0000-4000-8000-00000000000a"),
                    criteria: Sha256Digest::parse(DIGEST).unwrap(),
                    allocation: Allocation {
                        limit_ms: 1_200_000,
                        work_ms: 900_000,
                        verify_ms: 300_000,
                    },
                },
                deadline(),
            )
            .unwrap();
        let evidence = store
            .publish(b"retained fixture evidence", id(STAGE), deadline())
            .unwrap();
        Self {
            store: Some(store),
            area,
            evidence,
        }
    }
    fn store(&mut self) -> &mut Store {
        self.store.as_mut().unwrap()
    }
    fn revision(&mut self) -> String {
        self.store()
            .get(&principal(), id(TASK), deadline())
            .unwrap()
            .generation
    }
    fn running() -> Self {
        let mut r = Self::admitted();
        r.store()
            .begin_attempt(
                id(TASK),
                generation("1"),
                id(ATTEMPT),
                id(START),
                deadline(),
            )
            .unwrap();
        r
    }
    fn expected(&mut self) -> Expected<'static> {
        Expected {
            task: id(TASK),
            task_generation: generation(&self.revision()),
            attempt: id(ATTEMPT),
            attempt_generation: generation("1"),
        }
    }
    fn settle(&mut self, effect: Effect, used_ms: Option<u64>, cleanup_settled: bool) {
        let expected = self.expected();
        self.store()
            .settle_attempt(
                &expected,
                Settlement {
                    effect,
                    used_ms,
                    cleanup_settled,
                    ready_to_verify: true,
                },
                id(SETTLE),
                deadline(),
            )
            .unwrap();
    }
    fn ready() -> Self {
        let mut r = Self::running();
        r.settle(Effect::None, Some(47_977), true);
        r
    }
    fn verify(
        &mut self,
        verdict: VerificationVerdict,
        used_ms: Option<u64>,
        cleanup_settled: bool,
    ) {
        let e = self.expected();
        let evidence = self.evidence.clone();
        self.store()
            .record_verification(
                &e,
                &Verification {
                    verdict,
                    subject: Sha256Digest::parse(DIGEST).unwrap(),
                    evidence,
                    used_ms,
                    cleanup_settled,
                },
                id(CHECK),
                deadline(),
            )
            .unwrap();
    }
    fn accept(&mut self) {
        let e = self.expected();
        let evidence = self.evidence.clone();
        let publication = self
            .store()
            .prepare_verified_acceptance(
                &e,
                id(ACCEPT),
                Sha256Digest::parse(DIGEST).unwrap(),
                &evidence,
                std::slice::from_ref(&evidence),
                deadline(),
            )
            .unwrap();
        self.store().accept(&publication, 0, deadline()).unwrap();
    }
    fn cancel(&mut self) {
        let revision = self.revision();
        self.store()
            .cancel(id(TASK), generation(&revision), id(CANCEL), deadline())
            .unwrap();
    }
    fn snapshot(&mut self) -> RecoveryInventory {
        self.store()
            .recovery_inventory(id(EPOCH), limits(), deadline())
            .unwrap()
    }
    fn reopen(&mut self) -> RecoveryInventory {
        let before = self.area.ledger();
        let old = self.snapshot();
        drop(self.store.take());
        self.store = Some(self.area.open(false));
        let new = self.snapshot();
        assert_eq!(old, new);
        assert_eq!(before, self.area.ledger());
        new
    }
    fn refused(&mut self, bound: RecoveryLimits, epoch: &str, end: Instant) {
        let before = self.area.ledger();
        assert!(
            self.store()
                .recovery_inventory(id(epoch), bound, end)
                .is_err()
        );
        assert_eq!(before, self.area.ledger());
        assert!(!self.snapshot().tasks.is_empty());
    }
}
#[test]
fn empty_inventory_is_complete_and_epoch_bound() {
    let a = Area::new();
    let mut s = a.open(true);
    let r = s
        .recovery_inventory(id(EPOCH), limits(), deadline())
        .unwrap();
    assert_eq!(
        (&*r.epoch, &*r.generation, &*r.mode),
        (EPOCH, GEN, "normal")
    );
    assert_eq!((r.rows, r.event_high_water), (1, 0));
    // Two literal36-byte UUIDs + normal6 + SQLite integer8.
    assert_eq!(r.payload_bytes, 86);
    assert!(r.tasks.is_empty() && r.pending_delivery.is_empty());
    drop(s);
    let mut s = a.open(false);
    assert_eq!(
        r,
        s.recovery_inventory(id(EPOCH), limits(), deadline())
            .unwrap()
    );
}
#[test]
fn admitted_reopen_keeps_exact_reserved_budget() {
    let mut r = Rig::admitted();
    let v = r.reopen();
    // Metadata86 + fixed admitted task projection172 + its bound workspace id, one literal36-byte
    // UUID (B14-P2c); not derived from inventory.
    assert_eq!((v.rows, v.payload_bytes), (2, 294));
    // Recovery reads the principal after the head columns, not from them (P2c mutants: a role
    // read one column early would be the workspace id).
    assert_eq!(
        (v.tasks[0].principal_uid, v.tasks[0].principal_role.as_str()),
        (1000, "operator")
    );
    assert_eq!(
        (
            v.tasks[0].head.spent_ms,
            v.tasks[0].head.reserved_work_ms,
            v.tasks[0].head.reserved_verify_ms
        ),
        (0, 900_000, 300_000)
    );
    assert_eq!(v.tasks[0].head.state, "admitted");
    assert!(v.attempts.is_empty());
    assert_eq!(v.event_high_water, 1);
}
#[test]
fn running_attempt_reopen_is_not_redispatch() {
    let mut r = Rig::running();
    let v = r.reopen();
    assert_eq!(v.attempts.len(), 1);
    assert_eq!(
        (
            &*v.attempts[0].id,
            &*v.attempts[0].state,
            &*v.attempts[0].generation
        ),
        (ATTEMPT, "running", "1")
    );
    assert_eq!(v.event_high_water, 2);
    assert_eq!(v.attempts[0].used_ms, None);
}
#[test]
fn settled_worker_without_verifier_remains_recovery_obligation() {
    let mut r = Rig::ready();
    let v = r.reopen();
    assert_eq!(
        (&*v.tasks[0].head.state, &*v.attempts[0].state),
        ("verifying", "settled")
    );
    assert!(v.verifications.is_empty() && v.acceptances.is_empty());
    assert_eq!(
        (
            v.tasks[0].head.spent_ms,
            v.tasks[0].head.reserved_work_ms,
            v.tasks[0].head.reserved_verify_ms
        ),
        (47_977, 852_023, 300_000)
    );
}
#[test]
fn unknown_effect_retains_reserves_after_reopen() {
    let mut r = Rig::running();
    r.settle(Effect::Unknown, Some(100), true);
    let v = r.reopen();
    assert_eq!(
        (&*v.tasks[0].head.state, &*v.attempts[0].effect),
        ("effect_unknown", "unknown")
    );
    assert_eq!(
        (v.tasks[0].head.spent_ms, v.tasks[0].head.reserved_work_ms),
        (0, 900_000)
    );
}
#[test]
fn unknown_usage_is_not_fabricated_zero() {
    let mut r = Rig::running();
    r.settle(Effect::None, None, true);
    let v = r.reopen();
    assert_eq!(v.attempts[0].used_ms, None);
    assert_eq!(v.tasks[0].head.reserved_work_ms, 900_000);
}
#[test]
fn incomplete_cleanup_is_not_settled_by_reopen() {
    let mut r = Rig::running();
    r.settle(Effect::Committed, Some(100), false);
    let v = r.reopen();
    assert_eq!(
        (
            &*v.attempts[0].state,
            &*v.attempts[0].effect,
            &*v.attempts[0].cleanup
        ),
        ("unknown", "committed", "unknown")
    );
}
#[test]
fn pending_effect_remains_distinct_from_unknown() {
    let mut r = Rig::running();
    r.settle(Effect::Pending, None, false);
    assert_eq!(r.reopen().attempts[0].effect, "pending");
}
#[test]
fn recorded_pass_is_not_acceptance() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, Some(20), true);
    let v = r.reopen();
    assert_eq!(v.verifications[0].verdict, "passed");
    assert!(v.acceptances.is_empty() && v.pending_delivery.is_empty());
    assert_eq!(v.tasks[0].head.state, "verifying");
    assert_eq!(v.verifications[0].evidence, r.evidence.digest());
}
#[test]
fn recorded_failure_remains_repair_pending() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Failed, Some(20), true);
    let v = r.reopen();
    assert_eq!(v.tasks[0].head.state, "repair_pending");
    assert_eq!(v.verifications[0].verdict, "failed");
}
#[test]
fn recorded_error_is_not_success() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Error, Some(20), true);
    let v = r.reopen();
    assert_eq!(v.tasks[0].head.state, "failed");
    assert_eq!(v.verifications[0].verdict, "error");
    assert!(v.acceptances.is_empty());
}
#[test]
fn unknown_verification_cost_retains_full_reservation() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, None, true);
    let v = r.reopen();
    assert_eq!(v.verifications[0].used_ms, None);
    assert_eq!(v.tasks[0].head.reserved_verify_ms, 300_000);
    assert_eq!(v.tasks[0].head.state, "effect_unknown");
}
#[test]
fn verifier_cleanup_false_remains_false() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, Some(20), false);
    let v = r.reopen();
    assert!(!v.verifications[0].cleanup_settled);
    assert_eq!(v.tasks[0].head.reserved_verify_ms, 300_000);
}
#[test]
fn acceptance_and_outbox_reopen_once_without_new_events() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, Some(20), true);
    r.accept();
    let v = r.reopen();
    assert_eq!(v.acceptances.len(), 1);
    assert_eq!(v.pending_delivery.len(), 1);
    assert_eq!(
        (&*v.acceptances[0].event, &*v.pending_delivery[0].event),
        (ACCEPT, ACCEPT)
    );
    assert_eq!(v.tasks[0].head.accepted_event.as_deref(), Some(ACCEPT));
    assert_eq!(v.event_high_water, 5);
}
#[test]
fn delivery_ack_removes_only_pending_notification() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, Some(20), true);
    r.accept();
    r.store()
        .acknowledge_delivery(id(ACCEPT), "1000:operator", deadline())
        .unwrap();
    let v = r.reopen();
    assert!(v.pending_delivery.is_empty());
    assert_eq!(v.acceptances.len(), 1);
    assert_eq!(v.tasks[0].head.state, "accepted");
}
#[test]
fn earlier_cancellation_survives_and_refuses_acceptance() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, Some(20), true);
    r.cancel();
    let v = r.reopen();
    assert!(v.tasks[0].head.cancellation);
    let expected = r.expected();
    let evidence = r.evidence.clone();
    assert!(matches!(
        r.store().prepare_verified_acceptance(
            &expected,
            id(ACCEPT),
            Sha256Digest::parse(DIGEST).unwrap(),
            &evidence,
            std::slice::from_ref(&evidence),
            deadline()
        ),
        Err(Error::Cancelled)
    ));
}
#[test]
fn cancellation_after_acceptance_does_not_rewrite_history() {
    let mut r = Rig::ready();
    r.verify(VerificationVerdict::Passed, Some(20), true);
    r.accept();
    let before = r.snapshot();
    r.cancel();
    assert_eq!(before, r.reopen());
    assert!(!before.tasks[0].head.cancellation);
}
#[test]
fn preattempt_terminal_stop_survives_reopen() {
    let mut r = Rig::admitted();
    let evidence = r.evidence.clone();
    r.store()
        .finish_preparation(
            &principal(),
            Stop {
                task: id(TASK),
                generation: generation("1"),
                reason: &Name::new("prep_failed").unwrap(),
                evidence: &evidence,
                event: id(STOP),
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(100),
                cleanup_settled: true,
                ready_to_verify: false,
            },
            deadline(),
        )
        .unwrap();
    let v = r.reopen();
    assert_eq!(v.stops[0].reason, "prep_failed");
    assert_eq!(v.tasks[0].head.spent_ms, 100);
    assert_eq!(v.tasks[0].head.reserved_work_ms, 0);
    assert!(v.attempts.is_empty());
    assert_eq!(v.pending_delivery[0].event, STOP);
}
#[test]
fn exact_aggregate_row_budget_succeeds() {
    let mut r = Rig::ready();
    let v = r.snapshot();
    let bounded = r
        .store()
        .recovery_inventory(
            id(EPOCH),
            RecoveryLimits {
                rows: v.rows,
                bytes: v.payload_bytes,
            },
            deadline(),
        )
        .unwrap();
    assert_eq!(v, bounded);
}
#[test]
fn one_less_row_budget_refuses_without_truncation() {
    let mut r = Rig::ready();
    let count = r.snapshot().rows;
    r.refused(
        RecoveryLimits {
            rows: count - 1,
            ..limits()
        },
        EPOCH,
        deadline(),
    );
}
#[test]
fn one_less_byte_budget_refuses_without_partial_result() {
    let mut r = Rig::ready();
    let bytes = r.snapshot().payload_bytes;
    r.refused(
        RecoveryLimits {
            bytes: bytes - 1,
            ..limits()
        },
        EPOCH,
        deadline(),
    );
}
#[test]
fn zero_row_bound_refuses() {
    Rig::admitted().refused(
        RecoveryLimits {
            rows: 0,
            ..limits()
        },
        EPOCH,
        deadline(),
    );
}
#[test]
fn excessive_row_bound_refuses() {
    Rig::admitted().refused(
        RecoveryLimits {
            rows: 1025,
            ..limits()
        },
        EPOCH,
        deadline(),
    );
}
#[test]
fn zero_payload_bound_refuses() {
    Rig::admitted().refused(
        RecoveryLimits {
            bytes: 0,
            ..limits()
        },
        EPOCH,
        deadline(),
    );
}
#[test]
fn excessive_payload_bound_refuses() {
    Rig::admitted().refused(
        RecoveryLimits {
            bytes: 1_048_577,
            ..limits()
        },
        EPOCH,
        deadline(),
    );
}
#[test]
fn stale_epoch_refuses_and_benign_read_still_works() {
    Rig::admitted().refused(limits(), OTHER, deadline());
}
#[test]
fn expired_deadline_refuses_and_new_read_still_works() {
    Rig::admitted().refused(
        limits(),
        EPOCH,
        Instant::now().checked_sub(Duration::from_secs(1)).unwrap(),
    );
}
#[test]
fn malformed_task_generation_is_not_successful_readback() {
    let mut r = Rig::admitted();
    drop(r.store.take());
    let db = Connection::open(r.area.db()).unwrap();
    db.execute("UPDATE tasks SET generation='bogus'", [])
        .unwrap();
    db.close().unwrap();
    r.store = Some(r.area.open(false));
    assert!(matches!(
        r.store()
            .recovery_inventory(id(EPOCH), limits(), deadline()),
        Err(Error::Corrupt)
    ));
}
/// B14-P2c: migration 4's CHECK admits any 36 characters, so recovery validates the id itself: a
/// 36-character non-UUID is corrupt, and NULL (a task admitted before migration 4) is not.
#[test]
fn a_task_workspace_is_a_uuid_or_unbound() {
    for (value, corrupt) in [(Some("x".repeat(36)), true), (None, false)] {
        let mut r = Rig::admitted();
        drop(r.store.take());
        let db = Connection::open(r.area.db()).unwrap();
        db.execute("UPDATE tasks SET workspace_id=?", [value.as_deref()])
            .unwrap();
        db.close().unwrap();
        r.store = Some(r.area.open(false));
        let read = r
            .store()
            .recovery_inventory(id(EPOCH), limits(), deadline());
        if corrupt {
            assert!(matches!(read, Err(Error::Corrupt)), "{value:?}: {read:?}");
        } else {
            assert!(read.is_ok(), "{value:?}: {read:?}");
        }
    }
}
#[test]
fn malformed_attempt_identity_is_refused() {
    let mut r = Rig::running();
    drop(r.store.take());
    let db = Connection::open(r.area.db()).unwrap();
    db.execute("UPDATE attempts SET id='bogus'", []).unwrap();
    db.close().unwrap();
    r.store = Some(r.area.open(false));
    assert!(matches!(
        r.store()
            .recovery_inventory(id(EPOCH), limits(), deadline()),
        Err(Error::Corrupt)
    ));
}
#[test]
fn duplicate_startup_cannot_take_inventory_custody() {
    let mut r = Rig::ready();
    // The contended open is waited on with a budget: a lock retry that ignored its window would
    // otherwise hang this case instead of failing it (the T07 closure's mutant run found one).
    let path = r.area.path.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let refused = matches!(
            Store::open(&path, id(GEN), id(EPOCH), false, deadline()),
            Err(Error::Locked)
        );
        let _ = sender.send(refused);
    });
    assert_eq!(
        receiver.recv_timeout(std::time::Duration::from_secs(5)),
        Ok(true),
        "a duplicate startup is refused Locked within 5 s"
    );
    assert_eq!(r.snapshot().attempts.len(), 1);
}

fn rostered() -> Rig {
    use habitat_engine::contracts::roster::{
        Availability, Kind, Locality, ObservationInput, ObservationSource, RosterDefinitionV1,
        Selection, Update,
    };
    use habitat_engine::store::{RequestSource, RosterStart};
    let mut r = Rig::admitted();
    let input = Update {
        idempotency_key: OTHER.to_owned(),
        record_id: None,
        expected_revision: None,
        definition: RosterDefinitionV1 {
            kind: Kind::Agent,
            display_name: "recovery-fixture".to_owned(),
            owner_id: "fixture-worker".to_owned(),
            version: "v1".to_owned(),
            capabilities: vec!["text".to_owned()],
            locality: Locality::Local,
            endpoint_ref: Some(OTHER.to_owned()),
            limitations: "literal inventory fixture, no dispatch".to_owned(),
        },
        audit_reason: "retained recovery fixture".to_owned(),
    };
    let raw = serde_json::to_vec(&serde_json::json!({"protocol":"hee3.control","version":1,"kind":"request","request_id":OTHER,"action":"roster.update","action_version":1,"idempotency_key":OTHER,"deadline_unix_ms":"1030000","authority":{"grant_id":OTHER,"scope_sha256":DIGEST},"precondition":null,"body":{"record_id":null,"definition":input.definition,"audit_reason":input.audit_reason}})).unwrap();
    let outcome = r
        .store()
        .roster_apply(
            &principal(),
            &[input],
            RequestSource::Native(&raw),
            deadline(),
        )
        .unwrap()
        .remove(0);
    let head = outcome.head;
    r.store()
        .roster_observe_worker(
            &principal(),
            &ObservationInput {
                record_id: head.record_id.clone(),
                record_version: head.record_version.clone(),
                owner_id: head.definition.owner_id.clone(),
                endpoint_ref: head.definition.endpoint_ref.clone(),
                instance_id: None,
                instance_generation: None,
                source: ObservationSource::Worker,
                observed_unix_ms: None,
                availability: Availability::Available,
                actual_identity: Some("fixture/worker".to_owned()),
                immutable_revision: None,
                capabilities: vec!["text".to_owned()],
                evidence_ref: OTHER.to_owned(),
            },
            deadline(),
        )
        .unwrap();
    let selections = [Selection {
        record_id: head.record_id.clone(),
        expected_revision: head.record_version,
        capabilities: vec!["text".to_owned()],
        local_only: true,
        version: Some("v1".to_owned()),
        ttl_ms: 60_000,
    }];
    r.store()
        .begin_rostered_attempt(
            RosterStart {
                principal: &principal(),
                task: id(TASK),
                expected: generation("1"),
                attempt: id(ATTEMPT),
                event: id(START),
                agent_record_id: &head.record_id,
                session: id(KEY),
                workspace: id(STAGE),
                selections: &selections,
                lease_ms: 1,
            },
            deadline(),
        )
        .unwrap();
    r
}
#[test]
fn rostered_attempt_retains_actual_session_workspace_and_pin() {
    let mut r = rostered();
    let v = r.reopen();
    assert_eq!(v.instances.len(), 1);
    assert_eq!(v.pins.len(), 1);
    assert_eq!(
        (
            &*v.instances[0].session_id,
            &*v.instances[0].workspace_ref,
            &*v.pins[0].attempt_id
        ),
        (KEY, STAGE, ATTEMPT)
    );
    assert_eq!(
        v.instances[0].agent_record_id,
        v.pins[0].record.head.record_id
    );
}
#[test]
fn expired_logical_lease_does_not_release_reservation_or_work() {
    let mut r = rostered();
    std::thread::sleep(Duration::from_millis(5));
    let v = r.snapshot();
    let now = r
        .store()
        .roster_snapshot(&principal(), deadline())
        .unwrap()
        .now;
    assert_eq!(now.epoch, v.instances[0].started.epoch);
    assert!(now.monotonic_ms > v.instances[0].lease_expires_monotonic_ms);
    assert_eq!(
        v.instances[0].lease_expires_monotonic_ms,
        v.instances[0].started.monotonic_ms + 1
    );
    assert_eq!(v.tasks[0].head.reserved_work_ms, 900_000);
    assert_eq!(v.attempts[0].state, "running");
    assert_eq!(v, r.reopen());
}
#[test]
fn roster_payload_counts_towards_aggregate_byte_bound() {
    let mut r = rostered();
    let v = r.snapshot();
    assert!(v.payload_bytes > 1000);
    r.refused(
        RecoveryLimits {
            bytes: v.payload_bytes - 1,
            ..limits()
        },
        EPOCH,
        deadline(),
    );
}
#[test]
fn roster_rows_count_towards_aggregate_row_bound() {
    let mut r = rostered();
    let v = r.snapshot();
    r.refused(
        RecoveryLimits {
            rows: v.rows - 1,
            ..limits()
        },
        EPOCH,
        deadline(),
    );
}
fn corrupt_roster(sql: &str) -> Rig {
    corrupt_roster_with(|_| sql.to_owned())
}
/// Closed-ledger corruption whose SQL may be derived from the rostered ledger
/// itself; exactly one `roster_pins` row must change.
fn corrupt_roster_with(plant: impl FnOnce(&Area) -> String) -> Rig {
    let mut r = rostered();
    drop(r.store.take());
    let sql = plant(&r.area);
    let db = Connection::open(r.area.db()).unwrap();
    assert_eq!(
        db.execute(&sql, []).unwrap(),
        1,
        "corruption must change one row"
    );
    db.close().unwrap();
    r.store = Some(r.area.open(false));
    r
}
/// The single retained pin body, read through a separate read-only connection.
fn pin_body(area: &Area) -> serde_json::Value {
    let db = Connection::open_with_flags(
        area.db(),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .unwrap();
    let body: Vec<u8> = db
        .query_row("SELECT body FROM roster_pins", [], |row| row.get(0))
        .unwrap();
    db.close().unwrap();
    serde_json::from_slice(&body).unwrap()
}
fn pin_stamps(pin: &serde_json::Value) -> (u64, u64) {
    (
        pin["selected_at"]["monotonic_ms"].as_u64().unwrap(),
        pin["record"]["observation"]["received"]["monotonic_ms"]
            .as_u64()
            .unwrap(),
    )
}
/// A selection stamp that provably precedes its observation. Both stamps are
/// milliseconds since the store opened, so a literal 0 equals the original
/// whenever observe-then-select finished inside one millisecond and plants
/// nothing (QC-F2, 2026-09-21); and `received - 1` does not exist when the
/// observation itself carries 0, which every tmpfs run showed. The plant is
/// therefore derived from the selection stamp: the observation is moved one
/// millisecond past it, so the selection regresses relative to the
/// observation for every elapsed time. The pair must differ from the original
/// before the review is invoked.
fn regressed_selection_clock(area: &Area) -> String {
    let (selected, received) = pin_stamps(&pin_body(area));
    let planted = selected.checked_add(1).expect(
        "regressed_selection_clock: selection stamp at u64::MAX leaves no later observation stamp to plant",
    );
    assert_ne!(
        (selected, planted),
        (selected, received),
        "regressed_selection_clock: planted observation stamp {planted} equals the original; a plant that equals the original is not a plant"
    );
    format!(
        "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.received.monotonic_ms',{planted}) AS BLOB)"
    )
}
#[test]
fn mismatched_roster_instance_body_refuses() {
    let mut r = corrupt_roster(
        "UPDATE roster_instances SET body=CAST(json_set(body,'$.session_id','bad-uuid') AS BLOB)",
    );
    assert!(matches!(
        r.store()
            .recovery_inventory(id(EPOCH), limits(), deadline()),
        Err(Error::Corrupt)
    ));
}
#[test]
fn mismatched_roster_pin_body_refuses() {
    let mut r = corrupt_roster(
        "UPDATE roster_pins SET body=CAST(json_set(body,'$.attempt_id','07000000-0000-4000-8000-00000000000e') AS BLOB)",
    );
    assert!(matches!(
        r.store()
            .recovery_inventory(id(EPOCH), limits(), deadline()),
        Err(Error::Corrupt)
    ));
}
#[test]
fn forged_instance_attempt_generation_refuses() {
    let mut r = corrupt_roster(
        "UPDATE roster_instances SET body=CAST(json_set(body,'$.attempt_generation','2') AS BLOB)",
    );
    assert!(matches!(
        r.store()
            .recovery_inventory(id(EPOCH), limits(), deadline()),
        Err(Error::Corrupt)
    ));
}
#[test]
fn oversized_malformed_roster_body_refuses_bound_before_decode() {
    let mut r = corrupt_roster("UPDATE roster_instances SET body=zeroblob(1048577)");
    assert!(matches!(
        r.store()
            .recovery_inventory(id(EPOCH), limits(), deadline()),
        Err(Error::Bound)
    ));
}

#[test]
fn reopened_receiver_epoch_does_not_reinterpret_old_lease() {
    let mut r = rostered();
    let before = r.snapshot();
    let reopened = r.reopen();
    let now = r
        .store()
        .roster_snapshot(&principal(), deadline())
        .unwrap()
        .now;
    assert_ne!(now.epoch, before.instances[0].started.epoch);
    assert_eq!(before, reopened);
    assert_eq!(reopened.tasks[0].head.reserved_work_ms, 900_000);
    assert_eq!(reopened.attempts[0].cleanup, "pending");
}

#[test]
fn review_malformed_nested_pin_observation_must_refuse() {
    let mut r = corrupt_roster(
        "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.input.evidence_ref','not-a-uuid') AS BLOB)",
    );
    let result = r
        .store()
        .recovery_inventory(id(EPOCH), limits(), deadline());
    if let Ok(snapshot) = &result {
        let input = &snapshot.pins[0].record.observation.as_ref().unwrap().input;
        assert_eq!(input.evidence_ref, "not-a-uuid");
        assert!(input.validate().is_err());
    }
    assert!(
        matches!(result, Err(Error::Corrupt)),
        "typed inventory admitted malformed nested observation: {result:?}"
    );
}

#[test]
fn review_nested_pin_identity_and_binding_faults_must_refuse() {
    type Plant = fn(&Area) -> String;
    let cases: [(&str, Plant); 9] = [
        ("observation_id", |_| {
            "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.id','bad-uuid') AS BLOB)".to_owned()
        }),
        ("received_epoch", |_| {
            "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.received.epoch','bad-uuid') AS BLOB)".to_owned()
        }),
        ("foreign_record", |_| {
            "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.input.record_id','07000000-0000-4000-8000-00000000000e') AS BLOB)".to_owned()
        }),
        ("foreign_version", |_| {
            "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.input.record_version','999') AS BLOB)".to_owned()
        }),
        ("foreign_owner", |_| {
            "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.input.owner_id','foreign-owner') AS BLOB)".to_owned()
        }),
        ("unconfirmed_source", |_| {
            "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.confirmed_source',NULL) AS BLOB)".to_owned()
        }),
        ("foreign_clock_epoch", |_| {
            "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.received.epoch','07000000-0000-4000-8000-00000000000e') AS BLOB)".to_owned()
        }),
        ("regressed_selection_clock", regressed_selection_clock),
        ("malformed_cutoff", |_| {
            "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.head.observation_cutoff_unix_ms','not-decimal') AS BLOB)".to_owned()
        }),
    ];
    let mut admitted = Vec::new();
    for (label, plant) in cases {
        let mut r = corrupt_roster_with(plant);
        if label == "regressed_selection_clock" {
            // Read back what the ledger now holds before the review judges it.
            let (selected, received) = pin_stamps(&pin_body(&r.area));
            assert!(
                selected < received,
                "regressed_selection_clock: ledger holds selected_at {selected} not before received {received}"
            );
        }
        let result = r
            .store()
            .recovery_inventory(id(EPOCH), limits(), deadline());
        if let Ok(snapshot) = &result {
            let pin = &snapshot.pins[0];
            println!(
                "ADMITTED_NESTED_FAULT {label} original_selection_permits={} input_validate={:?}",
                pin.selection.permits(
                    &pin.record.head,
                    pin.record.observation.as_ref(),
                    &pin.selected_at
                ),
                pin.record.observation.as_ref().unwrap().input.validate()
            );
            admitted.push(label);
        } else {
            assert!(
                matches!(result, Err(Error::Corrupt)),
                "unexpected refusal for {label}: {result:?}"
            );
        }
    }
    assert!(
        admitted.is_empty(),
        "malformed/binding faults returned as success: {admitted:?}"
    );
}

#[test]
fn nested_pin_sequence_is_positive_and_within_captured_event_high_water() {
    for sequence in [0, 999_999] {
        let mut r = corrupt_roster(&format!(
            "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.observation.sequence',{sequence}) AS BLOB)"
        ));
        let before = r.area.ledger();
        assert!(matches!(
            r.store()
                .recovery_inventory(id(EPOCH), limits(), deadline()),
            Err(Error::Corrupt)
        ));
        assert_eq!(before, r.area.ledger());
    }
}
#[test]
fn nested_pin_well_formed_cutoff_still_must_match_retained_receipt() {
    let mut r = corrupt_roster(
        "UPDATE roster_pins SET body=CAST(json_set(body,'$.record.head.observation_cutoff_unix_ms','1') AS BLOB)",
    );
    let before = r.area.ledger();
    assert!(matches!(
        r.store()
            .recovery_inventory(id(EPOCH), limits(), deadline()),
        Err(Error::Corrupt)
    ));
    assert_eq!(before, r.area.ledger());
}

fn rotate_fixture_mode(area: &Area, epoch: &str, mode: &str) {
    // Test-only durable change; no production epoch setter exists.
    let mut db = Connection::open(area.db()).unwrap();
    let tx = db.transaction().unwrap();
    tx.execute(
        "UPDATE ledger_meta SET epoch=?,mode=? WHERE singleton=1",
        [epoch, mode],
    )
    .unwrap();
    tx.commit().unwrap();
    db.close().unwrap();
}

#[test]
fn ordinary_open_refuses_reconciliation_and_inspection_observes_it() {
    let mut r = Rig::admitted();
    drop(r.store.take());
    rotate_fixture_mode(&r.area, EPOCH, "reconciliation");
    let before = r.area.ledger();
    assert!(matches!(
        Store::open(&r.area.path, id(GEN), id(EPOCH), false, deadline()),
        Err(Error::RecoveryRequired)
    ));
    let mut inspected =
        Store::open_inspection(&r.area.path, id(GEN), id(EPOCH), deadline()).unwrap();
    let inventory = inspected
        .recovery_inventory(id(EPOCH), limits(), deadline())
        .unwrap();
    assert_eq!(inventory.mode, "reconciliation");
    assert_eq!(inventory.tasks.len(), 1);
    drop(inspected);
    assert_eq!(before, r.area.ledger());
}

#[test]
fn inspection_open_never_creates_missing_state() {
    let a = Area::new();
    assert!(Store::open_inspection(&a.path, id(GEN), id(EPOCH), deadline()).is_err());
    assert!(!a.path.join("generations").exists());
}

#[test]
fn inspection_normal_mode_is_still_non_authorizing() {
    let mut r = Rig::admitted();
    drop(r.store.take());
    let before = r.area.ledger();
    let mut s = Store::open_inspection(&r.area.path, id(GEN), id(EPOCH), deadline()).unwrap();
    assert!(matches!(
        s.cancel(id(TASK), generation("1"), id(CANCEL), deadline()),
        Err(Error::InspectionOnly)
    ));
    assert!(matches!(
        s.begin_attempt(
            id(TASK),
            generation("1"),
            id(ATTEMPT),
            id(START),
            deadline()
        ),
        Err(Error::InspectionOnly)
    ));
    assert!(matches!(
        s.publish(b"must not publish", id(OTHER), deadline()),
        Err(Error::InspectionOnly)
    ));
    assert!(matches!(
        s.with_artifact_staging(deadline(), |_, _| panic!("must not issue staging")),
        Err(Error::InspectionOnly)
    ));
    assert!(matches!(
        s.pending_delivery(1, deadline()),
        Err(Error::InspectionOnly)
    ));
    assert!(matches!(
        s.acknowledge_delivery(id(ADMIT), "1000:operator", deadline()),
        Err(Error::InspectionOnly)
    ));
    assert_eq!(
        s.read_object(&r.evidence, deadline()).unwrap(),
        b"retained fixture evidence"
    );
    drop(s);
    assert_eq!(before, r.area.ledger());
}

#[test]
fn active_normal_handle_rechecks_mode_before_mutation_and_publication() {
    let mut r = Rig::admitted();
    // Sole test fixture deliberately bypasses the public owner to model changed durable mode.
    rotate_fixture_mode(&r.area, EPOCH, "reconciliation");
    assert!(matches!(
        r.store()
            .cancel(id(TASK), generation("1"), id(CANCEL), deadline()),
        Err(Error::RecoveryRequired)
    ));
    assert!(matches!(
        r.store()
            .publish(b"must not publish", id(OTHER), deadline()),
        Err(Error::RecoveryRequired)
    ));
    assert!(matches!(
        r.store()
            .with_artifact_staging(deadline(), |_, _| panic!("staging issued")),
        Err(Error::RecoveryRequired)
    ));
    assert!(matches!(
        r.store().pending_delivery(1, deadline()),
        Err(Error::RecoveryRequired)
    ));
    assert_eq!(
        r.store()
            .recovery_inventory(id(EPOCH), limits(), deadline())
            .unwrap()
            .tasks[0]
            .head
            .generation,
        "1"
    );
}

#[test]
fn active_normal_handle_rechecks_epoch_even_if_mode_remains_normal() {
    let mut r = Rig::admitted();
    rotate_fixture_mode(&r.area, OTHER, "normal");
    assert!(matches!(
        r.store()
            .cancel(id(TASK), generation("1"), id(CANCEL), deadline()),
        Err(Error::Conflict)
    ));
    assert!(matches!(
        r.store()
            .publish(b"must not publish", id(OTHER), deadline()),
        Err(Error::Conflict)
    ));
    assert!(matches!(
        r.store().pending_delivery(1, deadline()),
        Err(Error::Conflict)
    ));
}

fn cursor(epoch: &str, sequence: &str) -> habitat_engine::contracts::events::EventCursorV1 {
    habitat_engine::contracts::events::EventCursorV1 {
        epoch: epoch.into(),
        sequence: sequence.into(),
        filter_sha256: DIGEST.into(),
        visibility_revision: "0".into(),
        issued_unix_ms: "100".into(),
        expires_unix_ms: "200".into(),
    }
}

#[test]
fn cursor_equal_epoch_is_snapshot_only_and_old_epoch_requires_resync() {
    use habitat_engine::contracts::events::CursorDisposition;
    let mut r = Rig::admitted();
    let v = r
        .store()
        .recovery_inventory(id(EPOCH), limits(), deadline())
        .unwrap();
    assert_eq!(
        v.cursor_disposition(&cursor(EPOCH, "1"), 150).unwrap(),
        CursorDisposition::SnapshotOnly
    );
    assert_eq!(
        v.cursor_disposition(&cursor(EPOCH, "0"), 150).unwrap(),
        CursorDisposition::SnapshotOnly
    );
    assert_eq!(
        v.cursor_disposition(&cursor(OTHER, "18446744073709551615"), 150)
            .unwrap(),
        CursorDisposition::ResyncRequired
    );
}

#[test]
fn cursor_same_epoch_future_sequence_and_expired_or_future_time_refuse() {
    let mut r = Rig::admitted();
    let v = r
        .store()
        .recovery_inventory(id(EPOCH), limits(), deadline())
        .unwrap();
    assert!(matches!(
        v.cursor_disposition(&cursor(EPOCH, "2"), 150),
        Err(Error::Invalid)
    ));
    for now in [99, 200, 201] {
        assert!(matches!(
            v.cursor_disposition(&cursor(EPOCH, "1"), now),
            Err(Error::Invalid)
        ));
    }
    let mut c = cursor(EPOCH, "1");
    c.expires_unix_ms = "100".into();
    assert!(matches!(v.cursor_disposition(&c, 100), Err(Error::Invalid)));
}

#[test]
fn cursor_wire_rejects_duplicate_extra_missing_nonstring_and_noncanonical_scalars() {
    use habitat_engine::contracts::events::EventCursorV1;
    let valid = serde_json::to_string(&cursor(EPOCH, "0")).unwrap();
    assert!(
        serde_json::from_str::<EventCursorV1>(&valid)
            .unwrap()
            .validate()
            .is_ok()
    );
    for bad in [
        valid.replacen('{', "{\"epoch\":\"duplicate\",", 1),
        valid.replacen('{', "{\"extra\":true,", 1),
        valid.replace("\"sequence\":\"0\",", ""),
        valid.replace("\"sequence\":\"0\"", "\"sequence\":0"),
    ] {
        assert!(
            serde_json::from_str::<EventCursorV1>(&bad).is_err(),
            "{bad}"
        );
    }
    for bad in ["00", "-1", " 1", "18446744073709551616"] {
        let mut c = cursor(EPOCH, bad);
        assert!(c.validate().is_err());
        c = cursor(EPOCH, "0");
        c.visibility_revision = bad.into();
        assert!(c.validate().is_err());
    }
    let mut c = cursor(EPOCH, "18446744073709551615");
    assert!(c.validate().is_ok());
    c.epoch = EPOCH.replace("-4000-", "-1000-");
    assert!(c.validate().is_err());
    c = cursor(EPOCH, "0");
    c.filter_sha256 = "sha256:A".into();
    assert!(c.validate().is_err());
}
