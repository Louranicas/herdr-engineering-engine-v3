//! Real Store preparation/revalidation controls; no process, grant or dispatch.
use habitat_engine::contracts::{
    UuidV4,
    roster::{ActiveAttemptPolicy, Disable, Kind, Locality, RosterDefinitionV1, Update},
};
use habitat_engine::service::{self, Class, Error, ProbeError, ProbePreparation, Profile};
use habitat_engine::store::{Principal, RequestSource, Store};
use serde::Serialize;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
const GEN: &str = "10000000-0000-4000-8000-000000000001";
const EPOCH: &str = "10000000-0000-4000-8000-000000000002";
const ENDPOINT: &str = "10000000-0000-4000-8000-000000000003";
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
        }
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
fn body(f: &Fixture) -> Vec<u8> {
    format!(r#"{{"service_id":"{}","probe_id":"useful-check","probe_version":1,"max_cost_microunits":"0","network_scope":"none"}}"#, f.profile.record_id).into_bytes()
}
fn prepare(f: &Fixture, raw: &[u8]) -> Result<ProbePreparation, ProbeError> {
    let start = Instant::now();
    service::prepare_probe(
        &f.store,
        &principal(),
        raw,
        std::slice::from_ref(&f.profile),
        start,
        start + Duration::from_secs(5),
        &AtomicBool::new(false),
    )
}
fn revalidate(f: &Fixture, p: &ProbePreparation) -> Result<(), ProbeError> {
    p.revalidate(
        &f.store,
        &principal(),
        std::slice::from_ref(&f.profile),
        &AtomicBool::new(false),
    )
}
#[test]
fn actual_local_preparation_and_revalidation_preserve_exact_request_without_effects() {
    let f = Fixture::new(Class::OneShot);
    let raw = body(&f);
    let before = f.store.roster_snapshot(&principal(), deadline()).unwrap();
    let p = prepare(&f, &raw).unwrap();
    assert_eq!(p.raw_body(), raw);
    assert_eq!(p.profile(), &f.profile);
    assert_eq!(p.requested_cost_ceiling(), 0);
    revalidate(&f, &p).unwrap();
    let after = f.store.roster_snapshot(&principal(), deadline()).unwrap();
    assert_eq!(before.cutoff, after.cutoff);
    assert_eq!(before.records[0].head, after.records[0].head);
    assert!(after.records[0].observation.is_none());
}
#[test]
fn equivalent_json_keeps_distinct_original_bytes_and_hashes() {
    let f = Fixture::new(Class::Library);
    let raw = body(&f);
    let mut padded = b" \n".to_vec();
    padded.extend_from_slice(&raw);
    padded.extend_from_slice(b"\t ");
    let a = prepare(&f, &raw).unwrap();
    let b = prepare(&f, &padded).unwrap();
    assert_ne!(a.body_sha256(), b.body_sha256());
    assert_eq!(b.raw_body(), padded);
    revalidate(&f, &b).unwrap();
}
#[test]
fn all_local_classes_prepare_without_claiming_useful_health() {
    for class in [
        Class::Daemon,
        Class::OneShot,
        Class::Library,
        Class::NeuralOperator,
    ] {
        let f = Fixture::new(class);
        let p = prepare(&f, &body(&f)).unwrap();
        assert_eq!(p.profile().class, class);
        revalidate(&f, &p).unwrap();
    }
}
#[test]
fn requested_cost_ceiling_never_changes_the_offline_profile() {
    let f = Fixture::new(Class::OneShot);
    let raw = String::from_utf8(body(&f))
        .unwrap()
        .replace("\"0\"", "\"18446744073709551615\"");
    let p = prepare(&f, raw.as_bytes()).unwrap();
    assert_eq!(p.requested_cost_ceiling(), u64::MAX);
    assert_eq!(p.profile(), &f.profile);
}
#[test]
fn closed_body_refuses_duplicate_unknown_missing_and_invalid_fields_without_events() {
    let f = Fixture::new(Class::OneShot);
    let raw = String::from_utf8(body(&f)).unwrap();
    let before = f
        .store
        .roster_snapshot(&principal(), deadline())
        .unwrap()
        .cutoff;
    let faults = [
        raw.replace(
            "\"probe_version\":1",
            "\"probe_version\":1,\"probe_version\":1",
        ),
        raw.replace("\"probe_version\":1", "\"probe_version\":2"),
        raw.replace("\"probe_version\":1", "\"probe_version\":1.0"),
        raw.replace("\"probe_version\":1,", ""),
        raw.replace("\"0\"", "\"00\""),
        raw.replace("\"0\"", "0"),
        raw.replace("\"0\"", "\"18446744073709551616\""),
        raw.replace("\"none\"", "\"invented\""),
        raw.replace("\"useful-check\"", "\"bad\\ud800\""),
        raw.replace("\"probe_version\":1", "\"probe_version\":1,\"grant\":true"),
    ];
    for fault in faults {
        assert!(
            matches!(
                prepare(&f, fault.as_bytes()),
                Err(ProbeError::InvalidRequest)
            ),
            "{fault}"
        );
    }
    assert_eq!(
        before,
        f.store
            .roster_snapshot(&principal(), deadline())
            .unwrap()
            .cutoff
    );
}
#[test]
fn empty_oversize_and_invalid_utf8_refuse_before_store_effects() {
    let f = Fixture::new(Class::OneShot);
    assert!(matches!(prepare(&f, b""), Err(ProbeError::Bounds)));
    assert!(matches!(
        prepare(&f, &vec![b' '; 1_048_577]),
        Err(ProbeError::Bounds)
    ));
    assert!(matches!(
        prepare(&f, b"\xff"),
        Err(ProbeError::InvalidRequest)
    ));
}
#[test]
fn remote_and_hybrid_targets_and_network_grants_remain_unavailable() {
    let f = Fixture::new(Class::RemoteEndpoint);
    assert!(matches!(
        prepare(&f, &body(&f)),
        Err(ProbeError::OfflineUnavailable)
    ));
    let mut f = Fixture::new(Class::Daemon);
    let raw = String::from_utf8(body(&f))
        .unwrap()
        .replace("\"none\"", "\"configured_allowlist\"");
    assert!(matches!(
        prepare(&f, raw.as_bytes()),
        Err(ProbeError::OfflineUnavailable)
    ));
    let mut change = f.update(2);
    change.definition.locality = Locality::Hybrid;
    let h = apply(&mut f.store, &change);
    f.profile.record_version = h.record_version;
    assert!(matches!(
        prepare(&f, &body(&f)),
        Err(ProbeError::OfflineUnavailable)
    ));
}
#[test]
fn missing_ambiguous_and_unbounded_profile_sets_refuse() {
    let f = Fixture::new(Class::OneShot);
    let start = Instant::now();
    let raw = body(&f);
    let stop = AtomicBool::new(false);
    for (profiles, expected) in [
        (vec![], 0),
        (vec![f.profile.clone(); 2], 1),
        (vec![f.profile.clone(); 257], 2),
    ] {
        let result = service::prepare_probe(
            &f.store,
            &principal(),
            &raw,
            &profiles,
            start,
            start + Duration::from_secs(5),
            &stop,
        );
        assert!(matches!(
            (expected, result),
            (0, Err(ProbeError::MissingProfile))
                | (1, Err(ProbeError::AmbiguousProfile))
                | (2, Err(ProbeError::Bounds))
        ));
    }
}
#[test]
fn wrong_probe_recipe_and_owner_binding_refuse() {
    let mut f = Fixture::new(Class::OneShot);
    let raw = body(&f);
    f.profile.probe_id = "another".into();
    assert!(matches!(prepare(&f, &raw), Err(ProbeError::Changed)));
    f.profile.probe_id = "useful-check".into();
    f.profile.owner_id = "foreign-owner".into();
    assert!(matches!(
        prepare(&f, &raw),
        Err(ProbeError::Service(Error::Binding))
    ));
}
#[test]
fn admitted_update_between_read_and_revalidation_refuses_old_subject() {
    let mut f = Fixture::new(Class::OneShot);
    let p = prepare(&f, &body(&f)).unwrap();
    let change = f.update(2);
    let h = apply(&mut f.store, &change);
    assert!(revalidate(&f, &p).is_err());
    f.profile.record_version = h.record_version;
    assert!(matches!(revalidate(&f, &p), Err(ProbeError::Changed)));
    let current = prepare(&f, &body(&f)).unwrap();
    revalidate(&f, &current).unwrap();
}
#[test]
fn admitted_disable_after_preparation_refuses_without_dispatch_or_observation() {
    let mut f = Fixture::new(Class::OneShot);
    let p = prepare(&f, &body(&f)).unwrap();
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
    assert!(revalidate(&f, &p).is_err());
    assert!(matches!(
        prepare(&f, &body(&f)),
        Err(ProbeError::Service(Error::Binding | Error::Disabled))
    ));
    assert!(
        f.store
            .roster_snapshot(&principal(), deadline())
            .unwrap()
            .records[0]
            .observation
            .is_none()
    );
}
#[test]
fn changed_protected_identity_or_class_refuses_with_unchanged_roster() {
    let mut f = Fixture::new(Class::OneShot);
    let p = prepare(&f, &body(&f)).unwrap();
    f.profile.immutable_revision = "changed".into();
    assert!(matches!(revalidate(&f, &p), Err(ProbeError::Changed)));
    f.profile = p.profile().clone();
    f.profile.class = Class::Library;
    assert!(matches!(revalidate(&f, &p), Err(ProbeError::Changed)));
}
#[test]
fn receiver_restart_invalidates_old_preparation_but_allows_fresh_preparation() {
    let mut f = Fixture::new(Class::OneShot);
    let p = prepare(&f, &body(&f)).unwrap();
    drop(f.store);
    f.store = f.root.open(false);
    assert!(matches!(revalidate(&f, &p), Err(ProbeError::Changed)));
    let fresh = prepare(&f, &body(&f)).unwrap();
    revalidate(&f, &fresh).unwrap();
}
#[test]
fn other_principal_cannot_reuse_visible_preparation() {
    let f = Fixture::new(Class::OneShot);
    let p = prepare(&f, &body(&f)).unwrap();
    let other = Principal::new(1001, "operator").unwrap();
    assert!(
        p.revalidate(
            &f.store,
            &other,
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(false)
        )
        .is_err()
    );
}
#[test]
fn cancellation_before_prepare_and_before_revalidate_is_explicit() {
    let f = Fixture::new(Class::OneShot);
    let start = Instant::now();
    assert!(matches!(
        service::prepare_probe(
            &f.store,
            &principal(),
            &body(&f),
            std::slice::from_ref(&f.profile),
            start,
            start + Duration::from_secs(5),
            &AtomicBool::new(true)
        ),
        Err(ProbeError::Cancelled)
    ));
    let p = prepare(&f, &body(&f)).unwrap();
    assert!(matches!(
        p.revalidate(
            &f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(true)
        ),
        Err(ProbeError::Cancelled)
    ));
}
#[test]
fn expired_future_or_overlong_original_window_refuses() {
    let f = Fixture::new(Class::OneShot);
    let now = Instant::now();
    let raw = body(&f);
    let stop = AtomicBool::new(false);
    for (start, end) in [
        (
            now.checked_sub(Duration::from_secs(2)).unwrap(),
            now.checked_sub(Duration::from_secs(1)).unwrap(),
        ),
        (now + Duration::from_secs(1), now + Duration::from_secs(2)),
        (now, now + Duration::from_secs(61)),
    ] {
        assert!(
            service::prepare_probe(
                &f.store,
                &principal(),
                &raw,
                std::slice::from_ref(&f.profile),
                start,
                end,
                &stop
            )
            .is_err()
        );
    }
}
#[test]
fn original_deadline_is_not_renewed_by_later_revalidation() {
    let f = Fixture::new(Class::OneShot);
    let start = Instant::now();
    let end = start + Duration::from_secs(1);
    let p = service::prepare_probe(
        &f.store,
        &principal(),
        &body(&f),
        std::slice::from_ref(&f.profile),
        start,
        end,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(p.original_deadline(), end);
    std::thread::sleep(Duration::from_millis(1050));
    assert!(matches!(revalidate(&f, &p), Err(ProbeError::Deadline)));
}

#[test]
fn fresh_unconfirmed_observation_does_not_change_admitted_authority() {
    use habitat_engine::contracts::roster::{Availability, ObservationInput, ObservationSource};
    let mut f = Fixture::new(Class::Daemon);
    let p = prepare(&f, &body(&f)).unwrap();
    let observed = f
        .store
        .roster_observe(
            &principal(),
            &ObservationInput {
                record_id: f.profile.record_id.clone(),
                record_version: f.profile.record_version.clone(),
                owner_id: f.profile.owner_id.clone(),
                endpoint_ref: f.profile.endpoint_ref.clone(),
                instance_id: None,
                instance_generation: None,
                source: ObservationSource::Requested,
                observed_unix_ms: None,
                availability: Availability::Unknown,
                actual_identity: None,
                immutable_revision: None,
                capabilities: vec![],
                evidence_ref: key(9),
            },
            deadline(),
        )
        .unwrap();
    assert_eq!(observed.input.availability, Availability::Unknown);
    revalidate(&f, &p).unwrap();
}
