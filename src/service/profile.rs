use super::Error;
use crate::contracts::{
    Generation, UuidV4,
    roster::{Kind, Locality, RosterHeadV1},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Class {
    Daemon,
    OneShot,
    Library,
    RemoteEndpoint,
    NeuralOperator,
}
/// Protected configuration binding only; no serialization into closed roster v1.
/// This value is neither a persistent registry nor a capability/grant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    pub record_id: String,
    pub record_version: String,
    pub owner_id: String,
    pub endpoint_ref: Option<String>,
    pub class: Class,
    pub probe_id: String,
    pub probe_version: u16,
    pub actual_identity: String,
    pub immutable_revision: String,
}
impl Profile {
    /// # Errors
    /// Refuses noncanonical identity and unbounded or unsupported profile fields.
    pub fn validate(&self) -> Result<(), Error> {
        UuidV4::parse(&self.record_id).map_err(|_| Error::InvalidProfile)?;
        self.record_version
            .parse::<Generation>()
            .map_err(|_| Error::InvalidProfile)?;
        if let Some(endpoint) = &self.endpoint_ref {
            UuidV4::parse(endpoint).map_err(|_| Error::InvalidProfile)?;
        }
        for value in [&self.owner_id, &self.probe_id] {
            text(value, 128)?;
        }
        for value in [&self.actual_identity, &self.immutable_revision] {
            text(value, 256)?;
        }
        if self.probe_version != 1
            || (self.class == Class::RemoteEndpoint && self.endpoint_ref.is_none())
        {
            return Err(Error::InvalidProfile);
        }
        Ok(())
    }
    pub(super) fn bind(&self, head: &RosterHeadV1) -> Result<(), Error> {
        self.validate()?;
        if head.definition.kind != Kind::Service {
            return Err(Error::WrongKind);
        }
        if head.record_id != self.record_id
            || head.record_version != self.record_version
            || head.definition.owner_id != self.owner_id
            || head.definition.endpoint_ref != self.endpoint_ref
            || (self.class == Class::RemoteEndpoint && head.definition.locality == Locality::Local)
        {
            return Err(Error::Binding);
        }
        Ok(())
    }
}
pub(super) fn text(value: &str, limit: usize) -> Result<(), Error> {
    if value.is_empty()
        || value.len() > limit
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b' ')
    {
        return Err(Error::InvalidProfile);
    }
    Ok(())
}
