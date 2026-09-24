//! Quiesced store snapshots. Packaging/service/effect restore policy remains T18.

use super::{
    CutPoint, Directory, Error, Object, Result, Store, digest, digest_text, read_number, remaining,
    schema,
};
use crate::contracts::Sha256Digest;
use rusqlite::{
    Connection, OpenFlags,
    backup::{Backup, StepResult},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

/// This store snapshot does not qualify operational restoration by itself.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RestoreStatus {
    Unqualified,
}

const BYTE_LIMIT: u64 = 32 * 1024 * 1024 * 1024;

/// A verified store snapshot; service/configuration and external effect recovery
/// must be assembled and qualified separately before operational restore.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupReport {
    pub scope: String,
    pub operational_status: RestoreStatus,
    pub epoch: String,
    pub generation: String,
    pub cutoff: u64,
    pub database_sha256: String,
    pub database_bytes: u64,
    pub objects: Vec<Object>,
    pub counts: BTreeMap<String, u64>,
    pub step_done: bool,
    pub finish_call_via_drop: bool,
    pub finish_return_observed: bool,
}

fn file_hash(mut file: File, deadline: Instant) -> Result<(String, u64)> {
    let length = file.metadata()?.len();
    if length > BYTE_LIMIT {
        return Err(Error::Bound);
    }
    let mut hash = Sha256::new();
    let mut total = 0;
    let mut buffer = vec![0_u8; 65_536];
    loop {
        remaining(deadline)?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > length {
            return Err(Error::Corrupt);
        }
        hash.update(&buffer[..count]);
    }
    if total != length {
        return Err(Error::Corrupt);
    }
    Ok((digest_text(&hash.finalize()), length))
}

fn counts(connection: &Connection) -> Result<BTreeMap<String, u64>> {
    let mut result = BTreeMap::new();
    for name in [
        "tasks",
        "operations",
        "attempts",
        "events",
        "artifacts",
        "acceptances",
        "acceptance_objects",
        "outbox",
        "roster_records",
        "roster_revisions",
        "roster_observations",
        "roster_instances",
        "roster_instance_history",
        "roster_pins",
        "roster_cancel_causes",
    ] {
        let query = format!("SELECT count(*) FROM {name}");
        result.insert(
            name.to_owned(),
            connection.query_row(&query, [], |row| read_number(row, 0))?,
        );
    }
    Ok(result)
}

impl Store {
    /// Copy a quiesced ledger and all its registered proof objects into a fresh
    /// private directory. Publish the store manifest only after independent readback.
    /// No current generation is overwritten and no dispatch is started.
    /// # Errors
    /// Refuses outstanding worker custody, expired bounds, partial/existing destinations
    /// or any missing/corrupt database, object or inventory member.
    pub fn backup(&mut self, destination: &Path, deadline: Instant) -> Result<BackupReport> {
        remaining(deadline)?;
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        schema::bound(&self.connection, deadline)?;
        let outstanding:bool=self.connection.query_row("SELECT EXISTS(SELECT 1 FROM attempts WHERE state!='settled' OR effect IN ('pending','unknown') OR cleanup NOT IN ('none','settled'))",[],|row|row.get(0))?;
        if outstanding {
            return Err(Error::Outstanding);
        }
        let dest = Directory::root(destination)?;
        if dest.path.starts_with(&self.root.path) || self.root.path.starts_with(&dest.path) {
            return Err(Error::Custody);
        }
        if let Some(entry) = std::fs::read_dir(&dest.path)?.next() {
            entry?;
            return Err(Error::Conflict);
        }
        let (database_sha256, database_bytes) = self.copy_database(&dest, deadline)?;
        let source_generation = self
            .generation
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(Error::Custody)?
            .to_owned();
        let reopened = Connection::open_with_flags(
            dest.path.join("ledger.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_ONLY
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        reopened.execute_batch("PRAGMA query_only=ON;")?;
        schema::validate(&reopened, &source_generation, deadline)?;
        let inventory = counts(&self.connection)?;
        if inventory != counts(&reopened)? {
            return Err(Error::Corrupt);
        }
        reopened
            .close()
            .map_err(|(_, error)| Error::Sqlite(error))?;
        let objects = self.copy_objects(&dest, database_bytes, deadline)?;
        let cutoff: u64 = self.connection.query_row(
            "SELECT coalesce(max(sequence),0) FROM events",
            [],
            |row| read_number(row, 0),
        )?;
        let report = BackupReport {
            scope: "hee3-store-backup/1".to_owned(),
            operational_status: RestoreStatus::Unqualified,
            epoch: self.epoch.clone(),
            generation: source_generation,
            cutoff,
            database_sha256,
            database_bytes,
            objects,
            counts: inventory,
            step_done: true,
            finish_call_via_drop: true,
            finish_return_observed: false,
        };
        let identity = publish_report(&dest, &report, self.fault())?;
        let readback = Self::inspect_backup(
            destination,
            Sha256Digest::parse(&identity).map_err(|_| Error::Corrupt)?,
            deadline,
        )?;
        if report != readback {
            return Err(Error::Corrupt);
        }
        Ok(report)
    }

    fn copy_objects(
        &self,
        dest: &Directory,
        database_bytes: u64,
        deadline: Instant,
    ) -> Result<Vec<Object>> {
        let mut statement = self
            .connection
            .prepare("SELECT digest,size FROM artifacts ORDER BY digest LIMIT 4097")?;
        let objects = statement
            .query_map([], |row| {
                Ok(Object {
                    digest: row.get(0)?,
                    size: read_number(row, 1)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if objects.len() > 4096 {
            return Err(Error::Bound);
        }
        let total = objects.iter().try_fold(database_bytes, |sum, object| {
            sum.checked_add(object.size).ok_or(Error::Bound)
        })?;
        if total > BYTE_LIMIT {
            return Err(Error::Bound);
        }
        let object_root = dest.child("objects", true)?.child("sha256", true)?;
        for object in &objects {
            remaining(deadline)?;
            let bytes = self.read_object(object, deadline)?;
            let staging =
                crate::contracts::UuidV4::parse(&self.epoch).map_err(|_| Error::Corrupt)?;
            let published = super::artifact::publish(&object_root, &bytes, staging, |point| {
                cut_point!(self.fault(), point);
                Ok(())
            })?;
            if &published != object {
                return Err(Error::Corrupt);
            }
            cut_point!(self.fault(), CutPoint::BackupObject);
        }
        Ok(objects)
    }

    fn copy_database(&self, dest: &Directory, deadline: Instant) -> Result<(String, u64)> {
        let file = dest.create_file("ledger.sqlite3")?;
        file.sync_all()?;
        dest.file.sync_all()?;
        drop(file);
        let mut copy = Connection::open_with_flags(
            dest.path.join("ledger.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        copy.busy_timeout(remaining(deadline)?.min(Duration::from_secs(5)))?;
        {
            let backup = Backup::new(&self.connection, &mut copy)?;
            loop {
                remaining(deadline)?;
                match backup.step(64)? {
                    StepResult::Done => break,
                    StepResult::More => (),
                    StepResult::Busy | StepResult::Locked => {
                        std::thread::sleep(remaining(deadline)?.min(Duration::from_millis(10)));
                    }
                    _ => return Err(Error::Corrupt),
                }
            }
            // rusqlite0.40.2 Drop calls backup_finish but does not expose its rc.
            // SQLite specifies OK after a checked Done with no retained step error.
            // Record that distinction and require fallible close/reopen/readback below.
        }
        cut_point!(self.fault(), CutPoint::BackupCopied);
        let journal: String = copy.query_row("PRAGMA journal_mode=DELETE", [], |row| row.get(0))?;
        if journal != "delete" {
            return Err(Error::Corrupt);
        }
        copy.execute_batch("PRAGMA synchronous=EXTRA;")?;
        copy.close().map_err(|(_, error)| Error::Sqlite(error))?;
        let file = dest.regular("ledger.sqlite3")?;
        file.sync_all()?;
        dest.file.sync_all()?;
        file_hash(file, deadline)
    }

    /// Validate a closed store snapshot without installing or replaying it.
    /// # Errors
    /// Refuses missing manifest, changed bytes, missing objects or inconsistent logical state.
    pub fn inspect_backup(
        path: &Path,
        expected: Sha256Digest<'_>,
        deadline: Instant,
    ) -> Result<BackupReport> {
        remaining(deadline)?;
        schema::runtime(deadline)?;
        let root = Directory::root(path)?;
        let file = root.regular("store-backup.json")?;
        if file.metadata()?.len() > 1_048_576 {
            return Err(Error::Bound);
        }
        let mut bytes = Vec::new();
        file.take(1_048_577).read_to_end(&mut bytes)?;
        if digest(&bytes) != expected.as_str() {
            return Err(Error::Corrupt);
        }
        let report: BackupReport = serde_json::from_slice(&bytes)?;
        if report.scope != "hee3-store-backup/1"
            || report.operational_status != RestoreStatus::Unqualified
            || !report.step_done
            || !report.finish_call_via_drop
            || report.finish_return_observed
            || report.objects.len() > 4096
        {
            return Err(Error::Invalid);
        }
        let total = report
            .objects
            .iter()
            .try_fold(report.database_bytes, |sum, object| {
                sum.checked_add(object.size).ok_or(Error::Bound)
            })?;
        if total > BYTE_LIMIT {
            return Err(Error::Bound);
        }
        let (hash, length) = file_hash(root.regular("ledger.sqlite3")?, deadline)?;
        if hash != report.database_sha256 || length != report.database_bytes {
            return Err(Error::Corrupt);
        }
        let connection = Connection::open_with_flags(
            root.path.join("ledger.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_ONLY
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        connection.execute_batch("PRAGMA query_only=ON;")?;
        schema::validate(&connection, &report.generation, deadline)?;
        let epoch: String =
            connection.query_row("SELECT epoch FROM ledger_meta", [], |row| row.get(0))?;
        let cutoff: u64 =
            connection.query_row("SELECT coalesce(max(sequence),0) FROM events", [], |row| {
                read_number(row, 0)
            })?;
        if epoch != report.epoch || cutoff != report.cutoff || counts(&connection)? != report.counts
        {
            return Err(Error::Corrupt);
        }
        let mut statement =
            connection.prepare("SELECT digest,size FROM artifacts ORDER BY digest LIMIT 4097")?;
        let actual = statement
            .query_map([], |row| {
                Ok(Object {
                    digest: row.get(0)?,
                    size: read_number(row, 1)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if actual != report.objects {
            return Err(Error::Corrupt);
        }
        let objects = root.child("objects", false)?.child("sha256", false)?;
        for object in &actual {
            remaining(deadline)?;
            super::artifact::verify(&objects, object)?;
        }
        drop(statement);
        connection
            .close()
            .map_err(|(_, error)| Error::Sqlite(error))?;
        remaining(deadline)?;
        Ok(report)
    }
}

fn publish_report(dest: &Directory, report: &BackupReport, fault: super::Fault) -> Result<String> {
    let bytes = serde_json::to_vec(report)?;
    if bytes.len() > 1_048_576 {
        return Err(Error::Bound);
    }
    let mut manifest = dest.create_file(".manifest-stage")?;
    manifest.write_all(&bytes)?;
    manifest.sync_all()?;
    drop(manifest);
    cut_point!(fault, CutPoint::BackupManifest);
    rustix::fs::renameat_with(
        &dest.file,
        ".manifest-stage",
        &dest.file,
        "store-backup.json",
        rustix::fs::RenameFlags::NOREPLACE,
    )?;
    dest.file.sync_all()?;
    Ok(digest(&bytes))
}
