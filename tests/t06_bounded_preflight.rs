//! Adapted from the independent bounded-review public API controls. Dummy tool
//! bytes never execute: the fixed invalid scope pin must refuse before channels.
use habitat_engine::{
    app::workload::{self, Outcome, Plan, Tools},
    worker::{
        namespace::{BwrapPlan, NamespaceError, PublicFile, ReadOnlyFile, prepare_bounded},
        resources::Scope,
        workspace::Snapshot,
    },
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const BAD_PIN: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
fn private(path: &Path) {
    fs::DirBuilder::new().mode(0o700).create(path).unwrap();
}
fn file(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
}
fn pinned(host: PathBuf, namespace: &str) -> ReadOnlyFile {
    ReadOnlyFile {
        sha256: Sha256::digest(fs::read(&host).unwrap()).into(),
        host,
        namespace: namespace.into(),
    }
}
fn scopes() -> [Scope; 3] {
    [
        "11111111-1111-4111-8111-111111111111",
        "22222222-2222-4222-8222-222222222222",
        "33333333-3333-4333-8333-333333333333",
    ]
    .map(|id| Scope {
        systemd_run: "/usr/bin/systemd-run".into(),
        systemd_run_sha256: BAD_PIN.into(),
        runtime_dir: format!("/run/user/{}", rustix::process::geteuid().as_raw()).into(),
        run_id: id.into(),
        aggregate: "hee3boundedcontrols.slice".into(),
    })
}
struct Fixture {
    root: PathBuf,
    source: Snapshot,
    protected: Snapshot,
    tools: Tools,
    cancelled: AtomicBool,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "hee3-bounded-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        private(&root);
        for name in ["source", "protected", "job", "work", "tools"] {
            private(&root.join(name));
        }
        private(&root.join("source/src"));
        file(
            &root.join("source/src/lib.rs"),
            include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/reference/src/lib.rs"),
        );
        file(
            &root.join("protected/oracle.json"),
            include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/oracle/cases.json"),
        );
        file(
            &root.join("protected/public-wrapper.rs"),
            include_bytes!("../evaluation/harnesses/u64-public-wrapper.rs"),
        );
        file(
            &root.join("tools/compiler"),
            b"dummy compiler, never dispatched",
        );
        file(&root.join("tools/shim"), b"dummy shim, never dispatched");
        let deadline = Instant::now() + Duration::from_secs(5);
        let source = Snapshot::capture(&root.join("source"), &[], deadline).unwrap();
        let protected = Snapshot::capture(&root.join("protected"), &[], deadline).unwrap();
        let tools = Tools {
            bwrap: "/usr/bin/bwrap".into(),
            compiler: pinned(root.join("tools/compiler"), "/toolchain/bin/rustc"),
            shim: pinned(root.join("tools/shim"), "/shim/namespace-shim"),
            runtime_files: vec![],
            namespace_directories: vec![],
        };
        Self {
            root,
            source,
            protected,
            tools,
            cancelled: AtomicBool::new(false),
        }
    }
    fn collect(&self, scopes: &[Scope; 3]) -> Result<workload::Run, workload::Error> {
        workload::collect_bounded(
            &Plan {
                source: &self.source,
                protected: &self.protected,
                job_root: &self.root.join("job"),
                tools: &self.tools,
                deadline: Instant::now() + Duration::from_secs(5),
                cancelled: &self.cancelled,
            },
            scopes,
        )
    }
    fn plan(&self) -> BwrapPlan {
        BwrapPlan {
            bwrap: self.tools.bwrap.clone(),
            shim: self.tools.shim.clone(),
            candidate: self.tools.compiler.clone(),
            read_only_files: vec![],
            namespace_directories: [
                "/work",
                "/shim",
                "/toolchain",
                "/toolchain/bin",
                "/channels",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
            work_host: self.root.join("work"),
            channels_host: self.root.join("channels"),
            public_files: [&self.tools.shim, &self.tools.compiler]
                .map(|value| PublicFile {
                    namespace: value.namespace.clone(),
                    sha256: value.sha256,
                })
                .into(),
            protected_paths: vec![],
            shim_arguments: ["__namespace-exec", "/work", "--", "/toolchain/bin/rustc"]
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn scope_pin_refusal_leaves_no_channels() {
    let fixture = Fixture::new();
    let Err(error) = prepare_bounded(
        fixture.plan(),
        scopes()[0].clone(),
        Instant::now() + Duration::from_secs(5),
    ) else {
        panic!("bad pin accepted")
    };
    assert_eq!(error.error, NamespaceError::Resources);
    assert!(error.partial_channels.is_none() && error.cleanup_complete);
    assert!(!fixture.root.join("channels").exists());
}
#[test]
fn duplicate_and_foreign_scopes_refuse_before_valid_workload_preflight() {
    for duplicate in [true, false] {
        let fixture = Fixture::new();
        let mut scope = scopes();
        if duplicate {
            scope[1].run_id = scope[0].run_id.clone();
        } else {
            scope[1].aggregate = "foreign.slice".into();
        }
        assert!(matches!(
            fixture.collect(&scope),
            Err(workload::Error::Layout)
        ));
        assert!(
            fixture
                .root
                .join("job")
                .read_dir()
                .unwrap()
                .next()
                .is_none()
        );
    }
    let neighbor = Fixture::new();
    let result = neighbor.collect(&scopes()).unwrap();
    assert!(matches!(result.outcome, Outcome::LauncherFailed));
    assert_eq!(result.steps.len(), 1);
    assert!(result.process_cleanup_complete && result.scratch_released);
    assert!(neighbor.root.join("job/public/inputs.hex").is_file());
}
#[test]
fn bounded_scratch_rejects_extra_declared_directories_before_scope_validation() {
    let fixture = Fixture::new();
    let mut plan = fixture.plan();
    plan.namespace_directories.push("/work/bin".into());
    let Err(error) = prepare_bounded(
        plan,
        scopes()[0].clone(),
        Instant::now() + Duration::from_secs(5),
    ) else {
        panic!("extra scratch directory accepted")
    };
    assert_eq!(error.error, NamespaceError::InvalidPlan);
    assert!(error.partial_channels.is_none() && error.cleanup_complete);
    assert!(!fixture.root.join("channels").exists());
}
