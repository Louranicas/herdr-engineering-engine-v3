//! Roster persistence in the existing ledger and transaction owner.

use super::{
    CutPoint, Error, Object, Principal, Result, Store, check_point, digest, next, number,
    read_number, remaining, schema,
};
use crate::contracts::roster::{
    self as dto, MAX_HISTORY, MAX_INPUT, MAX_RECORDS, Observation, ObservationInput,
    ObservationSource, Outcome, ReceiptTime, Record, RosterDefinitionV1, RosterHeadV1, Snapshot,
    Update,
};
use crate::contracts::{Generation, UuidV4};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

mod attempts;
pub use attempts::{RosterAttempt, RosterStart};

#[derive(Clone, Debug)]
pub(super) struct ReceiverClock {
    epoch: String,
    origin: Instant,
    unix_origin_ms: u64,
    #[cfg(test)]
    pub(super) test_time: Option<ReceiptTime>,
    #[cfg(test)]
    pub(super) test_ids: std::collections::VecDeque<String>,
}

impl ReceiverClock {
    pub(super) fn new(deadline: Instant) -> Result<Self> {
        let unix_origin_ms = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| Error::Runtime)?
                .as_millis(),
        )
        .map_err(|_| Error::Bound)?;
        Ok(Self {
            epoch: random_id(deadline)?,
            origin: Instant::now(),
            unix_origin_ms,
            #[cfg(test)]
            test_time: None,
            #[cfg(test)]
            test_ids: std::collections::VecDeque::new(),
        })
    }

    pub(super) fn sample(&self) -> Result<ReceiptTime> {
        #[cfg(test)]
        if let Some(value) = &self.test_time {
            return Ok(value.clone());
        }
        let monotonic_ms =
            u64::try_from(self.origin.elapsed().as_millis()).map_err(|_| Error::Bound)?;
        Ok(ReceiptTime {
            epoch: self.epoch.clone(),
            monotonic_ms,
            unix_ms: self
                .unix_origin_ms
                .checked_add(monotonic_ms)
                .ok_or(Error::Bound)?,
        })
    }

    #[cfg_attr(
        not(test),
        allow(
            clippy::unused_self,
            reason = "Only test builds inject exact UUID collisions; production always uses nonblocking getrandom."
        )
    )]
    fn id(&mut self, deadline: Instant) -> Result<String> {
        #[cfg(test)]
        if let Some(id) = self.test_ids.pop_front() {
            return Ok(id);
        }
        random_id(deadline)
    }
}

fn random_id(deadline: Instant) -> Result<String> {
    let mut bytes = [0_u8; 16];
    let mut offset = 0;
    while offset < bytes.len() {
        remaining(deadline)?;
        match rustix::rand::getrandom(&mut bytes[offset..], rustix::rand::GetRandomFlags::NONBLOCK)
        {
            Ok(0) => return Err(Error::Runtime),
            Ok(count) => offset += count,
            Err(rustix::io::Errno::INTR) => {}
            Err(error) => return Err(error.into()),
        }
    }
    bytes[6] = (bytes[6] & 15) | 64;
    bytes[8] = (bytes[8] & 63) | 128;
    let mut output = String::with_capacity(36);
    let alphabet = b"0123456789abcdef";
    for (index, byte) in bytes.into_iter().enumerate() {
        if [4, 6, 8, 10].contains(&index) {
            output.push('-');
        }
        output.push(char::from(alphabet[usize::from(byte >> 4)]));
        output.push(char::from(alphabet[usize::from(byte & 15)]));
    }
    Ok(output)
}

/// The trusted decoder binds native bytes to its decoded row. Import rows use
/// the entire exact TOML source plus ordinal, never native JSON canonicalization.
#[derive(Clone, Copy)]
pub enum RequestSource<'a> {
    Native(&'a [u8]),
    Import(&'a [u8]),
}

impl<'a> RequestSource<'a> {
    fn bytes(self) -> &'a [u8] {
        match self {
            Self::Native(bytes) | Self::Import(bytes) => bytes,
        }
    }
    fn row(self, index: usize) -> Result<Option<u32>> {
        match self {
            Self::Native(_) => Ok(None),
            Self::Import(_) => Ok(Some(u32::try_from(index).map_err(|_| Error::Bound)?)),
        }
    }
    fn request_digest(self, index: usize) -> Result<String> {
        if let Some(row) = self.row(index)? {
            let mut hasher = Sha256::new();
            hasher.update(b"HEE3-roster-import/1\0");
            hasher.update(row.to_be_bytes());
            hasher.update(self.bytes());
            Ok(super::digest_text(&hasher.finalize()))
        } else {
            Ok(digest(self.bytes()))
        }
    }
}

pub type RosterSnapshot = Snapshot;

fn invalid(error: dto::Invalid) -> Error {
    match error {
        dto::Invalid::Bound => Error::Bound,
        _ => Error::Invalid,
    }
}

fn operator(principal: &Principal) -> Result<()> {
    if principal.role == "operator" {
        Ok(())
    } else {
        Err(Error::Forbidden)
    }
}

fn capacity(connection: &Connection, table: &str, additional: usize, limit: usize) -> Result<()> {
    // Callers pass fixed private table names, never imported identifiers.
    let count: u64 = connection.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
        read_number(row, 0)
    })?;
    if count
        .checked_add(u64::try_from(additional).map_err(|_| Error::Bound)?)
        .is_none_or(|total| total > u64::try_from(limit).unwrap_or(0))
    {
        return Err(Error::Bound);
    }
    Ok(())
}

fn record(connection: &Connection, principal: &Principal, id: &str) -> Result<Record> {
    let data: Option<(String, Vec<u8>, bool, Option<String>)> = connection.query_row(
        "SELECT revision,definition,disabled,observation_id FROM roster_records WHERE id=? AND principal_uid=? AND principal_role=?",
        params![id,principal.uid,principal.role], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).optional()?;
    let (revision, definition, disabled, observation_id) = data.ok_or(Error::NotFound)?;
    dto::uuid(id)
        .and_then(|()| dto::generation(&revision))
        .map_err(|_| Error::Corrupt)?;
    let definition: RosterDefinitionV1 = serde_json::from_slice(&definition)?;
    definition.validate().map_err(|_| Error::Corrupt)?;
    let observation = observation_id
        .map(|id| observation(connection, &id))
        .transpose()?;
    let cutoff = observation
        .as_ref()
        .map(|value| value.received.unix_ms.to_string());
    Ok(Record {
        head: RosterHeadV1 {
            record_id: id.to_owned(),
            record_version: revision,
            definition,
            disabled,
            observation_cutoff_unix_ms: cutoff,
        },
        observation,
    })
}

fn observation(connection: &Connection, id: &str) -> Result<Observation> {
    let bytes: Vec<u8> = connection.query_row(
        "SELECT body FROM roster_observations WHERE id=?",
        [id],
        |row| row.get(0),
    )?;
    let observation: Observation = serde_json::from_slice(&bytes)?;
    observation.input.validate().map_err(|_| Error::Corrupt)?;
    dto::uuid(&observation.received.epoch).map_err(|_| Error::Corrupt)?;
    if observation.id != id {
        return Err(Error::Corrupt);
    }
    Ok(observation)
}

fn roster_event(
    tx: &Transaction<'_>,
    id: &str,
    record: &str,
    revision: &str,
    kind: &str,
    body: &[u8],
) -> Result<u64> {
    tx.execute(
        "INSERT INTO events(id,roster_id,generation,kind,body) VALUES(?,?,?,?,?)",
        params![id, record, revision, kind, body],
    )?;
    u64::try_from(tx.last_insert_rowid()).map_err(|_| Error::Bound)
}

fn register_source(tx: &Transaction<'_>, object: &Object) -> Result<()> {
    tx.execute(
        "INSERT INTO artifacts(digest,size) VALUES(?,?) ON CONFLICT(digest) DO NOTHING",
        params![object.digest(), number(object.size())?],
    )?;
    let size: u64 = tx.query_row(
        "SELECT size FROM artifacts WHERE digest=?",
        [object.digest()],
        |row| read_number(row, 0),
    )?;
    if size != object.size() {
        return Err(Error::Corrupt);
    }
    Ok(())
}

fn prior(
    tx: &Connection,
    principal: &Principal,
    action: &str,
    key: &str,
    request_digest: &str,
) -> Result<Option<Outcome>> {
    let row: Option<(String, Vec<u8>)> = tx.query_row("SELECT request_digest,result FROM operations WHERE principal_uid=? AND principal_role=? AND action=? AND version=1 AND request_key=?", params![principal.uid,principal.role,action,key], |row| Ok((row.get(0)?,row.get(1)?))).optional()?;
    row.map(|(old_digest, bytes)| {
        if old_digest != request_digest {
            return Err(Error::Conflict);
        }
        Ok(serde_json::from_slice(&bytes)?)
    })
    .transpose()
}

struct Operation<'a> {
    principal: &'a Principal,
    action: &'a str,
    key: &'a str,
    request_digest: &'a str,
    source: &'a Object,
    row: Option<u32>,
}

fn retain_operation(
    tx: &Transaction<'_>,
    operation: &Operation<'_>,
    outcome: &Outcome,
) -> Result<()> {
    register_source(tx, operation.source)?;
    tx.execute("INSERT INTO operations(principal_uid,principal_role,action,version,request_key,request_digest,roster_id,request_object,request_row,result) VALUES(?,?,?,1,?,?,?,?,?,?)", params![operation.principal.uid,operation.principal.role,operation.action,operation.key,operation.request_digest,outcome.head.record_id,operation.source.digest(),operation.row,serde_json::to_vec(outcome)?])?;
    Ok(())
}

fn retain_revision(tx: &Transaction<'_>, head: &RosterHeadV1, event: &str) -> Result<()> {
    tx.execute(
        "INSERT INTO roster_revisions VALUES(?,?,?,?,?)",
        params![
            head.record_id,
            head.record_version,
            serde_json::to_vec(&head.definition)?,
            head.disabled,
            event
        ],
    )?;
    Ok(())
}

fn apply_update(
    tx: &Transaction<'_>,
    operation: &Operation<'_>,
    update: &Update,
    ids: &(String, String),
    epoch: &str,
    fault: Option<CutPoint>,
) -> Result<Outcome> {
    if let Some(outcome) = prior(
        tx,
        operation.principal,
        operation.action,
        operation.key,
        operation.request_digest,
    )? {
        return Ok(outcome);
    }
    capacity(tx, "roster_revisions", 1, MAX_HISTORY)?;
    let head = if let Some(id) = &update.record_id {
        let existing = record(tx, operation.principal, id)?;
        if update.expected_revision.as_ref() != Some(&existing.head.record_version) {
            return Err(Error::Conflict);
        }
        let revision = next(
            existing
                .head
                .record_version
                .parse()
                .map_err(|_| Error::Corrupt)?,
        )?;
        tx.execute(
            "UPDATE roster_records SET revision=?,definition=?,observation_id=NULL WHERE id=?",
            params![revision, serde_json::to_vec(&update.definition)?, id],
        )?;
        RosterHeadV1 {
            record_id: id.clone(),
            record_version: revision,
            definition: update.definition.clone(),
            disabled: existing.head.disabled,
            observation_cutoff_unix_ms: None,
        }
    } else {
        capacity(tx, "roster_records", 1, MAX_RECORDS)?;
        tx.execute("INSERT INTO roster_records(id,principal_uid,principal_role,revision,definition) VALUES(?,?,?,'1',?)", params![ids.0,operation.principal.uid,operation.principal.role,serde_json::to_vec(&update.definition)?])?;
        RosterHeadV1 {
            record_id: ids.0.clone(),
            record_version: "1".to_owned(),
            definition: update.definition.clone(),
            disabled: false,
            observation_cutoff_unix_ms: None,
        }
    };
    check_point(fault, CutPoint::RosterWrite)?;
    let sequence = roster_event(
        tx,
        &ids.1,
        &head.record_id,
        &head.record_version,
        "roster.update",
        update.audit_reason.as_bytes(),
    )?;
    retain_revision(tx, &head, &ids.1)?;
    let outcome = Outcome {
        head,
        event_id: ids.1.clone(),
        epoch: epoch.to_owned(),
        sequence,
        active_attempts: Vec::new(),
        cancellation_causes: Vec::new(),
    };
    retain_operation(tx, operation, &outcome)?;
    Ok(outcome)
}

fn validate_updates(updates: &[Update], source: RequestSource<'_>) -> Result<()> {
    if updates.len() > MAX_RECORDS || source.bytes().is_empty() || source.bytes().len() > MAX_INPUT
    {
        return Err(Error::Bound);
    }
    if matches!(source, RequestSource::Native(_)) && updates.len() != 1 {
        return Err(Error::Invalid);
    }
    std::str::from_utf8(source.bytes()).map_err(|_| Error::Invalid)?;
    let mut keys = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for update in updates {
        update.validate().map_err(invalid)?;
        if !keys.insert(&update.idempotency_key)
            || update.record_id.as_ref().is_some_and(|id| !ids.insert(id))
        {
            return Err(Error::Invalid);
        }
    }
    Ok(())
}

impl Store {
    /// Apply already decoded updates through one authority and one transaction.
    /// The trusted native decoder or the roster TOML decoder binds rows to source.
    /// # Errors
    /// Any malformed, invisible, stale or conflicting row rolls back the batch.
    pub fn roster_apply(
        &mut self,
        principal: &Principal,
        updates: &[Update],
        source: RequestSource<'_>,
        deadline: Instant,
    ) -> Result<Vec<Outcome>> {
        operator(principal)?;
        validate_updates(updates, source)?;
        remaining(deadline)?;
        if updates.is_empty() {
            return Ok(Vec::new());
        }
        let temp = self.clock.id(deadline)?;
        let object = self.publish(
            source.bytes(),
            UuidV4::parse(&temp).map_err(|_| Error::Runtime)?,
            deadline,
        )?;
        let ids = (0..updates.len())
            .map(|_| Ok((self.clock.id(deadline)?, self.clock.id(deadline)?)))
            .collect::<Result<Vec<_>>>()?;
        let epoch = self.epoch.clone();
        let fault = self.fault();
        self.transaction(deadline, |tx| {
            updates
                .iter()
                .enumerate()
                .map(|(index, update)| {
                    let request_digest = source.request_digest(index)?;
                    let operation = Operation {
                        principal,
                        action: "roster.update",
                        key: &update.idempotency_key,
                        request_digest: &request_digest,
                        source: &object,
                        row: source.row(index)?,
                    };
                    apply_update(tx, &operation, update, &ids[index], &epoch, fault)
                })
                .collect()
        })
    }

    /// # Errors
    /// Missing or invisible records return the same `NotFound` result.
    pub fn roster_get(
        &self,
        principal: &Principal,
        id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Record> {
        schema::bound(&self.connection, deadline)?;
        record(&self.connection, principal, id.as_str())
    }

    /// Read attributable historical proof without promoting it to current health.
    /// # Errors
    /// Missing and invisible observation IDs both return `NotFound`.
    pub fn roster_observation(
        &self,
        principal: &Principal,
        id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Observation> {
        schema::bound(&self.connection, deadline)?;
        let visible:bool=self.connection.query_row("SELECT EXISTS(SELECT 1 FROM roster_observations o JOIN roster_records r ON r.id=o.record_id WHERE o.id=? AND r.principal_uid=? AND r.principal_role=?)",params![id.as_str(),principal.uid,principal.role],|row|row.get(0))?;
        if !visible {
            return Err(Error::NotFound);
        }
        observation(&self.connection, id.as_str())
    }

    /// Read the original disposition before a create's allocated ID is known.
    /// # Errors
    /// Refuses invalid action or invisible principal-scoped operation keys.
    pub fn roster_by_key(
        &self,
        principal: &Principal,
        action: &str,
        key: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Outcome> {
        if !matches!(action, "roster.update" | "roster.disable") {
            return Err(Error::Invalid);
        }
        schema::bound(&self.connection, deadline)?;
        let bytes:Vec<u8>=self.connection.query_row("SELECT result FROM operations WHERE principal_uid=? AND principal_role=? AND action=? AND version=1 AND request_key=?",params![principal.uid,principal.role,action,key.as_str()],|row|row.get(0)).optional()?.ok_or(Error::NotFound)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Capture one bounded read transaction. Later updates cannot alter the value.
    /// # Errors
    /// Refuses expired deadlines, invalid stored values or inventory overflow.
    pub fn roster_snapshot(
        &self,
        principal: &Principal,
        deadline: Instant,
    ) -> Result<RosterSnapshot> {
        schema::bound(&self.connection, deadline)?;
        let tx = self.connection.unchecked_transaction()?;
        let cutoff: u64 =
            tx.query_row("SELECT coalesce(max(sequence),0) FROM events", [], |row| {
                read_number(row, 0)
            })?;
        let now = self.clock.sample()?;
        let ids = {
            let mut statement=tx.prepare("SELECT id FROM roster_records WHERE principal_uid=? AND principal_role=? ORDER BY id LIMIT 257")?;
            statement
                .query_map(params![principal.uid, principal.role], |row| {
                    row.get::<_, String>(0)
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        if ids.len() > MAX_RECORDS {
            return Err(Error::Bound);
        }
        let records = ids
            .iter()
            .map(|id| record(&tx, principal, id))
            .collect::<Result<Vec<_>>>()?;
        remaining(deadline)?;
        tx.commit()?;
        Ok(RosterSnapshot {
            epoch: self.epoch.clone(),
            cutoff,
            now,
            records,
        })
    }

    /// Retain an unconfirmed source claim with the Store's receiver clock.
    /// A caller-controlled source enum cannot qualify capability selection.
    /// # Errors
    /// Refuses wrong owner/version/endpoint/instance, regression or exhausted history.
    pub fn roster_observe(
        &mut self,
        principal: &Principal,
        input: &ObservationInput,
        deadline: Instant,
    ) -> Result<Observation> {
        self.record_roster_observation(principal, input, None, deadline)
    }

    /// Trusted coordinator return from the worker owner; input labels cannot select this path.
    /// # Errors
    /// Refuses a non-worker claim and all shared observation predicates.
    pub fn roster_observe_worker(
        &mut self,
        principal: &Principal,
        input: &ObservationInput,
        deadline: Instant,
    ) -> Result<Observation> {
        self.record_roster_observation(principal, input, Some(ObservationSource::Worker), deadline)
    }

    /// Trusted coordinator return from the useful service-probe owner.
    /// # Errors
    /// Refuses a non-probe claim and all shared observation predicates.
    pub fn roster_observe_service_probe(
        &mut self,
        principal: &Principal,
        input: &ObservationInput,
        deadline: Instant,
    ) -> Result<Observation> {
        self.record_roster_observation(
            principal,
            input,
            Some(ObservationSource::ServiceProbe),
            deadline,
        )
    }

    /// Trusted coordinator return from the provider-response owner.
    /// # Errors
    /// Refuses a non-provider claim and all shared observation predicates.
    pub fn roster_observe_provider_response(
        &mut self,
        principal: &Principal,
        input: &ObservationInput,
        deadline: Instant,
    ) -> Result<Observation> {
        self.record_roster_observation(
            principal,
            input,
            Some(ObservationSource::ProviderResponse),
            deadline,
        )
    }

    fn record_roster_observation(
        &mut self,
        principal: &Principal,
        input: &ObservationInput,
        confirmation: Option<ObservationSource>,
        deadline: Instant,
    ) -> Result<Observation> {
        if confirmation.is_some_and(|source| source != input.source) {
            return Err(Error::Invalid);
        }
        operator(principal)?;
        input.validate().map_err(invalid)?;
        let id = self.clock.id(deadline)?;
        let now = self.clock.sample()?;
        let fault = self.fault();
        self.transaction(deadline,|tx| {
            let existing=record(tx,principal,&input.record_id)?;
            let definition=if let Some(instance_id)=&input.instance_id {
                let instance=attempts::instance(tx,principal,instance_id)?;
                if instance.agent_record_id!=input.record_id || instance.agent_record_version!=input.record_version || Some(&instance.generation)!=input.instance_generation.as_ref() { return Err(Error::Conflict); }
                let pinned:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM roster_pins WHERE attempt_id=? AND record_id=? AND record_version=?)",params![instance.attempt_id,input.record_id,input.record_version],|row|row.get(0))?;
                if !pinned { return Err(Error::Conflict); }
                let raw:Vec<u8>=tx.query_row("SELECT definition FROM roster_revisions WHERE record_id=? AND revision=?",params![input.record_id,input.record_version],|row|row.get(0))?;
                serde_json::from_slice::<RosterDefinitionV1>(&raw)?
            } else {
                if input.record_version!=existing.head.record_version { return Err(Error::Conflict); }
                existing.head.definition.clone()
            };
            if input.owner_id!=definition.owner_id || input.endpoint_ref!=definition.endpoint_ref { return Err(Error::Conflict); }
            if existing.observation.as_ref().is_some_and(|old|old.received.epoch==now.epoch && (old.received.monotonic_ms>now.monotonic_ms || old.received.unix_ms>now.unix_ms)) { return Err(Error::Conflict); }
            capacity(tx,"roster_observations",1,MAX_HISTORY)?;
            let sequence=roster_event(tx,&id,&input.record_id,&input.record_version,"roster.observed",input.evidence_ref.as_bytes())?;
            let observation=Observation { id:id.clone(),confirmed_source:confirmation,input:input.clone(),received:now,sequence };
            tx.execute("INSERT INTO roster_observations VALUES(?,?,?,?,?,?)",params![id,input.record_id,input.record_version,input.instance_id,serde_json::to_vec(&observation)?,number(sequence)?])?;
            if input.instance_id.is_none() {
                tx.execute("UPDATE roster_records SET observation_id=? WHERE id=?",params![id,input.record_id])?;
            }
            check_point(fault,CutPoint::RosterObservation)?;
            Ok(observation)
        })
    }
}
