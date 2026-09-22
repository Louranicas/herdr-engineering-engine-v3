//! Application adapter between the check publisher and the sole Store object owner.
//! The lookup map describes one collection closure; it is not a task-state ledger.

use crate::check::{
    collector::{Sink, SinkError},
    graph::{self, Objects},
};
use crate::contracts::{
    UuidV4,
    receipt::{Id, Name, Payload, Ref, Sha, Validate},
};
use crate::store::{self, ArtifactStaging, Object, Store};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::time::Instant;

pub struct Evidence<'a> {
    store: Owner<'a>,
    deadline: Instant,
    objects: BTreeMap<String, (Ref, Object)>,
    last_publication_error: Option<store::Error>,
    pending_publications: Vec<(Ref, Object)>,
}

enum Owner<'a> {
    Store(&'a Store),
    Staging(&'a ArtifactStaging),
}

impl Owner<'_> {
    fn publish(
        &self,
        bytes: &[u8],
        id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Object, store::Error> {
        match self {
            Self::Store(owner) => owner.publish(bytes, id, deadline),
            Self::Staging(owner) => owner.publish(bytes, id, deadline),
        }
    }

    fn read_object(&self, object: &Object, deadline: Instant) -> Result<Vec<u8>, store::Error> {
        match self {
            Self::Store(owner) => owner.read_object(object, deadline),
            Self::Staging(owner) => owner.read_object(object, deadline),
        }
    }
}

impl<'a> Evidence<'a> {
    #[must_use]
    pub fn new(store: &'a Store, deadline: Instant) -> Self {
        Self::with_owner(Owner::Store(store), deadline)
    }

    /// A collector child stages artifacts without opening the coordinator's ledger.
    #[must_use]
    pub fn staged(store: &'a ArtifactStaging, deadline: Instant) -> Self {
        Self::with_owner(Owner::Staging(store), deadline)
    }

    /// Borrow only the artifact staging owner; never exposes the ledger owner.
    #[must_use]
    pub const fn staging_owner(&self) -> Option<&ArtifactStaging> {
        match self.store {
            Owner::Staging(owner) => Some(owner),
            Owner::Store(_) => None,
        }
    }

    fn with_owner(store: Owner<'a>, deadline: Instant) -> Self {
        Self {
            store,
            deadline,
            objects: BTreeMap::new(),
            last_publication_error: None,
            pending_publications: Vec::new(),
        }
    }

    /// Publish exact raw bytes under a fresh local reference; no candidate path import.
    ///
    /// # Errors
    /// Refuses expired budgets, invalid media, entropy or immutable publication failure.
    pub fn payload(&mut self, bytes: &[u8], media: &str) -> Result<Payload, SinkError> {
        let reference = Ref {
            artifact_id: self.fresh_id()?,
            sha256: Sha::new(digest(bytes)).map_err(|_| SinkError::Publication)?,
            byte_length: u32::try_from(bytes.len()).map_err(|_| SinkError::Publication)?,
            media_type: Name::new(media).map_err(|_| SinkError::Publication)?,
            schema_id: Name::new("hee3.raw/1").map_err(|_| SinkError::Publication)?,
        };
        self.publish(&reference, bytes)?;
        Payload::new(reference).map_err(|_| SinkError::Publication)
    }

    /// Register preexisting input evidence only after exact immutable-object readback.
    ///
    /// # Errors
    /// Refuses conflicting identity, invalid metadata, missing/corrupt bytes or deadline.
    pub fn register(&mut self, reference: Ref, object: Object) -> Result<(), SinkError> {
        reference.validate().map_err(|_| SinkError::Publication)?;
        let pending = self
            .pending_publications
            .iter()
            .find(|(item, _)| item.artifact_id == reference.artifact_id);
        if self.objects.len() >= 4096
            || (self
                .objects
                .len()
                .saturating_add(self.pending_publications.len())
                >= 4096
                && pending.is_none())
            || pending.is_some_and(|(item, stored)| item != &reference || stored != &object)
            || self.objects.contains_key(reference.artifact_id.as_str())
            || reference.sha256.as_str() != object.digest()
            || u64::from(reference.byte_length) != object.size()
        {
            return Err(SinkError::Publication);
        }
        if let Err(error) = self.store.read_object(&object, self.deadline) {
            self.last_publication_error = Some(error);
            return Err(SinkError::Unavailable);
        }
        self.pending_publications
            .retain(|(item, _)| item.artifact_id != reference.artifact_id);
        self.objects.insert(
            reference.artifact_id.as_str().to_owned(),
            (reference, object),
        );
        Ok(())
    }

    #[must_use]
    pub fn registered(&self) -> &BTreeMap<String, (Ref, Object)> {
        &self.objects
    }

    /// Retained Store error may describe an uncertain publication; it never grants retry.
    #[must_use]
    pub fn last_publication_error(&self) -> Option<&store::Error> {
        self.last_publication_error.as_ref()
    }

    /// Published objects whose registration/readback failed remain available for reconciliation.
    #[must_use]
    pub fn pending_publications(&self) -> &[(Ref, Object)] {
        &self.pending_publications
    }

    #[must_use]
    pub fn into_registered(self) -> BTreeMap<String, (Ref, Object)> {
        self.objects
    }
}

impl Objects for Evidence<'_> {
    fn open(&self, reference: &Ref) -> Result<Box<dyn Read + '_>, graph::Error> {
        let (registered, object) = self
            .objects
            .get(reference.artifact_id.as_str())
            .ok_or(graph::Error::Missing)?;
        if registered != reference {
            return Err(graph::Error::Identity);
        }
        let bytes = self
            .store
            .read_object(object, self.deadline)
            .map_err(|_| graph::Error::Io)?;
        Ok(Box::new(Cursor::new(bytes)))
    }
}

impl Sink for Evidence<'_> {
    fn fresh_id(&mut self) -> Result<Id, SinkError> {
        for _ in 0..4 {
            let id = fresh_id(self.deadline)?;
            if !self.objects.contains_key(id.as_str())
                && !self
                    .pending_publications
                    .iter()
                    .any(|(reference, _)| reference.artifact_id == id)
            {
                return Ok(id);
            }
        }
        Err(SinkError::Publication)
    }

    fn contains_id(&self, id: &Id) -> Result<bool, SinkError> {
        if Instant::now() >= self.deadline {
            return Err(SinkError::Unavailable);
        }
        Ok(self.objects.contains_key(id.as_str())
            || self
                .pending_publications
                .iter()
                .any(|(reference, _)| reference.artifact_id == *id))
    }

    fn publish(&mut self, reference: &Ref, bytes: &[u8]) -> Result<Ref, SinkError> {
        reference.validate().map_err(|_| SinkError::Publication)?;
        if self
            .objects
            .len()
            .saturating_add(self.pending_publications.len())
            >= 4096
            || self.objects.contains_key(reference.artifact_id.as_str())
            || self
                .pending_publications
                .iter()
                .any(|(pending, _)| pending.artifact_id == reference.artifact_id)
            || bytes.len() != reference.byte_length as usize
            || reference.sha256.as_str() != digest(bytes)
        {
            return Err(SinkError::Publication);
        }
        let stage =
            UuidV4::parse(reference.artifact_id.as_str()).map_err(|_| SinkError::Publication)?;
        let object = match self.store.publish(bytes, stage, self.deadline) {
            Ok(object) => object,
            Err(error) => {
                self.last_publication_error = Some(error);
                return Err(SinkError::Publication);
            }
        };
        self.pending_publications
            .push((reference.clone(), object.clone()));
        self.register(reference.clone(), object)?;
        Ok(reference.clone())
    }
}

fn digest(bytes: &[u8]) -> String {
    let mut value = String::from("sha256:");
    let hex = b"0123456789abcdef";
    for byte in Sha256::digest(bytes) {
        value.push(char::from(hex[usize::from(byte >> 4)]));
        value.push(char::from(hex[usize::from(byte & 15)]));
    }
    value
}

/// Nonblocking OS entropy with a finite retry budget; no clock-derived identities.
///
/// # Errors
/// Refuses expired time, unavailable entropy or invalid generated identity.
pub fn fresh_id(deadline: Instant) -> Result<Id, SinkError> {
    let mut bytes = [0_u8; 16];
    let mut offset = 0;
    for _ in 0..32 {
        if Instant::now() >= deadline {
            return Err(SinkError::Unavailable);
        }
        match rustix::rand::getrandom(&mut bytes[offset..], rustix::rand::GetRandomFlags::NONBLOCK)
        {
            Ok(0) => return Err(SinkError::Unavailable),
            Ok(count) => offset += count,
            Err(rustix::io::Errno::INTR) => {}
            Err(_) => return Err(SinkError::Unavailable),
        }
        if offset == bytes.len() {
            bytes[6] = (bytes[6] & 15) | 64;
            bytes[8] = (bytes[8] & 63) | 128;
            let hex = b"0123456789abcdef";
            let mut value = String::with_capacity(36);
            for (index, byte) in bytes.into_iter().enumerate() {
                if [4, 6, 8, 10].contains(&index) {
                    value.push('-');
                }
                value.push(char::from(hex[usize::from(byte >> 4)]));
                value.push(char::from(hex[usize::from(byte & 15)]));
            }
            return Id::new(value).map_err(|_| SinkError::Unavailable);
        }
    }
    Err(SinkError::Unavailable)
}
