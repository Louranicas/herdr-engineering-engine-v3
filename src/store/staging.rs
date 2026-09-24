//! Exclusive artifact staging for a collector process, using the Store's exact
//! immutable publication primitive. This opens no database or task ledger.

use super::{Object, Result, Store, artifact, remaining};
use crate::contracts::UuidV4;
use std::fs::File;
use std::path::Path;
use std::time::Instant;

#[derive(Debug)]
pub struct ArtifactStaging {
    _root: artifact::Directory,
    objects: artifact::Directory,
    _lock: File,
    fault: super::Fault,
}

impl ArtifactStaging {
    /// Acquire one existing private collector directory and its artifact subdirectory.
    /// A caller may reopen retained staging only after the previous owner releases it.
    ///
    /// # Errors
    /// Refuses changed/nonprivate paths, another owner, missing layout or expiry.
    pub fn open(root: &Path, create: bool, deadline: Instant) -> Result<Self> {
        remaining(deadline)?;
        let root = artifact::Directory::root(root)?;
        let lock = root.lock(deadline)?;
        let objects = root.child("sha256", create)?;
        remaining(deadline)?;
        Ok(Self {
            _root: root,
            objects,
            _lock: lock,
            fault: super::NO_FAULT,
        })
    }

    pub(super) fn from_store(store: &Store, deadline: Instant) -> Result<Self> {
        remaining(deadline)?;
        let staging = Self {
            _root: store.root.try_clone()?,
            objects: store.objects.try_clone()?,
            // A duplicate shares the held open-file-description lock. Dropping
            // this descriptor never explicitly unlocks the Store's descriptor.
            _lock: store.lock.try_clone()?,
            fault: store.fault(),
        };
        remaining(deadline)?;
        Ok(staging)
    }

    fn fault(&self) -> super::Fault {
        self.fault
    }

    /// Publish exact bytes with the same fsync/no-replace/readback rules as Store.
    /// Expiry or failed publication may leave retained objects; it is not rollback.
    ///
    /// # Errors
    /// Refuses size/custody/identity failures, failed filesystem effects or expiry.
    pub fn publish(&self, bytes: &[u8], id: UuidV4<'_>, deadline: Instant) -> Result<Object> {
        remaining(deadline)?;
        let object = artifact::publish(&self.objects, bytes, id, |point| {
            cut_point!(self.fault(), point);
            remaining(deadline).map(|_| ())
        })?;
        remaining(deadline)?;
        Ok(object)
    }

    /// # Errors
    /// Refuses missing/corrupt objects, failed private custody or expired observation.
    pub fn read_object(&self, object: &Object, deadline: Instant) -> Result<Vec<u8>> {
        remaining(deadline)?;
        let bytes = artifact::verify(&self.objects, object)?;
        remaining(deadline)?;
        Ok(bytes)
    }
}
