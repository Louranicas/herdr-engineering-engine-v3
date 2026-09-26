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
use super::u64_receipt::{RUNTIME_OWNER, environment_rows};
use super::workload::{COMPILE_FLAGS, FIXED_DESTINATIONS, Tools};
use crate::check::collector::{self, Publisher, Sink as _};
use crate::check::u64_oracle::{self, FrozenOracle};
use crate::contracts::receipt::{
    BuildProfileV1, EffectPageV1, EffectV1, EnvironmentPageV1, GrantPageV1, GrantV1,
    LanguageFlagsPageV1, LanguageFlagsV1, List, LockPageV1, Maybe, Name, Payload, Ref, Sha,
    StandardPageV1, StandardV1, SubjectFileV1Origin, SubjectV1, Text, ToolPageV1, ToolV1, TypedRef,
};
use crate::store::Object;
use crate::worker::namespace::{self, NamespaceError, pinned_bytes};
use crate::worker::namespace_shim::ENVIRONMENT;
use crate::worker::process::{self, GroupState, Interruption, ProcessSpec, WaitOwnership};
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
/// The pause between polls while a probe's child is settled: pacing under the caller's deadline.
const SETTLE_PAUSE: std::time::Duration = std::time::Duration::from_millis(10);
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
/// decision 7), and the count of digests this publication added (decision 9).
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
    /// How many registry entries this publication added — receipt-graph nodes with fresh ids. The
    /// store's inventory counts DISTINCT DIGESTS, which the CAS dedupes across dispatches (the
    /// schema, the compiler, the closure), so N4's `S_new` is measured from the store side by
    /// 2c-iii's proof; this is the graph-side count.
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
        which: &'static str,
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
    /// `why` names the site (`run`, `child`, `exit`, `stream`, `utf8`, `release`).
    CompilerVersion {
        why: &'static str,
    },
    /// A value had no rendering the receipt admits.
    Encoding,
}

/// Name the stage a publisher refusal came from.
fn at(stage: &'static str) -> impl Fn(collector::Error) -> Refusal {
    move |error| Refusal::Publish { stage, error }
}

fn name(value: &str) -> Result<Name, Refusal> {
    Name::new(value).map_err(|_| Refusal::Encoding)
}

fn text(value: &str) -> Result<Text, Refusal> {
    Text::new(value).map_err(|_| Refusal::Encoding)
}

fn count(value: usize) -> Result<u32, Refusal> {
    u32::try_from(value).map_err(|_| Refusal::Encoding)
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
            return Err(Refusal::Encoding);
        }
        let record = $page {
            page_index: 0,
            page_count: 1,
            row_count: count(rows.len())?,
            total_rows: count(rows.len())?,
            rows: List::new(rows).map_err(|_| Refusal::Encoding)?,
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
    let read = |which: &'static str, path: &Path, pin: &[u8; 32]| {
        pinned_bytes(path, pin, MAX_TOOL_BYTES, inputs.deadline)
            .map_err(|error| Refusal::Pin { which, error })
    };
    Ok(Pinned {
        compiler: read(
            "compiler",
            &inputs.tools.compiler.host,
            &inputs.tools.compiler.sha256,
        )?,
        shim: read("shim", &inputs.tools.shim.host, &inputs.tools.shim.sha256)?,
        bwrap: read("bwrap", &inputs.tools.bwrap, &namespace::BWRAP_SHA256)?,
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
) -> Result<Pages, Refusal> {
    let schema = payload(sink, SCHEMA, "application/schema+json")?;
    let compiler_payload = payload(sink, compiler, "application/octet-stream")?;
    let version_payload = payload(sink, version_output, "application/octet-stream")?;
    let isolation = payload(sink, &isolation_profile()?, "application/json")?;
    let environment_rows = environment_rows().map_err(|_| Refusal::Encoding)?;
    let (authority, specification) = declared_payloads(sink, inputs)?;
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
                    .ok_or(Refusal::Encoding)?
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
            .map_err(|_| Refusal::Encoding)?,
        }],
        "language flags"
    );
    let build = publisher
        .record(&BuildProfileV1 {
            target: name(BUILD_TARGET)?,
            features: List::new(Vec::new()).map_err(|_| Refusal::Encoding)?,
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

/// The declared grant and effect files, read as declared (digest-bound, F11) and published.
fn declared_payloads(
    sink: &mut Evidence<'_>,
    inputs: &Inputs<'_>,
) -> Result<(Payload, Payload), Refusal> {
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
    Ok((
        payload(sink, &authority, "application/json")?,
        payload(sink, &specification, "application/json")?,
    ))
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
                .map_err(|_| Refusal::Encoding)?,
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
    let oracle = FrozenOracle::from_bytes(protected_file(inputs.protected, "oracle.json")?)
        .map_err(Refusal::Oracle)?;
    let public_inputs = oracle.public_inputs();
    let (version, version_output) = compiler_version(inputs)?;
    let pages = pages(sink, inputs, &pinned.compiler, &version, &version_output)?;
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
    let refuse = |why: &'static str| Refusal::CompilerVersion { why };
    let mut report =
        process::run(&spec, inputs.deadline, inputs.cancelled).map_err(|_| refuse("run"))?;
    // A child or group still live after the report is settled by polling until the deadline (N6):
    // a probe that leaves a process behind is not an observation this dispatch may build on.
    if let Some(pending) = report.pending.as_mut() {
        loop {
            let poll = pending.poll_cleanup(inputs.deadline);
            if poll.leader_terminal && poll.group == GroupState::Empty {
                break;
            }
            if poll.ownership == WaitOwnership::Lost || Instant::now() >= inputs.deadline {
                return Err(refuse("child"));
            }
            // `poll_cleanup` returns at once; a pause between polls keeps this from burning a core
            // for the whole deadline (pacing under the passed-through deadline, not a limit).
            std::thread::sleep(SETTLE_PAUSE);
        }
    }
    // The stream's own interruptions first, then the bytes, then the exit: an over-limit report is
    // refused as a stream, a cancelled or timed-out one as what it was.
    match report.interruption {
        Some(Interruption::Cancelled) => return Err(Refusal::Cancelled),
        Some(Interruption::Timeout) => return Err(Refusal::Deadline),
        Some(Interruption::OutputLimit) => return Err(refuse("stream")),
        Some(_) => return Err(refuse("child")),
        None => {}
    }
    if report.stdout.truncated || !report.stdout.eof {
        return Err(refuse("stream"));
    }
    if report.exit_code != Some(0) {
        return Err(refuse("exit"));
    }
    // The bytes verified before the probe are shown to be the bytes still at the path after it
    // (F5 named: the window between the read and the exec is not closed, it is measured).
    if namespace::sha256(&inputs.tools.compiler.host, inputs.deadline).map_err(|error| {
        Refusal::Pin {
            which: "compiler",
            error,
        }
    })? != inputs.tools.compiler.sha256
    {
        return Err(Refusal::Pin {
            which: "compiler",
            error: NamespaceError::Digest,
        });
    }
    let output = std::str::from_utf8(&report.stdout.bytes).map_err(|_| refuse("utf8"))?;
    let release = output
        .lines()
        .find_map(|line| line.strip_prefix("release: "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(refuse("release"))?;
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
    serde_json::to_vec(&value).map_err(|_| Refusal::Encoding)
}

/// The digest the receipt's `schema_sha256` names: the schema bytes this binary carries.
#[must_use]
pub fn schema_sha256() -> String {
    evidence::digest(SCHEMA)
}

#[cfg(test)]
mod tests {
    use super::{
        COLLECTOR_UNREAD, Inputs, Refusal, SCHEMA, SCHEMA_STANDARD, isolation_profile,
        schema_sha256, shared,
    };
    use crate::app::class_profile::{Profile, compose};
    use crate::app::evidence::Evidence;
    use crate::app::workload::{COMPILER_DESTINATION, Tools};
    use crate::check::graph::{Graph, Objects as _};
    use crate::check::u64_oracle::FrozenOracle;
    use crate::contracts::receipt::{
        BuildProfileV1, EffectPageV1, GrantPageV1, LanguageFlagsPageV1, LockPageV1, Ref,
        StandardPageV1, SubjectFilePageV1, SubjectFileV1Origin, SubjectV1, ToolPageV1, decode,
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
    const AUTHORITY: &[u8] = b"{\"authority\":\"WL-U64 fixed workload\",\"issuer\":\"operator\"}\n";
    const SPECIFICATION: &[u8] = b"{\"isolation\":\"bwrap --unshare-all; no network\"}\n";
    const PAYLOADS: usize = 6;
    const PAGES: usize = 8;
    const SUBJECT_OBJECTS: usize = 3 * 4 + 4 + 2;
    const CLOSURE_NODES: usize = 22;
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
        let base = tree(&class, "base", &[("lib.rs", BASE)]);
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
        Ok(Fixture {
            profile: Profile {
                declared,
                directory: class,
                digest: String::new(),
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
                crate::app::evidence::digest(AUTHORITY).as_str(),
                crate::app::evidence::digest(AUTHORITY).as_str()
            )
        );
        let effects: EffectPageV1 = resolved(sink, &shared.effects);
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
                crate::app::evidence::digest(SPECIFICATION).as_str()
            )
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
        assert_eq!(shared.objects.len(), sink.registered().len());
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
                vec![("lib.rs".to_owned(), SubjectFileV1Origin::Authored)],
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
                which: "compiler",
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
        // A declared file whose bytes are not the declaration's: refused by name, then restored.
        write(&f.profile.directory.join("authority.json"), SPECIFICATION);
        assert!(matches!(
            shared(sink, &base),
            Err(Refusal::Declared {
                which: "authority",
                error: crate::app::class_profile::ReviewedError::Mismatch
            })
        ));
        write(&f.profile.directory.join("authority.json"), AUTHORITY);
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
            (&b"#!/bin/sh\nexit 3\n"[..], "exit"),
            (
                &b"#!/bin/sh\nprintf 'rustc 0.0.0\\nrelease:  \\n'\n"[..],
                "release",
            ),
        ] {
            let stand_in = f.root.join(format!("compiler-{why}"));
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
                "{why}: {refused:?}"
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
        println!(
            "S_new={s_new} registry entries per dispatch (graph side); per_dispatch_after_first={}; \
             store-side distinct digests are measured by 2c-iii's proof (N4)",
            s_new - CLOSURE_NODES
        );
        assert_refused(&f, &staging, &cancelled)?;
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
}
