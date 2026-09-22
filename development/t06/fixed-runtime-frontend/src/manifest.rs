//! Closed fixed frontend inputs. Hashes bind bytes; they do not authenticate a build.
use crate::{Result, checked};
use habitat_engine::app::workload::Tools;
use habitat_engine::worker::{
    namespace::ReadOnlyFile,
    workspace::{Content, Snapshot},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TreeFile {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub executable: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tree {
    pub path: PathBuf,
    pub directories: Vec<String>,
    pub files: Vec<TreeFile>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Roles {
    pub baseline: Tree,
    pub repaired: Tree,
    pub protected: Tree,
    pub fixtures: Tree,
    pub oracle: Tree,
    pub harness: Tree,
    pub collector: Tree,
    pub launcher: Tree,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    pub input: Pin,
    pub namespace: PathBuf,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolSet {
    pub bwrap: Pin,
    pub compiler: Projection,
    pub shim: Projection,
    pub systemd_run: Pin,
    pub busctl: Pin,
    pub uname: Pin,
    pub runtime_files: Vec<Projection>,
    pub namespace_directories: Vec<PathBuf>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionIds {
    pub run: String,
    pub attempt: String,
    pub session: String,
    pub workspace: String,
    pub stages: [String; 3],
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub task: String,
    pub store_generation: String,
    pub store_epoch: String,
    pub submit_key: String,
    pub submit_event: String,
    pub roster_key: String,
    pub attempts: [ExecutionIds; 2],
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Assets {
    pub schema: Pin,
    pub readiness: Pin,
    pub contracts: Pin,
    pub cargo_lock: Pin,
    pub runtime_lock: Pin,
    pub build: Pin,
    pub shim_build: Pin,
    pub finite_files: Pin,
    pub review: Pin,
    pub patch: Pin,
    pub authority: Pin,
    pub isolation: Pin,
    pub cleanup: Pin,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub kind: String,
    pub identities: Identity,
    pub roles: Roles,
    pub tools: ToolSet,
    pub assets: Assets,
    pub executor: Pin,
    pub runtime_dir: PathBuf,
    pub owner_id: String,
    pub namespace_source_sha256: String,
    pub shim_source_sha256: String,
    pub build_target: String,
    pub build_profile: String,
    pub rust_flags: Vec<String>,
}

pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::new();
    for byte in bytes {
        use std::fmt::Write;
        write!(out, "{byte:02x}").expect("String");
    }
    out
}
pub fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(&Sha256::digest(bytes)))
}
pub fn digest_array(value: &str) -> Result<[u8; 32]> {
    let value = value.strip_prefix("sha256:").ok_or("digest prefix")?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("digest shape".into());
    }
    let mut output = [0; 32];
    for (slot, pair) in output
        .iter_mut()
        .zip(value.as_bytes().as_chunks::<2>().0.iter())
    {
        *slot = checked(u8::from_str_radix(checked(std::str::from_utf8(pair))?, 16))?;
    }
    Ok(output)
}
pub fn tick(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline {
        Err("original deadline expired".into())
    } else {
        Ok(())
    }
}
pub fn read(pin: &Pin, limit: u64, deadline: Instant) -> Result<Vec<u8>> {
    tick(deadline)?;
    digest_array(&pin.sha256)?;
    if pin.bytes > limit || !pin.path.is_absolute() || checked(pin.path.canonicalize())? != pin.path
    {
        return Err("file pin path/bound".into());
    }
    let mut f = checked(
        OpenOptions::new()
            .read(true)
            .custom_flags(
                rustix::fs::OFlags::NOFOLLOW.bits().cast_signed()
                    | rustix::fs::OFlags::NONBLOCK.bits().cast_signed(),
            )
            .open(&pin.path),
    )?;
    let before = checked(f.metadata())?;
    if !before.is_file() || before.len() != pin.bytes {
        return Err("file type/length".into());
    }
    let mut data = Vec::new();
    let mut chunk = vec![0; 65536];
    loop {
        tick(deadline)?;
        let n = checked(f.read(&mut chunk))?;
        if n == 0 {
            break;
        }
        if u64::try_from(data.len() + n).unwrap_or(u64::MAX) > limit {
            return Err("file grew beyond bound".into());
        }
        data.extend_from_slice(&chunk[..n]);
    }
    let after = checked(f.metadata())?;
    let named = checked(fs::symlink_metadata(&pin.path))?;
    let identity = |m: &fs::Metadata| {
        (
            m.dev(),
            m.ino(),
            m.mode(),
            m.len(),
            m.mtime(),
            m.mtime_nsec(),
            m.ctime(),
            m.ctime_nsec(),
        )
    };
    if identity(&before) != identity(&after)
        || identity(&after) != identity(&named)
        || digest(&data) != pin.sha256
    {
        return Err("file pin drift".into());
    }
    tick(deadline)?;
    Ok(data)
}
pub fn snapshot(tree: &Tree, deadline: Instant) -> Result<Snapshot> {
    let snap = checked(Snapshot::capture(&tree.path, &[], deadline))?;
    let dirs: Vec<_> = snap
        .entries()
        .filter_map(|e| matches!(e.content, Content::Directory).then_some(e.path.clone()))
        .collect();
    if dirs != tree.directories {
        return Err("role directory inventory drift".into());
    }
    let actual: Vec<_> = snap
        .entries()
        .filter_map(|e| match &e.content {
            Content::File {
                bytes, executable, ..
            } => Some((
                e.path.clone(),
                digest(bytes),
                bytes.len() as u64,
                *executable,
            )),
            Content::Directory => None,
        })
        .collect();
    let expected: Vec<_> = tree
        .files
        .iter()
        .map(|f| (f.path.clone(), f.sha256.clone(), f.bytes, f.executable))
        .collect();
    if actual != expected || actual.is_empty() {
        return Err("role file inventory drift".into());
    }
    Ok(snap)
}
fn projection(value: &Projection, deadline: Instant) -> Result<ReadOnlyFile> {
    read(&value.input, 1024 * 1024 * 1024, deadline)?;
    Ok(ReadOnlyFile {
        host: value.input.path.clone(),
        namespace: value.namespace.clone(),
        sha256: digest_array(&value.input.sha256)?,
    })
}
pub fn tools(value: &ToolSet, deadline: Instant) -> Result<Tools> {
    if value.runtime_files.len() > 512 {
        return Err("finite input count".into());
    }
    let mut seen = BTreeSet::new();
    let mut total = 0_u64;
    for file in value
        .runtime_files
        .iter()
        .chain([&value.compiler, &value.shim])
    {
        total = total
            .checked_add(file.input.bytes)
            .ok_or("finite input sum")?;
        if !seen.insert(&file.namespace) || total > 1024 * 1024 * 1024 {
            return Err("duplicate/bounded namespace projection".into());
        }
    }
    for file in [&value.bwrap, &value.systemd_run, &value.busctl] {
        read(file, 16 * 1024 * 1024, deadline)?;
    }
    Ok(Tools {
        bwrap: value.bwrap.path.clone(),
        compiler: projection(&value.compiler, deadline)?,
        shim: projection(&value.shim, deadline)?,
        runtime_files: value
            .runtime_files
            .iter()
            .map(|v| projection(v, deadline))
            .collect::<Result<_>>()?,
        namespace_directories: value.namespace_directories.clone(),
    })
}
pub fn private_dir(path: &Path) -> Result<()> {
    checked(fs::DirBuilder::new().mode(0o700).create(path))?;
    checked(fs::set_permissions(path, fs::Permissions::from_mode(0o700)))
}
pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = checked(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path),
    )?;
    checked(file.write_all(bytes))?;
    checked(file.sync_all())
}
pub fn json(path: &Path, value: &impl Serialize) -> Result<()> {
    write(path, &checked(serde_json::to_vec_pretty(value))?)
}
impl Manifest {
    pub fn load(path: &Path, retained: &Path, deadline: Instant) -> Result<Self> {
        tick(deadline)?;
        if !path.is_absolute() || checked(path.canonicalize())? != path {
            return Err("manifest canonical path".into());
        }
        let mut file = checked(
            OpenOptions::new()
                .read(true)
                .custom_flags(
                    (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK)
                        .bits()
                        .cast_signed(),
                )
                .open(path),
        )?;
        let before = checked(file.metadata())?;
        if !before.is_file() || before.len() > 4 * 1024 * 1024 {
            return Err("manifest type/bound".into());
        }
        let mut bytes = Vec::new();
        checked(
            Read::by_ref(&mut file)
                .take(4 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes),
        )?;
        let after = checked(file.metadata())?;
        let named = checked(fs::symlink_metadata(path))?;
        let identity = |m: &fs::Metadata| {
            (
                m.dev(),
                m.ino(),
                m.mode(),
                m.len(),
                m.mtime(),
                m.mtime_nsec(),
                m.ctime(),
                m.ctime_nsec(),
            )
        };
        if bytes.len() > 4 * 1024 * 1024
            || identity(&before) != identity(&after)
            || identity(&after) != identity(&named)
        {
            return Err("manifest drift/bound".into());
        }
        tick(deadline)?;
        write(retained, &bytes)?;
        let m: Self = checked(serde_json::from_slice(&bytes))?;
        if m.kind != "hee3-fixed-u64-frontend/1"
            || m.build_profile != "release"
            || m.build_target != "x86_64-unknown-linux-gnu"
        {
            return Err("unsupported fixed manifest".into());
        }
        let base = checked(path.parent().ok_or("manifest parent")?.canonicalize())?;
        for tree in [
            &m.roles.baseline,
            &m.roles.repaired,
            &m.roles.protected,
            &m.roles.fixtures,
            &m.roles.oracle,
            &m.roles.harness,
            &m.roles.collector,
            &m.roles.launcher,
        ] {
            if !tree.path.starts_with(&base) || tree.path == base {
                return Err("role outside frozen input root".into());
            }
        }
        if !m.executor.path.starts_with(&m.roles.collector.path) {
            return Err("executor outside collector subject".into());
        }
        Ok(m)
    }
}
#[cfg(test)]
#[path = "../tests/frontend_controls.rs"]
mod tests;
