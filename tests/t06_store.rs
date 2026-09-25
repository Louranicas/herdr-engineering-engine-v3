//! Independent T06 Store verification oracles, frozen before test bodies.
//!
//! Authority: original `PLAN_habitat_engine.json` task T06; revision-4 RC01/RC04;
//! parent's public Store verification API/semantics clarification. Existing T04
//! public Store APIs supply fixture setup only. The new verification implementation
//! was not read. These are trusted-owner persistence tests, not actual checker,
//! protected collector, provider, process cleanup or module-admission evidence.
//!
//! Predeclared seams: worker end is insufficient; pass is not acceptance; failed
//! checks permit repair; invalid/error/timeout stop; cancelled needs prior intent;
//! intent races acceptance in both orders; unknown measurement and unclean custody
//! retain the entire verification reservation; zero and exact-reserve costs differ
//! from unknown; running/unsettled workers refuse; task/attempt CAS and single-return
//! identity cannot be bypassed; repair uses its own checker return; exhausted reserve
//! blocks further work; subject and exact evidence must match; omitted/missing/corrupt
//! evidence refuses; event conflicts roll back; durable outcomes survive reopen;
//! an expired caller deadline cannot write; outbox and acceptance commit together.
//!
//! Fixed arithmetic: initial work800/verify200/limit1000; settled worker30 leaves
//! spent30/work770/verify200; settled checker20 leaves spent50/verify180. Unknown
//! cost OR unknown cleanup leaves `spent30/verify200/effect_unknown`. A known20 with
//! unclean custody is a retained lower-bound observation, not new free capacity.
//! Verified accept costs0 and releases both remaining reservations; a nonzero cost
//! refuses to double-charge. The public API exposes no verification-row getter, so
//! lower-bound row contents/event-body fidelity are not asserted from `TaskHead` alone.
//!
//! Each test uses an independently owned private directory. Database ownership is
//! closed before cleanup/reopen. No test changes shared source, runs a provider or
//! asserts real checker execution. Parent owns compilation and actual test runs.

use habitat_engine::contracts::{Generation, Sha256Digest, UuidV4};
use habitat_engine::store::{
    Allocation, Effect, Expected, Object, Principal, PublishedAcceptance, Settlement, Store,
    Submission, TaskHead, Verification, VerificationVerdict,
};
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const GEN: &str = "06000000-0000-4000-8000-000000000001";
const EPOCH: &str = "06000000-0000-4000-8000-000000000002";
const TASK: &str = "06000000-0000-4000-8000-000000000003";
const KEY: &str = "06000000-0000-4000-8000-000000000004";
const ADMITTED: &str = "06000000-0000-4000-8000-000000000005";
const ATTEMPT: &str = "06000000-0000-4000-8000-000000000006";
const STARTED: &str = "06000000-0000-4000-8000-000000000007";
const SETTLED: &str = "06000000-0000-4000-8000-000000000008";
const VERIFIED: &str = "06000000-0000-4000-8000-000000000009";
const ACCEPTED: &str = "06000000-0000-4000-8000-00000000000a";
const CANCELLED: &str = "06000000-0000-4000-8000-00000000000b";
const STAGE: &str = "06000000-0000-4000-8000-00000000000c";
const OTHER: &str = "06000000-0000-4000-8000-00000000000d";
const SECOND_ATTEMPT: &str = "06000000-0000-4000-8000-00000000000e";
const SECOND_START: &str = "06000000-0000-4000-8000-00000000000f";
const SECOND_SETTLE: &str = "06000000-0000-4000-8000-000000000010";
const SECOND_VERIFY: &str = "06000000-0000-4000-8000-000000000011";
const CRITERIA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
// Independently known SHA-256 of the literal UTF-8 candidate bytes "hello".
const SUBJECT: &str = "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
const EVIDENCE_BYTES: &[u8] = b"fixture checker return: criterion C1 satisfied\n";
static NEXT_AREA: AtomicU64 = AtomicU64::new(0);

// Root-authored T06 integration regressions for the larger verified RC04
// inventory. These are separate from the original independently frozen oracles.
fn inventory(store: &Store, evidence: &Object, count: u32) -> Vec<Object> {
    let mut objects = vec![evidence.clone()];
    for index in 1..count {
        let publication = format!("06000000-1000-4000-8000-{index:012x}");
        objects.push(
            store
                .publish(&index.to_be_bytes(), id(&publication), deadline())
                .unwrap(),
        );
    }
    objects
}

fn inventory_proof(
    store: &Store,
    evidence: &Object,
    objects: &[Object],
) -> Result<PublishedAcceptance, habitat_engine::store::Error> {
    store.prepare_verified_acceptance(
        &expected("4"),
        id(ACCEPTED),
        Sha256Digest::parse(SUBJECT).unwrap(),
        evidence,
        objects,
        deadline(),
    )
}

fn register_unrelated_evidence(store: &mut Store) {
    let ids: Vec<_> = (0..8_u8)
        .map(|index| format!("06000000-3000-4000-8000-{index:012x}"))
        .collect();
    store
        .submit(
            Submission {
                principal: &principal(),
                key: id(&ids[0]),
                task: id(&ids[1]),
                event: id(&ids[2]),
                request_bytes: b"unrelated task consumes one registered artifact slot",
                workspace_id: id("28f00000-0000-4000-8000-00000000000a"),
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
    store
        .begin_attempt(
            id(&ids[1]),
            generation("1"),
            id(&ids[3]),
            id(&ids[4]),
            deadline(),
        )
        .unwrap();
    let expected = Expected {
        task: id(&ids[1]),
        task_generation: generation("2"),
        attempt: id(&ids[3]),
        attempt_generation: generation("1"),
    };
    store
        .settle_attempt(
            &expected,
            Settlement {
                effect: Effect::None,
                used_ms: Some(0),
                cleanup_settled: true,
                ready_to_verify: true,
            },
            id(&ids[5]),
            deadline(),
        )
        .unwrap();
    let evidence = store
        .publish(
            b"unrelated retained verifier evidence",
            id(&ids[6]),
            deadline(),
        )
        .unwrap();
    store
        .record_verification(
            &Expected {
                task_generation: generation("3"),
                ..expected
            },
            &observation(&evidence, VerificationVerdict::Failed, Some(0), true),
            id(&ids[7]),
            deadline(),
        )
        .unwrap();
}

#[test]
fn verified_inventory_exact_snapshot_capacity_accepts_and_backs_up_the_manifest() {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let objects = inventory(&store, &evidence, 4095);
    let published = inventory_proof(&store, &evidence, &objects).unwrap();
    store.accept(&published, 0, deadline()).unwrap();
    let backup_area = Area::new();
    let backup = store
        .backup(&backup_area.path, Instant::now() + Duration::from_secs(90))
        .unwrap();
    assert_eq!(backup.objects.len(), 4096);
    assert_eq!(backup.counts["artifacts"], 4096);
    assert!(backup.objects.contains(published.object()));
    assert!(objects.iter().all(|object| backup.objects.contains(object)));
    let bytes = fs::read(backup_area.path.join("store-backup.json")).unwrap();
    let mut digest = String::from("sha256:");
    for byte in Sha256::digest(&bytes) {
        write!(digest, "{byte:02x}").unwrap();
    }
    let inspected = Store::inspect_backup(
        &backup_area.path,
        Sha256Digest::parse(&digest).unwrap(),
        Instant::now() + Duration::from_secs(90),
    )
    .unwrap();
    assert_eq!(backup, inspected);
}

#[test]
fn verified_inventory_capacity_counts_intervening_registrations_in_accept_and_prepare() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let mut objects = inventory(&store, &evidence, 4095);
    let published = inventory_proof(&store, &evidence, &objects).unwrap();
    let before = head(&store);
    register_unrelated_evidence(&mut store);
    assert_eq!(head(&store), before);
    assert!(matches!(
        store.accept(&published, 0, deadline()),
        Err(habitat_engine::store::Error::Bound)
    ));
    assert_eq!(head(&store), before);
    no_delivery(&store);
    // Failed acceptance leaves its already durable manifest as a retained orphan.
    assert!(
        !store
            .read_object(published.object(), deadline())
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        inventory_proof(&store, &evidence, &objects),
        Err(habitat_engine::store::Error::Bound)
    ));
    assert_eq!(head(&store), before);
    no_delivery(&store);
    objects.pop();
    // One fewer referenced child leaves room for both unrelated evidence and manifest.
    assert!(inventory_proof(&store, &evidence, &objects).is_ok());
}

#[test]
fn verified_inventory_beyond_sixty_four_commits_every_object_and_survives_reopen() {
    let (area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let objects = inventory(&store, &evidence, 65);
    let published = inventory_proof(&store, &evidence, &objects).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_slice(&store.read_object(published.object(), deadline()).unwrap())
            .unwrap();
    assert_eq!(manifest["objects"].as_array().unwrap().len(), 65);
    assert_eq!(head(&store).accepted_event, None);
    no_delivery(&store);
    store.accept(&published, 0, deadline()).unwrap();
    drop(store);
    let reopened = area.open(false);
    assert_eq!(head(&reopened).accepted_event.as_deref(), Some(ACCEPTED));
    assert_eq!(reopened.pending_delivery(256, deadline()).unwrap().len(), 1);
    for object in objects {
        assert_eq!(
            u64::try_from(reopened.read_object(&object, deadline()).unwrap().len()).unwrap(),
            object.size()
        );
    }
}

#[test]
fn general_acceptance_preserves_its_sixty_four_object_limit() {
    let (_area, store, evidence) = ready();
    let objects = inventory(&store, &evidence, 65);
    let before = head(&store);
    assert!(matches!(
        store.prepare_acceptance(&expected("3"), id(ACCEPTED), &objects, deadline()),
        Err(habitat_engine::store::Error::Bound)
    ));
    assert_eq!(head(&store), before);
    no_delivery(&store);
}

#[test]
fn verified_inventory_refuses_count_overflow_before_publication() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let before = head(&store);
    // Duplicated handles exercise the count guard before the duplicate guard.
    assert!(matches!(
        inventory_proof(&store, &evidence, &vec![evidence.clone(); 4097]),
        Err(habitat_engine::store::Error::Bound)
    ));
    assert_eq!(head(&store), before);
    no_delivery(&store);
}

#[test]
fn verified_inventory_still_refuses_duplicate_cas_objects() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let mut objects = inventory(&store, &evidence, 65);
    objects.push(evidence.clone());
    let before = head(&store);
    assert!(matches!(
        inventory_proof(&store, &evidence, &objects),
        Err(habitat_engine::store::Error::Invalid)
    ));
    assert_eq!(head(&store), before);
    no_delivery(&store);
}

#[test]
fn verified_inventory_checks_late_child_again_before_acceptance() {
    let (area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let objects = inventory(&store, &evidence, 65);
    let published = inventory_proof(&store, &evidence, &objects).unwrap();
    let before = head(&store);
    fs::remove_file(area.object_path(objects.last().unwrap())).unwrap();
    assert!(store.accept(&published, 0, deadline()).is_err());
    assert_eq!(head(&store), before);
    no_delivery(&store);
}

#[test]
fn verified_inventory_accepts_exact_byte_bound_and_refuses_one_more_byte() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let mut objects = vec![evidence.clone()];
    let object_limit = 16 * 1024 * 1024;
    for index in 1..=4_u8 {
        let size = if index == 4 {
            object_limit - EVIDENCE_BYTES.len()
        } else {
            object_limit
        };
        let publication = format!("06000000-2000-4000-8000-{index:012x}");
        objects.push(
            store
                .publish(&vec![index; size], id(&publication), deadline())
                .unwrap(),
        );
    }
    assert_eq!(
        objects.iter().map(Object::size).sum::<u64>(),
        64 * 1024 * 1024
    );
    let extra = store.publish(b"x", id(OTHER), deadline()).unwrap();
    objects.push(extra);
    let before = head(&store);
    assert!(matches!(
        inventory_proof(&store, &evidence, &objects),
        Err(habitat_engine::store::Error::Bound)
    ));
    assert_eq!(head(&store), before);
    no_delivery(&store);
    objects.pop();
    let published = inventory_proof(&store, &evidence, &objects).unwrap();
    store.accept(&published, 0, deadline()).unwrap();
    assert_eq!(head(&store).accepted_event.as_deref(), Some(ACCEPTED));
}

struct Area {
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl Area {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hee3-t06-verification-{}-{}",
            std::process::id(),
            NEXT_AREA.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
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

    fn object_path(&self, object: &Object) -> PathBuf {
        let hex = &object.digest()[7..];
        self.path
            .join("generations")
            .join(GEN)
            .join("objects/sha256")
            .join(&hex[..2])
            .join(hex)
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

fn id(text: &str) -> UuidV4<'_> {
    UuidV4::parse(text).unwrap()
}

fn generation(text: &str) -> Generation {
    text.parse().unwrap()
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}

fn principal() -> Principal {
    Principal::new(1000, "operator").unwrap()
}

fn expected(task_generation: &str) -> Expected<'static> {
    Expected {
        task: id(TASK),
        task_generation: generation(task_generation),
        attempt: id(ATTEMPT),
        attempt_generation: generation("1"),
    }
}

fn observation(
    evidence: &Object,
    verdict: VerificationVerdict,
    used_ms: Option<u64>,
    cleanup_settled: bool,
) -> Verification<'static> {
    Verification {
        verdict,
        subject: Sha256Digest::parse(SUBJECT).unwrap(),
        evidence: evidence.clone(),
        used_ms,
        cleanup_settled,
    }
}

fn head(store: &Store) -> TaskHead {
    store.get(&principal(), id(TASK), deadline()).unwrap()
}

fn no_delivery(store: &Store) {
    assert!(store.pending_delivery(256, deadline()).unwrap().is_empty());
}

fn started() -> (Area, Store, Object) {
    let area = Area::new();
    let mut store = area.open(true);
    store
        .submit(
            Submission {
                principal: &principal(),
                key: id(KEY),
                task: id(TASK),
                event: id(ADMITTED),
                request_bytes: b"independent T06 verification fixture",
                workspace_id: id("28f00000-0000-4000-8000-00000000000a"),
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
    let attempt = store
        .begin_attempt(
            id(TASK),
            generation("1"),
            id(ATTEMPT),
            id(STARTED),
            deadline(),
        )
        .unwrap();
    assert_eq!(attempt.task_generation, "2");
    assert_eq!(attempt.generation, "1");
    let evidence = store
        .publish(EVIDENCE_BYTES, id(STAGE), deadline())
        .unwrap();
    (area, store, evidence)
}

fn settle(store: &mut Store, current: &str, event: &str) -> String {
    store
        .settle_attempt(
            &expected(current),
            Settlement {
                effect: Effect::None,
                used_ms: Some(30),
                cleanup_settled: true,
                ready_to_verify: true,
            },
            id(event),
            deadline(),
        )
        .unwrap()
}

fn ready() -> (Area, Store, Object) {
    let (area, mut store, evidence) = started();
    assert_eq!(settle(&mut store, "2", SETTLED), "3");
    assert_eq!(head(&store).state, "verifying");
    (area, store, evidence)
}

fn record(store: &mut Store, evidence: &Object, verdict: VerificationVerdict) -> String {
    store
        .record_verification(
            &expected("3"),
            &observation(evidence, verdict, Some(20), true),
            id(VERIFIED),
            deadline(),
        )
        .unwrap()
}

fn proof(store: &Store, evidence: &Object, current: &str) -> PublishedAcceptance {
    store
        .prepare_verified_acceptance(
            &expected(current),
            id(ACCEPTED),
            Sha256Digest::parse(SUBJECT).unwrap(),
            evidence,
            std::slice::from_ref(evidence),
            deadline(),
        )
        .unwrap()
}

fn cannot_prepare(store: &Store, evidence: &Object, current: &str) {
    assert!(
        store
            .prepare_verified_acceptance(
                &expected(current),
                id(ACCEPTED),
                Sha256Digest::parse(SUBJECT).unwrap(),
                evidence,
                std::slice::from_ref(evidence),
                deadline(),
            )
            .is_err()
    );
}

#[test]
fn settled_worker_alone_cannot_supply_verified_acceptance() {
    let (_area, store, evidence) = ready();
    let before = head(&store);
    cannot_prepare(&store, &evidence, "3");
    assert_eq!(head(&store), before);
    no_delivery(&store);
}

#[test]
fn passed_checker_records_cost_but_does_not_accept_or_deliver() {
    let (_area, mut store, evidence) = ready();
    assert_eq!(
        record(&mut store, &evidence, VerificationVerdict::Passed),
        "4"
    );
    let current = head(&store);
    assert_eq!(current.state, "verifying");
    assert_eq!(current.spent_ms, 50);
    assert_eq!(current.reserved_work_ms, 770);
    assert_eq!(current.reserved_verify_ms, 180);
    assert_eq!(current.accepted_event, None);
    assert_eq!(
        store.read_object(&evidence, deadline()).unwrap(),
        EVIDENCE_BYTES
    );
    no_delivery(&store);
}

#[test]
fn failed_checker_charges_known_cost_and_requests_repair() {
    let (_area, mut store, evidence) = ready();
    assert_eq!(
        record(&mut store, &evidence, VerificationVerdict::Failed),
        "4"
    );
    let current = head(&store);
    assert_eq!(current.state, "repair_pending");
    assert_eq!((current.spent_ms, current.reserved_verify_ms), (50, 180));
    cannot_prepare(&store, &evidence, "4");
    no_delivery(&store);
}

#[test]
fn invalid_error_and_timeout_are_terminal_failure_not_repair_or_acceptance() {
    for verdict in [
        VerificationVerdict::Invalid,
        VerificationVerdict::Error,
        VerificationVerdict::Timeout,
    ] {
        let (_area, mut store, evidence) = ready();
        record(&mut store, &evidence, verdict);
        let current = head(&store);
        assert_eq!(current.state, "failed");
        assert_eq!((current.spent_ms, current.reserved_verify_ms), (50, 180));
        cannot_prepare(&store, &evidence, "4");
        assert!(
            store
                .begin_attempt(
                    id(TASK),
                    generation("4"),
                    id(OTHER),
                    id(SECOND_START),
                    deadline()
                )
                .is_err()
        );
        no_delivery(&store);
    }
}

#[test]
fn cancelled_verdict_without_intent_refuses_without_consuming_event_or_cost() {
    let (_area, mut store, evidence) = ready();
    let before = head(&store);
    assert!(
        store
            .record_verification(
                &expected("3"),
                &observation(&evidence, VerificationVerdict::Cancelled, Some(20), true),
                id(VERIFIED),
                deadline()
            )
            .is_err()
    );
    assert_eq!(head(&store), before);
    assert_eq!(
        record(&mut store, &evidence, VerificationVerdict::Passed),
        "4"
    );
}

#[test]
fn cancellation_before_pass_wins_and_still_records_checker_cost() {
    let (_area, mut store, evidence) = ready();
    assert_eq!(
        store
            .cancel(id(TASK), generation("3"), id(CANCELLED), deadline())
            .unwrap(),
        "4"
    );
    assert_eq!(
        store
            .record_verification(
                &expected("4"),
                &observation(&evidence, VerificationVerdict::Passed, Some(20), true),
                id(VERIFIED),
                deadline()
            )
            .unwrap(),
        "5"
    );
    let current = head(&store);
    assert!(current.cancellation);
    assert_eq!(current.state, "cancellation_requested");
    assert_eq!((current.spent_ms, current.reserved_verify_ms), (50, 180));
    cannot_prepare(&store, &evidence, "5");
    no_delivery(&store);
}

#[test]
fn cancelled_checker_with_intent_does_not_invent_terminal_cleanup() {
    let (_area, mut store, evidence) = ready();
    store
        .cancel(id(TASK), generation("3"), id(CANCELLED), deadline())
        .unwrap();
    store
        .record_verification(
            &expected("4"),
            &observation(&evidence, VerificationVerdict::Cancelled, Some(20), true),
            id(VERIFIED),
            deadline(),
        )
        .unwrap();
    assert_eq!(head(&store).state, "cancellation_requested");
    assert_eq!(head(&store).accepted_event, None);
    assert!(
        store
            .begin_attempt(
                id(TASK),
                generation("5"),
                id(OTHER),
                id(SECOND_START),
                deadline()
            )
            .is_err()
    );
    no_delivery(&store);
}

#[test]
fn cancellation_after_preparation_invalidates_uncommitted_acceptance() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let prepared = proof(&store, &evidence, "4");
    store
        .cancel(id(TASK), generation("4"), id(CANCELLED), deadline())
        .unwrap();
    assert!(store.accept(&prepared, 0, deadline()).is_err());
    assert_eq!(head(&store).state, "cancellation_requested");
    no_delivery(&store);
}

/// B14a-R1.2: a cancellation bumps the task's generation, and every acceptance door that meets it
/// with the generation it held before names the cancellation — `Cancelled`, never the stale
/// compare-and-set's `Conflict` — so a runtime can stop as cancelled without re-reading anything.
#[test]
fn a_cancellation_is_named_before_the_generation_it_bumped() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let prepared = proof(&store, &evidence, "4");
    store
        .cancel(id(TASK), generation("4"), id(CANCELLED), deadline())
        .unwrap();
    assert!(matches!(
        store.accept(&prepared, 0, deadline()),
        Err(habitat_engine::store::Error::Cancelled)
    ));
    assert!(matches!(
        inventory_proof(&store, &evidence, std::slice::from_ref(&evidence)),
        Err(habitat_engine::store::Error::Cancelled)
    ));
    no_delivery(&store);
}

#[test]
fn acceptance_before_cancellation_remains_historical_and_deliverable() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let prepared = proof(&store, &evidence, "4");
    store.accept(&prepared, 0, deadline()).unwrap();
    let accepted = head(&store);
    assert_eq!(accepted.state, "accepted");
    assert_eq!(
        store
            .cancel(
                id(TASK),
                generation(&accepted.generation),
                id(CANCELLED),
                deadline()
            )
            .unwrap(),
        accepted.generation
    );
    assert_eq!(head(&store), accepted);
    assert_eq!(store.pending_delivery(256, deadline()).unwrap().len(), 1);
}

#[test]
fn unknown_checker_cost_retains_full_liability_across_reopen() {
    let (area, mut store, evidence) = ready();
    store
        .record_verification(
            &expected("3"),
            &observation(&evidence, VerificationVerdict::Passed, None, true),
            id(VERIFIED),
            deadline(),
        )
        .unwrap();
    let unknown = head(&store);
    assert_eq!(unknown.state, "effect_unknown");
    assert_eq!((unknown.spent_ms, unknown.reserved_verify_ms), (30, 200));
    cannot_prepare(&store, &evidence, "4");
    drop(store);
    let mut reopened = area.open(false);
    assert_eq!(head(&reopened), unknown);
    assert!(
        reopened
            .begin_attempt(
                id(TASK),
                generation("4"),
                id(OTHER),
                id(SECOND_START),
                deadline()
            )
            .is_err()
    );
    no_delivery(&reopened);
}

#[test]
fn known_measurement_with_unclean_checker_does_not_free_reserved_capacity() {
    let (_area, mut store, evidence) = ready();
    store
        .record_verification(
            &expected("3"),
            &observation(&evidence, VerificationVerdict::Passed, Some(20), false),
            id(VERIFIED),
            deadline(),
        )
        .unwrap();
    let unknown = head(&store);
    assert_eq!(unknown.state, "effect_unknown");
    assert_eq!((unknown.spent_ms, unknown.reserved_verify_ms), (30, 200));
    cannot_prepare(&store, &evidence, "4");
    assert!(
        store
            .begin_attempt(
                id(TASK),
                generation("4"),
                id(OTHER),
                id(SECOND_START),
                deadline()
            )
            .is_err()
    );
    no_delivery(&store);
}

#[test]
fn measured_zero_is_settled_and_distinct_from_unknown_cost() {
    let (_area, mut store, evidence) = ready();
    store
        .record_verification(
            &expected("3"),
            &observation(&evidence, VerificationVerdict::Passed, Some(0), true),
            id(VERIFIED),
            deadline(),
        )
        .unwrap();
    assert_eq!(head(&store).state, "verifying");
    assert_eq!(
        (head(&store).spent_ms, head(&store).reserved_verify_ms),
        (30, 200)
    );
    let prepared = proof(&store, &evidence, "4");
    store.accept(&prepared, 0, deadline()).unwrap();
    assert_eq!(head(&store).spent_ms, 30);
}

#[test]
fn cost_above_reserve_refuses_but_exact_reserve_can_finish_without_extra_charge() {
    let (_area, mut store, evidence) = ready();
    let before = head(&store);
    assert!(
        store
            .record_verification(
                &expected("3"),
                &observation(&evidence, VerificationVerdict::Passed, Some(201), true),
                id(VERIFIED),
                deadline()
            )
            .is_err()
    );
    assert_eq!(head(&store), before);
    store
        .record_verification(
            &expected("3"),
            &observation(&evidence, VerificationVerdict::Passed, Some(200), true),
            id(VERIFIED),
            deadline(),
        )
        .unwrap();
    assert_eq!(
        (head(&store).spent_ms, head(&store).reserved_verify_ms),
        (230, 0)
    );
    let prepared = proof(&store, &evidence, "4");
    store.accept(&prepared, 0, deadline()).unwrap();
    assert_eq!(head(&store).spent_ms, 230);
}

#[test]
fn running_worker_cannot_be_checked_and_refusal_does_not_poison_later_settlement() {
    let (_area, mut store, evidence) = started();
    let before = head(&store);
    assert!(
        store
            .record_verification(
                &expected("2"),
                &observation(&evidence, VerificationVerdict::Passed, Some(20), true),
                id(VERIFIED),
                deadline()
            )
            .is_err()
    );
    assert_eq!(head(&store), before);
    settle(&mut store, "2", SETTLED);
    assert_eq!(
        record(&mut store, &evidence, VerificationVerdict::Passed),
        "4"
    );
}

#[test]
fn each_unsettled_worker_dimension_refuses_verification_until_real_settlement() {
    for (effect, used_ms, cleanup_settled) in [
        (Effect::Unknown, Some(30), true),
        (Effect::None, None, true),
        (Effect::None, Some(30), false),
    ] {
        let (_area, mut store, evidence) = started();
        store
            .settle_attempt(
                &expected("2"),
                Settlement {
                    effect,
                    used_ms,
                    cleanup_settled,
                    ready_to_verify: true,
                },
                id(SETTLED),
                deadline(),
            )
            .unwrap();
        let before = head(&store);
        assert_eq!(before.state, "effect_unknown");
        assert!(
            store
                .record_verification(
                    &expected("3"),
                    &observation(&evidence, VerificationVerdict::Passed, Some(20), true),
                    id(VERIFIED),
                    deadline()
                )
                .is_err()
        );
        assert_eq!(head(&store), before);
        settle(&mut store, "3", SECOND_SETTLE);
        store
            .record_verification(
                &expected("4"),
                &observation(&evidence, VerificationVerdict::Passed, Some(20), true),
                id(VERIFIED),
                deadline(),
            )
            .unwrap();
        assert_eq!(head(&store).spent_ms, 50);
    }
}

#[test]
fn stale_task_revision_cannot_record_against_a_current_attempt() {
    let (_area, mut store, evidence) = ready();
    let before = head(&store);
    assert!(
        store
            .record_verification(
                &expected("2"),
                &observation(&evidence, VerificationVerdict::Passed, Some(20), true),
                id(VERIFIED),
                deadline()
            )
            .is_err()
    );
    assert_eq!(head(&store), before);
    record(&mut store, &evidence, VerificationVerdict::Passed);
}

#[test]
fn wrong_attempt_identity_or_generation_cannot_borrow_settled_worker_authority() {
    let (_area, mut store, evidence) = ready();
    let before = head(&store);
    for (attempt, attempt_generation) in [(OTHER, "1"), (ATTEMPT, "2")] {
        let wrong = Expected {
            attempt: id(attempt),
            attempt_generation: generation(attempt_generation),
            ..expected("3")
        };
        assert!(
            store
                .record_verification(
                    &wrong,
                    &observation(&evidence, VerificationVerdict::Passed, Some(20), true),
                    id(VERIFIED),
                    deadline()
                )
                .is_err()
        );
        assert_eq!(head(&store), before);
    }
    record(&mut store, &evidence, VerificationVerdict::Passed);
}

#[test]
fn one_attempt_cannot_gain_a_second_checker_return_by_changing_event_or_verdict() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let before = head(&store);
    for (event, verdict) in [
        (VERIFIED, VerificationVerdict::Passed),
        (SECOND_VERIFY, VerificationVerdict::Failed),
    ] {
        assert!(
            store
                .record_verification(
                    &expected("4"),
                    &observation(&evidence, verdict, Some(20), true),
                    id(event),
                    deadline()
                )
                .is_err()
        );
        assert_eq!(head(&store), before);
    }
    no_delivery(&store);
}

#[test]
fn a_terminal_verifier_error_cannot_be_replaced_by_a_later_pass() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Error);
    let failed = head(&store);
    assert!(
        store
            .record_verification(
                &expected("4"),
                &observation(&evidence, VerificationVerdict::Passed, Some(0), true),
                id(SECOND_VERIFY),
                deadline()
            )
            .is_err()
    );
    assert_eq!(head(&store), failed);
    cannot_prepare(&store, &evidence, "4");
}

#[test]
fn repaired_attempt_requires_its_own_check_and_costs_accumulate_once() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Failed);
    let attempt = store
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
            attempt.task_generation.as_str(),
            attempt.generation.as_str()
        ),
        ("5", "2")
    );
    let active = Expected {
        task: id(TASK),
        task_generation: generation("5"),
        attempt: id(SECOND_ATTEMPT),
        attempt_generation: generation("2"),
    };
    store
        .settle_attempt(
            &active,
            Settlement {
                effect: Effect::None,
                used_ms: Some(40),
                cleanup_settled: true,
                ready_to_verify: true,
            },
            id(SECOND_SETTLE),
            deadline(),
        )
        .unwrap();
    let current = Expected {
        task_generation: generation("6"),
        ..active
    };
    assert!(
        store
            .prepare_verified_acceptance(
                &current,
                id(ACCEPTED),
                Sha256Digest::parse(SUBJECT).unwrap(),
                &evidence,
                std::slice::from_ref(&evidence),
                deadline()
            )
            .is_err()
    );
    store
        .record_verification(
            &current,
            &observation(&evidence, VerificationVerdict::Passed, Some(10), true),
            id(SECOND_VERIFY),
            deadline(),
        )
        .unwrap();
    assert_eq!(
        (head(&store).spent_ms, head(&store).reserved_verify_ms),
        (100, 170)
    );
    let checked = Expected {
        task_generation: generation("7"),
        ..current
    };
    let prepared = store
        .prepare_verified_acceptance(
            &checked,
            id(ACCEPTED),
            Sha256Digest::parse(SUBJECT).unwrap(),
            &evidence,
            std::slice::from_ref(&evidence),
            deadline(),
        )
        .unwrap();
    store.accept(&prepared, 0, deadline()).unwrap();
    assert_eq!(head(&store).spent_ms, 100);
}

#[test]
fn a_failed_check_that_exhausts_verification_reserve_cannot_start_repair() {
    let (_area, mut store, evidence) = ready();
    store
        .record_verification(
            &expected("3"),
            &observation(&evidence, VerificationVerdict::Failed, Some(200), true),
            id(VERIFIED),
            deadline(),
        )
        .unwrap();
    let exhausted = head(&store);
    assert_eq!(exhausted.state, "repair_pending");
    assert_eq!(exhausted.reserved_verify_ms, 0);
    assert!(
        store
            .begin_attempt(
                id(TASK),
                generation("4"),
                id(SECOND_ATTEMPT),
                id(SECOND_START),
                deadline()
            )
            .is_err()
    );
    assert_eq!(head(&store), exhausted);
    no_delivery(&store);
}

#[test]
fn pass_proof_is_bound_to_exact_subject_not_just_a_successful_verdict() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    assert!(
        store
            .prepare_verified_acceptance(
                &expected("4"),
                id(ACCEPTED),
                Sha256Digest::parse(CRITERIA).unwrap(),
                &evidence,
                std::slice::from_ref(&evidence),
                deadline()
            )
            .is_err()
    );
    let prepared = proof(&store, &evidence, "4");
    store.accept(&prepared, 0, deadline()).unwrap();
}

#[test]
fn a_different_valid_object_cannot_substitute_for_the_recorded_check_evidence() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let other = store
        .publish(b"different valid checker return", id(OTHER), deadline())
        .unwrap();
    cannot_prepare(&store, &other, "4");
    let prepared = proof(&store, &evidence, "4");
    store.accept(&prepared, 0, deadline()).unwrap();
}

#[test]
fn acceptance_object_inventory_must_include_the_exact_check_evidence() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let candidate = store.publish(b"hello", id(OTHER), deadline()).unwrap();
    assert_eq!(candidate.digest(), SUBJECT);
    assert!(
        store
            .prepare_verified_acceptance(
                &expected("4"),
                id(ACCEPTED),
                Sha256Digest::parse(SUBJECT).unwrap(),
                &evidence,
                std::slice::from_ref(&candidate),
                deadline()
            )
            .is_err()
    );
    let prepared = store
        .prepare_verified_acceptance(
            &expected("4"),
            id(ACCEPTED),
            Sha256Digest::parse(SUBJECT).unwrap(),
            &evidence,
            &[candidate, evidence.clone()],
            deadline(),
        )
        .unwrap();
    store.accept(&prepared, 0, deadline()).unwrap();
}

#[test]
fn an_object_from_another_store_cannot_create_a_dangling_verification_reference() {
    let (_area, mut store, _evidence) = ready();
    let foreign_area = Area::new();
    let foreign_store = foreign_area.open(true);
    let foreign = foreign_store
        .publish(b"foreign evidence not present here", id(STAGE), deadline())
        .unwrap();
    let before = head(&store);
    assert!(
        store
            .record_verification(
                &expected("3"),
                &observation(&foreign, VerificationVerdict::Passed, Some(20), true),
                id(VERIFIED),
                deadline()
            )
            .is_err()
    );
    assert_eq!(head(&store), before);
    let local = store
        .publish(b"foreign evidence not present here", id(OTHER), deadline())
        .unwrap();
    assert_eq!(local, foreign);
    record(&mut store, &local, VerificationVerdict::Passed);
}

#[test]
fn corrupt_evidence_is_refused_before_any_verification_state_or_cost_commit() {
    let (area, mut store, evidence) = ready();
    let before = head(&store);
    let retained = area.object_path(&evidence);
    let metadata = fs::symlink_metadata(&retained).unwrap();
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    fs::set_permissions(&retained, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&retained, b"corrupt retained bytes").unwrap();
    fs::set_permissions(&retained, fs::Permissions::from_mode(0o444)).unwrap();
    assert!(store.read_object(&evidence, deadline()).is_err());
    assert!(
        store
            .record_verification(
                &expected("3"),
                &observation(&evidence, VerificationVerdict::Passed, Some(20), true),
                id(VERIFIED),
                deadline()
            )
            .is_err()
    );
    assert_eq!(head(&store), before);
    no_delivery(&store);
}

#[test]
fn evidence_retention_loss_after_preparation_cannot_commit_acceptance_or_outbox() {
    let (area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let prepared = proof(&store, &evidence, "4");
    let before = head(&store);
    fs::remove_file(area.object_path(&evidence)).unwrap();
    assert!(store.accept(&prepared, 0, deadline()).is_err());
    assert_eq!(head(&store), before);
    no_delivery(&store);
}

#[test]
fn duplicate_event_conflict_rolls_back_verification_row_and_cost_across_reopen() {
    let (area, mut store, evidence) = ready();
    let before = head(&store);
    assert!(
        store
            .record_verification(
                &expected("3"),
                &observation(&evidence, VerificationVerdict::Passed, Some(20), true),
                id(ADMITTED),
                deadline()
            )
            .is_err()
    );
    assert_eq!(head(&store), before);
    drop(store);
    let mut reopened = area.open(false);
    assert_eq!(head(&reopened), before);
    assert_eq!(
        record(&mut reopened, &evidence, VerificationVerdict::Passed),
        "4"
    );
    assert_eq!(head(&reopened).spent_ms, 50);
}

#[test]
fn recorded_pass_survives_reopen_and_acceptance_commits_one_durable_delivery() {
    let (area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    drop(store);
    let mut reopened = area.open(false);
    let prepared = proof(&reopened, &evidence, "4");
    let sequence = reopened.accept(&prepared, 0, deadline()).unwrap();
    let accepted = head(&reopened);
    assert_eq!(accepted.state, "accepted");
    assert_eq!(accepted.accepted_event.as_deref(), Some(ACCEPTED));
    assert_eq!(
        (
            accepted.spent_ms,
            accepted.reserved_work_ms,
            accepted.reserved_verify_ms
        ),
        (50, 0, 0)
    );
    assert!(reopened.accept(&prepared, 0, deadline()).is_err());
    drop(reopened);
    let mut delivered = area.open(false);
    assert_eq!(head(&delivered), accepted);
    assert_eq!(
        delivered.pending_delivery(256, deadline()).unwrap(),
        vec![(ACCEPTED.to_owned(), "1000:operator".to_owned(), sequence)]
    );
    delivered
        .acknowledge_delivery(id(ACCEPTED), "1000:operator", deadline())
        .unwrap();
    no_delivery(&delivered);
    assert_eq!(head(&delivered), accepted);
}

#[test]
fn expired_verification_deadline_cannot_write_or_consume_the_event() {
    let (_area, mut store, evidence) = ready();
    let before = head(&store);
    assert!(
        store
            .record_verification(
                &expected("3"),
                &observation(&evidence, VerificationVerdict::Passed, Some(20), true),
                id(VERIFIED),
                Instant::now().checked_sub(Duration::from_secs(1)).unwrap()
            )
            .is_err()
    );
    assert_eq!(head(&store), before);
    assert_eq!(
        record(&mut store, &evidence, VerificationVerdict::Passed),
        "4"
    );
}

#[test]
fn verified_acceptance_refuses_a_second_cost_charge_then_accepts_zero() {
    let (_area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let prepared = proof(&store, &evidence, "4");
    let before = head(&store);
    assert!(store.accept(&prepared, 20, deadline()).is_err());
    assert_eq!(head(&store), before);
    no_delivery(&store);
    store.accept(&prepared, 0, deadline()).unwrap();
    assert_eq!(head(&store).spent_ms, 50);
    assert_eq!(store.pending_delivery(256, deadline()).unwrap().len(), 1);
}

/// B14a-1a · a bound task is accepted only on a bound attempt: the rule is kept where acceptance
/// commits, so it holds even against a ledger the begin doors could not have produced — here a
/// second connection binds another attempt of the same task, leaving the current one unbound. And a
/// ledger mixing bound and unbound attempts of one task no longer opens.
#[test]
fn a_bound_task_is_accepted_only_on_a_bound_attempt() {
    let (area, mut store, evidence) = ready();
    record(&mut store, &evidence, VerificationVerdict::Passed);
    let prepared = proof(&store, &evidence, "4");
    let digest = format!("sha256:{}", "1".repeat(64));
    let db = rusqlite::Connection::open(
        area.path
            .join("generations")
            .join(GEN)
            .join("ledger.sqlite3"),
    )
    .unwrap();
    db.execute_batch(&format!(
        "INSERT INTO attempts(id,task_id,generation,state,effect,cleanup,used_ms) \
         VALUES('{OTHER}','{TASK}','9','settled','none','settled',0); \
         INSERT INTO attempt_bindings(attempt_id,task_id,baseline_digest,protected_digest,profile_digest) \
         VALUES('{OTHER}','{TASK}','{digest}','{digest}','{digest}');"
    ))
    .unwrap();
    drop(db);
    assert!(matches!(
        store.accept(&prepared, 0, deadline()),
        Err(habitat_engine::store::Error::Conflict)
    ));
    no_delivery(&store);
    drop(store);
    assert!(matches!(
        Store::open(&area.path, id(GEN), id(EPOCH), false, deadline()),
        Err(habitat_engine::store::Error::Corrupt)
    ));
}
