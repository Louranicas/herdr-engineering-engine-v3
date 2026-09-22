//! Frozen source/support inputs, imported without constructing review authority.
use crate::{
    Result, checked,
    manifest::{self, Manifest},
    prepare,
    probes::Observations,
    support,
};
use habitat_engine::app::{evidence::Evidence, workload::Tools};
use habitat_engine::check::{collector::Sink, graph::Graph};
use habitat_engine::contracts::receipt::{self as r};
use habitat_engine::worker::resources::Scope;
use hee3_fixed_task_runtime::runtime::{Recipe, Sources};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::Instant;

pub fn sources(m: &Manifest, deadline: Instant) -> Result<Sources> {
    Ok(Sources {
        baseline: manifest::snapshot(&m.roles.baseline, deadline)?,
        repaired: manifest::snapshot(&m.roles.repaired, deadline)?,
        protected: manifest::snapshot(&m.roles.protected, deadline)?,
        fixtures: manifest::snapshot(&m.roles.fixtures, deadline)?,
        oracle: manifest::snapshot(&m.roles.oracle, deadline)?,
        harness: manifest::snapshot(&m.roles.harness, deadline)?,
        collector: manifest::snapshot(&m.roles.collector, deadline)?,
        launcher: manifest::snapshot(&m.roles.launcher, deadline)?,
        reference_patch: manifest::read(&m.assets.patch, 1024 * 1024, deadline)?,
    })
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Imported {
    scope: String,
    expectation: r::Ref,
    recipe: r::Ref,
    review: r::Ref,
    provenance: r::Ref,
    objects: Vec<ImportedObject>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportedObject {
    reference: r::Ref,
    hex: String,
}
fn unhex(value: &str) -> Result<Vec<u8>> {
    if !value.len().is_multiple_of(2) || value.len() > 32 * 1024 * 1024 {
        return Err("hex payload bound".into());
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| checked(u8::from_str_radix(checked(std::str::from_utf8(v))?, 16)))
        .collect()
}
pub fn review(e: &mut Evidence<'_>, bytes: &[u8]) -> Result<prepare::Reviewed> {
    let input: Imported = checked(serde_json::from_slice(bytes))?;
    if input.scope.is_empty() || input.objects.len() != 22 {
        return Err("exact reviewed input closure cardinality".into());
    }
    for object in input.objects {
        let bytes = unhex(&object.hex)?;
        checked(e.publish(&object.reference, &bytes))?;
    }
    let graph = checked(Graph::resolve(e, &input.review))?;
    checked(graph.get(&input.provenance))?;
    checked(Graph::resolve(e, &input.expectation))?;
    Ok(prepare::Reviewed {
        expectation: checked(r::TypedRef::new(input.expectation))?,
        recipe: checked(r::Payload::new(input.recipe))?,
        review: checked(r::TypedRef::new(input.review))?,
        provenance: checked(r::Payload::new(input.provenance))?,
    })
}
pub struct Assets {
    pub bytes: BTreeMap<&'static str, Vec<u8>>,
}
impl Assets {
    pub fn load(m: &Manifest, deadline: Instant) -> Result<Self> {
        let a = &m.assets;
        let mut bytes = BTreeMap::new();
        for (id, pin) in [
            ("schema", &a.schema),
            ("readiness", &a.readiness),
            ("contracts", &a.contracts),
            ("cargo_lock", &a.cargo_lock),
            ("runtime_lock", &a.runtime_lock),
            ("build", &a.build),
            ("shim_build", &a.shim_build),
            ("finite_files", &a.finite_files),
            ("review", &a.review),
            ("authority", &a.authority),
            ("isolation", &a.isolation),
            ("cleanup", &a.cleanup),
        ] {
            bytes.insert(id, manifest::read(pin, 16 * 1024 * 1024, deadline)?);
        }
        Ok(Self { bytes })
    }
    fn get(&self, id: &str) -> &[u8] {
        &self.bytes[id]
    }
}
fn environment() -> Result<Vec<r::EnvironmentV1>> {
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
    .into_iter()
    .map(|(name, value)| {
        Ok(r::EnvironmentV1 {
            name: checked(r::Name::new(name))?,
            value: r::Maybe::present(checked(r::Text::new(value))?),
            secret_handle: r::Maybe::unavailable(checked(r::Text::new("not_a_secret"))?),
        })
    })
    .collect()
}
fn attempt(m: &Manifest, index: usize, argv: &[String]) -> Result<prepare::AttemptIds> {
    let ids = &m.identities.attempts[index];
    let aggregate = format!("hee3aggregate{}.slice", ids.run.replace('-', ""));
    let scopes = ids.stages.clone().map(|id| Scope {
        systemd_run: m.tools.systemd_run.path.clone(),
        systemd_run_sha256: m.tools.systemd_run.sha256.clone(),
        runtime_dir: m.runtime_dir.clone(),
        run_id: id,
        aggregate: aggregate.clone(),
    });
    Ok(prepare::AttemptIds {
        run: checked(r::Id::new(&ids.run))?,
        attempt: checked(r::Id::new(&ids.attempt))?,
        session: checked(r::Id::new(&ids.session))?,
        workspace: checked(r::Id::new(&ids.workspace))?,
        scopes,
        argv: checked(r::List::new(
            argv.iter()
                .map(|s| r::Text::new(s.clone()))
                .collect::<std::result::Result<_, _>>()
                .map_err(|e| format!("{e:?}"))?,
        ))?,
    })
}
pub struct Preparation<'a> {
    pub manifest: &'a Manifest,
    pub assets: &'a Assets,
    pub sources: &'a Sources,
    pub tools: &'a Tools,
    pub observations: &'a Observations,
    pub argv: &'a [String],
    pub deadline: Instant,
}
pub fn pair(evidence: &mut Evidence<'_>, preparation: &Preparation<'_>) -> Result<[Recipe; 2]> {
    let manifest = preparation.manifest;
    let assets = preparation.assets;
    let sources = preparation.sources;
    let observations = preparation.observations;
    let reviewed = review(evidence, assets.get("review"))?;
    let versions = versions(manifest, observations)?;
    let isolation = observed_isolation(assets, observations)?;
    let standards = [
        support::StandardInput {
            id: "RC04",
            revision: "4",
            bytes: assets.get("readiness"),
        },
        support::StandardInput {
            id: "contract-decisions",
            revision: "4",
            bytes: assets.get("contracts"),
        },
    ];
    let locks = [
        support::LockInput {
            id: "engine-Cargo",
            path: "Cargo.lock",
            bytes: assets.get("cargo_lock"),
        },
        support::LockInput {
            id: "frontend-Cargo",
            path: "frontend/Cargo.lock",
            bytes: assets.get("runtime_lock"),
        },
    ];
    let env = environment()?;
    let flags: Vec<_> = manifest.rust_flags.iter().map(String::as_str).collect();
    let input = prepare::Inputs {
        task: checked(r::Id::new(&manifest.identities.task))?,
        attempts: [
            attempt(manifest, 0, preparation.argv)?,
            attempt(manifest, 1, preparation.argv)?,
        ],
        sources: prepare::Sources {
            baseline: &sources.baseline,
            repaired: &sources.repaired,
            protected: &sources.protected,
            fixtures: &sources.fixtures,
            oracle: &sources.oracle,
            harness: &sources.harness,
            collector: &sources.collector,
            launcher: &sources.launcher,
            reference_patch: &sources.reference_patch,
        },
        tools: preparation.tools,
        reviewed,
        support: support::Inputs {
            schema: assets.get("schema"),
            standards: &standards,
            locks: &locks,
            build: support::BuildInput {
                target: &manifest.build_target,
                profile: &manifest.build_profile,
                features: &[],
                default_features: true,
                rust_flags: &flags,
                provenance: assets.get("build"),
            },
            host: host(observations),
            tools: &versions,
            controller: support::ControllerInput {
                namespace_source_path: "candidate-src/worker/namespace.rs",
                namespace_source_sha256: manifest::digest_array(&manifest.namespace_source_sha256)?,
                shim_source_path: "namespace_shim.rs",
                shim_source_sha256: manifest::digest_array(&manifest.shim_source_sha256)?,
                shim_binary_sha256: preparation.tools.shim.sha256,
                build_provenance: assets.get("shim_build"),
            },
            original_finite_manifest: assets.get("finite_files"),
            environment: &env,
            invocation_authority: assets.get("authority"),
            isolation_specification: &isolation,
            cleanup_specification: assets.get("cleanup"),
            owner_id: &manifest.owner_id,
        },
    };
    Ok(checked(prepare::prepare_pair(
        evidence,
        &input,
        preparation.deadline,
    ))?
    .map(|preparation| Recipe {
        prepared: preparation.prepared,
        recipe: preparation.recipe,
        host: preparation.host,
        review_provenance: preparation.review_provenance,
        session: preparation.session,
        workspace: preparation.workspace,
        scopes: preparation.scopes,
    }))
}

fn versions<'a>(
    manifest: &'a Manifest,
    observations: &'a Observations,
) -> Result<[support::ToolInput<'a>; 3]> {
    Ok([
        support::ToolInput {
            id: "rustc",
            path: &manifest.tools.compiler.input.path,
            sha256: manifest::digest_array(&manifest.tools.compiler.input.sha256)?,
            version: &observations.versions[0],
        },
        support::ToolInput {
            id: "bubblewrap",
            path: &manifest.tools.bwrap.path,
            sha256: manifest::digest_array(&manifest.tools.bwrap.sha256)?,
            version: &observations.versions[1],
        },
        support::ToolInput {
            id: "systemd-run",
            path: &manifest.tools.systemd_run.path,
            sha256: manifest::digest_array(&manifest.tools.systemd_run.sha256)?,
            version: &observations.versions[2],
        },
    ])
}

fn host(observations: &Observations) -> support::HostInput<'_> {
    support::HostInput {
        os_release: &observations.host.os_release,
        kernel_release: &observations.host.kernel_release,
        boot_id: &observations.host.boot_id,
        cpuinfo: &observations.host.cpuinfo,
        meminfo: &observations.host.meminfo,
        architecture: &observations.host.architecture,
        observed_unix_ms: observations.host.observed_unix_ms,
    }
}

fn observed_isolation(assets: &Assets, observations: &Observations) -> Result<Vec<u8>> {
    checked(serde_json::to_vec(&serde_json::json!({
        "kind":"fixed-u64-isolation-with-actual-probes/1",
        "declaration":{"sha256":manifest::digest(assets.get("isolation")),"hex":manifest::hex(assets.get("isolation"))},
        "actual_local_probe_provenance":{"sha256":manifest::digest(&observations.provenance),"hex":manifest::hex(&observations.provenance)}
    })))
}
