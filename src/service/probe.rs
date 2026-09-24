//! Read-only preparation for a future service-owner probe. No value here admits
//! an operation, authenticates a grant, launches a child, or asserts useful health.
use super::{Class, Error, Profile};
use crate::contracts::{
    UuidV4, parse_u64_decimal,
    roster::{Locality, RosterHeadV1, Snapshot},
};
use crate::store::{Principal, Store};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const MAX_BODY: usize = 1_048_576;
const MAX_HORIZON: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub enum ProbeError {
    Service(Error),
    InvalidRequest,
    Bounds,
    MissingProfile,
    AmbiguousProfile,
    OfflineUnavailable,
    Cancelled,
    Deadline,
    Changed,
}
impl From<Error> for ProbeError {
    fn from(error: Error) -> Self {
        Self::Service(error)
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Network {
    None,
    ConfiguredAllowlist,
}
/// Internal decoding of the existing RC03 body; this is not the public envelope.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    service_id: String,
    probe_id: String,
    probe_version: u16,
    max_cost_microunits: String,
    network_scope: Network,
}

/// Exact read-only preparation facts. This is never a grant or a dispatch permit.
/// The existing action/Store owners must still authorize and durably admit any
/// effect. Revalidation is a point observation, not a lock against later updates.
#[derive(Debug)]
pub struct ProbePreparation {
    request: Vec<u8>,
    request_sha256: String,
    requested_ceiling: u64,
    profile: Profile,
    head: RosterHeadV1,
    ledger_epoch: String,
    receiver_epoch: String,
    receiver_ms: u64,
    start: Instant,
    deadline: Instant,
}
impl ProbePreparation {
    #[must_use]
    pub fn raw_body(&self) -> &[u8] {
        &self.request
    }
    /// Exact bytes, including whitespace; not normalized or an envelope digest.
    #[must_use]
    pub fn body_sha256(&self) -> &str {
        &self.request_sha256
    }
    #[must_use]
    pub const fn requested_cost_ceiling(&self) -> u64 {
        self.requested_ceiling
    }
    #[must_use]
    pub const fn profile(&self) -> &Profile {
        &self.profile
    }
    #[must_use]
    pub(super) const fn original_start(&self) -> Instant {
        self.start
    }
    #[must_use]
    pub const fn original_deadline(&self) -> Instant {
        self.deadline
    }
    /// Re-read the same admitted subject and protected profile before a future
    /// authorized owner proceeds. No observation, operation or grant is written.
    /// # Errors
    /// Refuses cancellation/expiry, invisible/update/disable/profile/epoch drift,
    /// clock regression and Store failure; it never renews the original horizon.
    pub fn revalidate(
        &self,
        store: &Store,
        principal: &Principal,
        profiles: &[Profile],
        cancelled: &AtomicBool,
    ) -> Result<(), ProbeError> {
        window(self.start, self.deadline, cancelled)?;
        let snapshot = store
            .roster_snapshot(principal, self.deadline)
            .map_err(Error::from)?;
        let (profile, head) = binding(&snapshot, &self.profile.record_id, profiles)?;
        if profile != &self.profile
            || !same_admission(head, &self.head)
            || snapshot.epoch != self.ledger_epoch
            || snapshot.now.epoch != self.receiver_epoch
            || snapshot.now.monotonic_ms < self.receiver_ms
        {
            return Err(ProbeError::Changed);
        }
        window(self.start, self.deadline, cancelled)
    }
}

/// Validate an exact existing service.probe body against one real Store snapshot.
/// Profiles are protected caller configuration; no request can supply authority.
/// The only supported execution policy is offline, zero external spend. A larger
/// caller ceiling remains just a ceiling and cannot increase that allowance.
/// # Errors
/// Refuses invalid/duplicate/unknown body fields, unbounded input, missing or
/// ambiguous profile, wrong admitted binding, offline-incompatible targets,
/// cancellation, expired/future/overlong original clock and Store read failures.
pub fn prepare_probe(
    store: &Store,
    principal: &Principal,
    raw: &[u8],
    profiles: &[Profile],
    start: Instant,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<ProbePreparation, ProbeError> {
    window(start, deadline, cancelled)?;
    if raw.is_empty() || raw.len() > MAX_BODY {
        return Err(ProbeError::Bounds);
    }
    let request: Request = serde_json::from_slice(raw).map_err(|_| ProbeError::InvalidRequest)?;
    UuidV4::parse(&request.service_id).map_err(|_| ProbeError::InvalidRequest)?;
    super::profile::text(&request.probe_id, 128).map_err(|_| ProbeError::InvalidRequest)?;
    let requested_ceiling =
        parse_u64_decimal(&request.max_cost_microunits).map_err(|_| ProbeError::InvalidRequest)?;
    if request.probe_version != 1 {
        return Err(ProbeError::InvalidRequest);
    }
    if request.network_scope != Network::None {
        return Err(ProbeError::OfflineUnavailable);
    }
    let snapshot = store
        .roster_snapshot(principal, deadline)
        .map_err(Error::from)?;
    let (profile, head) = binding(&snapshot, &request.service_id, profiles)?;
    if profile.probe_id != request.probe_id || profile.probe_version != request.probe_version {
        return Err(ProbeError::Changed);
    }
    window(start, deadline, cancelled)?;
    Ok(ProbePreparation {
        request: raw.to_vec(),
        request_sha256: body_digest(raw),
        requested_ceiling,
        profile: profile.clone(),
        head: head.clone(),
        ledger_epoch: snapshot.epoch,
        receiver_epoch: snapshot.now.epoch,
        receiver_ms: snapshot.now.monotonic_ms,
        start,
        deadline,
    })
}
fn window(start: Instant, deadline: Instant, cancelled: &AtomicBool) -> Result<(), ProbeError> {
    if cancelled.load(Ordering::Acquire) {
        return Err(ProbeError::Cancelled);
    }
    let now = Instant::now();
    if start > now || deadline <= now {
        return Err(ProbeError::Deadline);
    }
    if deadline
        .checked_duration_since(start)
        .is_none_or(|duration| duration > MAX_HORIZON)
    {
        return Err(ProbeError::Bounds);
    }
    Ok(())
}
fn binding<'a>(
    snapshot: &'a Snapshot,
    id: &str,
    profiles: &'a [Profile],
) -> Result<(&'a Profile, &'a RosterHeadV1), ProbeError> {
    if profiles.len() > crate::contracts::roster::MAX_RECORDS {
        return Err(ProbeError::Bounds);
    }
    let record = snapshot
        .records
        .iter()
        .find(|record| record.head.record_id == id)
        .ok_or(Error::Store(crate::store::Error::NotFound))?;
    let mut matching = profiles.iter().filter(|profile| profile.record_id == id);
    let profile = matching.next().ok_or(ProbeError::MissingProfile)?;
    if matching.next().is_some() {
        return Err(ProbeError::AmbiguousProfile);
    }
    profile.bind(&record.head)?;
    if record.head.disabled {
        return Err(Error::Disabled.into());
    }
    if record.head.definition.locality != Locality::Local || profile.class == Class::RemoteEndpoint
    {
        return Err(ProbeError::OfflineUnavailable);
    }
    Ok((profile, &record.head))
}

fn same_admission(left: &RosterHeadV1, right: &RosterHeadV1) -> bool {
    left.record_id == right.record_id
        && left.record_version == right.record_version
        && left.definition == right.definition
        && left.disabled == right.disabled
}
fn body_digest(raw: &[u8]) -> String {
    sha256_text(&Sha256::digest(raw))
}
/// The module's one `sha256:` + lowercase-hex rendering of a finished SHA-256
/// digest. `local_probe` renders executable digests through it, so the
/// published-vector test below pins both call sites.
pub(super) fn sha256_text(digest: &[u8]) -> String {
    let mut value = String::from("sha256:");
    let hex = b"0123456789abcdef";
    for &byte in digest {
        value.push(char::from(hex[usize::from(byte >> 4)]));
        value.push(char::from(hex[usize::from(byte & 15)]));
    }
    value
}
#[cfg(test)]
mod tests {
    use super::body_digest;
    #[test]
    fn exact_byte_hash_matches_independent_sha256_reference_vectors() {
        assert_eq!(body_digest(b"{\"service_id\":\"10000000-0000-4000-8000-000000000001\",\"probe_id\":\"useful-check\",\"probe_version\":1,\"max_cost_microunits\":\"0\",\"network_scope\":\"none\"}"), "sha256:0b935de78a2be7d5600d867d109fc48cfbb233daa77febdcef7f42c2f7d81fc3");
        assert_eq!(body_digest(b" \n{\"service_id\":\"10000000-0000-4000-8000-000000000001\",\"probe_id\":\"useful-check\",\"probe_version\":1,\"max_cost_microunits\":\"0\",\"network_scope\":\"none\"}\t "), "sha256:7793ba31cde08839afdcb903055f7e73edd79bbc9d3c559f4e4069a9f9c8549d");
        assert_eq!(
            body_digest(b"abc"),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            body_digest(b""),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
