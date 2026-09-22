use habitat_engine::app::evidence::Evidence;
use habitat_engine::contracts::roster::{Kind, Locality, Selection};
use habitat_engine::contracts::{Generation, Sha256Digest, UuidV4};
use habitat_engine::roster::parse_changes;
use habitat_engine::store::{
    Allocation, ArtifactStaging, Effect, Expected, Principal, RequestSource, RosterStart,
    Settlement, Store, Submission,
};
use hee3_fixed_task_runtime::native_executor_observation;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs::{self, DirBuilder, File};
use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const STORE_GENERATION: &str = "71000000-0000-4000-8000-000000000001";
const STORE_EPOCH: &str = "71000000-0000-4000-8000-000000000002";
const ROSTER_KEY: &str = "71000000-0000-4000-8000-000000000003";
const TASK_KEY: &str = "71000000-0000-4000-8000-000000000004";
const TASK: &str = "71000000-0000-4000-8000-000000000005";
const ADMITTED: &str = "71000000-0000-4000-8000-000000000006";
const ATTEMPT: &str = "71000000-0000-4000-8000-000000000007";
const STARTED: &str = "71000000-0000-4000-8000-000000000008";
const SESSION: &str = "71000000-0000-4000-8000-000000000009";
const WORKSPACE: &str = "71000000-0000-4000-8000-00000000000a";
const SETTLED: &str = "71000000-0000-4000-8000-00000000000b";
const CRITERIA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Area {
    root: PathBuf,
    staging: PathBuf,
    device: u64,
    inode: u64,
}
impl Area {
    fn new() -> Self {
        let container = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "hee3-native-store-controls-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        private(&container);
        let root = container.join("store");
        let staging = container.join("staging");
        private(&root);
        private(&staging);
        let metadata = fs::symlink_metadata(&container).unwrap();
        Self {
            root,
            staging,
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
    fn store(&self) -> Store {
        Store::open(
            &self.root,
            uuid(STORE_GENERATION),
            uuid(STORE_EPOCH),
            true,
            deadline(),
        )
        .unwrap()
    }
    fn staging(&self) -> ArtifactStaging {
        ArtifactStaging::open(&self.staging, true, deadline()).unwrap()
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        let container = self.root.parent().unwrap();
        let metadata = fs::symlink_metadata(container).unwrap();
        assert_eq!((metadata.dev(), metadata.ino()), (self.device, self.inode));
        fs::remove_dir_all(container).unwrap();
    }
}

fn private(path: &Path) {
    DirBuilder::new().mode(0o700).create(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
fn uuid(value: &str) -> UuidV4<'_> {
    UuidV4::parse(value).unwrap()
}
fn generation(value: &str) -> Generation {
    value.parse().unwrap()
}
fn principal() -> Principal {
    Principal::new(1000, "operator").unwrap()
}
fn wrong_principal() -> Principal {
    Principal::new(1001, "operator").unwrap()
}
fn executable_sha() -> String {
    let mut file = File::open(std::env::current_exe().unwrap()).unwrap();
    let mut buffer = vec![0_u8; 64 * 1024];
    let mut digest = Sha256::new();
    loop {
        let count = file.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let mut output = String::from("sha256:");
    for byte in digest.finalize() {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}
fn manifest(kind: Kind, locality: Locality, capabilities: &[&str]) -> Vec<u8> {
    let kind = match kind {
        Kind::Agent => "agent",
        Kind::Model => "model",
        Kind::Service => "service",
    };
    let locality = match locality {
        Locality::Local => "local",
        Locality::Remote => "remote",
        Locality::Hybrid => "hybrid",
    };
    let capabilities = capabilities
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "kind = \"hee3-roster-changes\"\nversion = 1\n\n[[updates]]\nidempotency_key = \"{ROSTER_KEY}\"\naudit_reason = \"fixed native test owner\"\n[updates.definition]\nkind = \"{kind}\"\ndisplay_name = \"fixed native executor\"\nowner_id = \"fixed-runtime-owner\"\nversion = \"rust-1.98-fixed\"\ncapabilities = [{capabilities}]\nlocality = \"{locality}\"\nlimitations = \"offline fixed workload only\"\n"
    )
    .into_bytes()
}
fn roster(
    store: &mut Store,
    kind: Kind,
    locality: Locality,
    capabilities: &[&str],
) -> habitat_engine::contracts::roster::RosterHeadV1 {
    let bytes = manifest(kind, locality, capabilities);
    let changes = parse_changes(&bytes).unwrap();
    store
        .roster_apply(
            &principal(),
            &changes,
            RequestSource::Import(&bytes),
            deadline(),
        )
        .unwrap()
        .remove(0)
        .head
}
fn selection(head: &habitat_engine::contracts::roster::RosterHeadV1) -> Selection {
    Selection {
        record_id: head.record_id.clone(),
        expected_revision: head.record_version.clone(),
        capabilities: vec!["u64-fixed-workload".to_owned()],
        local_only: true,
        version: Some(head.definition.version.clone()),
        ttl_ms: 60_000,
    }
}
fn admit(store: &mut Store) {
    store
        .submit(
            Submission {
                principal: &principal(),
                key: uuid(TASK_KEY),
                task: uuid(TASK),
                event: uuid(ADMITTED),
                request_bytes: b"fixed native Store control",
                criteria: Sha256Digest::parse(CRITERIA).unwrap(),
                allocation: Allocation {
                    limit_ms: 1_200_000,
                    work_ms: 900_000,
                    verify_ms: 300_000,
                },
            },
            deadline(),
        )
        .unwrap();
}
fn begin(
    store: &mut Store,
    head: &habitat_engine::contracts::roster::RosterHeadV1,
    selected: &Selection,
) -> Result<habitat_engine::store::RosterAttempt, habitat_engine::store::Error> {
    store.begin_rostered_attempt(
        RosterStart {
            principal: &principal(),
            task: uuid(TASK),
            expected: generation("1"),
            attempt: uuid(ATTEMPT),
            event: uuid(STARTED),
            agent_record_id: &head.record_id,
            session: uuid(SESSION),
            workspace: uuid(WORKSPACE),
            selections: std::slice::from_ref(selected),
            lease_ms: 1_200_000,
        },
        deadline(),
    )
}
fn no_outbox(store: &Store) {
    assert!(store.pending_delivery(256, deadline()).unwrap().is_empty());
}

#[test]
fn confirmed_worker_evidence_is_read_back_and_pinned_by_rostered_begin() {
    let area = Area::new();
    let mut store = area.store();
    let staging = area.staging();
    let head = roster(
        &mut store,
        Kind::Agent,
        Locality::Local,
        &["u64-fixed-workload"],
    );
    let selected = selection(&head);
    admit(&mut store);
    let mut evidence = Evidence::staged(&staging, deadline());
    let observed = native_executor_observation::observe(
        &mut store,
        &principal(),
        &mut evidence,
        &selected,
        &std::env::current_exe().unwrap(),
        &executable_sha(),
        deadline(),
    )
    .unwrap();
    assert_eq!(
        staging.read_object(&observed.object, deadline()).unwrap(),
        observed.raw_bytes
    );
    assert_eq!(
        observed.evidence.artifact_id.as_str(),
        observed.observation.input.evidence_ref
    );
    assert_eq!(
        observed.observation.confirmed_source,
        Some(habitat_engine::contracts::roster::ObservationSource::Worker)
    );
    let started = begin(&mut store, &head, &selected).unwrap();
    assert_eq!(started.pins.len(), 1);
    assert_eq!(
        started.pins[0].record.observation.as_ref().unwrap(),
        &observed.observation
    );
    no_outbox(&store);
}

#[test]
fn wrong_authority_definition_selection_and_executable_never_begin() {
    for (kind, locality, capabilities) in [
        (Kind::Model, Locality::Local, vec!["u64-fixed-workload"]),
        (Kind::Agent, Locality::Remote, vec!["u64-fixed-workload"]),
        (Kind::Agent, Locality::Local, vec!["different-capability"]),
    ] {
        let area = Area::new();
        let mut store = area.store();
        let staging = area.staging();
        let head = roster(&mut store, kind, locality, &capabilities);
        let selected = selection(&head);
        admit(&mut store);
        let mut evidence = Evidence::staged(&staging, deadline());
        assert!(
            native_executor_observation::observe(
                &mut store,
                &principal(),
                &mut evidence,
                &selected,
                &std::env::current_exe().unwrap(),
                &executable_sha(),
                deadline()
            )
            .is_err()
        );
        assert!(begin(&mut store, &head, &selected).is_err());
        no_outbox(&store);
    }

    let area = Area::new();
    let mut store = area.store();
    let staging = area.staging();
    let head = roster(
        &mut store,
        Kind::Agent,
        Locality::Local,
        &["u64-fixed-workload"],
    );
    let mut selected = selection(&head);
    admit(&mut store);
    let mut evidence = Evidence::staged(&staging, deadline());
    assert!(
        native_executor_observation::observe(
            &mut store,
            &wrong_principal(),
            &mut evidence,
            &selected,
            &std::env::current_exe().unwrap(),
            &executable_sha(),
            deadline()
        )
        .is_err()
    );
    selected.expected_revision = "2".to_owned();
    assert!(
        native_executor_observation::observe(
            &mut store,
            &principal(),
            &mut evidence,
            &selected,
            &std::env::current_exe().unwrap(),
            &executable_sha(),
            deadline()
        )
        .is_err()
    );
    selected.expected_revision = head.record_version.clone();
    assert!(
        native_executor_observation::observe(
            &mut store,
            &principal(),
            &mut evidence,
            &selected,
            Path::new("/usr/bin/true"),
            &executable_sha(),
            deadline()
        )
        .is_err()
    );
    assert!(
        store
            .roster_get(&principal(), uuid(&head.record_id), deadline())
            .unwrap()
            .observation
            .is_none()
    );
    assert!(begin(&mut store, &head, &selected).is_err());
    no_outbox(&store);
}

#[test]
fn failed_publication_and_repeated_post_settlement_observation_keep_exact_custody() {
    let area = Area::new();
    let mut store = area.store();
    let staging = area.staging();
    let head = roster(
        &mut store,
        Kind::Agent,
        Locality::Local,
        &["u64-fixed-workload"],
    );
    let selected = selection(&head);
    admit(&mut store);
    let expired = Instant::now().checked_sub(Duration::from_nanos(1)).unwrap();
    let mut unavailable = Evidence::staged(&staging, expired);
    assert!(
        native_executor_observation::observe(
            &mut store,
            &principal(),
            &mut unavailable,
            &selected,
            &std::env::current_exe().unwrap(),
            &executable_sha(),
            deadline()
        )
        .is_err()
    );
    assert!(begin(&mut store, &head, &selected).is_err());

    let mut evidence = Evidence::staged(&staging, deadline());
    let first = native_executor_observation::observe(
        &mut store,
        &principal(),
        &mut evidence,
        &selected,
        &std::env::current_exe().unwrap(),
        &executable_sha(),
        deadline(),
    )
    .unwrap();
    let started = begin(&mut store, &head, &selected).unwrap();
    store
        .settle_attempt(
            &Expected {
                task: uuid(TASK),
                task_generation: generation("2"),
                attempt: uuid(ATTEMPT),
                attempt_generation: generation("1"),
            },
            Settlement {
                effect: Effect::None,
                used_ms: Some(1),
                cleanup_settled: true,
                ready_to_verify: true,
            },
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    assert_eq!(started.attempt.id, ATTEMPT);
    let second = native_executor_observation::observe(
        &mut store,
        &principal(),
        &mut evidence,
        &selected,
        &std::env::current_exe().unwrap(),
        &executable_sha(),
        deadline(),
    )
    .unwrap();
    assert_ne!(first.observation.id, second.observation.id);
    assert_ne!(first.evidence.artifact_id, second.evidence.artifact_id);
    assert_eq!(
        staging.read_object(&second.object, deadline()).unwrap(),
        second.raw_bytes
    );
    no_outbox(&store);
}
