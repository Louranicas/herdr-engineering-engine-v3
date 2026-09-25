//! Compose the fixed offline workload with frozen source and the namespace owner.
//! Raw reports survive every post-launch failure; this layer never accepts a task.

use crate::check::u64_oracle::{Evaluation, FrozenOracle, OutputError};
use crate::worker::{
    namespace::{
        BwrapPlan, NamespaceReport, NamespaceRunError, NativeStatus, PublicFile, ReadOnlyFile,
        run_bounded_namespace, run_namespace,
    },
    process::Interruption,
    resources::Scope,
    workspace::{self, Content, Snapshot},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{self, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// The namespace destinations the workload itself mounts, every one fixed by the workload: the
/// compiler it runs, the source it compiles, the link stage's wrapper and library, the linked
/// driver and the oracle's public inputs. The class profile refuses a runtime file at, above or
/// among these (B14-P2b), so this list is the one it reads, never a copy of it.
pub const COMPILER_DESTINATION: &str = "/toolchain/bin/rustc";
pub const SOURCE_DESTINATION: &str = "/frozen/source/lib.rs";
pub const WRAPPER_DESTINATION: &str = "/frozen/public-wrapper.rs";
pub const LIBRARY_DESTINATION: &str = "/frozen/libstrict_u64_workload.rlib";
pub const DRIVER_DESTINATION: &str = "/frozen/bin/workload-driver";
pub const INPUTS_DESTINATION: &str = "/frozen/inputs.hex";
pub const FIXED_DESTINATIONS: [&str; 6] = [
    COMPILER_DESTINATION,
    SOURCE_DESTINATION,
    WRAPPER_DESTINATION,
    LIBRARY_DESTINATION,
    DRIVER_DESTINATION,
    INPUTS_DESTINATION,
];

/// Trusted application configuration. Every file is separately pinned and mounted.
pub struct Tools {
    pub bwrap: PathBuf,
    pub compiler: ReadOnlyFile,
    pub shim: ReadOnlyFile,
    pub runtime_files: Vec<ReadOnlyFile>,
    pub namespace_directories: Vec<PathBuf>,
}

pub struct Plan<'a> {
    pub source: &'a Snapshot,
    /// Contains oracle.json and public-wrapper.rs; the oracle is never mounted.
    pub protected: &'a Snapshot,
    /// Fresh empty canonical private directory, owned by this invocation.
    pub job_root: &'a Path,
    pub tools: &'a Tools,
    pub deadline: Instant,
    pub cancelled: &'a AtomicBool,
}

#[derive(Debug)]
pub enum Outcome {
    Matched(Evaluation),
    Mismatch(Evaluation),
    InvalidOutput(OutputError),
    InvalidSubject,
    ProducerFailed,
    ProducerError,
    LauncherFailed,
    Timeout,
    Cancelled,
    PendingCleanup,
    SetupFailed,
}

pub enum Step {
    Completed {
        label: &'static str,
        report: Box<NamespaceReport>,
    },
    Refused {
        label: &'static str,
        error: NamespaceRunError,
    },
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "distinct process, scratch, source and cancellation observations"
)]
pub struct Run {
    pub outcome: Outcome,
    pub steps: Vec<Step>,
    /// Immutable captured outputs, not a claim that their producing process passed.
    pub outputs: Vec<Snapshot>,
    /// Invocation-owned paths remain for import and explicit later cleanup.
    pub retained_paths: Vec<PathBuf>,
    /// Process and FIFO settlement is independent of verdict and retained disk artifacts.
    pub process_cleanup_complete: bool,
    pub subjects_unchanged: bool,
    pub cancellation_observed: bool,
    /// Retained tmpfs descriptors are separate from terminated process custody.
    pub scratch_released: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Subject(workspace::Error),
    Oracle,
    Layout,
    Io,
    Deadline,
}

/// Compile the exact frozen source, link the fixed wrapper, and compare outputs.
/// Every stage runs in a separate namespace with one original caller deadline.
/// Returned Matched means finite output agreement, never module/task acceptance.
///
/// # Errors
/// Refuses invalid preflight subjects, oracle bytes, layout or expired time before
/// candidate launch. Post-launch failures retain raw reports in the returned Run.
pub fn collect(plan: &Plan<'_>) -> Result<Run, Error> {
    collect_inner(plan, None)
}

/// Execute the same fixed workload with three fresh scopes under one owned aggregate.
/// The caller retains aggregate lifecycle/accounting ownership.
/// # Errors
/// Refuses duplicate scope names, different aggregates and ordinary workload preflight errors.
pub fn collect_bounded(plan: &Plan<'_>, scopes: &[Scope; 3]) -> Result<Run, Error> {
    let units = scopes
        .iter()
        .map(Scope::unit)
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|_| Error::Layout)?;
    if units.len() != 3
        || scopes
            .iter()
            .any(|scope| scope.aggregate != scopes[0].aggregate)
    {
        return Err(Error::Layout);
    }
    collect_inner(plan, Some(scopes))
}

fn collect_inner(plan: &Plan<'_>, scopes: Option<&[Scope; 3]>) -> Result<Run, Error> {
    preflight(plan)?;
    let oracle = FrozenOracle::from_bytes(file(plan.protected, "oracle.json")?)
        .map_err(|_| Error::Oracle)?;
    let public = plan.job_root.join("public");
    let mut run = Run {
        outcome: Outcome::SetupFailed,
        steps: Vec::new(),
        outputs: Vec::new(),
        retained_paths: vec![public.clone()],
        process_cleanup_complete: true,
        subjects_unchanged: true,
        cancellation_observed: false,
        scratch_released: true,
    };
    let result = (|| {
        private_directory(&public)?;
        write_new(&public.join("inputs.hex"), &oracle.public_inputs())?;
        execute_stages(plan, scopes, &oracle, &public, &mut run)
    })();
    if let Err(error) = result {
        run.outcome = match error {
            Error::Subject(_) => Outcome::InvalidSubject,
            Error::Deadline => Outcome::Timeout,
            _ => Outcome::SetupFailed,
        };
    }
    run.subjects_unchanged = !(plan.source.readback_source(plan.deadline).is_err()
        || plan.protected.readback_source(plan.deadline).is_err());
    run.cancellation_observed |= plan.cancelled.load(Ordering::Acquire);
    if !run.process_cleanup_complete || !run.scratch_released {
        run.outcome = Outcome::PendingCleanup;
    } else if run.cancellation_observed {
        run.outcome = Outcome::Cancelled;
    } else if Instant::now() >= plan.deadline {
        run.outcome = Outcome::Timeout;
    } else if !run.subjects_unchanged {
        run.outcome = Outcome::InvalidSubject;
    }
    Ok(run)
}

fn preflight(plan: &Plan<'_>) -> Result<(), Error> {
    if Instant::now() >= plan.deadline {
        return Err(Error::Deadline);
    }
    plan.source
        .readback_source(plan.deadline)
        .map_err(Error::Subject)?;
    plan.protected
        .readback_source(plan.deadline)
        .map_err(Error::Subject)?;
    let job = Snapshot::capture(plan.job_root, &[], plan.deadline).map_err(Error::Subject)?;
    if job.entries().next().is_some()
        || plan.job_root.starts_with(plan.source.root())
        || plan.job_root.starts_with(plan.protected.root())
        || plan.source.root().starts_with(plan.job_root)
        || plan.protected.root().starts_with(plan.job_root)
        || plan.tools.compiler.namespace != Path::new(COMPILER_DESTINATION)
    {
        return Err(Error::Layout);
    }
    file(plan.source, "src/lib.rs")?;
    file(plan.protected, "public-wrapper.rs")?;
    Ok(())
}

fn execute_stages(
    plan: &Plan<'_>,
    scopes: Option<&[Scope; 3]>,
    oracle: &FrozenOracle,
    public: &Path,
    run: &mut Run,
) -> Result<(), Error> {
    let source = binding(plan.source, "src/lib.rs", SOURCE_DESTINATION)?;
    let lib_args = [
        "--sysroot",
        "/toolchain",
        "--edition=2024",
        "--crate-name",
        "strict_u64_workload",
        "--crate-type",
        "rlib",
        "-Dwarnings",
        SOURCE_DESTINATION,
        "-o",
        "/work/libstrict_u64_workload.rlib",
    ];
    if !stage(
        plan,
        run,
        "compile-library",
        plan.tools.compiler.clone(),
        vec![source],
        &lib_args,
        scopes.map(|value| &value[0]),
    )? {
        return Ok(());
    }
    let library = freeze_output(
        plan,
        run,
        "compile-library",
        "frozen-library",
        scopes.is_some(),
    )?;
    let wrapper = binding(plan.protected, "public-wrapper.rs", WRAPPER_DESTINATION)?;
    let rlib = binding(&library, "libstrict_u64_workload.rlib", LIBRARY_DESTINATION)?;
    let library_argument = format!("strict_u64_workload={LIBRARY_DESTINATION}");
    let link_args = [
        "--sysroot",
        "/toolchain",
        "--edition=2024",
        "--crate-name",
        "workload_driver",
        "-Dwarnings",
        "-C",
        "linker=/usr/bin/gcc",
        "--extern",
        library_argument.as_str(),
        WRAPPER_DESTINATION,
        "-o",
        "/work/workload-driver",
    ];
    run.outputs.push(library);
    if !stage(
        plan,
        run,
        "link-driver",
        plan.tools.compiler.clone(),
        vec![wrapper, rlib],
        &link_args,
        scopes.map(|value| &value[1]),
    )? {
        return Ok(());
    }
    let driver = freeze_output(plan, run, "link-driver", "frozen-driver", scopes.is_some())?;
    let executable = binding(&driver, "workload-driver", DRIVER_DESTINATION)?;
    run.outputs.push(driver);
    let projection = ReadOnlyFile {
        host: public.join("inputs.hex"),
        namespace: INPUTS_DESTINATION.into(),
        sha256: Sha256::digest(oracle.public_inputs()).into(),
    };
    if !stage(
        plan,
        run,
        "execute-driver",
        executable,
        vec![projection],
        &[],
        scopes.map(|value| &value[2]),
    )? {
        return Ok(());
    }
    evaluate(plan, run, oracle)
}

fn evaluate(plan: &Plan<'_>, run: &mut Run, oracle: &FrozenOracle) -> Result<(), Error> {
    let Some(Step::Completed { report, .. }) = run.steps.last() else {
        return Err(Error::Layout);
    };
    if plan.cancelled.load(Ordering::Acquire) {
        run.cancellation_observed = true;
        run.outcome = Outcome::Cancelled;
        return Ok(());
    }
    run.outcome = match oracle.evaluate(&report.observer.candidate_stdout.bytes) {
        Ok(value) if value.failed == 0 => Outcome::Matched(value),
        Ok(value) => Outcome::Mismatch(value),
        Err(error) => Outcome::InvalidOutput(error),
    };
    Ok(())
}

fn stage(
    plan: &Plan<'_>,
    run: &mut Run,
    label: &'static str,
    candidate: ReadOnlyFile,
    extra: Vec<ReadOnlyFile>,
    arguments: &[&str],
    scope: Option<&Scope>,
) -> Result<bool, Error> {
    if Instant::now() >= plan.deadline {
        return Err(Error::Deadline);
    }
    if plan.cancelled.load(Ordering::Acquire) {
        run.cancellation_observed = true;
        run.outcome = Outcome::Cancelled;
        return Ok(false);
    }
    let work = plan.job_root.join(if scope.is_some() {
        format!("{label}-unmounted")
    } else {
        label.into()
    });
    run.retained_paths.push(work.clone());
    private_directory(&work)?;
    for name in ["home", "target", "julia-depot"] {
        private_directory(&work.join(name))?;
    }
    let channels = plan.job_root.join(format!("{label}-channels"));
    run.retained_paths.push(channels.clone());
    let mut files = plan.tools.runtime_files.clone();
    files.extend(extra);
    let public_files = files
        .iter()
        .chain([&plan.tools.shim, &candidate])
        .map(|entry| PublicFile {
            namespace: entry.namespace.clone(),
            sha256: entry.sha256,
        })
        .collect();
    let mut directories: BTreeSet<PathBuf> =
        plan.tools.namespace_directories.iter().cloned().collect();
    if scope.is_some() {
        // Bounded scratch has only the fixed runtime directories. The finite
        // toolchain closure's legacy /work/bin was hidden by the old host bind.
        directories.retain(|path| !path.starts_with("/work"));
    }
    for file in files.iter().chain([&plan.tools.shim, &candidate]) {
        for parent in file.namespace.ancestors().skip(1) {
            if parent != Path::new("/") {
                directories.insert(parent.to_path_buf());
            }
        }
    }
    let mut shim_arguments: Vec<OsString> = ["__namespace-exec", "/work", "--"]
        .into_iter()
        .map(OsString::from)
        .collect();
    shim_arguments.push(candidate.namespace.as_os_str().to_owned());
    shim_arguments.extend(arguments.iter().map(OsString::from));
    let namespace = BwrapPlan {
        bwrap: plan.tools.bwrap.clone(),
        shim: plan.tools.shim.clone(),
        candidate,
        read_only_files: files,
        namespace_directories: directories.into_iter().collect(),
        work_host: work,
        channels_host: channels,
        public_files,
        protected_paths: vec![
            plan.source.root().to_path_buf(),
            plan.protected.root().to_path_buf(),
            plan.job_root.to_path_buf(),
        ],
        shim_arguments,
    };
    let result = match scope {
        Some(scope) => {
            run_bounded_namespace(namespace, scope.clone(), plan.deadline, plan.cancelled)
        }
        None => run_namespace(namespace, plan.deadline, plan.cancelled),
    };
    match result {
        Ok(report) => complete_stage(plan, run, label, report, scope.is_some()),
        Err(error) => {
            run.process_cleanup_complete &= match &error {
                NamespaceRunError::Prepare(failure) => failure.cleanup_complete,
                NamespaceRunError::Process {
                    channel_cleanup_complete,
                    ..
                } => *channel_cleanup_complete,
            };
            run.outcome = Outcome::LauncherFailed;
            run.steps.push(Step::Refused { label, error });
            Ok(false)
        }
    }
}

fn complete_stage(
    plan: &Plan<'_>,
    run: &mut Run,
    label: &'static str,
    mut report: NamespaceReport,
    bounded: bool,
) -> Result<bool, Error> {
    let clean = report.cleanup_channels();
    run.process_cleanup_complete &= clean && report.process.pending.is_none();
    run.cancellation_observed |= report.process.interruption == Some(Interruption::Cancelled);
    let mut success = classify(&report, clean, &mut run.outcome);
    if bounded && report.observer.scratch_retained() {
        match report.export_scratch(
            plan.job_root,
            label,
            &plan.source.source_identities(),
            plan.deadline,
        ) {
            Ok(output) => {
                run.retained_paths.push(output.path);
                if !report.release_scratch() {
                    run.outcome = Outcome::PendingCleanup;
                    success = false;
                }
            }
            Err(error) => {
                if let Some(path) = error.partial_path {
                    run.retained_paths.push(path);
                }
                run.outcome = Outcome::PendingCleanup;
                success = false;
            }
        }
    }
    run.scratch_released &= !report.observer.scratch_retained();
    run.steps.push(Step::Completed {
        label,
        report: Box::new(report),
    });
    if success {
        capture_output(plan, label, bounded)?;
    }
    Ok(success)
}

fn classify(report: &NamespaceReport, clean: bool, outcome: &mut Outcome) -> bool {
    let process = &report.process;
    *outcome = match process.interruption {
        _ if !clean || process.pending.is_some() => Outcome::PendingCleanup,
        Some(Interruption::Cancelled) => Outcome::Cancelled,
        Some(Interruption::Timeout) => Outcome::Timeout,
        _ if !report.postrun_sources_match => Outcome::InvalidSubject,
        _ if process.interruption.is_some()
            || !report.json_exit_matches
            || report.observer.failure().is_some() =>
        {
            Outcome::LauncherFailed
        }
        _ if !report.observer.facts.native_status_consistent => Outcome::ProducerError,
        _ if matches!(
            report.observer.facts.native_status,
            Some(NativeStatus::Signal { .. }) | None
        ) =>
        {
            Outcome::ProducerError
        }
        _ if !matches!(
            report.observer.facts.native_status,
            Some(NativeStatus::Exit { code: 0, .. })
        ) =>
        {
            Outcome::ProducerFailed
        }
        // This fixed baseline requires empty compiler/wrapper diagnostics. Raw
        // nonempty bytes are retained; they are never parsed away as benign noise.
        _ if !report.observer.candidate_stderr.bytes.is_empty() => Outcome::ProducerFailed,
        _ => return true,
    };
    false
}

fn freeze_output(
    plan: &Plan<'_>,
    run: &mut Run,
    source: &str,
    target: &str,
    bounded: bool,
) -> Result<Snapshot, Error> {
    let captured = capture_output(plan, source, bounded)?;
    run.retained_paths.push(plan.job_root.join(target));
    let frozen = captured
        .materialize(plan.job_root, target, &[], plan.deadline)
        .map_err(|_| Error::Io)?;
    Snapshot::capture(&frozen.path, &captured.source_identities(), plan.deadline)
        .map_err(Error::Subject)
}

fn capture_output(plan: &Plan<'_>, stage: &str, bounded: bool) -> Result<Snapshot, Error> {
    let expected = match stage {
        "compile-library" => Some(("libstrict_u64_workload.rlib", false)),
        "link-driver" => Some(("workload-driver", true)),
        "execute-driver" => None,
        _ => return Err(Error::Layout),
    };
    let captured = Snapshot::capture(
        &plan.job_root.join(stage),
        &plan.source.source_identities(),
        plan.deadline,
    )
    .map_err(Error::Subject)?;
    let expected_directories: &[&str] = if bounded {
        &["home", "target", "julia-depot", "tmp"]
    } else {
        &["home", "target", "julia-depot"]
    };
    let mut directories = BTreeSet::new();
    let mut output = false;
    for entry in captured.entries() {
        match &entry.content {
            Content::Directory if expected_directories.contains(&entry.path.as_str()) => {
                directories.insert(entry.path.as_str());
            }
            Content::File {
                bytes, executable, ..
            } if expected == Some((entry.path.as_str(), *executable)) && !bytes.is_empty() => {
                output = true;
            }
            _ => return Err(Error::Subject(workspace::Error::Policy)),
        }
    }
    if directories.len() != expected_directories.len() || output != expected.is_some() {
        return Err(Error::Subject(workspace::Error::Policy));
    }
    Ok(captured)
}

fn file<'a>(snapshot: &'a Snapshot, name: &str) -> Result<&'a [u8], Error> {
    snapshot
        .entries()
        .find_map(|entry| match &entry.content {
            Content::File { bytes, .. } if entry.path == name => Some(bytes.as_slice()),
            _ => None,
        })
        .ok_or(Error::Layout)
}

fn binding(snapshot: &Snapshot, name: &str, namespace: &str) -> Result<ReadOnlyFile, Error> {
    Ok(ReadOnlyFile {
        host: snapshot.root().join(name),
        namespace: namespace.into(),
        sha256: Sha256::digest(file(snapshot, name)?).into(),
    })
}

fn private_directory(path: &Path) -> Result<(), Error> {
    fs::create_dir(path).map_err(|_| Error::Io)?;
    fs::set_permissions(path, Permissions::from_mode(0o700)).map_err(|_| Error::Io)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| Error::Io)?;
    file.write_all(bytes).map_err(|_| Error::Io)?;
    file.set_permissions(Permissions::from_mode(0o400))
        .map_err(|_| Error::Io)?;
    file.sync_all().map_err(|_| Error::Io)
}
