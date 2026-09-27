//! The operator's native-provider file (B14b-2; R21 D1 as amended by N6, N9, N10, N11): what this
//! machine holds for the class's native model — the HTTP client, the model's install, the daemon's
//! unit and executable, the client's working directory, and the two keys the roster install is
//! keyed by. Read once under the class profile's custody, from `<config root>/`[`NATIVE_DIRECTORY`].
//!
//! [`compose`] is pure: it refuses by name what the file's own bytes decide — the
//! schema, a path that is not clean and absolute (the namespace's `host_shape`, N9), a digest that
//! is not `sha256:` plus 64 lowercase hex, a pin of zero bytes, a scope other than the user's, a
//! roster key that is not a v4 UUID. Nothing here hashes a file, canonicalises a path or reaches
//! the daemon: canonicality and the working directory's 0700/euid check are `worker::native`'s
//! doors at use (N9, N11), and the daemon is resolved from `[daemon]` per dispatch (N6). There is
//! no `[tools]` table: busctl and systemd-run are fixed host paths pinned by the class (N6).
//!
//! [`install`] is the one roster write the file drives (N12): the native agent record, created once
//! under the file's request key and revised in place when the file changes, once per `serve` start.
//!
//! [`NativeProvider`] is the dispatcher's production [`Provider`] (R21 D2 as amended by N1, N6,
//! N7, N22 and S20a): per dispatch it checks the file against the class's native row, resolves the
//! daemon, reads the class prompt from the reviewed closure and the admitted baseline, and composes
//! the live verifier over the check-scoped aggregate lifecycle.

use super::candidates::{ClassPrompt, NativeCandidates};
use super::class_profile;
use super::custody::{DirectoryError, FileError, PrivateDirectory};
use super::dispatcher::{NativeWhy, NoNative, Provider, Unavailable};
use super::evidence::fresh_id;
use super::live_verifier::{LiveVerifier, Manager};
use super::runtime::Admitted;
use super::tasks::StoreTasks;
use crate::contracts::roster::{
    Kind, Locality, MAX_TTL_MS, RosterDefinitionV1, RosterHeadV1, Selection, Update,
};
use crate::contracts::{OPERATOR_ROLE, Sha256Digest, UuidV4};
use crate::store::{Dispatchable, Error as StoreError, Principal, RequestSource, Store};
use crate::worker::namespace;
use crate::worker::native::{self, AdapterProfile, DaemonPin, FilePin, MainPid};
use crate::worker::workspace::{Content, Snapshot};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The provider's own directory under the configuration root (`coordinator::config_path`): 0700,
/// holding [`NATIVE_FILE`].
pub const NATIVE_DIRECTORY: &str = "native";
/// The declaration inside [`NATIVE_DIRECTORY`] (0600).
pub const NATIVE_FILE: &str = "native.toml";
/// The acquisition bound: `toml` decodes the whole file before any field is checked, so its size
/// is the only bound on everything declared in it — the class profile's own bound.
pub const MAX_NATIVE_BYTES: u64 = super::class_profile::MAX_PROFILE_BYTES;
const _: () = assert!(MAX_NATIVE_BYTES == 65_536);

const SCHEMA: &str = "hee3.native/1";
/// The one scope a daemon's unit is reached in: the operator's user manager (`busctl --user`).
const USER_SCOPE: &str = "user";

/// The operator's native-provider file, as declared.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NativeFile {
    pub schema: String,
    /// The HTTP client the adapter runs, as `worker::native::Profile.client`.
    pub client: Pin,
    pub install: Install,
    pub daemon: DaemonDecl,
    /// The client's working directory (0700, the operator's; checked at use).
    pub directory: PathBuf,
    pub roster: Roster,
}

/// A host file by path, digest and length (`worker::native::FilePin`'s fields).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}

/// The model's install: its manifest and the blob directory its layers are read from.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Install {
    pub manifest: Pin,
    pub blobs: PathBuf,
}

/// The daemon as declared (never derived, A6): its user unit and the executable the resolver's
/// one match must hash to.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaemonDecl {
    pub unit: String,
    pub scope: String,
    pub executable: Executable,
}

/// The daemon executable's digest and length.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Executable {
    pub sha256: String,
    pub bytes: u64,
}

/// The two v4 UUIDs the roster install is keyed by (N12): the request key of the install's
/// `roster.update`, and the record's endpoint reference.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Roster {
    pub idempotency_key: String,
    pub endpoint_ref: String,
}

/// Why a field the file declares is refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldWhy {
    /// Not a clean absolute path (`namespace::host_shape`).
    Path,
    /// Not `sha256:` plus 64 lowercase hex.
    Digest,
    /// A pin of zero bytes: no file the adapter reads is empty.
    Bytes,
    /// Not the user's scope.
    Scope,
    /// Not a v4 UUID.
    Uuid,
}

/// Every named refusal of the operator's native-provider file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeFileError {
    /// The directory or the file is absent: nothing is installed.
    NotInstalled,
    /// The custody door refused the directory or the file (never "absent": that is above).
    Read {
        what: &'static str,
    },
    Encoding,
    /// toml's own refusal — syntax, or a key unknown, missing or of the wrong type — at its
    /// position only (its text is multi-line and quotes the file).
    Decode {
        line: usize,
        column: usize,
    },
    Schema {
        found: String,
    },
    /// A declared field, at its key path.
    Field {
        field: &'static str,
        why: FieldWhy,
    },
}

/// Decode and shape-check an operator file (pure).
///
/// # Errors
/// [`NativeFileError::Encoding`]; `Decode` at toml's position; `Schema`; `Field` naming the key
/// path and the rule, checked in declaration order.
pub fn compose(bytes: &[u8]) -> Result<NativeFile, NativeFileError> {
    let text = std::str::from_utf8(bytes).map_err(|_| NativeFileError::Encoding)?;
    let file: NativeFile = toml::from_str(text).map_err(|error: toml::de::Error| {
        let (line, column) =
            super::class_profile::position(text, error.span().map_or(0, |span| span.start));
        NativeFileError::Decode { line, column }
    })?;
    if file.schema != SCHEMA {
        return Err(NativeFileError::Schema { found: file.schema });
    }
    let refuse = |field, why| NativeFileError::Field { field, why };
    let path = |field, path: &Path| {
        if namespace::host_shape(path) {
            Ok(())
        } else {
            Err(refuse(field, FieldWhy::Path))
        }
    };
    let digest = |field, text: &str| {
        Sha256Digest::parse(text)
            .map(|_| ())
            .map_err(|_| refuse(field, FieldWhy::Digest))
    };
    let bytes = |field, length: u64| {
        if length == 0 {
            Err(refuse(field, FieldWhy::Bytes))
        } else {
            Ok(())
        }
    };
    let uuid = |field, text: &str| {
        UuidV4::parse(text)
            .map(|_| ())
            .map_err(|_| refuse(field, FieldWhy::Uuid))
    };
    path("client.path", &file.client.path)?;
    digest("client.sha256", &file.client.sha256)?;
    bytes("client.bytes", file.client.bytes)?;
    path("install.manifest.path", &file.install.manifest.path)?;
    digest("install.manifest.sha256", &file.install.manifest.sha256)?;
    bytes("install.manifest.bytes", file.install.manifest.bytes)?;
    path("install.blobs", &file.install.blobs)?;
    if file.daemon.scope != USER_SCOPE {
        return Err(refuse("daemon.scope", FieldWhy::Scope));
    }
    digest("daemon.executable.sha256", &file.daemon.executable.sha256)?;
    bytes("daemon.executable.bytes", file.daemon.executable.bytes)?;
    path("directory", &file.directory)?;
    uuid("roster.idempotency_key", &file.roster.idempotency_key)?;
    uuid("roster.endpoint_ref", &file.roster.endpoint_ref)?;
    Ok(file)
}

/// Read and compose `directory`/[`NATIVE_FILE`] under custody (the only I/O here), returning the
/// file and the exact bytes it was composed from — the bytes the roster install publishes.
///
/// # Errors
/// [`NativeFileError::NotInstalled`] when the directory or the file is absent; `Read` with the
/// custody refusal's kind; any refusal of [`compose`].
pub fn read(directory: &Path) -> Result<(NativeFile, Vec<u8>), NativeFileError> {
    let refused = |what| NativeFileError::Read { what };
    let held = match PrivateDirectory::open(directory) {
        Ok(held) => held,
        Err(DirectoryError::NotFound) => return Err(NativeFileError::NotInstalled),
        Err(DirectoryError::Custody) => return Err(refused("directory custody")),
        Err(DirectoryError::Io(_)) => return Err(refused("directory io")),
    };
    let bytes = match held.read(NATIVE_FILE, MAX_NATIVE_BYTES) {
        Ok(bytes) => bytes,
        Err(FileError::NotFound) => return Err(NativeFileError::NotInstalled),
        Err(FileError::Custody) => return Err(refused("file custody")),
        Err(FileError::TooLarge) => return Err(refused("file too large")),
        Err(FileError::Io(_)) => return Err(refused("file io")),
    };
    Ok((compose(&bytes)?, bytes))
}

/// The dispatcher's kind for a refusal of the operator's file (R21 round-1 LOW F8): absent, not
/// read under custody, or read and not decoded. One arm per refusal, so a new one is a compile
/// error here.
#[must_use]
pub const fn no_native(error: &NativeFileError) -> NoNative {
    match error {
        NativeFileError::NotInstalled => NoNative::NotInstalled,
        NativeFileError::Read { .. } => NoNative::Read,
        NativeFileError::Encoding
        | NativeFileError::Decode { .. }
        | NativeFileError::Schema { .. }
        | NativeFileError::Field { .. } => NoNative::Decode,
    }
}

/// What the install established (R21 N3, N12): the agent record every begin names, and the one
/// selection it is begun under, pinned to the head the install read or wrote. Fixed at `serve`
/// start and handed to the dispatcher through its existing fields: one door.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Installed {
    pub record_id: String,
    pub selections: Vec<Selection>,
}

/// Why the install refused.
#[derive(Debug)]
pub enum InstallError {
    /// The ledger's owner panicked.
    Poisoned,
    /// No entropy for an update's fresh request key, within the deadline.
    Entropy,
    /// The roster's own refusal: the operator door, the deadline, a stale head (`Conflict`), a bound.
    Store(StoreError),
}

/// The action every install is recorded under: the roster's own write.
const ROSTER_UPDATE: &str = "roster.update";

/// The record the operator file and the class's adapter row declare (R21 D5): the native agent,
/// local, versioned by the adapter row it runs under, evidencing text, at the file's endpoint.
fn definition(file: &NativeFile, adapter: &AdapterProfile) -> RosterDefinitionV1 {
    RosterDefinitionV1 {
        kind: Kind::Agent,
        display_name: "native".to_owned(),
        owner_id: OPERATOR_ROLE.to_owned(),
        version: adapter.id.to_owned(),
        capabilities: vec![TEXT.to_owned()],
        locality: Locality::Local,
        endpoint_ref: Some(file.roster.endpoint_ref.clone()),
        limitations: format!(
            "adapter {}: num_ctx {}, num_predict {}, {}",
            adapter.id,
            adapter.num_ctx,
            adapter.num_predict,
            if adapter.templated {
                "templated"
            } else {
                "raw"
            }
        ),
    }
}

/// The one capability the native agent evidences.
const TEXT: &str = "text";

/// Install the operator file's agent record (R21 N12), once per `serve` start, in one hold:
/// 1. the create's recorded outcome, read by the file's request key (`roster_by_key`);
/// 2. none: create the record under that key, the file's bytes the request's source;
/// 3. found: read the record's current head; a definition equal to the file's applies nothing;
/// 4. different: one UPDATE of that record under a fresh request key, expecting the head's revision
///    — compare-and-swap, so a replay after a crash is equal or `Conflict`, never a second revision.
///
/// The file key is never reused for an edit: its request digest covers the whole file, so the
/// store would refuse the edited bytes under it as `Conflict`.
///
/// # Errors
/// [`InstallError::Poisoned`]; `Entropy` for an update's key; `Store` with the roster's refusal.
pub fn install(
    tasks: &StoreTasks,
    principal: &Principal,
    file: &NativeFile,
    bytes: &[u8],
    adapter: &AdapterProfile,
    deadline: Instant,
) -> Result<Installed, InstallError> {
    let declared = definition(file, adapter);
    let key = UuidV4::parse(&file.roster.idempotency_key)
        .map_err(|_| InstallError::Store(StoreError::Invalid))?;
    let update = |idempotency_key: String, head: Option<&RosterHeadV1>| Update {
        idempotency_key,
        record_id: head.map(|head| head.record_id.clone()),
        expected_revision: head.map(|head| head.record_version.clone()),
        definition: declared.clone(),
        audit_reason: "native provider install from the operator file".to_owned(),
    };
    let head = tasks
        .with_store(|store| -> Result<RosterHeadV1, InstallError> {
            let apply = |store: &mut Store, update: Update| {
                store
                    .roster_apply(principal, &[update], RequestSource::Native(bytes), deadline)?
                    .pop()
                    .map(|outcome| outcome.head)
                    .ok_or(StoreError::Runtime)
            };
            match store.roster_by_key(principal, ROSTER_UPDATE, key, deadline) {
                Err(StoreError::NotFound) => {
                    apply(store, update(file.roster.idempotency_key.clone(), None))
                        .map_err(InstallError::Store)
                }
                Err(error) => Err(InstallError::Store(error)),
                Ok(created) => {
                    let id = UuidV4::parse(&created.head.record_id)
                        .map_err(|_| InstallError::Store(StoreError::Corrupt))?;
                    let head = store
                        .roster_get(principal, id, deadline)
                        .map_err(InstallError::Store)?
                        .head;
                    if head.definition == declared {
                        return Ok(head);
                    }
                    let fresh = fresh_id(deadline).map_err(|_| InstallError::Entropy)?;
                    apply(store, update(fresh.as_str().to_owned(), Some(&head)))
                        .map_err(InstallError::Store)
                }
            }
        })
        .map_err(|_| InstallError::Poisoned)??;
    Ok(Installed {
        selections: vec![Selection {
            record_id: head.record_id.clone(),
            expected_revision: head.record_version,
            capabilities: vec![TEXT.to_owned()],
            local_only: true,
            version: Some(adapter.id.to_owned()),
            ttl_ms: MAX_TTL_MS,
        }],
        record_id: head.record_id,
    })
}

/// The production provider (R21 D2): the operator's file, where a unit's main pid comes from
/// (`native::Systemd` in `serve`, a double in the gate), and the engine's runtime directory the
/// aggregate and the scopes are pinned to.
pub struct NativeProvider<M> {
    file: NativeFile,
    main_pid: M,
    runtime_dir: PathBuf,
}

impl<M: MainPid> NativeProvider<M> {
    #[must_use]
    pub const fn new(file: NativeFile, main_pid: M, runtime_dir: PathBuf) -> Self {
        Self {
            file,
            main_pid,
            runtime_dir,
        }
    }
}

/// A host file's pin as the adapter reads it, from the operator's declaration.
fn file_pin(pin: &Pin) -> FilePin {
    FilePin {
        path: pin.path.clone(),
        sha256: pin.sha256.clone(),
        bytes: pin.bytes,
    }
}

/// The bytes of one file in the admitted baseline, by its baseline-relative path; empty when the
/// baseline holds no such file, which the class prompt refuses by the input's name.
fn baseline_file(baseline: &Snapshot, path: &str) -> Vec<u8> {
    baseline
        .entries()
        .find(|entry| entry.path == path)
        .and_then(|entry| match &entry.content {
            Content::File { bytes, .. } => Some(bytes.clone()),
            Content::Directory => None,
        })
        .unwrap_or_default()
}

impl<M: MainPid> Provider for NativeProvider<M> {
    type Source = NativeCandidates;
    type Verifier = LiveVerifier<Manager>;

    /// In order, each refusal the dispatcher's named state (the task stays `admitted`, N2):
    /// 1. the class's native row, else `ClassNotNative`;
    /// 2. the operator's manifest pin equal to the row's, and the row's adapter known;
    /// 3. the daemon resolved under the dispatch deadline and the dispatch's drain (N6, N7);
    /// 4. the client's working directory (N11);
    /// 5. the class prompt: the reviewed closure's pins and task text, the admitted baseline's
    ///    `Cargo.toml` and `src/lib.rs` (N22);
    /// 6. the live verifier over the class's systemd-run pin and a check-scoped aggregate
    ///    lifecycle over the class's busctl pin (S20a).
    fn open(
        &mut self,
        _next: &Dispatchable,
        admitted: &Admitted<'_>,
    ) -> Result<(NativeCandidates, LiveVerifier<Manager>), Unavailable> {
        let class = admitted.profile();
        let declared = &class.declared;
        let row = declared
            .native
            .as_ref()
            .ok_or(Unavailable::ClassNotNative)?;
        if self.file.install.manifest.sha256 != row.manifest_sha256 {
            return Err(Unavailable::Native(NativeWhy::Manifest));
        }
        let adapter =
            native::adapter(&row.adapter).ok_or(Unavailable::Native(NativeWhy::Adapter))?;
        let (_, deadline) = admitted.window();
        let daemon = native::resolve(
            &DaemonPin {
                unit: self.file.daemon.unit.clone(),
                executable_sha256: self.file.daemon.executable.sha256.clone(),
                executable_bytes: self.file.daemon.executable.bytes,
            },
            &mut self.main_pid,
            deadline,
            admitted.drain(),
        )
        .map_err(Unavailable::Daemon)?;
        native::working_directory(&self.file.directory)
            .map_err(|_| Unavailable::Native(NativeWhy::Directory))?;
        let (pins, task) =
            class_profile::candidate_inputs(class).map_err(|_| Unavailable::Closure)?;
        let prompt = ClassPrompt::new(
            &task,
            &baseline_file(admitted.baseline(), "Cargo.toml"),
            &baseline_file(admitted.baseline(), "src/lib.rs"),
            &pins,
        )
        .map_err(Unavailable::Prompt)?;
        let profile = native::Profile {
            model: row.model.clone(),
            manifest: file_pin(&self.file.install.manifest),
            blobs: self.file.install.blobs.clone(),
            client: file_pin(&self.file.client),
            daemon,
            directory: self.file.directory.clone(),
        };
        Ok((
            NativeCandidates::new(profile, adapter, prompt),
            LiveVerifier::new(
                declared.systemd_run_sha256.clone(),
                self.runtime_dir.clone(),
                Manager::new(declared.busctl_sha256.clone(), self.runtime_dir.clone()),
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    /// The host's own file, every value measured on this machine 2026-09-27: `curl` (the toolbox's
    /// `/usr/bin/curl`, coreutils `sha256sum` and `ls -l`), the `llama3.2:3b` manifest
    /// (`sha256sum`, 1,005 B), and the daemon executable (DS18-busctl: `/usr/bin/ollama`,
    /// 32,276,424 B). The directory and the roster keys are the operator's to choose.
    const HOST: &str = r#"schema = "hee3.native/1"
directory = "/var/home/Louranicas/.local/state/hee3-native"

[client]
path = "/usr/bin/curl"
sha256 = "sha256:a57a75f1b0c309eb4a21cc82efd24645be868c3a2689912b26fa4b7e940dcdd6"
bytes = 218440

[install]
manifest = { path = "/var/home/Louranicas/.ollama/models/manifests/registry.ollama.ai/library/llama3.2/3b", sha256 = "sha256:a80c4f17acd55265feec403c7aef86be0c25983ab279d83f3bcd3abbcb5b8b72", bytes = 1005 }
blobs = "/var/home/Louranicas/.ollama/models/blobs"

[daemon]
unit = "ollama.service"
scope = "user"
executable = { sha256 = "sha256:12ff8654a500a29048e2a40ff297e778f98c31742ecbd354dc948dbab3cea1aa", bytes = 32276424 }

[roster]
idempotency_key = "28f00000-0000-4000-8000-000000000001"
endpoint_ref = "28f00000-0000-4000-8000-000000000002"
"#;

    /// A second file differing from [`HOST`] in every field the schema lets differ (the schema and
    /// the scope each admit one value).
    const OTHER: &str = r#"schema = "hee3.native/1"
directory = "/srv/hee/client-cwd"

[client]
path = "/opt/net/bin/http-client"
sha256 = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
bytes = 7

[install]
manifest = { path = "/srv/models/manifests/m", sha256 = "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210", bytes = 2 }
blobs = "/srv/models/blobs"

[daemon]
unit = "hee3-other.service"
scope = "user"
executable = { sha256 = "sha256:1111111111111111111111111111111111111111111111111111111111111111", bytes = 3 }

[roster]
idempotency_key = "5a000000-0000-4000-8000-00000000000a"
endpoint_ref = "5a000000-0000-4000-9000-00000000000b"
"#;

    fn pin(path: &str, sha256: &str, bytes: u64) -> Pin {
        Pin {
            path: path.into(),
            sha256: sha256.into(),
            bytes,
        }
    }

    fn with(old: &str, new: &str) -> String {
        assert_eq!(HOST.matches(old).count(), 1, "anchor {old:?}");
        HOST.replacen(old, new, 1)
    }

    fn private(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "hee3-native-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos())
        ));
        assert!(fs::DirBuilder::new().mode(0o700).create(&path).is_ok());
        path
    }

    /// Every rule `compose` keeps after decoding, one case per refusal site, in declaration order.
    fn refusals() -> Vec<(String, NativeFileError)> {
        let field = |field, why| NativeFileError::Field { field, why };
        vec![
            (
                with("schema = \"hee3.native/1\"", "schema = \"hee3.native/2\""),
                NativeFileError::Schema {
                    found: "hee3.native/2".into(),
                },
            ),
            (
                with("path = \"/usr/bin/curl\"", "path = \"usr/bin/curl\""),
                field("client.path", FieldWhy::Path),
            ),
            (
                with("sha256 = \"sha256:a57a75", "sha256 = \"sha256:A57a75"),
                field("client.sha256", FieldWhy::Digest),
            ),
            (
                with("bytes = 218440", "bytes = 0"),
                field("client.bytes", FieldWhy::Bytes),
            ),
            (
                with(
                    "path = \"/var/home/Louranicas/.ollama/models/manifests/",
                    "path = \"/var/home/Louranicas/../.ollama/models/manifests/",
                ),
                field("install.manifest.path", FieldWhy::Path),
            ),
            (
                with("sha256 = \"sha256:a80c4f17", "sha256 = \"a80c4f17"),
                field("install.manifest.sha256", FieldWhy::Digest),
            ),
            (
                with("bytes = 1005", "bytes = 0"),
                field("install.manifest.bytes", FieldWhy::Bytes),
            ),
            (
                with(
                    "blobs = \"/var/home/Louranicas/.ollama/models/blobs\"",
                    "blobs = \"\"",
                ),
                field("install.blobs", FieldWhy::Path),
            ),
            (
                with("scope = \"user\"", "scope = \"system\""),
                field("daemon.scope", FieldWhy::Scope),
            ),
            (
                with("sha256 = \"sha256:12ff8654", "sha256 = \"sha256:12ff"),
                field("daemon.executable.sha256", FieldWhy::Digest),
            ),
            (
                with("bytes = 32276424", "bytes = 0"),
                field("daemon.executable.bytes", FieldWhy::Bytes),
            ),
            (
                with(
                    "directory = \"/var/home/Louranicas/.local/state/hee3-native\"",
                    "directory = \"/var/home/Louranicas/.local/state/hee3\\u0000native\"",
                ),
                field("directory", FieldWhy::Path),
            ),
            (
                with(
                    "idempotency_key = \"28f00000-0000-4000-8000-000000000001\"",
                    "idempotency_key = \"28f00000-0000-1000-8000-000000000001\"",
                ),
                field("roster.idempotency_key", FieldWhy::Uuid),
            ),
            (
                with(
                    "endpoint_ref = \"28f00000-0000-4000-8000-000000000002\"",
                    "endpoint_ref = \"ollama.service\"",
                ),
                field("roster.endpoint_ref", FieldWhy::Uuid),
            ),
        ]
    }

    /// R21 D1/N11 · two files differing in every field compose into exactly what each declares,
    /// compared whole; `read` returns the file and the very bytes it composed, names absence
    /// `NotInstalled` and a file another user could have written a custody refusal.
    #[test]
    fn compose_reads_two_files_differing_in_every_field_whole() -> Result<(), NativeFileError> {
        assert_eq!(
            compose(HOST.as_bytes())?,
            NativeFile {
                schema: "hee3.native/1".into(),
                client: pin(
                    "/usr/bin/curl",
                    "sha256:a57a75f1b0c309eb4a21cc82efd24645be868c3a2689912b26fa4b7e940dcdd6",
                    218_440,
                ),
                install: Install {
                    manifest: pin(
                        "/var/home/Louranicas/.ollama/models/manifests/registry.ollama.ai/library/llama3.2/3b",
                        "sha256:a80c4f17acd55265feec403c7aef86be0c25983ab279d83f3bcd3abbcb5b8b72",
                        1005,
                    ),
                    blobs: "/var/home/Louranicas/.ollama/models/blobs".into(),
                },
                daemon: DaemonDecl {
                    unit: "ollama.service".into(),
                    scope: "user".into(),
                    executable: Executable {
                        sha256:
                            "sha256:12ff8654a500a29048e2a40ff297e778f98c31742ecbd354dc948dbab3cea1aa"
                                .into(),
                        bytes: 32_276_424,
                    },
                },
                directory: "/var/home/Louranicas/.local/state/hee3-native".into(),
                roster: Roster {
                    idempotency_key: "28f00000-0000-4000-8000-000000000001".into(),
                    endpoint_ref: "28f00000-0000-4000-8000-000000000002".into(),
                },
            }
        );
        assert_eq!(
            compose(OTHER.as_bytes())?,
            NativeFile {
                schema: "hee3.native/1".into(),
                client: pin(
                    "/opt/net/bin/http-client",
                    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                    7,
                ),
                install: Install {
                    manifest: pin(
                        "/srv/models/manifests/m",
                        "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
                        2,
                    ),
                    blobs: "/srv/models/blobs".into(),
                },
                daemon: DaemonDecl {
                    unit: "hee3-other.service".into(),
                    scope: "user".into(),
                    executable: Executable {
                        sha256:
                            "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                                .into(),
                        bytes: 3,
                    },
                },
                directory: "/srv/hee/client-cwd".into(),
                roster: Roster {
                    idempotency_key: "5a000000-0000-4000-8000-00000000000a".into(),
                    endpoint_ref: "5a000000-0000-4000-9000-00000000000b".into(),
                },
            }
        );
        let root = private("read");
        assert_eq!(
            read(&root.join("absent")),
            Err(NativeFileError::NotInstalled)
        );
        assert_eq!(read(&root), Err(NativeFileError::NotInstalled));
        let file = root.join(NATIVE_FILE);
        assert!(fs::write(&file, OTHER).is_ok());
        assert!(fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).is_ok());
        assert_eq!(
            read(&root)?,
            (compose(OTHER.as_bytes())?, OTHER.as_bytes().to_vec())
        );
        assert!(fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).is_ok());
        assert_eq!(
            read(&root),
            Err(NativeFileError::Read {
                what: "file custody"
            })
        );
        Ok(())
    }

    /// Every rule `compose` keeps, one case per refusal site, each asserting the one field path
    /// only that site names; toml's own refusals at the position its message prints.
    #[test]
    fn compose_refuses_each_rule_by_name() {
        let cases = refusals();
        assert_eq!(cases.len(), 14);
        for (text, expected) in cases {
            assert_eq!(compose(text.as_bytes()), Err(expected), "{text}");
        }
        assert_eq!(compose(b"\xff"), Err(NativeFileError::Encoding));
        // toml's own refusals, at the position its message prints: an unknown table (the `[tools]`
        // N6 deleted), a missing key, a mistyped one.
        for text in [
            format!("{HOST}\n[tools]\nsystemctl = \"/usr/bin/systemctl\"\n"),
            HOST.replace("unit = \"ollama.service\"\n", ""),
            with("bytes = 218440", "bytes = \"218440\""),
            with("bytes = 218440", "bytes = -1"),
        ] {
            let message = toml::from_str::<NativeFile>(&text)
                .err()
                .map(|error| error.to_string())
                .unwrap_or_default();
            let numbers: Vec<usize> = message
                .lines()
                .next()
                .unwrap_or_default()
                .split(|c: char| !c.is_ascii_digit())
                .filter_map(|part| part.parse().ok())
                .collect();
            assert_eq!(numbers.len(), 2, "{message}");
            assert_eq!(
                compose(text.as_bytes()),
                Err(NativeFileError::Decode {
                    line: numbers[0],
                    column: numbers[1]
                }),
                "{message}"
            );
        }
    }

    /// R21 round-1 LOW F8 · every refusal of the operator's file has the dispatcher's kind for it:
    /// absent is `NotInstalled`, a custody door's refusal `Read`, and every refusal the compose
    /// door itself produces — its fourteen named rules, a non-UTF-8 file, toml's own position —
    /// `Decode`. The decode refusals are the door's own outputs, not values typed beside it.
    #[test]
    fn each_file_refusal_has_its_no_provider_kind() {
        let mut table = vec![
            (NativeFileError::NotInstalled, NoNative::NotInstalled),
            (
                NativeFileError::Read {
                    what: "file custody",
                },
                NoNative::Read,
            ),
            (
                NativeFileError::Read {
                    what: "directory io",
                },
                NoNative::Read,
            ),
        ];
        let decoded = refusals()
            .into_iter()
            .map(|(text, _)| text.into_bytes())
            .chain([b"\xff".to_vec(), format!("{HOST}\n[tools]\n").into_bytes()])
            .filter_map(|bytes| compose(&bytes).err());
        table.extend(decoded.map(|error| (error, NoNative::Decode)));
        assert_eq!(table.len(), 3 + 14 + 2);
        assert!(
            table
                .iter()
                .any(|(error, _)| *error == NativeFileError::Encoding)
        );
        assert!(
            table
                .iter()
                .any(|(error, _)| matches!(error, NativeFileError::Decode { .. }))
        );
        for (error, kind) in table {
            assert_eq!(no_native(&error), kind, "{error:?}");
        }
    }
}
