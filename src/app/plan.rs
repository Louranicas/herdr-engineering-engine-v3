//! The verification plan the runtime composes for itself (B14a-2c-ii-b, design R17 round 2).
//!
//! Two publications make a `consistency::Prepared`: the SHARED part, published once per dispatch
//! by [`shared`] before any attempt begins — the schema, the toolchain and build pages, the empty
//! locks page and the one standards row, the environment page, the isolation profile, the reviewed
//! closures, and the six subjects that do not change per attempt — and the per-check part (result
//! subject, patch, limits, cleanup contract, identity), which 2c-iii composes at `verify()`.
//! Splitting them is what keeps the store's object inventory from filling with a copy of the
//! shared MBs per check (R17 round 2, F4/F10). Every byte published here is read under the pin
//! that names it BEFORE it is copied, executed or published (F5): nothing here is a declaration.
//!
//! Subjects the runtime materialises (fixtures, oracle, harness, launcher) are written as one-file
//! or two-file trees under a plan root of the caller's choosing — a SIBLING of every job root,
//! never inside one (the workload refuses a non-empty job root, F1) — captured with the one
//! `Snapshot::capture` door, published through the one `subjects::publish` door, and torn down.
//! The collector subject is [`crate::app::subjects::unread`]: the engine does not read its own
//! executable (R16r2.3), and the wire's form for an unread subject says so (Q1).

use super::class_profile::{Profile, ReviewedError, Which, read_declared, read_reviewed_closure};
use super::evidence::{self, Evidence};
use super::live_verifier::BWRAP;
use super::subjects;
use super::u64_receipt::{
    OBLIGATIONS, RUNTIME_OWNER, cleanup_contract, environment_rows, limits, obligation_rows,
};
use super::workload::{COMPILE_FLAGS, DRIVER_DESTINATION, FIXED_DESTINATIONS, Tools};
use crate::check::collector::{self, Publisher, Sink as _};
use crate::check::consistency::{
    self, Editable, Prepared, U64_BOUNDS, U64_EDITABLE, U64Attempt, derive_patch, patch_ceiling,
    prepare_u64,
};
use crate::check::patch;
use crate::check::u64_oracle::{self, FrozenOracle};
use crate::contracts::receipt::{
    BuildProfileV1, EffectPageV1, EffectV1, EnvironmentPageV1, Generation, GrantPageV1, GrantV1,
    Id, LanguageFlagsPageV1, LanguageFlagsV1, List, LockPageV1, Maybe, Name, ObligationPageV1,
    Payload, Ref, RelPath, Sha, StandardPageV1, StandardV1, SubjectFileV1Origin, SubjectV1, Text,
    ToolPageV1, ToolV1, TypedRef,
};
use crate::store::Object;
use crate::worker::namespace::{self, NamespaceError, pinned_bytes};
use crate::worker::namespace_shim::ENVIRONMENT;
use crate::worker::process::{
    self, Interruption, ProcessSpec, SETTLE_PAUSE, SettleStep, settle_step,
};
use crate::worker::workspace::{self, Snapshot};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

/// The receipt schema every plan binds (`schema_sha256`), published once as a payload and named
/// by the one standards row (R17 round 2, decision 5 and Q5).
pub const SCHEMA: &[u8] = include_bytes!("../../schemas/receipts/receipt-v1.schema.json");
/// The standards row's id and revision for the schema, as the 003 lane recorded them.
pub const SCHEMA_STANDARD: (&str, &str) = ("hee3-receipt-schema", "1");
/// The most bytes a pinned tool may be: the compiler driver, the shim and bwrap are each well
/// under it; a file over it is not the pin, whatever its digest would have been.
pub const MAX_TOOL_BYTES: usize = 64 * 1024 * 1024;
/// The most stdout a `<compiler> -Vv` may print before it is refused as not a version report.
pub const MAX_VERSION_BYTES: usize = 4096;
/// The build profile's fixed facts: the target the compiler pin is for and the profile name.
pub const BUILD_TARGET: &str = "x86_64-unknown-linux-gnu";
pub const BUILD_PROFILE: &str = "frozen";
/// The isolation profile's kind, a rendering of the namespace's own constants.
pub const ISOLATION_KIND: &str = "hee3.isolation-profile/1";

/// The one subject the runtime does not read, and why (R16r2.3; Q1).
pub const COLLECTOR_UNREAD: &str = "R16r2.3: the engine does not read its own executable";

/// What the shared publication needs: the pins the verifier will launch with (the same `Tools`
/// value — F5), the class profile (its typed reviewed references and directory), the baseline
/// and protected snapshots the runtime already holds, a plan root that must not exist yet, the
/// dispatch's own deadline passed through (no limit is created here), and the cancellation flag.
#[derive(Clone, Copy)]
pub struct Inputs<'a> {
    pub tools: &'a Tools,
    pub profile: &'a Profile,
    pub baseline: &'a Snapshot,
    pub protected: &'a Snapshot,
    pub plan_root: &'a Path,
    pub deadline: Instant,
    pub cancelled: &'a AtomicBool,
}

/// The shared part of every plan this dispatch will compose: typed references into the sink that
/// published them, the objects behind them for the carry-over into each check's sink (R17 round 2,
/// decision 7), and the registry entries this publication added (decision 9, graph side).
#[derive(Clone, Debug)]
pub struct Shared {
    pub schema: Payload,
    pub toolchain: TypedRef<ToolPageV1>,
    pub build: TypedRef<BuildProfileV1>,
    pub locks: TypedRef<LockPageV1>,
    pub standards: TypedRef<StandardPageV1>,
    pub environment: TypedRef<EnvironmentPageV1>,
    pub isolation: Payload,
    pub grants: TypedRef<GrantPageV1>,
    pub effects: TypedRef<EffectPageV1>,
    pub seed: TypedRef<SubjectV1>,
    pub fixtures: TypedRef<SubjectV1>,
    pub oracle: TypedRef<SubjectV1>,
    pub harness: TypedRef<SubjectV1>,
    pub launcher: TypedRef<SubjectV1>,
    pub collector: TypedRef<SubjectV1>,
    /// The compiler's `-Vv` `release:` value, as observed on this host.
    pub compiler_version: String,
    /// The sink's whole registry after this publication, in artifact-id order — the carry-over
    /// into each check's sink (decision 7). The runtime's prepare sink is fresh, so this is the
    /// shared set; over a sink that already held objects it holds those too.
    pub objects: Vec<(Ref, Object)>,
    /// How many registry entries this publication added — receipt-graph nodes with fresh ids:
    /// 54 on a fresh sink, 32 on a sink already holding the closure (the proof's constants). The
    /// store's inventory counts DISTINCT DIGESTS, which the CAS dedupes across dispatches (the
    /// schema, the compiler, the closure), so the store-side `S_new` of N4 is at most this and is
    /// read from the ledger by the Tier-3 host run (R17 proof (b)); the per-check plan adds 8.
    pub added: usize,
}

/// Why the shared plan could not be published — each a `Refusal` stop before dispatch (R17 round
/// 2, N2), named for the input that refused.
#[derive(Debug)]
pub enum Refusal {
    Deadline,
    Cancelled,
    /// The plan root already exists: a leftover this runtime will not reuse.
    PlanRootExists,
    /// The plan root's parent is not a canonical path: `Snapshot::capture` would refuse it later,
    /// after pages were published.
    PlanRootNotCanonical,
    /// One artifact id reached the closure walk under two different references (the profile's
    /// expectation and the review's citation of it disagree): nothing is published under it.
    ClosureConflict {
        artifact_id: String,
    },
    /// The plan root or a subject tree could not be created or removed.
    PlanTeardown,
    /// A pinned tool's bytes are not the pin's, or could not be read under the bound.
    Pin {
        which: PinWhich,
        error: NamespaceError,
    },
    /// `oracle.json` or the public wrapper is not in the protected tree, or the oracle refused.
    Protected(&'static str),
    Oracle(u64_oracle::OracleError),
    /// A reviewed closure did not resolve from the class directory.
    Closure(ReviewedError),
    /// A declared grant or effect file (`authority`, `specification`) was not read as declared.
    Declared {
        which: &'static str,
        error: ReviewedError,
    },
    /// The publisher refused at the named stage.
    Publish {
        stage: &'static str,
        error: collector::Error,
    },
    /// A subject could not be captured or published, by role.
    Subject {
        role: &'static str,
        error: subjects::ErrorKind,
    },
    Capture {
        role: &'static str,
        error: workspace::Error,
    },
    /// `<compiler> -Vv` did not run to a settled exit with a `release:` line within the bound;
    /// `why` names the site.
    CompilerVersion {
        why: VersionWhy,
    },
    /// A value had no rendering the receipt admits; the site names which.
    Encoding(&'static str),
    /// The seed-to-result patch could not be derived (past its bound, or not a change to the
    /// editable alone).
    Patch(patch::Error),
    /// A snapshot holds no file at the class's editable path (the file named).
    Editable(&'static str),
    /// `prepare_u64` refused the plan (ids not distinct, an empty argv).
    Plan(consistency::Error),
}

/// Which pinned tool a `Pin` refusal is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinWhich {
    Compiler,
    Shim,
    Bwrap,
}

/// Where the compiler probe (`<compiler> -Vv`) refused: the launch, a child left behind, the
/// stream (interrupted, truncated or not at EOF), the exit code, the bytes not UTF-8, no
/// `release:` line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionWhy {
    Run,
    Child,
    Stream,
    Exit,
    Utf8,
    Release,
}

impl Refusal {
    /// The stop reason a pre-dispatch plan refusal is recorded under (R17 round 2, N2): the kind,
    /// never the detail — the detail is this value's `Debug`.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Cancelled => "plan_cancelled",
            Self::PlanRootExists => "plan_root_exists",
            Self::PlanRootNotCanonical => "plan_root_not_canonical",
            Self::PlanTeardown => "plan_teardown",
            Self::ClosureConflict { .. } => "plan_closure_conflict",
            Self::Deadline
            | Self::Pin {
                error: NamespaceError::Deadline,
                ..
            } => "plan_deadline",
            Self::Pin {
                which: PinWhich::Compiler,
                ..
            } => "plan_pin_compiler",
            Self::Pin {
                which: PinWhich::Shim,
                ..
            } => "plan_pin_shim",
            Self::Pin {
                which: PinWhich::Bwrap,
                ..
            } => "plan_pin_bwrap",
            Self::Protected(_) => "plan_protected",
            Self::Oracle(_) => "plan_oracle",
            Self::Closure(_) => "plan_closure",
            Self::Declared { .. } => "plan_declared",
            Self::Publish { .. } => "plan_publish",
            Self::Subject { .. } => "plan_subject",
            Self::Capture { .. } => "plan_capture",
            Self::CompilerVersion {
                why: VersionWhy::Run,
            } => "plan_compiler_version_run",
            Self::CompilerVersion {
                why: VersionWhy::Child,
            } => "plan_compiler_version_child",
            Self::CompilerVersion {
                why: VersionWhy::Stream,
            } => "plan_compiler_version_stream",
            Self::CompilerVersion {
                why: VersionWhy::Exit,
            } => "plan_compiler_version_exit",
            Self::CompilerVersion {
                why: VersionWhy::Utf8,
            } => "plan_compiler_version_utf8",
            Self::CompilerVersion {
                why: VersionWhy::Release,
            } => "plan_compiler_version_release",
            Self::Editable(_) => "plan_editable",
            Self::Encoding(_) => "plan_encoding",
            Self::Patch(_) => "plan_patch",
            Self::Plan(_) => "plan_prepare",
        }
    }
}

/// Name the stage a publisher refusal came from.
fn at(stage: &'static str) -> impl Fn(collector::Error) -> Refusal {
    move |error| Refusal::Publish { stage, error }
}

fn name(value: &str) -> Result<Name, Refusal> {
    Name::new(value).map_err(|_| Refusal::Encoding("name"))
}

fn text(value: &str) -> Result<Text, Refusal> {
    Text::new(value).map_err(|_| Refusal::Encoding("text"))
}

fn count(value: usize) -> Result<u32, Refusal> {
    u32::try_from(value).map_err(|_| Refusal::Encoding("count"))
}

/// Publish `bytes` as a raw payload under `media`.
fn payload(sink: &mut Evidence<'_>, bytes: &[u8], media: &str) -> Result<Payload, Refusal> {
    sink.payload(bytes, media)
        .map_err(|error| Refusal::Publish {
            stage: "payload",
            error: collector::Error::Sink(error),
        })
}

/// One page of at most 256 rows for the page types the shared plan emits (the plan has one row
/// or none per page; a longer inventory is refused rather than chained here).
macro_rules! page {
    ($publisher:expr, $page:ident, $rows:expr, $stage:literal) => {{
        let rows = $rows;
        if rows.len() > 256 {
            return Err(Refusal::Encoding("page rows"));
        }
        let record = $page {
            page_index: 0,
            page_count: 1,
            row_count: count(rows.len())?,
            total_rows: count(rows.len())?,
            rows: List::new(rows).map_err(|_| Refusal::Encoding("page list"))?,
            next: Maybe::unavailable(text("end_of_inventory")?),
        };
        $publisher.record(&record).map_err(at($stage))?
    }};
}

/// Publish the shared part of the plan into `sink` (R17 round 2, shape S). Every refusal is
/// named; on any refusal the plan root is removed and what was published stays in the CAS
/// uncited (L6 — the caller's stop record lists the objects).
///
/// # Errors
/// Each [`Refusal`], named for the input that refused.
pub fn shared(sink: &mut Evidence<'_>, inputs: &Inputs<'_>) -> Result<Shared, Refusal> {
    if inputs.cancelled.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(Refusal::Cancelled);
    }
    if Instant::now() >= inputs.deadline {
        return Err(Refusal::Deadline);
    }
    if inputs.plan_root.symlink_metadata().is_ok() {
        return Err(Refusal::PlanRootExists);
    }
    let parent = inputs
        .plan_root
        .parent()
        .ok_or(Refusal::PlanRootNotCanonical)?;
    if parent.canonicalize().ok().as_deref() != Some(parent) {
        return Err(Refusal::PlanRootNotCanonical);
    }
    fs::DirBuilder::new()
        .mode(0o700)
        .create(inputs.plan_root)
        .map_err(|_| Refusal::PlanTeardown)?;
    let before = sink.registered().len();
    let published = publish_shared(sink, inputs);
    // The root is removed whether or not the publication refused; a refusal is returned as itself
    // (a root that could not be removed after a refusal is the refusal's, not a second finding).
    let removed = fs::remove_dir_all(inputs.plan_root).is_ok();
    let mut shared = published?;
    if !removed {
        return Err(Refusal::PlanTeardown);
    }
    shared.objects = sink.registered().values().cloned().collect();
    shared.added = sink.registered().len().saturating_sub(before);
    Ok(shared)
}

/// The three pinned tools' bytes, each refused unless it hashes to its pin (F5).
struct Pinned {
    compiler: Vec<u8>,
    shim: Vec<u8>,
    bwrap: Vec<u8>,
}

fn pinned(inputs: &Inputs<'_>) -> Result<Pinned, Refusal> {
    let read = |which: PinWhich, path: &Path, pin: &[u8; 32]| {
        pinned_bytes(path, pin, MAX_TOOL_BYTES, inputs.deadline)
            .map_err(|error| Refusal::Pin { which, error })
    };
    Ok(Pinned {
        compiler: read(
            PinWhich::Compiler,
            &inputs.tools.compiler.host,
            &inputs.tools.compiler.sha256,
        )?,
        shim: read(
            PinWhich::Shim,
            &inputs.tools.shim.host,
            &inputs.tools.shim.sha256,
        )?,
        bwrap: read(
            PinWhich::Bwrap,
            &inputs.tools.bwrap,
            &namespace::BWRAP_SHA256,
        )?,
    })
}

/// The typed pages and records every attempt of the dispatch cites.
struct Pages {
    schema: Payload,
    toolchain: TypedRef<ToolPageV1>,
    build: TypedRef<BuildProfileV1>,
    locks: TypedRef<LockPageV1>,
    standards: TypedRef<StandardPageV1>,
    environment: TypedRef<EnvironmentPageV1>,
    isolation: Payload,
    grants: TypedRef<GrantPageV1>,
    effects: TypedRef<EffectPageV1>,
}

fn pages(
    sink: &mut Evidence<'_>,
    inputs: &Inputs<'_>,
    compiler: &[u8],
    version: &str,
    version_output: &[u8],
    declared: &(Vec<u8>, Vec<u8>),
) -> Result<Pages, Refusal> {
    let schema = payload(sink, SCHEMA, "application/schema+json")?;
    let compiler_payload = payload(sink, compiler, "application/octet-stream")?;
    let version_payload = payload(sink, version_output, "application/octet-stream")?;
    let isolation = payload(sink, &isolation_profile()?, "application/json")?;
    let environment_rows = environment_rows().map_err(|_| Refusal::Encoding("environment rows"))?;
    let authority = payload(sink, &declared.0, "application/json")?;
    let specification = payload(sink, &declared.1, "application/json")?;
    let mut publisher = Publisher::new(sink);
    let (grants, effects) = authority_pages(&mut publisher, inputs, authority, specification)?;
    let toolchain = page!(
        publisher,
        ToolPageV1,
        vec![ToolV1 {
            tool_id: name("rustc")?,
            executable: compiler_payload,
            executable_path: text(
                inputs
                    .tools
                    .compiler
                    .host
                    .to_str()
                    .ok_or(Refusal::Encoding("compiler path"))?
            )?,
            version: text(version)?,
            version_output: version_payload,
            target: name(BUILD_TARGET)?,
        }],
        "toolchain"
    );
    let flags = page!(
        publisher,
        LanguageFlagsPageV1,
        vec![LanguageFlagsV1 {
            language: name("rust")?,
            argv: List::new(
                COMPILE_FLAGS
                    .iter()
                    .map(|flag| text(flag))
                    .collect::<Result<Vec<_>, _>>()?
            )
            .map_err(|_| Refusal::Encoding("language flags"))?,
        }],
        "language flags"
    );
    let build = publisher
        .record(&BuildProfileV1 {
            target: name(BUILD_TARGET)?,
            features: List::new(Vec::new()).map_err(|_| Refusal::Encoding("features"))?,
            default_features: false,
            build_profile: name(BUILD_PROFILE)?,
            language_flags: flags,
        })
        .map_err(at("build profile"))?;
    let locks = page!(publisher, LockPageV1, Vec::new(), "locks");
    let standards = page!(
        publisher,
        StandardPageV1,
        vec![StandardV1 {
            standard_id: name(SCHEMA_STANDARD.0)?,
            revision: name(SCHEMA_STANDARD.1)?,
            document: schema.clone(),
        }],
        "standards"
    );
    let environment = page!(
        publisher,
        EnvironmentPageV1,
        environment_rows,
        "environment"
    );
    Ok(Pages {
        schema,
        toolchain,
        build,
        locks,
        standards,
        environment,
        isolation,
        grants,
        effects,
    })
}

/// The declared grant and effect files, read as declared (digest-bound, F11) — beside the pins,
/// before anything is executed or published (review 2c-ii-c, LOW).
fn declared_bytes(inputs: &Inputs<'_>) -> Result<(Vec<u8>, Vec<u8>), Refusal> {
    let grant = &inputs.profile.declared.grant;
    let effect = &inputs.profile.declared.effect;
    let authority =
        read_declared(inputs.profile, &grant.authority).map_err(|error| Refusal::Declared {
            which: "authority",
            error,
        })?;
    let specification = read_declared(inputs.profile, &effect.specification).map_err(|error| {
        Refusal::Declared {
            which: "specification",
            error,
        }
    })?;
    Ok((authority, specification))
}

/// The grant and effect pages: one row each from the profile's declarations, the grant's
/// `scope_sha256` the authority document's digest, the effect under that grant, owned by the
/// runtime (R17 round 2, decision 6).
fn authority_pages(
    publisher: &mut Publisher<'_, Evidence<'_>>,
    inputs: &Inputs<'_>,
    authority: Payload,
    specification: Payload,
) -> Result<(TypedRef<GrantPageV1>, TypedRef<EffectPageV1>), Refusal> {
    let grant = &inputs.profile.declared.grant;
    let effect = &inputs.profile.declared.effect;
    let grants = page!(
        publisher,
        GrantPageV1,
        vec![GrantV1 {
            grant_id: grant.grant_id.clone(),
            scope_sha256: Sha::new(authority.as_ref().sha256.as_str().to_owned())
                .map_err(|_| Refusal::Encoding("scope digest"))?,
            issuer_id: grant.issuer_id.clone(),
            grant: authority,
        }],
        "grants"
    );
    let effects = page!(
        publisher,
        EffectPageV1,
        vec![EffectV1 {
            effect_id: effect.effect_id.clone(),
            grant_id: grant.grant_id.clone(),
            owner_id: name(RUNTIME_OWNER)?,
            scope: effect.scope.clone(),
            specification,
        }],
        "effects"
    );
    Ok((grants, effects))
}

/// The six subjects that do not change per attempt, by role.
struct Subjects {
    seed: TypedRef<SubjectV1>,
    fixtures: TypedRef<SubjectV1>,
    oracle: TypedRef<SubjectV1>,
    harness: TypedRef<SubjectV1>,
    launcher: TypedRef<SubjectV1>,
    collector: TypedRef<SubjectV1>,
}

fn subjects(
    sink: &mut Evidence<'_>,
    inputs: &Inputs<'_>,
    pinned: &Pinned,
    public_inputs: &[u8],
) -> Result<Subjects, Refusal> {
    let oracle_bytes = protected_file(inputs.protected, "oracle.json")?;
    let wrapper = protected_file(inputs.protected, "public-wrapper.rs")?;
    let seed = subjects::publish(
        sink,
        inputs.baseline,
        SubjectFileV1Origin::Authored,
        inputs.deadline,
    )
    .map_err(|error| Refusal::Subject {
        role: "seed",
        error: error.kind,
    })?;
    let fixtures = materialised(sink, inputs, "fixtures", &[("inputs.hex", public_inputs)])?;
    let oracle = materialised(sink, inputs, "oracle", &[("oracle.json", oracle_bytes)])?;
    let harness = materialised(sink, inputs, "harness", &[("public-wrapper.rs", wrapper)])?;
    let launcher = materialised(
        sink,
        inputs,
        "launcher",
        &[
            ("namespace-shim", pinned.shim.as_slice()),
            ("bwrap", pinned.bwrap.as_slice()),
        ],
    )?;
    let collector = subjects::unread(sink, "collector", COLLECTOR_UNREAD, inputs.deadline)
        .map_err(|error| Refusal::Subject {
            role: "collector",
            error: error.kind,
        })?;
    Ok(Subjects {
        seed,
        fixtures,
        oracle,
        harness,
        launcher,
        collector,
    })
}

fn publish_shared(sink: &mut Evidence<'_>, inputs: &Inputs<'_>) -> Result<Shared, Refusal> {
    // The pins first, before anything is copied, executed or published (F5).
    let pinned = pinned(inputs)?;
    let declared = declared_bytes(inputs)?;
    let oracle = FrozenOracle::from_bytes(protected_file(inputs.protected, "oracle.json")?)
        .map_err(Refusal::Oracle)?;
    let public_inputs = oracle.public_inputs();
    let (version, version_output) = compiler_version(inputs)?;
    let pages = pages(
        sink,
        inputs,
        &pinned.compiler,
        &version,
        &version_output,
        &declared,
    )?;
    let subjects = subjects(sink, inputs, &pinned, &public_inputs)?;
    // The reviewed closures, every node published under its own reference (F3).
    for which in [Which::Expectation, Which::Review] {
        let root = match which {
            Which::Expectation => inputs.profile.declared.reviewed.expectation().as_ref(),
            Which::Review => inputs.profile.declared.reviewed.review().as_ref(),
        };
        let graph = read_reviewed_closure(inputs.profile, which, root).map_err(Refusal::Closure)?;
        for node in graph.nodes() {
            // The expectation's closure lies inside the review's: a node already registered under
            // the same reference is the same object; the same id under another reference is a
            // conflict the sink refuses.
            if let Some((registered, _)) =
                sink.registered().get(node.reference().artifact_id.as_str())
            {
                if registered == node.reference() {
                    continue;
                }
                return Err(Refusal::ClosureConflict {
                    artifact_id: node.reference().artifact_id.as_str().to_owned(),
                });
            }
            sink.publish(node.reference(), node.bytes())
                .map_err(|error| Refusal::Publish {
                    stage: "reviewed closure",
                    error: collector::Error::Sink(error),
                })?;
        }
    }
    Ok(Shared {
        schema: pages.schema,
        toolchain: pages.toolchain,
        build: pages.build,
        locks: pages.locks,
        standards: pages.standards,
        environment: pages.environment,
        isolation: pages.isolation,
        grants: pages.grants,
        effects: pages.effects,
        seed: subjects.seed,
        fixtures: subjects.fixtures,
        oracle: subjects.oracle,
        harness: subjects.harness,
        launcher: subjects.launcher,
        collector: subjects.collector,
        compiler_version: version,
        objects: Vec::new(),
        added: 0,
    })
}

/// A file of the protected tree by name, or the refusal naming it.
fn protected_file<'a>(protected: &'a Snapshot, file: &'static str) -> Result<&'a [u8], Refusal> {
    protected
        .entries()
        .find_map(|entry| match &entry.content {
            workspace::Content::File { bytes, .. } if entry.path == file => Some(bytes.as_slice()),
            _ => None,
        })
        .ok_or(Refusal::Protected(file))
}

/// Write `files` as a 0700 tree under `<plan root>/<role>/`, capture it, publish it as an
/// authored subject, and remove the tree — the one door for what a subject is (Q3).
fn materialised(
    sink: &mut Evidence<'_>,
    inputs: &Inputs<'_>,
    role: &'static str,
    files: &[(&str, &[u8])],
) -> Result<TypedRef<SubjectV1>, Refusal> {
    let root = inputs.plan_root.join(role);
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .map_err(|_| Refusal::PlanTeardown)?;
    for (file, bytes) in files {
        let path = root.join(file);
        fs::write(&path, bytes).map_err(|_| Refusal::PlanTeardown)?;
        fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600))
            .map_err(|_| Refusal::PlanTeardown)?;
    }
    let snapshot = Snapshot::capture(&root, &[], inputs.deadline)
        .map_err(|error| Refusal::Capture { role, error })?;
    let published = subjects::publish(
        sink,
        &snapshot,
        SubjectFileV1Origin::Authored,
        inputs.deadline,
    )
    .map_err(|error| Refusal::Subject {
        role,
        error: error.kind,
    });
    let removed = fs::remove_dir_all(&root).is_ok();
    let published = published?;
    if !removed {
        return Err(Refusal::PlanTeardown);
    }
    Ok(published)
}

/// Observe the compiler's version once: `<compiler> -Vv` under the dispatch's deadline, stdout
/// bounded, the child settled, and the `release:` line required (R17 round 2, Q4, N6). Returns
/// the release value and the whole stdout as the version output.
fn compiler_version(inputs: &Inputs<'_>) -> Result<(String, Vec<u8>), Refusal> {
    let spec = ProcessSpec {
        executable: inputs.tools.compiler.host.clone(),
        arguments: vec![OsString::from("-Vv")],
        directory: inputs.plan_root.to_path_buf(),
        environment: ENVIRONMENT
            .iter()
            .map(|(key, value)| (OsString::from(key), OsString::from(value)))
            .collect(),
        input: Vec::new(),
        stream_limit: MAX_VERSION_BYTES,
    };
    let refuse = |why: VersionWhy| Refusal::CompilerVersion { why };
    let mut report = process::run(&spec, inputs.deadline, inputs.cancelled)
        .map_err(|_| refuse(VersionWhy::Run))?;
    // A child or group still live after the report is settled by polling until the deadline (N6):
    // a probe that leaves a process behind is not an observation this dispatch may build on.
    if let Some(pending) = report.pending.as_mut() {
        loop {
            let poll = pending.poll_cleanup(inputs.deadline);
            match settle_step(&poll, Instant::now(), inputs.deadline) {
                SettleStep::Settled => break,
                SettleStep::Refused => return Err(refuse(VersionWhy::Child)),
                // `poll_cleanup` returns at once; a pause between polls keeps this from burning a
                // core for the whole deadline (pacing under the passed-through deadline, not a
                // limit).
                SettleStep::Wait => std::thread::sleep(SETTLE_PAUSE),
            }
        }
    }
    // The stream's own interruptions first, then the bytes, then the exit: an over-limit report is
    // refused as a stream, a cancelled or timed-out one as what it was.
    match report.interruption {
        Some(Interruption::Cancelled) => return Err(Refusal::Cancelled),
        Some(Interruption::Timeout) => return Err(Refusal::Deadline),
        Some(Interruption::OutputLimit) => return Err(refuse(VersionWhy::Stream)),
        Some(_) => return Err(refuse(VersionWhy::Child)),
        None => {}
    }
    if report.stdout.truncated || !report.stdout.eof {
        return Err(refuse(VersionWhy::Stream));
    }
    if report.exit_code != Some(0) {
        return Err(refuse(VersionWhy::Exit));
    }
    // The bytes verified before the probe are shown to be the bytes still at the path after it
    // (F5 named: the window between the read and the exec is not closed, it is measured).
    if namespace::sha256(&inputs.tools.compiler.host, inputs.deadline).map_err(|error| {
        Refusal::Pin {
            which: PinWhich::Compiler,
            error,
        }
    })? != inputs.tools.compiler.sha256
    {
        return Err(Refusal::Pin {
            which: PinWhich::Compiler,
            error: NamespaceError::Digest,
        });
    }
    let output = std::str::from_utf8(&report.stdout.bytes).map_err(|_| refuse(VersionWhy::Utf8))?;
    let release = output
        .lines()
        .find_map(|line| line.strip_prefix("release: "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(refuse(VersionWhy::Release))?;
    Ok((release.to_owned(), report.stdout.bytes.clone()))
}

/// The isolation profile: a JSON rendering of the namespace's own constants — `--unshare-all`,
/// the fixed destinations and the environment table — by one function, so the receipt and the
/// namespace read one source (R17 round 2, decision 3). A derived document, recorded as such.
///
/// # Errors
/// `Encoding` when the rendering fails — never an empty document in its place.
pub fn isolation_profile() -> Result<Vec<u8>, Refusal> {
    let destinations: Vec<&str> = FIXED_DESTINATIONS.to_vec();
    let environment: Vec<(&str, &str)> = ENVIRONMENT.to_vec();
    let value = serde_json::json!({
        "kind": ISOLATION_KIND,
        "unshare_all": true,
        "fixed_destinations": destinations,
        "environment": environment.iter().map(|(k, v)| serde_json::json!({"name": k, "value": v})).collect::<Vec<_>>(),
        "launcher": BWRAP,
    });
    serde_json::to_vec(&value).map_err(|_| Refusal::Encoding("isolation profile"))
}

/// The readback specification the cleanup contract cites: the four obligations the runtime owns
/// and the rule it settles them by, rendered from the constants that enforce it (R16 round 2, 9).
///
/// # Errors
/// `Encoding` when the rendering fails.
pub fn readback_specification() -> Result<Vec<u8>, Refusal> {
    let value = serde_json::json!({
        "kind": "hee3.readback-specification/1",
        "owner": RUNTIME_OWNER,
        "obligations": OBLIGATIONS.to_vec(),
        "rule": "after the check's cutoff: every owned process and FIFO terminated, the scratch \
                 descriptors released, every retained path removed within the teardown share, and \
                 the resource aggregate read back; each obligation settled separately, never inferred",
    });
    serde_json::to_vec(&value).map_err(|_| Refusal::Encoding("readback specification"))
}

/// What the per-check plan needs (R17 round 2, shape C): the shared part, the profile, the seed
/// and applied snapshots, the identities the runtime minted before the plan (the run id IS the
/// verification's evidence artifact id — decision 3), the previous attempt's run when it recorded
/// a receipt, the check's wall in ms (also its cleanup deadline — L4), and the deadline.
pub struct CheckInputs<'a> {
    pub shared: &'a Shared,
    pub profile: &'a Profile,
    pub baseline: &'a Snapshot,
    pub applied: &'a Snapshot,
    pub task: &'a str,
    pub attempt: &'a str,
    pub run: &'a str,
    pub generation: &'a str,
    pub parent_run: Option<&'a str>,
    pub obligation_ids: &'a [String; 4],
    pub wall_ms: u64,
    pub deadline: Instant,
}

/// The per-check plan: the frozen `Prepared` the collector copies into the receipt, and the
/// objects this publication registered (the carry-over into compose's sink — decision 7).
#[derive(Clone, Debug)]
pub struct Planned {
    pub prepared: Prepared,
    pub objects: Vec<(Ref, Object)>,
    pub added: usize,
}

/// Compose the per-check plan into `sink` (a fresh sink over the ledger, in the check's own hold):
/// the result subject, the seed-to-result patch by the one derivation `patch_binding` re-derives
/// (N5), the limits and the cleanup contract over the four pre-execution obligations, and the
/// identity — then `prepare_u64`, the one door for the plan.
///
/// # Errors
/// Each [`Refusal`], named.
pub fn check(sink: &mut Evidence<'_>, inputs: &CheckInputs<'_>) -> Result<Planned, Refusal> {
    if Instant::now() >= inputs.deadline {
        return Err(Refusal::Deadline);
    }
    let before = sink.registered().len();
    let result = subjects::publish(
        sink,
        inputs.applied,
        SubjectFileV1Origin::Authored,
        inputs.deadline,
    )
    .map_err(|error| Refusal::Subject {
        role: "result",
        error: error.kind,
    })?;
    let editable = Editable {
        path: RelPath::new(U64_EDITABLE.to_owned())
            .map_err(|_| Refusal::Encoding("editable path"))?,
        bounds: U64_BOUNDS,
    };
    let patch_bytes = derive_patch(
        snapshot_file(inputs.baseline, U64_EDITABLE)?,
        snapshot_file(inputs.applied, U64_EDITABLE)?,
        &editable,
        patch_ceiling(&editable),
    )
    .map_err(Refusal::Patch)?;
    let patch_payload = payload(sink, &patch_bytes, "text/x-diff")?;
    let specification = payload(sink, &readback_specification()?, "application/json")?;
    let mut publisher = Publisher::new(sink);
    let limits_record = publisher
        .record(&limits(inputs.wall_ms, inputs.wall_ms).map_err(|_| Refusal::Encoding("limits"))?)
        .map_err(at("limits"))?;
    let rows = obligation_rows(inputs.obligation_ids, specification.as_ref())
        .map_err(|_| Refusal::Encoding("obligation rows"))?;
    let obligations = page!(publisher, ObligationPageV1, rows, "obligations");
    let contract = publisher
        .record(
            &cleanup_contract(inputs.wall_ms, obligations, specification)
                .map_err(|_| Refusal::Encoding("cleanup contract"))?,
        )
        .map_err(at("cleanup contract"))?;
    let shared = inputs.shared;
    let reviewed = &inputs.profile.declared.reviewed;
    let attempt = U64Attempt {
        schema_sha256: Sha::new(schema_sha256()).map_err(|_| Refusal::Encoding("schema digest"))?,
        run_id: Id::new(inputs.run).map_err(|_| Refusal::Encoding("run id"))?,
        task_id: Id::new(inputs.task).map_err(|_| Refusal::Encoding("task id"))?,
        attempt_id: Id::new(inputs.attempt).map_err(|_| Refusal::Encoding("attempt id"))?,
        generation: Generation::new(inputs.generation)
            .map_err(|_| Refusal::Encoding("generation"))?,
        profile_id: name(&format!(
            "{}@{}",
            super::class_profile::CLASS,
            inputs.profile.digest
        ))?,
        parent_run: match inputs.parent_run {
            Some(run) => Maybe::present(Id::new(run).map_err(|_| Refusal::Encoding("parent run"))?),
            None => Maybe::unavailable(text(if inputs.generation == "1" {
                "first_attempt"
            } else {
                "separate_attempt_same_task"
            })?),
        },
        subjects: crate::contracts::receipt::SubjectsV1 {
            seed_subject: shared.seed.clone(),
            result_subject: Maybe::present(result),
            seed_to_result_patch: Maybe::present(patch_payload),
            fixtures: shared.fixtures.clone(),
            oracle: shared.oracle.clone(),
            harness: shared.harness.clone(),
            collector: shared.collector.clone(),
            launcher: shared.launcher.clone(),
            locks: shared.locks.clone(),
            toolchain: shared.toolchain.clone(),
            target_features_build_profile: shared.build.clone(),
            standards: shared.standards.clone(),
            isolation_profile: shared.isolation.clone(),
        },
        argv: List::new(vec![text(DRIVER_DESTINATION)?]).map_err(|_| Refusal::Encoding("argv"))?,
        environment: shared.environment.clone(),
        grants: shared.grants.clone(),
        limits: limits_record,
        allowed_effects: shared.effects.clone(),
        cleanup_contract: contract,
        expectation: reviewed.expectation().clone(),
        case_design_review: reviewed.review().clone(),
    };
    let prepared = prepare_u64(attempt).map_err(Refusal::Plan)?;
    Ok(Planned {
        prepared,
        objects: sink.registered().values().cloned().collect(),
        added: sink.registered().len().saturating_sub(before),
    })
}

/// A file of a snapshot by path, or `Editable` naming the path the snapshot lacks.
fn snapshot_file<'a>(snapshot: &'a Snapshot, file: &'static str) -> Result<&'a [u8], Refusal> {
    snapshot
        .entries()
        .find_map(|entry| match &entry.content {
            workspace::Content::File { bytes, .. } if entry.path == file => Some(bytes.as_slice()),
            _ => None,
        })
        .ok_or(Refusal::Editable(file))
}

/// The digest the receipt's `schema_sha256` names: the schema bytes this binary carries.
#[must_use]
pub fn schema_sha256() -> String {
    evidence::digest(SCHEMA)
}

#[cfg(test)]
mod tests {
    use super::{
        COLLECTOR_UNREAD, CheckInputs, Inputs, PinWhich, Refusal, SCHEMA, SCHEMA_STANDARD,
        SettleStep, VersionWhy, check, isolation_profile, schema_sha256, settle_step, shared,
    };
    use crate::app::class_profile::{Profile, compose};
    use crate::app::evidence::Evidence;
    use crate::app::workload::{COMPILER_DESTINATION, Tools};
    use crate::check::consistency;
    use crate::check::graph::{Graph, Objects as _};
    use crate::check::u64_oracle::FrozenOracle;
    use crate::contracts::receipt::{
        BuildProfileV1, CleanupContractV1, EffectPageV1, GrantPageV1, LanguageFlagsPageV1,
        LimitsV1, LockPageV1, ObligationPageV1, Ref, StandardPageV1, SubjectFilePageV1,
        SubjectFileV1Origin, SubjectV1, ToolPageV1, decode,
    };
    use crate::store::ArtifactStaging;
    use crate::worker::namespace::ReadOnlyFile;
    use crate::worker::workspace::Snapshot;
    use sha2::{Digest as _, Sha256};
    use std::fs;
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    static N: AtomicU64 = AtomicU64::new(1);
    const ORACLE: &[u8] =
        include_bytes!("../../evaluation/tasks/WL-U64-PARSE-001/v1/oracle/cases.json");
    const WRAPPER: &[u8] = include_bytes!("../../evaluation/harnesses/u64-public-wrapper.rs");
    const BASE: &[u8] =
        include_bytes!("../../evaluation/tasks/WL-U64-PARSE-001/v1/base/src/lib.rs");
    const REFERENCE: &[u8] =
        include_bytes!("../../evaluation/tasks/WL-U64-PARSE-001/v1/reference/src/lib.rs");
    const AUTHORITY: &[u8] = b"{\"authority\":\"WL-U64 fixed workload\",\"issuer\":\"operator\"}\n";
    const SPECIFICATION: &[u8] = b"{\"isolation\":\"bwrap --unshare-all; no network\"}\n";
    const PAYLOADS: usize = 6;
    const PAGES: usize = 8;
    const SUBJECT_OBJECTS: usize = 3 * 4 + 4 + 2;
    const CLOSURE_NODES: usize = 22;
    /// coreutils `sha256sum` of `AUTHORITY` and `SPECIFICATION`, the independent source.
    const AUTH: &str = "sha256:e4b4fcd5d15d45eed7a553536e73027e7aedbd72f466be6dee168986b6fe1234";
    const SPEC: &str = "sha256:61ac0e7ad368ef48bd6d34e7cd6995ca5ff992693b700b9fccf3f58c8cfc32f1";
    const SCOPE: &str =
        "compile, link and execute the fixed workload in a private bounded scratch; no network";

    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(120)
    }

    fn private(name: &str) -> PathBuf {
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap_or_else(|_| std::env::temp_dir())
            .join(format!(
                "hee3-plan-{name}-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
        assert!(fs::DirBuilder::new().mode(0o700).create(&root).is_ok());
        root
    }

    fn write(path: &Path, bytes: &[u8]) {
        assert!(fs::write(path, bytes).is_ok());
        assert!(fs::set_permissions(path, fs::Permissions::from_mode(0o600)).is_ok());
    }

    fn tree(root: &Path, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let dir = root.join(name);
        assert!(fs::DirBuilder::new().mode(0o700).create(&dir).is_ok());
        for (file, bytes) in files {
            write(&dir.join(file), bytes);
        }
        dir
    }

    /// The compiler this test observes: `RUSTC` when the gate sets it, else the toolchain binary
    /// behind `rustc` on `PATH` (`rustc --print sysroot`, so a rustup proxy — which needs the
    /// host's environment to resolve a toolchain — is never the pinned executable).
    fn rustc() -> Result<PathBuf, Box<dyn std::error::Error>> {
        if let Some(path) = std::env::var_os("RUSTC") {
            return Ok(PathBuf::from(path));
        }
        let sysroot = std::process::Command::new("rustc")
            .arg("--print")
            .arg("sysroot")
            .output()?;
        let sysroot = String::from_utf8(sysroot.stdout)?;
        let path = PathBuf::from(sysroot.trim()).join("bin/rustc");
        if !path.is_file() {
            return Err(format!("no toolchain rustc at {}", path.display()).into());
        }
        Ok(path)
    }

    fn sha(bytes: &[u8]) -> [u8; 32] {
        Sha256::digest(bytes).into()
    }

    struct Fixture {
        root: PathBuf,
        profile: Profile,
        tools: Tools,
        baseline: Snapshot,
        protected: Snapshot,
    }

    /// A class directory holding the 003 reviewed closure, a WL-U64 baseline and protected tree,
    /// and tools whose pins are the digests of the files they name: the real compiler and
    /// bwrap, and a stand-in shim (any pinned bytes — the plan reads, verifies and copies it).
    fn fixture() -> Result<Fixture, Box<dyn std::error::Error>> {
        let root = private("shared");
        let class = tree(&root, "class", &[]);
        let reviewed = tree(&class, "reviewed", &[]);
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reviewed-003");
        for entry in fs::read_dir(fixture)? {
            let entry = entry?;
            write(&reviewed.join(entry.file_name()), &fs::read(entry.path())?);
        }
        let base = tree(&class, "base", &[]);
        tree(&base, "src", &[("lib.rs", BASE)]);
        let protected = tree(
            &class,
            "protected",
            &[("oracle.json", ORACLE), ("public-wrapper.rs", WRAPPER)],
        );
        let shim_bytes = b"#!/bin/sh\nexit 0\n";
        let shim = class.join("stand-in-shim");
        write(&shim, shim_bytes);
        let compiler = rustc()?;
        let compiler_bytes = fs::read(&compiler)?;
        let zero = format!("sha256:{}", "0".repeat(64));
        let text = format!(
            "schema = \"hee3.class-profile/1\"\nclass = \"rust-library-change/1\"\n\n\
             [[workspace]]\nid = \"28f80000-0000-4000-8000-000000000001\"\nbaseline = \"base\"\n\
             baseline_digest = \"{zero}\"\nprotected = \"protected\"\nprotected_digest = \"{zero}\"\n\n\
             [pins]\ncompiler = {{ host = \"{}\", sha256 = \"sha256:{}\" }}\n\
             shim = {{ host = \"{}\", sha256 = \"sha256:{}\" }}\nruntime_files = []\n\
             namespace_directories = []\nbusctl_sha256 = \"{zero}\"\nsystemd_run_sha256 = \"{zero}\"\n\
             [reviewed]\nexpectation = {{ artifact_id = \"c220e7ce-0753-47ef-bdac-15710bc4981c\", sha256 = \"sha256:3a7faa5510790c20322ad5829091211eb8c04ab6016e16ec8391433dae3392b9\", byte_length = 794, media_type = \"application/json\", schema_id = \"hee3.receipt/1:ExpectationV1\" }}\n\
             review = {{ artifact_id = \"a47470c5-f11c-4f64-9ac8-6dcf80750ed6\", sha256 = \"sha256:f288225476120254f5c3a93266fc8f2a8763810462fbddd7c62161107cb39adb\", byte_length = 1145, media_type = \"application/json\", schema_id = \"hee3.receipt/1:ReviewV1\" }}\n\
             [grant]\ngrant_id = \"28f90000-0000-4000-8000-000000000001\"\nissuer_id = \"operator\"\n\
             authority = {{ file = \"authority.json\", sha256 = \"{}\" }}\n\
             [effect]\neffect_id = \"fixed-u64-workload-output\"\nscope = \"{SCOPE}\"\n\
             specification = {{ file = \"isolation.json\", sha256 = \"{}\" }}\n",
            compiler.display(),
            hex(&sha(&compiler_bytes)),
            shim.display(),
            hex(&sha(shim_bytes)),
            crate::app::evidence::digest(AUTHORITY),
            crate::app::evidence::digest(SPECIFICATION),
        );
        write(&class.join("authority.json"), AUTHORITY);
        write(&class.join("isolation.json"), SPECIFICATION);
        // The profile is installed as the operator installs it, so its digest is the READ one — the
        // value the runtime re-reads at teardown, never an empty placeholder.
        write(&class.join("profile.toml"), text.as_bytes());
        let declared = compose(text.as_bytes()).map_err(|e| format!("{e:?}"))?;
        let tools = Tools {
            bwrap: PathBuf::from(super::BWRAP),
            compiler: ReadOnlyFile {
                host: compiler,
                namespace: COMPILER_DESTINATION.into(),
                sha256: sha(&compiler_bytes),
            },
            shim: ReadOnlyFile {
                host: shim,
                namespace: "/shim/namespace-shim".into(),
                sha256: sha(shim_bytes),
            },
            runtime_files: Vec::new(),
            namespace_directories: Vec::new(),
        };
        let digest = super::super::class_profile::read(&class)
            .map_err(|e| format!("{e:?}"))?
            .digest;
        Ok(Fixture {
            profile: Profile {
                declared,
                directory: class,
                digest,
            },
            tools,
            baseline: Snapshot::capture(&base, &[], deadline()).map_err(|e| format!("{e:?}"))?,
            protected: Snapshot::capture(&protected, &[], deadline())
                .map_err(|e| format!("{e:?}"))?,
            root,
        })
    }

    fn hex(bytes: &[u8; 32]) -> String {
        bytes.iter().fold(String::new(), |mut text, byte| {
            use std::fmt::Write as _;
            let _ = write!(text, "{byte:02x}");
            text
        })
    }

    fn subject_paths(
        sink: &Evidence<'_>,
        subject: &SubjectV1,
    ) -> Vec<(String, SubjectFileV1Origin)> {
        let mut bytes = Vec::new();
        assert!(
            sink.open(subject.files.as_ref())
                .and_then(|mut r| std::io::Read::read_to_end(&mut r, &mut bytes)
                    .map_err(|_| crate::check::graph::Error::Io))
                .is_ok()
        );
        let page: SubjectFilePageV1 = decode(&bytes).unwrap_or_else(|_| unreachable!("a page"));
        page.rows
            .as_slice()
            .iter()
            .map(|row| (row.path.as_str().to_owned(), row.origin))
            .collect()
    }

    fn resolved<T: crate::contracts::receipt::ReceiptRecord + serde::de::DeserializeOwned>(
        sink: &Evidence<'_>,
        reference: &crate::contracts::receipt::TypedRef<T>,
    ) -> T {
        let graph =
            Graph::resolve(sink, reference.as_ref()).unwrap_or_else(|e| unreachable!("{e:?}"));
        decode(
            graph
                .get(reference.as_ref())
                .unwrap_or_else(|e| unreachable!("{e:?}"))
                .bytes(),
        )
        .unwrap_or_else(|_| unreachable!("a typed record"))
    }

    /// The grant and effect rows, whole.
    fn assert_authority(
        sink: &Evidence<'_>,
        shared: &super::Shared,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // The grant and effect rows, whole: the declaration's ids, the authority's digest as the
        // grant's scope, the effect under the grant and owned by the runtime.
        let grants: GrantPageV1 = resolved(sink, &shared.grants);
        assert_eq!(
            (grants.row_count, grants.total_rows, grants.page_count),
            (1, 1, 1)
        );
        let grant = &grants.rows.as_slice()[0];
        assert_eq!(
            (
                grant.grant_id.as_str(),
                grant.issuer_id.as_str(),
                grant.scope_sha256.as_str(),
                grant.grant.as_ref().sha256.as_str()
            ),
            (
                "28f90000-0000-4000-8000-000000000001",
                "operator",
                AUTH,
                AUTH
            )
        );
        assert_eq!(
            u64::from(grant.grant.as_ref().byte_length),
            u64::try_from(AUTHORITY.len())?
        );
        let effects: EffectPageV1 = resolved(sink, &shared.effects);
        assert_eq!(
            (effects.row_count, effects.total_rows, effects.page_count),
            (1, 1, 1)
        );
        let effect = &effects.rows.as_slice()[0];
        assert_eq!(
            (
                effect.effect_id.as_str(),
                effect.grant_id.as_str(),
                effect.owner_id.as_str(),
                effect.scope.as_str(),
                effect.specification.as_ref().sha256.as_str()
            ),
            (
                "fixed-u64-workload-output",
                "28f90000-0000-4000-8000-000000000001",
                crate::app::u64_receipt::RUNTIME_OWNER,
                SCOPE,
                SPEC
            )
        );
        assert_eq!(
            u64::from(effect.specification.as_ref().byte_length),
            u64::try_from(SPECIFICATION.len())?
        );
        assert!(
            serde_json::from_slice::<serde_json::Value>(
                &isolation_profile().map_err(|e| format!("{e:?}"))?
            )
            .is_ok()
        );
        Ok(())
    }

    /// One dispatch's shared publication, asserted whole against the sink that holds it.
    fn assert_dispatch(
        sink: &Evidence<'_>,
        shared: &super::Shared,
        f: &Fixture,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Every reference the shared plan names is among the objects it carries over.
        for reference in [
            shared.schema.as_ref(),
            shared.isolation.as_ref(),
            shared.toolchain.as_ref(),
            shared.build.as_ref(),
            shared.locks.as_ref(),
            shared.standards.as_ref(),
            shared.environment.as_ref(),
            shared.grants.as_ref(),
            shared.effects.as_ref(),
            shared.seed.as_ref(),
            shared.fixtures.as_ref(),
            shared.oracle.as_ref(),
            shared.harness.as_ref(),
            shared.launcher.as_ref(),
            shared.collector.as_ref(),
        ] {
            assert!(
                shared.objects.iter().any(|(held, _)| held == reference),
                "{reference:?} carried"
            );
        }
        // The toolchain row: the pinned compiler, its version as an independent probe reads it.
        let toolchain: ToolPageV1 = resolved(sink, &shared.toolchain);
        let row = &toolchain.rows.as_slice()[0];
        let probe = std::process::Command::new(&f.tools.compiler.host)
            .arg("-Vv")
            .output()?;
        let release = String::from_utf8(probe.stdout)?
            .lines()
            .find_map(|line| line.strip_prefix("release: ").map(str::to_owned))
            .ok_or("release line")?;
        assert_eq!(
            (row.version.as_str(), &shared.compiler_version),
            (release.as_str(), &release)
        );
        assert_eq!(
            row.executable_path.as_str(),
            f.tools.compiler.host.to_str().ok_or("path")?
        );
        assert_eq!(
            row.executable.as_ref().byte_length as usize,
            fs::read(&f.tools.compiler.host)?.len()
        );
        let build: BuildProfileV1 = resolved(sink, &shared.build);
        assert_eq!(
            (
                build.target.as_str(),
                build.default_features,
                build.features.as_slice().len(),
                build.build_profile.as_str()
            ),
            (super::BUILD_TARGET, false, 0, super::BUILD_PROFILE)
        );
        let flags: LanguageFlagsPageV1 = resolved(sink, &build.language_flags);
        let argv: Vec<&str> = flags.rows.as_slice()[0]
            .argv
            .as_slice()
            .iter()
            .map(crate::contracts::receipt::Text::as_str)
            .collect();
        assert_eq!(argv, super::COMPILE_FLAGS.to_vec());
        // The fixtures subject holds the oracle's PUBLIC projection, never the oracle itself.
        let fixtures: SubjectV1 = resolved(sink, &shared.fixtures);
        let fixtures_page: SubjectFilePageV1 = resolved(sink, &fixtures.files);
        let projection = fixtures_page.rows.as_slice()[0]
            .content
            .value
            .as_ref()
            .ok_or("fixtures content")?;
        let expected = FrozenOracle::from_bytes(ORACLE)
            .map_err(|e| format!("{e:?}"))?
            .public_inputs();
        assert_eq!(
            projection.as_ref().sha256.as_str(),
            crate::app::evidence::digest(&expected)
        );
        assert_ne!(
            projection.as_ref().sha256.as_str(),
            crate::app::evidence::digest(ORACLE)
        );
        let locks: LockPageV1 = resolved(sink, &shared.locks);
        assert_eq!((locks.row_count, locks.total_rows), (0, 0));
        let standards: StandardPageV1 = resolved(sink, &shared.standards);
        let standard = &standards.rows.as_slice()[0];
        assert_eq!(
            (standard.standard_id.as_str(), standard.revision.as_str()),
            SCHEMA_STANDARD
        );
        assert_eq!(standard.document, shared.schema);
        assert_eq!(shared.schema.as_ref().byte_length as usize, SCHEMA.len());
        assert_eq!(shared.schema.as_ref().sha256.as_str(), schema_sha256());
        assert_authority(sink, shared)?;
        assert_subjects(sink, shared, f)?;
        Ok(())
    }

    /// The six subjects by role, and the closures' nodes in the registry.
    fn assert_subjects(
        sink: &Evidence<'_>,
        shared: &super::Shared,
        f: &Fixture,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // The subjects: each role's files, the collector excluded with its reason.
        for (role, reference, files) in [
            (
                "seed",
                &shared.seed,
                vec![
                    ("src".to_owned(), SubjectFileV1Origin::Authored),
                    ("src/lib.rs".to_owned(), SubjectFileV1Origin::Authored),
                ],
            ),
            (
                "fixtures",
                &shared.fixtures,
                vec![("inputs.hex".to_owned(), SubjectFileV1Origin::Authored)],
            ),
            (
                "oracle",
                &shared.oracle,
                vec![("oracle.json".to_owned(), SubjectFileV1Origin::Authored)],
            ),
            (
                "harness",
                &shared.harness,
                vec![(
                    "public-wrapper.rs".to_owned(),
                    SubjectFileV1Origin::Authored,
                )],
            ),
            (
                "launcher",
                &shared.launcher,
                vec![
                    ("bwrap".to_owned(), SubjectFileV1Origin::Authored),
                    ("namespace-shim".to_owned(), SubjectFileV1Origin::Authored),
                ],
            ),
            (
                "collector",
                &shared.collector,
                vec![("collector".to_owned(), SubjectFileV1Origin::Excluded)],
            ),
        ] {
            let subject: SubjectV1 = resolved(sink, reference);
            assert_eq!(subject_paths(sink, &subject), files, "{role}");
        }
        let collector: SubjectV1 = resolved(sink, &shared.collector);
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(
            &mut sink
                .open(collector.files.as_ref())
                .map_err(|e| format!("{e:?}"))?,
            &mut bytes,
        )?;
        assert!(std::str::from_utf8(&bytes)?.contains(COLLECTOR_UNREAD));
        // The reviewed closures: every node of the review's 22 registered under its own id.
        let review = f.profile.declared.reviewed.review().as_ref().clone();
        let graph = Graph::resolve(sink, &review).map_err(|e| format!("{e:?}"))?;
        assert_eq!(graph.object_count(), 22);
        assert!(
            sink.registered().contains_key(
                f.profile
                    .declared
                    .reviewed
                    .expectation()
                    .as_ref()
                    .artifact_id
                    .as_str()
            )
        );
        Ok(())
    }

    /// Every refusal named, none leaving a plan root behind.
    fn assert_refused(
        f: &Fixture,
        staging: &ArtifactStaging,
        cancelled: &AtomicBool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Refusals, each named, each leaving no plan root.
        let mut sink = Evidence::staged(staging, deadline());
        let plan_root = f.root.join("refused.plan");
        let base = Inputs {
            tools: &f.tools,
            profile: &f.profile,
            baseline: &f.baseline,
            protected: &f.protected,
            plan_root: &plan_root,
            deadline: deadline(),
            cancelled,
        };
        assert!(matches!(
            shared(
                &mut sink,
                &Inputs {
                    deadline: Instant::now(),
                    ..base
                }
            ),
            Err(Refusal::Deadline)
        ));
        let stop = AtomicBool::new(true);
        assert!(matches!(
            shared(
                &mut sink,
                &Inputs {
                    cancelled: &stop,
                    ..base
                }
            ),
            Err(Refusal::Cancelled)
        ));
        let mut wrong = Tools {
            bwrap: f.tools.bwrap.clone(),
            compiler: f.tools.compiler.clone(),
            shim: f.tools.shim.clone(),
            runtime_files: Vec::new(),
            namespace_directories: Vec::new(),
        };
        wrong.compiler.sha256[0] ^= 0x01;
        assert!(matches!(
            shared(
                &mut sink,
                &Inputs {
                    tools: &wrong,
                    ..base
                }
            ),
            Err(Refusal::Pin {
                which: PinWhich::Compiler,
                ..
            })
        ));
        assert!(!plan_root.exists());
        // The shim's pin, which no probe re-hashes: the read under the pin is the only door.
        let mut wrong_shim = Tools {
            bwrap: f.tools.bwrap.clone(),
            compiler: f.tools.compiler.clone(),
            shim: f.tools.shim.clone(),
            runtime_files: Vec::new(),
            namespace_directories: Vec::new(),
        };
        wrong_shim.shim.sha256[0] ^= 0x01;
        assert!(matches!(
            shared(
                &mut sink,
                &Inputs {
                    tools: &wrong_shim,
                    ..base
                }
            ),
            Err(Refusal::Pin {
                which: PinWhich::Shim,
                ..
            })
        ));
        assert!(!plan_root.exists());
        let unprotected = Snapshot::capture(&tree(&f.root, "empty", &[]), &[], deadline())
            .map_err(|e| format!("{e:?}"))?;
        assert!(matches!(
            shared(
                &mut sink,
                &Inputs {
                    protected: &unprotected,
                    ..base
                }
            ),
            Err(Refusal::Protected("oracle.json"))
        ));
        assert_refused_sink(&mut sink, staging, &base, f)?;
        assert_refused_probe(&mut sink, &base, &plan_root, f)?;
        Ok(())
    }

    /// A sink already holding a closure id under another reference, a declared file whose bytes
    /// are not the declaration's, and a plan root under a symlinked parent: each refused by name.
    fn assert_refused_sink(
        sink: &mut Evidence<'_>,
        staging: &ArtifactStaging,
        base: &Inputs<'_>,
        f: &Fixture,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let base = *base;
        // A sink already holding the expectation's id under another reference: named, not skipped.
        {
            use crate::check::collector::Sink as _;
            let mut held = Evidence::staged(staging, deadline());
            let other = b"{\"not\":\"the expectation\"}";
            let reference = Ref {
                artifact_id: f
                    .profile
                    .declared
                    .reviewed
                    .expectation()
                    .as_ref()
                    .artifact_id
                    .clone(),
                sha256: crate::contracts::receipt::Sha::new(crate::app::evidence::digest(other))?,
                byte_length: u32::try_from(other.len())?,
                media_type: crate::contracts::receipt::Name::new("application/json")?,
                schema_id: crate::contracts::receipt::Name::new("hee3.raw/1")?,
            };
            held.publish(&reference, other)
                .map_err(|e| format!("{e:?}"))?;
            let conflict_root = f.root.join("conflict.plan");
            let refused = shared(
                &mut held,
                &Inputs {
                    plan_root: &conflict_root,
                    ..base
                },
            );
            assert!(
                matches!(&refused, Err(Refusal::ClosureConflict { artifact_id }) if artifact_id == reference.artifact_id.as_str()),
                "{refused:?}"
            );
            assert!(!conflict_root.exists());
        }
        // A declared file whose bytes are not the declaration's: each refused by name before
        // anything is published, the plan root removed; then restored.
        for (file, which, other, original) in [
            ("authority.json", "authority", SPECIFICATION, AUTHORITY),
            ("isolation.json", "specification", AUTHORITY, SPECIFICATION),
        ] {
            write(&f.profile.directory.join(file), other);
            let before = sink.registered().len();
            let refused = shared(sink, &base);
            assert!(
                matches!(
                    &refused,
                    Err(Refusal::Declared { which: named, error: crate::app::class_profile::ReviewedError::Mismatch }) if *named == which
                ),
                "{refused:?}"
            );
            assert_eq!(
                sink.registered().len(),
                before,
                "nothing published before the declared read"
            );
            assert!(!base.plan_root.exists());
            write(&f.profile.directory.join(file), original);
        }
        // A plan root under a symlinked parent is refused before anything is published.
        let alias = f.root.join("alias");
        std::os::unix::fs::symlink(&f.root, &alias)?;
        let before = sink.registered().len();
        assert!(matches!(
            shared(
                sink,
                &Inputs {
                    plan_root: &alias.join("aliased.plan"),
                    ..base
                }
            ),
            Err(Refusal::PlanRootNotCanonical)
        ));
        assert_eq!(sink.registered().len(), before);
        Ok(())
    }

    /// A compiler whose bytes are its pin but whose version report is not one, then a leftover plan
    /// root and a missing closure member: each refused at its own site.
    fn assert_refused_probe(
        sink: &mut Evidence<'_>,
        base: &Inputs<'_>,
        plan_root: &Path,
        f: &Fixture,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let base = *base;
        // A compiler whose bytes ARE its pin but which exits non-zero, and one whose version report
        // has no release: each refused at its own site.
        for (script, why) in [
            (&b"#!/bin/sh\nexit 3\n"[..], VersionWhy::Exit),
            (
                &b"#!/bin/sh\nprintf 'rustc 0.0.0\\nrelease:  \\n'\n"[..],
                VersionWhy::Release,
            ),
        ] {
            let stand_in = f.root.join(format!("compiler-{why:?}"));
            write(&stand_in, script);
            fs::set_permissions(&stand_in, fs::Permissions::from_mode(0o700))?;
            let tools = Tools {
                bwrap: f.tools.bwrap.clone(),
                compiler: ReadOnlyFile {
                    host: stand_in,
                    namespace: f.tools.compiler.namespace.clone(),
                    sha256: sha(script),
                },
                shim: f.tools.shim.clone(),
                runtime_files: Vec::new(),
                namespace_directories: Vec::new(),
            };
            let refused = shared(
                sink,
                &Inputs {
                    tools: &tools,
                    ..base
                },
            );
            assert!(
                matches!(refused, Err(Refusal::CompilerVersion { why: site }) if site == why),
                "{why:?}: {refused:?}"
            );
            assert!(!plan_root.exists());
        }
        let leftover = tree(&f.root, "leftover.plan", &[]);
        assert!(matches!(
            shared(
                sink,
                &Inputs {
                    plan_root: &leftover,
                    ..base
                }
            ),
            Err(Refusal::PlanRootExists)
        ));
        let member = f
            .profile
            .directory
            .join("reviewed")
            .join("8352c1851cfba4e85f26d180f4174bc572cdecc01d08b40d7efeab16f1dc0d81");
        fs::remove_file(&member)?;
        assert!(matches!(
            shared(sink, &base),
            Err(Refusal::Closure(
                crate::app::class_profile::ReviewedError::Closure(
                    crate::check::graph::Error::Missing
                )
            ))
        ));
        assert!(!plan_root.exists());
        Ok(())
    }

    /// R17 round 2, proof (a), the shared part: over a staged sink, the plan publishes the schema,
    /// the pages, the six subjects and both reviewed closures; every reference resolves through
    /// the graph; the toolchain row's version is the `release:` line an independent `rustc -Vv`
    /// prints; the locks page is empty and the standards row is the schema; the collector is the
    /// one excluded row; the closures' 22 nodes are in the registry; a second dispatch over a
    /// fresh sink adds the same number of digests (F129: nothing pinned at an origin); and each
    /// refusal is named without leaving a plan root behind.
    #[test]
    fn the_shared_plan_publishes_every_pre_execution_input_from_its_pin()
    -> Result<(), Box<dyn std::error::Error>> {
        let f = fixture()?;
        let staging_root = tree(&f.root, "staging", &[]);
        let staging =
            ArtifactStaging::open(&staging_root, true, deadline()).map_err(|e| format!("{e:?}"))?;
        let cancelled = AtomicBool::new(false);
        let mut added = Vec::new();
        // One sink across two dispatches: the second finds the closures already registered under
        // the same references and adds only the fresh-id part, so both counts are pinned off the
        // origin and the registry delta (`before`) is exercised (F129, review MED-3).
        let mut sink = Evidence::staged(&staging, deadline());
        for dispatch in 0..2 {
            let plan_root = f.root.join(format!("dispatch-{dispatch}.plan"));
            let inputs = Inputs {
                tools: &f.tools,
                profile: &f.profile,
                baseline: &f.baseline,
                protected: &f.protected,
                plan_root: &plan_root,
                deadline: deadline(),
                cancelled: &cancelled,
            };
            let shared = shared(&mut sink, &inputs).map_err(|e| format!("{e:?}"))?;
            assert!(!plan_root.exists(), "the plan root is torn down");
            added.push(shared.added);
            assert_dispatch(&sink, &shared, &f)?;
        }
        // The graph-side count, derived from the parts: 6 payloads (schema, compiler, version,
        // isolation, authority, specification) + 8 pages/records (grants, effects, toolchain, flags,
        // build, locks, standards, environment) + 18 subject objects (seed, fixtures, oracle, harness
        // 3 each; launcher 4; collector 2) + the review closure's 22 nodes; the second dispatch on
        // the same sink skips the 22 it finds registered.
        let s_new = PAYLOADS + PAGES + SUBJECT_OBJECTS + CLOSURE_NODES;
        assert_eq!(added, vec![s_new, s_new - CLOSURE_NODES], "{added:?}");
        assert_eq!(sink.registered().len(), 2 * s_new - CLOSURE_NODES);
        // N4, graph side: a fresh store's 4096-object inventory admits
        // (4096 − S_new − baseline) / 8 checks after the first dispatch; the store-side count is
        // at most this (the CAS dedupes) and is read by the Tier-3 host run.
        println!(
            "S_new={s_new} registry entries per dispatch (graph side); per_dispatch_after_first={}; \
             checks_until_bound_graph_side={}",
            s_new - CLOSURE_NODES,
            (4096 - s_new) / 8
        );
        assert_refused(&f, &staging, &cancelled)?;
        fs::remove_dir_all(&f.root)?;
        Ok(())
    }

    /// A stand-in compiler whose bytes are its pin, as a shell script with `body`, 0700.
    fn stand_in(f: &Fixture, name: &str, body: &str) -> Result<Tools, Box<dyn std::error::Error>> {
        let path = f.root.join(name);
        write(&path, body.as_bytes());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
        Ok(Tools {
            bwrap: f.tools.bwrap.clone(),
            compiler: ReadOnlyFile {
                host: path,
                namespace: f.tools.compiler.namespace.clone(),
                sha256: sha(body.as_bytes()),
            },
            shim: f.tools.shim.clone(),
            runtime_files: Vec::new(),
            namespace_directories: Vec::new(),
        })
    }

    /// The refusal a compiler pin earns when its bytes are not the pin's, or its path cannot be
    /// opened as a regular file.
    fn assert_compiler_digest(refused: &Result<super::Shared, Refusal>) {
        assert!(
            matches!(
                refused,
                Err(Refusal::Pin {
                    which: PinWhich::Compiler,
                    error: crate::worker::namespace::NamespaceError::Digest
                })
            ),
            "{refused:?}"
        );
    }

    /// Review 2c-ii-c MED-1/MED-2 · the doors the first proof never reached: a compiler that
    /// rewrites itself while the probe runs is refused by the post-probe re-hash; a symlink at the
    /// compiler's path is refused at the open whatever digest it is pinned under; a FIFO pinned at
    /// the empty input's digest is refused by the regular-file check alone (a child-leaving probe is
    /// not exercised here — see the comment in the body).
    #[test]
    fn the_probe_and_the_pin_doors_refuse_what_the_first_proof_never_reached()
    -> Result<(), Box<dyn std::error::Error>> {
        let f = fixture()?;
        let staging_root = tree(&f.root, "staging", &[]);
        let staging =
            ArtifactStaging::open(&staging_root, true, deadline()).map_err(|e| format!("{e:?}"))?;
        let cancelled = AtomicBool::new(false);
        let mut sink = Evidence::staged(&staging, deadline());
        let plan_root = f.root.join("doors.plan");
        let base = Inputs {
            tools: &f.tools,
            profile: &f.profile,
            baseline: &f.baseline,
            protected: &f.protected,
            plan_root: &plan_root,
            deadline: deadline(),
            cancelled: &cancelled,
        };
        // Rewrites itself after printing a release: the bytes that ran are no longer the pin's.
        // `$0` under the probe is a basename in the plan root, so the script names its own path.
        let self_path = f.root.join("compiler-rewriting");
        let rewriting = stand_in(
            &f,
            "compiler-rewriting",
            &format!(
                "#!/bin/sh\nprintf 'rustc 0.0.0\\nrelease: 1.99.0\\n'\nprintf '\\n# changed\\n' >> {}\nexit 0\n",
                self_path.display()
            ),
        )?;
        let refused = shared(
            &mut sink,
            &Inputs {
                tools: &rewriting,
                ..base
            },
        );
        assert_compiler_digest(&refused);
        assert!(!plan_root.exists());
        // A probe that leaves a background child (`sleep 30 &` in a stand-in) is NOT a proof here,
        // MEASURED 2026-09-26: the plan accepted the probe (the report carried no pending child and
        // no interruption), and the orphan — in the leader's group, reparented to whatever subreaper
        // owns the test process, live or as a zombie after the group kill — was counted by the repo
        // gate's harness as a leaked descendant (`gate-runs/20260926T090043895328Z`), failing a
        // step whose every test passed. The pin over the compiler's bytes is the guard against a
        // probe that forks; the settle loop is reached only when the process module hands a pending
        // child back, and no stand-in reaches it. Recorded in DESIGN.md (B14a-4 item); no test here
        // may leave a process.
        // A symlink to the real compiler, pinned with the real digest: never followed.
        let link = f.root.join("compiler-link");
        std::os::unix::fs::symlink(&f.tools.compiler.host, &link)?;
        let mut linked = stand_in(&f, "compiler-unused", "#!/bin/sh\nexit 0\n")?;
        linked.compiler.host = link;
        linked.compiler.sha256 = f.tools.compiler.sha256;
        let refused = shared(
            &mut sink,
            &Inputs {
                tools: &linked,
                ..base
            },
        );
        assert_compiler_digest(&refused);
        // A FIFO at the path: not a regular file, refused at the open without a read.
        let fifo = f.root.join("compiler-fifo");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()?
                .success()
        );
        // Pinned at the empty input's digest: opened non-blocking with no writer, a FIFO reads
        // zero bytes and would HASH TO ITS PIN, so only the regular-file check can refuse it.
        let mut piped = stand_in(&f, "compiler-unused-2", "")?;
        piped.compiler.host = fifo;
        let refused = shared(
            &mut sink,
            &Inputs {
                tools: &piped,
                ..base
            },
        );
        assert_compiler_digest(&refused);
        assert!(!plan_root.exists());
        fs::remove_dir_all(&f.root)?;
        Ok(())
    }

    /// The per-check plan's refusals by name: ids that are not pairwise distinct, and an applied
    /// tree without the editable file.
    fn assert_check_refused(
        sink: &mut Evidence<'_>,
        shared: &super::Shared,
        f: &Fixture,
        applied: &Snapshot,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Refusals by name: ids that are not pairwise distinct; an applied tree without the editable.
        let obligation_ids: [String; 4] =
            std::array::from_fn(|i| format!("28fa0000-0000-4000-8000-00000000009{i}"));
        let refused = check(
            sink,
            &CheckInputs {
                shared,
                profile: &f.profile,
                baseline: &f.baseline,
                applied,
                task: "28fa0000-0000-4000-8000-0000000000a0",
                attempt: "28fa0000-0000-4000-8000-0000000000a0",
                run: "28fa0000-0000-4000-8000-0000000000f0",
                generation: "3",
                parent_run: None,
                obligation_ids: &obligation_ids,
                wall_ms: 250_000,
                deadline: deadline(),
            },
        );
        assert!(
            matches!(refused, Err(Refusal::Plan(consistency::Error::Binding))),
            "{refused:?}"
        );
        let bare = Snapshot::capture(&tree(&f.root, "bare", &[]), &[], deadline())
            .map_err(|e| format!("{e:?}"))?;
        let refused = check(
            sink,
            &CheckInputs {
                shared,
                profile: &f.profile,
                baseline: &f.baseline,
                applied: &bare,
                task: "28fa0000-0000-4000-8000-0000000000a0",
                attempt: "28fa0000-0000-4000-8000-0000000000b3",
                run: "28fa0000-0000-4000-8000-0000000000f3",
                generation: "3",
                parent_run: None,
                obligation_ids: &obligation_ids,
                wall_ms: 250_000,
                deadline: deadline(),
            },
        );
        assert!(
            matches!(refused, Err(Refusal::Editable(super::U64_EDITABLE))),
            "{refused:?}"
        );
        let before = sink.registered().len();
        let refused = check(
            sink,
            &CheckInputs {
                shared,
                profile: &f.profile,
                baseline: &f.baseline,
                applied,
                task: "28fa0000-0000-4000-8000-0000000000a0",
                attempt: "28fa0000-0000-4000-8000-0000000000b4",
                run: "28fa0000-0000-4000-8000-0000000000f4",
                generation: "3",
                parent_run: None,
                obligation_ids: &obligation_ids,
                wall_ms: 250_000,
                deadline: Instant::now(),
            },
        );
        assert!(matches!(refused, Err(Refusal::Deadline)), "{refused:?}");
        assert_eq!(
            sink.registered().len(),
            before,
            "a plan refused at its deadline published nothing"
        );
        Ok(())
    }

    /// One attempt's caller-varied fields (F129: two attempts differ in every one).
    struct Attempt<'a> {
        n: u8,
        generation: &'a str,
        parent: Option<&'a str>,
        obligation_ids: &'a [String; 4],
        profile_digest: &'a str,
    }

    /// One per-check plan asserted whole against its inputs: the identity, the invocation (the
    /// limits and the cleanup contract decoded from the sink — the wall differs per attempt — and
    /// the obligation page's four ids the caller's), and the subjects (the result's rows, the
    /// patch's own lines — a `+` line only the reference holds and a `-` line only the base holds —
    /// and every shared subject carried).
    fn assert_planned(
        sink: &Evidence<'_>,
        planned: &super::Planned,
        shared: &super::Shared,
        f: &Fixture,
        attempt: &Attempt<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(planned.prepared.schema_sha256.as_str(), schema_sha256());
        assert_identity(&planned.prepared.identity, attempt);
        assert_invocation(sink, &planned.prepared.invocation, shared, f, attempt);
        assert_planned_subjects(sink, &planned.prepared.subjects, shared)?;
        let case = &planned.prepared.cases[0];
        assert_eq!(
            case.reviewed_design.as_ref(),
            Some(f.profile.declared.reviewed.review())
        );
        assert_eq!(case.fixture_sha256, shared.fixtures.as_ref().sha256);
        Ok(())
    }

    fn assert_identity(identity: &crate::contracts::receipt::IdentityV1, attempt: &Attempt<'_>) {
        let n = attempt.n;
        assert_eq!(
            (
                identity.run_id.as_str(),
                identity.task_id.as_str(),
                identity.attempt_id.as_str(),
                identity.generation.as_str(),
                identity.module_id.as_str(),
                identity.profile_id.as_str(),
            ),
            (
                format!("28fa0000-0000-4000-8000-00000000001{n}").as_str(),
                "28fa0000-0000-4000-8000-0000000000a0",
                format!("28fa0000-0000-4000-8000-0000000000b{n}").as_str(),
                attempt.generation,
                "check",
                format!("rust-library-change/1@{}", attempt.profile_digest).as_str(),
            )
        );
        assert!(
            attempt.profile_digest.starts_with("sha256:") && attempt.profile_digest.len() == 71,
            "the fixture's profile digest is the read one, never empty: {}",
            attempt.profile_digest
        );
        assert_eq!(
            identity
                .parent_run
                .value
                .as_ref()
                .map(crate::contracts::receipt::Id::as_str),
            attempt.parent
        );
        if attempt.parent.is_none() {
            assert_eq!(
                identity
                    .parent_run
                    .unavailable_reason
                    .as_ref()
                    .map(crate::contracts::receipt::Text::as_str),
                Some(if attempt.generation == "1" {
                    "first_attempt"
                } else {
                    "separate_attempt_same_task"
                })
            );
        }
    }

    fn assert_invocation(
        sink: &Evidence<'_>,
        invocation: &crate::contracts::receipt::InvocationV1,
        shared: &super::Shared,
        f: &Fixture,
        attempt: &Attempt<'_>,
    ) {
        assert_eq!(
            invocation
                .argv
                .as_slice()
                .iter()
                .map(crate::contracts::receipt::Text::as_str)
                .collect::<Vec<_>>(),
            vec![super::DRIVER_DESTINATION]
        );
        assert_eq!(invocation.cwd_logical.as_str(), "work");
        assert_eq!(
            &invocation.expected,
            f.profile.declared.reviewed.expectation()
        );
        assert_eq!(invocation.grants, shared.grants);
        assert_eq!(invocation.allowed_effects, shared.effects);
        assert_eq!(invocation.environment, shared.environment);
        let wall = 250_000 + u64::from(attempt.n);
        let limits: LimitsV1 = resolved(sink, &invocation.limits);
        assert_eq!(
            (limits.wall_ms.get(), limits.cleanup_deadline_ms.get()),
            (wall, wall)
        );
        let contract: CleanupContractV1 = resolved(sink, &invocation.cleanup_contract);
        assert_eq!(contract.deadline_ms.get(), wall);
        let obligations: ObligationPageV1 = resolved(sink, &contract.obligations);
        assert_eq!(
            obligations
                .rows
                .as_slice()
                .iter()
                .map(|row| row.obligation_id.as_str().to_owned())
                .collect::<Vec<_>>(),
            attempt.obligation_ids.to_vec()
        );
    }

    fn assert_planned_subjects(
        sink: &Evidence<'_>,
        subjects: &crate::contracts::receipt::SubjectsV1,
        shared: &super::Shared,
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(subjects.seed_subject, shared.seed);
        let result = subjects
            .result_subject
            .value
            .as_ref()
            .ok_or("result present")?;
        assert_eq!(
            subject_paths(sink, &resolved(sink, result)),
            vec![
                ("src".to_owned(), SubjectFileV1Origin::Authored),
                ("src/lib.rs".to_owned(), SubjectFileV1Origin::Authored),
            ]
        );
        let patch = subjects
            .seed_to_result_patch
            .value
            .as_ref()
            .ok_or("patch present")?;
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(
            &mut sink.open(patch.as_ref()).map_err(|e| format!("{e:?}"))?,
            &mut bytes,
        )?;
        let text = std::str::from_utf8(&bytes)?;
        assert!(
            text.contains("\n+    if !input.bytes().all(|byte| byte.is_ascii_digit()) {\n")
                && text.contains("\n-/// Parse decimal text.\n"),
            "the patch is base -> reference:\n{text}"
        );
        assert_eq!(subjects.collector, shared.collector);
        assert_eq!(subjects.fixtures, shared.fixtures);
        assert_eq!(subjects.oracle, shared.oracle);
        assert_eq!(subjects.harness, shared.harness);
        assert_eq!(subjects.launcher, shared.launcher);
        assert_eq!(subjects.locks, shared.locks);
        assert_eq!(subjects.toolchain, shared.toolchain);
        assert_eq!(subjects.target_features_build_profile, shared.build);
        assert_eq!(subjects.standards, shared.standards);
        assert_eq!(subjects.isolation_profile, shared.isolation);
        Ok(())
    }

    /// R17 round 2 shape C · the per-check plan composes a `Prepared` from the shared part and the
    /// applied snapshot: two attempts differing in every caller field (F129), the identity and the
    /// invocation whole, the result subject and a non-empty patch present, the case's expectation
    /// and review the profile's, the second attempt's parent the first's run; the per-check
    /// registry adds exactly its eight objects; ids that are not distinct and an applied tree
    /// without the editable file are refused by name.
    #[test]
    fn the_per_check_plan_composes_a_prepared_over_the_shared_part()
    -> Result<(), Box<dyn std::error::Error>> {
        let f = fixture()?;
        let staging_root = tree(&f.root, "staging", &[]);
        let staging =
            ArtifactStaging::open(&staging_root, true, deadline()).map_err(|e| format!("{e:?}"))?;
        let cancelled = AtomicBool::new(false);
        let mut sink = Evidence::staged(&staging, deadline());
        let plan_root = f.root.join("shared.plan");
        let shared = shared(
            &mut sink,
            &Inputs {
                tools: &f.tools,
                profile: &f.profile,
                baseline: &f.baseline,
                protected: &f.protected,
                plan_root: &plan_root,
                deadline: deadline(),
                cancelled: &cancelled,
            },
        )
        .map_err(|e| format!("{e:?}"))?;
        let applied_root = tree(&f.root, "applied", &[]);
        let src = tree(&applied_root, "src", &[("lib.rs", REFERENCE)]);
        let _ = src;
        let applied =
            Snapshot::capture(&applied_root, &[], deadline()).map_err(|e| format!("{e:?}"))?;
        let ids = |n: u8| -> [String; 4] {
            std::array::from_fn(|i| format!("28fa0000-0000-4000-8000-0000000000{n}{i}"))
        };
        let mut runs = Vec::new();
        // Three attempts: a first, a second with the first's run as parent, and a third whose
        // previous attempt recorded no receipt (`separate_attempt_same_task`).
        for (generation, n, parent) in [
            ("1", 1_u8, None),
            ("2", 2_u8, Some("28fa0000-0000-4000-8000-000000000011")),
            ("3", 3_u8, None),
        ] {
            let obligation_ids = ids(n + 2);
            let before = sink.registered().len();
            let planned = check(
                &mut sink,
                &CheckInputs {
                    shared: &shared,
                    profile: &f.profile,
                    baseline: &f.baseline,
                    applied: &applied,
                    task: "28fa0000-0000-4000-8000-0000000000a0",
                    attempt: &format!("28fa0000-0000-4000-8000-0000000000b{n}"),
                    run: &format!("28fa0000-0000-4000-8000-00000000001{n}"),
                    generation,
                    parent_run: parent,
                    obligation_ids: &obligation_ids,
                    wall_ms: 250_000 + u64::from(n),
                    deadline: deadline(),
                },
            )
            .map_err(|e| format!("{e:?}"))?;
            assert_eq!(
                planned.added, 8,
                "result subject 3, patch, specification, limits, obligations, contract"
            );
            assert_eq!(sink.registered().len(), before + 8);
            assert_planned(
                &sink,
                &planned,
                &shared,
                &f,
                &Attempt {
                    n,
                    generation,
                    parent,
                    obligation_ids: &obligation_ids,
                    profile_digest: &f.profile.digest,
                },
            )?;
            runs.push(planned.prepared.identity.run_id.as_str().to_owned());
        }
        assert_eq!(
            runs.iter().collect::<std::collections::BTreeSet<_>>().len(),
            3
        );
        assert_check_refused(&mut sink, &shared, &f, &applied)?;
        fs::remove_dir_all(&f.root)?;
        Ok(())
    }

    /// The schema constant hashes to the value t06 pins from the file itself (an independent
    /// source: coreutils `sha256sum schemas/receipts/receipt-v1.schema.json`), and the isolation
    /// profile renders every namespace constant.
    #[test]
    fn the_schema_and_the_isolation_profile_are_the_namespaces_own_constants()
    -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            schema_sha256(),
            "sha256:b926a67e80bc326af582898c3e865fd50618786ef27b71d1fd2f66523776e62b"
        );
        assert!(!SCHEMA.is_empty());
        let value: serde_json::Value =
            serde_json::from_slice(&isolation_profile().map_err(|e| format!("{e:?}"))?)?;
        assert_eq!(value["unshare_all"], true);
        assert_eq!(
            value["fixed_destinations"].as_array().map(Vec::len),
            Some(super::FIXED_DESTINATIONS.len())
        );
        assert_eq!(
            value["environment"].as_array().map(Vec::len),
            Some(super::ENVIRONMENT.len())
        );
        Ok(())
    }

    /// The settle loop's one decision, apart from its I/O (F95): terminal leader and empty group
    /// settle; lost wait ownership refuses whatever the deadline; a live group refuses at the
    /// deadline and waits before it — the deadline itself is the refusing instant.
    #[test]
    fn a_settle_step_settles_refuses_or_waits_by_the_poll_alone() {
        use crate::worker::process::{CleanupPoll, GroupState, WaitOwnership};
        let poll = |ownership, leader_terminal, group| CleanupPoll {
            ownership,
            leader_terminal,
            exit_code: None,
            signal: None,
            group,
        };
        let now = Instant::now();
        let later = now + Duration::from_millis(50);
        assert_eq!(
            settle_step(
                &poll(WaitOwnership::Waitable, true, GroupState::Empty),
                now,
                later
            ),
            SettleStep::Settled
        );
        assert_eq!(
            settle_step(
                &poll(WaitOwnership::Lost, true, GroupState::Empty),
                now,
                later
            ),
            SettleStep::Settled,
            "an empty group under a terminal leader is settled whatever the ownership"
        );
        assert_eq!(
            settle_step(
                &poll(WaitOwnership::Lost, true, GroupState::Live),
                now,
                later
            ),
            SettleStep::Refused
        );
        assert_eq!(
            settle_step(
                &poll(WaitOwnership::Waitable, false, GroupState::Empty),
                later,
                later
            ),
            SettleStep::Refused,
            "the deadline itself refuses"
        );
        assert_eq!(
            settle_step(
                &poll(WaitOwnership::Waitable, true, GroupState::Live),
                now,
                later
            ),
            SettleStep::Wait
        );
        assert_eq!(
            settle_step(
                &poll(WaitOwnership::Waitable, false, GroupState::Unknown),
                now,
                later
            ),
            SettleStep::Wait
        );
    }
}
