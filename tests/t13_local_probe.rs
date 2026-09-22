//! Actual disposable local probes; no lifecycle or network service effects.
use habitat_engine::contracts::{
    UuidV4,
    roster::{ActiveAttemptPolicy, Disable, Kind, Locality, RosterDefinitionV1, Update},
};
use habitat_engine::service::{
    self, Class, Health, LocalProbe, LocalProbeError, LocalRecipe, ProbeError, Profile,
};
use habitat_engine::store::{Principal, RequestSource, Store};
use habitat_engine::worker::process::Interruption;
use serde::Serialize;
use sha2::{Digest, Sha256};
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
            if std::thread::panicking() {
                return;
            }
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
fn digest(path: &std::path::Path) -> String {
    let mut value = String::from("sha256:");
    for byte in Sha256::digest(fs::read(path).unwrap()) {
        use std::fmt::Write as _;
        write!(value, "{byte:02x}").unwrap();
    }
    value
}
fn recipe(f: &mut Fixture, executable: &str, args: &[&str]) -> LocalRecipe {
    let executable = std::path::Path::new(executable).canonicalize().unwrap();
    let sha = digest(&executable);
    f.profile.actual_identity = executable.to_str().unwrap().into();
    f.profile.immutable_revision.clone_from(&sha);
    LocalRecipe {
        executable,
        executable_sha256: sha,
        arguments: args.iter().map(Into::into).collect(),
        expected_stdout: b"useful\n".to_vec(),
        directory: f.root.path.clone(),
    }
}
fn execute(f: &Fixture, r: &LocalRecipe, cancel: &AtomicBool, seconds: u64) -> LocalProbe {
    let start = Instant::now();
    let prepared = service::prepare_probe(
        &f.store,
        &principal(),
        &body(f),
        std::slice::from_ref(&f.profile),
        start,
        start + Duration::from_secs(seconds),
        &AtomicBool::new(false),
    )
    .unwrap();
    let result = service::run_local_probe(
        &f.store,
        &principal(),
        prepared,
        std::slice::from_ref(&f.profile),
        r,
        cancel,
        uuid(&key(900)),
    );
    retain(&result);
    result
}
fn retain(r: &LocalProbe) {
    let root = PathBuf::from(std::env::var_os("T13_PROCESS_EVIDENCE").unwrap());
    let name = std::thread::current()
        .name()
        .unwrap_or("unknown")
        .to_string();
    let out = root.join(format!("{}-{}", name, NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("observation.txt"), format!("{r:#?}")).unwrap();
    if let Some(p) = &r.process {
        fs::write(out.join("stdout"), &p.stdout.bytes).unwrap();
        fs::write(out.join("stderr"), &p.stderr.bytes).unwrap();
    }
}
fn settled(r: &LocalProbe) {
    let p = r.process.as_ref().unwrap();
    assert!(
        p.leader_reaped
            && p.process_group_settled
            && p.pending.is_none()
            && p.stdout.eof
            && p.stderr.eof
    );
    assert!(r.cpu_elapsed.is_none());
}
fn published(f: &mut Fixture, r: &mut LocalProbe) -> Health {
    let receipt = r
        .record(
            &mut f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(false),
        )
        .unwrap();
    let inspected = service::inspect(
        &f.store,
        &principal(),
        uuid(&f.profile.record_id),
        std::slice::from_ref(&f.profile),
        Some(receipt),
        60_000,
        deadline(),
    )
    .unwrap();
    let health = inspected.health;
    retain(r);
    health
}
#[test]
fn real_useful_return_publishes_without_changing_admission() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["%s\n", "useful"]);
    let before = f
        .store
        .roster_get(&principal(), uuid(&f.profile.record_id), deadline())
        .unwrap()
        .head;
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    settled(&r);
    assert!(r.error.is_none());
    assert_eq!(
        r.facts.as_ref().unwrap().usage_ms,
        Some(
            u64::try_from(
                r.process
                    .as_ref()
                    .unwrap()
                    .elapsed
                    .as_nanos()
                    .div_ceil(1_000_000)
            )
            .unwrap()
        )
    );
    assert_eq!(r.facts.as_ref().unwrap().cost_microunits, Some(0));
    assert_eq!(published(&mut f, &mut r), Health::Useful);
    let after = f
        .store
        .roster_get(&principal(), uuid(&f.profile.record_id), deadline())
        .unwrap()
        .head;
    assert_eq!(before.definition, after.definition);
    assert_eq!(before.record_version, after.record_version);
    assert!(!after.disabled);
}
#[test]
fn real_zero_exit_useless_output_is_unavailable() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["%s\n", "wrong"]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    settled(&r);
    assert_eq!(r.process.as_ref().unwrap().exit_code, Some(0));
    assert_eq!(published(&mut f, &mut r), Health::Unavailable);
}
#[test]
fn real_nonzero_cannot_be_useful() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/false", &[]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    settled(&r);
    assert_eq!(r.process.as_ref().unwrap().exit_code, Some(1));
    assert_eq!(published(&mut f, &mut r), Health::Unavailable);
}
#[test]
fn wrong_digest_never_launches() {
    let mut f = Fixture::new(Class::OneShot);
    let mut recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    recipe.executable_sha256 = "sha256:wrong".into();
    let r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    assert!(r.process.is_none());
    assert!(matches!(r.error, Some(LocalProbeError::Recipe)));
}
#[test]
fn wrong_class_never_launches() {
    let mut f = Fixture::new(Class::Daemon);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    let r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    assert!(r.process.is_none());
    assert!(matches!(r.error, Some(LocalProbeError::Recipe)));
}
#[test]
fn precancel_has_no_child_or_observation() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(true), 60);
    assert!(r.process.is_none());
    assert!(matches!(
        r.error,
        Some(LocalProbeError::Preparation(ProbeError::Cancelled))
    ));
    assert!(
        r.record(
            &mut f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(false)
        )
        .is_err()
    );
}
#[test]
fn cancellation_during_child_retains_clean_custody_without_observation() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/sleep", &["20"]);
    let cancel = AtomicBool::new(false);
    let mut r = std::thread::scope(|scope| {
        scope.spawn(|| {
            std::thread::sleep(Duration::from_millis(150));
            cancel.store(true, Ordering::Release);
        });
        execute(&f, &recipe, &cancel, 60)
    });
    settled(&r);
    assert!(matches!(
        r.error,
        Some(LocalProbeError::Interrupted(Interruption::Cancelled))
    ));
    assert!(
        r.record(
            &mut f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &cancel
        )
        .is_err()
    );
}
#[test]
fn timeout_uses_original_horizon_and_keeps_cleanup_reserve() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/sleep", &["20"]);
    let r = execute(&f, &recipe, &AtomicBool::new(false), 11);
    settled(&r);
    assert!(matches!(
        r.error,
        Some(LocalProbeError::Interrupted(Interruption::Timeout))
    ));
    assert!(r.wall_elapsed < Duration::from_secs(11));
}
#[test]
fn actual_return_after_update_cannot_refresh_new_revision() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    let mut update = f.update(2);
    update.definition.version = "service-v2".into();
    apply(&mut f.store, &update);
    assert!(
        r.record(
            &mut f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(false)
        )
        .is_err()
    );
    assert!(r.process.is_some());
    assert!(
        f.store
            .roster_get(&principal(), uuid(&f.profile.record_id), deadline())
            .unwrap()
            .observation
            .is_none()
    );
}
#[test]
fn actual_return_after_disable_cannot_revive_service() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    let disable = Disable {
        record_id: f.profile.record_id.clone(),
        expected_revision: f.profile.record_version.clone(),
        idempotency_key: key(2),
        audit_reason: "disable after return".into(),
        active_attempt_policy: ActiveAttemptPolicy::LeaveRunning,
    };
    let bytes=serde_json::to_vec(&serde_json::json!({"action":"roster.disable","record_id":disable.record_id,"expected_revision":disable.expected_revision,"idempotency_key":disable.idempotency_key,"active_attempt_policy":disable.active_attempt_policy,"audit_reason":disable.audit_reason})).unwrap();
    f.store
        .roster_disable(&principal(), &disable, &bytes, deadline())
        .unwrap();
    assert!(
        r.record(
            &mut f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(false)
        )
        .is_err()
    );
    assert!(r.process.is_some());
}
#[test]
fn actual_return_after_restart_is_stale() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    drop(f.store);
    f.store = f.root.open(false);
    assert!(matches!(
        r.record(
            &mut f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(false)
        ),
        Err(LocalProbeError::Preparation(ProbeError::Changed))
    ));
    assert!(r.process.is_some());
    let mut fresh = execute(&f, &recipe, &AtomicBool::new(false), 60);
    assert_eq!(published(&mut f, &mut fresh), Health::Useful);
}
#[test]
fn cancellation_after_return_does_not_publish_new_health() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    assert!(matches!(
        r.record(
            &mut f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(true)
        ),
        Err(LocalProbeError::Preparation(ProbeError::Cancelled))
    ));
    settled(&r);
}
#[test]
fn local_value_cannot_publish_twice() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    assert_eq!(published(&mut f, &mut r), Health::Useful);
    assert!(matches!(
        r.record(
            &mut f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(false)
        ),
        Err(LocalProbeError::AlreadyRecorded)
    ));
}
#[test]
fn collected_return_does_not_gain_freshness_from_late_publication() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    std::thread::sleep(Duration::from_millis(150));
    let receipt = r
        .record(
            &mut f.store,
            &principal(),
            std::slice::from_ref(&f.profile),
            &AtomicBool::new(false),
        )
        .unwrap();
    let inspected = service::inspect(
        &f.store,
        &principal(),
        uuid(&f.profile.record_id),
        std::slice::from_ref(&f.profile),
        Some(receipt),
        50,
        deadline(),
    )
    .unwrap();
    assert_eq!(inspected.health, Health::Unknown);
}

#[test]
fn unsupported_executable_is_not_a_generic_executor() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/true", &[]);
    let r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    assert!(r.process.is_none());
    assert!(matches!(r.error, Some(LocalProbeError::Recipe)));
}
#[test]
fn invalid_import_cannot_replace_actual_return_authority() {
    let mut f = Fixture::new(Class::OneShot);
    let recipe = recipe(&mut f, "/usr/bin/printf", &["useful\n"]);
    let mut r = execute(&f, &recipe, &AtomicBool::new(false), 60);
    let mut invalid = f.update(2);
    invalid.expected_revision = Some("900".into());
    let raw = manifest(&invalid);
    assert!(
        f.store
            .roster_apply(
                &principal(),
                &[invalid],
                RequestSource::Import(&raw),
                deadline()
            )
            .is_err()
    );
    assert_eq!(published(&mut f, &mut r), Health::Useful);
}
