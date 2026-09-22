//! Local development one-shot composition; no service grant or lifecycle owner.
use super::{
    Class, Facts, ProbeError, ProbeObservation, ProbePreparation, Profile, Recorded, UsefulResult,
};
use crate::contracts::UuidV4;
use crate::store::{Principal, Store};
use crate::worker::process::{self as worker, Interruption, ProcessReport, ProcessSpec, Refusal};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
const MAX_STREAM: usize = 65_536;

/// Protected service-owner input, never request data or an execution grant.
/// Literal arguments and expected bytes belong to the reviewed useful recipe.
/// This development slice supports only the three finite local fixture binaries:
/// /usr/bin/printf, /usr/bin/false and /usr/bin/sleep. It is not a general executor.
#[derive(Clone, Debug)]
pub struct LocalRecipe {
    pub executable: PathBuf,
    pub executable_sha256: String,
    pub arguments: Vec<OsString>,
    pub expected_stdout: Vec<u8>,
    pub directory: PathBuf,
}
#[derive(Debug)]
pub enum LocalProbeError {
    Preparation(ProbeError),
    Recipe,
    Io(Option<i32>),
    Launch(Refusal),
    Interrupted(Interruption),
    Unsettled,
    Deadline,
    Cancelled,
    NotReady,
    AlreadyRecorded,
    Publication(super::Error),
}
/// Original process custody remains present after interpretation/publication error.
/// This value is neither durable admission nor proof that a public grant existed.
#[derive(Debug)]
pub struct LocalProbe {
    preparation: ProbePreparation,
    pub recipe: LocalRecipe,
    pub process: Option<ProcessReport>,
    pub facts: Option<ProbeObservation>,
    pub error: Option<LocalProbeError>,
    pub postflight_error: Option<LocalProbeError>,
    pub wall_elapsed: Duration,
    pub cpu_elapsed: Option<Duration>,
    recorded: Option<Recorded>,
}
impl LocalProbe {
    /// Bind the actual collected return to the same admitted subject and original
    /// clock before publishing through the existing sole Store owner. The caller
    /// retains this whole value even if publication fails. No deadline is renewed.
    /// # Errors
    /// Refuses failed/incomplete execution, cancelled/expired/stale preparations,
    /// repeated publication through this value and real Store publication failures.
    pub fn record(
        &mut self,
        store: &mut Store,
        principal: &Principal,
        profiles: &[Profile],
        cancelled: &AtomicBool,
    ) -> Result<&Recorded, LocalProbeError> {
        if self.recorded.is_some() {
            return Err(LocalProbeError::AlreadyRecorded);
        }
        if self.error.is_some() || self.postflight_error.is_some() {
            return Err(LocalProbeError::NotReady);
        }
        self.preparation
            .revalidate(store, principal, profiles, cancelled)
            .map_err(LocalProbeError::Preparation)?;
        let facts = self.facts.as_ref().ok_or(LocalProbeError::NotReady)?;
        self.recorded = Some(
            super::record_probe(
                store,
                principal,
                self.preparation.profile(),
                facts,
                self.preparation.original_deadline(),
            )
            .map_err(LocalProbeError::Publication)?,
        );
        self.recorded.as_ref().ok_or(LocalProbeError::NotReady)
    }
    #[must_use]
    pub const fn recorded(&self) -> Option<&Recorded> {
        self.recorded.as_ref()
    }
}

/// Run one local one-shot from protected configuration using the existing worker
/// owner. Ordinary fixture scope only: no network/memory/namespace containment is
/// established. Hashes before/after do not stop hostile same-UID source mutation.
/// Actual CPU is unknown; the usage observation is measured worker wall time.
#[must_use]
pub fn run_local_probe(
    store: &Store,
    principal: &Principal,
    preparation: ProbePreparation,
    profiles: &[Profile],
    recipe: &LocalRecipe,
    cancelled: &AtomicBool,
    evidence_id: UuidV4<'_>,
) -> LocalProbe {
    let start = preparation.original_start();
    let mut result = LocalProbe {
        preparation,
        recipe: recipe.clone(),
        process: None,
        facts: None,
        error: None,
        postflight_error: None,
        wall_elapsed: Duration::ZERO,
        cpu_elapsed: None,
        recorded: None,
    };
    let outcome = (|| {
        result
            .preparation
            .revalidate(store, principal, profiles, cancelled)
            .map_err(LocalProbeError::Preparation)?;
        let deadline = result.preparation.original_deadline();
        let work = deadline
            .checked_sub(Duration::from_secs(10))
            .filter(|time| *time > Instant::now())
            .ok_or(LocalProbeError::Deadline)?;
        validate(recipe, result.preparation.profile(), work, cancelled)?;
        let spec = ProcessSpec {
            executable: recipe.executable.clone(),
            arguments: recipe.arguments.clone(),
            directory: recipe.directory.clone(),
            environment: vec![("LC_ALL".into(), "C".into())],
            input: Vec::new(),
            stream_limit: MAX_STREAM,
        };
        result.process =
            Some(worker::run(&spec, work, cancelled).map_err(LocalProbeError::Launch)?);
        result.postflight_error =
            validate(recipe, result.preparation.profile(), deadline, cancelled).err();
        let observed = result.process.as_ref().ok_or(LocalProbeError::Unsettled)?;
        if let Some(reason) = observed.interruption {
            return Err(LocalProbeError::Interrupted(reason));
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
            return Err(LocalProbeError::Unsettled);
        }
        poll(deadline, cancelled)?;
        if result.postflight_error.is_some() {
            return Err(LocalProbeError::NotReady);
        }
        let usage_ms = u64::try_from(observed.elapsed.as_nanos().div_ceil(1_000_000))
            .map_err(|_| LocalProbeError::Deadline)?;
        if usage_ms > 60_000 {
            return Err(LocalProbeError::Deadline);
        }
        let output = if observed.stdout.bytes == recipe.expected_stdout
            && observed.stderr.bytes.is_empty()
        {
            UsefulResult::Passed
        } else {
            UsefulResult::Failed
        };
        result.facts = Some(ProbeObservation {
            facts: Facts::OneShot {
                exit_code: observed.exit_code,
                output,
            },
            probe_id: result.preparation.profile().probe_id.clone(),
            probe_version: result.preparation.profile().probe_version,
            actual_identity: Some(recipe.executable.to_string_lossy().into_owned()),
            immutable_revision: Some(recipe.executable_sha256.clone()),
            observed_unix_ms: Some(
                store
                    .roster_snapshot(principal, deadline)
                    .map_err(|error| {
                        LocalProbeError::Preparation(ProbeError::Service(error.into()))
                    })?
                    .now
                    .unix_ms,
            ),
            latency_ms: Some(usage_ms),
            usage_ms: Some(usage_ms),
            cost_microunits: Some(0),
            cleanup_settled: true,
            evidence_ref: evidence_id.as_str().into(),
        });
        Ok(())
    })();
    result.error = outcome.err();
    result.wall_elapsed = start.elapsed();
    result
}
fn poll(deadline: Instant, cancelled: &AtomicBool) -> Result<(), LocalProbeError> {
    if cancelled.load(Ordering::Acquire) {
        return Err(LocalProbeError::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(LocalProbeError::Deadline);
    }
    Ok(())
}
fn validate(
    recipe: &LocalRecipe,
    profile: &Profile,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<(), LocalProbeError> {
    poll(deadline, cancelled)?;
    if !matches!(
        recipe.executable.to_str(),
        Some("/usr/bin/printf" | "/usr/bin/false" | "/usr/bin/sleep")
    ) || profile.class != Class::OneShot
        || recipe.expected_stdout.is_empty()
        || recipe.expected_stdout.len() > MAX_STREAM
        || !recipe.executable.is_absolute()
        || recipe.executable.to_str() != Some(profile.actual_identity.as_str())
        || recipe.executable_sha256 != profile.immutable_revision
        || fs::canonicalize(&recipe.executable).map_err(LocalProbeError::from)? != recipe.executable
        || !recipe.directory.is_absolute()
        || fs::canonicalize(&recipe.directory).map_err(LocalProbeError::from)? != recipe.directory
        || !recipe.directory.is_dir()
        || fs::metadata(&recipe.directory)
            .map_err(LocalProbeError::from)?
            .permissions()
            .mode()
            & 0o777
            != 0o700
    {
        return Err(LocalProbeError::Recipe);
    }
    let metadata = fs::symlink_metadata(&recipe.executable).map_err(LocalProbeError::from)?;
    if !metadata.is_file() || metadata.len() > 16 * 1024 * 1024 {
        return Err(LocalProbeError::Recipe);
    }
    let mut file = File::open(&recipe.executable).map_err(LocalProbeError::from)?;
    let mut hash = Sha256::new();
    let mut total = 0_u64;
    loop {
        poll(deadline, cancelled)?;
        let mut bytes = [0_u8; 8192];
        let n = file.read(&mut bytes).map_err(LocalProbeError::from)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > 16 * 1024 * 1024 {
            return Err(LocalProbeError::Recipe);
        }
        hash.update(&bytes[..n]);
    }
    let mut digest = String::from("sha256:");
    for byte in hash.finalize() {
        const HEX: &[u8] = b"0123456789abcdef";
        digest.push(char::from(HEX[usize::from(byte >> 4)]));
        digest.push(char::from(HEX[usize::from(byte & 15)]));
    }
    if total != metadata.len() || digest != recipe.executable_sha256 {
        return Err(LocalProbeError::Recipe);
    }
    Ok(())
}
impl From<std::io::Error> for LocalProbeError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.raw_os_error())
    }
}
