//! Author-owned preparation controls; synthetic process facts are not observations.
use super::*;
use habitat_engine::worker::process::{SignalFacts, Stream};
use habitat_engine::{check::graph::Graph, store::ArtifactStaging};
use std::os::unix::fs::{DirBuilderExt, symlink};
use std::path::PathBuf;
use std::time::Duration;
struct Area(PathBuf);
impl Area {
    fn new() -> Self {
        let id = evidence::fresh_id(Instant::now() + Duration::from_secs(5)).unwrap();
        let path = std::env::temp_dir().join(format!("hee3-u64-prep-{}", id.as_str()));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn observation() -> ProcessReport {
    ProcessReport {
        started_at: Instant::now(),
        leader_pid: 17,
        exit_code: Some(0),
        signal: None,
        interruption: None,
        interruption_observed_at: None,
        stdout: Stream {
            bytes: b"rustc fixture version\n".to_vec(),
            observed_bytes: 22,
            eof: true,
            truncated: false,
            failed: false,
        },
        stderr: Stream {
            eof: true,
            ..Stream::default()
        },
        elapsed: Duration::from_millis(1),
        signals: SignalFacts::default(),
        leader_reaped: true,
        process_group_settled: true,
        observer_ready: true,
        pending: None,
    }
}
fn host_input() -> HostInput<'static> {
    HostInput {
        os_release: "ID=fedora\nVERSION_ID=44\n",
        kernel_release: "test-kernel\n",
        boot_id: "10000000-0000-4000-8000-000000000001\n",
        cpuinfo: "processor\t: 0\nprocessor\t: 1\n",
        meminfo: "MemTotal: 8192 kB\n",
        architecture: "x86_64",
        observed_unix_ms: 1,
    }
}
#[test]
fn actual_file_pin_and_retained_bytes_match_independent_literal() {
    let area = Area::new();
    let path = area.0.join("input");
    fs::write(&path, b"abc").unwrap();
    let pin: [u8; 32] = Sha256::digest(b"abc").into();
    let (fact, bytes) = read_file(&path, Path::new("/input"), Some(pin), true, deadline()).unwrap();
    assert_eq!(bytes, b"abc");
    assert_eq!(fact.byte_length, 3);
    assert_eq!(
        fact.sha256,
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
#[test]
fn wrong_file_digest_is_refused() {
    let area = Area::new();
    let path = area.0.join("input");
    fs::write(&path, b"abc").unwrap();
    assert!(matches!(
        read_file(&path, Path::new("/input"), Some([0; 32]), true, deadline())
            .err()
            .unwrap()
            .kind,
        ErrorKind::Binding
    ));
}
#[test]
fn symbolic_tool_path_cannot_substitute_for_canonical_input() {
    let area = Area::new();
    let path = area.0.join("input");
    let alias = area.0.join("alias");
    fs::write(&path, b"abc").unwrap();
    symlink(&path, &alias).unwrap();
    assert!(matches!(
        read_file(&alias, Path::new("/input"), None, false, deadline())
            .err()
            .unwrap()
            .kind,
        ErrorKind::Binding
    ));
}
#[test]
fn expired_read_does_not_open_or_create_missing_path() {
    let area = Area::new();
    let path = area.0.join("absent");
    assert!(matches!(
        read_file(&path, Path::new("/input"), None, false, Instant::now())
            .err()
            .unwrap()
            .kind,
        ErrorKind::Deadline
    ));
    assert!(!path.exists());
}
#[test]
fn raw_executable_bound_refuses_adjacent_oversize_file() {
    let area = Area::new();
    let path = area.0.join("input");
    let file = File::create(&path).unwrap();
    file.set_len(16 * 1024 * 1024 + 1).unwrap();
    assert!(matches!(
        read_file(&path, Path::new("/input"), None, true, deadline())
            .err()
            .unwrap()
            .kind,
        ErrorKind::Bound
    ));
}
#[test]
fn host_memory_has_explicit_units_and_checked_bytes() {
    let mut input = host_input();
    assert_eq!(host_fields(&input).unwrap(), ("44", 2, 8_388_608));
    input.meminfo = "MemTotal: 8192 MB\n";
    assert!(matches!(
        host_fields(&input).err().unwrap().kind,
        ErrorKind::Binding
    ));
    input.meminfo = "MemTotal: 18446744073709551615 kB\n";
    assert!(matches!(
        host_fields(&input).err().unwrap().kind,
        ErrorKind::Bound
    ));
}
#[test]
fn missing_cpu_or_memory_observation_cannot_become_zero_host_capacity() {
    let mut input = host_input();
    input.cpuinfo = "";
    assert!(host_fields(&input).is_err());
    input.cpuinfo = "processor\t: 0\n";
    input.meminfo = "";
    assert!(matches!(
        host_fields(&input).err().unwrap().kind,
        ErrorKind::Missing
    ));
}
#[test]
fn version_success_requires_measured_complete_stdout_and_empty_stderr() {
    let mut input = observation();
    input.stdout.observed_bytes = u64::try_from(input.stdout.bytes.len()).unwrap();
    assert_eq!(version(&input).unwrap(), "rustc fixture version");
    input.stdout.eof = false;
    assert!(version(&input).is_err());
    input.stdout.eof = true;
    input.stderr.bytes = b"warning\n".to_vec();
    input.stderr.observed_bytes = 8;
    assert!(version(&input).is_err());
}
#[test]
fn numeric_exit_zero_does_not_override_signal_or_unsettled_process() {
    let mut input = observation();
    input.stdout.observed_bytes = u64::try_from(input.stdout.bytes.len()).unwrap();
    input.signal = Some(9);
    assert!(version(&input).is_err());
    input.signal = None;
    input.process_group_settled = false;
    assert!(version(&input).is_err());
    input.process_group_settled = true;
    assert!(version(&input).is_ok());
}
#[test]
fn lock_rows_derive_from_exact_bytes_and_duplicate_identity_is_refused() {
    let valid=b"version=4\n[[package]]\nname='local'\nversion='0.1.0'\n[[package]]\nname='dep'\nversion='1.0.0'\nsource='registry+https://example.invalid'\nchecksum='0000000000000000000000000000000000000000000000000000000000000000'\n";
    let rows = lock_packages(valid).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows["local-0.1.0"].name, "local");
    assert!(rows["local-0.1.0"].checksum.is_none());
    let duplicate=b"version=4\n[[package]]\nname='local'\nversion='0.1.0'\n[[package]]\nname='local'\nversion='0.1.0'\n";
    assert!(matches!(
        lock_packages(duplicate).err().unwrap().kind,
        ErrorKind::Binding
    ));
}
#[test]
fn registry_lock_without_checksum_and_wrong_lock_version_are_refused() {
    assert!(lock_packages(b"version=4\n[[package]]\nname='dep'\nversion='1'\nsource='registry+https://example.invalid'\n").is_err());
    assert!(lock_packages(b"version=3\n[[package]]\nname='local'\nversion='1'\n").is_err());
}
#[test]
fn fixed_limits_preserve_common_origin_and_zero_external_spend() {
    let limits = limits().unwrap();
    assert_eq!(limits.wall_ms.get(), 1_200_000);
    assert_eq!(limits.cleanup_deadline_ms.get(), 1_200_000);
    assert_eq!(limits.term_grace_ms.get(), 5_000);
    assert_eq!(limits.external_requests.get(), 0);
    assert_eq!(limits.external_cost_microunits.get(), 0);
    assert_eq!(limits.artifact_bytes.get(), 67_108_864);
    assert_eq!(limits.scratch_bytes.get(), 4_294_967_296);
}

#[test]
fn support_provenance_is_self_contained_in_resolved_graph() {
    let source_area = Area::new();
    let staging_area = Area::new();
    let deadline = deadline();
    let (snapshot, tools) = source_fixture(&source_area, deadline);
    let version = observation();
    let tool_rows = [
        ToolInput {
            id: "rustc",
            path: &tools.compiler.host,
            sha256: tools.compiler.sha256,
            version: &version,
        },
        ToolInput {
            id: "bubblewrap",
            path: &tools.bwrap,
            sha256: Sha256::digest(b"fixture launcher").into(),
            version: &version,
        },
    ];
    let environment = environment_fixture();
    let original = b"{ \"entries\": [] }\n";
    let control = b"{\"controller\":\"fixture\"}\n";
    let build = b"{\"build\":\"fixture\"}\n";
    let isolation = b"{\"isolation\":\"fixture\"}\n";
    let authority = b"{\"grant\":\"fixture\"}\n";
    let locks = [LockInput {
        id: "fixture",
        path: "Cargo.lock",
        bytes: b"version=4\n[[package]]\nname='fixture'\nversion='1'\n",
    }];
    let standards = [StandardInput {
        id: "RC04",
        revision: "4",
        bytes: b"synthetic test document; no normative authority",
    }];
    let input = Inputs {
        schema: b"{}",
        standards: &standards,
        locks: &locks,
        build: BuildInput {
            target: "x86_64-unknown-linux-gnu",
            profile: "fixture",
            features: &[],
            default_features: false,
            rust_flags: &["-Dwarnings"],
            provenance: build,
        },
        host: host_input(),
        tools: &tool_rows,
        controller: ControllerInput {
            namespace_source_path: "namespace.rs",
            namespace_source_sha256: Sha256::digest(b"controller source").into(),
            shim_source_path: "shim.rs",
            shim_source_sha256: Sha256::digest(b"shim source").into(),
            shim_binary_sha256: tools.shim.sha256,
            build_provenance: control,
        },
        original_finite_manifest: original,
        environment: &environment,
        invocation_authority: authority,
        isolation_specification: isolation,
        cleanup_specification: b"{\"cleanup\":\"fixture\"}\n",
        owner_id: "fixture-owner",
    };
    let sources = sources_fixture(&snapshot);
    let staging = ArtifactStaging::open(&staging_area.0, true, deadline).unwrap();
    let mut evidence = Evidence::staged(&staging, deadline);
    let published = publish(&mut evidence, &input, &tools, &sources, deadline).unwrap();
    let graph = Graph::resolve(&evidence, published.isolation.as_ref()).unwrap();
    let value: serde_json::Value =
        serde_json::from_slice(graph.get(published.isolation.as_ref()).unwrap().bytes()).unwrap();
    assert_embedded(
        &value,
        &[
            ("original_compiler_manifest", original.as_slice()),
            ("controller_build", control),
            ("build_provenance", build),
            ("declared_isolation", isolation),
            ("invocation_authority", authority),
        ],
    );
    assert_eq!(
        value["selected_tool_inputs"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        value["tool_version_observations"][0]["stdout_hex"],
        "727573746320666978747572652076657273696f6e0a"
    );
    assert_eq!(
        graph.object_count(),
        1,
        "all support provenance bytes are in the reachable raw object"
    );
    drop(evidence);
    drop(staging);
}

fn source_fixture(source_area: &Area, deadline: Instant) -> (Snapshot, workload::Tools) {
    use habitat_engine::worker::namespace::ReadOnlyFile;
    for (path, bytes) in [
        ("namespace.rs", b"controller source".as_slice()),
        ("shim.rs", b"shim source"),
        ("rustc", b"fixture compiler"),
        ("bwrap", b"fixture launcher"),
        ("shim", b"fixture shim"),
    ] {
        fs::write(source_area.0.join(path), bytes).unwrap();
    }
    let snapshot = Snapshot::capture(&source_area.0, &[], deadline).unwrap();
    let binding = |file: &str, namespace: &str| ReadOnlyFile {
        host: source_area.0.join(file),
        namespace: namespace.into(),
        sha256: Sha256::digest(fs::read(source_area.0.join(file)).unwrap()).into(),
    };
    let tools = workload::Tools {
        bwrap: source_area.0.join("bwrap"),
        compiler: binding("rustc", "/toolchain/bin/rustc"),
        shim: binding("shim", "/shim/namespace-shim"),
        runtime_files: Vec::new(),
        namespace_directories: vec!["/toolchain/bin".into(), "/shim".into()],
    };
    (snapshot, tools)
}

fn environment_fixture() -> [r::EnvironmentV1; 14] {
    [
        ("PATH", "/toolchain/bin"),
        ("HOME", "/work/home"),
        ("TMPDIR", "/tmp"),
        ("LANG", "C.UTF-8"),
        ("LC_ALL", "C.UTF-8"),
        ("TZ", "UTC"),
        ("CARGO_HOME", "/toolchain/cargo-home"),
        ("CARGO_TARGET_DIR", "/work/target"),
        ("CARGO_BUILD_JOBS", "2"),
        (
            "JULIA_DEPOT_PATH",
            "/work/julia-depot:/toolchain/julia-depot",
        ),
        ("JULIA_NUM_THREADS", "1"),
        ("OPENBLAS_NUM_THREADS", "1"),
        ("OMP_NUM_THREADS", "1"),
        ("RUST_BACKTRACE", "0"),
    ]
    .map(|(key, value)| r::EnvironmentV1 {
        name: name(key).unwrap(),
        value: r::Maybe::present(text(value).unwrap()),
        secret_handle: none("not_secret").unwrap(),
    })
}

fn assert_embedded(value: &serde_json::Value, samples: &[(&str, &[u8])]) {
    for &(key, expected) in samples {
        let encoded = value[key]["hex"]
            .as_str()
            .expect("exact support bytes must survive through resolved isolation object");
        let (pairs, remainder) = encoded.as_bytes().as_chunks::<2>();
        assert!(remainder.is_empty());
        let actual: Vec<u8> = pairs
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(
            value[key]["byte_length"],
            u64::try_from(expected.len()).unwrap()
        );
    }
}

fn sources_fixture(snapshot: &Snapshot) -> Sources<'_> {
    Sources {
        baseline: snapshot,
        repaired: snapshot,
        protected: snapshot,
        fixtures: snapshot,
        oracle: snapshot,
        harness: snapshot,
        collector: snapshot,
        launcher: snapshot,
        reference_patch: b"not-used-by-support",
    }
}
