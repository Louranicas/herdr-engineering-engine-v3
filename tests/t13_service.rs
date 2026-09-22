//! T13 service-owned controls using the actual admitted Store/roster API.
//! No service process, remote endpoint or lifecycle command is invoked.
use habitat_engine::contracts::{
    UuidV4,
    roster::{
        self, ActiveAttemptPolicy, Disable, Kind, Locality, ObservationSource, RosterDefinitionV1,
        Update,
    },
};
use habitat_engine::service::{
    self, Class, Error, Facts, Health, ProbeObservation, Profile, Recorded, UnknownReason,
    UsefulResult,
};
use habitat_engine::store::{Principal, RequestSource, Store};
use serde::Serialize;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
const GEN: &str = "10000000-0000-4000-8000-000000000001";
const EPOCH: &str = "10000000-0000-4000-8000-000000000002";
const ENDPOINT: &str = "10000000-0000-4000-8000-000000000003";
const EVIDENCE: &str = "10000000-0000-4000-8000-000000000004";
static NEXT: AtomicU64 = AtomicU64::new(0);
fn uuid(value: &str) -> UuidV4<'_> {
    UuidV4::parse(value).unwrap()
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn principal() -> Principal {
    Principal::new(1000, "operator").unwrap()
}
fn key(value: u64) -> String {
    format!("20000000-0000-4000-8000-{value:012x}")
}
struct Area {
    path: PathBuf,
    inode: u64,
}
impl Area {
    fn new() -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "t13-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
        let inode = fs::metadata(&path).unwrap().ino();
        Self { path, inode }
    }
    fn open(&self, create: bool) -> Store {
        Store::open(&self.path, uuid(GEN), uuid(EPOCH), create, deadline()).unwrap()
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
#[derive(Serialize)]
struct Changes<'a> {
    kind: &'static str,
    version: u16,
    updates: Vec<Row<'a>>,
}
#[derive(Serialize)]
struct Row<'a> {
    idempotency_key: &'a str,
    record_id: Option<&'a str>,
    expected_revision: Option<&'a str>,
    audit_reason: &'a str,
    definition: &'a RosterDefinitionV1,
}
fn manifest(update: &Update) -> Vec<u8> {
    toml::to_string(&Changes {
        kind: "hee3-roster-changes",
        version: 1,
        updates: vec![Row {
            idempotency_key: &update.idempotency_key,
            record_id: update.record_id.as_deref(),
            expected_revision: update.expected_revision.as_deref(),
            audit_reason: &update.audit_reason,
            definition: &update.definition,
        }],
    })
    .unwrap()
    .into_bytes()
}
fn apply(store: &mut Store, update: &Update) -> habitat_engine::contracts::roster::RosterHeadV1 {
    let bytes = manifest(update);
    store
        .roster_apply(
            &principal(),
            std::slice::from_ref(update),
            RequestSource::Import(&bytes),
            deadline(),
        )
        .unwrap()[0]
        .head
        .clone()
}
struct Fixture {
    store: Store,
    root: Area,
    profile: Profile,
    create: Update,
}
impl Fixture {
    fn new(class: Class) -> Self {
        let root = Area::new();
        let mut store = root.open(true);
        let create = Update {
            idempotency_key: key(1),
            record_id: None,
            expected_revision: None,
            audit_reason: "reviewed service fixture".into(),
            definition: RosterDefinitionV1 {
                kind: Kind::Service,
                display_name: "fixture service".into(),
                owner_id: "fixture-owner".into(),
                version: "service-v1".into(),
                capabilities: vec!["useful-work".into()],
                locality: if class == Class::RemoteEndpoint {
                    Locality::Remote
                } else {
                    Locality::Local
                },
                endpoint_ref: Some(ENDPOINT.into()),
                limitations: "no lifecycle grant".into(),
            },
        };
        let head = apply(&mut store, &create);
        let profile = Profile {
            record_id: head.record_id,
            record_version: head.record_version,
            owner_id: head.definition.owner_id,
            endpoint_ref: head.definition.endpoint_ref,
            class,
            probe_id: "useful-check".into(),
            probe_version: 1,
            actual_identity: "fixture-target".into(),
            immutable_revision: "sha256:fixture-version".into(),
        };
        Self {
            store,
            root,
            profile,
            create,
        }
    }
    fn observe(&mut self, facts: &ProbeObservation) -> Recorded {
        service::record_probe(
            &mut self.store,
            &principal(),
            &self.profile,
            facts,
            deadline(),
        )
        .unwrap()
    }
    fn inspect(&self, evidence: Option<&Recorded>) -> service::Inspection {
        service::inspect(
            &self.store,
            &principal(),
            uuid(&self.profile.record_id),
            std::slice::from_ref(&self.profile),
            evidence,
            60_000,
            deadline(),
        )
        .unwrap()
    }
    fn update(&mut self, key_id: u64) -> Update {
        let head = self
            .store
            .roster_get(&principal(), uuid(&self.profile.record_id), deadline())
            .unwrap()
            .head;
        Update {
            idempotency_key: key(key_id),
            record_id: Some(head.record_id),
            expected_revision: Some(head.record_version),
            definition: head.definition,
            audit_reason: "reviewed update".into(),
        }
    }
}
fn facts(class: Class, result: UsefulResult) -> ProbeObservation {
    let facts = match class {
        Class::Daemon => Facts::Daemon {
            alive: Some(true),
            request: result,
        },
        Class::OneShot => Facts::OneShot {
            exit_code: Some(0),
            output: result,
        },
        Class::Library => Facts::Library {
            loaded: Some(true),
            call: result,
        },
        Class::RemoteEndpoint => Facts::RemoteEndpoint {
            reachable: Some(true),
            request: result,
        },
        Class::NeuralOperator => Facts::NeuralOperator {
            model_matches: Some(true),
            result,
        },
    };
    ProbeObservation {
        facts,
        probe_id: "useful-check".into(),
        probe_version: 1,
        actual_identity: Some("fixture-target".into()),
        immutable_revision: Some("sha256:fixture-version".into()),
        observed_unix_ms: None,
        latency_ms: Some(2),
        usage_ms: Some(3),
        cost_microunits: Some(0),
        cleanup_settled: true,
        evidence_ref: EVIDENCE.into(),
    }
}
fn useful(class: Class) {
    let mut f = Fixture::new(class);
    let record = f.observe(&facts(class, UsefulResult::Passed));
    assert_eq!(record.health(), Health::Useful);
    assert_eq!(f.inspect(Some(&record)).health, Health::Useful);
    assert!(record.observation().input.capabilities.is_empty());
}
#[test]
fn daemon_useful_request() {
    useful(Class::Daemon);
}
#[test]
fn one_shot_useful_output() {
    useful(Class::OneShot);
}
#[test]
fn library_useful_call_without_daemon_state() {
    useful(Class::Library);
}
#[test]
fn remote_useful_return_is_observation_not_network_dispatch() {
    useful(Class::RemoteEndpoint);
}
#[test]
fn neural_operator_useful_result() {
    useful(Class::NeuralOperator);
}
#[test]
fn live_daemon_with_failed_request_is_unavailable() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Failed));
    assert_eq!(f.inspect(Some(&r)).health, Health::Unavailable);
}
#[test]
fn zero_exit_without_useful_output_is_unknown() {
    let mut f = Fixture::new(Class::OneShot);
    let r = f.observe(&facts(Class::OneShot, UsefulResult::Unknown));
    assert_eq!(f.inspect(Some(&r)).health, Health::Unknown);
}
#[test]
fn loaded_library_failed_call_is_unavailable() {
    let mut f = Fixture::new(Class::Library);
    let r = f.observe(&facts(Class::Library, UsefulResult::Failed));
    assert_eq!(f.inspect(Some(&r)).health, Health::Unavailable);
}
#[test]
fn reachable_remote_failed_request_is_unavailable() {
    let mut f = Fixture::new(Class::RemoteEndpoint);
    let r = f.observe(&facts(Class::RemoteEndpoint, UsefulResult::Failed));
    assert_eq!(f.inspect(Some(&r)).health, Health::Unavailable);
}
#[test]
fn wrong_neural_model_cannot_be_useful() {
    let mut f = Fixture::new(Class::NeuralOperator);
    let mut input = facts(Class::NeuralOperator, UsefulResult::Passed);
    input.facts = Facts::NeuralOperator {
        model_matches: Some(false),
        result: UsefulResult::Passed,
    };
    let r = f.observe(&input);
    assert_eq!(f.inspect(Some(&r)).health, Health::Unavailable);
}
#[test]
fn cancelled_probe_remains_unknown() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Cancelled));
    assert_eq!(r.health(), Health::Unknown);
}
#[test]
fn timed_out_probe_remains_unknown() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::TimedOut));
    assert_eq!(r.health(), Health::Unknown);
}
#[test]
fn unknown_cleanup_cannot_be_useful() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.cleanup_settled = false;
    let r = f.observe(&input);
    assert_eq!(r.health(), Health::Unknown);
    assert!(!r.facts().cleanup_settled);
}
#[test]
fn unknown_cost_is_not_zero() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.cost_microunits = None;
    let r = f.observe(&input);
    assert_eq!(r.health(), Health::Unknown);
    assert_eq!(r.facts().cost_microunits, None);
}
#[test]
fn unknown_usage_is_not_zero() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.usage_ms = None;
    let r = f.observe(&input);
    assert_eq!(r.health(), Health::Unknown);
}
#[test]
fn paid_profile_refused_without_write() {
    let mut f = Fixture::new(Class::Daemon);
    let before = f
        .store
        .roster_snapshot(&principal(), deadline())
        .unwrap()
        .cutoff;
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.cost_microunits = Some(1);
    assert!(
        service::record_probe(&mut f.store, &principal(), &f.profile, &input, deadline()).is_err()
    );
    assert_eq!(
        f.store
            .roster_snapshot(&principal(), deadline())
            .unwrap()
            .cutoff,
        before
    );
}
#[test]
fn missing_proof_never_promotes_generic_available() {
    let mut f = Fixture::new(Class::Daemon);
    let _r = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    assert_eq!(
        f.inspect(None).unknown,
        Some(UnknownReason::MissingEvidence)
    );
}
#[test]
fn changed_profile_cannot_reuse_old_useful_receipt() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    f.profile.class = Class::Library;
    assert_eq!(
        f.inspect(Some(&r)).unknown,
        Some(UnknownReason::EvidenceBinding)
    );
}
#[test]
fn changed_recipe_cannot_reuse_old_useful_receipt() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    f.profile.probe_id = "other-probe".into();
    assert_eq!(
        f.inspect(Some(&r)).unknown,
        Some(UnknownReason::EvidenceBinding)
    );
}
#[test]
fn missing_profile_is_unknown() {
    let f = Fixture::new(Class::Daemon);
    let r = service::inspect(
        &f.store,
        &principal(),
        uuid(&f.profile.record_id),
        &[],
        None,
        60_000,
        deadline(),
    )
    .unwrap();
    assert_eq!(r.unknown, Some(UnknownReason::MissingProfile));
}
#[test]
fn ambiguous_profiles_are_unknown() {
    let f = Fixture::new(Class::Daemon);
    let r = service::inspect(
        &f.store,
        &principal(),
        uuid(&f.profile.record_id),
        &[f.profile.clone(), f.profile.clone()],
        None,
        60_000,
        deadline(),
    )
    .unwrap();
    assert_eq!(r.unknown, Some(UnknownReason::AmbiguousProfile));
}
#[test]
fn inspect_has_no_store_write_or_probe() {
    let f = Fixture::new(Class::Daemon);
    let before = f.store.roster_snapshot(&principal(), deadline()).unwrap();
    let _r = f.inspect(None);
    let after = f.store.roster_snapshot(&principal(), deadline()).unwrap();
    assert_eq!(before.cutoff, after.cutoff);
    assert_eq!(before.records, after.records);
}
#[test]
fn stale_observation_unknown_at_exact_ttl() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    let mut snapshot = f.store.roster_snapshot(&principal(), deadline()).unwrap();
    snapshot.now.monotonic_ms = r.observation().received.monotonic_ms + 60_000;
    assert_eq!(
        service::inspect_snapshot(
            &snapshot,
            uuid(&f.profile.record_id),
            std::slice::from_ref(&f.profile),
            Some(&r),
            60_000
        )
        .unwrap()
        .unknown,
        Some(UnknownReason::Freshness(roster::Freshness::Expired))
    );
}
#[test]
fn clock_regression_unknown() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    let mut snapshot = f.store.roster_snapshot(&principal(), deadline()).unwrap();
    snapshot.now.unix_ms = 0;
    assert_eq!(
        service::inspect_snapshot(
            &snapshot,
            uuid(&f.profile.record_id),
            std::slice::from_ref(&f.profile),
            Some(&r),
            60_000
        )
        .unwrap()
        .unknown,
        Some(UnknownReason::Freshness(roster::Freshness::ClockRegression))
    );
}
#[test]
fn restart_preserves_inventory_but_expires_receiver_epoch() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    drop(f.store);
    f.store = f.root.open(false);
    assert_eq!(
        f.inspect(Some(&r)).unknown,
        Some(UnknownReason::Freshness(roster::Freshness::EpochMismatch))
    );
    let fresh = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    assert_eq!(f.inspect(Some(&fresh)).health, Health::Useful);
}
#[test]
fn disabled_record_and_stale_profile_cannot_revive() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    let disable = Disable {
        idempotency_key: key(2),
        record_id: f.profile.record_id.clone(),
        expected_revision: f.profile.record_version.clone(),
        active_attempt_policy: ActiveAttemptPolicy::LeaveRunning,
        audit_reason: "fixture disable".into(),
    };
    let bytes=serde_json::to_vec(&serde_json::json!({"action":"roster.disable","record_id":disable.record_id,"expected_revision":disable.expected_revision,"idempotency_key":disable.idempotency_key,"active_attempt_policy":disable.active_attempt_policy,"audit_reason":disable.audit_reason})).unwrap();
    f.store
        .roster_disable(&principal(), &disable, &bytes, deadline())
        .unwrap();
    assert_eq!(f.inspect(Some(&r)).unknown, Some(UnknownReason::Disabled));
    assert!(
        service::record_probe(
            &mut f.store,
            &principal(),
            &f.profile,
            &facts(Class::Daemon, UsefulResult::Passed),
            deadline()
        )
        .is_err()
    );
}
#[test]
fn admitted_update_invalidates_protected_profile() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    let mut change = f.update(2);
    change.definition.version = "service-v2".into();
    apply(&mut f.store, &change);
    assert_eq!(
        f.inspect(Some(&r)).unknown,
        Some(UnknownReason::ProfileBinding)
    );
}
#[test]
fn foreign_owner_profile_refused() {
    let mut f = Fixture::new(Class::Daemon);
    f.profile.owner_id = "other-owner".into();
    assert!(
        service::record_probe(
            &mut f.store,
            &principal(),
            &f.profile,
            &facts(Class::Daemon, UsefulResult::Passed),
            deadline()
        )
        .is_err()
    );
}
#[test]
fn wrong_endpoint_profile_refused() {
    let mut f = Fixture::new(Class::Daemon);
    f.profile.endpoint_ref = None;
    assert!(
        service::record_probe(
            &mut f.store,
            &principal(),
            &f.profile,
            &facts(Class::Daemon, UsefulResult::Passed),
            deadline()
        )
        .is_err()
    );
}
#[test]
fn mismatched_fact_class_refused() {
    let mut f = Fixture::new(Class::Daemon);
    assert!(matches!(
        service::record_probe(
            &mut f.store,
            &principal(),
            &f.profile,
            &facts(Class::Library, UsefulResult::Passed),
            deadline()
        ),
        Err(Error::Binding)
    ));
}
#[test]
fn future_source_observation_refused_without_write() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.observed_unix_ms = Some(u64::MAX);
    assert!(matches!(
        service::record_probe(&mut f.store, &principal(), &f.profile, &input, deadline()),
        Err(Error::FutureObservation)
    ));
    assert!(
        f.store
            .roster_get(&principal(), uuid(&f.profile.record_id), deadline())
            .unwrap()
            .observation
            .is_none()
    );
}
#[test]
fn actual_identity_mismatch_remains_unknown() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.actual_identity = Some("foreign-target".into());
    let r = f.observe(&input);
    assert_eq!(r.health(), Health::Unknown);
}
#[test]
fn actual_revision_mismatch_remains_unknown() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.immutable_revision = Some("foreign-revision".into());
    let r = f.observe(&input);
    assert_eq!(r.health(), Health::Unknown);
}
#[test]
fn invisible_principal_gets_not_found() {
    let f = Fixture::new(Class::Daemon);
    let other = Principal::new(1001, "operator").unwrap();
    assert!(matches!(
        service::inspect(
            &f.store,
            &other,
            uuid(&f.profile.record_id),
            std::slice::from_ref(&f.profile),
            None,
            60_000,
            deadline()
        ),
        Err(Error::Store(habitat_engine::store::Error::NotFound))
    ));
}
#[test]
fn stale_import_cannot_overwrite_admitted_service() {
    let mut f = Fixture::new(Class::Daemon);
    let mut update = f.update(2);
    update.definition.version = "v2".into();
    apply(&mut f.store, &update);
    let stale = f.create.clone();
    let result = apply(&mut f.store, &stale);
    assert_eq!(result.record_version, "1");
    assert_eq!(
        f.store
            .roster_get(&principal(), uuid(&f.profile.record_id), deadline())
            .unwrap()
            .head
            .definition
            .version,
        "v2"
    );
}
#[test]
fn conflicting_import_key_is_refused() {
    let mut f = Fixture::new(Class::Daemon);
    let mut changed = f.create.clone();
    changed.definition.owner_id = "unreviewed".into();
    let bytes = manifest(&changed);
    assert!(
        f.store
            .roster_apply(
                &principal(),
                &[changed],
                RequestSource::Import(&bytes),
                deadline()
            )
            .is_err()
    );
}
#[test]
fn exported_observation_is_not_import_authority() {
    let mut f = Fixture::new(Class::Daemon);
    let _record = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    let snapshot = f.store.roster_snapshot(&principal(), deadline()).unwrap();
    let exported = habitat_engine::roster::export_snapshot(&snapshot).unwrap();
    assert!(habitat_engine::roster::parse_changes(exported.as_bytes()).is_err());
    assert_eq!(
        f.store
            .roster_snapshot(&principal(), deadline())
            .unwrap()
            .cutoff,
        snapshot.cutoff
    );
}
#[test]
fn untrusted_source_label_never_promotes_service_health() {
    let mut f = Fixture::new(Class::Daemon);
    let r = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    let mut claimed = r.observation().input.clone();
    claimed.source = ObservationSource::ServiceProbe;
    f.store
        .roster_observe(&principal(), &claimed, deadline())
        .unwrap();
    assert_eq!(
        f.inspect(Some(&r)).unknown,
        Some(UnknownReason::Freshness(roster::Freshness::UntrustedSource))
    );
}

#[test]
fn new_observation_invalidates_older_recorded_proof() {
    let mut f = Fixture::new(Class::Daemon);
    let old = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    let fresh = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    assert_eq!(
        f.inspect(Some(&old)).unknown,
        Some(UnknownReason::EvidenceBinding)
    );
    assert_eq!(f.inspect(Some(&fresh)).health, Health::Useful);
}
#[test]
fn confirmed_worker_availability_is_not_service_proof() {
    let mut f = Fixture::new(Class::Daemon);
    let old = f.observe(&facts(Class::Daemon, UsefulResult::Passed));
    let mut input = old.observation().input.clone();
    input.source = ObservationSource::Worker;
    f.store
        .roster_observe_worker(&principal(), &input, deadline())
        .unwrap();
    assert_eq!(f.inspect(Some(&old)).health, Health::Unknown);
}
#[test]
fn unknown_type_prerequisite_prevents_useful_health() {
    let mut f = Fixture::new(Class::Library);
    let mut input = facts(Class::Library, UsefulResult::Passed);
    input.facts = Facts::Library {
        loaded: None,
        call: UsefulResult::Passed,
    };
    let observed = f.observe(&input);
    assert_eq!(observed.health(), Health::Unknown);
}
#[test]
fn zero_known_usage_is_distinct_from_unknown() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.usage_ms = Some(0);
    let observed = f.observe(&input);
    assert_eq!(observed.health(), Health::Useful);
    assert_eq!(observed.facts().usage_ms, Some(0));
}
#[test]
fn invalid_evidence_uuid_never_records() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.evidence_ref = "arbitrary://evidence".into();
    assert!(
        service::record_probe(&mut f.store, &principal(), &f.profile, &input, deadline()).is_err()
    );
    assert!(
        f.store
            .roster_get(&principal(), uuid(&f.profile.record_id), deadline())
            .unwrap()
            .observation
            .is_none()
    );
}
#[test]
fn unsupported_probe_version_refused() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.probe_version = 2;
    assert!(
        service::record_probe(&mut f.store, &principal(), &f.profile, &input, deadline()).is_err()
    );
}
#[test]
fn overlong_probe_usage_refused() {
    let mut f = Fixture::new(Class::Daemon);
    let mut input = facts(Class::Daemon, UsefulResult::Passed);
    input.usage_ms = Some(60_001);
    assert!(
        service::record_probe(&mut f.store, &principal(), &f.profile, &input, deadline()).is_err()
    );
}
#[test]
fn impossible_exit_code_refused() {
    let mut f = Fixture::new(Class::OneShot);
    let mut input = facts(Class::OneShot, UsefulResult::Passed);
    input.facts = Facts::OneShot {
        exit_code: Some(-1),
        output: UsefulResult::Passed,
    };
    assert!(
        service::record_probe(&mut f.store, &principal(), &f.profile, &input, deadline()).is_err()
    );
}
#[test]
fn expired_deadline_cannot_publish_observation() {
    let mut f = Fixture::new(Class::Daemon);
    let expired = Instant::now();
    assert!(
        service::record_probe(
            &mut f.store,
            &principal(),
            &f.profile,
            &facts(Class::Daemon, UsefulResult::Passed),
            expired
        )
        .is_err()
    );
    assert!(
        f.store
            .roster_get(&principal(), uuid(&f.profile.record_id), deadline())
            .unwrap()
            .observation
            .is_none()
    );
}
#[test]
fn ttl_must_be_bounded_and_nonzero() {
    let f = Fixture::new(Class::Daemon);
    for ttl in [0, 60_001] {
        assert!(matches!(
            service::inspect(
                &f.store,
                &principal(),
                uuid(&f.profile.record_id),
                std::slice::from_ref(&f.profile),
                None,
                ttl,
                deadline()
            ),
            Err(Error::Bounds)
        ));
    }
}
#[test]
fn profile_inventory_bound_is_enforced() {
    let f = Fixture::new(Class::Daemon);
    let profiles = vec![f.profile.clone(); 257];
    assert!(matches!(
        service::inspect(
            &f.store,
            &principal(),
            uuid(&f.profile.record_id),
            &profiles,
            None,
            60_000,
            deadline()
        ),
        Err(Error::Bounds)
    ));
}
#[test]
fn wrong_roster_kind_refuses_service_inspection() {
    let mut f = Fixture::new(Class::Daemon);
    let mut update = f.update(2);
    update.definition.kind = Kind::Agent;
    apply(&mut f.store, &update);
    assert!(matches!(
        service::inspect(
            &f.store,
            &principal(),
            uuid(&f.profile.record_id),
            std::slice::from_ref(&f.profile),
            None,
            60_000,
            deadline()
        ),
        Err(Error::WrongKind)
    ));
}
