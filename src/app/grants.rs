//! The file grant store behind the control receiver (RC03 §5).
//!
//! RC03: "`grant_id` is a server-side grant reference scoped to that principal, action/version,
//! resource selectors, bounds and expiry. It is not a bearer grant. `scope_sha256` binds the
//! reviewed scope record and prevents a same-ID/different-scope substitution." This store makes
//! each half structural:
//!
//! * **The record is the file.** A grant is `<grants>/<grant_id>.json`, and the scope a request
//!   names is SHA-256 over that file's exact bytes — the reviewed record, not a re-serialization
//!   of it. Editing the file changes the scope, so every request that named the old one refuses.
//! * **Not a bearer.** The record names the principal (uid and role) it belongs to; a peer that
//!   presents it under any other principal is refused exactly as an unknown grant is.
//! * **Custody before content.** The directory must be the operator's, mode 0700, opened without
//!   following a link; the record must be a regular 0600 file of the operator's, read under
//!   [`MAX_GRANT_BYTES`]. A grant a second user could have written is not a grant.
//!
//! Every refusal resolves to `None`, which the receiver renders as `forbidden`: the store never
//! tells a caller which of the checks it failed.

use crate::actions::control::Grants;
use crate::actions::{Caller, Effect, Owner};
use crate::contracts::control::request_sha256;
use crate::contracts::{UuidV4, parse_u64_decimal};
use crate::store::Principal;
use rustix::fs::{Mode, OFlags, openat};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// The largest grant record read.
pub const MAX_GRANT_BYTES: u64 = 64 * 1024;
/// The schema every grant record declares.
pub const GRANT_SCHEMA: &str = "hee3.grant/1";

/// Why the store could not be opened at all.
#[derive(Debug)]
pub enum Error {
    /// The directory is not the operator's private 0700 directory.
    Custody,
    /// The directory could not be opened.
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custody => {
                f.write_str("grant directory is not the operator's private 0700 directory")
            }
            Self::Io(error) => write!(f, "grant directory could not be opened: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<rustix::io::Errno> for Error {
    fn from(value: rustix::io::Errno) -> Self {
        Self::Io(value.into())
    }
}

/// A grant record, exactly as reviewed. Unknown members refuse the record.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: String,
    grant_id: String,
    uid: u32,
    role: String,
    owners: Vec<String>,
    effects: Vec<String>,
    expires_unix_ms: String,
}

/// Grants held as reviewed files in one private directory.
#[derive(Debug)]
pub struct FileGrants {
    directory: File,
}

impl FileGrants {
    /// Open the grant directory under custody.
    ///
    /// # Errors
    ///
    /// [`Error::Custody`] unless `path` is a directory owned by this process's effective user
    /// with mode 0700, reached without following a final symlink; [`Error::Io`] otherwise.
    pub fn open(path: &Path) -> Result<Self, Error> {
        let directory = File::from(rustix::fs::open(
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        let meta = directory.metadata().map_err(Error::Io)?;
        if !meta.is_dir() || !owned_private(&meta, 0o700) {
            return Err(Error::Custody);
        }
        Ok(Self { directory })
    }

    fn record(&self, grant_id: &str) -> Option<Vec<u8>> {
        // The name is a validated UUIDv4, so it can hold no separator and no dot segment.
        UuidV4::parse(grant_id).ok()?;
        let file = File::from(
            openat(
                &self.directory,
                format!("{grant_id}.json"),
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
                Mode::empty(),
            )
            .ok()?,
        );
        let meta = file.metadata().ok()?;
        if !meta.is_file() || !owned_private(&meta, 0o600) || meta.len() > MAX_GRANT_BYTES {
            return None;
        }
        let mut bytes = Vec::new();
        file.take(MAX_GRANT_BYTES + 1)
            .read_to_end(&mut bytes)
            .ok()?;
        (bytes.len() as u64 <= MAX_GRANT_BYTES).then_some(bytes)
    }
}

fn owned_private(meta: &std::fs::Metadata, mode: u32) -> bool {
    meta.uid() == rustix::process::geteuid().as_raw() && meta.mode() & 0o777 == mode
}

impl Grants for FileGrants {
    fn resolve(
        &self,
        principal: &Principal,
        grant_id: &str,
        scope_sha256: &str,
        now_unix_ms: u64,
    ) -> Option<Caller> {
        let bytes = self.record(grant_id)?;
        if request_sha256(&bytes) != scope_sha256 {
            return None;
        }
        let record: Record = serde_json::from_slice(&bytes).ok()?;
        let expires = parse_u64_decimal(&record.expires_unix_ms).ok()?;
        if record.schema != GRANT_SCHEMA
            || record.grant_id != grant_id
            || !principal.is(record.uid, &record.role)
            || expires <= now_unix_ms
        {
            return None;
        }
        let mut caller = Caller::new();
        for name in &record.owners {
            caller = caller.seeing(Owner::ALL.into_iter().find(|owner| owner.name() == name)?);
        }
        for name in &record.effects {
            caller = caller.granted(
                Effect::ALL
                    .into_iter()
                    .find(|effect| effect.name() == name)?,
            );
        }
        Some(caller)
    }
}
