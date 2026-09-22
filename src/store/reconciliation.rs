//! Durable reconciliation records for the startup pass, in the ledger's own event
//! journal. A record is an event of kind `reconciliation_decided` or
//! `reconciliation_readback` whose id is derived from its content, so the same
//! decision over the same world can be recorded once and never twice; the second
//! recording finds the first and reports it as not fresh. Recording confers no
//! dispatch, settlement or acceptance authority; the one ledger effect a record
//! may carry is settling an `unknown` attempt's cleanup column after a readback
//! said the cleanup is complete, and it commits with the readback record.

use super::{Error, Object, Result, Store, digest_text, read_number, remaining, schema};
use crate::contracts::UuidV4;
use crate::contracts::roster::ReceiptTime;
use rusqlite::{OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::time::Instant;

/// Largest record body the ledger accepts, in bytes.
pub const RECORD_BODY_LIMIT: usize = 65_536;

/// Which of the two record kinds an event row is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordKind {
    /// The policy's decision and the intended action, written before any effect.
    Decided,
    /// What a readback found after an effect, written after it.
    Readback,
}

impl RecordKind {
    /// The `events.kind` spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Decided => "reconciliation_decided",
            Self::Readback => "reconciliation_readback",
        }
    }
    /// Parse the `events.kind` spelling; any other text is `None`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "reconciliation_decided" => Some(Self::Decided),
            "reconciliation_readback" => Some(Self::Readback),
            _ => None,
        }
    }
}

/// One record to write for one attempt.
#[derive(Clone, Copy, Debug)]
pub struct ReconciliationRecord<'a> {
    pub attempt: UuidV4<'a>,
    pub kind: RecordKind,
    /// A JSON object, at most [`RECORD_BODY_LIMIT`] bytes.
    pub body: &'a [u8],
    /// Settle the attempt's `cleanup` column in the same transaction. Permitted
    /// only for a `Readback` record on an `unknown` attempt whose cleanup is not
    /// yet settled; any other request is refused whole.
    pub settle_cleanup: bool,
}

/// What recording did.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct Recorded {
    pub event: String,
    pub sequence: u64,
    /// `true` when this call inserted the row; `false` when an identical record
    /// already existed and nothing was written.
    pub fresh: bool,
    pub cleanup_settled: bool,
}

/// A recorded row read back from the journal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordRow {
    pub event: String,
    pub sequence: u64,
    pub kind: RecordKind,
    pub generation: String,
    pub body: Vec<u8>,
}

/// The event ordinals of a task's committed terminal history, when present.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalOrdinals {
    pub accepted: Option<(String, u64)>,
    pub cancellation: Option<u64>,
}

/// Whether an evidence object named by a verification row is retrievable now.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceAvailability {
    Published,
    Absent,
}

/// Content-derived event id: the digest of the record's coordinates, spelled as
/// the ledger's UUID grammar so every existing id check accepts it. Two records
/// with the same epoch, attempt, kind and body share one id, which is the
/// mechanism that makes a duplicate write unrepresentable.
#[must_use]
pub fn record_id(epoch: &str, attempt: &str, kind: RecordKind, body: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"hee3-reconciliation-record/1\0");
    hasher.update(epoch.as_bytes());
    hasher.update(b"\0");
    hasher.update(attempt.as_bytes());
    hasher.update(b"\0");
    hasher.update(kind.name().as_bytes());
    hasher.update(b"\0");
    hasher.update(body);
    let hex = digest_text(&hasher.finalize());
    let hex = &hex["sha256:".len()..];
    let mut id = String::with_capacity(36);
    for (index, byte) in hex.bytes().take(32).enumerate() {
        if matches!(index, 8 | 12 | 16 | 20) {
            id.push('-');
        }
        id.push(match index {
            12 => '4',
            16 => '8',
            _ => char::from(byte),
        });
    }
    id
}

fn body_is_object(body: &[u8]) -> bool {
    !body.is_empty()
        && body.len() <= RECORD_BODY_LIMIT
        && serde_json::from_slice::<serde_json::Value>(body).is_ok_and(|value| value.is_object())
}

impl Store {
    /// Record one reconciliation event for an attempt, once. An identical record
    /// (same epoch, attempt, kind and body) is found rather than written again.
    /// # Errors
    /// Refuses a missing attempt, a body that is not a bounded JSON object, an
    /// inspection-only or non-normal ledger, and a cleanup settlement the row's
    /// state does not admit.
    pub fn record_reconciliation(
        &mut self,
        record: &ReconciliationRecord<'_>,
        deadline: Instant,
    ) -> Result<Recorded> {
        if !body_is_object(record.body) {
            return Err(Error::Bound);
        }
        if record.settle_cleanup && record.kind != RecordKind::Readback {
            return Err(Error::Invalid);
        }
        let epoch = self.epoch.clone();
        let id = record_id(&epoch, record.attempt.as_str(), record.kind, record.body);
        self.transaction(deadline, |tx| {
            let (task, state, cleanup): (String, String, String) = tx
                .query_row(
                    "SELECT task_id,state,cleanup FROM attempts WHERE id=?",
                    [record.attempt.as_str()],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?
                .ok_or(Error::NotFound)?;
            let existing: Option<i64> = tx
                .query_row("SELECT sequence FROM events WHERE id=?", [&id], |row| {
                    row.get(0)
                })
                .optional()?;
            if let Some(sequence) = existing {
                let sequence = u64::try_from(sequence).map_err(|_| Error::Corrupt)?;
                return Ok(Recorded {
                    event: id.clone(),
                    sequence,
                    fresh: false,
                    cleanup_settled: false,
                });
            }
            let generation: String =
                tx.query_row("SELECT generation FROM tasks WHERE id=?", [&task], |row| {
                    row.get(0)
                })?;
            let mut cleanup_settled = false;
            if record.settle_cleanup {
                if state != "unknown" || cleanup == "settled" {
                    return Err(Error::Conflict);
                }
                if tx.execute(
                    "UPDATE attempts SET cleanup='settled' WHERE id=? AND state='unknown' AND cleanup!='settled'",
                    [record.attempt.as_str()],
                )? != 1
                {
                    return Err(Error::Conflict);
                }
                cleanup_settled = true;
            }
            tx.execute(
                "INSERT INTO events(id,task_id,generation,kind,body) VALUES(?,?,?,?,?)",
                params![id, task, generation, record.kind.name(), record.body],
            )?;
            let sequence = u64::try_from(tx.last_insert_rowid()).map_err(|_| Error::Bound)?;
            Ok(Recorded {
                event: id.clone(),
                sequence,
                fresh: true,
                cleanup_settled,
            })
        })
    }

    /// Every reconciliation record of one attempt, in journal order.
    /// # Errors
    /// Refuses a poisoned store, an expired deadline or a malformed row.
    pub fn reconciliation_records(
        &self,
        attempt: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Vec<RecordRow>> {
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        schema::bound(&self.connection, deadline)?;
        let mut statement = self.connection.prepare(
            "SELECT e.id,e.sequence,e.kind,e.generation,e.body FROM events e WHERE e.task_id=(SELECT task_id FROM attempts WHERE id=?) AND e.kind IN ('reconciliation_decided','reconciliation_readback') ORDER BY e.sequence LIMIT 4097",
        )?;
        let rows = statement.query_map([attempt.as_str()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                read_number(row, 1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        })?;
        let mut records = Vec::new();
        for row in rows {
            remaining(deadline)?;
            let (event, sequence, kind, generation, body) = row?;
            let kind = RecordKind::parse(&kind).ok_or(Error::Corrupt)?;
            let value: serde_json::Value = serde_json::from_slice(&body)?;
            if value["attempt"].as_str() != Some(attempt.as_str()) {
                continue;
            }
            records.push(RecordRow {
                event,
                sequence,
                kind,
                generation,
                body,
            });
            if records.len() > 4096 {
                return Err(Error::Bound);
            }
        }
        Ok(records)
    }

    /// The event ordinals of a task's committed acceptance and cancellation.
    /// # Errors
    /// Refuses a poisoned store, an expired deadline or an unknown task.
    pub fn terminal_ordinals(
        &self,
        task: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<TerminalOrdinals> {
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        schema::bound(&self.connection, deadline)?;
        let known: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?)",
            [task.as_str()],
            |row| row.get(0),
        )?;
        if !known {
            return Err(Error::NotFound);
        }
        let accepted = self
            .connection
            .query_row(
                "SELECT id,sequence FROM events WHERE task_id=? AND kind='accepted' ORDER BY sequence LIMIT 1",
                [task.as_str()],
                |row| Ok((row.get::<_, String>(0)?, read_number(row, 1)?)),
            )
            .optional()?;
        let cancellation = self
            .connection
            .query_row(
                "SELECT sequence FROM events WHERE task_id=? AND kind='cancellation_requested' ORDER BY sequence LIMIT 1",
                [task.as_str()],
                |row| read_number(row, 0),
            )
            .optional()?;
        Ok(TerminalOrdinals {
            accepted,
            cancellation,
        })
    }

    /// Whether the evidence object a verification row names can be read back now:
    /// its registered size is looked up and the bytes are verified against it.
    /// # Errors
    /// Refuses a poisoned store or an expired deadline; a missing or corrupt object
    /// is `Absent`, not an error.
    pub fn evidence_available(
        &self,
        digest: &str,
        deadline: Instant,
    ) -> Result<EvidenceAvailability> {
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        schema::bound(&self.connection, deadline)?;
        let size: Option<u64> = self
            .connection
            .query_row(
                "SELECT size FROM artifacts WHERE digest=?",
                [digest],
                |row| read_number(row, 0),
            )
            .optional()?;
        let Some(size) = size else {
            return Ok(EvidenceAvailability::Absent);
        };
        let object = Object {
            digest: digest.to_owned(),
            size,
        };
        match self.read_object(&object, deadline) {
            Ok(_) => Ok(EvidenceAvailability::Published),
            Err(Error::Deadline) => Err(Error::Deadline),
            Err(_) => Ok(EvidenceAvailability::Absent),
        }
    }

    /// This open's receiver clock reading, in its own epoch. The epoch is minted
    /// at open, so a lease retained from an earlier open is not comparable to it.
    /// # Errors
    /// Refuses an unrepresentable elapsed time.
    pub fn receiver_clock(&self) -> Result<ReceiptTime> {
        self.clock.sample()
    }
}
