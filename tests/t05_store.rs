//! Independent T05 persistence oracles over disposable private Store roots.
//! Roster behavior is primary; the final three schema controls are Store-owned.

use super::*;
use crate::contracts::roster::{
    ActiveAttemptPolicy, Availability, Disable, Freshness, InstanceState, Kind, Locality,
    Observation, ObservationInput, ObservationSource, Outcome, ReceiptTime, RosterDefinitionV1,
    RosterHeadV1, Selection, Update, freshness,
};
use rusqlite::types::Value as SqlValue;
use serde_json::json;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const GEN: &str = "00000000-0000-4000-8000-000000000001";
const LEDGER_EPOCH: &str = "00000000-0000-4000-8000-000000000002";
const TASK: &str = "00000000-0000-4000-8000-000000000003";
const TASK_KEY: &str = "00000000-0000-4000-8000-000000000004";
const ADMITTED: &str = "00000000-0000-4000-8000-000000000005";
const ATTEMPT: &str = "00000000-0000-4000-8000-000000000006";
const STARTED: &str = "00000000-0000-4000-8000-000000000007";
const SETTLED: &str = "00000000-0000-4000-8000-000000000008";
const ACCEPTED: &str = "00000000-0000-4000-8000-000000000009";
const SESSION: &str = "00000000-0000-4000-8000-000000000010";
const WORKSPACE: &str = "00000000-0000-4000-8000-000000000011";
const ENDPOINT: &str = "00000000-0000-4000-8000-000000000012";
const EVIDENCE: &str = "00000000-0000-4000-8000-000000000013";
const BOOT: &str = "90000000-0000-4000-8000-000000000001";
const CRITERIA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct Area {
    path: PathBuf,
    inode: u64,
}
impl Area {
    fn new() -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "hee3-t05-store-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
        let inode = fs::metadata(&path).unwrap().ino();
        Self { path, inode }
    }
    fn open(&self) -> Store {
        Store::open(&self.path, uuid(GEN), uuid(LEDGER_EPOCH), true, deadline()).unwrap()
    }
    fn reopen(&self) -> Store {
        Store::open(&self.path, uuid(GEN), uuid(LEDGER_EPOCH), false, deadline()).unwrap()
    }
    fn database(&self) -> PathBuf {
        self.path
            .join("generations")
            .join(GEN)
            .join("ledger.sqlite3")
    }
    fn inspect(&self) -> Connection {
        let connection = Connection::open_with_flags(
            self.database(),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        connection.execute_batch("PRAGMA query_only=ON;").unwrap();
        connection
    }
    fn object_bytes(&self, digest: &str) -> Vec<u8> {
        let hex = digest.strip_prefix("sha256:").unwrap();
        fs::read(
            self.path
                .join("generations")
                .join(GEN)
                .join("objects/sha256")
                .join(&hex[..2])
                .join(hex),
        )
        .unwrap()
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        if fs::symlink_metadata(&self.path).is_ok_and(|m| m.is_dir() && m.ino() == self.inode) {
            let result = fs::remove_dir_all(&self.path);
            if !std::thread::panicking() {
                result.unwrap();
            }
        }
    }
}
fn uuid(value: &str) -> UuidV4<'_> {
    UuidV4::parse(value).unwrap()
}
fn generation(value: u64) -> Generation {
    value.to_string().parse().unwrap()
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn principal() -> Principal {
    Principal::new(1000, "operator").unwrap()
}
fn id(value: u64) -> String {
    format!("80000000-0000-4000-8000-{value:012x}")
}
fn clock(store: &mut Store, elapsed: u64) {
    store.clock.test_time = Some(ReceiptTime {
        epoch: BOOT.to_owned(),
        monotonic_ms: elapsed,
        unix_ms: 1_000_000 + elapsed,
    });
}
fn definition() -> RosterDefinitionV1 {
    RosterDefinitionV1 {
        kind: Kind::Agent,
        display_name: "fixture-agent".to_owned(),
        owner_id: "fixture-worker".to_owned(),
        version: "v1".to_owned(),
        capabilities: vec!["text".to_owned()],
        locality: Locality::Local,
        endpoint_ref: Some(ENDPOINT.to_owned()),
        limitations: String::new(),
    }
}
fn create_request(key: u64) -> Update {
    Update {
        idempotency_key: id(key),
        record_id: None,
        expected_revision: None,
        definition: definition(),
        audit_reason: "Reviewed fixture".to_owned(),
    }
}
fn change(head: &RosterHeadV1, key: u64) -> Update {
    Update {
        idempotency_key: id(key),
        record_id: Some(head.record_id.clone()),
        expected_revision: Some(head.record_version.clone()),
        definition: head.definition.clone(),
        audit_reason: "Reviewed update".to_owned(),
    }
}
fn native_bytes(input: &Update) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "protocol":"hee3.control","version":1,"kind":"request","request_id":input.idempotency_key,
        "action":"roster.update","action_version":1,"idempotency_key":input.idempotency_key,
        "deadline_unix_ms":"1030000","authority":{"grant_id":id(9999),"scope_sha256":CRITERIA},
        "precondition":input.record_id.as_ref().map(|record| json!({"resource":"roster","id":record,"generation":input.expected_revision})),
        "body":{"record_id":input.record_id,"definition":input.definition,"audit_reason":input.audit_reason}
    })).unwrap()
}
fn manifest(inputs: &[Update]) -> Vec<u8> {
    let mut text = String::from("kind = \"hee3-roster-changes\"\nversion = 1\n");
    for input in inputs {
        text.push_str("\n[[updates]]\n");
        for (key, value) in [
            ("idempotency_key", Some(input.idempotency_key.as_str())),
            ("record_id", input.record_id.as_deref()),
            ("expected_revision", input.expected_revision.as_deref()),
            ("audit_reason", Some(input.audit_reason.as_str())),
        ] {
            if let Some(value) = value {
                writeln!(text, "{key} = {}", serde_json::to_string(value).unwrap()).unwrap();
            }
        }
        text.push_str("[updates.definition]\n");
        let value = serde_json::to_value(&input.definition).unwrap();
        for key in [
            "kind",
            "display_name",
            "owner_id",
            "version",
            "capabilities",
            "locality",
            "endpoint_ref",
            "limitations",
        ] {
            if !value[key].is_null() {
                writeln!(text, "{key} = {}", value[key]).unwrap();
            }
        }
    }
    text.into_bytes()
}
fn apply(store: &mut Store, input: &Update) -> Outcome {
    let bytes = native_bytes(input);
    store
        .roster_apply(
            &principal(),
            std::slice::from_ref(input),
            RequestSource::Native(&bytes),
            deadline(),
        )
        .unwrap()
        .remove(0)
}
fn create(store: &mut Store, key: u64) -> Outcome {
    apply(store, &create_request(key))
}
fn read(store: &Store, head: &RosterHeadV1) -> crate::contracts::roster::Record {
    store
        .roster_get(&principal(), uuid(&head.record_id), deadline())
        .unwrap()
}
fn observation_input(head: &RosterHeadV1) -> ObservationInput {
    ObservationInput {
        record_id: head.record_id.clone(),
        record_version: head.record_version.clone(),
        owner_id: head.definition.owner_id.clone(),
        endpoint_ref: head.definition.endpoint_ref.clone(),
        instance_id: None,
        instance_generation: None,
        source: ObservationSource::Worker,
        observed_unix_ms: None,
        availability: Availability::Available,
        actual_identity: Some("fixture/provider-model".to_owned()),
        immutable_revision: None,
        capabilities: head.definition.capabilities.clone(),
        evidence_ref: EVIDENCE.to_owned(),
    }
}
fn observe(store: &mut Store, head: &RosterHeadV1) -> Observation {
    store
        .roster_observe_worker(&principal(), &observation_input(head), deadline())
        .unwrap()
}
fn choose(head: &RosterHeadV1) -> Selection {
    Selection {
        record_id: head.record_id.clone(),
        expected_revision: head.record_version.clone(),
        capabilities: vec!["text".to_owned()],
        local_only: true,
        version: Some(head.definition.version.clone()),
        ttl_ms: 1000,
    }
}
fn disable_input(head: &RosterHeadV1, key: u64, policy: ActiveAttemptPolicy) -> Disable {
    Disable {
        idempotency_key: id(key),
        record_id: head.record_id.clone(),
        expected_revision: head.record_version.clone(),
        active_attempt_policy: policy,
        audit_reason: "Reviewed disable".to_owned(),
    }
}
fn disable(
    store: &mut Store,
    head: &RosterHeadV1,
    key: u64,
    policy: ActiveAttemptPolicy,
) -> Outcome {
    let input = disable_input(head, key, policy);
    let bytes=serde_json::to_vec(&json!({
        "protocol":"hee3.control","version":1,"kind":"request","request_id":input.idempotency_key,
        "action":"roster.disable","action_version":1,"idempotency_key":input.idempotency_key,"deadline_unix_ms":"1030000",
        "authority":{"grant_id":id(9999),"scope_sha256":CRITERIA},
        "precondition":{"resource":"roster","id":input.record_id,"generation":input.expected_revision},
        "body":{"record_id":input.record_id,"active_attempt_policy":input.active_attempt_policy,"audit_reason":input.audit_reason}
    })).unwrap();
    store
        .roster_disable(&principal(), &input, &bytes, deadline())
        .unwrap()
}
fn admit(store: &mut Store) {
    store
        .submit(
            Submission {
                principal: &principal(),
                key: uuid(TASK_KEY),
                task: uuid(TASK),
                event: uuid(ADMITTED),
                request_bytes: b"T05 independent task fixture",
                workspace_id: uuid("28f00000-0000-4000-8000-00000000000a"),
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
}
fn begin(
    store: &mut Store,
    agent: &RosterHeadV1,
    selections: &[Selection],
) -> Result<RosterAttempt> {
    store.begin_rostered_attempt(
        RosterStart {
            principal: &principal(),
            task: uuid(TASK),
            expected: generation(1),
            attempt: uuid(ATTEMPT),
            event: uuid(STARTED),
            agent_record_id: &agent.record_id,
            session: uuid(SESSION),
            workspace: uuid(WORKSPACE),
            selections,
            lease_ms: 500,
        },
        deadline(),
    )
}
fn counts(area: &Area, tables: &[&str]) -> Vec<i64> {
    let db = area.inspect();
    tables
        .iter()
        .map(|name| {
            db.query_row(&format!("SELECT count(*) FROM {name}"), [], |row| {
                row.get(0)
            })
            .unwrap()
        })
        .collect()
}
type LedgerState = Vec<Vec<Vec<SqlValue>>>;
/// Every table the ledger holds, read from `sqlite_master` (the world, not a list of it: a
/// hand-kept list missed `verifications`, `task_stops`, `task_dispositions` and would have missed
/// `attempt_bindings`; F65, review B14a-R2 G2).
fn tables(db: &rusqlite::Connection) -> Vec<String> {
    let mut query = db
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap();
    query
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<String>>>()
        .unwrap()
}
fn ledger(area: &Area) -> LedgerState {
    let db = area.inspect();
    let names = tables(&db);
    assert!(names.len() >= 20, "the ledger's tables: {names:?}");
    names
        .iter()
        .map(|table| {
            let mut query = db
                .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
                .unwrap();
            let columns = query.column_count();
            query
                .query_map([], |row| {
                    (0..columns)
                        .map(|column| row.get(column))
                        .collect::<rusqlite::Result<Vec<SqlValue>>>()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        })
        .collect()
}
fn injected<T>(result: Result<T>, point: CutPoint) {
    match result {
        Err(Error::Injected(name)) => assert_eq!(name, format!("{point:?}")),
        Err(other) => panic!("wrong fault: {other:?}"),
        Ok(_) => panic!("fault did not fire"),
    }
}

#[test]
fn profile_creation_persists_exact_definition_without_instances_or_task_dispatch() {
    let area = Area::new();
    let mut store = area.open();
    let result = create(&mut store, 1);
    assert_eq!(result.head.definition, definition());
    assert_eq!(result.head.record_version, "1");
    assert!(!result.head.disabled);
    assert_eq!(read(&store, &result.head).head, result.head);
    assert_eq!(
        counts(
            &area,
            &[
                "roster_records",
                "roster_revisions",
                "operations",
                "events",
                "tasks",
                "attempts",
                "roster_instances"
            ]
        ),
        vec![1, 1, 1, 1, 0, 0, 0]
    );
    drop(store);
    let reopened = area.reopen();
    assert_eq!(read(&reopened, &result.head).head, result.head);
}

#[test]
fn validated_update_keeps_exact_prior_revision_and_independent_current_readback() {
    let area = Area::new();
    let mut store = area.open();
    let first = create(&mut store, 1);
    let mut update = change(&first.head, 2);
    update.definition.display_name = "second definition".to_owned();
    let second = apply(&mut store, &update);
    assert_eq!(second.head.record_version, "2");
    assert_eq!(second.head.record_id, first.head.record_id);
    let db = area.inspect();
    let bytes: Vec<u8> = db
        .query_row(
            "SELECT definition FROM roster_revisions WHERE record_id=? AND revision='1'",
            [&first.head.record_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<RosterDefinitionV1>(&bytes).unwrap(),
        definition()
    );
    assert_eq!(
        read(&store, &second.head).head.definition,
        update.definition
    );
    assert_eq!(counts(&area, &["roster_revisions"]), vec![2]);
}

#[test]
fn exact_import_replay_returns_old_outcome_without_undoing_later_disable() {
    let area = Area::new();
    let mut store = area.open();
    let requests = vec![create_request(1)];
    let bytes = manifest(&requests);
    let first = store
        .roster_apply(
            &principal(),
            &requests,
            RequestSource::Import(&bytes),
            deadline(),
        )
        .unwrap();
    let disabled = disable(
        &mut store,
        &first[0].head,
        2,
        ActiveAttemptPolicy::LeaveRunning,
    );
    let before = ledger(&area);
    let replay = store
        .roster_apply(
            &principal(),
            &requests,
            RequestSource::Import(&bytes),
            deadline(),
        )
        .unwrap();
    assert_eq!(replay, first);
    assert_eq!(ledger(&area), before);
    assert!(read(&store, &disabled.head).head.disabled);
    drop(store);
    let mut reopened = area.reopen();
    assert_eq!(
        reopened
            .roster_apply(
                &principal(),
                &requests,
                RequestSource::Import(&bytes),
                deadline()
            )
            .unwrap(),
        first
    );
    assert!(read(&reopened, &disabled.head).head.disabled);
}

#[test]
fn changed_manifest_bytes_conflict_under_reused_keys_without_partial_updates() {
    let area = Area::new();
    let mut store = area.open();
    let requests = vec![create_request(1), create_request(2)];
    let bytes = manifest(&requests);
    let admitted = store
        .roster_apply(
            &principal(),
            &requests,
            RequestSource::Import(&bytes),
            deadline(),
        )
        .unwrap();
    let before = ledger(&area);
    let mut reformatted = bytes.clone();
    reformatted.push(b'\n');
    assert!(matches!(
        store.roster_apply(
            &principal(),
            &requests,
            RequestSource::Import(&reformatted),
            deadline()
        ),
        Err(Error::Conflict)
    ));
    assert_eq!(ledger(&area), before);
    let updates = vec![change(&admitted[0].head, 3), change(&admitted[1].head, 4)];
    let changed = manifest(&updates);
    assert_eq!(
        store
            .roster_apply(
                &principal(),
                &updates,
                RequestSource::Import(&changed),
                deadline()
            )
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn trusted_principal_visibility_and_mutation_role_do_not_transfer_with_record_ids() {
    let area = Area::new();
    let mut store = area.open();
    let first = create(&mut store, 1);
    let other = Principal::new(1001, "operator").unwrap();
    let reader = Principal::new(1000, "reader").unwrap();
    for denied in [&other, &reader] {
        assert!(
            store
                .roster_get(denied, uuid(&first.head.record_id), deadline())
                .is_err()
        );
        assert!(
            store
                .roster_by_key(denied, "roster.update", uuid(&id(1)), deadline())
                .is_err()
        );
        match store.roster_snapshot(denied, deadline()) {
            Ok(snapshot) => assert!(snapshot.records.is_empty()),
            Err(Error::Forbidden) => {}
            Err(other) => panic!("unexpected visibility failure: {other:?}"),
        }
    }
    let update = change(&first.head, 2);
    let bytes = native_bytes(&update);
    let before = ledger(&area);
    assert!(matches!(
        store.roster_apply(
            &reader,
            &[update],
            RequestSource::Native(&bytes),
            deadline()
        ),
        Err(Error::Forbidden)
    ));
    assert_eq!(ledger(&area), before);
    let independent = create_request(1);
    let other_bytes = native_bytes(&independent);
    let second = store
        .roster_apply(
            &other,
            &[independent],
            RequestSource::Native(&other_bytes),
            deadline(),
        )
        .unwrap();
    assert_ne!(second[0].head.record_id, first.head.record_id);
}

#[test]
fn invalid_later_manifest_member_leaves_every_admitted_record_unchanged() {
    let area = Area::new();
    let mut store = area.open();
    let existing = create(&mut store, 1);
    let before = ledger(&area);
    let valid = change(&existing.head, 2);
    let mut invalid = create_request(3);
    invalid.definition.display_name.clear();
    let inputs = vec![valid.clone(), invalid];
    let bytes = manifest(&inputs);
    assert!(
        store
            .roster_apply(
                &principal(),
                &inputs,
                RequestSource::Import(&bytes),
                deadline()
            )
            .is_err()
    );
    assert_eq!(ledger(&area), before);
    let inputs = vec![valid, create_request(3)];
    let bytes = manifest(&inputs);
    assert_eq!(
        store
            .roster_apply(
                &principal(),
                &inputs,
                RequestSource::Import(&bytes),
                deadline()
            )
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn one_stale_manifest_member_rolls_back_all_other_valid_changes() {
    let area = Area::new();
    let mut store = area.open();
    let first = create(&mut store, 1);
    let second = create(&mut store, 2);
    let old = change(&second.head, 4);
    let latest = apply(&mut store, &change(&second.head, 3));
    let before = ledger(&area);
    let inputs = vec![change(&first.head, 5), old];
    let bytes = manifest(&inputs);
    assert!(matches!(
        store.roster_apply(
            &principal(),
            &inputs,
            RequestSource::Import(&bytes),
            deadline()
        ),
        Err(Error::Conflict)
    ));
    assert_eq!(ledger(&area), before);
    let fresh = vec![change(&first.head, 6), change(&latest.head, 7)];
    let bytes = manifest(&fresh);
    assert_eq!(
        store
            .roster_apply(
                &principal(),
                &fresh,
                RequestSource::Import(&bytes),
                deadline()
            )
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn two_distinct_updates_of_one_expected_revision_cannot_both_commit() {
    let area = Area::new();
    let mut store = area.open();
    let first = create(&mut store, 1);
    let mut loser = change(&first.head, 3);
    loser.definition.display_name = "loser".to_owned();
    let winner = apply(&mut store, &change(&first.head, 2));
    let before = ledger(&area);
    let bytes = native_bytes(&loser);
    assert!(matches!(
        store.roster_apply(
            &principal(),
            &[loser],
            RequestSource::Native(&bytes),
            deadline()
        ),
        Err(Error::Conflict)
    ));
    assert_eq!(ledger(&area), before);
    assert_eq!(read(&store, &first.head).head, winner.head);
}

#[test]
fn allocated_create_uuid_collision_refuses_the_entire_batch_without_label_uniqueness() {
    let area = Area::new();
    let mut store = area.open();
    let existing = create(&mut store, 1);
    let before = ledger(&area);
    store.clock.test_ids = VecDeque::from(vec![id(100), existing.head.record_id.clone(), id(102)]);
    let input = create_request(2);
    let bytes = native_bytes(&input);
    assert!(
        store
            .roster_apply(
                &principal(),
                std::slice::from_ref(&input),
                RequestSource::Native(&bytes),
                deadline()
            )
            .is_err()
    );
    assert_eq!(ledger(&area), before);
    store.clock.test_ids.clear();
    let same_label = apply(&mut store, &input);
    assert_ne!(same_label.head.record_id, existing.head.record_id);
    assert_eq!(same_label.head.definition, existing.head.definition);
}

#[test]
fn import_row_digest_and_shared_source_object_bind_exact_domain_ordinal_and_bytes() {
    let area = Area::new();
    let mut store = area.open();
    let inputs = vec![create_request(1), create_request(2)];
    let bytes = manifest(&inputs);
    store
        .roster_apply(
            &principal(),
            &inputs,
            RequestSource::Import(&bytes),
            deadline(),
        )
        .unwrap();
    let db = area.inspect();
    let mut objects = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        let (object,ordinal,stored):(String,u32,String)=db.query_row("SELECT request_object,request_row,request_digest FROM operations WHERE request_key=?",[&input.idempotency_key],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
        let mut expected = b"HEE3-roster-import/1\0".to_vec();
        expected.extend_from_slice(&u32::try_from(index).unwrap().to_be_bytes());
        expected.extend_from_slice(&bytes);
        let mut expected_digest = String::from("sha256:");
        for byte in &Sha256::digest(&expected) {
            write!(expected_digest, "{byte:02x}").unwrap();
        }
        assert_eq!(stored, expected_digest);
        assert_eq!(ordinal, u32::try_from(index).unwrap());
        assert_eq!(area.object_bytes(&object), bytes);
        objects.push(object);
    }
    assert_eq!(objects[0], objects[1]);
}

#[test]
fn source_publication_failure_prevents_a_roster_operation_or_durable_revision() {
    let area = Area::new();
    let mut store = area.open();
    let before = ledger(&area);
    let input = create_request(1);
    let bytes = native_bytes(&input);
    store.fault = Some(CutPoint::ObjectSync);
    injected(
        store.roster_apply(
            &principal(),
            std::slice::from_ref(&input),
            RequestSource::Native(&bytes),
            deadline(),
        ),
        CutPoint::ObjectSync,
    );
    store.fault = None;
    assert_eq!(ledger(&area), before);
    assert_eq!(apply(&mut store, &input).head.record_version, "1");
}

fn accepted_task(store: &mut Store) {
    admit(store);
    store
        .begin_attempt(
            uuid(TASK),
            generation(1),
            uuid(ATTEMPT),
            uuid(STARTED),
            deadline(),
        )
        .unwrap();
    let expected = Expected {
        task: uuid(TASK),
        task_generation: generation(2),
        attempt: uuid(ATTEMPT),
        attempt_generation: generation(1),
    };
    store
        .settle_attempt(
            &expected,
            Settlement {
                effect: Effect::None,
                used_ms: Some(30),
                cleanup_settled: true,
                ready_to_verify: true,
            },
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    let expected = Expected {
        task_generation: generation(3),
        ..expected
    };
    let object = store
        .publish(
            b"T05 retained preexisting task proof",
            uuid(&id(900)),
            deadline(),
        )
        .unwrap();
    let proof = store
        .prepare_acceptance(&expected, uuid(ACCEPTED), &[object], deadline())
        .unwrap();
    store.accept(&proof, 20, deadline()).unwrap();
}

#[test]
fn roster_write_fault_preserves_existing_task_acceptance_reservations_and_outbox() {
    let area = Area::new();
    let mut store = area.open();
    accepted_task(&mut store);
    let existing = create(&mut store, 1);
    let before = ledger(&area);
    let mut update = change(&existing.head, 2);
    update.definition.version = "v2".to_owned();
    let bytes = native_bytes(&update);
    store.fault = Some(CutPoint::RosterWrite);
    injected(
        store.roster_apply(
            &principal(),
            std::slice::from_ref(&update),
            RequestSource::Native(&bytes),
            deadline(),
        ),
        CutPoint::RosterWrite,
    );
    store.fault = None;
    assert_eq!(ledger(&area), before);
    assert_eq!(
        store
            .get(&principal(), uuid(TASK), deadline())
            .unwrap()
            .accepted_event
            .as_deref(),
        Some(ACCEPTED)
    );
    assert_eq!(apply(&mut store, &update).head.record_version, "2");
    assert_eq!(counts(&area, &["acceptances", "outbox"]), vec![1, 1]);
}

#[test]
fn lost_committed_roster_reply_is_uncertain_and_recovers_once_after_reopen() {
    let area = Area::new();
    let mut store = area.open();
    let input = create_request(1);
    let bytes = native_bytes(&input);
    store.fault = Some(CutPoint::AfterCommit);
    assert!(matches!(
        store.roster_apply(
            &principal(),
            std::slice::from_ref(&input),
            RequestSource::Native(&bytes),
            deadline()
        ),
        Err(Error::UncertainCommit)
    ));
    drop(store);
    let mut reopened = area.reopen();
    let recovered = reopened
        .roster_by_key(
            &principal(),
            "roster.update",
            uuid(&input.idempotency_key),
            deadline(),
        )
        .unwrap();
    let before = ledger(&area);
    let replay = reopened
        .roster_apply(
            &principal(),
            &[input],
            RequestSource::Native(&bytes),
            deadline(),
        )
        .unwrap();
    assert_eq!(replay, vec![recovered]);
    assert_eq!(ledger(&area), before);
    assert_eq!(
        counts(
            &area,
            &["roster_records", "roster_revisions", "operations", "events"]
        ),
        vec![1, 1, 1, 1]
    );
}

#[test]
fn self_labeled_generic_observation_cannot_select_but_confirmed_worker_path_can() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    let input = observation_input(&profile.head);
    let generic = store
        .roster_observe(&principal(), &input, deadline())
        .unwrap();
    assert_eq!(generic.confirmed_source, None);
    let record = read(&store, &profile.head);
    let sample = store.roster_snapshot(&principal(), deadline()).unwrap().now;
    assert!(!choose(&record.head).permits(&record.head, record.observation.as_ref(), &sample));
    let confirmed = store
        .roster_observe_worker(&principal(), &input, deadline())
        .unwrap();
    assert_eq!(confirmed.confirmed_source, Some(ObservationSource::Worker));
    let record = read(&store, &profile.head);
    assert!(choose(&record.head).permits(&record.head, record.observation.as_ref(), &sample));
    assert_eq!(
        store
            .roster_observation(&principal(), uuid(&generic.id), deadline())
            .unwrap(),
        generic
    );
}

#[test]
fn same_millisecond_observations_have_durable_order_and_retained_immutable_bodies() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    let first = observe(&mut store, &profile.head);
    let mut input = observation_input(&profile.head);
    input.availability = Availability::Unavailable;
    let second = store
        .roster_observe_worker(&principal(), &input, deadline())
        .unwrap();
    assert_eq!(first.received, second.received);
    assert!(second.sequence > first.sequence);
    assert_ne!(first.id, second.id);
    assert_eq!(read(&store, &profile.head).observation, Some(second));
    assert_eq!(
        store
            .roster_observation(&principal(), uuid(&first.id), deadline())
            .unwrap(),
        first
    );
    assert_eq!(counts(&area, &["roster_observations"]), vec![2]);
}

#[test]
fn stored_observation_refuses_wrong_owner_revision_endpoint_or_instance_binding() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    let original = observation_input(&profile.head);
    let before = ledger(&area);
    let mut invalids = Vec::new();
    let mut wrong = original.clone();
    wrong.owner_id = "other-worker".to_owned();
    invalids.push(wrong);
    let mut wrong = original.clone();
    wrong.record_version = "2".to_owned();
    invalids.push(wrong);
    let mut wrong = original.clone();
    wrong.endpoint_ref = Some(id(201));
    invalids.push(wrong);
    let mut wrong = original.clone();
    wrong.instance_id = Some(id(202));
    wrong.instance_generation = Some("1".to_owned());
    invalids.push(wrong);
    for input in invalids {
        assert!(
            store
                .roster_observe_worker(&principal(), &input, deadline())
                .is_err()
        );
        assert_eq!(ledger(&area), before);
    }
    assert_eq!(
        store
            .roster_observe_worker(&principal(), &original, deadline())
            .unwrap()
            .input,
        original
    );
}

#[test]
fn observation_fault_rolls_back_head_history_and_event_together() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    let first = observe(&mut store, &profile.head);
    let before = ledger(&area);
    let mut input = observation_input(&profile.head);
    input.availability = Availability::Unknown;
    store.fault = Some(CutPoint::RosterObservation);
    injected(
        store.roster_observe_worker(&principal(), &input, deadline()),
        CutPoint::RosterObservation,
    );
    store.fault = None;
    assert_eq!(ledger(&area), before);
    assert_eq!(read(&store, &profile.head).observation, Some(first));
    assert_eq!(
        store
            .roster_observe_worker(&principal(), &input, deadline())
            .unwrap()
            .input
            .availability,
        Availability::Unknown
    );
}

#[test]
fn reopen_retains_observations_but_new_receiver_epoch_requires_fresh_proof() {
    let area = Area::new();
    let mut store = area.open();
    let profile = create(&mut store, 1);
    let observation = observe(&mut store, &profile.head);
    let before = store.roster_snapshot(&principal(), deadline()).unwrap();
    assert_eq!(before.now.epoch, observation.received.epoch);
    assert_ne!(before.now.epoch, LEDGER_EPOCH);
    drop(store);
    let mut reopened = area.reopen();
    let snapshot = reopened.roster_snapshot(&principal(), deadline()).unwrap();
    let record = read(&reopened, &profile.head);
    assert_ne!(snapshot.now.epoch, before.now.epoch);
    assert_eq!(record.observation, Some(observation));
    assert_eq!(
        freshness(
            &record.head,
            record.observation.as_ref(),
            &snapshot.now,
            60_000
        ),
        Freshness::EpochMismatch
    );
    let renewed = observe(&mut reopened, &profile.head);
    assert_eq!(renewed.received.epoch, snapshot.now.epoch);
    assert_eq!(counts(&area, &["roster_observations"]), vec![2]);
}

#[test]
fn captured_snapshot_does_not_change_when_admitted_definition_and_observation_change() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    let first = observe(&mut store, &profile.head);
    let captured = store.roster_snapshot(&principal(), deadline()).unwrap();
    let mut update = change(&profile.head, 2);
    update.definition.version = "v2".to_owned();
    let current = apply(&mut store, &update);
    clock(&mut store, 101);
    observe(&mut store, &current.head);
    assert_eq!(captured.records.len(), 1);
    assert_eq!(captured.records[0].head.definition.version, "v1");
    assert_eq!(captured.records[0].observation, Some(first));
    let latest = store.roster_snapshot(&principal(), deadline()).unwrap();
    assert_eq!(latest.records[0].head.definition.version, "v2");
    assert!(latest.cutoff > captured.cutoff);
}

#[test]
fn beginning_rostered_attempt_atomically_pins_profile_and_creates_distinct_instance() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    let observation = observe(&mut store, &profile.head);
    admit(&mut store);
    let running = begin(&mut store, &profile.head, &[choose(&profile.head)]).unwrap();
    assert_eq!(running.attempt.task_generation, "2");
    assert_eq!(running.attempt.generation, "1");
    assert_ne!(running.instance.id, profile.head.record_id);
    assert_eq!(running.instance.task_id, TASK);
    assert_eq!(running.instance.attempt_id, ATTEMPT);
    assert_eq!(running.instance.session_id, SESSION);
    assert_eq!(running.instance.workspace_ref, WORKSPACE);
    assert_eq!(running.instance.state, InstanceState::Starting);
    assert_eq!(running.instance.usage_ms, None);
    assert_eq!(running.pins.len(), 1);
    assert_eq!(running.pins[0].record.head.record_version, "1");
    assert_eq!(running.pins[0].record.observation, Some(observation));
    assert_eq!(
        counts(
            &area,
            &[
                "attempts",
                "roster_instances",
                "roster_instance_history",
                "roster_pins"
            ]
        ),
        vec![1, 1, 1, 1]
    );
}

#[test]
fn unavailable_required_proof_refuses_before_any_attempt_instance_or_pin_write() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    admit(&mut store);
    let before = ledger(&area);
    assert!(begin(&mut store, &profile.head, &[choose(&profile.head)]).is_err());
    assert_eq!(ledger(&area), before);
    let mut input = observation_input(&profile.head);
    input.capabilities.clear();
    store
        .roster_observe_worker(&principal(), &input, deadline())
        .unwrap();
    let before = ledger(&area);
    assert!(begin(&mut store, &profile.head, &[choose(&profile.head)]).is_err());
    assert_eq!(ledger(&area), before);
    observe(&mut store, &profile.head);
    assert!(begin(&mut store, &profile.head, &[choose(&profile.head)]).is_ok());
}

#[test]
fn pin_fault_rolls_back_task_attempt_instance_history_and_pins_as_one_transition() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    observe(&mut store, &profile.head);
    admit(&mut store);
    let before = ledger(&area);
    store.fault = Some(CutPoint::RosterPin);
    injected(
        begin(&mut store, &profile.head, &[choose(&profile.head)]),
        CutPoint::RosterPin,
    );
    store.fault = None;
    assert_eq!(ledger(&area), before);
    assert_eq!(
        store
            .get(&principal(), uuid(TASK), deadline())
            .unwrap()
            .generation,
        "1"
    );
    assert!(begin(&mut store, &profile.head, &[choose(&profile.head)]).is_ok());
}

#[test]
fn disable_committed_before_pin_prevents_new_work_even_with_fresh_prior_proof() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    observe(&mut store, &profile.head);
    admit(&mut store);
    let disabled = disable(
        &mut store,
        &profile.head,
        2,
        ActiveAttemptPolicy::LeaveRunning,
    );
    let before = ledger(&area);
    assert!(begin(&mut store, &disabled.head, &[choose(&disabled.head)]).is_err());
    assert_eq!(ledger(&area), before);
    assert!(read(&store, &profile.head).head.disabled);
    assert_eq!(
        counts(&area, &["attempts", "roster_instances", "roster_pins"]),
        vec![0, 0, 0]
    );
}

#[test]
fn leave_running_preserves_active_pins_without_cancel_obligations_or_task_intent() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    observe(&mut store, &profile.head);
    admit(&mut store);
    let running = begin(&mut store, &profile.head, &[choose(&profile.head)]).unwrap();
    let disabled = disable(
        &mut store,
        &profile.head,
        2,
        ActiveAttemptPolicy::LeaveRunning,
    );
    assert!(disabled.cancellation_causes.is_empty());
    assert_eq!(
        store
            .roster_pins(&principal(), uuid(ATTEMPT), deadline())
            .unwrap(),
        running.pins
    );
    let task = store.get(&principal(), uuid(TASK), deadline()).unwrap();
    assert!(!task.cancellation);
    assert_eq!(task.generation, "2");
    assert_eq!(task.state, "running");
    assert_eq!(counts(&area, &["roster_cancel_causes"]), vec![0]);
}

#[test]
fn two_disabled_selected_profiles_keep_two_causes_but_one_task_cancellation_transition() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let agent = create(&mut store, 1);
    let mut model_request = create_request(2);
    model_request.definition.kind = Kind::Model;
    let model = apply(&mut store, &model_request);
    observe(&mut store, &agent.head);
    observe(&mut store, &model.head);
    admit(&mut store);
    let running = begin(
        &mut store,
        &agent.head,
        &[choose(&agent.head), choose(&model.head)],
    )
    .unwrap();
    let first = disable(
        &mut store,
        &agent.head,
        3,
        ActiveAttemptPolicy::RequestCancel,
    );
    let after_first = store.get(&principal(), uuid(TASK), deadline()).unwrap();
    assert!(after_first.cancellation);
    assert_eq!(after_first.generation, "3");
    let second = disable(
        &mut store,
        &model.head,
        4,
        ActiveAttemptPolicy::RequestCancel,
    );
    let after_second = store.get(&principal(), uuid(TASK), deadline()).unwrap();
    assert_eq!(after_second.generation, after_first.generation);
    assert!(after_second.cancellation);
    for (result, record) in [(&first, &agent.head), (&second, &model.head)] {
        assert_eq!(result.cancellation_causes.len(), 1);
        let cause = &result.cancellation_causes[0];
        assert_eq!(cause.disable_event, result.event_id);
        assert_eq!(cause.record_id, record.record_id);
        assert_eq!(cause.task_id, TASK);
        assert_eq!(cause.attempt_id, ATTEMPT);
    }
    assert_ne!(first.event_id, second.event_id);
    assert_eq!(counts(&area, &["roster_cancel_causes"]), vec![2]);
    let before = ledger(&area);
    assert_eq!(
        disable(
            &mut store,
            &agent.head,
            3,
            ActiveAttemptPolicy::RequestCancel
        ),
        first
    );
    assert_eq!(
        disable(
            &mut store,
            &model.head,
            4,
            ActiveAttemptPolicy::RequestCancel
        ),
        second
    );
    assert_eq!(ledger(&area), before);
    let mut returned_pins = store
        .roster_pins(&principal(), uuid(ATTEMPT), deadline())
        .unwrap();
    let mut expected_pins = running.pins;
    returned_pins
        .sort_by(|left, right| left.record.head.record_id.cmp(&right.record.head.record_id));
    expected_pins
        .sort_by(|left, right| left.record.head.record_id.cmp(&right.record.head.record_id));
    assert_eq!(returned_pins, expected_pins);
    let state: (String, String) = area
        .inspect()
        .query_row(
            "SELECT effect,cleanup FROM attempts WHERE id=?",
            [ATTEMPT],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_ne!(state.1, "settled");
    assert_ne!(after_second.state, "cancelled");
}

/// A store error as a test failure: `store::Error` is not a `std::error::Error`.
fn debug(error: &Error) -> String {
    format!("{error:?}")
}

/// B05 deferred (b): the roster's cancellation is the task's one cancellation transition
/// (`request_cancellation`), not a second door. A task whose attempt's effect is unknown keeps
/// `effect_unknown` -- a cancellation never masks a liability -- while the flag, the generation and
/// the intent's event are still written, and the intent records why: an operator's disable.
#[test]
fn a_roster_cancellation_keeps_an_unknown_effect_and_records_its_reason()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let agent = create(&mut store, 1);
    observe(&mut store, &agent.head);
    admit(&mut store);
    begin(&mut store, &agent.head, &[choose(&agent.head)]).map_err(|error| debug(&error))?;
    store
        .settle_attempt(
            &Expected {
                task: uuid(TASK),
                task_generation: generation(2),
                attempt: uuid(ATTEMPT),
                attempt_generation: generation(1),
            },
            Settlement {
                effect: Effect::Unknown,
                used_ms: None,
                cleanup_settled: false,
                ready_to_verify: false,
            },
            uuid(SETTLED),
            deadline(),
        )
        .map_err(|error| debug(&error))?;
    let before = store
        .get(&principal(), uuid(TASK), deadline())
        .map_err(|error| debug(&error))?;
    assert_eq!(
        (
            before.state.as_str(),
            before.generation.as_str(),
            before.cancellation
        ),
        ("effect_unknown", "3", false)
    );
    let result = disable(
        &mut store,
        &agent.head,
        2,
        ActiveAttemptPolicy::RequestCancel,
    );
    assert_eq!(result.active_attempts, vec![ATTEMPT.to_owned()]);
    let after = store
        .get(&principal(), uuid(TASK), deadline())
        .map_err(|error| debug(&error))?;
    assert_eq!(
        (
            after.state.as_str(),
            after.generation.as_str(),
            after.cancellation
        ),
        ("effect_unknown", "4", true)
    );
    let db = area.inspect();
    let mut statement = db.prepare(
        "SELECT generation,body FROM events WHERE task_id=? AND kind='cancellation_requested'",
    )?;
    let intents = statement
        .query_map([TASK], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(intents.len(), 1, "one intent");
    assert_eq!(intents[0].0, "4");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&intents[0].1)?,
        json!({"reason": "operator_request", "note": null})
    );
    Ok(())
}

#[test]
fn disabling_one_profile_does_not_cancel_an_unrelated_owned_attempt() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let first = create(&mut store, 1);
    let other = create(&mut store, 2);
    observe(&mut store, &first.head);
    observe(&mut store, &other.head);
    admit(&mut store);
    begin(&mut store, &first.head, &[choose(&first.head)]).unwrap();
    let other_task = id(501);
    let other_attempt = id(502);
    let other_key = id(503);
    let other_event = id(504);
    let other_start = id(505);
    store
        .submit(
            Submission {
                principal: &principal(),
                key: uuid(&other_key),
                task: uuid(&other_task),
                event: uuid(&other_event),
                request_bytes: b"second independent task",
                workspace_id: uuid("28f00000-0000-4000-8000-00000000000a"),
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
    let selected = vec![choose(&other.head)];
    store
        .begin_rostered_attempt(
            RosterStart {
                principal: &principal(),
                task: uuid(&other_task),
                expected: generation(1),
                attempt: uuid(&other_attempt),
                event: uuid(&other_start),
                agent_record_id: &other.head.record_id,
                session: uuid(&id(506)),
                workspace: uuid(&id(507)),
                selections: &selected,
                lease_ms: 500,
            },
            deadline(),
        )
        .unwrap();
    let result = disable(
        &mut store,
        &first.head,
        3,
        ActiveAttemptPolicy::RequestCancel,
    );
    assert_eq!(result.cancellation_causes.len(), 1);
    assert_eq!(result.cancellation_causes[0].attempt_id, ATTEMPT);
    let unaffected = store
        .get(&principal(), uuid(&other_task), deadline())
        .unwrap();
    assert!(!unaffected.cancellation);
    assert_eq!(unaffected.generation, "2");
    assert_eq!(unaffected.state, "running");
}

#[test]
fn instance_observations_append_history_without_settling_task_effects_or_cleanup() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    observe(&mut store, &profile.head);
    admit(&mut store);
    let running = begin(&mut store, &profile.head, &[choose(&profile.head)]).unwrap();
    let instance_id = uuid(&running.instance.id);
    let busy = store
        .roster_change_instance(
            &principal(),
            instance_id,
            generation(1),
            InstanceState::Busy,
            Some(5),
            deadline(),
        )
        .unwrap();
    assert_eq!(busy.revision, "2");
    assert_eq!(busy.generation, running.instance.generation);
    assert_eq!(busy.usage_ms, Some(5));
    assert!(
        store
            .roster_change_instance(
                &principal(),
                instance_id,
                generation(1),
                InstanceState::Exited,
                Some(7),
                deadline()
            )
            .is_err()
    );
    let exited = store
        .roster_change_instance(
            &principal(),
            instance_id,
            generation(2),
            InstanceState::Exited,
            Some(7),
            deadline(),
        )
        .unwrap();
    assert_eq!(exited.revision, "3");
    assert_eq!(
        store
            .roster_instance(&principal(), instance_id, deadline())
            .unwrap(),
        exited
    );
    let instances = store
        .roster_instances(&principal(), uuid(&profile.head.record_id), deadline())
        .unwrap();
    assert_eq!(instances, vec![exited]);
    let task = store.get(&principal(), uuid(TASK), deadline()).unwrap();
    assert_eq!(task.state, "running");
    assert_eq!(task.spent_ms, 0);
    assert_eq!(task.reserved_work_ms, 800);
    let retained: Vec<u8> = area
        .inspect()
        .query_row(
            "SELECT body FROM roster_instance_history WHERE instance_id=? AND revision='1'",
            [&running.instance.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<crate::contracts::roster::Instance>(&retained).unwrap(),
        running.instance
    );
}

#[test]
fn old_instance_report_binds_historical_pin_without_replacing_new_profile_proof() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    observe(&mut store, &profile.head);
    admit(&mut store);
    let running = begin(&mut store, &profile.head, &[choose(&profile.head)]).unwrap();
    let mut update = change(&profile.head, 2);
    update.definition.version = "v2".to_owned();
    let current = apply(&mut store, &update);
    clock(&mut store, 101);
    let generic = observe(&mut store, &current.head);
    let mut historical = observation_input(&profile.head);
    historical.instance_id = Some(running.instance.id.clone());
    historical.instance_generation = Some(running.instance.generation.clone());
    let stored = store
        .roster_observe_worker(&principal(), &historical, deadline())
        .unwrap();
    assert_eq!(stored.input.record_version, "1");
    assert_eq!(read(&store, &current.head).observation, Some(generic));
    assert_eq!(
        store
            .roster_observation(&principal(), uuid(&stored.id), deadline())
            .unwrap(),
        stored
    );
    assert_eq!(
        store
            .roster_pins(&principal(), uuid(ATTEMPT), deadline())
            .unwrap(),
        running.pins
    );
    let before = ledger(&area);
    historical.instance_generation = Some("2".to_owned());
    assert!(
        store
            .roster_observe_worker(&principal(), &historical, deadline())
            .is_err()
    );
    assert_eq!(ledger(&area), before);
}

#[test]
fn every_roster_read_and_mutation_honors_an_already_expired_caller_deadline() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    let observation = observe(&mut store, &profile.head);
    let before = ledger(&area);
    let expired = Instant::now()
        .checked_sub(Duration::from_millis(1))
        .unwrap();
    assert!(matches!(
        store.roster_get(&principal(), uuid(&profile.head.record_id), expired),
        Err(Error::Deadline)
    ));
    assert!(matches!(
        store.roster_by_key(&principal(), "roster.update", uuid(&id(1)), expired),
        Err(Error::Deadline)
    ));
    assert!(matches!(
        store.roster_snapshot(&principal(), expired),
        Err(Error::Deadline)
    ));
    assert!(matches!(
        store.roster_observation(&principal(), uuid(&observation.id), expired),
        Err(Error::Deadline)
    ));
    assert!(matches!(
        store.roster_instances(&principal(), uuid(&profile.head.record_id), expired),
        Err(Error::Deadline)
    ));
    let update = change(&profile.head, 2);
    let bytes = native_bytes(&update);
    assert!(matches!(
        store.roster_apply(
            &principal(),
            &[update],
            RequestSource::Native(&bytes),
            expired
        ),
        Err(Error::Deadline)
    ));
    assert!(matches!(
        store.roster_observe_worker(&principal(), &observation_input(&profile.head), expired),
        Err(Error::Deadline)
    ));
    let input = disable_input(&profile.head, 3, ActiveAttemptPolicy::LeaveRunning);
    assert!(matches!(
        store.roster_disable(&principal(), &input, b"expired trusted disable", expired),
        Err(Error::Deadline)
    ));
    assert_eq!(ledger(&area), before);
}

#[test]
fn update_reopen_and_stale_reimport_all_preserve_disabled_state() {
    let area = Area::new();
    let mut store = area.open();
    let original = create_request(1);
    let bytes = manifest(std::slice::from_ref(&original));
    let decoded = crate::roster::parse_changes(&bytes).unwrap();
    assert_eq!(decoded.as_slice(), std::slice::from_ref(&original));
    let profile = store
        .roster_apply(
            &principal(),
            &decoded,
            RequestSource::Import(&bytes),
            deadline(),
        )
        .unwrap()
        .remove(0);
    let disabled = disable(
        &mut store,
        &profile.head,
        2,
        ActiveAttemptPolicy::LeaveRunning,
    );
    let mut update = change(&disabled.head, 3);
    update.definition.display_name = "still disabled".to_owned();
    let latest = apply(&mut store, &update);
    assert!(latest.head.disabled);
    drop(store);
    let mut reopened = area.reopen();
    assert_eq!(read(&reopened, &latest.head).head, latest.head);
    let replay_rows = crate::roster::parse_changes(&bytes).unwrap();
    assert_eq!(replay_rows, vec![original]);
    let replay = reopened
        .roster_apply(
            &principal(),
            &replay_rows,
            RequestSource::Import(&bytes),
            deadline(),
        )
        .unwrap();
    assert_eq!(replay, vec![profile.clone()]);
    assert!(read(&reopened, &latest.head).head.disabled);
    let stale = change(&profile.head, 4);
    let stale_bytes = manifest(std::slice::from_ref(&stale));
    let stale_rows = crate::roster::parse_changes(&stale_bytes).unwrap();
    assert_eq!(stale_rows, vec![stale]);
    let before = ledger(&area);
    assert!(matches!(
        reopened.roster_apply(
            &principal(),
            &stale_rows,
            RequestSource::Import(&stale_bytes),
            deadline()
        ),
        Err(Error::Conflict)
    ));
    assert_eq!(ledger(&area), before);
}

#[test]
fn consistent_development_backup_retains_roster_history_disabled_state_and_pins() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    let proof = observe(&mut store, &profile.head);
    admit(&mut store);
    let running = begin(&mut store, &profile.head, &[choose(&profile.head)]).unwrap();
    disable(
        &mut store,
        &profile.head,
        2,
        ActiveAttemptPolicy::LeaveRunning,
    );
    store
        .settle_attempt(
            &Expected {
                task: uuid(TASK),
                task_generation: generation(2),
                attempt: uuid(ATTEMPT),
                attempt_generation: generation(1),
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(10),
                cleanup_settled: true,
                ready_to_verify: false,
            },
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    let destination_area = Area::new();
    let destination = destination_area.path.clone();
    let backup = store.backup(&destination, deadline()).unwrap();
    assert_eq!(backup.operational_status, RestoreStatus::Unqualified);
    assert!(!backup.finish_return_observed);
    let db = Connection::open_with_flags(
        destination.join("ledger.sqlite3"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    db.execute_batch("PRAGMA query_only=ON;").unwrap();
    let disabled: i64 = db
        .query_row(
            "SELECT disabled FROM roster_records WHERE id=?",
            [&profile.head.record_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(disabled, 1);
    let pins: Vec<u8> = db
        .query_row(
            "SELECT body FROM roster_pins WHERE attempt_id=?",
            [ATTEMPT],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<crate::contracts::roster::Pin>(&pins).unwrap(),
        running.pins[0]
    );
    let observed: Vec<u8> = db
        .query_row(
            "SELECT body FROM roster_observations WHERE id=?",
            [&proof.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<Observation>(&observed).unwrap(),
        proof
    );
    let revisions: i64 = db
        .query_row(
            "SELECT count(*) FROM roster_revisions WHERE record_id=?",
            [&profile.head.record_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(revisions, 2);
    db.close().unwrap();
}

#[test]
fn request_cancel_includes_current_verifying_attempt_after_worker_settlement_only() {
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    observe(&mut store, &profile.head);
    admit(&mut store);
    begin(&mut store, &profile.head, &[choose(&profile.head)]).unwrap();
    store
        .settle_attempt(
            &Expected {
                task: uuid(TASK),
                task_generation: generation(2),
                attempt: uuid(ATTEMPT),
                attempt_generation: generation(1),
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(1),
                cleanup_settled: true,
                ready_to_verify: false,
            },
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    let current_attempt = id(601);
    let current_event = id(602);
    let selected = vec![choose(&profile.head)];
    store
        .begin_rostered_attempt(
            RosterStart {
                principal: &principal(),
                task: uuid(TASK),
                expected: generation(3),
                attempt: uuid(&current_attempt),
                event: uuid(&current_event),
                agent_record_id: &profile.head.record_id,
                session: uuid(&id(603)),
                workspace: uuid(&id(604)),
                selections: &selected,
                lease_ms: 500,
            },
            deadline(),
        )
        .unwrap();
    store
        .settle_attempt(
            &Expected {
                task: uuid(TASK),
                task_generation: generation(4),
                attempt: uuid(&current_attempt),
                attempt_generation: generation(2),
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(1),
                cleanup_settled: true,
                ready_to_verify: true,
            },
            uuid(&id(605)),
            deadline(),
        )
        .unwrap();
    assert_eq!(
        store
            .get(&principal(), uuid(TASK), deadline())
            .unwrap()
            .state,
        "verifying"
    );
    let result = disable(
        &mut store,
        &profile.head,
        2,
        ActiveAttemptPolicy::RequestCancel,
    );
    assert_eq!(result.active_attempts, vec![current_attempt.clone()]);
    assert_eq!(result.cancellation_causes.len(), 1);
    assert_eq!(result.cancellation_causes[0].attempt_id, current_attempt);
    assert_ne!(result.cancellation_causes[0].attempt_id, ATTEMPT);
    let task = store.get(&principal(), uuid(TASK), deadline()).unwrap();
    assert!(task.cancellation);
    assert_eq!(task.generation, "6");
    assert_ne!(task.state, "accepted");
}

#[test]
fn store_secondary_fresh_roster_schema_failure_rolls_back_schema_history_and_version() {
    let area = Area::new();
    injected(
        Store::open_inner(
            &area.path,
            uuid(GEN),
            uuid(LEDGER_EPOCH),
            true,
            deadline(),
            Some(CutPoint::MigrationWrite),
        ),
        CutPoint::MigrationWrite,
    );
    let db = area.inspect();
    let tables: i64 = db
        .query_row("SELECT count(*) FROM sqlite_schema", [], |row| row.get(0))
        .unwrap();
    let version: i64 = db
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!((tables, version), (0, 0));
    db.close().unwrap();
    let neighbor = Area::new();
    let store = neighbor.open();
    assert_eq!(
        store
            .roster_snapshot(&principal(), deadline())
            .unwrap()
            .records
            .len(),
        0
    );
}

#[test]
fn store_secondary_prior_draft_is_refused_without_rebasing_or_erasing_it() {
    let area = Area::new();
    let generations = area.path.join("generations");
    DirBuilder::new().mode(0o700).create(&generations).unwrap();
    DirBuilder::new()
        .mode(0o700)
        .create(generations.join(GEN))
        .unwrap();
    let db = Connection::open(area.database()).unwrap();
    db.execute_batch(PRE_ROSTER_SCHEMA).unwrap();
    db.execute(
        "INSERT INTO migration_history VALUES(1,?,0,NULL,?)",
        params![PRE_ROSTER_SHA256, PRE_ROSTER_PACKAGE],
    )
    .unwrap();
    db.execute(
        "INSERT INTO ledger_meta VALUES(1,?,?,'normal')",
        params![LEDGER_EPOCH, GEN],
    )
    .unwrap();
    db.pragma_update(None, "application_id", 0x4845_4533_i64)
        .unwrap();
    db.pragma_update(None, "user_version", 1).unwrap();
    db.close().unwrap();
    fs::set_permissions(area.database(), fs::Permissions::from_mode(0o600)).unwrap();
    let before = fs::read(area.database()).unwrap();
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(LEDGER_EPOCH), false, deadline()),
        Err(Error::Chain(Chain::Checksum { version: 1 }))
    ));
    assert_eq!(fs::read(area.database()).unwrap(), before);
    let db = area.inspect();
    let checksum: String = db
        .query_row("SELECT checksum FROM migration_history", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(checksum, PRE_ROSTER_SHA256);
    let roster: i64 = db
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name='roster_records'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(roster, 0);
    db.close().unwrap();
}

#[test]
fn store_secondary_finite_targets_retain_task_and_roster_foreign_key_constraints() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    let profile = create(&mut store, 1);
    let db = &store.connection;
    for (task, roster) in [
        (None, None),
        (Some(TASK), Some(profile.head.record_id.as_str())),
        (Some("absent-task"), None),
        (None, Some("absent-roster")),
    ] {
        assert!(
            db.execute(
                "INSERT INTO events(id,task_id,roster_id,generation,kind,body) VALUES(?,?,?,?,?,?)",
                params![id(700), task, roster, "1", "fixture", b"{}".as_slice()]
            )
            .is_err()
        );
    }
    db.execute(
        "INSERT INTO events(id,task_id,roster_id,generation,kind,body) VALUES(?,?,?,?,?,?)",
        params![
            id(701),
            Some(TASK),
            None::<&str>,
            "1",
            "fixture",
            b"{}".as_slice()
        ],
    )
    .unwrap();
    db.execute(
        "INSERT INTO events(id,task_id,roster_id,generation,kind,body) VALUES(?,?,?,?,?,?)",
        params![
            id(702),
            None::<&str>,
            Some(profile.head.record_id.as_str()),
            "1",
            "fixture",
            b"{}".as_slice()
        ],
    )
    .unwrap();
    assert!(
        db.execute(
            "UPDATE operations SET resource_id=? WHERE request_key=?",
            params![TASK, id(1)]
        )
        .is_err()
    );
    assert!(
        db.execute(
            "UPDATE operations SET roster_id=? WHERE request_key=?",
            params![profile.head.record_id, TASK_KEY]
        )
        .is_err()
    );
    assert!(
        db.execute(
            "UPDATE operations SET roster_id='absent-roster' WHERE request_key=?",
            [id(1)]
        )
        .is_err()
    );
    let task_target: (Option<String>, Option<String>) = db
        .query_row(
            "SELECT resource_id,roster_id FROM operations WHERE request_key=?",
            [TASK_KEY],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let roster_target: (Option<String>, Option<String>) = db
        .query_row(
            "SELECT resource_id,roster_id FROM operations WHERE request_key=?",
            [id(1)],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(task_target, (Some(TASK.to_owned()), None));
    assert_eq!(roster_target, (None, Some(profile.head.record_id)));
}

// Exact authored SQL from the retained prior development schema; no live upgrade claim.
const PRE_ROSTER_SCHEMA: &str = r"

-- Draft migration 1. Freeze exact complete bytes before first release.
CREATE TABLE migration_history (
    version INTEGER PRIMARY KEY CHECK(version > 0),
    checksum TEXT NOT NULL CHECK(length(checksum)=71),
    predecessor_version INTEGER NOT NULL,
    predecessor_checksum TEXT,
    package_identity TEXT NOT NULL
) STRICT;
CREATE TABLE ledger_meta (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    epoch TEXT NOT NULL,
    generation TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('normal','reconciliation'))
) STRICT;
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    principal_uid INTEGER NOT NULL CHECK(principal_uid BETWEEN 0 AND 4294967295),
    principal_role TEXT NOT NULL,
    spec BLOB NOT NULL CHECK(length(spec) BETWEEN 1 AND 1048576),
    criteria_digest TEXT NOT NULL,
    generation TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('admitted','queued','running','verifying','repair_pending','cancellation_requested','blocked','accepted','failed','cancelled','abandoned','effect_unknown')),
    cancellation INTEGER NOT NULL DEFAULT 0 CHECK(cancellation IN (0,1)),
    accepted_event TEXT REFERENCES events(id),
    limit_ms INTEGER NOT NULL CHECK(limit_ms BETWEEN 1 AND 1200000),
    spent_ms INTEGER NOT NULL DEFAULT 0 CHECK(spent_ms>=0),
    reserved_work_ms INTEGER NOT NULL CHECK(reserved_work_ms>=0),
    reserved_verify_ms INTEGER NOT NULL CHECK(reserved_verify_ms>=0),
    CHECK(spent_ms+reserved_work_ms+reserved_verify_ms<=limit_ms)
) STRICT;
CREATE TABLE operations (
    principal_uid INTEGER NOT NULL,
    principal_role TEXT NOT NULL,
    action TEXT NOT NULL,
    version INTEGER NOT NULL CHECK(version=1),
    request_key TEXT NOT NULL,
    request_digest TEXT NOT NULL,
    resource_id TEXT NOT NULL REFERENCES tasks(id),
    result BLOB NOT NULL,
    PRIMARY KEY(principal_uid,principal_role,action,version,request_key)
) STRICT;
CREATE TABLE attempts (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    generation TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('queued','running','settled','unknown')),
    effect TEXT NOT NULL CHECK(effect IN ('none','committed','pending','unknown')),
    cleanup TEXT NOT NULL CHECK(cleanup IN ('none','pending','settled','unknown')),
    used_ms INTEGER CHECK(used_ms>=0),
    UNIQUE(task_id,generation)
) STRICT;
CREATE UNIQUE INDEX one_unsettled_attempt ON attempts(task_id) WHERE state!='settled';
CREATE TABLE events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    generation TEXT NOT NULL,
    kind TEXT NOT NULL,
    body BLOB NOT NULL
) STRICT;
CREATE TABLE artifacts (
    digest TEXT PRIMARY KEY,
    size INTEGER NOT NULL CHECK(size BETWEEN 0 AND 16777216)
) STRICT;
CREATE TABLE acceptances (
    event_id TEXT PRIMARY KEY REFERENCES events(id),
    task_id TEXT NOT NULL UNIQUE REFERENCES tasks(id),
    attempt_id TEXT NOT NULL REFERENCES attempts(id),
    generation TEXT NOT NULL,
    criteria_digest TEXT NOT NULL,
    manifest_digest TEXT NOT NULL REFERENCES artifacts(digest)
) STRICT;
CREATE TABLE acceptance_objects (
    event_id TEXT NOT NULL REFERENCES acceptances(event_id),
    digest TEXT NOT NULL REFERENCES artifacts(digest),
    PRIMARY KEY(event_id,digest)
) STRICT;
CREATE TABLE outbox (
    event_id TEXT NOT NULL REFERENCES events(id),
    recipient TEXT NOT NULL,
    delivered INTEGER NOT NULL DEFAULT 0 CHECK(delivered IN (0,1)),
    PRIMARY KEY(event_id,recipient)
) STRICT;
";
const PRE_ROSTER_SHA256: &str =
    "sha256:14c3efd534517403866229f898748ab0685bdad58e0d1da23b6aa6e96140c044";
const PRE_ROSTER_PACKAGE: &str = "hee3-draft-schema1/0.1.0";

/// The digests and ids the binding tests use (B14a-1a).
const BIND_BASE: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const BIND_PROT: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const BIND_PROF: &str = "sha256:3333333333333333333333333333333333333333333333333333333333333333";
const BIND_OTHER: &str = "00000000-0000-4000-8000-0000000000b2";
const BIND_RESTARTED: &str = "00000000-0000-4000-8000-0000000000b3";
const BIND_CANCELLED: &str = "00000000-0000-4000-8000-0000000000b4";
fn bound_start<'a>(
    owner: &'a Principal,
    agent: &'a RosterHeadV1,
    selections: &'a [Selection],
) -> RosterStart<'a> {
    RosterStart {
        principal: owner,
        task: uuid(TASK),
        expected: generation(1),
        attempt: uuid(ATTEMPT),
        event: uuid(STARTED),
        agent_record_id: &agent.record_id,
        session: uuid(SESSION),
        workspace: uuid(WORKSPACE),
        selections,
        lease_ms: 500,
    }
}

fn binding() -> Binding<'static> {
    Binding {
        baseline: Sha256Digest::parse(BIND_BASE).unwrap(),
        protected: Sha256Digest::parse(BIND_PROT).unwrap(),
        profile: Sha256Digest::parse(BIND_PROF).unwrap(),
    }
}

/// B14a-1a · a bound begin records the attempt's baseline, protected and profile digests in the
/// same transaction, and a bound task refuses an unbound begin `Conflict` — from a state where the
/// same begin, bound, is admitted — writing nothing.
#[test]
fn a_bound_task_refuses_an_unbound_attempt() {
    let owner = principal();
    // Bound first: the row holds exactly the three digests, and an unbound begin is refused.
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    observe(&mut store, &profile.head);
    admit(&mut store);
    let selections = [choose(&profile.head)];
    store
        .begin_bound_attempt(
            bound_start(&owner, &profile.head, &selections),
            &binding(),
            deadline(),
        )
        .unwrap();
    let row: (String, String, String, String, String) = area
        .inspect()
        .query_row(
            "SELECT attempt_id,task_id,baseline_digest,protected_digest,profile_digest FROM attempt_bindings",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .unwrap();
    assert_eq!(
        row,
        (
            ATTEMPT.into(),
            TASK.into(),
            BIND_BASE.into(),
            BIND_PROT.into(),
            BIND_PROF.into()
        )
    );
    // Settle the first attempt to `repair_pending`, so every other begin predicate passes and only
    // the mixing rule can refuse (F140: Conflict is shared by many sites).
    repair_pending(&mut store);
    let before = ledger(&area);
    assert!(matches!(
        store.begin_attempt(
            uuid(TASK),
            generation(3),
            uuid(BIND_OTHER),
            uuid(STARTED),
            deadline()
        ),
        Err(Error::Conflict)
    ));
    assert_eq!(ledger(&area), before, "a refused begin writes nothing");
    let mut again = bound_start(&owner, &profile.head, &selections);
    again.expected = generation(3);
    again.attempt = uuid(BIND_OTHER);
    again.event = uuid(BIND_RESTARTED);
    assert!(
        store
            .begin_bound_attempt(again, &binding(), deadline())
            .is_ok(),
        "the same begin, bound, is admitted from that state"
    );
}

/// B14a-1a · a task with an unbound attempt refuses a bound begin `Conflict` — from a state where
/// the same begin, unbound, is admitted — writing nothing.
#[test]
fn an_unbound_task_refuses_a_bound_attempt() {
    let owner = principal();
    // Unbound first: a bound begin is refused.
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    observe(&mut store, &profile.head);
    admit(&mut store);
    let selections = [choose(&profile.head)];
    store
        .begin_rostered_attempt(bound_start(&owner, &profile.head, &selections), deadline())
        .unwrap();
    repair_pending(&mut store);
    let before = ledger(&area);
    let mut second = bound_start(&owner, &profile.head, &selections);
    second.expected = generation(3);
    second.attempt = uuid(BIND_OTHER);
    assert!(matches!(
        store.begin_bound_attempt(second, &binding(), deadline()),
        Err(Error::Conflict)
    ));
    assert_eq!(
        ledger(&area),
        before,
        "a refused bound begin writes nothing"
    );
    let mut again = bound_start(&owner, &profile.head, &selections);
    again.expected = generation(3);
    again.attempt = uuid(BIND_OTHER);
    again.event = uuid(BIND_RESTARTED);
    assert!(
        store.begin_rostered_attempt(again, deadline()).is_ok(),
        "the same begin, unbound, is admitted from that state"
    );
}

/// B14a-1a · a cancelled task is named `Cancelled` at the wrong begin door too: cancellation is
/// checked before the mixing rule, which would otherwise answer `Conflict` (review F3).
#[test]
fn a_cancelled_task_meets_either_begin_door_as_cancelled() {
    let owner = principal();
    let area = Area::new();
    let mut store = area.open();
    clock(&mut store, 100);
    let profile = create(&mut store, 1);
    observe(&mut store, &profile.head);
    admit(&mut store);
    let selections = [choose(&profile.head)];
    store
        .begin_rostered_attempt(bound_start(&owner, &profile.head, &selections), deadline())
        .unwrap();
    repair_pending(&mut store);
    store
        .cancel(uuid(TASK), generation(3), uuid(BIND_CANCELLED), deadline())
        .unwrap();
    let before = ledger(&area);
    let mut second = bound_start(&owner, &profile.head, &selections);
    second.expected = generation(4);
    second.attempt = uuid(BIND_OTHER);
    second.event = uuid(BIND_RESTARTED);
    assert!(matches!(
        store.begin_bound_attempt(second, &binding(), deadline()),
        Err(Error::Cancelled)
    ));
    assert_eq!(ledger(&area), before, "a refused begin writes nothing");
}

/// Settle the fixture's first attempt as not ready to verify: the task returns to
/// `repair_pending` at generation 3, where a second attempt may begin.
fn repair_pending(store: &mut Store) {
    store
        .settle_attempt(
            &Expected {
                task: uuid(TASK),
                task_generation: generation(2),
                attempt: uuid(ATTEMPT),
                attempt_generation: generation(1),
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(30),
                cleanup_settled: true,
                ready_to_verify: false,
            },
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
}
