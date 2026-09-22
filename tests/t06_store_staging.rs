//! Scoped artifact capability controls; no runtime responsiveness claim.
use super::*;
use std::fs::{self, DirBuilder, Permissions};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt, symlink};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc,
};

const GEN: &str = "06600000-0000-4000-8000-000000000001";
const EPOCH: &str = "06600000-0000-4000-8000-000000000002";
const TASK: &str = "06600000-0000-4000-8000-000000000003";
const STAGE: &str = "06600000-0000-4000-8000-000000000004";
const EVENT: &str = "06600000-0000-4000-8000-000000000005";
const CANCEL: &str = "06600000-0000-4000-8000-000000000006";
const ABC: &str = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Area {
    path: PathBuf,
    identity: (u64, u64),
}
impl Area {
    fn new() -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "hee3-scoped-staging-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
        let meta = fs::symlink_metadata(&path).unwrap();
        Self {
            path,
            identity: (meta.dev(), meta.ino()),
        }
    }
    fn open(&self) -> Store {
        Store::open(&self.path, id(GEN), id(EPOCH), true, deadline()).unwrap()
    }
    fn object_path(&self) -> PathBuf {
        self.path
            .join("generations")
            .join(GEN)
            .join("objects/sha256/ba")
            .join(&ABC[7..])
    }
    fn locked(&self) -> bool {
        matches!(
            Store::open(&self.path, id(GEN), id(EPOCH), false, deadline()),
            Err(Error::Locked)
        )
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        let meta = fs::symlink_metadata(&self.path).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!((meta.dev(), meta.ino()), self.identity);
        fs::remove_dir_all(&self.path).unwrap();
    }
}
fn id(text: &str) -> UuidV4<'_> {
    UuidV4::parse(text).unwrap()
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn expired() -> Instant {
    Instant::now().checked_sub(Duration::from_secs(1)).unwrap()
}
fn publish(stage: &ArtifactStaging) -> Object {
    stage.publish(b"abc", id(STAGE), deadline()).unwrap()
}

#[test]
fn scoped_publication_is_exact_store_custody_without_ledger_reference() {
    let area = Area::new();
    let mut store = area.open();
    let object = store
        .with_artifact_staging(deadline(), |owner, stage| {
            let object = publish(stage);
            assert_eq!(object.digest(), ABC);
            assert_eq!(owner.read_object(&object, deadline()).unwrap(), b"abc");
            object
        })
        .unwrap();
    assert_eq!(store.read_object(&object, deadline()).unwrap(), b"abc");
    let n: i64 = store
        .connection
        .query_row("SELECT COUNT(*) FROM artifacts", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
}
#[test]
fn dropping_capability_does_not_unlock_owner_and_reopen_works_after_owner_drop() {
    let area = Area::new();
    let mut store = area.open();
    for _ in 0..2 {
        store
            .with_artifact_staging(deadline(), |_, stage| {
                assert!(area.locked());
                publish(stage)
            })
            .unwrap();
        assert!(area.locked());
    }
    drop(store);
    drop(area.open());
}
#[test]
fn capability_holds_original_lock_even_if_callback_replaces_store() {
    let area = Area::new();
    let other = Area::new();
    let mut store = area.open();
    store
        .with_artifact_staging(deadline(), |owner, stage| {
            drop(std::mem::replace(owner, other.open()));
            assert!(area.locked());
            let object = publish(stage);
            assert_eq!(stage.read_object(&object, deadline()).unwrap(), b"abc");
        })
        .unwrap();
    drop(area.open());
    assert!(other.locked());
}
#[test]
fn callback_error_preserves_result_and_published_bytes() {
    let area = Area::new();
    let mut store = area.open();
    let returned = store
        .with_artifact_staging(deadline(), |_, stage| {
            Err::<(), _>((publish(stage), "original"))
        })
        .unwrap();
    let (object, error) = returned.unwrap_err();
    assert_eq!(error, "original");
    assert_eq!(store.read_object(&object, deadline()).unwrap(), b"abc");
    assert!(area.locked());
}
#[test]
fn expired_entry_does_not_invoke_callback() {
    let area = Area::new();
    let mut store = area.open();
    let mut invoked = false;
    assert!(matches!(
        store.with_artifact_staging(expired(), |_, _| invoked = true),
        Err(Error::Deadline)
    ));
    assert!(!invoked);
    assert!(store.with_artifact_staging(deadline(), |_, _| 7).is_ok());
}
#[test]
fn callback_result_is_preserved_if_deadline_expires_after_effect() {
    let area = Area::new();
    let mut store = area.open();
    let end = Instant::now() + Duration::from_millis(60);
    let object = store
        .with_artifact_staging(end, |_, stage| {
            let object = stage.publish(b"abc", id(STAGE), end).unwrap();
            std::thread::sleep(
                end.saturating_duration_since(Instant::now()) + Duration::from_millis(2),
            );
            object
        })
        .unwrap();
    assert_eq!(store.read_object(&object, deadline()).unwrap(), b"abc");
}
#[test]
fn expired_operations_refuse_without_claiming_objects() {
    let area = Area::new();
    let mut store = area.open();
    store
        .with_artifact_staging(deadline(), |_, stage| {
            assert!(matches!(
                stage.publish(b"abc", id(STAGE), expired()),
                Err(Error::Deadline)
            ));
            assert!(!area.object_path().exists());
            let object = publish(stage);
            assert!(matches!(
                stage.read_object(&object, expired()),
                Err(Error::Deadline)
            ));
        })
        .unwrap();
}
#[test]
fn panic_releases_scoped_fds_but_keeps_store_locked_and_reusable() {
    let area = Area::new();
    let mut store = area.open();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        store.with_artifact_staging(deadline(), |_, stage| {
            publish(stage);
            panic!("intended callback failure")
        })
    }));
    assert!(result.is_err());
    assert!(area.locked());
    store
        .with_artifact_staging(deadline(), |_, stage| {
            assert_eq!(publish(stage).digest(), ABC);
        })
        .unwrap();
    drop(store);
    drop(area.open());
}
#[test]
fn aliases_cannot_acquire_independent_staging_or_store_custody() {
    let area = Area::new();
    let mut store = area.open();
    let alias = area.path.join("alias");
    symlink(&area.path, &alias).unwrap();
    store
        .with_artifact_staging(deadline(), |_, stage| {
            assert!(ArtifactStaging::open(&alias, false, deadline()).is_err());
            assert!(ArtifactStaging::open(&area.path, false, deadline()).is_err());
            assert!(Store::open(&alias, id(GEN), id(EPOCH), false, deadline()).is_err());
            publish(stage);
        })
        .unwrap();
}
#[test]
fn corrupted_object_is_not_replaced_or_claimed() {
    let area = Area::new();
    let mut store = area.open();
    let object = store.publish(b"abc", id(STAGE), deadline()).unwrap();
    let path = area.object_path();
    fs::set_permissions(&path, Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"xyz").unwrap();
    fs::set_permissions(&path, Permissions::from_mode(0o400)).unwrap();
    store
        .with_artifact_staging(deadline(), |_, stage| {
            assert!(matches!(
                stage.read_object(&object, deadline()),
                Err(Error::Corrupt)
            ));
            assert!(matches!(
                stage.publish(b"abc", id(STAGE), deadline()),
                Err(Error::Corrupt)
            ));
        })
        .unwrap();
    assert_eq!(fs::read(path).unwrap(), b"xyz");
}
#[test]
fn duplicate_publication_keeps_original_inode_and_bytes() {
    let area = Area::new();
    let mut store = area.open();
    let first = store.publish(b"abc", id(STAGE), deadline()).unwrap();
    let before = fs::metadata(area.object_path()).unwrap();
    store
        .with_artifact_staging(deadline(), |_, stage| assert_eq!(publish(stage), first))
        .unwrap();
    let after = fs::metadata(area.object_path()).unwrap();
    assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
}
#[test]
fn publication_cutpoints_remain_effective_with_truthful_retained_objects() {
    for point in [
        CutPoint::ObjectWrite,
        CutPoint::ObjectSync,
        CutPoint::ObjectRename,
        CutPoint::ObjectDirectorySync,
    ] {
        let area = Area::new();
        let mut store = area.open();
        store.fault = Some(point);
        let result = store
            .with_artifact_staging(deadline(), |_, stage| {
                stage.publish(b"abc", id(STAGE), deadline())
            })
            .unwrap();
        assert!(matches!(result,Err(Error::Injected(actual)) if actual==format!("{point:?}")));
        assert_eq!(
            area.object_path().exists(),
            matches!(
                point,
                CutPoint::ObjectRename | CutPoint::ObjectDirectorySync
            )
        );
        store.fault = None;
        store
            .with_artifact_staging(deadline(), |_, stage| {
                assert_eq!(publish(stage).digest(), ABC);
            })
            .unwrap();
    }
}
#[test]
fn cancellation_commits_while_scoped_worker_retains_artifact_custody() {
    let area = Area::new();
    let mut store = area.open();
    let principal = Principal::new(1000, "operator").unwrap();
    store
        .submit(
            Submission {
                principal: &principal,
                key: id(STAGE),
                task: id(TASK),
                event: id(EVENT),
                request_bytes: b"request",
                criteria: Sha256Digest::parse(ABC).unwrap(),
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
        .with_artifact_staging(deadline(), |owner, stage| {
            std::thread::scope(|scope| {
                let (ready_tx, ready_rx) = mpsc::sync_channel(1);
                let (finish_tx, finish_rx) = mpsc::sync_channel(1);
                let worker = scope.spawn(move || {
                    let object = publish(stage);
                    ready_tx.send(()).unwrap();
                    finish_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                    assert_eq!(stage.read_object(&object, deadline()).unwrap(), b"abc");
                    object
                });
                ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                let result = owner.cancel(id(TASK), "1".parse().unwrap(), id(CANCEL), deadline());
                let observed = owner.get(&principal, id(TASK), deadline());
                finish_tx.send(()).unwrap();
                let object = worker.join().unwrap();
                result.unwrap();
                let head = observed.unwrap();
                assert!(head.cancellation);
                assert_eq!(head.generation, "2");
                assert!(head.accepted_event.is_none());
                assert_eq!(owner.read_object(&object, deadline()).unwrap(), b"abc");
            });
        })
        .unwrap();
    drop(store);
    let store = area.open();
    assert!(
        store
            .get(&principal, id(TASK), deadline())
            .unwrap()
            .cancellation
    );
}
