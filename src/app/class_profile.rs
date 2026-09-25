//! The installed profile of the one admitted task class, `rust-library-change/1` (B14-P2b): which
//! workspaces a task may name and the install-specific pins the workload mounts. Read once at start
//! under custody, like routing; declaration only — nothing here captures a workspace or hashes a
//! pin. The dispatcher (B14a/B14b) captures each workspace at dispatch and compares its declared
//! digest, and every pin is checked at use by its own door (design P2-R1.4, P2-R2.4, P2b-R1).
//!
//! What the class fixes is not declared here: the editable file, the candidate bounds and the
//! criteria are the check's (`consistency::U64_*`), the compiler's and the shim's namespaces are
//! the workload's and the namespace's, and bwrap, busctl and systemd-run live at fixed host paths.

use super::custody::{DirectoryError, FileError, PrivateDirectory};
use super::workload::FIXED_DESTINATIONS;
use crate::contracts::{Sha256Digest, UuidV4};
use crate::worker::namespace::{self, MAX_MOUNTS, SHIM_DESTINATION};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The class's own directory under `$HOME`: 0700, holding [`PROFILE_FILE`] and its workspaces.
pub const CLASS_DIRECTORY: &str =
    ".config/herdr-engineering-engine-v3/classes/rust-library-change-1";
/// The declaration inside [`CLASS_DIRECTORY`] (0600).
pub const PROFILE_FILE: &str = "profile.toml";
/// The acquisition bound: `toml` parses the whole file before any row is read, so the file's size
/// is the only bound on everything declared in it (the workspace rows included, P2b-R1.4).
pub const MAX_PROFILE_BYTES: u64 = 65_536;
/// The link stage's own read-only mounts beside the runtime files: the public wrapper and the
/// frozen library (`app::workload`, `/frozen/public-wrapper.rs`, `/frozen/libstrict_u64_workload.rlib`).
pub const WORKLOAD_MOUNTS: usize = 2;
/// The most runtime files a profile may declare: what the namespace mounts less the workload's own.
pub const MAX_RUNTIME_FILES: usize = MAX_MOUNTS - WORKLOAD_MOUNTS;
/// The class directory's store of independently reviewed records (B14a-2a): each a 0600 file named
/// by the 64 lowercase hex of its sha256, in this 0700 directory beside [`PROFILE_FILE`]. The repo
/// never carries the answer to its own review; the profile names each record by digest.
pub const REVIEWED_DIRECTORY: &str = "reviewed";
/// A reviewed record's acquisition bound: the profile's own. The class records it holds measured
/// 794 B (the expectation) and 8,888 B (its review provenance) in `fixed-task-execution-003`.
pub const MAX_REVIEWED_BYTES: u64 = MAX_PROFILE_BYTES;

const SCHEMA: &str = "hee3.class-profile/1";
const CLASS: &str = "rust-library-change/1";
/// Destinations under which the workload mounts and writes its own files.
const RESERVED_PREFIXES: [&str; 2] = ["/frozen", "/work"];

/// One workspace a task may name by `workspace_id`: its baseline and protected directories (beside
/// [`PROFILE_FILE`]) and the `Snapshot::content_digest` each must have when captured.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    pub id: String,
    pub baseline: String,
    pub baseline_digest: String,
    pub protected: String,
    pub protected_digest: String,
}

/// A host file mounted at a namespace destination the workload fixes: its path and digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostPin {
    pub host: PathBuf,
    pub sha256: [u8; 32],
}

/// A host file mounted at a declared namespace destination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeFile {
    pub host: PathBuf,
    pub namespace: PathBuf,
    pub sha256: [u8; 32],
}

/// The independently reviewed records a task's receipt binds (B14a-R8/R9): the expectation and its
/// review, each named by its `sha256:` digest; the objects are in [`REVIEWED_DIRECTORY`] and are
/// read only through [`read_reviewed`], which verifies them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reviewed {
    pub expectation: String,
    pub review: String,
}

/// What a profile declares, typed and shape-checked, before any of it is used.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declared {
    pub workspaces: Vec<Workspace>,
    pub compiler: HostPin,
    pub shim: HostPin,
    pub runtime_files: Vec<RuntimeFile>,
    pub namespace_directories: Vec<PathBuf>,
    /// `sha256:` spellings, the aggregate's and the resources' own field types.
    pub busctl_sha256: String,
    pub systemd_run_sha256: String,
    pub reviewed: Reviewed,
}

/// A read profile: its declaration, the directory it was read from, and the `sha256:` of the exact
/// bytes that declaration was composed from — the digest an attempt's binding records (B14a-1c).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    pub declared: Declared,
    pub directory: PathBuf,
    pub digest: String,
}

/// Which workspace field a refusal names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Field {
    Id,
    Baseline,
    BaselineDigest,
    Protected,
    ProtectedDigest,
}

/// Why a workspace row is refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceWhy {
    Uuid,
    DuplicateId,
    /// Not one plain directory name (empty, `.`, `..`, a `/`, a NUL, a control byte).
    Component,
    DuplicateDirectory,
    /// The baseline and the protected directory are one directory.
    Aliased,
    /// The name of the declaration itself.
    Reserved,
    Digest,
}

/// Why a pin is refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinWhy {
    /// A host path that is not clean and absolute.
    Host,
    /// A destination the namespace's own shape rule refuses.
    Namespace,
    Duplicate,
    /// A destination the workload or the namespace fixes.
    Reserved,
    /// A destination the plan creates as a directory: a declared one, or the parent of another
    /// destination (review P2b-3).
    Directory,
    Digest,
    Count {
        found: usize,
        limit: usize,
    },
}

/// Every named refusal of a profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfileError {
    /// The custody door refused the directory or the file (never "absent": that is `NotInstalled`).
    Read {
        what: &'static str,
    },
    Encoding,
    /// The toml error's position only: its own text is multi-line and quotes the file.
    Syntax {
        line: usize,
        column: usize,
    },
    UnknownKey {
        path: String,
    },
    MissingKey {
        path: String,
    },
    WrongType {
        path: String,
    },
    Schema {
        found: String,
    },
    Class {
        found: String,
    },
    Workspaces {
        count: usize,
    },
    Workspace {
        index: usize,
        id: String,
        field: Field,
        why: WorkspaceWhy,
    },
    Pin {
        name: String,
        why: PinWhy,
    },
    /// A `[reviewed]` entry: not a `sha256:` digest, or the review naming the expectation itself.
    Reviewed {
        name: &'static str,
        why: ReviewedWhy,
    },
}

/// Why a `[reviewed]` entry is refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedWhy {
    Digest,
    /// The review and the expectation are one record: a review of itself reviews nothing.
    Same,
}

/// Why dispatch has no profile: none installed, or one refused with its name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Unready {
    NotInstalled,
    Refused(ProfileError),
}

impl Unready {
    /// The one start line's reason: static for absence, the refusal's own name otherwise.
    #[must_use]
    pub fn constraint(&self) -> String {
        match self {
            Self::NotInstalled => "class profile not installed".to_owned(),
            Self::Refused(why) => format!("class profile refused: {why:?}"),
        }
    }
}

/// Why admission refuses the workspace a task names (B14-P2c).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    /// A profile is read and declares no workspace by that id.
    WorkspaceNotInstalled,
    /// A profile is installed and refused: a broken install admits nothing it cannot dispatch.
    ProfileRefused,
}

impl Screen {
    /// The refusal's message, one per reason (review P2c-8).
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::WorkspaceNotInstalled => "the installed class profile declares no such workspace",
            Self::ProfileRefused => "the installed class profile was refused at start",
        }
    }

    /// The refusal's constraint text, one per reason.
    #[must_use]
    pub const fn constraint(self) -> &'static str {
        match self {
            Self::WorkspaceNotInstalled => "workspace not installed",
            Self::ProfileRefused => "class profile refused",
        }
    }
}

/// The one rule admission screens a task's `workspace_id` by, for every door that screens it
/// (submit now, preview in B14-P2c-2), so two doors cannot keep it differently. A declared id is
/// admitted; an undeclared one, under a profile that was read, is not; a refused profile admits
/// nothing. With no profile installed a task is admitted, as before any profile existed — admission
/// without dispatch, which the dispatcher stops `no_workspace` (decision P2c-R1.5, recorded to
/// revisit when the dispatcher lands).
///
/// # Errors
/// [`Screen::WorkspaceNotInstalled`] and [`Screen::ProfileRefused`], as above.
pub fn screen(profile: &Result<Profile, Unready>, workspace_id: &str) -> Result<(), Screen> {
    match profile {
        Ok(read)
            if read
                .declared
                .workspaces
                .iter()
                .any(|workspace| workspace.id == workspace_id) =>
        {
            Ok(())
        }
        Ok(_) => Err(Screen::WorkspaceNotInstalled),
        Err(Unready::Refused(_)) => Err(Screen::ProfileRefused),
        Err(Unready::NotInstalled) => Ok(()),
    }
}

/// Read and compose `directory`/[`PROFILE_FILE`] under custody (the only I/O here).
///
/// # Errors
/// [`Unready::NotInstalled`] when the directory or the file is absent; [`Unready::Refused`] with
/// the custody refusal's kind, or with any refusal of [`compose`].
pub fn read(directory: &Path) -> Result<Profile, Unready> {
    let refused = |what| Unready::Refused(ProfileError::Read { what });
    let held = match PrivateDirectory::open(directory) {
        Ok(held) => held,
        Err(DirectoryError::NotFound) => return Err(Unready::NotInstalled),
        Err(DirectoryError::Custody) => return Err(refused("directory custody")),
        Err(DirectoryError::Io(_)) => return Err(refused("directory io")),
    };
    let bytes = match held.read(PROFILE_FILE, MAX_PROFILE_BYTES) {
        Ok(bytes) => bytes,
        Err(FileError::NotFound) => return Err(Unready::NotInstalled),
        Err(FileError::Custody) => return Err(refused("file custody")),
        Err(FileError::TooLarge) => return Err(refused("file too large")),
        Err(FileError::Io(_)) => return Err(refused("file io")),
    };
    Ok(Profile {
        declared: compose(&bytes).map_err(Unready::Refused)?,
        directory: directory.to_path_buf(),
        digest: super::evidence::digest(&bytes),
    })
}

/// Parse and shape-check a declaration (pure).
///
/// # Errors
/// Every refusal is named: [`ProfileError::Encoding`], `Syntax` with its position, an unknown,
/// missing or mistyped key with its path, the schema and class, the workspace rows (each with its
/// index, id, field and reason) and the pins (each with its name and reason).
pub fn compose(bytes: &[u8]) -> Result<Declared, ProfileError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ProfileError::Encoding)?;
    let table: toml::Table = text.parse().map_err(|error: toml::de::Error| {
        let (line, column) = position(text, error.span().map_or(0, |span| span.start));
        ProfileError::Syntax { line, column }
    })?;
    only(
        &table,
        "",
        &["schema", "class", "workspace", "pins", "reviewed"],
    )?;
    let schema = string(&table, "", "schema")?;
    if schema != SCHEMA {
        return Err(ProfileError::Schema { found: schema });
    }
    let class = string(&table, "", "class")?;
    if class != CLASS {
        return Err(ProfileError::Class { found: class });
    }
    let workspaces = workspaces(&table)?;
    let pins = sub_table(&table, "", "pins")?;
    only(
        pins,
        "pins",
        &[
            "compiler",
            "shim",
            "runtime_files",
            "namespace_directories",
            "busctl_sha256",
            "systemd_run_sha256",
        ],
    )?;
    let compiler = host_pin(pins, "compiler")?;
    let shim = host_pin(pins, "shim")?;
    let runtime_files = runtime_files(pins)?;
    let namespace_directories = directories(pins)?;
    derived(&runtime_files, &namespace_directories)?;
    Ok(Declared {
        workspaces,
        compiler,
        shim,
        runtime_files,
        namespace_directories,
        busctl_sha256: digest_text(pins, "busctl_sha256")?,
        systemd_run_sha256: digest_text(pins, "systemd_run_sha256")?,
        reviewed: reviewed(&table)?,
    })
}

/// 1-based line and column of byte `offset` in `text`, the column in characters. That is the toml
/// crate's convention where the error falls on a character boundary after ASCII text (a test
/// compares the two there); toml counts bytes when the erroring byte starts a multi-byte
/// character, so there the two may differ by that character's extra bytes (review P2b-4).
fn position(text: &str, offset: usize) -> (usize, usize) {
    let before = text.get(..offset.min(text.len())).unwrap_or(text);
    let (line, last) = before
        .split('\n')
        .fold((0, 0), |(count, _), part| (count + 1, part.chars().count()));
    (line, last + 1)
}

fn key_path(at: &str, key: &str) -> String {
    if at.is_empty() {
        key.to_owned()
    } else {
        format!("{at}.{key}")
    }
}

fn only(table: &toml::Table, at: &str, known: &[&str]) -> Result<(), ProfileError> {
    match table.keys().find(|key| !known.contains(&key.as_str())) {
        Some(key) => Err(ProfileError::UnknownKey {
            path: key_path(at, key),
        }),
        None => Ok(()),
    }
}

fn required<'a>(
    table: &'a toml::Table,
    at: &str,
    key: &str,
) -> Result<&'a toml::Value, ProfileError> {
    table.get(key).ok_or_else(|| ProfileError::MissingKey {
        path: key_path(at, key),
    })
}

fn wrong(at: &str, key: &str) -> ProfileError {
    ProfileError::WrongType {
        path: key_path(at, key),
    }
}

fn string(table: &toml::Table, at: &str, key: &str) -> Result<String, ProfileError> {
    match required(table, at, key)? {
        toml::Value::String(value) => Ok(value.clone()),
        _ => Err(wrong(at, key)),
    }
}

fn sub_table<'a>(
    table: &'a toml::Table,
    at: &str,
    key: &str,
) -> Result<&'a toml::Table, ProfileError> {
    match required(table, at, key)? {
        toml::Value::Table(value) => Ok(value),
        _ => Err(wrong(at, key)),
    }
}

fn array<'a>(
    table: &'a toml::Table,
    at: &str,
    key: &str,
) -> Result<&'a [toml::Value], ProfileError> {
    match required(table, at, key)? {
        toml::Value::Array(values) => Ok(values),
        _ => Err(wrong(at, key)),
    }
}

/// A plain directory name beside the declaration: one component, printable.
fn component(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name
            .bytes()
            .any(|byte| byte == b'/' || byte < 0x20 || byte == 0x7f)
}

fn workspaces(table: &toml::Table) -> Result<Vec<Workspace>, ProfileError> {
    let rows = array(table, "", "workspace")?;
    if rows.is_empty() {
        return Err(ProfileError::Workspaces { count: 0 });
    }
    let mut out: Vec<Workspace> = Vec::with_capacity(rows.len());
    let mut directories = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let at = format!("workspace[{index}]");
        let toml::Value::Table(row) = row else {
            return Err(ProfileError::WrongType { path: at });
        };
        only(
            row,
            &at,
            &[
                "id",
                "baseline",
                "baseline_digest",
                "protected",
                "protected_digest",
            ],
        )?;
        let id = string(row, &at, "id")?;
        let refuse = |field, why| ProfileError::Workspace {
            index,
            id: id.clone(),
            field,
            why,
        };
        if UuidV4::parse(&id).is_err() {
            return Err(refuse(Field::Id, WorkspaceWhy::Uuid));
        }
        if out.iter().any(|seen| seen.id == id) {
            return Err(refuse(Field::Id, WorkspaceWhy::DuplicateId));
        }
        let mut name = |field, key| -> Result<String, ProfileError> {
            let value = string(row, &at, key)?;
            if !component(&value) {
                return Err(refuse(field, WorkspaceWhy::Component));
            }
            if value == PROFILE_FILE {
                return Err(refuse(field, WorkspaceWhy::Reserved));
            }
            if !directories.insert(value.clone()) {
                return Err(refuse(field, WorkspaceWhy::DuplicateDirectory));
            }
            Ok(value)
        };
        let baseline = name(Field::Baseline, "baseline")?;
        let protected = string(row, &at, "protected")?;
        if protected == baseline {
            return Err(refuse(Field::Protected, WorkspaceWhy::Aliased));
        }
        let protected = name(Field::Protected, "protected")?;
        let digest = |field, key| -> Result<String, ProfileError> {
            let value = string(row, &at, key)?;
            Sha256Digest::parse(&value).map_err(|_| refuse(field, WorkspaceWhy::Digest))?;
            Ok(value)
        };
        let baseline_digest = digest(Field::BaselineDigest, "baseline_digest")?;
        let protected_digest = digest(Field::ProtectedDigest, "protected_digest")?;
        out.push(Workspace {
            id,
            baseline,
            baseline_digest,
            protected,
            protected_digest,
        });
    }
    Ok(out)
}

/// The 32 bytes of a `sha256:` spelling, or `None`.
fn digest_bytes(text: &str) -> Option<[u8; 32]> {
    Sha256Digest::parse(text).ok()?;
    let hex = text.strip_prefix("sha256:")?.as_bytes();
    let mut bytes = [0_u8; 32];
    for (slot, pair) in bytes.iter_mut().zip(hex.chunks(2)) {
        *slot = u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?;
    }
    Some(bytes)
}

fn pin_refusal(name: &str, why: PinWhy) -> ProfileError {
    ProfileError::Pin {
        name: name.to_owned(),
        why,
    }
}

fn host_pin(pins: &toml::Table, key: &str) -> Result<HostPin, ProfileError> {
    let at = format!("pins.{key}");
    let table = sub_table(pins, "pins", key)?;
    only(table, &at, &["host", "sha256"])?;
    let host = PathBuf::from(string(table, &at, "host")?);
    if !namespace::host_shape(&host) {
        return Err(pin_refusal(key, PinWhy::Host));
    }
    let sha256 = digest_bytes(&string(table, &at, "sha256")?)
        .ok_or_else(|| pin_refusal(key, PinWhy::Digest))?;
    Ok(HostPin { host, sha256 })
}

fn runtime_files(pins: &toml::Table) -> Result<Vec<RuntimeFile>, ProfileError> {
    let rows = array(pins, "pins", "runtime_files")?;
    if rows.len() > MAX_RUNTIME_FILES {
        return Err(pin_refusal(
            "runtime_files",
            PinWhy::Count {
                found: rows.len(),
                limit: MAX_RUNTIME_FILES,
            },
        ));
    }
    let mut out: Vec<RuntimeFile> = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let at = format!("pins.runtime_files[{index}]");
        let toml::Value::Table(row) = row else {
            return Err(ProfileError::WrongType { path: at });
        };
        only(row, &at, &["host", "namespace", "sha256"])?;
        let host = PathBuf::from(string(row, &at, "host")?);
        let destination = PathBuf::from(string(row, &at, "namespace")?);
        if !namespace::host_shape(&host) {
            return Err(pin_refusal(&at, PinWhy::Host));
        }
        if !namespace::file_shape(&destination) {
            return Err(pin_refusal(&at, PinWhy::Namespace));
        }
        if destination == Path::new(SHIM_DESTINATION)
            || FIXED_DESTINATIONS
                .iter()
                .any(|fixed| destination == Path::new(fixed))
            || RESERVED_PREFIXES
                .iter()
                .any(|prefix| destination.starts_with(prefix))
        {
            return Err(pin_refusal(&at, PinWhy::Reserved));
        }
        if out.iter().any(|seen| seen.namespace == destination) {
            return Err(pin_refusal(&at, PinWhy::Duplicate));
        }
        let sha256 = digest_bytes(&string(row, &at, "sha256")?)
            .ok_or_else(|| pin_refusal(&at, PinWhy::Digest))?;
        out.push(RuntimeFile {
            host,
            namespace: destination,
            sha256,
        });
    }
    Ok(out)
}

fn directories(pins: &toml::Table) -> Result<Vec<PathBuf>, ProfileError> {
    let rows = array(pins, "pins", "namespace_directories")?;
    if rows.len() > MAX_MOUNTS {
        return Err(pin_refusal(
            "namespace_directories",
            PinWhy::Count {
                found: rows.len(),
                limit: MAX_MOUNTS,
            },
        ));
    }
    let mut out: Vec<PathBuf> = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let at = format!("pins.namespace_directories[{index}]");
        let toml::Value::String(path) = row else {
            return Err(ProfileError::WrongType { path: at });
        };
        let path = PathBuf::from(path);
        if !namespace::directory_shape(&path) {
            return Err(pin_refusal(&at, PinWhy::Namespace));
        }
        if out.contains(&path) {
            return Err(pin_refusal(&at, PinWhy::Duplicate));
        }
        out.push(path);
    }
    Ok(out)
}

/// The directories the plan will create are acquired too (review P2b-2): the declared ones and
/// every parent of every destination the plan mounts — the runtime files, the workload's fixed
/// destinations and the shim's. That set must fit the namespace's mounts, and no runtime file may
/// be one of its directories (a file where the plan creates a directory, or above another file).
fn derived(files: &[RuntimeFile], declared: &[PathBuf]) -> Result<(), ProfileError> {
    let destinations = files
        .iter()
        .map(|file| file.namespace.as_path())
        .chain(FIXED_DESTINATIONS.iter().map(Path::new))
        .chain(std::iter::once(Path::new(SHIM_DESTINATION)));
    let mut directories: BTreeSet<&Path> = declared.iter().map(PathBuf::as_path).collect();
    for destination in destinations {
        directories.extend(
            destination
                .ancestors()
                .skip(1)
                .filter(|parent| *parent != Path::new("/")),
        );
    }
    if directories.len() > MAX_MOUNTS {
        return Err(pin_refusal(
            "namespace_directories",
            PinWhy::Count {
                found: directories.len(),
                limit: MAX_MOUNTS,
            },
        ));
    }
    match files
        .iter()
        .position(|file| directories.contains(file.namespace.as_path()))
    {
        Some(index) => Err(pin_refusal(
            &format!("pins.runtime_files[{index}]"),
            PinWhy::Directory,
        )),
        None => Ok(()),
    }
}

/// The `[reviewed]` table: exactly the expectation's and the review's digests, distinct.
fn reviewed(table: &toml::Table) -> Result<Reviewed, ProfileError> {
    let reviewed = sub_table(table, "", "reviewed")?;
    only(
        reviewed,
        "reviewed",
        &["expectation_sha256", "review_sha256"],
    )?;
    let digest = |name: &'static str| -> Result<String, ProfileError> {
        let value = string(reviewed, "reviewed", name)?;
        Sha256Digest::parse(&value).map_err(|_| ProfileError::Reviewed {
            name,
            why: ReviewedWhy::Digest,
        })?;
        Ok(value)
    };
    let (expectation, review) = (digest("expectation_sha256")?, digest("review_sha256")?);
    if expectation == review {
        return Err(ProfileError::Reviewed {
            name: "review_sha256",
            why: ReviewedWhy::Same,
        });
    }
    Ok(Reviewed {
        expectation,
        review,
    })
}

/// Which reviewed record to read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Which {
    Expectation,
    Review,
}

/// Why a reviewed record could not be read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedError {
    /// The directory or the file is absent.
    NotInstalled,
    /// Something is there and it is not the operator's private record (0700 directory, 0600 file).
    Custody,
    /// Larger than [`MAX_REVIEWED_BYTES`].
    TooLarge,
    /// The bytes are not the record the profile names.
    Mismatch,
    Io,
}

/// Read the reviewed record `which` names under custody, at most [`MAX_REVIEWED_BYTES`], and return
/// it only if its bytes hash to the digest the profile declares for it (B14a-2a). The profile's
/// digest is the only name a caller can ask for: there is no door to read an arbitrary record.
///
/// # Errors
/// Each [`ReviewedError`], named.
pub fn read_reviewed(profile: &Profile, which: Which) -> Result<Vec<u8>, ReviewedError> {
    let declared = match which {
        Which::Expectation => &profile.declared.reviewed.expectation,
        Which::Review => &profile.declared.reviewed.review,
    };
    let name = declared
        .strip_prefix("sha256:")
        .ok_or(ReviewedError::Mismatch)?;
    let held = match PrivateDirectory::open(&profile.directory.join(REVIEWED_DIRECTORY)) {
        Ok(held) => held,
        Err(DirectoryError::NotFound) => return Err(ReviewedError::NotInstalled),
        Err(DirectoryError::Custody) => return Err(ReviewedError::Custody),
        Err(DirectoryError::Io(_)) => return Err(ReviewedError::Io),
    };
    let bytes = match held.read(name, MAX_REVIEWED_BYTES) {
        Ok(bytes) => bytes,
        Err(FileError::NotFound) => return Err(ReviewedError::NotInstalled),
        Err(FileError::Custody) => return Err(ReviewedError::Custody),
        Err(FileError::TooLarge) => return Err(ReviewedError::TooLarge),
        Err(FileError::Io(_)) => return Err(ReviewedError::Io),
    };
    if super::evidence::digest(&bytes) != *declared {
        return Err(ReviewedError::Mismatch);
    }
    Ok(bytes)
}

fn digest_text(pins: &toml::Table, key: &str) -> Result<String, ProfileError> {
    let value = string(pins, "pins", key)?;
    Sha256Digest::parse(&value).map_err(|_| pin_refusal(key, PinWhy::Digest))?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    const ID: &str = "28e00000-0000-4000-8000-000000000001";
    const ID2: &str = "28e00000-0000-4000-8000-000000000002";
    const HEX: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    const HEX2: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    /// coreutils `sha256sum` of `EXPECTATION` and `REVIEW`.
    const EXP: &str = "sha256:7a1f9aa11864adf4fdc42e57bccf14c9721c288f749cf314525c8c183df64f13";
    const REV: &str = "sha256:7676d865aaf08640fa14c6526535f9e8e47da1a955bdf479688c5f3800423740";
    const EXPECTATION: &[u8] = b"the frozen expectation\n";
    const REVIEW: &[u8] = b"its independent review\n";

    /// A valid declaration: two workspaces and every pin kind.
    fn valid() -> String {
        format!(
            r#"schema = "hee3.class-profile/1"
class = "rust-library-change/1"

[[workspace]]
id = "{ID}"
baseline = "base-1"
baseline_digest = "{HEX}"
protected = "protected-1"
protected_digest = "{HEX2}"

[[workspace]]
id = "{ID2}"
baseline = "base-2"
baseline_digest = "{HEX2}"
protected = "protected-2"
protected_digest = "{HEX}"

[pins]
compiler = {{ host = "/opt/rust/bin/rustc", sha256 = "{HEX2}" }}
shim = {{ host = "/opt/hee/namespace-shim", sha256 = "{HEX}" }}
runtime_files = [
  {{ host = "/opt/rust/lib/libstd.so", namespace = "/toolchain/lib/libstd.so", sha256 = "{HEX}" }},
  {{ host = "/usr/bin/cc", namespace = "/usr/bin/cc", sha256 = "{HEX2}" }},
]
namespace_directories = ["/toolchain/lib", "/usr/lib64"]
busctl_sha256 = "{HEX}"
systemd_run_sha256 = "{HEX2}"

[reviewed]
expectation_sha256 = "{EXP}"
review_sha256 = "{REV}"
"#
        )
    }

    /// The refusal `text` composes to; a text that composes fails the calling test.
    fn refused(text: &str) -> ProfileError {
        let composed = compose(text.as_bytes());
        assert!(composed.is_err(), "composed: {composed:?}");
        composed.err().unwrap_or(ProfileError::Encoding)
    }

    fn with(old: &str, new: &str) -> String {
        let text = valid();
        assert_eq!(text.matches(old).count(), 1, "anchor {old:?}");
        text.replacen(old, new, 1)
    }

    /// B14-P2b · a valid declaration composes into exactly what it declares, digests as bytes.
    #[test]
    fn a_valid_profile_composes_into_its_declaration() -> Result<(), ProfileError> {
        let declared = compose(valid().as_bytes())?;
        assert_eq!(
            declared.workspaces,
            vec![
                Workspace {
                    id: ID.into(),
                    baseline: "base-1".into(),
                    baseline_digest: HEX.into(),
                    protected: "protected-1".into(),
                    protected_digest: HEX2.into(),
                },
                Workspace {
                    id: ID2.into(),
                    baseline: "base-2".into(),
                    baseline_digest: HEX2.into(),
                    protected: "protected-2".into(),
                    protected_digest: HEX.into(),
                },
            ]
        );
        let ramp: [u8; 32] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
            0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67,
            0x89, 0xab, 0xcd, 0xef,
        ];
        assert_eq!(
            (declared.compiler, declared.shim),
            (
                HostPin {
                    host: "/opt/rust/bin/rustc".into(),
                    sha256: ramp
                },
                HostPin {
                    host: "/opt/hee/namespace-shim".into(),
                    sha256: [0; 32]
                }
            )
        );
        assert_eq!(
            declared.runtime_files,
            vec![
                RuntimeFile {
                    host: "/opt/rust/lib/libstd.so".into(),
                    namespace: "/toolchain/lib/libstd.so".into(),
                    sha256: [0; 32],
                },
                RuntimeFile {
                    host: "/usr/bin/cc".into(),
                    namespace: "/usr/bin/cc".into(),
                    sha256: ramp,
                },
            ]
        );
        assert_eq!(
            declared.namespace_directories,
            vec![PathBuf::from("/toolchain/lib"), PathBuf::from("/usr/lib64")]
        );
        assert_eq!(
            (
                declared.busctl_sha256.as_str(),
                declared.systemd_run_sha256.as_str()
            ),
            (HEX, HEX2)
        );
        Ok(())
    }

    /// Every document-level refusal names its cause and its key path.
    #[test]
    fn syntax_positions_and_encoding_are_named() {
        // Positions are the toml crate's own: its message prints "line L, column C".
        for text in [
            "\u{fffd}",
            "a = 1\nb = \n",
            "a = \"é\" x\n",
            "[t]\nk = [1,\n",
        ] {
            let parsed = text.parse::<toml::Table>();
            assert!(parsed.is_err(), "{text:?} parsed");
            let message = parsed
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
            let (line, column) = (numbers[0], numbers[1]);
            assert_eq!(
                refused(text),
                ProfileError::Syntax { line, column },
                "{message}"
            );
        }
        assert_eq!(compose(b"\xff"), Err(ProfileError::Encoding));
        assert_eq!(position("ab\ncd", 4), (2, 2));
        assert_eq!(position("ab\n", 3), (2, 1));
        assert_eq!(
            refused(&with(
                "class = \"rust-library-change/1\"\n",
                "class = \"rust-library-change/1\"\nclass = \"x\"\n"
            )),
            ProfileError::Syntax { line: 3, column: 1 }
        );
    }

    /// Every key-level refusal names its cause and its key path.
    #[test]
    fn document_refusals_name_their_key() {
        let path = |p: &str| p.to_owned();
        for (text, expected) in [
            (
                with("schema = ", "extra = 1\nschema = "),
                ProfileError::UnknownKey {
                    path: path("extra"),
                },
            ),
            (
                with("busctl_sha256", "stray = 1\nbusctl_sha256"),
                ProfileError::UnknownKey {
                    path: path("pins.stray"),
                },
            ),
            (
                with("baseline = \"base-1\"", "baseline = \"base-1\"\nextra = 1"),
                ProfileError::UnknownKey {
                    path: path("workspace[0].extra"),
                },
            ),
            (
                with(
                    "namespace = \"/usr/bin/cc\",",
                    "namespace = \"/usr/bin/cc\", odd = 1,",
                ),
                ProfileError::UnknownKey {
                    path: path("pins.runtime_files[1].odd"),
                },
            ),
            (
                with(
                    "shim = { host = \"/opt/hee/namespace-shim\", sha256 = \"",
                    "shim = { size = 1, host = \"/opt/hee/namespace-shim\", sha256 = \"",
                ),
                ProfileError::UnknownKey {
                    path: path("pins.shim.size"),
                },
            ),
            (
                with("schema = \"hee3.class-profile/1\"\n", ""),
                ProfileError::MissingKey {
                    path: path("schema"),
                },
            ),
            (
                with("protected = \"protected-2\"\n", ""),
                ProfileError::MissingKey {
                    path: path("workspace[1].protected"),
                },
            ),
            (
                with("schema = \"hee3.class-profile/1\"", "schema = 1"),
                ProfileError::WrongType {
                    path: path("schema"),
                },
            ),
            (
                with(
                    "namespace_directories = [\"/toolchain/lib\", \"/usr/lib64\"]",
                    "namespace_directories = [1]",
                ),
                ProfileError::WrongType {
                    path: path("pins.namespace_directories[0]"),
                },
            ),
            (
                with(
                    "schema = \"hee3.class-profile/1\"",
                    "schema = \"hee3.class-profile/2\"",
                ),
                ProfileError::Schema {
                    found: path("hee3.class-profile/2"),
                },
            ),
            (
                with(
                    "class = \"rust-library-change/1\"",
                    "class = \"rust-library-change/2\"",
                ),
                ProfileError::Class {
                    found: path("rust-library-change/2"),
                },
            ),
        ] {
            assert_eq!(refused(&text), expected, "{text}");
        }
    }

    /// Every workspace refusal names its row, id, field and reason.
    #[test]
    fn workspace_refusals_name_the_row_and_field() {
        let at = |index, id: &str, field, why| ProfileError::Workspace {
            index,
            id: id.to_owned(),
            field,
            why,
        };
        // The lowest byte a name may hold is 0x20: a space is a name, 0x1f is not (P2b mutants).
        assert!(compose(with("baseline = \"base-1\"", "baseline = \"base 1\"").as_bytes()).is_ok());
        assert_eq!(
            refused(&with(
                "baseline = \"base-1\"",
                "baseline = \"base\\u001f1\""
            )),
            at(0, ID, Field::Baseline, WorkspaceWhy::Component)
        );
        let none: String = valid()
            .split("[[workspace]]")
            .enumerate()
            .filter_map(|(index, part)| (index == 0).then_some(part.to_owned()))
            .chain(std::iter::once(
                valid()[valid().find("[pins]").unwrap_or(0)..].to_owned(),
            ))
            .collect();
        assert_eq!(
            refused(&none.replace("[pins]", "workspace = []\n[pins]")),
            ProfileError::Workspaces { count: 0 }
        );
        for (text, expected) in [
            (
                with(&format!("id = \"{ID}\""), "id = \"not-a-uuid\""),
                at(0, "not-a-uuid", Field::Id, WorkspaceWhy::Uuid),
            ),
            (
                with(&format!("id = \"{ID2}\""), &format!("id = \"{ID}\"")),
                at(1, ID, Field::Id, WorkspaceWhy::DuplicateId),
            ),
            (
                with("baseline = \"base-1\"", "baseline = \"..\""),
                at(0, ID, Field::Baseline, WorkspaceWhy::Component),
            ),
            (
                with("baseline = \"base-1\"", "baseline = \"a/b\""),
                at(0, ID, Field::Baseline, WorkspaceWhy::Component),
            ),
            (
                with("baseline = \"base-1\"", "baseline = \"a\\u0001\""),
                at(0, ID, Field::Baseline, WorkspaceWhy::Component),
            ),
            (
                with("baseline = \"base-1\"", "baseline = \"\""),
                at(0, ID, Field::Baseline, WorkspaceWhy::Component),
            ),
            (
                with(
                    "protected = \"protected-2\"",
                    "protected = \"profile.toml\"",
                ),
                at(1, ID2, Field::Protected, WorkspaceWhy::Reserved),
            ),
            (
                with("baseline = \"base-2\"", "baseline = \"protected-1\""),
                at(1, ID2, Field::Baseline, WorkspaceWhy::DuplicateDirectory),
            ),
            (
                with("protected = \"protected-1\"", "protected = \"base-1\""),
                at(0, ID, Field::Protected, WorkspaceWhy::Aliased),
            ),
            (
                with(
                    &format!("baseline_digest = \"{HEX2}\""),
                    "baseline_digest = \"sha256:00\"",
                ),
                at(1, ID2, Field::BaselineDigest, WorkspaceWhy::Digest),
            ),
            (
                with(
                    &format!("protected_digest = \"{HEX2}\""),
                    &format!("protected_digest = \"{}\"", HEX2.to_uppercase()),
                ),
                at(0, ID, Field::ProtectedDigest, WorkspaceWhy::Digest),
            ),
        ] {
            assert_eq!(refused(&text), expected, "{text}");
        }
    }

    /// Every pin refusal names the pin and why; the runtime file count is the namespace's mounts
    /// less the workload's own two, at the limit and one past it.
    #[test]
    fn pin_refusals_name_the_pin() {
        let pin = |name: &str, why| ProfileError::Pin {
            name: name.to_owned(),
            why,
        };
        let file = "pins.runtime_files[1]";
        for (text, expected) in [
            (
                with("host = \"/opt/rust/bin/rustc\"", "host = \"opt/rust\""),
                pin("compiler", PinWhy::Host),
            ),
            (
                with("host = \"/opt/hee/namespace-shim\"", "host = \"/opt/../x\""),
                pin("shim", PinWhy::Host),
            ),
            (
                with("host = \"/usr/bin/cc\"", "host = \"cc\""),
                pin(file, PinWhy::Host),
            ),
            (
                with("namespace = \"/usr/bin/cc\"", "namespace = \"/etc/cc\""),
                pin(file, PinWhy::Namespace),
            ),
            (
                with(
                    "namespace = \"/usr/bin/cc\"",
                    "namespace = \"/toolchain/bin/rustc\"",
                ),
                pin(file, PinWhy::Reserved),
            ),
            (
                with(
                    "namespace = \"/usr/bin/cc\"",
                    "namespace = \"/shim/namespace-shim\"",
                ),
                pin(file, PinWhy::Reserved),
            ),
            (
                with("namespace = \"/usr/bin/cc\"", "namespace = \"/frozen/x\""),
                pin(file, PinWhy::Reserved),
            ),
            (
                with("namespace = \"/usr/bin/cc\"", "namespace = \"/work/bin/x\""),
                pin(file, PinWhy::Reserved),
            ),
            (
                with(
                    "namespace = \"/usr/bin/cc\"",
                    "namespace = \"/toolchain/lib/libstd.so\"",
                ),
                pin(file, PinWhy::Duplicate),
            ),
            (
                with(
                    &format!("\"/usr/bin/cc\", sha256 = \"{HEX2}\""),
                    "\"/usr/bin/cc\", sha256 = \"00\"",
                ),
                pin(file, PinWhy::Digest),
            ),
            (
                with(
                    &format!("rustc\", sha256 = \"{HEX2}\""),
                    "rustc\", sha256 = \"sha256:zz\"",
                ),
                pin("compiler", PinWhy::Digest),
            ),
            (
                with(
                    "\"/toolchain/lib\", \"/usr/lib64\"",
                    "\"/toolchain/lib\", \"/etc\"",
                ),
                pin("pins.namespace_directories[1]", PinWhy::Namespace),
            ),
            (
                with(
                    "\"/toolchain/lib\", \"/usr/lib64\"",
                    "\"/toolchain/lib\", \"/toolchain/lib\"",
                ),
                pin("pins.namespace_directories[1]", PinWhy::Duplicate),
            ),
            (
                with(
                    &format!("busctl_sha256 = \"{HEX}\""),
                    "busctl_sha256 = \"x\"",
                ),
                pin("busctl_sha256", PinWhy::Digest),
            ),
            (
                with(
                    &format!("systemd_run_sha256 = \"{HEX2}\""),
                    "systemd_run_sha256 = \"x\"",
                ),
                pin("systemd_run_sha256", PinWhy::Digest),
            ),
        ] {
            assert_eq!(refused(&text), expected, "{text}");
        }
    }

    /// The runtime file count is the namespace's mounts less the workload's own two, at the limit
    /// and one past it.
    #[test]
    fn runtime_files_are_bounded_by_the_namespaces_mounts() {
        let pin = |name: &str, why| ProfileError::Pin {
            name: name.to_owned(),
            why,
        };
        assert_eq!((MAX_MOUNTS, MAX_RUNTIME_FILES), (512, 510));
        let files = |count: usize| {
            let rows: Vec<String> = (0..count)
                .map(|index| format!("{{ host = \"/h/{index}\", namespace = \"/usr/bin/f{index}\", sha256 = \"{HEX}\" }}"))
                .collect();
            let start = valid();
            let begin = start.find("runtime_files = [").unwrap_or(0);
            let end = start[begin..].find("]\n").map_or(0, |end| begin + end + 2);
            format!(
                "{}runtime_files = [{}]\n{}",
                &start[..begin],
                rows.join(", "),
                &start[end..]
            )
        };
        assert!(compose(files(MAX_RUNTIME_FILES).as_bytes()).is_ok());
        assert_eq!(
            refused(&files(MAX_RUNTIME_FILES + 1)),
            pin(
                "runtime_files",
                PinWhy::Count {
                    found: 511,
                    limit: 510
                }
            )
        );
    }

    fn private(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "hee3-class-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos())
        ));
        assert!(fs::DirBuilder::new().mode(0o700).create(&path).is_ok());
        path
    }

    fn write(path: &Path, bytes: &[u8], mode: u32) {
        assert!(fs::write(path, bytes).is_ok());
        assert!(fs::set_permissions(path, fs::Permissions::from_mode(mode)).is_ok());
    }

    /// B14-P2b · `read` is the only I/O: absence of the directory or of the file is
    /// `NotInstalled`; custody, size and every compose refusal are `Refused` and named.
    #[test]
    fn read_names_absence_custody_and_size() {
        let root = private("read");
        let absent = root.join("absent");
        assert_eq!(read(&absent), Err(Unready::NotInstalled));
        let empty = root.join("empty");
        assert!(fs::DirBuilder::new().mode(0o700).create(&empty).is_ok());
        assert_eq!(read(&empty), Err(Unready::NotInstalled));
        let good = root.join("good");
        assert!(fs::DirBuilder::new().mode(0o700).create(&good).is_ok());
        write(&good.join(PROFILE_FILE), valid().as_bytes(), 0o600);
        // B14a-1c · the digest is over the exact bytes read; the literal is coreutils `sha256sum` of
        // `valid()` rendered outside Rust (the constants substituted, `{{`/`}}` unescaped).
        assert_eq!(
            read(&good).map(|profile| (
                profile.directory,
                profile.declared.workspaces.len(),
                profile.digest
            )),
            Ok((
                good.clone(),
                2,
                "sha256:146b45de5f8da3028b197e3c6e0cffd45bdd990c209a31ee947fd2219e3e62d9"
                    .to_owned()
            ))
        );
        write(&good.join(PROFILE_FILE), valid().as_bytes(), 0o644);
        assert_eq!(
            read(&good),
            Err(Unready::Refused(ProfileError::Read {
                what: "file custody"
            }))
        );
        let wide = root.join("wide");
        assert!(fs::DirBuilder::new().mode(0o755).create(&wide).is_ok());
        assert!(fs::set_permissions(&wide, fs::Permissions::from_mode(0o755)).is_ok());
        assert_eq!(
            read(&wide),
            Err(Unready::Refused(ProfileError::Read {
                what: "directory custody"
            }))
        );
        let big = root.join("big");
        assert!(fs::DirBuilder::new().mode(0o700).create(&big).is_ok());
        let padded = format!("{}#{}\n", valid(), " ".repeat(65_536));
        write(&big.join(PROFILE_FILE), padded.as_bytes(), 0o600);
        assert_eq!(
            read(&big),
            Err(Unready::Refused(ProfileError::Read {
                what: "file too large"
            }))
        );
        let bad = root.join("bad");
        assert!(fs::DirBuilder::new().mode(0o700).create(&bad).is_ok());
        write(&bad.join(PROFILE_FILE), b"schema = 1\n", 0o600);
        assert_eq!(
            read(&bad),
            Err(Unready::Refused(ProfileError::WrongType {
                path: "schema".into()
            }))
        );
        assert_eq!(
            Unready::NotInstalled.constraint(),
            "class profile not installed"
        );
        assert_eq!(
            Unready::Refused(ProfileError::Encoding).constraint(),
            "class profile refused: Encoding"
        );
        assert!(fs::remove_dir_all(&root).is_ok());
    }

    /// B14-P2b review 2, 3 · the directories the plan creates are acquired too: a runtime file where
    /// the plan makes a directory — a fixed destination's parent, a declared directory, the parent of
    /// another runtime file — is `Directory`; declared directories past the mounts are `Count` with
    /// both numbers, and so is a set whose derived parents carry it past them.
    #[test]
    fn the_derived_directories_are_bounded_and_never_a_file() {
        let pin = |name: &str, why| ProfileError::Pin {
            name: name.to_owned(),
            why,
        };
        let file = "pins.runtime_files[1]";
        for (text, expected) in [
            (
                with(
                    "namespace = \"/usr/bin/cc\"",
                    "namespace = \"/toolchain/bin\"",
                ),
                pin(file, PinWhy::Directory),
            ),
            (
                with(
                    "namespace = \"/usr/bin/cc\"",
                    "namespace = \"/toolchain/lib\"",
                ),
                pin(file, PinWhy::Directory),
            ),
            (
                with(
                    "namespace = \"/toolchain/lib/libstd.so\"",
                    "namespace = \"/usr/bin/cc/x\"",
                ),
                ProfileError::Pin {
                    name: "pins.runtime_files[1]".into(),
                    why: PinWhy::Directory,
                },
            ),
            (
                with(
                    &format!(
                        "{{ host = \"/usr/bin/cc\", namespace = \"/usr/bin/cc\", sha256 = \"{HEX2}\" }}"
                    ),
                    "[1, 2]",
                ),
                ProfileError::WrongType {
                    path: "pins.runtime_files[1]".into(),
                },
            ),
            (
                with(
                    &format!("namespace-shim\", sha256 = \"{HEX}\""),
                    "namespace-shim\", sha256 = \"x\"",
                ),
                pin("shim", PinWhy::Digest),
            ),
        ] {
            assert_eq!(refused(&text), expected, "{text}");
        }
        // The workspace list as something other than a list of tables: two sites, two paths.
        let text = valid();
        let (head, pins) = (
            &text[..text.find("[[workspace]]").unwrap_or(0)],
            &text[text.find("[pins]").unwrap_or(0)..],
        );
        for (rows, path) in [
            ("workspace = 1", "workspace"),
            ("workspace = [1]", "workspace[0]"),
        ] {
            assert_eq!(
                refused(&format!("{head}{rows}\n{pins}")),
                ProfileError::WrongType { path: path.into() }
            );
        }
    }

    /// The declared directory count and the derived set's, each at its bound, with both numbers.
    #[test]
    fn the_declared_and_derived_directory_counts_are_bounded() {
        let pin = |name: &str, why| ProfileError::Pin {
            name: name.to_owned(),
            why,
        };
        let declared = |count: usize| {
            let rows: Vec<String> = (0..count)
                .map(|index| format!("\"/usr/lib64/d{index}\""))
                .collect();
            with(
                "namespace_directories = [\"/toolchain/lib\", \"/usr/lib64\"]",
                &format!("namespace_directories = [{}]", rows.join(", ")),
            )
        };
        assert_eq!(
            refused(&declared(MAX_MOUNTS + 1)),
            pin(
                "namespace_directories",
                PinWhy::Count {
                    found: MAX_MOUNTS + 1,
                    limit: MAX_MOUNTS
                }
            )
        );
        // Counted by hand from the rule: the fixed destinations add six parents (/toolchain/bin,
        // /toolchain, /frozen/source, /frozen, /frozen/bin, /shim) and the two runtime files three
        // (/toolchain/lib, /usr/bin, /usr), so n declared directories derive n + 9: 503 fit
        // exactly, 504 do not.
        assert!(compose(declared(503).as_bytes()).is_ok());
        // Exactly the mounts declared: not the declared list's refusal (it admits 512), but the
        // derived set's, 512 + 9 (P2b mutants: `>` against `>=` at the declared bound).
        assert_eq!(
            refused(&declared(MAX_MOUNTS)),
            pin(
                "namespace_directories",
                PinWhy::Count {
                    found: MAX_MOUNTS + 9,
                    limit: MAX_MOUNTS
                }
            )
        );
        assert_eq!(
            refused(&declared(504)),
            pin(
                "namespace_directories",
                PinWhy::Count {
                    found: 513,
                    limit: MAX_MOUNTS
                }
            )
        );
    }

    /// B14-P2c · the one screen over its four states: declared admits, undeclared under a read
    /// profile and anything under a refused profile do not, and no profile admits.
    #[test]
    fn screen_decides_by_the_profile_s_state() -> Result<(), ProfileError> {
        let read = Ok(Profile {
            declared: compose(valid().as_bytes())?,
            directory: PathBuf::from("/p"),
            digest: String::new(),
        });
        assert_eq!(screen(&read, ID), Ok(()));
        assert_eq!(screen(&read, ID2), Ok(()));
        assert_eq!(
            screen(&read, "28e00000-0000-4000-8000-000000000003"),
            Err(Screen::WorkspaceNotInstalled)
        );
        assert_eq!(
            screen(&Err(Unready::Refused(ProfileError::Encoding)), ID),
            Err(Screen::ProfileRefused)
        );
        assert_eq!(screen(&Err(Unready::NotInstalled), ID), Ok(()));
        assert_eq!(
            (
                Screen::WorkspaceNotInstalled.constraint(),
                Screen::ProfileRefused.constraint()
            ),
            ("workspace not installed", "class profile refused")
        );
        assert_eq!(
            (
                Screen::WorkspaceNotInstalled.message(),
                Screen::ProfileRefused.message()
            ),
            (
                "the installed class profile declares no such workspace",
                "the installed class profile was refused at start"
            )
        );
        Ok(())
    }

    /// B14a-2a · `[reviewed]` is required, holds exactly two digests, and a review naming the
    /// expectation itself is refused.
    #[test]
    fn the_reviewed_table_names_two_distinct_digests() -> Result<(), ProfileError> {
        let declared = compose(valid().as_bytes())?;
        assert_eq!(
            declared.reviewed,
            Reviewed {
                expectation: EXP.to_owned(),
                review: REV.to_owned()
            }
        );
        let without = valid();
        let without = &without[..without.find("\n[reviewed]").unwrap_or(without.len())];
        assert_eq!(
            refused(without),
            ProfileError::MissingKey {
                path: "reviewed".into()
            }
        );
        assert_eq!(
            refused(&format!("{}extra = 1\n", valid())),
            ProfileError::UnknownKey {
                path: "reviewed.extra".into()
            }
        );
        assert_eq!(
            refused(&with(
                &format!("review_sha256 = \"{REV}\""),
                "review_sha256 = \"x\""
            )),
            ProfileError::Reviewed {
                name: "review_sha256",
                why: ReviewedWhy::Digest
            }
        );
        assert_eq!(
            refused(&with(
                &format!("expectation_sha256 = \"{EXP}\""),
                "expectation_sha256 = \"x\""
            )),
            ProfileError::Reviewed {
                name: "expectation_sha256",
                why: ReviewedWhy::Digest
            }
        );
        assert_eq!(
            refused(&with(
                &format!("review_sha256 = \"{REV}\""),
                &format!("review_sha256 = \"{EXP}\"")
            )),
            ProfileError::Reviewed {
                name: "review_sha256",
                why: ReviewedWhy::Same
            }
        );
        Ok(())
    }

    /// B14a-2a · a reviewed record is returned only when its bytes hash to the digest the profile
    /// names; absence, custody, size and a substituted record are each refused by name.
    #[test]
    fn a_reviewed_record_is_read_only_as_the_record_the_profile_names() -> Result<(), ProfileError>
    {
        let root = private("reviewed");
        let profile = Profile {
            declared: compose(valid().as_bytes())?,
            directory: root.clone(),
            digest: String::new(),
        };
        assert_eq!(
            read_reviewed(&profile, Which::Expectation),
            Err(ReviewedError::NotInstalled)
        );
        let store = root.join(REVIEWED_DIRECTORY);
        assert!(fs::DirBuilder::new().mode(0o700).create(&store).is_ok());
        let name = |digest: &str| digest.trim_start_matches("sha256:").to_owned();
        assert_eq!(
            read_reviewed(&profile, Which::Review),
            Err(ReviewedError::NotInstalled)
        );
        write(&store.join(name(EXP)), EXPECTATION, 0o600);
        write(&store.join(name(REV)), REVIEW, 0o600);
        assert_eq!(
            read_reviewed(&profile, Which::Expectation),
            Ok(EXPECTATION.to_vec())
        );
        assert_eq!(read_reviewed(&profile, Which::Review), Ok(REVIEW.to_vec()));
        // The review's bytes under the expectation's name: substituted, not the named record.
        write(&store.join(name(EXP)), REVIEW, 0o600);
        assert_eq!(
            read_reviewed(&profile, Which::Expectation),
            Err(ReviewedError::Mismatch)
        );
        write(&store.join(name(EXP)), EXPECTATION, 0o644);
        assert_eq!(
            read_reviewed(&profile, Which::Expectation),
            Err(ReviewedError::Custody)
        );
        let big = vec![b'x'; usize::try_from(MAX_REVIEWED_BYTES).unwrap_or(0) + 1];
        write(&store.join(name(REV)), &big, 0o600);
        assert_eq!(
            read_reviewed(&profile, Which::Review),
            Err(ReviewedError::TooLarge)
        );
        assert!(fs::set_permissions(&store, fs::Permissions::from_mode(0o755)).is_ok());
        assert_eq!(
            read_reviewed(&profile, Which::Review),
            Err(ReviewedError::Custody)
        );
        Ok(())
    }
}
