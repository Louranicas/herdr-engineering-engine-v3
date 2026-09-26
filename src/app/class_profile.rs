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
use crate::contracts::receipt::{
    Address, ExpectationV1, Id, Name, RECORD_MEDIA_TYPE, Ref, ReviewV1, Sha, Text, TypedRef,
};
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
/// never carries the answer to its own review; the profile names each record by its full reference
/// and the file carries the digest's hex.
pub const REVIEWED_DIRECTORY: &str = "reviewed";
/// A reviewed record's acquisition bound: the profile's own. Measured against the closure of
/// `fixed-task-execution-003` (the repo's fixture `tests/fixtures/reviewed-003/`): 21 files from
/// 87 B to 65,533 B — the expectation's raw specification, three bytes under this bound. A lane
/// whose specification is larger is refused `Bound` until that decision is taken and recorded
/// beside it (F122: a bound sits next to the record it was fitted to, never quietly widened).
pub const MAX_REVIEWED_BYTES: u64 = MAX_PROFILE_BYTES;
// The bound holds the largest member it was measured against, at compile time.
const _: () = assert!(MAX_REVIEWED_BYTES >= 65_533);

const SCHEMA: &str = "hee3.class-profile/1";
/// The one class this profile schema admits; the plan's `profile_id` is `<class>@<digest>`.
pub const CLASS: &str = "rust-library-change/1";
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
/// review — the review provenance record, which carries the reviewer's verdict over the
/// expectation — each as the full reference the receipt will cite (artifact id, sha256, byte
/// length, media type and its fixed schema id), so the profile's declaration and the receipt's
/// citation are one spelling (B14a-2c-ii). The objects are in [`REVIEWED_DIRECTORY`] and are read
/// through [`read_reviewed`], which returns only bytes of the declared length hashing to the
/// declared digest, or whole through [`read_reviewed_closure`]. The schema each is fixed to is the
/// type's, and [`Reviewed::new`] is the one door for the rest: a `Reviewed` holding a reference
/// under another schema, another media type, no bytes, or one record twice cannot be built.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reviewed {
    expectation: TypedRef<ExpectationV1>,
    review: TypedRef<ReviewV1>,
}

/// Why two typed references are not a [`Reviewed`], and which of the two.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewedRefusal {
    pub which: Which,
    pub why: ReviewedWhy,
}

impl Reviewed {
    /// The two records a class binds, as the receipt will cite them: each of `application/json`
    /// bytes of non-zero length, naming different records by digest and by id.
    ///
    /// # Errors
    /// [`ReviewedRefusal`] naming the reference and the rule (`MediaType`, `Empty`, `Same`).
    pub fn new(
        expectation: TypedRef<ExpectationV1>,
        review: TypedRef<ReviewV1>,
    ) -> Result<Self, ReviewedRefusal> {
        for (which, reference) in [
            (Which::Expectation, expectation.as_ref()),
            (Which::Review, review.as_ref()),
        ] {
            if reference.media_type.as_str() != RECORD_MEDIA_TYPE {
                return Err(ReviewedRefusal {
                    which,
                    why: ReviewedWhy::MediaType,
                });
            }
            if reference.byte_length == 0 {
                return Err(ReviewedRefusal {
                    which,
                    why: ReviewedWhy::Empty,
                });
            }
        }
        if review.as_ref().sha256 == expectation.as_ref().sha256
            || review.as_ref().artifact_id == expectation.as_ref().artifact_id
        {
            return Err(ReviewedRefusal {
                which: Which::Review,
                why: ReviewedWhy::Same,
            });
        }
        Ok(Self {
            expectation,
            review,
        })
    }

    #[must_use]
    pub const fn expectation(&self) -> &TypedRef<ExpectationV1> {
        &self.expectation
    }

    #[must_use]
    pub const fn review(&self) -> &TypedRef<ReviewV1> {
        &self.review
    }
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
    pub grant: Grant,
    pub effect: Effect,
}

/// A file beside `profile.toml` the profile declares by name AND digest (B14a-2c-ii-c, R17 round 2
/// F11): read through [`read_declared`], which returns only bytes hashing to `sha256` — the
/// reviewed-reference discipline for the two declared inputs the receipt's grant and effect rows
/// carry, so swapping the file cannot leave the profile identity reading `Matched`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredFile {
    /// One path component under the class directory.
    pub file: String,
    pub sha256: Sha,
}

/// The one grant a class's invocation carries (`GrantV1`): its id, the issuer the operator names,
/// and the authority document the row cites — the receipt's `scope_sha256` is that document's.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grant {
    pub grant_id: Id,
    pub issuer_id: Name,
    pub authority: DeclaredFile,
}

/// The one effect a class's invocation allows (`EffectV1`), under the grant above: its id, its
/// scope text, and the isolation specification the row cites. The runtime is its owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Effect {
    pub effect_id: Name,
    pub scope: Text,
    pub specification: DeclaredFile,
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
    /// A `[reviewed]` entry, at its key path: not a receipt reference, not its fixed schema, or the
    /// review naming the expectation itself.
    Reviewed {
        path: String,
        why: ReviewedWhy,
    },
    /// A `[grant]` or `[effect]` field, at its key path: not the scalar the receipt row needs, or a
    /// file name that is not one component.
    Declared {
        path: String,
        why: DeclaredWhy,
    },
}

/// Why a `[grant]`/`[effect]` field is refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclaredWhy {
    /// Not a v4 UUID.
    Id,
    /// Not a receipt name (1..=128 ASCII).
    Name,
    /// Not a receipt text.
    Text,
    /// Not a `sha256:` digest.
    Digest,
    /// Not one path component (empty, `.`, `..`, or holding `/`, NUL or another control byte).
    FileName,
}

/// Why a `[reviewed]` entry is refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedWhy {
    /// Not a `sha256:` digest.
    Digest,
    /// Not a receipt reference: the artifact id is not a v4 UUID, or the byte length is not a
    /// count.
    Reference,
    /// A byte length of zero: no record has no bytes.
    Empty,
    /// Not `application/json`, the one media type a receipt cites a record under.
    MediaType,
    /// Not the schema the entry is fixed to: `ExpectationV1` for the expectation, `ReviewV1` for
    /// the review.
    Schema,
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
        &[
            "schema",
            "class",
            "workspace",
            "pins",
            "reviewed",
            "grant",
            "effect",
        ],
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
        grant: grant(&table)?,
        effect: effect(&table)?,
    })
}

/// The `[grant]` table: `grant_id`, `issuer_id`, `authority = { file, sha256 }`.
fn grant(table: &toml::Table) -> Result<Grant, ProfileError> {
    let grant = sub_table(table, "", "grant")?;
    only(grant, "grant", &["grant_id", "issuer_id", "authority"])?;
    let refuse = |field: &str, why| ProfileError::Declared {
        path: key_path("grant", field),
        why,
    };
    Ok(Grant {
        grant_id: Id::new(string(grant, "grant", "grant_id")?)
            .map_err(|_| refuse("grant_id", DeclaredWhy::Id))?,
        issuer_id: Name::new(string(grant, "grant", "issuer_id")?)
            .map_err(|_| refuse("issuer_id", DeclaredWhy::Name))?,
        authority: declared_file(grant, "grant", "authority")?,
    })
}

/// The `[effect]` table: `effect_id`, `scope`, `specification = { file, sha256 }`.
fn effect(table: &toml::Table) -> Result<Effect, ProfileError> {
    let effect = sub_table(table, "", "effect")?;
    only(effect, "effect", &["effect_id", "scope", "specification"])?;
    let refuse = |field: &str, why| ProfileError::Declared {
        path: key_path("effect", field),
        why,
    };
    Ok(Effect {
        effect_id: Name::new(string(effect, "effect", "effect_id")?)
            .map_err(|_| refuse("effect_id", DeclaredWhy::Name))?,
        scope: Text::new(string(effect, "effect", "scope")?)
            .map_err(|_| refuse("scope", DeclaredWhy::Text))?,
        specification: declared_file(effect, "effect", "specification")?,
    })
}

/// A `{ file, sha256 }` sub-table: the file one path component, the digest a `sha256:` spelling.
fn declared_file(table: &toml::Table, at: &str, key: &str) -> Result<DeclaredFile, ProfileError> {
    let path = key_path(at, key);
    let entry = sub_table(table, at, key)?;
    only(entry, &path, &["file", "sha256"])?;
    let file = string(entry, &path, "file")?;
    // The one file-name rule this module keeps (`component`): one path component, no control
    // bytes — the same rule the workspace declarations are held to (review 2c-ii-c, LOW).
    if !component(&file) {
        return Err(ProfileError::Declared {
            path: key_path(&path, "file"),
            why: DeclaredWhy::FileName,
        });
    }
    let sha256 = Sha::new(string(entry, &path, "sha256")?).map_err(|_| ProfileError::Declared {
        path: key_path(&path, "sha256"),
        why: DeclaredWhy::Digest,
    })?;
    Ok(DeclaredFile { file, sha256 })
}

/// Read a declared file beside `profile.toml` under the class directory's custody (0700 directory,
/// 0600 file, at most [`MAX_PROFILE_BYTES`]) and return it only if its bytes hash to the digest the
/// profile declares (B14a-2c-ii-c). The refusals are the reviewed records' own.
///
/// # Errors
/// Each [`ReviewedError`], named; `Mismatch` when the bytes are not the declaration's.
pub fn read_declared(profile: &Profile, declared: &DeclaredFile) -> Result<Vec<u8>, ReviewedError> {
    let held = match PrivateDirectory::open(&profile.directory) {
        Ok(held) => held,
        Err(DirectoryError::NotFound) => return Err(ReviewedError::NotInstalled),
        Err(DirectoryError::Custody) => return Err(ReviewedError::Custody),
        Err(DirectoryError::Io(_)) => return Err(ReviewedError::Io),
    };
    let bytes = match held.read(&declared.file, MAX_PROFILE_BYTES) {
        Ok(bytes) => bytes,
        Err(FileError::NotFound) => return Err(ReviewedError::NotInstalled),
        Err(FileError::Custody) => return Err(ReviewedError::Custody),
        Err(FileError::TooLarge) => return Err(ReviewedError::TooLarge),
        Err(FileError::Io(_)) => return Err(ReviewedError::Io),
    };
    if super::evidence::digest(&bytes) != declared.sha256.as_str() {
        return Err(ReviewedError::Mismatch);
    }
    Ok(bytes)
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

/// The `[reviewed]` table: the expectation's and the review's full references, each under its
/// type's schema, made a [`Reviewed`] by its one door — a refusal there is named at the field it
/// concerns.
fn reviewed(table: &toml::Table) -> Result<Reviewed, ProfileError> {
    let reviewed = sub_table(table, "", "reviewed")?;
    only(reviewed, "reviewed", &["expectation", "review"])?;
    let expectation = reference(reviewed, "expectation")?;
    let review = reference(reviewed, "review")?;
    Reviewed::new(expectation, review).map_err(|ReviewedRefusal { which, why }| {
        let at = key_path(
            "reviewed",
            match which {
                Which::Expectation => "expectation",
                Which::Review => "review",
            },
        );
        ProfileError::Reviewed {
            path: match why {
                ReviewedWhy::MediaType => key_path(&at, "media_type"),
                ReviewedWhy::Empty => key_path(&at, "byte_length"),
                ReviewedWhy::Digest
                | ReviewedWhy::Reference
                | ReviewedWhy::Schema
                | ReviewedWhy::Same => at,
            },
            why,
        }
    })
}

/// One reviewed reference under `reviewed.<key>`: the five fields of a receipt reference, each
/// refused at its path, typed to `T`'s schema by the receipt's own `TypedRef` door.
fn reference<T: Address>(reviewed: &toml::Table, key: &str) -> Result<TypedRef<T>, ProfileError> {
    let at = key_path("reviewed", key);
    let table = sub_table(reviewed, "reviewed", key)?;
    only(
        table,
        &at,
        &[
            "artifact_id",
            "sha256",
            "byte_length",
            "media_type",
            "schema_id",
        ],
    )?;
    let refuse = |field: &str, why: ReviewedWhy| ProfileError::Reviewed {
        path: key_path(&at, field),
        why,
    };
    let artifact_id = Id::new(string(table, &at, "artifact_id")?)
        .map_err(|_| refuse("artifact_id", ReviewedWhy::Reference))?;
    let sha256 = Sha::new(string(table, &at, "sha256")?)
        .map_err(|_| refuse("sha256", ReviewedWhy::Digest))?;
    let byte_length = match required(table, &at, "byte_length")? {
        toml::Value::Integer(value) => {
            u32::try_from(*value).map_err(|_| refuse("byte_length", ReviewedWhy::Reference))?
        }
        _ => return Err(wrong(&at, "byte_length")),
    };
    let media_type = Name::new(string(table, &at, "media_type")?)
        .map_err(|_| refuse("media_type", ReviewedWhy::MediaType))?;
    let schema_id = Name::new(string(table, &at, "schema_id")?)
        .map_err(|_| refuse("schema_id", ReviewedWhy::Schema))?;
    TypedRef::new(Ref {
        artifact_id,
        sha256,
        byte_length,
        media_type,
        schema_id,
    })
    .map_err(|_| refuse("schema_id", ReviewedWhy::Schema))
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
    /// A member of the record's closure is absent, of another size or digest, over the bound, or
    /// unreadable, as the walker reports it (`Missing` names the kind of absence, not the member).
    Closure(crate::check::graph::Error),
}

/// Read the reviewed record `which` names under custody, at most [`MAX_REVIEWED_BYTES`], and return
/// it only if its bytes are the declared length and hash to the digest `profile` declares for it
/// (B14a-2a). What comes back always hashes to what was asked for; which reference is asked for is
/// the profile's, and a profile is composed only from the file [`read`] read — a hand-built one (as
/// tests build) can name any record in the store, never forge one.
///
/// # Errors
/// Each [`ReviewedError`], named.
pub fn read_reviewed(profile: &Profile, which: Which) -> Result<Vec<u8>, ReviewedError> {
    let declared = declared(profile, which);
    let name = file_name(declared);
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
    if u64::try_from(bytes.len()).ok() != Some(u64::from(declared.byte_length))
        || super::evidence::digest(&bytes) != declared.sha256.as_str()
    {
        return Err(ReviewedError::Mismatch);
    }
    Ok(bytes)
}

/// The class's `reviewed/` directory as a graph owner (B14a-2c-ii-a): a reference resolves to the
/// file named by its sha256 hex, read under [`MAX_REVIEWED_BYTES`]; the walker itself refuses bytes
/// that are not the reference's length and digest — so the one graph walker that exists walks a
/// reviewed record's whole closure (its specification, assumption, finding and obligation pages)
/// and refuses a member it cannot find (`Missing`), one that is not its reference (`Identity`) or
/// one over the bound (`Bound`).
pub struct ReviewedObjects {
    held: PrivateDirectory,
}

impl ReviewedObjects {
    /// Open the class's `reviewed/` directory under custody.
    ///
    /// # Errors
    /// [`ReviewedError::NotInstalled`] when absent; [`ReviewedError::Custody`] unless 0700 and owned.
    pub fn open(profile: &Profile) -> Result<Self, ReviewedError> {
        match PrivateDirectory::open(&profile.directory.join(REVIEWED_DIRECTORY)) {
            Ok(held) => Ok(Self { held }),
            Err(DirectoryError::NotFound) => Err(ReviewedError::NotInstalled),
            Err(DirectoryError::Custody) => Err(ReviewedError::Custody),
            Err(DirectoryError::Io(_)) => Err(ReviewedError::Io),
        }
    }
}

impl crate::check::graph::Objects for ReviewedObjects {
    fn open(
        &self,
        reference: &crate::contracts::receipt::Ref,
    ) -> Result<Box<dyn std::io::Read + '_>, crate::check::graph::Error> {
        use crate::check::graph::Error;
        let name = reference
            .sha256
            .as_str()
            .strip_prefix("sha256:")
            .ok_or(Error::Identity)?;
        // The walker checks every node's length and digest against its reference itself
        // (`graph::Graph::resolve`); a second check here was a second door on one rule.
        match self.held.read(name, MAX_REVIEWED_BYTES) {
            Ok(bytes) => Ok(Box::new(std::io::Cursor::new(bytes))),
            Err(FileError::NotFound) => Err(Error::Missing),
            Err(FileError::TooLarge) => Err(Error::Bound),
            Err(FileError::Custody | FileError::Io(_)) => Err(Error::Io),
        }
    }
}

/// The whole closure of the reviewed record `root` names, resolved from the class's `reviewed/`
/// directory: every member present, of its declared size and digest, or the walk names the one that
/// is not (B14a-2c-ii-a). The root is the reference the receipt will cite; it must be, whole, the
/// one the profile declares for `which`, or the closure is `Mismatch` — a profile names records,
/// a caller cannot substitute one.
///
/// # Errors
/// [`ReviewedError::Mismatch`] when `root` is not the profile's reference; the directory's own
/// refusals; `Missing`/`Identity`/`Bound`/`Io` from the walk, wrapped as [`ReviewedError::Closure`].
pub fn read_reviewed_closure(
    profile: &Profile,
    which: Which,
    root: &crate::contracts::receipt::Ref,
) -> Result<crate::check::graph::Graph, ReviewedError> {
    let declared = declared(profile, which);
    if root != declared {
        return Err(ReviewedError::Mismatch);
    }
    let objects = ReviewedObjects::open(profile)?;
    crate::check::graph::Graph::resolve(&objects, root).map_err(ReviewedError::Closure)
}

/// The reference the profile declares for `which`, untyped for comparison with what a caller cites.
fn declared(profile: &Profile, which: Which) -> &Ref {
    match which {
        Which::Expectation => profile.declared.reviewed.expectation().as_ref(),
        Which::Review => profile.declared.reviewed.review().as_ref(),
    }
}

/// A reviewed record's file name: the 64 hex of its declared digest.
fn file_name(reference: &Ref) -> &str {
    reference.sha256.as_str().trim_start_matches("sha256:")
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
    /// The 003 lane's reviewed records, as the world produced them (B14a-2c-ii sub-design 5): the
    /// expectation and review roots from `fixed-task-execution-003`'s staging CAS, their ids as the
    /// lane's preparation registry cites them; `EXP`/`REV` are coreutils `sha256sum` of the files.
    const EXP: &str = "sha256:3a7faa5510790c20322ad5829091211eb8c04ab6016e16ec8391433dae3392b9";
    const REV: &str = "sha256:f288225476120254f5c3a93266fc8f2a8763810462fbddd7c62161107cb39adb";
    const EXP_ID: &str = "c220e7ce-0753-47ef-bdac-15710bc4981c";
    const REV_ID: &str = "a47470c5-f11c-4f64-9ac8-6dcf80750ed6";
    const EXPECTATION: &[u8] = include_bytes!(
        "../../tests/fixtures/reviewed-003/3a7faa5510790c20322ad5829091211eb8c04ab6016e16ec8391433dae3392b9"
    );
    const REVIEW: &[u8] = include_bytes!(
        "../../tests/fixtures/reviewed-003/f288225476120254f5c3a93266fc8f2a8763810462fbddd7c62161107cb39adb"
    );
    /// The review's `shared_assumptions` page: a closure member that is neither root.
    const REVIEW_ASSUMPTIONS: &str =
        "8352c1851cfba4e85f26d180f4174bc572cdecc01d08b40d7efeab16f1dc0d81";
    /// The declared grant and effect files: bytes the operator would install beside the profile,
    /// their digests coreutils `sha256sum` of these constants.
    const AUTHORITY: &[u8] = b"{\"authority\":\"WL-U64 fixed workload\",\"issuer\":\"operator\"}\n";
    const SPECIFICATION: &[u8] = b"{\"isolation\":\"bwrap --unshare-all; no network\"}\n";
    const AUTH: &str = "sha256:e4b4fcd5d15d45eed7a553536e73027e7aedbd72f466be6dee168986b6fe1234";
    const SPEC: &str = "sha256:61ac0e7ad368ef48bd6d34e7cd6995ca5ff992693b700b9fccf3f58c8cfc32f1";
    const GRANT_ID: &str = "28f90000-0000-4000-8000-000000000001";
    /// Nodes of the review's and the expectation's closures (the expectation's is inside the
    /// review's), counted by an independent walk over the fixture directory keyed by artifact id
    /// as the graph is: 22 references over 21 files, because the review's empty finding and
    /// obligation pages are one byte string under two ids.
    const REVIEW_CLOSURE: usize = 22;
    const EXPECTATION_CLOSURE: usize = 5;

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
expectation = {{ artifact_id = "{EXP_ID}", sha256 = "{EXP}", byte_length = {EXP_LEN}, media_type = "application/json", schema_id = "{EXP_SCHEMA}" }}
review = {{ artifact_id = "{REV_ID}", sha256 = "{REV}", byte_length = {REV_LEN}, media_type = "application/json", schema_id = "{REV_SCHEMA}" }}

[grant]
grant_id = "{GRANT_ID}"
issuer_id = "operator"
authority = {{ file = "authority.json", sha256 = "{AUTH}" }}

[effect]
effect_id = "fixed-u64-workload-output"
scope = "compile, link and execute the fixed workload in a private bounded scratch; no network"
specification = {{ file = "isolation.json", sha256 = "{SPEC}" }}
"#,
            EXP_LEN = EXPECTATION.len(),
            REV_LEN = REVIEW.len(),
            EXP_SCHEMA = ExpectationV1::SCHEMA_ID,
            REV_SCHEMA = ReviewV1::SCHEMA_ID,
        )
    }

    /// The reference `valid()` declares for one reviewed record, whole.
    fn declared_reference<T: Address>(
        id: &str,
        digest: &str,
        bytes: &[u8],
    ) -> Result<TypedRef<T>, Box<dyn std::error::Error>> {
        Ok(TypedRef::new(Ref {
            artifact_id: Id::new(id)?,
            sha256: Sha::new(digest)?,
            byte_length: u32::try_from(bytes.len())?,
            media_type: Name::new(RECORD_MEDIA_TYPE)?,
            schema_id: Name::new(T::SCHEMA_ID)?,
        })?)
    }

    /// Install the 003 closure into `store` by digest, 0600 each, as the operator would.
    fn install_closure(store: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reviewed-003");
        for entry in fs::read_dir(fixture)? {
            let entry = entry?;
            write(
                &store.join(entry.file_name()),
                &fs::read(entry.path())?,
                0o600,
            );
        }
        Ok(())
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
                "sha256:4eb32e8bcb253168d1a64a6dcf3b4cdc0cae2fe183624101291eda06e2611264"
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

    /// B14a-2a/2c-ii · `[reviewed]` is required and holds exactly the expectation's and the
    /// review's full references under their fixed schemas; a field that is not a reference, a
    /// schema that is not the fixed one, and a review naming the expectation itself (by digest or
    /// by artifact id) are each refused at their key path.
    #[test]
    fn the_reviewed_table_names_two_distinct_references() -> Result<(), Box<dyn std::error::Error>>
    {
        let declared = compose(valid().as_bytes()).map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            declared.reviewed,
            Reviewed::new(
                declared_reference(EXP_ID, EXP, EXPECTATION)?,
                declared_reference(REV_ID, REV, REVIEW)?,
            )
            .map_err(|e| format!("{e:?}"))?
        );
        let review_schema = format!("schema_id = \"{}\"", ReviewV1::SCHEMA_ID);
        let review_line_start = format!("review = {{ artifact_id = \"{REV_ID}\"");
        let review_length = format!("sha256 = \"{REV}\", byte_length = {}", REVIEW.len());
        for (old, new, expected) in [
            (
                review_line_start.clone(),
                "extra = 1\nreview = { artifact_id = 1".to_owned(),
                ProfileError::UnknownKey {
                    path: "reviewed.extra".into(),
                },
            ),
            (
                review_line_start.clone(),
                "review = { artifact_id = 1".to_owned(),
                ProfileError::WrongType {
                    path: "reviewed.review.artifact_id".into(),
                },
            ),
            (
                review_line_start.clone(),
                "review = 1 #".to_owned(),
                ProfileError::WrongType {
                    path: "reviewed.review".into(),
                },
            ),
            (
                review_length.clone(),
                format!("sha256 = \"{REV}\", byte_length = \"{}\"", REVIEW.len()),
                ProfileError::WrongType {
                    path: "reviewed.review.byte_length".into(),
                },
            ),
            (
                format!("media_type = \"application/json\", {review_schema}"),
                review_schema.clone(),
                ProfileError::MissingKey {
                    path: "reviewed.review.media_type".into(),
                },
            ),
            (
                review_schema.clone(),
                format!("{review_schema}, extra = 1"),
                ProfileError::UnknownKey {
                    path: "reviewed.review.extra".into(),
                },
            ),
        ] {
            assert_eq!(refused(&with(&old, &new)), expected, "{old} -> {new}");
        }
        Ok(())
    }

    /// B14a-2c-ii · each field of a reviewed reference is refused at its own path: an id that is
    /// not a v4 UUID, a digest that is not `sha256:` lowercase hex, a length that is negative, past
    /// `u32` or zero, a media type that is not the one a receipt cites, a schema that is not the
    /// entry's type (or no name at all), and a review that is the expectation by digest or by id.
    #[test]
    fn a_reviewed_reference_is_refused_at_its_field() {
        let review_schema = format!("schema_id = \"{}\"", ReviewV1::SCHEMA_ID);
        let review_length = format!("sha256 = \"{REV}\", byte_length = {}", REVIEW.len());
        let expectation_length = format!("sha256 = \"{EXP}\", byte_length = {}", EXPECTATION.len());
        for (old, new, path, why) in [
            (
                format!("artifact_id = \"{REV_ID}\""),
                "artifact_id = \"not-a-uuid\"".to_owned(),
                "reviewed.review.artifact_id",
                ReviewedWhy::Reference,
            ),
            (
                format!("sha256 = \"{REV}\""),
                "sha256 = \"x\"".to_owned(),
                "reviewed.review.sha256",
                ReviewedWhy::Digest,
            ),
            (
                format!("sha256 = \"{EXP}\""),
                format!(
                    "sha256 = \"sha256:{}\"",
                    EXP.trim_start_matches("sha256:").to_uppercase()
                ),
                "reviewed.expectation.sha256",
                ReviewedWhy::Digest,
            ),
            (
                expectation_length.clone(),
                format!("sha256 = \"{EXP}\", byte_length = -1"),
                "reviewed.expectation.byte_length",
                ReviewedWhy::Reference,
            ),
            (
                review_length.clone(),
                format!("sha256 = \"{REV}\", byte_length = 4294967296"),
                "reviewed.review.byte_length",
                ReviewedWhy::Reference,
            ),
            (
                review_length.clone(),
                format!("sha256 = \"{REV}\", byte_length = 0"),
                "reviewed.review.byte_length",
                ReviewedWhy::Empty,
            ),
            (
                format!("media_type = \"application/json\", {review_schema}"),
                format!("media_type = \"text/plain\", {review_schema}"),
                "reviewed.review.media_type",
                ReviewedWhy::MediaType,
            ),
            (
                format!("media_type = \"application/json\", {review_schema}"),
                format!("media_type = \"\", {review_schema}"),
                "reviewed.review.media_type",
                ReviewedWhy::MediaType,
            ),
            (
                review_schema.clone(),
                format!("schema_id = \"{}\"", ExpectationV1::SCHEMA_ID),
                "reviewed.review.schema_id",
                ReviewedWhy::Schema,
            ),
            (
                review_schema.clone(),
                "schema_id = \"\"".to_owned(),
                "reviewed.review.schema_id",
                ReviewedWhy::Schema,
            ),
            (
                format!("sha256 = \"{REV}\""),
                format!("sha256 = \"{EXP}\""),
                "reviewed.review",
                ReviewedWhy::Same,
            ),
            (
                format!("artifact_id = \"{REV_ID}\""),
                format!("artifact_id = \"{EXP_ID}\""),
                "reviewed.review",
                ReviewedWhy::Same,
            ),
        ] {
            assert_eq!(
                refused(&with(&old, &new)),
                ProfileError::Reviewed {
                    path: path.into(),
                    why
                },
                "{old} -> {new}"
            );
        }
        let without = valid();
        let without = &without[..without.find("\n[reviewed]").unwrap_or(without.len())];
        assert_eq!(
            refused(without),
            ProfileError::MissingKey {
                path: "reviewed".into()
            }
        );
    }

    /// B14a-2c-ii-a · a reviewed record's whole closure resolves from the class directory through
    /// the one graph walker: the 003 lane's closure installed by digest, the review walked to its
    /// 22 nodes and the expectation to its 5 (counted from the CAS independently); a root that is
    /// not the declared reference in any one field is `Mismatch`; a member removed is `Missing` at
    /// its reference, a member's bytes changed at the same length `Identity` (the walker's digest),
    /// a declared length that is not the file's `Identity` (the walker's length), a member over the
    /// reviewed bound `Bound`; an absent directory `NotInstalled`.
    #[test]
    fn a_reviewed_closure_resolves_whole_or_names_the_member_it_cannot()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::check::graph::Error;
        let root = private("reviewed-closure");
        let mut profile = Profile {
            declared: compose(valid().as_bytes()).map_err(|e| format!("{e:?}"))?,
            directory: root.clone(),
            digest: String::new(),
        };
        let review = profile.declared.reviewed.review().as_ref().clone();
        let expectation = profile.declared.reviewed.expectation().as_ref().clone();
        assert!(matches!(
            read_reviewed_closure(&profile, Which::Review, &review),
            Err(ReviewedError::NotInstalled)
        ));
        let store = root.join(REVIEWED_DIRECTORY);
        fs::DirBuilder::new().mode(0o700).create(&store)?;
        install_closure(&store)?;
        let graph = read_reviewed_closure(&profile, Which::Review, &review)
            .map_err(|e| format!("{e:?}"))?;
        assert_eq!(graph.object_count(), REVIEW_CLOSURE);
        let graph = read_reviewed_closure(&profile, Which::Expectation, &expectation)
            .map_err(|e| format!("{e:?}"))?;
        assert_eq!(graph.object_count(), EXPECTATION_CLOSURE);
        // A root the profile does not name, one field at a time: the reference is compared whole.
        let mut others = vec![review.clone(); 5];
        others[0].artifact_id = expectation.artifact_id.clone();
        others[1].sha256 = expectation.sha256.clone();
        others[2].byte_length += 1;
        others[3].media_type = Name::new("application/octet-stream")?;
        others[4].schema_id = expectation.schema_id.clone();
        for other in &others {
            assert!(
                matches!(
                    read_reviewed_closure(&profile, Which::Review, other),
                    Err(ReviewedError::Mismatch)
                ),
                "{other:?}"
            );
        }
        // A member removed: `Missing`.
        let member = store.join(REVIEW_ASSUMPTIONS);
        let bytes = fs::read(&member)?;
        fs::remove_file(&member)?;
        assert!(matches!(
            read_reviewed_closure(&profile, Which::Review, &review),
            Err(ReviewedError::Closure(Error::Missing))
        ));
        // A member's bytes changed under its name, at the same length: the walker's digest check
        // alone sees it.
        let mut altered = bytes.clone();
        altered[0] ^= 0x01;
        write(&member, &altered, 0o600);
        assert!(matches!(
            read_reviewed_closure(&profile, Which::Review, &review),
            Err(ReviewedError::Closure(Error::Identity))
        ));
        // A member over the reviewed bound is refused at the bound, before any digest is taken.
        let big = vec![b'x'; usize::try_from(MAX_REVIEWED_BYTES)? + 1];
        write(&member, &big, 0o600);
        assert!(matches!(
            read_reviewed_closure(&profile, Which::Review, &review),
            Err(ReviewedError::Closure(Error::Bound))
        ));
        write(&member, &bytes, 0o600);
        // The declared root's length is not the file's while its digest is: the walker's length
        // check alone sees it, at the root the profile names.
        let mut longer = review.clone();
        longer.byte_length += 1;
        profile.declared.reviewed = Reviewed::new(
            profile.declared.reviewed.expectation().clone(),
            TypedRef::new(longer.clone())?,
        )
        .map_err(|e| format!("{e:?}"))?;
        assert!(matches!(
            read_reviewed_closure(&profile, Which::Review, &longer),
            Err(ReviewedError::Closure(Error::Identity))
        ));
        Ok(())
    }

    /// B14a-2a · a reviewed record is returned only when its bytes hash to the digest the profile
    /// names; absence, custody, size and a substituted record are each refused by name.
    #[test]
    fn a_reviewed_record_is_read_only_as_the_record_the_profile_names()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = private("reviewed");
        let mut profile = Profile {
            declared: compose(valid().as_bytes()).map_err(|e| format!("{e:?}"))?,
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
        // The declared length is not the file's: not the named record, whatever its digest.
        let declared = profile.declared.reviewed.clone();
        let mut longer = declared.review().as_ref().clone();
        longer.byte_length += 1;
        profile.declared.reviewed =
            Reviewed::new(declared.expectation().clone(), TypedRef::new(longer)?)
                .map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            read_reviewed(&profile, Which::Review),
            Err(ReviewedError::Mismatch)
        );
        profile.declared.reviewed = declared;
        // The review's bytes under the expectation's name: substituted, not the named record.
        write(&store.join(name(EXP)), REVIEW, 0o600);
        assert_eq!(
            read_reviewed(&profile, Which::Expectation),
            Err(ReviewedError::Mismatch)
        );
        // The expectation's bytes with one bit changed, at its declared length: the digest alone
        // sees it.
        let mut altered = EXPECTATION.to_vec();
        altered[0] ^= 0x01;
        write(&store.join(name(EXP)), &altered, 0o600);
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
        // A symlink under the record's name is not followed, whatever it points at.
        assert!(fs::remove_file(store.join(name(REV))).is_ok());
        write(&root.join("review-elsewhere"), REVIEW, 0o600);
        assert!(
            std::os::unix::fs::symlink(root.join("review-elsewhere"), store.join(name(REV)))
                .is_ok()
        );
        assert_eq!(
            read_reviewed(&profile, Which::Review),
            Err(ReviewedError::Custody)
        );
        assert!(fs::set_permissions(&store, fs::Permissions::from_mode(0o755)).is_ok());
        assert_eq!(
            read_reviewed(&profile, Which::Review),
            Err(ReviewedError::Custody)
        );
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    /// B14a-2c-ii-c · `[grant]` and `[effect]` are required, each field the receipt row's own
    /// scalar, each file declared with its digest (the refusals at their key paths are
    /// `a_grant_or_effect_field_is_refused_at_its_path`'s).
    #[test]
    fn the_grant_and_effect_tables_are_declared_by_digest() -> Result<(), Box<dyn std::error::Error>>
    {
        let declared = compose(valid().as_bytes()).map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            declared.grant,
            Grant {
                grant_id: Id::new(GRANT_ID)?,
                issuer_id: Name::new("operator")?,
                authority: DeclaredFile {
                    file: "authority.json".to_owned(),
                    sha256: Sha::new(AUTH)?,
                },
            }
        );
        assert_eq!(
            declared.effect,
            Effect {
                effect_id: Name::new("fixed-u64-workload-output")?,
                scope: Text::new(
                    "compile, link and execute the fixed workload in a private bounded scratch; no network"
                )?,
                specification: DeclaredFile {
                    file: "isolation.json".to_owned(),
                    sha256: Sha::new(SPEC)?,
                },
            }
        );
        Ok(())
    }

    /// B14a-2c-ii-c · every `[grant]`/`[effect]` field refused at its own key path: an id that is
    /// not a v4 UUID, an empty name, a file name that is not one component (`..`, `.`, empty, a
    /// NUL), a digest that is not `sha256:` hex, a scope past the text bound, an unknown key at
    /// either level.
    #[test]
    fn a_grant_or_effect_field_is_refused_at_its_path() {
        // A file name that is not one component, each spelling refused at the same path.
        for name in ["../authority.json", "..", ".", ""] {
            assert_eq!(
                refused(&with(
                    "file = \"authority.json\"",
                    &format!("file = \"{name}\"")
                )),
                ProfileError::Declared {
                    path: "grant.authority.file".into(),
                    why: DeclaredWhy::FileName,
                },
                "{name:?}"
            );
        }
        for (old, new, expected) in [
            (
                format!("grant_id = \"{GRANT_ID}\""),
                "grant_id = \"not-a-uuid\"".to_owned(),
                ProfileError::Declared {
                    path: "grant.grant_id".into(),
                    why: DeclaredWhy::Id,
                },
            ),
            (
                "issuer_id = \"operator\"".to_owned(),
                "issuer_id = \"\"".to_owned(),
                ProfileError::Declared {
                    path: "grant.issuer_id".into(),
                    why: DeclaredWhy::Name,
                },
            ),
            (
                "file = \"isolation.json\"".to_owned(),
                "file = \"iso\\u0000lation.json\"".to_owned(),
                ProfileError::Declared {
                    path: "effect.specification.file".into(),
                    why: DeclaredWhy::FileName,
                },
            ),
            (
                format!("sha256 = \"{AUTH}\""),
                "sha256 = \"x\"".to_owned(),
                ProfileError::Declared {
                    path: "grant.authority.sha256".into(),
                    why: DeclaredWhy::Digest,
                },
            ),
            (
                "scope = \"compile, link and execute the fixed workload in a private bounded scratch; no network\"".to_owned(),
                format!("scope = \"{}\"", "s".repeat(4097)),
                ProfileError::Declared {
                    path: "effect.scope".into(),
                    why: DeclaredWhy::Text,
                },
            ),
            (
                format!("sha256 = \"{SPEC}\""),
                "sha256 = \"x\"".to_owned(),
                ProfileError::Declared {
                    path: "effect.specification.sha256".into(),
                    why: DeclaredWhy::Digest,
                },
            ),
            (
                "effect_id = \"fixed-u64-workload-output\"".to_owned(),
                "effect_id = \"\"".to_owned(),
                ProfileError::Declared {
                    path: "effect.effect_id".into(),
                    why: DeclaredWhy::Name,
                },
            ),
            (
                "issuer_id = \"operator\"".to_owned(),
                "issuer_id = \"operator\"\nextra = 1".to_owned(),
                ProfileError::UnknownKey {
                    path: "grant.extra".into(),
                },
            ),
            (
                "[effect]\n".to_owned(),
                "[effect_]\n".to_owned(),
                ProfileError::UnknownKey {
                    path: "effect_".into(),
                },
            ),
        ] {
            assert_eq!(refused(&with(&old, &new)), expected, "{old} -> {new}");
        }
    }

    /// B14a-2c-ii-c · a declared file is returned only from the class directory's custody and only
    /// when its bytes hash to the declared digest; absence, substitution and custody each refused.
    #[test]
    fn a_declared_file_is_read_only_as_declared() -> Result<(), Box<dyn std::error::Error>> {
        let root = private("declared");
        let profile = Profile {
            declared: compose(valid().as_bytes()).map_err(|e| format!("{e:?}"))?,
            directory: root.clone(),
            digest: String::new(),
        };
        let authority = &profile.declared.grant.authority;
        assert_eq!(
            read_declared(&profile, authority),
            Err(ReviewedError::NotInstalled)
        );
        write(&root.join("authority.json"), SPECIFICATION, 0o600);
        assert_eq!(
            read_declared(&profile, authority),
            Err(ReviewedError::Mismatch)
        );
        write(&root.join("authority.json"), AUTHORITY, 0o600);
        assert_eq!(read_declared(&profile, authority), Ok(AUTHORITY.to_vec()));
        write(&root.join("authority.json"), AUTHORITY, 0o644);
        assert_eq!(
            read_declared(&profile, authority),
            Err(ReviewedError::Custody)
        );
        let big = vec![b'x'; usize::try_from(MAX_PROFILE_BYTES)? + 1];
        write(&root.join("isolation.json"), &big, 0o600);
        assert_eq!(
            read_declared(&profile, &profile.declared.effect.specification),
            Err(ReviewedError::TooLarge)
        );
        fs::remove_dir_all(&root)?;
        Ok(())
    }

    /// The reviewed bound is the reviewed decision's value (F122: pinned beside its reason, the
    /// 65,533 B specification it must hold), not merely whatever the profile's bound becomes.
    #[test]
    fn the_reviewed_bound_is_sixty_four_kibibytes() {
        assert_eq!(MAX_REVIEWED_BYTES, 65_536);
    }
}
