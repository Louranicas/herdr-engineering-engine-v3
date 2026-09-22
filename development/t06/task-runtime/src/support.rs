//! Publish caller-supplied observations and exact selected input identities.
use crate::prepare::{
    Error, ErrorKind, Sources, budget, count, fail, json_raw, list, name, none, page, raw, record,
    scalar, text,
};
use habitat_engine::app::{
    evidence::{self, Evidence},
    workload,
};
use habitat_engine::contracts::receipt::{self as r};
use habitat_engine::worker::{
    process::ProcessReport,
    workspace::{Content, Snapshot},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::Instant;

pub struct ToolInput<'a> {
    pub id: &'a str,
    pub path: &'a Path,
    pub sha256: [u8; 32],
    pub version: &'a ProcessReport,
}
pub struct LockInput<'a> {
    pub id: &'a str,
    pub path: &'a str,
    pub bytes: &'a [u8],
}
pub struct StandardInput<'a> {
    pub id: &'a str,
    pub revision: &'a str,
    pub bytes: &'a [u8],
}
#[derive(Serialize)]
pub struct HostInput<'a> {
    pub os_release: &'a str,
    pub kernel_release: &'a str,
    pub boot_id: &'a str,
    pub cpuinfo: &'a str,
    pub meminfo: &'a str,
    pub architecture: &'a str,
    pub observed_unix_ms: u64,
}
pub struct BuildInput<'a> {
    pub target: &'a str,
    pub profile: &'a str,
    pub features: &'a [&'a str],
    pub default_features: bool,
    pub rust_flags: &'a [&'a str],
    pub provenance: &'a [u8],
}
pub struct ControllerInput<'a> {
    pub namespace_source_path: &'a str,
    pub namespace_source_sha256: [u8; 32],
    pub shim_source_path: &'a str,
    pub shim_source_sha256: [u8; 32],
    pub shim_binary_sha256: [u8; 32],
    pub build_provenance: &'a [u8],
}
pub struct Inputs<'a> {
    pub schema: &'a [u8],
    pub standards: &'a [StandardInput<'a>],
    pub locks: &'a [LockInput<'a>],
    pub build: BuildInput<'a>,
    pub host: HostInput<'a>,
    pub tools: &'a [ToolInput<'a>],
    pub controller: ControllerInput<'a>,
    pub original_finite_manifest: &'a [u8],
    pub environment: &'a [r::EnvironmentV1],
    pub invocation_authority: &'a [u8],
    pub isolation_specification: &'a [u8],
    pub cleanup_specification: &'a [u8],
    pub owner_id: &'a str,
}
pub(crate) struct Published {
    pub schema: r::Payload,
    pub locks: r::TypedRef<r::LockPageV1>,
    pub toolchain: r::TypedRef<r::ToolPageV1>,
    pub build: r::TypedRef<r::BuildProfileV1>,
    pub standards: r::TypedRef<r::StandardPageV1>,
    pub environment: r::TypedRef<r::EnvironmentPageV1>,
    pub isolation: r::Payload,
    pub grants: r::TypedRef<r::GrantPageV1>,
    pub effects: r::TypedRef<r::EffectPageV1>,
    pub limits: r::TypedRef<r::LimitsV1>,
    pub cleanup: r::TypedRef<r::CleanupContractV1>,
    pub host: r::TypedRef<r::HostV1>,
}
pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(&Sha256::digest(bytes)))
}
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut output, byte| {
        use std::fmt::Write;
        write!(output, "{byte:02x}").expect("String write");
        output
    })
}
fn source_file<'a>(snapshot: &'a Snapshot, path: &str) -> Result<&'a [u8], Error> {
    snapshot
        .entries()
        .find_map(|entry| {
            if entry.path == path {
                if let Content::File { bytes, .. } = &entry.content {
                    Some(bytes.as_slice())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok_or_else(|| fail(ErrorKind::Missing))
}
pub(crate) fn publish(
    e: &mut Evidence<'_>,
    input: &Inputs<'_>,
    tools: &workload::Tools,
    sources: &Sources<'_>,
    deadline: Instant,
) -> Result<Published, Error> {
    budget(deadline)?;
    validate_required(input)?;
    validate_controller(input, tools, sources, deadline)?;
    let actual_tools = selected_manifest(tools, deadline)?;
    let versions = version_observations(input.tools)?;
    let authority = raw(e, input.invocation_authority, "application/json")?;
    // Raw payloads are opaque to Graph: embed exact support bytes here. A Ref
    // written inside raw JSON would not make its target reachable on import.
    let isolation = json_raw(
        e,
        &serde_json::json!({
            "profile":"T06-u64-fixed-runtime-THDEV",
            "selected_tool_inputs":actual_tools,
            "tool_version_observations":versions,
            "original_compiler_manifest":exact_bytes(input.original_finite_manifest),
            "original_manifest_is_not_current_target_byte_proof":true,
            "controller_build":exact_bytes(input.controller.build_provenance),
            "controller_namespace_source_sha256":format!("sha256:{}",hex(&input.controller.namespace_source_sha256)),
            "shim_source_sha256":format!("sha256:{}",hex(&input.controller.shim_source_sha256)),
            "shim_binary_sha256":format!("sha256:{}",hex(&input.controller.shim_binary_sha256)),
            "build_provenance":exact_bytes(input.build.provenance),
            "declared_isolation":exact_bytes(input.isolation_specification),
            "invocation_authority":exact_bytes(input.invocation_authority),
            "recipe":{"candidate_cutoff_ms":900_000,"verification_cutoff_ms":1_190_000,"total_wall_ms":1_200_000},
            "scope":"pre-execution declarations and retained input identities; no runtime observation or custody qualification"
        }),
    )?;
    let schema = raw(e, input.schema, "application/schema+json")?;
    let standards = standards(e, input.standards, &schema)?;
    let locks = locks(e, input.locks)?;
    let build = build(e, &input.build)?;
    let toolchain = toolchain(e, input.tools, tools, deadline)?;
    let host = host(e, &input.host)?;
    let environment = page!(e, EnvironmentPageV1, input.environment.to_vec());
    let grant_id = evidence::fresh_id(deadline).map_err(|_| fail(ErrorKind::Deadline))?;
    let grants = page!(
        e,
        GrantPageV1,
        vec![r::GrantV1 {
            grant_id: grant_id.clone(),
            scope_sha256: authority.as_ref().sha256.clone(),
            issuer_id: name(input.owner_id)?,
            grant: authority
        }]
    );
    let effects = page!(
        e,
        EffectPageV1,
        vec![r::EffectV1 {
            effect_id: name("fixed-u64-workload-output")?,
            grant_id,
            owner_id: name(input.owner_id)?,
            scope: text(
                "Only selected fixed compile-library/link-driver/execute-driver operations in private bounded scratch; no network or provider calls. Runtime and Store retain separate authority."
            )?,
            specification: isolation.clone()
        }]
    );
    let cleanup_spec = raw(e, input.cleanup_specification, "application/json")?;
    let obligation = r::ObligationV1 {
        obligation_id: evidence::fresh_id(deadline).map_err(|_| fail(ErrorKind::Deadline))?,
        owner_id: name(input.owner_id)?,
        scope: text(
            "Settle every owned leader, descendant, namespace writer, channel, retained scratch mount and aggregate before completion.",
        )?,
        material: true,
        state: r::ObligationV1State::Open,
        evidence: list(vec![cleanup_spec.as_ref().clone()])?,
        reason: text("Pre-execution obligation; no settlement observation has occurred.")?,
    };
    let obligations = page!(e, ObligationPageV1, vec![obligation]);
    let cleanup = record(
        e,
        &r::CleanupContractV1 {
            owner_id: name(input.owner_id)?,
            term_grace_ms: count(5_000)?,
            deadline_ms: count(1_200_000)?,
            require_empty_descendants: true,
            obligations,
            readback_specification: cleanup_spec,
        },
    )?;
    let limits = record(e, &limits()?)?;
    budget(deadline)?;
    Ok(Published {
        schema,
        locks,
        toolchain,
        build,
        standards,
        environment,
        isolation,
        grants,
        effects,
        limits,
        cleanup,
        host,
    })
}
fn validate_required(input: &Inputs<'_>) -> Result<(), Error> {
    for bytes in [
        input.schema,
        input.original_finite_manifest,
        input.controller.build_provenance,
        input.build.provenance,
        input.invocation_authority,
        input.isolation_specification,
        input.cleanup_specification,
    ] {
        if bytes.is_empty() {
            return Err(fail(ErrorKind::Missing));
        }
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(fail(ErrorKind::Bound));
        }
    }
    if input.locks.is_empty()
        || input.standards.is_empty()
        || input.tools.len() < 2
        || input.environment.is_empty()
        || input.owner_id.is_empty()
    {
        return Err(fail(ErrorKind::Missing));
    }
    for bytes in [
        input.schema,
        input.original_finite_manifest,
        input.controller.build_provenance,
        input.build.provenance,
        input.invocation_authority,
        input.isolation_specification,
        input.cleanup_specification,
    ] {
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| fail(ErrorKind::Encoding))?;
        if !value.is_object() {
            return Err(fail(ErrorKind::Encoding));
        }
    }
    validate_environment(input.environment)?;
    unique(input.tools.iter().map(|row| row.id))?;
    unique(input.standards.iter().map(|row| row.id))?;
    unique(input.locks.iter().map(|row| row.id))?;
    Ok(())
}
fn unique<'a>(values: impl Iterator<Item = &'a str>) -> Result<(), Error> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.is_empty() || !seen.insert(value) {
            return Err(fail(ErrorKind::Binding));
        }
    }
    Ok(())
}
fn validate_controller(
    input: &Inputs<'_>,
    tools: &workload::Tools,
    sources: &Sources<'_>,
    deadline: Instant,
) -> Result<(), Error> {
    sources
        .collector
        .readback_source(deadline)
        .map_err(|_| fail(ErrorKind::Binding))?;
    sources
        .launcher
        .readback_source(deadline)
        .map_err(|_| fail(ErrorKind::Binding))?;
    let namespace = source_file(sources.collector, input.controller.namespace_source_path)?;
    let shim = source_file(sources.launcher, input.controller.shim_source_path)?;
    if <[u8; 32]>::from(Sha256::digest(namespace)) != input.controller.namespace_source_sha256
        || <[u8; 32]>::from(Sha256::digest(shim)) != input.controller.shim_source_sha256
        || tools.shim.sha256 != input.controller.shim_binary_sha256
    {
        return Err(fail(ErrorKind::Binding));
    }
    // Exact source and build evidence are retained; source text is not proof of a
    // descriptor grant. The actual namespace owner validates FD7 at runtime.
    Ok(())
}
#[derive(Serialize)]
struct FileFact {
    host: String,
    namespace: String,
    sha256: String,
    byte_length: u64,
    device: u64,
    inode: u64,
    mode: u32,
}
#[derive(Serialize)]
struct ToolManifest {
    profile: &'static str,
    entries: Vec<FileFact>,
    projected_namespace_bytes: u64,
    namespace_directories: Vec<String>,
    bwrap: FileFact,
}
fn selected_manifest(tools: &workload::Tools, deadline: Instant) -> Result<ToolManifest, Error> {
    if tools.runtime_files.len() > 510 {
        return Err(fail(ErrorKind::Bound));
    }
    let mut entries = Vec::new();
    let mut total = 0_u64;
    let mut namespace = BTreeSet::new();
    for input in tools
        .runtime_files
        .iter()
        .chain([&tools.compiler, &tools.shim])
    {
        if !input.namespace.is_absolute() || !namespace.insert(input.namespace.clone()) {
            return Err(fail(ErrorKind::Binding));
        }
        let (fact, _) = read_file(
            &input.host,
            &input.namespace,
            Some(input.sha256),
            false,
            deadline,
        )?;
        total = total
            .checked_add(fact.byte_length)
            .ok_or_else(|| fail(ErrorKind::Bound))?;
        if total > 1024 * 1024 * 1024 {
            return Err(fail(ErrorKind::Bound));
        }
        entries.push(fact);
    }
    entries.sort_by(|left, right| left.namespace.as_bytes().cmp(right.namespace.as_bytes()));
    let (bwrap, _) = read_file(&tools.bwrap, &tools.bwrap, None, false, deadline)?;
    Ok(ToolManifest {
        profile: "actual-selected-tool-file-identities/1",
        entries,
        projected_namespace_bytes: total,
        namespace_directories: tools
            .namespace_directories
            .iter()
            .map(|p| {
                p.to_str()
                    .map(str::to_owned)
                    .ok_or_else(|| fail(ErrorKind::Encoding))
            })
            .collect::<Result<_, _>>()?,
        bwrap,
    })
}
fn same_metadata(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    (
        left.dev(),
        left.ino(),
        left.len(),
        left.mode(),
        left.mtime(),
        left.mtime_nsec(),
        left.ctime(),
        left.ctime_nsec(),
    ) == (
        right.dev(),
        right.ino(),
        right.len(),
        right.mode(),
        right.mtime(),
        right.mtime_nsec(),
        right.ctime(),
        right.ctime_nsec(),
    )
}
fn read_file(
    path: &Path,
    namespace: &Path,
    wanted: Option<[u8; 32]>,
    retain: bool,
    deadline: Instant,
) -> Result<(FileFact, Vec<u8>), Error> {
    budget(deadline)?;
    if !path.is_absolute() || fs::canonicalize(path).map_err(|_| fail(ErrorKind::Io))? != path {
        return Err(fail(ErrorKind::Binding));
    }
    let before = fs::symlink_metadata(path).map_err(|_| fail(ErrorKind::Io))?;
    if !before.is_file()
        || before.len() > 1024 * 1024 * 1024
        || (retain && before.len() > 16 * 1024 * 1024)
    {
        return Err(fail(ErrorKind::Bound));
    }
    let mut file = File::open(path).map_err(|_| fail(ErrorKind::Io))?;
    if !same_metadata(&before, &file.metadata().map_err(|_| fail(ErrorKind::Io))?) {
        return Err(fail(ErrorKind::Binding));
    }
    let mut digest = Sha256::new();
    let mut kept = Vec::new();
    let mut count = 0_u64;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        budget(deadline)?;
        let read = file.read(&mut buffer).map_err(|_| fail(ErrorKind::Io))?;
        if read == 0 {
            break;
        }
        count += u64::try_from(read).map_err(|_| fail(ErrorKind::Bound))?;
        if count > before.len() {
            return Err(fail(ErrorKind::Binding));
        }
        digest.update(&buffer[..read]);
        if retain {
            kept.extend_from_slice(&buffer[..read]);
        }
    }
    let actual: [u8; 32] = digest.finalize().into();
    if count != before.len()
        || wanted.is_some_and(|value| value != actual)
        || !same_metadata(&before, &file.metadata().map_err(|_| fail(ErrorKind::Io))?)
        || !same_metadata(
            &before,
            &fs::symlink_metadata(path).map_err(|_| fail(ErrorKind::Io))?,
        )
    {
        return Err(fail(ErrorKind::Binding));
    }
    budget(deadline)?;
    Ok((
        FileFact {
            host: path
                .to_str()
                .ok_or_else(|| fail(ErrorKind::Encoding))?
                .into(),
            namespace: namespace
                .to_str()
                .ok_or_else(|| fail(ErrorKind::Encoding))?
                .into(),
            sha256: format!("sha256:{}", hex(&actual)),
            byte_length: count,
            device: before.dev(),
            inode: before.ino(),
            mode: before.mode(),
        },
        kept,
    ))
}
fn version(report: &ProcessReport) -> Result<&str, Error> {
    if report.exit_code != Some(0)
        || report.signal.is_some()
        || report.interruption.is_some()
        || !report.leader_reaped
        || !report.process_group_settled
        || report.pending.is_some()
        || !report.stdout.eof
        || !report.stderr.eof
        || report.stdout.truncated
        || report.stderr.truncated
        || report.stdout.failed
        || report.stderr.failed
        || !report.stderr.bytes.is_empty()
        || report.stdout.observed_bytes
            != u64::try_from(report.stdout.bytes.len()).map_err(|_| fail(ErrorKind::Bound))?
        || report.stderr.observed_bytes != 0
    {
        return Err(fail(ErrorKind::Binding));
    }
    std::str::from_utf8(&report.stdout.bytes)
        .map_err(|_| fail(ErrorKind::Encoding))?
        .lines()
        .next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| fail(ErrorKind::Missing))
}
fn toolchain(
    e: &mut Evidence<'_>,
    input: &[ToolInput<'_>],
    tools: &workload::Tools,
    deadline: Instant,
) -> Result<r::TypedRef<r::ToolPageV1>, Error> {
    for (id, path, pin) in [
        (
            "rustc",
            tools.compiler.host.as_path(),
            Some(tools.compiler.sha256),
        ),
        ("bubblewrap", tools.bwrap.as_path(), None),
    ] {
        let item = input
            .iter()
            .find(|row| row.id == id)
            .ok_or_else(|| fail(ErrorKind::Missing))?;
        if item.path != path || pin.is_some_and(|pin| pin != item.sha256) {
            return Err(fail(ErrorKind::Binding));
        }
    }
    let mut rows = Vec::new();
    for input in input {
        let version = version(input.version)?;
        let (_, bytes) = read_file(input.path, input.path, Some(input.sha256), true, deadline)?;
        rows.push(r::ToolV1 {
            tool_id: name(input.id)?,
            executable: raw(e, &bytes, "application/octet-stream")?,
            executable_path: text(
                input
                    .path
                    .to_str()
                    .ok_or_else(|| fail(ErrorKind::Encoding))?,
            )?,
            version: text(version)?,
            version_output: raw(e, &input.version.stdout.bytes, "application/octet-stream")?,
            target: name("x86_64-unknown-linux-gnu")?,
        });
    }
    Ok(page!(e, ToolPageV1, rows))
}
fn host(e: &mut Evidence<'_>, input: &HostInput<'_>) -> Result<r::TypedRef<r::HostV1>, Error> {
    let (release, cpus, memory) = host_fields(input)?;
    let facts = json_raw(e, input)?;
    record(
        e,
        &r::HostV1 {
            os: name("linux")?,
            release: name(release)?,
            architecture: name(input.architecture)?,
            kernel: text(input.kernel_release.trim())?,
            boot_id: name(input.boot_id.trim())?,
            logical_cpus: cpus,
            memory_bytes: count(memory)?,
            facts,
        },
    )
}
fn host_fields<'a>(input: &'a HostInput<'_>) -> Result<(&'a str, u32, u64), Error> {
    if input.observed_unix_ms == 0
        || input.kernel_release.trim().is_empty()
        || input.boot_id.trim().is_empty()
        || input.architecture != "x86_64"
    {
        return Err(fail(ErrorKind::Missing));
    }
    let release = input
        .os_release
        .lines()
        .find_map(|line| line.strip_prefix("VERSION_ID="))
        .map(|value| value.trim_matches('"'))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail(ErrorKind::Missing))?;
    let cpus = u32::try_from(
        input
            .cpuinfo
            .lines()
            .filter(|line| line.starts_with("processor\t:"))
            .count(),
    )
    .map_err(|_| fail(ErrorKind::Bound))?;
    let line = input
        .meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .ok_or_else(|| fail(ErrorKind::Missing))?;
    let fields: Vec<_> = line.split_whitespace().collect();
    if fields.len() != 2 || fields[1] != "kB" || cpus == 0 {
        return Err(fail(ErrorKind::Binding));
    }
    let memory = fields[0]
        .parse::<u64>()
        .map_err(|_| fail(ErrorKind::Encoding))?
        .checked_mul(1024)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail(ErrorKind::Bound))?;
    Ok((release, cpus, memory))
}
fn standards(
    e: &mut Evidence<'_>,
    inputs: &[StandardInput<'_>],
    schema: &r::Payload,
) -> Result<r::TypedRef<r::StandardPageV1>, Error> {
    if !inputs
        .iter()
        .any(|input| input.id == "RC04" && input.revision == "4")
    {
        return Err(fail(ErrorKind::Missing));
    }
    let mut rows = Vec::new();
    for input in inputs {
        rows.push(r::StandardV1 {
            standard_id: name(input.id)?,
            revision: name(input.revision)?,
            document: raw(e, input.bytes, "text/markdown")?,
        });
    }
    rows.push(r::StandardV1 {
        standard_id: name("hee3-receipt-schema")?,
        revision: name("1")?,
        document: schema.clone(),
    });
    Ok(page!(e, StandardPageV1, rows))
}
fn build(
    e: &mut Evidence<'_>,
    input: &BuildInput<'_>,
) -> Result<r::TypedRef<r::BuildProfileV1>, Error> {
    if input.rust_flags.is_empty()
        || input.target != "x86_64-unknown-linux-gnu"
        || input.profile.is_empty()
    {
        return Err(fail(ErrorKind::Missing));
    }
    let flags = page!(
        e,
        LanguageFlagsPageV1,
        vec![r::LanguageFlagsV1 {
            language: name("rust")?,
            argv: list(
                input
                    .rust_flags
                    .iter()
                    .map(|value| text(*value))
                    .collect::<Result<_, _>>()?
            )?
        }]
    );
    record(
        e,
        &r::BuildProfileV1 {
            target: name(input.target)?,
            features: list(
                input
                    .features
                    .iter()
                    .map(|value| name(value))
                    .collect::<Result<_, _>>()?,
            )?,
            default_features: input.default_features,
            build_profile: name(input.profile)?,
            language_flags: flags,
        },
    )
}
fn locks(
    e: &mut Evidence<'_>,
    inputs: &[LockInput<'_>],
) -> Result<r::TypedRef<r::LockPageV1>, Error> {
    let mut rows = Vec::new();
    for input in inputs {
        let lock = raw(e, input.bytes, "application/toml")?;
        let packages = lock_packages(input.bytes)?;
        let mut dependencies = Vec::new();
        for (id, package) in packages {
            dependencies.push(r::DependencyV1 {
                dependency_id: name(&id)?,
                name: name(&package.name)?,
                version: name(&package.version)?,
                source: text(
                    package
                        .source
                        .unwrap_or_else(|| "local_package_in_exact_retained_lock".into()),
                )?,
                checksum: match package.checksum {
                    Some(value) => {
                        r::Maybe::present(scalar(r::Sha::new(format!("sha256:{value}")))?)
                    }
                    None => none("local_lock_entry_without_registry_checksum")?,
                },
                locked_by: lock.clone(),
            });
        }
        let dependencies = page!(e, DependencyPageV1, dependencies);
        rows.push(r::LockV1 {
            lock_id: name(input.id)?,
            ecosystem: name("cargo")?,
            path: scalar(r::RelPath::new(input.path))?,
            content: lock,
            dependencies,
        });
    }
    Ok(page!(e, LockPageV1, rows))
}
#[derive(serde::Deserialize)]
struct LockedPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
}
#[derive(serde::Deserialize)]
struct Lock {
    version: u32,
    package: Vec<LockedPackage>,
}
fn lock_packages(bytes: &[u8]) -> Result<BTreeMap<String, LockedPackage>, Error> {
    let lock: Lock =
        toml::from_str(std::str::from_utf8(bytes).map_err(|_| fail(ErrorKind::Encoding))?)
            .map_err(|_| fail(ErrorKind::Encoding))?;
    if lock.version != 4 || lock.package.is_empty() || lock.package.len() > 256 {
        return Err(fail(ErrorKind::Bound));
    }
    let mut rows = BTreeMap::new();
    for package in lock.package {
        if package.name.is_empty()
            || package.version.is_empty()
            || package.source.is_some() != package.checksum.is_some()
        {
            return Err(fail(ErrorKind::Binding));
        }
        let id = format!("{}-{}", package.name, package.version);
        if rows.insert(id, package).is_some() {
            return Err(fail(ErrorKind::Binding));
        }
    }
    Ok(rows)
}
fn limits() -> Result<r::LimitsV1, Error> {
    Ok(r::LimitsV1 {
        wall_ms: count(1_200_000)?,
        memory_bytes: count(8_589_934_592)?,
        memory_swap_bytes: count(0)?,
        scratch_bytes: count(4_294_967_296)?,
        stdout_bytes: count(8_388_608)?,
        stderr_bytes: count(8_388_608)?,
        artifact_bytes: count(67_108_864)?,
        external_requests: count(0)?,
        external_cost_microunits: count(0)?,
        term_grace_ms: count(5000)?,
        cleanup_deadline_ms: count(1_200_000)?,
        cpu_quota_percent: 200,
        tasks_max: 128,
        compiler_jobs: 2,
        julia_threads: 1,
        blas_threads: 1,
        currency: none("zero_external_cost")?,
    })
}

fn version_observations(inputs: &[ToolInput<'_>]) -> Result<Vec<serde_json::Value>, Error> {
    inputs.iter().map(|input|{version(input.version)?;let report=input.version;Ok(serde_json::json!({"tool_id":input.id,"executable_path":input.path,"executable_sha256":format!("sha256:{}",hex(&input.sha256)),"stdout_hex":hex(&report.stdout.bytes),"stderr_hex":hex(&report.stderr.bytes),"stdout_observed_bytes":report.stdout.observed_bytes,"stderr_observed_bytes":report.stderr.observed_bytes,"stdout_eof":report.stdout.eof,"stderr_eof":report.stderr.eof,"stdout_truncated":report.stdout.truncated,"stderr_truncated":report.stderr.truncated,"stdout_failed":report.stdout.failed,"stderr_failed":report.stderr.failed,"exit_code":report.exit_code,"signal":report.signal,"interruption":format!("{:?}",report.interruption),"leader_reaped":report.leader_reaped,"group_settled":report.process_group_settled,"pending":report.pending.is_some(),"elapsed_ns":report.elapsed.as_nanos().to_string(),"origin":"caller-supplied actual ProcessReport; observation authenticity not inferred from this JSON"}))}).collect()
}
fn validate_environment(rows: &[r::EnvironmentV1]) -> Result<(), Error> {
    const ENV: [(&str, &str); 14] = [
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
    ];
    if rows.len() != ENV.len() {
        return Err(fail(ErrorKind::Binding));
    }
    unique(rows.iter().map(|row| row.name.as_str()))?;
    for (name, value) in ENV {
        let row = rows
            .iter()
            .find(|row| row.name.as_str() == name)
            .ok_or_else(|| fail(ErrorKind::Binding))?;
        if row.value.value.as_ref().map(r::Text::as_str) != Some(value)
            || row.secret_handle.value.is_some()
        {
            return Err(fail(ErrorKind::Binding));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/support_controls.rs"]
mod tests;

fn exact_bytes(bytes: &[u8]) -> serde_json::Value {
    serde_json::json!({"sha256":hash(bytes),"byte_length":bytes.len(),"hex":hex(bytes)})
}
