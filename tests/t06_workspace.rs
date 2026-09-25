//! Independent filesystem oracles; design.md was frozen before bodies.
//! No workspace implementation or private security draft was read. Completed
//! filesystem mutations are not asynchronous-race proof. Child controls alter
//! only their own limits/umask; fixture teardown is separate from API retention.

use habitat_engine::worker::workspace::{Content, Error, FileIdentity, Materialized, Snapshot};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{self, DirBuilder, File, FileTimes, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt, symlink};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

static NEXT_AREA: AtomicU64 = AtomicU64::new(0);
const ABC_SHA: [u8; 32] = [
    0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
    0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
];

struct Area {
    path: PathBuf,
    identity: FileIdentity,
}
impl Area {
    fn new() -> Self {
        // A short, canonical Linux temporary base also permits a real Unix socket.
        let path = Path::new("/tmp").canonicalize().unwrap().join(format!(
            "hee3-ws-{}-{}-{}",
            std::process::id(),
            NEXT_AREA.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        private_dir(&path);
        let identity = identity(&path);
        Self { path, identity }
    }
    fn root(&self, name: &str) -> PathBuf {
        let path = self.path.join(name);
        private_dir(&path);
        path
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        let metadata = fs::symlink_metadata(&self.path).unwrap();
        assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
        assert_eq!(identity(&self.path), self.identity);
        remove_owned(&self.path);
    }
}
fn remove_owned(path: &Path) {
    let metadata = fs::symlink_metadata(path).unwrap();
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        for child in fs::read_dir(path).unwrap() {
            remove_owned(&child.unwrap().path());
        }
        fs::remove_dir(path).unwrap();
    } else {
        fs::remove_file(path).unwrap();
    }
}
fn private_dir(path: &Path) {
    DirBuilder::new().mode(0o700).create(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
fn identity(path: &Path) -> FileIdentity {
    let metadata = fs::symlink_metadata(path).unwrap();
    FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}
fn mode(path: &Path) -> u32 {
    fs::symlink_metadata(path).unwrap().mode() & 0o7777
}
fn file(path: &Path, bytes: &[u8], permissions: u32) {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        assert!(metadata.is_file() && !metadata.file_type().is_symlink());
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(permissions)).unwrap();
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
fn expired() -> Instant {
    Instant::now().checked_sub(Duration::from_secs(1)).unwrap()
}
fn capture(root: &Path) -> Snapshot {
    Snapshot::capture(root, &[], deadline()).unwrap()
}
fn content<'a>(snapshot: &'a Snapshot, path: &str) -> &'a Content {
    &snapshot
        .entries()
        .find(|entry| entry.path == path)
        .unwrap()
        .content
}
fn copied(snapshot: &Snapshot, parent: &Path, name: &str, editable: &[String]) -> Materialized {
    snapshot
        .materialize(parent, name, editable, deadline())
        .unwrap_or_else(|failure| panic!("copy failed: {:?}", failure.error))
}
fn simple(area: &Area) -> (PathBuf, Snapshot, PathBuf) {
    let source = area.root("source");
    file(&source.join("plain"), b"abc", 0o600);
    private_dir(&source.join("empty"));
    let snapshot = capture(&source);
    let parent = area.root("copies");
    (source, snapshot, parent)
}

#[test]
fn empty_snapshot_has_no_entries_or_bytes_and_includes_real_root_identity() {
    let area = Area::new();
    let root = area.root("source");
    let snapshot = capture(&root);
    assert_eq!(snapshot.entries().count(), 0);
    assert_eq!(snapshot.total_bytes(), 0);
    assert_eq!(snapshot.source_identities(), vec![identity(&root)]);
}

#[test]
fn exact_bytes_known_sha_and_any_execute_bit_are_captured_without_normalization() {
    let area = Area::new();
    let root = area.root("source");
    file(&root.join("plain"), b"abc", 0o644);
    file(&root.join("group-executable"), b"abc", 0o610);
    let snapshot = capture(&root);
    assert_eq!(snapshot.total_bytes(), 6);
    assert_eq!(
        content(&snapshot, "plain"),
        &Content::File {
            bytes: b"abc".to_vec(),
            sha256: ABC_SHA,
            executable: false
        }
    );
    assert_eq!(
        content(&snapshot, "group-executable"),
        &Content::File {
            bytes: b"abc".to_vec(),
            sha256: ABC_SHA,
            executable: true
        }
    );
    for entry in snapshot.entries() {
        assert_eq!(entry.source_identity(), identity(&root.join(&entry.path)));
    }
}

#[test]
fn root_must_be_absolute_canonical_and_a_real_directory() {
    let area = Area::new();
    let root = area.root("source");
    assert!(Snapshot::capture(&root.join("."), &[], deadline()).is_err());
    assert!(Snapshot::capture(Path::new("relative-root"), &[], deadline()).is_err());
    file(&area.path.join("regular"), b"x", 0o600);
    assert!(Snapshot::capture(&area.path.join("regular"), &[], deadline()).is_err());
    symlink(&root, area.path.join("alias")).unwrap();
    assert!(Snapshot::capture(&area.path.join("alias"), &[], deadline()).is_err());
    assert_eq!(capture(&root).entries().count(), 0);
}

#[test]
fn canonical_root_requires_exact_private_mode_0700() {
    let area = Area::new();
    let root = area.root("source");
    for permissions in [0o750, 0o770, 0o755, 0o777] {
        fs::set_permissions(&root, fs::Permissions::from_mode(permissions)).unwrap();
        assert!(Snapshot::capture(&root, &[], deadline()).is_err());
    }
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(capture(&root).entries().count(), 0);
}

#[test]
fn group_or_world_writable_files_and_nested_directories_refuse_capture() {
    let area = Area::new();
    let root = area.root("source");
    file(&root.join("item"), b"x", 0o600);
    private_dir(&root.join("directory"));
    for permissions in [0o660, 0o602] {
        fs::set_permissions(root.join("item"), fs::Permissions::from_mode(permissions)).unwrap();
        assert!(Snapshot::capture(&root, &[], deadline()).is_err());
    }
    fs::set_permissions(root.join("item"), fs::Permissions::from_mode(0o644)).unwrap();
    for permissions in [0o770, 0o702] {
        fs::set_permissions(
            root.join("directory"),
            fs::Permissions::from_mode(permissions),
        )
        .unwrap();
        assert!(Snapshot::capture(&root, &[], deadline()).is_err());
    }
    fs::set_permissions(root.join("directory"), fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(capture(&root).entries().count(), 2);
}

#[test]
fn file_directory_and_dangling_symlinks_are_never_followed() {
    for target in ["file", "directory", "missing"] {
        let area = Area::new();
        let root = area.root("source");
        file(&root.join("file"), b"x", 0o600);
        private_dir(&root.join("directory"));
        symlink(target, root.join("link")).unwrap();
        assert!(Snapshot::capture(&root, &[], deadline()).is_err());
    }
}

#[test]
fn real_special_files_are_refused_without_connecting_or_executing_them() {
    let area = Area::new();
    let root = area.root("source");
    let listener = UnixListener::bind(root.join("socket")).unwrap();
    assert!(Snapshot::capture(&root, &[], deadline()).is_err());
    drop(listener);
}

#[test]
fn non_utf8_filesystem_names_refuse_instead_of_lossy_replacement() {
    let area = Area::new();
    let root = area.root("source");
    file(
        &root.join(OsString::from_vec(vec![0xff, 0xfe])),
        b"x",
        0o600,
    );
    assert!(Snapshot::capture(&root, &[], deadline()).is_err());
}

#[test]
fn hardlinked_regular_files_refuse_even_when_both_names_are_inside_root() {
    let area = Area::new();
    let root = area.root("source");
    file(&root.join("one"), b"abc", 0o600);
    fs::hard_link(root.join("one"), root.join("two")).unwrap();
    assert_eq!(identity(&root.join("one")), identity(&root.join("two")));
    assert_eq!(fs::metadata(root.join("one")).unwrap().nlink(), 2);
    assert!(Snapshot::capture(&root, &[], deadline()).is_err());
}

#[test]
fn protected_aliases_cover_root_directory_and_file_with_real_foreign_benign() {
    let area = Area::new();
    let root = area.root("source");
    private_dir(&root.join("nested"));
    file(&root.join("file"), b"x", 0o600);
    for path in [&root, &root.join("nested"), &root.join("file")] {
        assert!(Snapshot::capture(&root, &[identity(path)], deadline()).is_err());
    }
    let foreign = area.root("foreign");
    assert_eq!(
        Snapshot::capture(&root, &[identity(&foreign)], deadline())
            .unwrap()
            .entries()
            .count(),
        2
    );
}

#[test]
fn stable_readback_survives_read_only_access_and_preserves_all_original_bytes() {
    let area = Area::new();
    let root = area.root("source");
    file(&root.join("file"), b"abc", 0o600);
    let snapshot = capture(&root);
    assert_eq!(fs::read(root.join("file")).unwrap(), b"abc");
    snapshot.readback_source(deadline()).unwrap();
    assert_eq!(
        content(&snapshot, "file"),
        &Content::File {
            bytes: b"abc".to_vec(),
            sha256: ABC_SHA,
            executable: false
        }
    );
}

#[test]
fn same_length_content_change_is_detected_before_source_reuse() {
    let area = Area::new();
    let root = area.root("source");
    file(&root.join("file"), b"abc", 0o600);
    let snapshot = capture(&root);
    file(&root.join("file"), b"abd", 0o600);
    assert!(snapshot.readback_source(deadline()).is_err());
}

#[test]
fn same_bytes_in_new_inode_are_not_the_original_captured_source() {
    let area = Area::new();
    let root = area.root("source");
    file(&root.join("file"), b"abc", 0o600);
    let original = identity(&root.join("file"));
    let snapshot = capture(&root);
    file(&area.path.join("replacement"), b"abc", 0o600);
    assert_ne!(identity(&area.path.join("replacement")), original);
    fs::rename(area.path.join("replacement"), root.join("file")).unwrap();
    assert_eq!(fs::read(root.join("file")).unwrap(), b"abc");
    assert!(snapshot.readback_source(deadline()).is_err());
}

#[test]
fn source_mode_and_timestamp_changes_are_binding_even_with_unchanged_bytes() {
    for timestamp in [false, true] {
        let area = Area::new();
        let root = area.root("source");
        file(&root.join("file"), b"abc", 0o600);
        let snapshot = capture(&root);
        if timestamp {
            File::open(root.join("file"))
                .unwrap()
                .set_times(
                    FileTimes::new().set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
                )
                .unwrap();
        } else {
            fs::set_permissions(root.join("file"), fs::Permissions::from_mode(0o400)).unwrap();
        }
        assert_eq!(fs::read(root.join("file")).unwrap(), b"abc");
        assert!(snapshot.readback_source(deadline()).is_err());
    }
}

#[test]
fn source_inventory_addition_deletion_and_directory_rename_are_detected() {
    for change in ["add", "delete", "rename"] {
        let area = Area::new();
        let root = area.root("source");
        file(&root.join("file"), b"abc", 0o600);
        private_dir(&root.join("directory"));
        let snapshot = capture(&root);
        match change {
            "add" => file(&root.join("new"), b"new", 0o600),
            "delete" => fs::remove_file(root.join("file")).unwrap(),
            _ => fs::rename(root.join("directory"), root.join("renamed")).unwrap(),
        }
        assert!(snapshot.readback_source(deadline()).is_err());
    }
}

#[test]
fn replaced_root_with_same_named_bytes_is_not_the_original_custody_identity() {
    let area = Area::new();
    let root = area.root("source");
    file(&root.join("file"), b"abc", 0o600);
    let snapshot = capture(&root);
    fs::rename(&root, area.path.join("old-source")).unwrap();
    private_dir(&root);
    file(&root.join("file"), b"abc", 0o600);
    assert!(snapshot.readback_source(deadline()).is_err());
}

#[test]
fn entry_and_protected_identity_limits_use_their_distinct_inclusive_denominators() {
    let area = Area::new();
    let root = area.root("source");
    for index in 0..4096 {
        file(&root.join(format!("f{index:04}")), b"", 0o600);
    }
    let snapshot = capture(&root);
    assert_eq!(snapshot.entries().count(), 4096);
    let mut protected = snapshot.source_identities();
    assert_eq!(protected.len(), 4097);
    let other = area.root("other");
    assert_eq!(
        Snapshot::capture(&other, &protected, deadline())
            .unwrap()
            .entries()
            .count(),
        0
    );
    let extra = area.root("extra");
    protected.push(identity(&extra));
    assert!(Snapshot::capture(&other, &protected, deadline()).is_err());
    file(&root.join("overflow"), b"", 0o600);
    assert!(Snapshot::capture(&root, &[], deadline()).is_err());
}

#[test]
fn path_depth_32_is_allowed_and_depth_33_refuses() {
    let area = Area::new();
    let root = area.root("source");
    let mut deepest = root.clone();
    for _ in 0..31 {
        deepest = deepest.join("d");
        private_dir(&deepest);
    }
    file(&deepest.join("leaf"), b"x", 0o600);
    assert_eq!(capture(&root).entries().count(), 32);
    let beyond = deepest.join("deeper");
    private_dir(&beyond);
    file(&beyond.join("leaf"), b"x", 0o600);
    assert!(Snapshot::capture(&root, &[], deadline()).is_err());
}

#[test]
fn file_byte_limit_accepts_16_mib_and_refuses_one_more_byte() {
    let area = Area::new();
    let root = area.root("source");
    file(&root.join("file"), &vec![7; 16_777_216], 0o600);
    assert_eq!(capture(&root).total_bytes(), 16_777_216);
    OpenOptions::new()
        .append(true)
        .open(root.join("file"))
        .unwrap()
        .write_all(&[7])
        .unwrap();
    assert!(Snapshot::capture(&root, &[], deadline()).is_err());
}

#[test]
fn aggregate_byte_limit_includes_every_file_and_allows_exactly_64_mib() {
    let area = Area::new();
    let root = area.root("source");
    for index in 0..4 {
        file(&root.join(index.to_string()), &vec![7; 16_777_216], 0o600);
    }
    assert_eq!(capture(&root).total_bytes(), 67_108_864);
    file(&root.join("extra"), b"x", 0o600);
    assert!(Snapshot::capture(&root, &[], deadline()).is_err());
}

#[test]
fn entry_order_is_ascending_utf8_bytes_independent_of_creation_order() {
    let area = Area::new();
    let root = area.root("source");
    for name in ["目录", "é", "a", "Z"] {
        file(&root.join(name), b"x", 0o600);
    }
    let snapshot = capture(&root);
    let names: Vec<_> = snapshot
        .entries()
        .map(|entry| entry.path.as_str())
        .collect();
    assert_eq!(names, ["Z", "a", "é", "目录"]);
}

#[test]
fn copies_use_fresh_inodes_exact_private_modes_and_leave_source_unchanged() {
    let area = Area::new();
    let source = area.root("source");
    private_dir(&source.join("sub"));
    file(&source.join("plain"), b"abc", 0o644);
    file(&source.join("sub/tool"), b"abc", 0o610);
    let snapshot = capture(&source);
    let parent = area.root("copies");
    let mut seen: BTreeSet<_> = snapshot.source_identities().into_iter().collect();
    for (name, editable, plain_mode, tool_mode) in [
        ("first", vec!["plain".to_owned()], 0o600, 0o500),
        ("second", vec!["sub/tool".to_owned()], 0o400, 0o700),
    ] {
        let output = copied(&snapshot, &parent, name, &editable);
        assert_eq!(mode(&output.path), 0o700);
        assert_eq!(mode(&output.path.join("sub")), 0o700);
        assert_eq!(mode(&output.path.join("plain")), plain_mode);
        assert_eq!(mode(&output.path.join("sub/tool")), tool_mode);
        let result = capture(&output.path);
        for inode in result.source_identities() {
            assert!(seen.insert(inode), "copy alias detected");
        }
        assert_eq!(content(&result, "plain"), content(&snapshot, "plain"));
        assert_eq!(content(&result, "sub/tool"), content(&snapshot, "sub/tool"));
    }
    assert_eq!(mode(&source.join("plain")), 0o644);
    assert_eq!(mode(&source.join("sub/tool")), 0o610);
    snapshot.readback_source(deadline()).unwrap();
}

#[test]
fn existing_destination_is_not_overwritten_or_claimed_as_partial_output() {
    let area = Area::new();
    let (_source, snapshot, parent) = simple(&area);
    let existing = parent.join("taken");
    private_dir(&existing);
    file(&existing.join("sentinel"), b"leave unchanged", 0o600);
    let before = identity(&existing);
    let failure = snapshot
        .materialize(&parent, "taken", &[], deadline())
        .expect_err("existing output must refuse");
    assert!(failure.partial_path.is_none());
    assert_eq!(identity(&existing), before);
    assert_eq!(
        fs::read(existing.join("sentinel")).unwrap(),
        b"leave unchanged"
    );
}

#[test]
fn destination_names_are_one_fresh_component_and_invalid_names_create_nothing() {
    let area = Area::new();
    let (_source, snapshot, parent) = simple(&area);
    for name in ["", ".", "..", "a/b", "/absolute", "nul\0name"] {
        let failure = snapshot
            .materialize(&parent, name, &[], deadline())
            .expect_err("invalid name must refuse");
        assert!(failure.partial_path.is_none());
        assert_eq!(fs::read_dir(&parent).unwrap().count(), 0);
    }
    let output = copied(&snapshot, &parent, "résultat", &[]);
    assert_eq!(output.path, parent.join("résultat"));
}

#[test]
fn editable_policy_rejects_duplicates_traversal_missing_and_directory_entries() {
    let area = Area::new();
    let (source, snapshot, parent) = simple(&area);
    for editable in [
        vec!["plain".to_owned(), "plain".to_owned()],
        vec!["../plain".to_owned()],
        vec!["missing".to_owned()],
        vec!["empty".to_owned()],
        vec![source.join("plain").to_string_lossy().into_owned()],
    ] {
        let failure = snapshot
            .materialize(&parent, "copy", &editable, deadline())
            .expect_err("invalid editable policy");
        assert_eq!(failure.error, Error::Policy);
        assert!(failure.partial_path.is_none());
        assert!(!parent.join("copy").exists());
    }
}

#[test]
fn materialize_cannot_create_output_inside_the_captured_source_inventory() {
    let area = Area::new();
    let (source, snapshot, _parent) = simple(&area);
    for parent in [&source, &source.join("empty")] {
        let failure = snapshot
            .materialize(parent, "copy", &[], deadline())
            .expect_err("source ancestry must refuse");
        assert_eq!(failure.error, Error::Policy);
        assert!(failure.partial_path.is_none());
    }
    snapshot.readback_source(deadline()).unwrap();
}

#[test]
fn result_permission_only_changes_do_not_count_as_content_changes() {
    let area = Area::new();
    let (_source, snapshot, parent) = simple(&area);
    let output = copied(&snapshot, &parent, "copy", &[]);
    fs::set_permissions(output.path.join("plain"), fs::Permissions::from_mode(0o644)).unwrap();
    fs::set_permissions(output.path.join("empty"), fs::Permissions::from_mode(0o755)).unwrap();
    let result = snapshot
        .validate_result(&output.path, &[], deadline())
        .unwrap();
    assert_eq!(content(&result, "plain"), content(&snapshot, "plain"));
}

#[test]
fn changed_content_requires_the_explicit_result_allowlist_even_if_copy_was_editable() {
    let area = Area::new();
    let (_source, snapshot, parent) = simple(&area);
    let output = copied(&snapshot, &parent, "copy", &["plain".to_owned()]);
    file(&output.path.join("plain"), b"changed", 0o600);
    assert!(
        snapshot
            .validate_result(&output.path, &[], deadline())
            .is_err()
    );
    let result = snapshot
        .validate_result(&output.path, &["plain".to_owned()], deadline())
        .unwrap();
    assert!(matches!(content(&result,"plain"),Content::File{bytes,..} if bytes==b"changed"));
    snapshot.readback_source(deadline()).unwrap();
}

#[test]
fn executable_semantics_may_change_only_on_an_allowlisted_regular_file() {
    let area = Area::new();
    let (_source, snapshot, parent) = simple(&area);
    let output = copied(&snapshot, &parent, "copy", &[]);
    fs::set_permissions(output.path.join("plain"), fs::Permissions::from_mode(0o500)).unwrap();
    assert!(
        snapshot
            .validate_result(&output.path, &[], deadline())
            .is_err()
    );
    let result = snapshot
        .validate_result(&output.path, &["plain".to_owned()], deadline())
        .unwrap();
    assert!(matches!(
        content(&result, "plain"),
        Content::File {
            executable: true,
            ..
        }
    ));
}

#[test]
fn new_missing_renamed_or_changed_type_paths_cannot_expand_result_scope() {
    for change in ["new", "missing", "renamed", "type"] {
        let area = Area::new();
        let (_source, snapshot, parent) = simple(&area);
        let output = copied(&snapshot, &parent, "copy", &[]);
        match change {
            "new" => file(&output.path.join("new"), b"new", 0o600),
            "missing" => fs::remove_file(output.path.join("plain")).unwrap(),
            "renamed" => {
                fs::rename(output.path.join("plain"), output.path.join("renamed")).unwrap();
            }
            _ => {
                fs::remove_file(output.path.join("plain")).unwrap();
                private_dir(&output.path.join("plain"));
            }
        }
        assert!(
            snapshot
                .validate_result(&output.path, &["plain".to_owned()], deadline())
                .is_err()
        );
    }
}

#[test]
fn result_source_aliases_and_writable_custody_cannot_be_authorized_by_allowlist() {
    let area = Area::new();
    let (source, snapshot, parent) = simple(&area);
    assert!(
        snapshot
            .validate_result(&source, &["plain".to_owned()], deadline())
            .is_err()
    );
    let output = copied(&snapshot, &parent, "copy", &[]);
    fs::set_permissions(output.path.join("plain"), fs::Permissions::from_mode(0o666)).unwrap();
    assert!(
        snapshot
            .validate_result(&output.path, &["plain".to_owned()], deadline())
            .is_err()
    );
    fs::remove_file(output.path.join("plain")).unwrap();
    fs::hard_link(source.join("plain"), output.path.join("plain")).unwrap();
    assert!(
        snapshot
            .validate_result(&output.path, &["plain".to_owned()], deadline())
            .is_err()
    );
}

#[test]
fn result_allowlist_itself_cannot_name_unknown_directory_duplicate_or_traversal_paths() {
    let area = Area::new();
    let (_source, snapshot, parent) = simple(&area);
    let output = copied(&snapshot, &parent, "copy", &[]);
    for allowed in [
        vec!["missing".to_owned()],
        vec!["empty".to_owned()],
        vec!["../plain".to_owned()],
        vec!["plain".to_owned(), "plain".to_owned()],
    ] {
        assert!(
            snapshot
                .validate_result(&output.path, &allowed, deadline())
                .is_err()
        );
    }
}

#[test]
fn expired_deadlines_refuse_capture_readback_materialize_and_result_validation() {
    let area = Area::new();
    let (source, snapshot, parent) = simple(&area);
    let output = copied(&snapshot, &parent, "valid", &[]);
    assert!(matches!(
        Snapshot::capture(&source, &[], expired()),
        Err(Error::Deadline)
    ));
    assert_eq!(snapshot.readback_source(expired()), Err(Error::Deadline));
    let failure = snapshot
        .materialize(&parent, "expired", &[], expired())
        .expect_err("expired copy must refuse");
    assert_eq!(failure.error, Error::Deadline);
    assert!(failure.partial_path.is_none());
    assert!(!parent.join("expired").exists());
    assert!(matches!(
        snapshot.validate_result(&output.path, &[], expired()),
        Err(Error::Deadline)
    ));
}

fn bounded_child(script: &str, marker: &str, source: &Path, parent: &Path) {
    let mut child = Command::new("/usr/bin/bash")
        .args(["-c", script, "hee3-workspace-oracle"])
        .arg(std::env::current_exe().unwrap())
        .env("HEE3_WORKSPACE_CHILD", marker)
        .env("HEE3_WORKSPACE_SOURCE", source)
        .env("HEE3_WORKSPACE_PARENT", parent)
        .env("RUST_BACKTRACE", "0")
        .env("RUST_LIB_BACKTRACE", "0")
        .env_remove("POSIXLY_CORRECT")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let readers = [
        thread::spawn(move || bounded_stream(stdout)),
        thread::spawn(move || bounded_stream(stderr)),
    ];
    let cutoff = Instant::now() + Duration::from_secs(20);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= cutoff {
            timed_out = true;
            child.kill().unwrap();
            break child.wait().unwrap();
        }
        thread::sleep(Duration::from_millis(10));
    };
    let [stdout, stderr] = readers.map(|reader| reader.join().unwrap());
    assert!(!timed_out, "isolated child exceeded its finite deadline");
    assert!(stdout.len() <= 65_536 && stderr.len() <= 65_536);
    let observed = String::from_utf8(stdout).unwrap();
    let diagnostics = String::from_utf8(stderr).unwrap();
    assert!(
        status.success(),
        "child {status}: {observed}\n{diagnostics}"
    );
    assert!(
        diagnostics.is_empty(),
        "unexpected child diagnostics: {diagnostics}"
    );
    assert!(
        observed.contains("test result: ok. 1 passed; 0 failed"),
        "missing exact child selection: {observed}"
    );
    assert!(
        observed.contains(marker),
        "missing actual child observation: {observed}"
    );
}

fn bounded_stream(reader: impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    reader.take(65_537).read_to_end(&mut bytes).unwrap();
    bytes
}

fn child_paths(marker: &str) -> Option<(PathBuf, PathBuf)> {
    if std::env::var("HEE3_WORKSPACE_CHILD").ok().as_deref() != Some(marker) {
        return None;
    }
    Some((
        PathBuf::from(std::env::var_os("HEE3_WORKSPACE_SOURCE").unwrap()),
        PathBuf::from(std::env::var_os("HEE3_WORKSPACE_PARENT").unwrap()),
    ))
}

#[test]
fn postcreation_file_size_fault_retains_partial_output() {
    const MARKER: &str = "hee3-workspace-retained-size-fault";
    if let Some((source, parent)) = child_paths(MARKER) {
        let snapshot = capture(&source);
        let failure = snapshot
            .materialize(&parent, "partial", &[], deadline())
            .expect_err("file-size-limited materialize must fail");
        assert_eq!(failure.error, Error::Io);
        let retained = failure
            .partial_path
            .expect("created output must be retained");
        assert_eq!(retained, parent.join("partial"));
        assert!(retained.is_dir());
        assert_eq!(fs::read(source.join("payload")).unwrap(), vec![0x61; 4096]);
        let entries: Vec<_> = fs::read_dir(&retained)
            .unwrap()
            .map(|entry| fs::symlink_metadata(entry.unwrap().path()).unwrap())
            .collect();
        // Retention does not promise a staging filename or a minimum partial length.
        assert!(
            entries
                .iter()
                .all(|entry| entry.is_file() && entry.len() <= 1024)
        );
        println!(
            "{MARKER}: retained={} files={} bytes={}",
            retained.display(),
            entries.len(),
            entries.iter().map(std::fs::Metadata::len).sum::<u64>()
        );
        return;
    }
    let area = Area::new();
    let source = area.root("source");
    let parent = area.root("copies");
    file(&source.join("payload"), &vec![0x61; 4096], 0o600);
    bounded_child(
        "set -eu\ntrap '' XFSZ\nulimit -f 1\nexec \"$1\" --exact postcreation_file_size_fault_retains_partial_output --nocapture --test-threads=1",
        MARKER,
        &source,
        &parent,
    );
    assert!(parent.join("partial").is_dir());
    assert_eq!(fs::read(source.join("payload")).unwrap(), vec![0x61; 4096]);
}

#[test]
fn child_umask_077_preserves_successful_private_modes() {
    const MARKER: &str = "hee3-workspace-private-umask-success";
    if let Some((source, parent)) = child_paths(MARKER) {
        let snapshot = capture(&source);
        let output = copied(&snapshot, &parent, "copy", &["plain".to_owned()]);
        assert_eq!(mode(&output.path), 0o700);
        assert_eq!(mode(&output.path.join("directory")), 0o700);
        assert_eq!(mode(&output.path.join("plain")), 0o600);
        assert_eq!(mode(&output.path.join("directory/tool")), 0o500);
        snapshot.readback_source(deadline()).unwrap();
        println!("{MARKER}");
        return;
    }
    let area = Area::new();
    let source = area.root("source");
    let parent = area.root("copies");
    private_dir(&source.join("directory"));
    file(&source.join("plain"), b"plain", 0o600);
    file(&source.join("directory/tool"), b"tool", 0o700);
    bounded_child(
        "set -eu\numask 077\nexec \"$1\" --exact child_umask_077_preserves_successful_private_modes --nocapture --test-threads=1",
        MARKER,
        &source,
        &parent,
    );
    assert_eq!(mode(&parent.join("copy")), 0o700);
    assert_eq!(mode(&parent.join("copy/plain")), 0o600);
    assert_eq!(mode(&parent.join("copy/directory/tool")), 0o500);
}

/// Build the digest fixture's tree (tests/fixtures/digest/cases.json) under a fresh private root.
fn digest_tree(area: &Area, name: &str, entries: &serde_json::Value) -> PathBuf {
    let root = area.root(name);
    for entry in entries.as_array().unwrap() {
        let path = root.join(entry["path"].as_str().unwrap());
        let mode = u32::try_from(entry["mode"].as_u64().unwrap()).unwrap();
        if entry["kind"] == "directory" {
            private_dir(&path);
        } else {
            file(&path, entry["content"].as_str().unwrap().as_bytes(), mode);
        }
    }
    root
}

fn sha256_text(text: &str) -> String {
    use sha2::Digest;
    let digest = sha2::Sha256::digest(text.as_bytes());
    digest
        .iter()
        .fold(String::from("sha256:"), |mut text, byte| {
            use std::fmt::Write as _;
            // Writing to a `String` cannot fail.
            let _ = write!(text, "{byte:02x}");
            text
        })
}

/// B14-P2a · a snapshot's content digest is the one a coreutils pipeline gives the same tree
/// (`gen-digest-fixtures.sh`: find, sha256sum, `LC_ALL=C sort`), never this module's reading of
/// its own rule: equal over two roots (stamps and the root are not content), the empty tree's is
/// the digest of no text, and each field — contents, the execute bit, a name, an empty directory —
/// moves it exactly as editing that line of the world's manifest does. A name holding a C0 control
/// has no digest.
#[test]
fn the_content_digest_is_the_coreutils_manifest_s() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/digest/cases.json")).unwrap();
    let (entries, manifest) = (
        &fixture["tree"]["entries"],
        fixture["tree"]["manifest"].as_str().unwrap(),
    );
    let expected = format!("sha256:{}", fixture["tree"]["digest"].as_str().unwrap());
    assert_eq!(
        sha256_text(manifest),
        expected,
        "the fixture's digest is its manifest's"
    );
    let area = Area::new();
    let one = digest_tree(&area, "one", entries);
    let two = digest_tree(&area, "two", entries);
    assert_eq!(capture(&one).content_digest(), Some(expected.clone()));
    assert_eq!(capture(&two).content_digest(), Some(expected));
    assert_eq!(
        capture(&area.root("none")).content_digest(),
        Some(format!(
            "sha256:{}",
            fixture["empty"]["digest"].as_str().unwrap()
        ))
    );
    let line = |path: &str| {
        manifest
            .lines()
            .find(|line| line.split('\t').next() == Some(path))
            .unwrap()
            .to_owned()
    };
    // Contents: `a b` holds other bytes.
    let changed = digest_tree(&area, "contents", entries);
    file(&changed.join("a b"), b"other\n", 0o600);
    let other = sha256_text("other\n");
    let edited = manifest.replace(
        &line("a b"),
        &format!("a b\tf\t-\t{}", &other["sha256:".len()..]),
    );
    assert_eq!(
        capture(&changed).content_digest(),
        Some(sha256_text(&edited))
    );
    // The execute bit: run.sh loses it.
    let plain = digest_tree(&area, "mode", entries);
    fs::set_permissions(plain.join("run.sh"), fs::Permissions::from_mode(0o600)).unwrap();
    let edited = manifest.replace(
        &line("run.sh"),
        &line("run.sh").replace("\tf\tx\t", "\tf\t-\t"),
    );
    assert_eq!(capture(&plain).content_digest(), Some(sha256_text(&edited)));
    // A name: `é` becomes `ê` (0xc3 0xaa, still the last line, so only the name moves).
    let renamed = digest_tree(&area, "name", entries);
    fs::rename(renamed.join("é"), renamed.join("ê")).unwrap();
    let edited = manifest.replace(&line("é"), &line("é").replacen('é', "ê", 1));
    assert_eq!(
        capture(&renamed).content_digest(),
        Some(sha256_text(&edited))
    );
    // An empty directory: `empty` removed, its line gone.
    let fewer = digest_tree(&area, "directory", entries);
    fs::remove_dir(fewer.join("empty")).unwrap();
    let edited = manifest.replace(&format!("{}\n", line("empty")), "");
    assert_eq!(capture(&fewer).content_digest(), Some(sha256_text(&edited)));
    // A control byte in a name: no digest.
    let control = digest_tree(&area, "control", entries);
    file(&control.join("a\u{1}"), b"c\n", 0o600);
    assert_eq!(capture(&control).content_digest(), None);
    let del = digest_tree(&area, "del", entries);
    file(&del.join("a\u{7f}"), b"c\n", 0o600);
    assert_eq!(capture(&del).content_digest(), None);
    let separator = digest_tree(&area, "separator", entries);
    file(&separator.join("a\u{1f}b"), b"c\n", 0o600);
    assert_eq!(capture(&separator).content_digest(), None);
}
