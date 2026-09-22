use habitat_engine::{
    app::{
        evidence::Evidence,
        subjects::{self, ErrorKind},
    },
    check::graph::Objects,
    contracts::{
        UuidV4,
        receipt::{self, SubjectFilePageV1, SubjectFileV1Origin, SubjectV1},
    },
    store::Store,
    worker::workspace::Snapshot,
};
use std::{
    fs::{self, DirBuilder},
    io::Read,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
static N: AtomicU64 = AtomicU64::new(1);
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
struct Area {
    base: PathBuf,
}
impl Area {
    fn new() -> Self {
        let base = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "hee3-subject-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&base).unwrap();
        Self { base }
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.base).unwrap();
    }
}
fn setup() -> (Area, Store, Snapshot) {
    let a = Area::new();
    let source = a.base.join("source");
    DirBuilder::new().mode(0o700).create(&source).unwrap();
    fs::create_dir(source.join("dir")).unwrap();
    fs::set_permissions(source.join("dir"), fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(source.join("a.txt"), b"alpha").unwrap();
    fs::set_permissions(source.join("a.txt"), fs::Permissions::from_mode(0o600)).unwrap();
    let snap = Snapshot::capture(&source, &[], deadline()).unwrap();
    let store_root = a.base.join("store");
    DirBuilder::new().mode(0o700).create(&store_root).unwrap();
    let store = Store::open(
        &store_root,
        UuidV4::parse("06700000-0000-4000-8000-000000000001").unwrap(),
        UuidV4::parse("06700000-0000-4000-8000-000000000002").unwrap(),
        true,
        deadline(),
    )
    .unwrap();
    (a, store, snap)
}
fn read(e: &Evidence<'_>, r: &receipt::Ref) -> Vec<u8> {
    let mut b = vec![];
    e.open(r).unwrap().read_to_end(&mut b).unwrap();
    b
}
#[test]
fn publishes_exact_subject_and_first_page_digest() {
    let (_a, s, snap) = setup();
    let mut e = Evidence::new(&s, deadline());
    let root = subjects::publish(&mut e, &snap, SubjectFileV1Origin::Authored, deadline()).unwrap();
    let subject: SubjectV1 = receipt::decode(&read(&e, root.as_ref())).unwrap();
    assert_eq!(subject.tree_sha256, subject.files.as_ref().sha256);
    assert!(subject.dirty_patch.value.is_none());
    let page: SubjectFilePageV1 = receipt::decode(&read(&e, subject.files.as_ref())).unwrap();
    assert_eq!(page.total_rows, 2);
    assert_eq!(page.rows.as_slice()[0].path.as_str(), "a.txt");
    assert_eq!(page.rows.as_slice()[1].path.as_str(), "dir");
    assert!(page.rows.as_slice()[0].content.value.is_some());
    assert!(page.rows.as_slice()[1].content.value.is_none());
}
#[test]
fn excluded_origin_refuses_without_publication() {
    let (_a, s, snap) = setup();
    let mut e = Evidence::new(&s, deadline());
    let err =
        subjects::publish(&mut e, &snap, SubjectFileV1Origin::Excluded, deadline()).unwrap_err();
    assert_eq!(err.kind, ErrorKind::ExcludedUnsupported);
    assert!(err.attempted.is_empty());
    assert!(e.registered().is_empty());
}
#[test]
fn changed_source_refuses_before_typed_pages() {
    let (_a, s, snap) = setup();
    fs::write(snap.root().join("a.txt"), b"changed").unwrap();
    let mut e = Evidence::new(&s, deadline());
    let err =
        subjects::publish(&mut e, &snap, SubjectFileV1Origin::Generated, deadline()).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::Snapshot(_)));
    assert!(e.registered().is_empty());
}

#[test]
fn two_pages_bind_all_257_sorted_rows_tail_first() {
    let (area, store, _) = setup();
    let source = area.base.join("many");
    DirBuilder::new().mode(0o700).create(&source).unwrap();
    for index in 0..257 {
        let path = source.join(format!("f{index:03}"));
        fs::write(&path, [u8::try_from(index % 251).unwrap()]).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let snapshot = Snapshot::capture(&source, &[], deadline()).unwrap();
    let mut evidence = Evidence::new(&store, deadline());
    let root = subjects::publish(
        &mut evidence,
        &snapshot,
        SubjectFileV1Origin::Generated,
        deadline(),
    )
    .unwrap();
    let subject: SubjectV1 = receipt::decode(&read(&evidence, root.as_ref())).unwrap();
    let first: SubjectFilePageV1 =
        receipt::decode(&read(&evidence, subject.files.as_ref())).unwrap();
    assert_eq!(
        (
            first.page_index,
            first.page_count,
            first.row_count,
            first.total_rows
        ),
        (0, 2, 256, 257)
    );
    let second_ref = first.next.value.unwrap();
    let second: SubjectFilePageV1 = receipt::decode(&read(&evidence, second_ref.as_ref())).unwrap();
    assert_eq!((second.page_index, second.row_count), (1, 1));
    assert_eq!(first.rows.as_slice()[0].path.as_str(), "f000");
    assert_eq!(second.rows.as_slice()[0].path.as_str(), "f256");
    subjects::verify(
        &evidence,
        &snapshot,
        SubjectFileV1Origin::Generated,
        &root,
        deadline(),
    )
    .unwrap();
}

#[test]
fn published_identity_accepts_a_fresh_copy_of_the_exact_same_tree() {
    let (area, store, snapshot) = setup();
    let mut evidence = Evidence::new(&store, deadline());
    let root = subjects::publish(
        &mut evidence,
        &snapshot,
        SubjectFileV1Origin::Authored,
        deadline(),
    )
    .unwrap();
    let materialized = snapshot
        .materialize(&area.base, "copy", &[], deadline())
        .unwrap();
    let copy = Snapshot::capture(
        &materialized.path,
        &snapshot.source_identities(),
        deadline(),
    )
    .unwrap();
    let before = evidence.registered().len();
    subjects::verify(
        &evidence,
        &copy,
        SubjectFileV1Origin::Authored,
        &root,
        deadline(),
    )
    .unwrap();
    assert_eq!(evidence.registered().len(), before);
}

#[test]
fn independently_captured_content_mode_and_inventory_changes_refuse_the_old_subject() {
    for change in ["bytes", "executable", "add", "remove", "rename"] {
        let (_area, store, snapshot) = setup();
        let mut evidence = Evidence::new(&store, deadline());
        let root = subjects::publish(
            &mut evidence,
            &snapshot,
            SubjectFileV1Origin::Authored,
            deadline(),
        )
        .unwrap();
        let source = snapshot.root();
        match change {
            "bytes" => fs::write(source.join("a.txt"), b"other").unwrap(),
            "executable" => {
                fs::set_permissions(source.join("a.txt"), fs::Permissions::from_mode(0o700))
                    .unwrap();
            }
            "add" => fs::create_dir(source.join("extra")).unwrap(),
            "remove" => fs::remove_dir(source.join("dir")).unwrap(),
            "rename" => fs::rename(source.join("a.txt"), source.join("b.txt")).unwrap(),
            _ => unreachable!(),
        }
        let changed = Snapshot::capture(source, &[], deadline()).unwrap();
        assert_eq!(
            subjects::verify(
                &evidence,
                &changed,
                SubjectFileV1Origin::Authored,
                &root,
                deadline()
            )
            .unwrap_err()
            .kind,
            ErrorKind::Mismatch,
            "{change}"
        );
    }
}

#[test]
fn frozen_capture_must_still_match_its_live_source_and_declared_origin() {
    let (_area, store, snapshot) = setup();
    let mut evidence = Evidence::new(&store, deadline());
    let root = subjects::publish(
        &mut evidence,
        &snapshot,
        SubjectFileV1Origin::Authored,
        deadline(),
    )
    .unwrap();
    assert_eq!(
        subjects::verify(
            &evidence,
            &snapshot,
            SubjectFileV1Origin::Generated,
            &root,
            deadline()
        )
        .unwrap_err()
        .kind,
        ErrorKind::Mismatch
    );
    fs::write(snapshot.root().join("a.txt"), b"other").unwrap();
    assert!(matches!(
        subjects::verify(
            &evidence,
            &snapshot,
            SubjectFileV1Origin::Authored,
            &root,
            deadline()
        )
        .unwrap_err()
        .kind,
        ErrorKind::Snapshot(_)
    ));
}

#[test]
fn retained_subject_payload_loss_refuses_despite_matching_source_bytes() {
    let (area, store, snapshot) = setup();
    let mut evidence = Evidence::new(&store, deadline());
    let root = subjects::publish(
        &mut evidence,
        &snapshot,
        SubjectFileV1Origin::Authored,
        deadline(),
    )
    .unwrap();
    let subject: SubjectV1 = receipt::decode(&read(&evidence, root.as_ref())).unwrap();
    let page: SubjectFilePageV1 =
        receipt::decode(&read(&evidence, subject.files.as_ref())).unwrap();
    let payload = page.rows.as_slice()[0].content.value.as_ref().unwrap();
    let hex = payload
        .as_ref()
        .sha256
        .as_str()
        .strip_prefix("sha256:")
        .unwrap();
    fs::remove_file(
        area.base
            .join("store/generations/06700000-0000-4000-8000-000000000001/objects/sha256")
            .join(&hex[..2])
            .join(hex),
    )
    .unwrap();
    assert!(matches!(
        subjects::verify(
            &evidence,
            &snapshot,
            SubjectFileV1Origin::Authored,
            &root,
            deadline()
        )
        .unwrap_err()
        .kind,
        ErrorKind::Graph(_)
    ));
}
