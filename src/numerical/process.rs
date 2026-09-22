//! Trusted local Julia composition. This TH-DEV adapter grants no task authority
//! and does not provide namespace, memory or credential isolation.
use super::{Dataset, Invalid, MAX_REPORT, Report};
use crate::worker::process::{self as worker, Interruption, ProcessReport, ProcessSpec, Refusal};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FILES: [&str; 6] = [
    "Project.toml",
    "Manifest.toml",
    "src/HabitatAnalysis.jl",
    "src/Evaluate.jl",
    "src/Cohesion.jl",
    "bin/analysis.jl",
];
const CLEANUP_RESERVE: Duration = Duration::from_secs(10);

/// Protected caller configuration, never decoded from a request or child report.
/// Pins must originate from the admitted package owner. This value is no grant.
#[derive(Debug)]
pub struct JuliaProfile {
    pub executable: PathBuf,
    pub executable_sha256: String,
    pub project: PathBuf,
    pub project_files: BTreeMap<String, String>,
    pub scratch: PathBuf,
    pub dependency_depot: PathBuf,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Failure {
    Profile,
    Io(Option<i32>),
    Clock,
    Deadline,
    Cancelled,
    Input(Invalid),
    Launch(Refusal),
    Interrupted(Interruption),
    Unsettled,
    Nonzero,
    Diagnostic,
    Response(Invalid),
}
/// Every started child returns its original custody, raw streams and signals,
/// including `PendingChild` on unknown cleanup. Dropping it is not settlement.
#[derive(Debug)]
pub struct Exchange {
    pub outcome: Result<Report, Failure>,
    pub process: Option<ProcessReport>,
    pub postflight: Option<Failure>,
    pub wall_elapsed: Duration,
    /// This process owner measures wall time, not CPU usage; unknown stays unknown.
    pub cpu_elapsed: Option<Duration>,
}

/// Run one immutable dataset using the existing sole child/pipe owner. The caller
/// retains the original clock and supplies at most60s including10s cleanup reserve.
/// Exact package pins are checked before/after; same-UID concurrent hostile writes
/// are outside this development profile and require the worker isolation profile.
#[must_use]
pub fn analyze(
    profile: &JuliaProfile,
    dataset: &Dataset,
    start: Instant,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Exchange {
    let mut report = None;
    let mut postflight = None;
    let outcome = (|| {
        window(start, deadline, cancelled)?;
        let work_deadline = deadline
            .checked_sub(CLEANUP_RESERVE)
            .ok_or(Failure::Deadline)?;
        if work_deadline <= Instant::now() {
            return Err(Failure::Deadline);
        }
        validate_profile(profile, work_deadline, cancelled)?;
        Dataset::decode(dataset.raw(), unix_ms()?).map_err(Failure::Input)?;
        let spec = specification(profile, dataset);
        let observed = worker::run(&spec, work_deadline, cancelled).map_err(Failure::Launch)?;
        report = Some(observed);
        let observed = report.as_ref().ok_or(Failure::Unsettled)?;
        let result = classify(dataset, observed, cancelled);
        postflight = validate_profile(profile, deadline, cancelled).err();
        match result {
            Err(error) => Err(error),
            Ok(value) => {
                if let Some(error) = postflight {
                    return Err(error);
                }
                window(start, deadline, cancelled)?;
                Ok(value)
            }
        }
    })();
    Exchange {
        outcome,
        process: report,
        postflight,
        wall_elapsed: start.elapsed(),
        cpu_elapsed: None,
    }
}
fn classify(
    dataset: &Dataset,
    observed: &ProcessReport,
    cancelled: &AtomicBool,
) -> Result<Report, Failure> {
    if let Some(reason) = observed.interruption {
        return Err(Failure::Interrupted(reason));
    }
    if !observed.leader_reaped
        || !observed.process_group_settled
        || observed.pending.is_some()
        || !observed.stdout.eof
        || !observed.stderr.eof
        || observed.stdout.failed
        || observed.stderr.failed
        || observed.stdout.truncated
        || observed.stderr.truncated
    {
        return Err(Failure::Unsettled);
    }
    if observed.exit_code != Some(0) || observed.signal.is_some() {
        return Err(Failure::Nonzero);
    }
    if !observed.stderr.bytes.is_empty() {
        return Err(Failure::Diagnostic);
    }
    if cancelled.load(Ordering::Acquire) {
        return Err(Failure::Cancelled);
    }
    dataset
        .report(&observed.stdout.bytes, unix_ms()?)
        .map_err(Failure::Response)
}
fn window(start: Instant, deadline: Instant, cancelled: &AtomicBool) -> Result<(), Failure> {
    if cancelled.load(Ordering::Acquire) {
        return Err(Failure::Cancelled);
    }
    let now = Instant::now();
    if start > now
        || deadline <= now
        || deadline
            .checked_duration_since(start)
            .is_none_or(|d| d > Duration::from_secs(60))
    {
        return Err(Failure::Deadline);
    }
    Ok(())
}
fn unix_ms() -> Result<u64, Failure> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Failure::Clock)?
        .as_millis()
        .try_into()
        .map_err(|_| Failure::Clock)
}
fn specification(profile: &JuliaProfile, dataset: &Dataset) -> ProcessSpec {
    let args = vec![
        OsString::from("--startup-file=no"),
        OsString::from(format!("--project={}", profile.project.display())),
        OsString::from("--check-bounds=yes"),
        OsString::from("--depwarn=error"),
        OsString::from("--threads=1"),
        OsString::from("--compiled-modules=no"),
        profile.project.join("bin/analysis.jl").into_os_string(),
    ];
    let environment = vec![
        ("LC_ALL", "C"),
        ("LANG", "C"),
        ("JULIA_LOAD_PATH", "@:@stdlib"),
        ("JULIA_PKG_OFFLINE", "true"),
        ("JULIA_PKG_PRECOMPILE_AUTO", "0"),
        ("JULIA_NUM_THREADS", "1"),
        ("OPENBLAS_NUM_THREADS", "1"),
    ];
    let mut environment: Vec<(OsString, OsString)> = environment
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
    environment.push((
        "JULIA_DEPOT_PATH".into(),
        format!(
            "{}:{}",
            profile.scratch.display(),
            profile.dependency_depot.display()
        )
        .into(),
    ));
    environment.push(("HOME".into(), profile.scratch.clone().into_os_string()));
    environment.push(("TMPDIR".into(), profile.scratch.clone().into_os_string()));
    ProcessSpec {
        executable: profile.executable.clone(),
        arguments: args,
        directory: profile.scratch.clone(),
        environment,
        input: dataset.raw().to_vec(),
        stream_limit: MAX_REPORT,
    }
}
fn validate_profile(
    profile: &JuliaProfile,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<(), Failure> {
    for path in [
        &profile.project,
        &profile.scratch,
        &profile.dependency_depot,
    ] {
        if !path.is_absolute()
            || path.canonicalize().map_err(Failure::from)? != *path
            || !path.is_dir()
            || path.to_str().is_none_or(|s| s.contains(':'))
        {
            return Err(Failure::Profile);
        }
    }
    // Julia 1.12 selects these before the pinned conventional files, or loads
    // their unpinned preferences. Refuse aliases rather than silently executing
    // a different project/manifest configuration.
    for name in [
        "JuliaProject.toml",
        "JuliaManifest-v1.12.toml",
        "Manifest-v1.12.toml",
        "JuliaManifest.toml",
        "JuliaLocalPreferences.toml",
        "LocalPreferences.toml",
    ] {
        match fs::symlink_metadata(profile.project.join(name)) {
            Ok(_) => return Err(Failure::Profile),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(Failure::from(error)),
        }
    }
    if fs::metadata(&profile.scratch)
        .map_err(Failure::from)?
        .permissions()
        .mode()
        & 0o777
        != 0o700
        || profile.project_files.len() != FILES.len()
    {
        return Err(Failure::Profile);
    }
    pinned(
        &profile.executable,
        &profile.executable_sha256,
        256 * 1024 * 1024,
        deadline,
        cancelled,
    )?;
    for name in FILES {
        let expected = profile.project_files.get(name).ok_or(Failure::Profile)?;
        pinned(
            &profile.project.join(name),
            expected,
            1024 * 1024,
            deadline,
            cancelled,
        )?;
    }
    Ok(())
}
fn pinned(
    path: &Path,
    expected: &str,
    cap: u64,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<(), Failure> {
    if !path.is_absolute() || path.canonicalize().map_err(Failure::from)? != path {
        return Err(Failure::Profile);
    }
    let metadata = fs::symlink_metadata(path).map_err(Failure::from)?;
    if !metadata.is_file() || metadata.len() > cap {
        return Err(Failure::Profile);
    }
    let mut file = File::open(path).map_err(Failure::from)?;
    let mut hash = Sha256::new();
    let mut total = 0_u64;
    loop {
        if cancelled.load(Ordering::Acquire) {
            return Err(Failure::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(Failure::Deadline);
        }
        let mut chunk = [0_u8; 16_384];
        let n = file.read(&mut chunk).map_err(Failure::from)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > cap {
            return Err(Failure::Profile);
        }
        hash.update(&chunk[..n]);
    }
    let mut actual = String::from("sha256:");
    for b in hash.finalize() {
        actual.push(char::from(super::HEX[usize::from(b >> 4)]));
        actual.push(char::from(super::HEX[usize::from(b & 15)]));
    }
    if total != metadata.len() || actual != expected {
        return Err(Failure::Profile);
    }
    Ok(())
}
impl From<std::io::Error> for Failure {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.raw_os_error())
    }
}
