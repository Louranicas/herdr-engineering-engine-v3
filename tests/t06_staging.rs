//! Independent public `ArtifactStaging` controls; implementation was not read.
//! Private filesystem fixtures confer no custody, durability or admission claim.
use habitat_engine::contracts::UuidV4;
use habitat_engine::store::{ArtifactStaging, Object};
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const FIRST: &str = "06800000-0000-4000-8000-000000000001";
const SECOND: &str = "06800000-0000-4000-8000-000000000002";
const ABC: &str = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const EMPTY: &str = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Area {
    container: PathBuf,
    root: PathBuf,
    device: u64,
    inode: u64,
}
impl Area {
    fn new() -> Self {
        let container = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "hee3-t06-staging-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        private_dir(&container);
        let root = container.join("staging");
        private_dir(&root);
        let meta = fs::symlink_metadata(&container).unwrap();
        Self {
            container,
            root,
            device: meta.dev(),
            inode: meta.ino(),
        }
    }
    fn open(&self, create: bool) -> ArtifactStaging {
        ArtifactStaging::open(&self.root, create, deadline()).unwrap()
    }
    fn object_path(&self, digest: &str) -> PathBuf {
        let hex = digest.strip_prefix("sha256:").unwrap();
        self.root.join("sha256").join(&hex[..2]).join(hex)
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        let meta = fs::symlink_metadata(&self.container).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!((meta.dev(), meta.ino()), (self.device, self.inode));
        fs::remove_dir_all(&self.container).unwrap();
    }
}
fn private_dir(path: &Path) {
    DirBuilder::new().mode(0o700).create(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
fn expired() -> Instant {
    Instant::now().checked_sub(Duration::from_secs(1)).unwrap()
}
fn publish(staging: &ArtifactStaging, bytes: &[u8]) -> Object {
    staging
        .publish(bytes, UuidV4::parse(FIRST).unwrap(), deadline())
        .unwrap()
}
fn corrupt_same_length(path: &Path) {
    let meta = fs::symlink_metadata(path).unwrap();
    assert!(meta.is_file() && !meta.file_type().is_symlink());
    assert_eq!(meta.len(), 3);
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(path, b"xyz").unwrap();
    fs::set_permissions(path, meta.permissions()).unwrap();
}
#[test]
fn create_initializes_existing_private_root_and_false_reopens_it() {
    let area = Area::new();
    assert!(ArtifactStaging::open(&area.root, false, deadline()).is_err());
    let staging = area.open(true);
    let object = publish(&staging, b"abc");
    drop(staging);
    let reopened = area.open(false);
    assert_eq!(reopened.read_object(&object, deadline()).unwrap(), b"abc");
}
#[test]
fn create_does_not_create_a_missing_root() {
    let area = Area::new();
    let absent = area.container.join("absent");
    assert!(ArtifactStaging::open(&absent, true, deadline()).is_err());
    assert!(!absent.exists());
}
#[test]
fn private_root_permissions_are_required_without_silent_repair() {
    let area = Area::new();
    fs::set_permissions(&area.root, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(ArtifactStaging::open(&area.root, true, deadline()).is_err());
    assert_eq!(
        fs::metadata(&area.root).unwrap().permissions().mode() & 0o777,
        0o755
    );
    fs::set_permissions(&area.root, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(ArtifactStaging::open(&area.root, true, deadline()).is_ok());
}
#[test]
fn a_symlink_root_is_not_a_canonical_owned_root() {
    let area = Area::new();
    let alias = area.container.join("alias");
    symlink(&area.root, &alias).unwrap();
    assert!(ArtifactStaging::open(&alias, true, deadline()).is_err());
    assert!(ArtifactStaging::open(&area.root, true, deadline()).is_ok());
}
#[test]
fn independent_owner_of_same_root_is_refused_while_first_remains_usable() {
    let area = Area::new();
    let first = area.open(true);
    assert!(
        ArtifactStaging::open(
            &area.root,
            false,
            Instant::now() + Duration::from_millis(100)
        )
        .is_err()
    );
    let object = publish(&first, b"abc");
    assert_eq!(first.read_object(&object, deadline()).unwrap(), b"abc");
}
#[test]
fn distinct_roots_can_be_owned_concurrently() {
    let a = Area::new();
    let b = Area::new();
    let first = a.open(true);
    let second = b.open(true);
    let left = publish(&first, b"abc");
    let right = publish(&second, b"different");
    assert_eq!(first.read_object(&left, deadline()).unwrap(), b"abc");
    assert_eq!(
        second.read_object(&right, deadline()).unwrap(),
        b"different"
    );
}
#[test]
fn normal_reopen_retains_the_same_lock_inode() {
    let area = Area::new();
    let staging = area.open(true);
    let before = fs::symlink_metadata(area.root.join("store.lock")).unwrap();
    assert!(before.is_file() && !before.file_type().is_symlink());
    drop(staging);
    let reopened = area.open(false);
    let after = fs::symlink_metadata(area.root.join("store.lock")).unwrap();
    assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
    assert!(
        ArtifactStaging::open(
            &area.root,
            false,
            Instant::now() + Duration::from_millis(100)
        )
        .is_err()
    );
    drop(reopened);
}
#[test]
fn publication_preserves_literal_known_digest_and_exact_bytes() {
    let area = Area::new();
    let staging = area.open(true);
    let object = publish(&staging, b"abc");
    assert_eq!(object.digest(), ABC);
    assert_eq!(object.size(), 3);
    assert_eq!(staging.read_object(&object, deadline()).unwrap(), b"abc");
}
#[test]
fn empty_and_non_utf8_payloads_are_opaque() {
    let area = Area::new();
    let staging = area.open(true);
    let empty = publish(&staging, b"");
    assert_eq!(empty.digest(), EMPTY);
    assert_eq!(empty.size(), 0);
    assert!(staging.read_object(&empty, deadline()).unwrap().is_empty());
    let bytes = b"\0\xff\xc3\x28\r\n";
    let binary = staging
        .publish(bytes, UuidV4::parse(SECOND).unwrap(), deadline())
        .unwrap();
    assert_eq!(binary.size(), 6);
    assert_eq!(staging.read_object(&binary, deadline()).unwrap(), bytes);
}
#[test]
fn repeated_valid_digest_preserves_immutable_object_inode() {
    let area = Area::new();
    let staging = area.open(true);
    let first = publish(&staging, b"abc");
    let before = fs::symlink_metadata(area.object_path(ABC)).unwrap();
    let second = staging
        .publish(b"abc", UuidV4::parse(SECOND).unwrap(), deadline())
        .unwrap();
    let after = fs::symlink_metadata(area.object_path(ABC)).unwrap();
    assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
    assert_eq!(first.digest(), second.digest());
    assert_eq!(staging.read_object(&first, deadline()).unwrap(), b"abc");
}
#[test]
fn same_length_corruption_is_detected_on_read() {
    let area = Area::new();
    let staging = area.open(true);
    let object = publish(&staging, b"abc");
    corrupt_same_length(&area.object_path(ABC));
    assert!(staging.read_object(&object, deadline()).is_err());
}
#[test]
fn publication_does_not_silently_repair_a_corrupt_immutable_digest() {
    let area = Area::new();
    let staging = area.open(true);
    let _object = publish(&staging, b"abc");
    corrupt_same_length(&area.object_path(ABC));
    assert!(
        staging
            .publish(b"abc", UuidV4::parse(SECOND).unwrap(), deadline())
            .is_err()
    );
    assert_eq!(fs::read(area.object_path(ABC)).unwrap(), b"xyz");
}
#[test]
fn absent_object_is_not_satisfied_by_its_retained_descriptor() {
    let area = Area::new();
    let staging = area.open(true);
    let object = publish(&staging, b"abc");
    fs::remove_file(area.object_path(ABC)).unwrap();
    assert!(staging.read_object(&object, deadline()).is_err());
}
#[test]
fn matching_bytes_through_a_symlink_do_not_establish_object_custody() {
    let area = Area::new();
    let staging = area.open(true);
    let object = publish(&staging, b"abc");
    let path = area.object_path(ABC);
    let target = area.container.join("matching-target");
    fs::rename(&path, &target).unwrap();
    symlink(&target, &path).unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"abc");
    assert!(staging.read_object(&object, deadline()).is_err());
}
#[test]
fn matching_hardlinked_bytes_do_not_establish_exclusive_custody() {
    let area = Area::new();
    let staging = area.open(true);
    let object = publish(&staging, b"abc");
    let path = area.object_path(ABC);
    let alias = area.container.join("hardlink");
    fs::hard_link(&path, &alias).unwrap();
    assert_eq!(fs::metadata(&path).unwrap().nlink(), 2);
    assert!(staging.read_object(&object, deadline()).is_err());
}
#[test]
fn broad_object_permissions_refuse_even_when_digest_matches() {
    let area = Area::new();
    let staging = area.open(true);
    let object = publish(&staging, b"abc");
    let path = area.object_path(ABC);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(staging.read_object(&object, deadline()).is_err());
}
#[test]
fn substituted_existing_sha256_layout_is_refused() {
    let area = Area::new();
    let staging = area.open(true);
    drop(staging);
    fs::rename(area.root.join("sha256"), area.container.join("old-sha256")).unwrap();
    fs::write(area.root.join("sha256"), b"not a directory").unwrap();
    assert!(ArtifactStaging::open(&area.root, false, deadline()).is_err());
    assert_eq!(
        fs::read(area.root.join("sha256")).unwrap(),
        b"not a directory"
    );
}
#[test]
fn a_preexisting_symlink_lock_is_not_followed_or_replaced() {
    let area = Area::new();
    let target = area.container.join("lock-target");
    fs::write(&target, b"sentinel").unwrap();
    symlink(&target, area.root.join("store.lock")).unwrap();
    assert!(ArtifactStaging::open(&area.root, true, deadline()).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"sentinel");
    assert!(
        fs::symlink_metadata(area.root.join("store.lock"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}
#[test]
fn a_directory_at_the_digest_path_is_not_overwritten_or_globally_poisoning() {
    let area = Area::new();
    let staging = area.open(true);
    let path = area.object_path(ABC);
    if !path.parent().unwrap().exists() {
        private_dir(path.parent().unwrap());
    }
    private_dir(&path);
    assert!(
        staging
            .publish(b"abc", UuidV4::parse(FIRST).unwrap(), deadline())
            .is_err()
    );
    assert!(fs::symlink_metadata(&path).unwrap().is_dir());
    let other = staging
        .publish(b"different", UuidV4::parse(SECOND).unwrap(), deadline())
        .unwrap();
    assert_eq!(
        staging.read_object(&other, deadline()).unwrap(),
        b"different"
    );
}
#[test]
fn expired_calls_refuse_with_successful_neighbors() {
    let area = Area::new();
    assert!(ArtifactStaging::open(&area.root, true, expired()).is_err());
    let staging = area.open(true);
    assert!(
        staging
            .publish(b"abc", UuidV4::parse(FIRST).unwrap(), expired())
            .is_err()
    );
    assert!(!area.object_path(ABC).exists());
    let object = publish(&staging, b"abc");
    assert!(staging.read_object(&object, expired()).is_err());
    assert_eq!(staging.read_object(&object, deadline()).unwrap(), b"abc");
}
