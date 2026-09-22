//! Black-box Store-backed evidence sink oracles. Design frozen before bodies.
//! Public API only; app/evidence.rs was not read. Fixtures supply bytes, not
//! authenticated producer evidence. Each completed filesystem fault is private.

use habitat_engine::app::evidence::{self, Evidence};
use habitat_engine::check::collector::{Publisher, Sink};
use habitat_engine::check::graph::{Graph, Objects};
use habitat_engine::contracts::UuidV4;
use habitat_engine::contracts::receipt::{
    self, Id, List, Maybe, Name, Ref, ResourcePageV1, Sha, Text,
};
use habitat_engine::store::{Error as StoreError, Object, Store};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs::{self, DirBuilder};
use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const GEN: &str = "06700000-0000-4000-8000-000000000001";
const EPOCH: &str = "06700000-0000-4000-8000-000000000002";
const STAGE: &str = "06700000-0000-4000-8000-000000000003";
const ABC_SHA: &str = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
static NEXT_AREA: AtomicU64 = AtomicU64::new(0);

struct Area {
    path: PathBuf,
    device: u64,
    inode: u64,
}
impl Area {
    fn new() -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "hee3-t06-evidence-{}-{}",
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
        Store::open(
            &self.path,
            UuidV4::parse(GEN).unwrap(),
            UuidV4::parse(EPOCH).unwrap(),
            create,
            deadline(),
        )
        .unwrap()
    }
    fn object_path(&self, digest: &str) -> PathBuf {
        let hex = &digest[7..];
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
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
fn expired() -> Instant {
    Instant::now().checked_sub(Duration::from_secs(1)).unwrap()
}
fn id(ordinal: u32) -> Id {
    Id::new(format!("06700000-0000-4000-9000-{ordinal:012x}")).unwrap()
}
fn digest(bytes: &[u8]) -> String {
    let mut value = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(value, "{byte:02x}").unwrap();
    }
    value
}
fn reference(ordinal: u32, bytes: &[u8]) -> Ref {
    Ref {
        artifact_id: id(ordinal),
        sha256: Sha::new(digest(bytes)).unwrap(),
        byte_length: u32::try_from(bytes.len()).unwrap(),
        media_type: Name::new("application/octet-stream").unwrap(),
        schema_id: Name::new("hee3.raw/1").unwrap(),
    }
}
fn read(sink: &Evidence<'_>, reference: &Ref) -> Vec<u8> {
    let mut bytes = Vec::new();
    sink.open(reference)
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}
fn published(store: &Store, bytes: &[u8]) -> Object {
    store
        .publish(bytes, UuidV4::parse(STAGE).unwrap(), deadline())
        .unwrap()
}
fn assert_uuid(value: &Id) {
    let bytes = value.as_str().as_bytes();
    assert_eq!(bytes.len(), 36);
    assert_eq!(bytes[14], b'4');
    assert!(matches!(bytes[19], b'8' | b'9' | b'a' | b'b'));
    assert_eq!([bytes[8], bytes[13], bytes[18], bytes[23]], [b'-'; 4]);
}
fn empty_page() -> ResourcePageV1 {
    ResourcePageV1 {
        page_index: 0,
        page_count: 1,
        row_count: 0,
        total_rows: 0,
        rows: List::new(vec![]).unwrap(),
        next: Maybe::unavailable(Text::new("end_of_inventory").unwrap()),
    }
}

#[test]
fn raw_payload_preserves_known_bytes_digest_media_and_complete_metadata() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let payload = sink.payload(b"abc", "text/plain").unwrap();
    let expected = payload.as_ref();
    assert_uuid(&expected.artifact_id);
    assert_eq!(expected.sha256.as_str(), ABC_SHA);
    assert_eq!(expected.byte_length, 3);
    assert_eq!(expected.media_type.as_str(), "text/plain");
    assert_eq!(expected.schema_id.as_str(), "hee3.raw/1");
    assert_eq!(read(&sink, expected), b"abc");
    let (stored, object) = &sink.registered()[expected.artifact_id.as_str()];
    assert_eq!(stored, expected);
    assert_eq!(object.digest(), ABC_SHA);
    assert_eq!(object.size(), 3);
    assert!(sink.pending_publications().is_empty());
    assert!(sink.last_publication_error().is_none());
}

#[test]
fn empty_and_non_utf8_payloads_remain_opaque_exact_bytes() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    for bytes in [b"".as_slice(), b"\x00\xff\xc3\x28\r\n"] {
        let payload = sink.payload(bytes, "application/octet-stream").unwrap();
        assert_eq!(read(&sink, payload.as_ref()), bytes);
        assert_eq!(payload.as_ref().sha256.as_str(), digest(bytes));
        assert_eq!(
            usize::try_from(payload.as_ref().byte_length).unwrap(),
            bytes.len()
        );
    }
    assert_eq!(sink.registered().len(), 2);
}

#[test]
fn invalid_media_is_refused_without_registering_an_object() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    for media in [String::new(), "x".repeat(129), "médiatype".to_owned()] {
        assert!(sink.payload(b"abc", &media).is_err());
        assert!(sink.registered().is_empty());
    }
    assert!(sink.payload(b"abc", "text/plain").is_ok());
}

#[test]
fn existing_store_object_can_be_imported_with_exact_metadata_and_read_back() {
    let area = Area::new();
    let store = area.open(true);
    let object = published(&store, b"abc");
    let mut sink = Evidence::new(&store, deadline());
    let expected = reference(1, b"abc");
    sink.register(expected.clone(), object).unwrap();
    assert_eq!(read(&sink, &expected), b"abc");
    assert!(sink.contains_id(&expected.artifact_id).unwrap());
    assert_eq!(sink.registered().len(), 1);
}

#[test]
fn distinct_logical_ids_may_share_the_same_immutable_store_object() {
    let area = Area::new();
    let store = area.open(true);
    let object = published(&store, b"abc");
    let mut sink = Evidence::new(&store, deadline());
    let first = reference(1, b"abc");
    let second = reference(2, b"abc");
    sink.register(first.clone(), object.clone()).unwrap();
    sink.register(second.clone(), object).unwrap();
    assert_eq!(read(&sink, &first), read(&sink, &second));
    assert_eq!(sink.registered().len(), 2);
}

#[test]
fn an_identical_duplicate_registration_still_refuses_id_reuse() {
    let area = Area::new();
    let store = area.open(true);
    let object = published(&store, b"abc");
    let mut sink = Evidence::new(&store, deadline());
    let expected = reference(1, b"abc");
    sink.register(expected.clone(), object.clone()).unwrap();
    assert!(sink.register(expected.clone(), object).is_err());
    assert_eq!(sink.registered().len(), 1);
    assert_eq!(read(&sink, &expected), b"abc");
}

#[test]
fn imported_object_digest_and_length_must_both_match_the_reference() {
    let area = Area::new();
    let store = area.open(true);
    let object = published(&store, b"abc");
    let mut sink = Evidence::new(&store, deadline());
    let mut wrong_digest = reference(1, b"xyz");
    let mut wrong_length = reference(2, b"abc");
    wrong_length.byte_length = 2;
    assert!(sink.register(wrong_digest.clone(), object.clone()).is_err());
    assert!(sink.register(wrong_length, object.clone()).is_err());
    assert!(sink.registered().is_empty());
    wrong_digest.sha256 = Sha::new(ABC_SHA).unwrap();
    sink.register(wrong_digest.clone(), object).unwrap();
    assert_eq!(read(&sink, &wrong_digest), b"abc");
}

#[test]
fn unknown_schema_reference_is_refused_by_register_and_publish() {
    let area = Area::new();
    let store = area.open(true);
    let object = published(&store, b"abc");
    let mut sink = Evidence::new(&store, deadline());
    let mut wrong = reference(1, b"abc");
    wrong.schema_id = Name::new("hee3.raw/2").unwrap();
    assert!(sink.register(wrong.clone(), object).is_err());
    assert!(sink.publish(&wrong, b"abc").is_err());
    assert!(sink.registered().is_empty());
    assert!(sink.pending_publications().is_empty());
}

#[test]
fn direct_publication_checks_actual_digest_before_registration() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let wrong = reference(1, b"xyz");
    assert!(sink.publish(&wrong, b"abc").is_err());
    assert!(sink.registered().is_empty());
    assert!(sink.last_publication_error().is_none());
    let correct = reference(1, b"abc");
    assert_eq!(sink.publish(&correct, b"abc").unwrap(), correct);
}

#[test]
fn direct_publication_checks_actual_length_independently_of_digest() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let mut wrong = reference(1, b"abc");
    wrong.byte_length = 4;
    assert!(sink.publish(&wrong, b"abc").is_err());
    assert!(sink.registered().is_empty());
    wrong.byte_length = 3;
    sink.publish(&wrong, b"abc").unwrap();
    assert_eq!(read(&sink, &wrong), b"abc");
}

#[test]
fn duplicate_publication_cannot_shadow_original_bytes_or_metadata() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let original = reference(1, b"abc");
    sink.publish(&original, b"abc").unwrap();
    assert!(sink.publish(&original, b"abc").is_err());
    assert!(sink.publish(&reference(1, b"xyz"), b"xyz").is_err());
    assert_eq!(read(&sink, &original), b"abc");
    assert_eq!(sink.registered().len(), 1);
}

#[test]
fn publishing_same_bytes_under_a_fresh_id_can_reuse_existing_cas_bytes() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let first = sink.payload(b"abc", "text/plain").unwrap();
    let second = sink.payload(b"abc", "text/plain").unwrap();
    assert_ne!(first.as_ref().artifact_id, second.as_ref().artifact_id);
    assert_eq!(first.as_ref().sha256, second.as_ref().sha256);
    assert_eq!(sink.registered().len(), 2);
    assert_eq!(read(&sink, second.as_ref()), b"abc");
    assert!(sink.pending_publications().is_empty());
}

#[test]
fn opening_requires_every_field_of_the_registered_complete_reference() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let original = reference(1, b"abc");
    sink.publish(&original, b"abc").unwrap();
    let mut alternatives = Vec::new();
    let mut changed = original.clone();
    changed.sha256 = Sha::new(digest(b"xyz")).unwrap();
    alternatives.push(changed);
    let mut changed = original.clone();
    changed.byte_length = 4;
    alternatives.push(changed);
    let mut changed = original.clone();
    changed.media_type = Name::new("text/plain").unwrap();
    alternatives.push(changed);
    let mut changed = original.clone();
    changed.schema_id = Name::new("hee3.receipt/1:ResourcePageV1").unwrap();
    alternatives.push(changed);
    for changed in alternatives {
        assert!(sink.open(&changed).is_err());
    }
    assert_eq!(read(&sink, &original), b"abc");
}

#[test]
fn unregistered_identity_cannot_open_even_when_bytes_exist_in_the_store() {
    let area = Area::new();
    let store = area.open(true);
    let _object = published(&store, b"abc");
    let sink = Evidence::new(&store, deadline());
    let unknown = reference(1, b"abc");
    assert!(!sink.contains_id(&unknown.artifact_id).unwrap());
    assert!(sink.open(&unknown).is_err());
}

#[test]
fn registered_object_is_rehashed_on_each_open_after_same_length_corruption() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let payload = sink.payload(b"abc", "text/plain").unwrap();
    let path = area.object_path(payload.as_ref().sha256.as_str());
    let metadata = fs::symlink_metadata(&path).unwrap();
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"xyz").unwrap();
    fs::set_permissions(&path, metadata.permissions()).unwrap();
    assert!(sink.open(payload.as_ref()).is_err());
    assert_eq!(sink.registered().len(), 1);
}

#[test]
fn missing_registered_object_is_not_satisfied_by_cached_registry_metadata() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let payload = sink.payload(b"abc", "text/plain").unwrap();
    fs::remove_file(area.object_path(payload.as_ref().sha256.as_str())).unwrap();
    assert!(sink.open(payload.as_ref()).is_err());
    assert!(sink.contains_id(&payload.as_ref().artifact_id).unwrap());
}

#[test]
fn failed_import_readback_preserves_store_error_across_later_success() {
    let area = Area::new();
    let store = area.open(true);
    let object = published(&store, b"abc");
    let path = area.object_path(object.digest());
    let metadata = fs::symlink_metadata(&path).unwrap();
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"xyz").unwrap();
    fs::set_permissions(&path, metadata.permissions()).unwrap();
    let mut sink = Evidence::new(&store, deadline());
    assert!(sink.register(reference(1, b"abc"), object).is_err());
    assert!(sink.registered().is_empty());
    assert!(matches!(
        sink.last_publication_error(),
        Some(StoreError::Corrupt)
    ));
    sink.payload(b"unrelated benign payload", "text/plain")
        .unwrap();
    assert!(matches!(
        sink.last_publication_error(),
        Some(StoreError::Corrupt)
    ));
    assert_eq!(sink.registered().len(), 1);
}

#[test]
fn missing_import_object_refuses_without_registering_a_dangling_reference() {
    let area = Area::new();
    let store = area.open(true);
    let object = published(&store, b"abc");
    fs::remove_file(area.object_path(object.digest())).unwrap();
    let mut sink = Evidence::new(&store, deadline());
    assert!(sink.register(reference(1, b"abc"), object).is_err());
    assert!(sink.registered().is_empty());
    assert!(sink.last_publication_error().is_some());
}

#[test]
fn fresh_uuid_is_local_and_unreserved_until_successful_publication() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let fresh = sink.fresh_id().unwrap();
    assert_uuid(&fresh);
    assert!(!sink.contains_id(&fresh).unwrap());
    assert!(sink.registered().is_empty());
    let mut expected = reference(1, b"abc");
    expected.artifact_id = fresh.clone();
    sink.publish(&expected, b"abc").unwrap();
    assert!(sink.contains_id(&fresh).unwrap());
    let next = sink.fresh_id().unwrap();
    assert_uuid(&next);
    assert_ne!(next, fresh);
    assert_eq!(sink.registered().len(), 1);
}

#[test]
fn free_uuid_allocator_obeys_deadline_and_returns_version_four() {
    assert!(evidence::fresh_id(expired()).is_err());
    assert_uuid(&evidence::fresh_id(deadline()).unwrap());
}

#[test]
fn every_sink_operation_uses_the_original_expired_deadline() {
    let area = Area::new();
    let store = area.open(true);
    let object = published(&store, b"abc");
    let expected = reference(1, b"abc");
    let mut sink = Evidence::new(&store, expired());
    assert!(sink.fresh_id().is_err());
    assert!(sink.contains_id(&expected.artifact_id).is_err());
    assert!(sink.payload(b"abc", "text/plain").is_err());
    assert!(sink.publish(&expected, b"abc").is_err());
    assert!(sink.register(expected.clone(), object).is_err());
    assert!(sink.open(&expected).is_err());
    assert!(sink.registered().is_empty());
}

#[test]
fn registry_transfer_preserves_exact_objects_but_reopen_does_not_invent_a_global_registry() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let payload = sink.payload(b"abc", "text/plain").unwrap();
    let registry = sink.into_registered();
    assert_eq!(registry.len(), 1);
    drop(store);
    let reopened = area.open(false);
    let mut restored = Evidence::new(&reopened, deadline());
    assert!(!restored.contains_id(&payload.as_ref().artifact_id).unwrap());
    for (key, (reference, object)) in registry {
        assert_eq!(key, reference.artifact_id.as_str());
        restored.register(reference, object).unwrap();
    }
    assert_eq!(read(&restored, payload.as_ref()), b"abc");
}

#[test]
fn registry_limit_accepts_4096_and_refuses_adjacent_register_and_publish() {
    let area = Area::new();
    let store = area.open(true);
    let object = published(&store, b"abc");
    let mut sink = Evidence::new(&store, Instant::now() + Duration::from_secs(60));
    for ordinal in 1..=4096 {
        sink.register(reference(ordinal, b"abc"), object.clone())
            .unwrap();
    }
    assert_eq!(sink.registered().len(), 4096);
    assert_eq!(read(&sink, &reference(4096, b"abc")), b"abc");
    assert!(sink.register(reference(4097, b"abc"), object).is_err());
    assert!(sink.publish(&reference(4097, b"abc"), b"abc").is_err());
    assert_eq!(sink.registered().len(), 4096);
    assert!(sink.pending_publications().is_empty());
}

#[test]
fn store_bound_failure_is_retained_after_a_later_successful_payload() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let bytes = vec![0x61; 16 * 1024 * 1024 + 1];
    assert!(sink.payload(&bytes, "application/octet-stream").is_err());
    assert!(matches!(
        sink.last_publication_error(),
        Some(StoreError::Bound)
    ));
    assert!(sink.registered().is_empty());
    assert!(sink.pending_publications().is_empty());
    sink.payload(b"benign", "text/plain").unwrap();
    assert!(matches!(
        sink.last_publication_error(),
        Some(StoreError::Bound)
    ));
    assert_eq!(sink.registered().len(), 1);
}

#[test]
fn actual_publisher_records_and_pages_resolve_through_store_backed_graph() {
    let area = Area::new();
    let store = area.open(true);
    let mut sink = Evidence::new(&store, deadline());
    let page = empty_page();
    let encoded = receipt::encode(&page).unwrap();
    let (typed, named, attempted) = {
        let mut publisher = Publisher::new(&mut sink);
        let typed = publisher.record(&page).unwrap();
        let named = publisher.resource_pages(&[]).unwrap();
        (typed, named, publisher.attempted_refs().to_vec())
    };
    assert_eq!(attempted.len(), 2);
    assert_eq!(read(&sink, typed.as_ref()), encoded);
    assert_eq!(typed.as_ref().sha256.as_str(), digest(&encoded));
    for reference in [typed.as_ref(), named.as_ref()] {
        let graph = Graph::resolve(&sink, reference).unwrap();
        assert_eq!(graph.object_count(), 1);
        assert!(graph.rows(reference).unwrap().is_empty());
        let decoded: ResourcePageV1 = receipt::decode(&read(&sink, reference)).unwrap();
        assert_eq!(
            (
                decoded.page_index,
                decoded.page_count,
                decoded.row_count,
                decoded.total_rows
            ),
            (0, 1, 0, 0)
        );
    }
    assert!(sink.pending_publications().is_empty());
    assert_eq!(sink.registered().len(), 2);
}

#[test]
fn failed_store_publication_retains_publisher_attempt_and_error_without_a_usable_ref() {
    let area = Area::new();
    let store = area.open(true);
    let page = empty_page();
    let encoded = receipt::encode(&page).unwrap();
    let blocked = area.object_path(&digest(&encoded));
    let shard = blocked.parent().unwrap();
    fs::create_dir_all(shard).unwrap();
    fs::set_permissions(shard, fs::Permissions::from_mode(0o700)).unwrap();
    DirBuilder::new().mode(0o700).create(&blocked).unwrap();
    let mut sink = Evidence::new(&store, deadline());
    let attempted = {
        let mut publisher = Publisher::new(&mut sink);
        assert!(publisher.record(&page).is_err());
        publisher.attempted_refs().to_vec()
    };
    assert_eq!(attempted.len(), 1);
    assert_eq!(attempted[0].sha256.as_str(), digest(&encoded));
    assert_eq!(
        usize::try_from(attempted[0].byte_length).unwrap(),
        encoded.len()
    );
    assert!(sink.registered().is_empty());
    assert!(sink.last_publication_error().is_some());
    assert!(sink.open(&attempted[0]).is_err());
    assert!(blocked.is_dir());
    fs::remove_dir(&blocked).unwrap();
    let good = {
        let mut publisher = Publisher::new(&mut sink);
        publisher.record(&page).unwrap()
    };
    assert_eq!(read(&sink, good.as_ref()), encoded);
    assert!(sink.last_publication_error().is_some());
    assert!(sink.pending_publications().is_empty());
}
