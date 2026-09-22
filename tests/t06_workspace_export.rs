//! Author controls for retained-FD export. Filesystem helpers and bounded child
//! shape are adapted from the independent T06 workspace oracle suite. These new
//! controls are implementation-authored and claim no namespace-exit or race proof.

use habitat_engine::worker::workspace::{Error, FileIdentity, export_directory};
use rustix::fs::{Mode, OFlags, open};
use std::fs::{self, DirBuilder, File};
use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt, symlink};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Area {
    path: PathBuf,
    identity: FileIdentity,
}
impl Area {
    fn new(base: &str) -> Self {
        let path = Path::new(base).canonicalize().unwrap().join(format!(
            "hee3-export-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        private_dir(&path);
        Self {
            identity: identity(&path),
            path,
        }
    }
    fn root(&self, name: &str) -> PathBuf {
        let path = self.path.join(name);
        private_dir(&path);
        path
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        assert_eq!(identity(&self.path), self.identity);
        remove_owned(&self.path);
    }
}
fn remove_owned(path: &Path) {
    let meta = fs::symlink_metadata(path).unwrap();
    if meta.is_dir() && !meta.file_type().is_symlink() {
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
    let m = fs::symlink_metadata(path).unwrap();
    FileIdentity {
        device: m.dev(),
        inode: m.ino(),
    }
}
fn write(path: &Path, bytes: &[u8], mode: u32) {
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
fn setup() -> (Area, PathBuf, Area) {
    let source = Area::new("/dev/shm");
    let root = source.root("source");
    let target = Area::new("/tmp");
    (source, root, target)
}

#[test]
fn retained_fd_ignores_replaced_path_and_exports_exact_fresh_frozen_tree() {
    let (source, root, target) = setup();
    private_dir(&root.join("z"));
    write(&root.join("z/tool"), b"original", 0o700);
    write(&root.join("a"), b"2\n", 0o600);
    let descriptor = File::open(&root).unwrap();
    let held_path = source.path.join("moved");
    fs::rename(&root, &held_path).unwrap();
    private_dir(&root);
    write(&root.join("a"), b"substitution", 0o600);
    let output = export_directory(&descriptor, &target.path, "copy", &[], deadline()).unwrap();
    assert_eq!(fs::read(output.path.join("a")).unwrap(), b"2\n");
    assert_eq!(fs::read(output.path.join("z/tool")).unwrap(), b"original");
    assert_eq!(
        fs::metadata(output.path.join("a")).unwrap().mode() & 0o7777,
        0o400
    );
    assert_eq!(
        fs::metadata(output.path.join("z/tool")).unwrap().mode() & 0o7777,
        0o500
    );
    assert_eq!(
        fs::metadata(output.path.join("z")).unwrap().mode() & 0o7777,
        0o700
    );
    assert_ne!(identity(&output.path), identity(&held_path));
    assert_ne!(
        identity(&output.path.join("a")),
        identity(&held_path.join("a"))
    );
    assert_eq!(fs::read(root.join("a")).unwrap(), b"substitution");
}

#[test]
fn path_only_descriptor_and_nonprivate_root_refuse_without_output() {
    let (_source, root, target) = setup();
    let path_only =
        File::from(open(&root, OFlags::PATH | OFlags::DIRECTORY, Mode::empty()).unwrap());
    let error =
        export_directory(&path_only, &target.path, "path_only", &[], deadline()).unwrap_err();
    assert_eq!(error.error, Error::Custody);
    assert!(error.partial_path.is_none());
    let readable = File::open(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o750)).unwrap();
    let error =
        export_directory(&readable, &target.path, "nonprivate", &[], deadline()).unwrap_err();
    assert_eq!(error.error, Error::Custody);
    assert!(error.partial_path.is_none());
    assert_eq!(fs::read_dir(&target.path).unwrap().count(), 0);
}

#[test]
fn protected_root_and_nested_file_inodes_refuse_before_creation() {
    let (_source, root, target) = setup();
    write(&root.join("value"), b"value", 0o600);
    let descriptor = File::open(&root).unwrap();
    for protected in [identity(&root), identity(&root.join("value"))] {
        let error = export_directory(&descriptor, &target.path, "copy", &[protected], deadline())
            .unwrap_err();
        assert_eq!(error.error, Error::Alias);
        assert!(error.partial_path.is_none());
    }
    assert!(!target.path.join("copy").exists());
}

#[test]
fn destination_inside_captured_tree_and_protected_parent_refuse() {
    let (_source, root, target) = setup();
    private_dir(&root.join("nested"));
    let descriptor = File::open(&root).unwrap();
    for parent in [&root, &root.join("nested")] {
        let error = export_directory(&descriptor, parent, "copy", &[], deadline()).unwrap_err();
        assert_eq!(error.error, Error::Policy);
        assert!(error.partial_path.is_none());
    }
    let error = export_directory(
        &descriptor,
        &target.path,
        "copy",
        &[identity(&target.path)],
        deadline(),
    )
    .unwrap_err();
    assert_eq!(error.error, Error::Alias);
    assert!(error.partial_path.is_none());
}

#[test]
fn symlink_hardlink_and_socket_are_not_exportable_payloads() {
    let (_source, root, target) = setup();
    write(&root.join("file"), b"value", 0o600);
    symlink("file", root.join("link")).unwrap();
    let descriptor = File::open(&root).unwrap();
    assert_eq!(
        export_directory(&descriptor, &target.path, "link", &[], deadline())
            .unwrap_err()
            .error,
        Error::Type
    );
    fs::remove_file(root.join("link")).unwrap();
    fs::hard_link(root.join("file"), root.join("hard")).unwrap();
    assert_eq!(
        export_directory(&descriptor, &target.path, "hard", &[], deadline())
            .unwrap_err()
            .error,
        Error::Alias
    );
    fs::remove_file(root.join("hard")).unwrap();
    let socket = UnixListener::bind(root.join("socket")).unwrap();
    assert_eq!(
        export_directory(&descriptor, &target.path, "socket", &[], deadline())
            .unwrap_err()
            .error,
        Error::Type
    );
    drop(socket);
    assert_eq!(fs::read_dir(&target.path).unwrap().count(), 0);
}

#[test]
fn existing_output_is_preserved_and_never_reported_as_own_partial() {
    let (_source, root, target) = setup();
    let existing = target.root("copy");
    write(&existing.join("sentinel"), b"keep", 0o600);
    let before = identity(&existing);
    let error = export_directory(
        &File::open(&root).unwrap(),
        &target.path,
        "copy",
        &[],
        deadline(),
    )
    .unwrap_err();
    assert_eq!(error.error, Error::Io);
    assert!(error.partial_path.is_none());
    assert_eq!(identity(&existing), before);
    assert_eq!(fs::read(existing.join("sentinel")).unwrap(), b"keep");
}

#[test]
fn expired_export_refuses_without_output_or_source_change() {
    let (_source, root, target) = setup();
    write(&root.join("value"), b"2\n", 0o600);
    let error = export_directory(
        &File::open(&root).unwrap(),
        &target.path,
        "copy",
        &[],
        Instant::now().checked_sub(Duration::from_secs(1)).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.error, Error::Deadline);
    assert!(error.partial_path.is_none());
    assert!(!target.path.join("copy").exists());
    assert_eq!(fs::read(root.join("value")).unwrap(), b"2\n");
}

#[test]
fn exact_file_bound_succeeds_and_adjacent_larger_file_refuses() {
    let (_source, root, target) = setup();
    let bytes = vec![0x53; 16 * 1024 * 1024];
    write(&root.join("value"), &bytes, 0o600);
    let descriptor = File::open(&root).unwrap();
    let output = export_directory(&descriptor, &target.path, "maximum", &[], deadline()).unwrap();
    assert_eq!(fs::read(output.path.join("value")).unwrap(), bytes);
    File::options()
        .write(true)
        .open(root.join("value"))
        .unwrap()
        .set_len(16 * 1024 * 1024 + 1)
        .unwrap();
    let error = export_directory(&descriptor, &target.path, "excess", &[], deadline()).unwrap_err();
    assert_eq!(error.error, Error::Bound);
    assert!(error.partial_path.is_none());
    assert!(!target.path.join("excess").exists());
}

#[test]
fn postcreation_permission_fault_retains_owned_partial_directory() {
    const MARKER: &str = "HEE3_EXPORT_PERMISSION_CHILD";
    if std::env::var(MARKER).as_deref() == Ok("1") {
        let source = PathBuf::from(std::env::var_os("HEE3_EXPORT_SOURCE").unwrap());
        let parent = PathBuf::from(std::env::var_os("HEE3_EXPORT_PARENT").unwrap());
        let failure = export_directory(
            &File::open(source).unwrap(),
            &parent,
            "partial",
            &[],
            deadline(),
        )
        .unwrap_err();
        assert_eq!(failure.error, Error::Io);
        assert_eq!(failure.partial_path, Some(parent.join("partial")));
        assert_eq!(
            fs::metadata(parent.join("partial")).unwrap().mode() & 0o7777,
            0
        );
        println!("retained-owned-partial");
        return;
    }
    let (_source, root, target) = setup();
    let mut child = Command::new("/usr/bin/bash")
        .args(["-c", "set -eu\numask 0777\nexec \"$1\" --exact postcreation_permission_fault_retains_owned_partial_directory --nocapture --test-threads=1", "hee3-export-oracle"])
        .arg(std::env::current_exe().unwrap()).env(MARKER, "1")
        .env("HEE3_EXPORT_SOURCE", &root).env("HEE3_EXPORT_PARENT", &target.path)
        .env("RUST_BACKTRACE", "0").env("RUST_LIB_BACKTRACE", "0")
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let readers = [
        thread::spawn(move || read_bounded(stdout)),
        thread::spawn(move || read_bounded(stderr)),
    ];
    let cutoff = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= cutoff {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("child exceeded finite deadline");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let [stdout, stderr] = readers.map(|reader| reader.join().unwrap());
    assert!(
        status.success(),
        "child {status}: {} {}",
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    );
    assert!(stderr.is_empty());
    assert!(stdout.len() <= 65_536);
    assert!(
        String::from_utf8(stdout)
            .unwrap()
            .contains("retained-owned-partial")
    );
    assert!(target.path.join("partial").is_dir());
}
fn read_bounded(reader: impl Read) -> Vec<u8> {
    let mut output = Vec::new();
    reader.take(65_537).read_to_end(&mut output).unwrap();
    output
}
