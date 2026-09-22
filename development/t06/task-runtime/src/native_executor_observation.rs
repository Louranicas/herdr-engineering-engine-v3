//! Fresh evidence-backed liveness observation for the fixed native T06 executor.

use habitat_engine::app::evidence::Evidence;
use habitat_engine::check::collector::SinkError;
use habitat_engine::contracts::Sha256Digest;
use habitat_engine::contracts::receipt::{Payload, Ref};
use habitat_engine::contracts::roster::{
    Availability, Kind, Locality, Observation, ObservationInput, ObservationSource, Selection,
};
use habitat_engine::store::{self, Object, Principal, Store};
use rustix::fs::{Mode, OFlags, open};
use rustix::process::geteuid;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs::{File, Metadata};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const MAX_EXECUTABLE_BYTES: u64 = 64 * 1024 * 1024;
const CHUNK: usize = 64 * 1024;
const CAPABILITY: &str = "u64-fixed-workload";
const OPEN: OFlags = OFlags::RDONLY
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK)
    .union(OFlags::CLOEXEC);

#[derive(Debug)]
pub enum Error {
    Deadline,
    Identity,
    Bound,
    Io(std::io::Error),
    Store(store::Error),
    Publication(SinkError),
    Encoding(serde_json::Error),
}

impl From<store::Error> for Error {
    fn from(value: store::Error) -> Self {
        Self::Store(value)
    }
}

#[derive(Clone, Debug)]
pub struct PinnedExecutable {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub uid: u32,
}

#[derive(Debug)]
pub struct FreshObservation {
    pub observation: Observation,
    pub evidence: Ref,
    pub object: Object,
    pub raw_bytes: Vec<u8>,
    pub executable: PinnedExecutable,
}

#[derive(Serialize)]
struct EvidenceRecord<'a> {
    kind: &'static str,
    record_id: &'a str,
    record_version: &'a str,
    owner_id: &'a str,
    endpoint_ref: &'a Option<String>,
    source: &'static str,
    availability: &'static str,
    capability: &'static str,
    executable_path_bytes: &'a [u8],
    executable_sha256: &'a str,
    executable_bytes: u64,
    executable_device: u64,
    executable_inode: u64,
    executable_mode: u32,
    executable_uid: u32,
    observed_unix_ms: u64,
    scope: &'static str,
}

/// Observe the currently executing fixed native owner immediately before roster start.
/// This publishes and reads back exact evidence before recording a trusted Worker observation.
/// It does not register profiles, begin attempts, grant capabilities, or select a model/provider.
///
/// # Errors
/// Refuses stale/wrong selections, non-local/non-agent profiles, an arbitrary or mutable
/// executable, changed source identity, publication/readback failure, or expired deadline.
pub fn observe(
    store: &mut Store,
    principal: &Principal,
    staged: &mut Evidence<'_>,
    selection: &Selection,
    expected_executable: &Path,
    expected_sha256: &str,
    deadline: Instant,
) -> Result<FreshObservation, Error> {
    tick(deadline)?;
    selection.validate().map_err(|_| Error::Identity)?;
    if selection.capabilities.as_slice() != [CAPABILITY] || !selection.local_only {
        return Err(Error::Identity);
    }
    let id = habitat_engine::contracts::UuidV4::parse(&selection.record_id)
        .map_err(|_| Error::Identity)?;
    let record = store.roster_get(principal, id, deadline)?;
    if record.head.disabled
        || record.head.record_version != selection.expected_revision
        || record.head.definition.kind != Kind::Agent
        || record.head.definition.locality != Locality::Local
        || !record
            .head
            .definition
            .capabilities
            .iter()
            .any(|value| value == CAPABILITY)
        || selection
            .version
            .as_ref()
            .is_some_and(|value| value != &record.head.definition.version)
    {
        return Err(Error::Identity);
    }

    let executable = pin_current_executable(expected_executable, expected_sha256, deadline)?;
    let observed_unix_ms = unix_ms()?;
    let raw_bytes = serde_json::to_vec(&EvidenceRecord {
        kind: "hee3-fixed-native-executor-observation/1",
        record_id: &record.head.record_id,
        record_version: &record.head.record_version,
        owner_id: &record.head.definition.owner_id,
        endpoint_ref: &record.head.definition.endpoint_ref,
        source: "worker",
        availability: "available",
        capability: CAPABILITY,
        executable_path_bytes: expected_executable.as_os_str().as_encoded_bytes(),
        executable_sha256: &executable.sha256,
        executable_bytes: executable.bytes,
        executable_device: executable.device,
        executable_inode: executable.inode,
        executable_mode: executable.mode,
        executable_uid: executable.uid,
        observed_unix_ms,
        scope: "fixed local native owner; no provider or model identity",
    })
    .map_err(Error::Encoding)?;
    tick(deadline)?;
    let payload: Payload = staged
        .payload(&raw_bytes, "application/json")
        .map_err(Error::Publication)?;
    let evidence = payload.as_ref().clone();
    let object = staged
        .registered()
        .get(evidence.artifact_id.as_str())
        .filter(|(registered, object)| {
            registered == &evidence
                && object.digest() == evidence.sha256.as_str()
                && object.size() == u64::from(evidence.byte_length)
        })
        .map(|(_, object)| object.clone())
        .ok_or(Error::Identity)?;
    tick(deadline)?;

    let input = ObservationInput {
        record_id: record.head.record_id,
        record_version: record.head.record_version,
        owner_id: record.head.definition.owner_id,
        endpoint_ref: record.head.definition.endpoint_ref,
        instance_id: None,
        instance_generation: None,
        source: ObservationSource::Worker,
        // This same-process boundary has no independent source clock. Preserve the
        // local sample in raw evidence; Store's receiver clock owns freshness.
        observed_unix_ms: None,
        availability: Availability::Available,
        actual_identity: Some(format!("native-executor:{}", executable.sha256)),
        immutable_revision: Some(executable.sha256.clone()),
        capabilities: vec![CAPABILITY.to_owned()],
        evidence_ref: evidence.artifact_id.as_str().to_owned(),
    };
    let observation = store.roster_observe_worker(principal, &input, deadline)?;
    if observation.confirmed_source != Some(ObservationSource::Worker)
        || observation.input != input
        || observation.input.instance_id.is_some()
    {
        return Err(Error::Identity);
    }
    Ok(FreshObservation {
        observation,
        evidence,
        object,
        raw_bytes,
        executable,
    })
}

fn tick(deadline: Instant) -> Result<(), Error> {
    if Instant::now() >= deadline {
        Err(Error::Deadline)
    } else {
        Ok(())
    }
}

fn unix_ms() -> Result<u64, Error> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Identity)?
            .as_millis(),
    )
    .map_err(|_| Error::Bound)
}

fn same(left: &Metadata, right: &Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mode() == right.mode()
        && left.uid() == right.uid()
        && left.gid() == right.gid()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

fn validate(metadata: &Metadata, proc_exe: &Metadata) -> Result<(), Error> {
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.uid() != geteuid().as_raw()
        || metadata.mode() & 0o022 != 0
        || metadata.mode() & 0o111 == 0
        || metadata.len() == 0
        || metadata.len() > MAX_EXECUTABLE_BYTES
        || metadata.dev() != proc_exe.dev()
        || metadata.ino() != proc_exe.ino()
    {
        return Err(Error::Identity);
    }
    Ok(())
}

fn digest_file(file: &mut File, length: u64, deadline: Instant) -> Result<String, Error> {
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = vec![0_u8; CHUNK];
    loop {
        tick(deadline)?;
        let count = file.read(&mut buffer).map_err(Error::Io)?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(u64::try_from(count).map_err(|_| Error::Bound)?)
            .ok_or(Error::Bound)?;
        if total > length || total > MAX_EXECUTABLE_BYTES {
            return Err(Error::Bound);
        }
        hasher.update(&buffer[..count]);
    }
    if total != length {
        return Err(Error::Identity);
    }
    let mut value = String::from("sha256:");
    for byte in hasher.finalize() {
        write!(&mut value, "{byte:02x}").map_err(|_| Error::Identity)?;
    }
    Ok(value)
}

fn pin_current_executable(
    path: &Path,
    expected_sha256: &str,
    deadline: Instant,
) -> Result<PinnedExecutable, Error> {
    tick(deadline)?;
    if !path.is_absolute() {
        return Err(Error::Identity);
    }
    Sha256Digest::parse(expected_sha256).map_err(|_| Error::Identity)?;
    let mut file = File::from(open(path, OPEN, Mode::empty()).map_err(|_| Error::Identity)?);
    let before = file.metadata().map_err(Error::Io)?;
    let proc_before = std::fs::metadata("/proc/self/exe").map_err(Error::Io)?;
    validate(&before, &proc_before)?;
    let sha256 = digest_file(&mut file, before.len(), deadline)?;
    let after = file.metadata().map_err(Error::Io)?;
    let named = File::from(open(path, OPEN, Mode::empty()).map_err(|_| Error::Identity)?)
        .metadata()
        .map_err(Error::Io)?;
    let proc_after = std::fs::metadata("/proc/self/exe").map_err(Error::Io)?;
    if sha256 != expected_sha256
        || !same(&before, &after)
        || !same(&before, &named)
        || !same(&proc_before, &proc_after)
        || before.dev() != proc_after.dev()
        || before.ino() != proc_after.ino()
    {
        return Err(Error::Identity);
    }
    tick(deadline)?;
    Ok(PinnedExecutable {
        path: path.to_path_buf(),
        sha256,
        bytes: before.len(),
        device: before.dev(),
        inode: before.ino(),
        mode: before.mode(),
        uid: before.uid(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::Duration;

    fn sha(path: &Path) -> String {
        let mut file = File::open(path).unwrap();
        let length = file.metadata().unwrap().len();
        digest_file(&mut file, length, Instant::now() + Duration::from_secs(30)).unwrap()
    }

    #[test]
    fn current_executable_is_bound_to_kernel_identity_and_exact_hash() {
        let path = std::env::current_exe().unwrap();
        let expected = sha(&path);
        let pinned =
            pin_current_executable(&path, &expected, Instant::now() + Duration::from_secs(30))
                .unwrap();
        assert_eq!(pinned.sha256, expected);
        assert_eq!(pinned.inode, fs::metadata("/proc/self/exe").unwrap().ino());
        assert!(matches!(
            pin_current_executable(
                &path,
                &format!("sha256:{}", "0".repeat(64)),
                Instant::now() + Duration::from_secs(30)
            ),
            Err(Error::Identity)
        ));
        let root =
            std::env::temp_dir().join(format!("hee3-native-observe-link-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let alias = root.join("alias");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(matches!(
            pin_current_executable(&alias, &expected, Instant::now() + Duration::from_secs(30)),
            Err(Error::Identity)
        ));
        fs::remove_dir_all(root).unwrap();
        assert!(matches!(
            pin_current_executable(
                Path::new("/usr/bin/true"),
                &expected,
                Instant::now() + Duration::from_secs(30)
            ),
            Err(Error::Identity)
        ));
    }

    #[test]
    fn mutable_hardlinked_and_nonexecuting_inputs_refuse_before_read() {
        let root = std::env::temp_dir().join(format!("hee3-native-observe-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let path = root.join("fixture");
        fs::write(&path, b"fixed").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        let linked = root.join("linked");
        fs::hard_link(&path, &linked).unwrap();
        let metadata = fs::metadata(&path).unwrap();
        assert!(matches!(
            validate(&metadata, &metadata),
            Err(Error::Identity)
        ));
        fs::remove_file(&linked).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o775)).unwrap();
        let metadata = fs::metadata(&path).unwrap();
        assert!(matches!(
            validate(&metadata, &metadata),
            Err(Error::Identity)
        ));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let metadata = fs::metadata(&path).unwrap();
        assert!(matches!(
            validate(&metadata, &metadata),
            Err(Error::Identity)
        ));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn expired_deadline_and_noncanonical_hash_refuse() {
        let path = std::env::current_exe().unwrap();
        assert!(matches!(
            pin_current_executable(
                &path,
                "not-a-digest",
                Instant::now().checked_sub(Duration::from_nanos(1)).unwrap()
            ),
            Err(Error::Deadline)
        ));
        assert!(matches!(
            pin_current_executable(
                &path,
                "not-a-digest",
                Instant::now() + Duration::from_secs(1)
            ),
            Err(Error::Identity)
        ));
    }
}
