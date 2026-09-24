//! Independent T04 store-primary contract tests on disposable private roots.
//! SQLite readbacks inspect committed facts; no worker, provider or restore runs.

use super::*;
use std::fs::{self, DirBuilder, Permissions};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt, symlink};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const GEN: &str = "00000000-0000-4000-8000-000000000001";
const EPOCH: &str = "00000000-0000-4000-8000-000000000002";
const TASK: &str = "00000000-0000-4000-8000-000000000003";
const KEY: &str = "00000000-0000-4000-8000-000000000004";
const ADMITTED: &str = "00000000-0000-4000-8000-000000000005";
const ATTEMPT: &str = "00000000-0000-4000-8000-000000000006";
const STARTED: &str = "00000000-0000-4000-8000-000000000007";
const SETTLED: &str = "00000000-0000-4000-8000-000000000008";
const ACCEPTED: &str = "00000000-0000-4000-8000-000000000009";
const CANCELLED: &str = "00000000-0000-4000-8000-00000000000a";
const STAGE: &str = "00000000-0000-4000-8000-00000000000b";
const OTHER: &str = "00000000-0000-4000-8000-00000000000c";
const CRITERIA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HELLO: &str = "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
/// Migration 1's identity: SHA-256 of the bytes after `-- HEE3-ANCHORS-END\n`, computed by
/// `sha256sum` over that suffix at 0ec69d6, f74fb14, d6cd92c and 20bd971 (identical at all four
/// while the whole-file digest changed three times). Not derived from the crate's own digest.
const MIGRATION_1_BODY: &str =
    "sha256:ac5916feaee05749404dd7d87d98cde7e2ae93048e8b07e133ba7868fc1ee9f2";
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct Area {
    path: PathBuf,
    inode: u64,
}
impl Area {
    fn new() -> Self {
        let base = std::env::temp_dir().canonicalize().unwrap();
        let path = base.join(format!(
            "hee3-t04-test-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
        let inode = fs::metadata(&path).unwrap().ino();
        Self { path, inode }
    }
    fn open(&self) -> Store {
        Store::open(&self.path, uuid(GEN), uuid(EPOCH), true, deadline()).unwrap()
    }
    fn reopen(&self) -> Store {
        Store::open(&self.path, uuid(GEN), uuid(EPOCH), false, deadline()).unwrap()
    }
    fn database(&self) -> PathBuf {
        self.path
            .join("generations")
            .join(GEN)
            .join("ledger.sqlite3")
    }
    fn inspect(&self) -> Connection {
        let db = Connection::open_with_flags(
            self.database(),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        db.execute_batch("PRAGMA query_only=ON;").unwrap();
        db
    }
    fn edit_closed(&self, sql: &str) {
        let db = Connection::open_with_flags(
            self.database(),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        db.execute_batch(sql).unwrap();
        db.close().unwrap();
    }
    fn object_path(&self, object: &Object) -> PathBuf {
        let hex = &object.digest()[7..];
        self.path
            .join("generations")
            .join(GEN)
            .join("objects/sha256")
            .join(&hex[..2])
            .join(hex)
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        // Areas are declared before database handles, so handles close first.
        // Never follow a substituted fixture root during cleanup.
        let owned = fs::symlink_metadata(&self.path)
            .is_ok_and(|meta| meta.is_dir() && meta.ino() == self.inode);
        let result = if owned {
            fs::remove_dir_all(&self.path)
        } else {
            return;
        };
        if !std::thread::panicking() {
            result.unwrap();
        }
    }
}
fn uuid(text: &str) -> UuidV4<'_> {
    UuidV4::parse(text).unwrap()
}
fn revision(value: u64) -> Generation {
    value.to_string().parse().unwrap()
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn principal() -> Principal {
    Principal::new(1000, "operator").unwrap()
}
fn allocation() -> Allocation {
    Allocation {
        limit_ms: 1000,
        work_ms: 800,
        verify_ms: 200,
    }
}
fn submission(owner: &Principal) -> Submission<'_> {
    Submission {
        principal: owner,
        key: uuid(KEY),
        task: uuid(TASK),
        event: uuid(ADMITTED),
        request_bytes: b"canonical request",
        criteria: Sha256Digest::parse(CRITERIA).unwrap(),
        allocation: allocation(),
    }
}
fn admit(store: &mut Store) -> Admission {
    store.submit(submission(&principal()), deadline()).unwrap()
}
fn head_of(store: &Store) -> TaskHead {
    store.get(&principal(), uuid(TASK), deadline()).unwrap()
}
fn expected(task_generation: u64, attempt_generation: u64) -> Expected<'static> {
    Expected {
        task: uuid(TASK),
        task_generation: revision(task_generation),
        attempt: uuid(ATTEMPT),
        attempt_generation: revision(attempt_generation),
    }
}
fn running(store: &mut Store) -> Expected<'static> {
    admit(store);
    let started = store
        .begin_attempt(
            uuid(TASK),
            revision(1),
            uuid(ATTEMPT),
            uuid(STARTED),
            deadline(),
        )
        .unwrap();
    assert_eq!(
        (
            started.task_generation.as_str(),
            started.generation.as_str()
        ),
        ("2", "1")
    );
    expected(2, 1)
}
fn settled(
    effect: Effect,
    used_ms: Option<u64>,
    cleanup_settled: bool,
    ready_to_verify: bool,
) -> Settlement {
    Settlement {
        effect,
        used_ms,
        cleanup_settled,
        ready_to_verify,
    }
}
fn verifying(store: &mut Store) -> Expected<'static> {
    let active = running(store);
    assert_eq!(
        store
            .settle_attempt(
                &active,
                settled(Effect::None, Some(30), true, true),
                uuid(SETTLED),
                deadline()
            )
            .unwrap(),
        "3"
    );
    expected(3, 1)
}
fn proof(store: &Store, active: &Expected<'_>) -> PublishedAcceptance {
    let object = store.publish(b"hello", uuid(STAGE), deadline()).unwrap();
    store
        .prepare_acceptance(active, uuid(ACCEPTED), &[object], deadline())
        .unwrap()
}
fn accepted(store: &mut Store) -> PublishedAcceptance {
    let active = verifying(store);
    let published = proof(store, &active);
    assert_eq!(store.accept(&published, 20, deadline()).unwrap(), 4);
    published
}
fn count(area: &Area, table: &str) -> i64 {
    area.inspect()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}
fn no_admission(area: &Area) {
    for table in ["tasks", "operations", "attempts", "events", "outbox"] {
        assert_eq!(count(area, table), 0, "{table}");
    }
}
fn no_acceptance(area: &Area) {
    for table in ["acceptances", "acceptance_objects", "artifacts", "outbox"] {
        assert_eq!(count(area, table), 0, "{table}");
    }
}
fn injected<T>(result: Result<T>, point: CutPoint) {
    match result {
        Err(Error::Injected(name)) => assert_eq!(name, format!("{point:?}")),
        _ => panic!("expected injected {point:?}"),
    }
}
fn backup_digest(area: &Area) -> String {
    digest(&fs::read(area.path.join("store-backup.json")).unwrap())
}
fn inspect_backup(area: &Area) -> Result<BackupReport> {
    let identity = backup_digest(area);
    Store::inspect_backup(
        &area.path,
        Sha256Digest::parse(&identity).unwrap(),
        deadline(),
    )
}

/// Exact runtime/profile and complete initial migration are independently visible.
#[test]
fn fresh_ledger_has_exact_runtime_profile_and_migration() {
    let area = Area::new();
    let store = area.open();
    assert_eq!(rusqlite::version(), "3.53.4");
    let source: String = store
        .connection
        .query_row("SELECT sqlite_source_id()", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        source,
        "2026-07-24 19:02:57 bf7c7f30031888f4e796e429ab3978879485813aaca6f641c7b33e4e09459bcc"
    );
    for (query, expected) in [
        ("PRAGMA synchronous", 2),
        ("PRAGMA foreign_keys", 1),
        ("PRAGMA read_uncommitted", 0),
        ("PRAGMA wal_autocheckpoint", 1000),
    ] {
        assert_eq!(
            store
                .connection
                .query_row(query, [], |row| row.get::<_, i64>(0))
                .unwrap(),
            expected
        );
    }
    let db = area.inspect();
    assert_eq!(
        db.query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "wal"
    );
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    let history: (i64,String,i64,Option<String>) = db.query_row("SELECT version,checksum,predecessor_version,predecessor_checksum FROM migration_history", [], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).unwrap();
    assert_eq!(history, (1, MIGRATION_1_BODY.to_owned(), 0, None));
    assert_eq!(count(&area, "migration_history"), 1);
    no_admission(&area);
}

/// Restart preserves committed admission, allocation and epoch.
#[test]
fn reopening_preserves_admission_and_allocations() {
    let area = Area::new();
    let mut store = area.open();
    let admission = admit(&mut store);
    drop(store);
    let reopened = area.reopen();
    let head = head_of(&reopened);
    assert_eq!(admission.epoch, EPOCH);
    assert_eq!(head.generation, "1");
    assert_eq!(
        (
            head.spent_ms,
            head.reserved_work_ms,
            head.reserved_verify_ms
        ),
        (0, 800, 200)
    );
    assert_eq!(
        reopened
            .get_by_key(&principal(), uuid(KEY), deadline())
            .unwrap(),
        head
    );
}

#[test]
fn wrong_epoch_refuses_without_replacing_persisted_epoch() {
    let area = Area::new();
    drop(area.open());
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(OTHER), false, deadline()),
        Err(Error::Conflict)
    ));
    assert_eq!(
        area.inspect()
            .query_row("SELECT epoch FROM ledger_meta", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        EPOCH
    );
    drop(area.reopen());
}

#[test]
fn second_writer_refuses_and_stable_lock_inode_survives_restart() {
    let area = Area::new();
    let mut store = area.open();
    let inode = fs::metadata(area.path.join("store.lock")).unwrap().ino();
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()),
        Err(Error::Locked)
    ));
    admit(&mut store);
    drop(store);
    drop(area.reopen());
    assert_eq!(
        fs::metadata(area.path.join("store.lock")).unwrap().ino(),
        inode
    );
}

#[test]
fn symlink_root_refuses_before_database_creation() {
    let actual = Area::new();
    let links = Area::new();
    let alias = links.path.join("alias");
    symlink(&actual.path, &alias).unwrap();
    assert!(matches!(
        Store::open(&alias, uuid(GEN), uuid(EPOCH), true, deadline()),
        Err(Error::Custody)
    ));
    assert!(!actual.path.join("store.lock").exists());
    drop(actual.open());
}

#[test]
fn nonprivate_root_refuses_and_private_neighbor_opens() {
    let area = Area::new();
    fs::set_permissions(&area.path, Permissions::from_mode(0o750)).unwrap();
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), true, deadline()),
        Err(Error::Custody)
    ));
    assert!(!area.path.join("store.lock").exists());
    fs::set_permissions(&area.path, Permissions::from_mode(0o700)).unwrap();
    drop(area.open());
}

#[test]
fn hardlinked_mutable_ledger_is_not_admitted() {
    let area = Area::new();
    drop(area.open());
    let alias = area.path.join("linked-ledger");
    fs::hard_link(area.database(), &alias).unwrap();
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()),
        Err(Error::Custody)
    ));
    fs::remove_file(alias).unwrap();
    drop(area.reopen());
}

/// SQLite opens a file it cannot write read-only, silently, whatever the open flags asked for.
/// A writable open reads back whether it can take the write lock: a ledger that cannot take a
/// write is refused by name at open, not discovered as an unexplained write failure at the first
/// admission. An inspection open is read-only by design and still opens and reads.
#[test]
fn unwritable_ledger_refuses_a_writable_open_and_still_inspects() {
    let area = Area::new();
    drop(area.open());
    fs::set_permissions(area.database(), Permissions::from_mode(0o400)).unwrap();
    let refused = Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline());
    assert!(
        matches!(refused, Err(Error::NotWritable)),
        "{:?}",
        refused.err()
    );
    // The benign mirror: inspection is read-only by design, opens, and reads the ledger back.
    let mut inspected =
        Store::open_inspection(&area.path, uuid(GEN), uuid(EPOCH), deadline()).unwrap();
    let inventory = inspected
        .recovery_inventory(
            uuid(EPOCH),
            crate::app::coordinator::START_LIMITS,
            deadline(),
        )
        .unwrap();
    assert_eq!(
        (inventory.mode.as_str(), inventory.generation.as_str()),
        ("normal", GEN)
    );
    drop(inspected);
    // SQLite gave the WAL index the inspection created the file's mode: repairing the file alone
    // leaves a ledger that cannot take a write, and it is refused by the same name.
    let shm = area.database().with_extension("sqlite3-shm");
    assert_eq!(
        fs::metadata(&shm).unwrap().permissions().mode() & 0o777,
        0o400
    );
    fs::set_permissions(area.database(), Permissions::from_mode(0o600)).unwrap();
    let refused = Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline());
    assert!(
        matches!(refused, Err(Error::NotWritable)),
        "{:?}",
        refused.err()
    );
    fs::set_permissions(&shm, Permissions::from_mode(0o600)).unwrap();
    assert_eq!(admit(&mut area.reopen()).sequence, 1);
}

#[test]
fn symlink_lock_cannot_transfer_writer_custody() {
    let area = Area::new();
    let neighbor = Area::new();
    let target = neighbor.path.join("lock");
    fs::write(&target, b"retained").unwrap();
    symlink(&target, area.path.join("store.lock")).unwrap();
    assert!(Store::open(&area.path, uuid(GEN), uuid(EPOCH), true, deadline()).is_err());
    assert_eq!(fs::read(target).unwrap(), b"retained");
    assert!(!area.path.join("generations").exists());
}

#[test]
fn unrelated_version_zero_database_is_preserved_and_refused() {
    let area = Area::new();
    drop(area.open());
    let database = area.database();
    fs::remove_file(&database).unwrap();
    let db = Connection::open(&database).unwrap();
    db.execute_batch("CREATE TABLE unrelated(value TEXT); INSERT INTO unrelated VALUES('keep');")
        .unwrap();
    db.close().unwrap();
    fs::set_permissions(&database, Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), true, deadline()),
        Err(Error::UnsupportedSchema)
    ));
    assert_eq!(
        area.inspect()
            .query_row("SELECT value FROM unrelated", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "keep"
    );
}

#[test]
fn future_schema_refuses_without_downgrade() {
    let area = Area::new();
    drop(area.open());
    area.edit_closed("PRAGMA user_version=2;");
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()),
        Err(Error::UnsupportedSchema)
    ));
    assert_eq!(
        area.inspect()
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        2
    );
}

#[test]
fn changed_migration_checksum_refuses_even_at_supported_version() {
    let area = Area::new();
    drop(area.open());
    area.edit_closed(&format!(
        "UPDATE migration_history SET checksum='{CRITERIA}';"
    ));
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()),
        Err(Error::UnsupportedSchema)
    ));
}

/// A ledger that cannot grow refuses admission by name and leaves no partial task, event or
/// operation behind: the clamp is SQLite's own page bound, not a filesystem arrangement.
#[test]
fn full_storage_refuses_admission_by_name_with_no_partial_rows() {
    let area = Area::new();
    let mut store = area.open();
    let pages: i64 = store
        .connection
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .unwrap();
    let clamped: i64 = store
        .connection
        .query_row(&format!("PRAGMA max_page_count={pages}"), [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(clamped, pages, "the clamp must hold before the write");
    // A small request fits in free space inside existing pages and is admitted under this clamp
    // (measured); 512 KiB needs new overflow pages the clamp forbids, under the 1 MiB request bound.
    let owner = principal();
    let large = vec![b'x'; 512 * 1024];
    let mut request = submission(&owner);
    request.request_bytes = &large;
    let refused = store.submit(request, deadline());
    assert!(matches!(refused, Err(Error::Full)), "{refused:?}");
    // SQLite already rolled that transaction back; the store is not poisoned by it, and a small
    // request that fits in existing pages is still admitted under the same clamp.
    assert_eq!(admit(&mut store).sequence, 1);
    drop(store);
    drop(area.reopen());
    for table in ["tasks", "events", "operations"] {
        assert_eq!(
            count(&area, table),
            1,
            "{table} kept a partial row of the refused request"
        );
    }
    let spec: Vec<u8> = area
        .inspect()
        .query_row("SELECT spec FROM tasks", [], |row| row.get(0))
        .unwrap();
    assert_eq!(spec, b"canonical request");
}

/// A COMMIT that SQLite itself fails (its commit hook turns it into a rollback) is reported as
/// uncertain, the same answer the poisoned store gives every later write, never as success.
#[test]
fn failed_commit_is_uncertain_and_poisons_later_writes() {
    let area = Area::new();
    let mut store = area.open();
    store.connection.commit_hook(Some(|| true)).unwrap();
    assert!(matches!(
        store.submit(submission(&principal()), deadline()),
        Err(Error::UncertainCommit)
    ));
    store.connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert!(matches!(
        store.submit(submission(&principal()), deadline()),
        Err(Error::UncertainCommit)
    ));
    drop(store);
    drop(area.reopen());
    assert_eq!(count(&area, "tasks"), 0, "the hook rolled the commit back");
}

/// B03: a task's view reads that task alone. Another task's attempt is not in it, and the read
/// point is the ledger's event high-water, the same point the recovery inventory reports.
#[test]
fn task_view_reads_one_task_and_the_ledger_read_point() {
    let area = Area::new();
    let mut store = area.open();
    running(&mut store);
    let owner = principal();
    let mut other = submission(&owner);
    other.key = uuid("00000000-0000-4000-8000-0000000000e1");
    other.task = uuid(OTHER);
    other.event = uuid("00000000-0000-4000-8000-0000000000e2");
    other.request_bytes = b"another request";
    store.submit(other, deadline()).unwrap();
    store
        .begin_attempt(
            uuid(OTHER),
            revision(1),
            uuid("00000000-0000-4000-8000-0000000000e3"),
            uuid("00000000-0000-4000-8000-0000000000e4"),
            deadline(),
        )
        .unwrap();
    let view = store.task_view(&owner, uuid(TASK), deadline()).unwrap();
    assert_eq!(view.head, head_of(&store));
    assert_eq!(
        view.attempts
            .iter()
            .map(|attempt| (attempt.id.as_str(), attempt.task.as_str()))
            .collect::<Vec<_>>(),
        [(ATTEMPT, TASK)]
    );
    assert_eq!(view.pending_deliveries, 0);
    let inventory = store
        .recovery_inventory(
            uuid(EPOCH),
            crate::app::coordinator::START_LIMITS,
            deadline(),
        )
        .unwrap();
    assert_eq!(view.event_high_water, inventory.event_high_water);
    assert_eq!(
        view.event_high_water, 4,
        "two admissions and two attempt starts"
    );
}

/// B03: the view counts this task's delivery obligations that are still owed, and no others:
/// delivered rows leave the count; the count is read from the ledger, never assumed.
#[test]
fn task_view_counts_only_undelivered_obligations_of_its_task() {
    let area = Area::new();
    let mut store = area.open();
    accepted(&mut store);
    let owed: i64 = area
        .inspect()
        .query_row(
            "SELECT count(*) FROM outbox o JOIN events e ON e.id=o.event_id WHERE e.task_id=?",
            [TASK],
            |row| row.get(0),
        )
        .unwrap();
    // Measured: the fixture's acceptance owes this task one delivery.
    assert_eq!(owed, 1);
    let view = store
        .task_view(&principal(), uuid(TASK), deadline())
        .unwrap();
    assert_eq!(view.pending_deliveries, usize::try_from(owed).unwrap());
    drop(store);
    // One delivered: the view's count falls by exactly one.
    area.edit_closed(
        "UPDATE outbox SET delivered=1 WHERE rowid=(SELECT min(o.rowid) FROM outbox o JOIN events e ON e.id=o.event_id);",
    );
    let mut store = area.reopen();
    let view = store
        .task_view(&principal(), uuid(TASK), deadline())
        .unwrap();
    assert_eq!(view.pending_deliveries, usize::try_from(owed - 1).unwrap());
}

/// B03: a task holding more attempts than the ledger's own attempt bound is not a task the view
/// can describe; it is refused by name with both numbers, never truncated to the bound.
#[test]
fn task_view_refuses_more_attempts_than_the_bound_with_both_numbers() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    drop(store);
    area.edit_closed(&format!(
        "INSERT INTO attempts VALUES \
         ('00000000-0000-4000-8000-0000000000f1','{TASK}','1','settled','none','settled',0),\
         ('00000000-0000-4000-8000-0000000000f2','{TASK}','2','settled','none','settled',0),\
         ('00000000-0000-4000-8000-0000000000f3','{TASK}','3','settled','none','settled',0),\
         ('00000000-0000-4000-8000-0000000000f4','{TASK}','4','settled','none','settled',0);"
    ));
    let mut store = area.reopen();
    let refused = store.task_view(&principal(), uuid(TASK), deadline());
    assert!(
        matches!(
            refused,
            Err(Error::TaskViewBound {
                attempts: 4,
                limit: 3
            })
        ),
        "{refused:?}"
    );
}

/// B03 (the unification ruled 2026-09-25): the view refuses a poisoned store, as the recovery
/// inventory does; `get_by_key` keeps its pinned exception for the admission-recovery readback.
#[test]
fn task_view_refuses_a_poisoned_store_and_hides_another_principals_task() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    let stranger = Principal::new(1001, "operator").unwrap();
    assert!(matches!(
        store.task_view(&stranger, uuid(TASK), deadline()),
        Err(Error::NotFound)
    ));
    store.fault = Some(CutPoint::AfterCommit);
    let owner = principal();
    let mut other = submission(&owner);
    other.key = uuid("00000000-0000-4000-8000-0000000000e1");
    other.task = uuid(OTHER);
    other.event = uuid("00000000-0000-4000-8000-0000000000e2");
    other.request_bytes = b"another request";
    assert!(matches!(
        store.submit(other, deadline()),
        Err(Error::UncertainCommit)
    ));
    assert!(matches!(
        store.task_view(&owner, uuid(TASK), deadline()),
        Err(Error::UncertainCommit)
    ));
    assert_eq!(
        store.get_by_key(&owner, uuid(KEY), deadline()).unwrap().id,
        TASK,
        "the pinned admission-recovery exception"
    );
}

/// One closure case: what it is, the record written (if any), and whether it closes.
type ClosureCase = (&'static str, Option<(RecordKind, String)>, bool);

/// The closure cases, hand-shaped after the engine's writer (`app::startup`); the real-writer
/// check is `t07_startup`, whose pins close attempts only through records the pass itself wrote.
fn closure_cases() -> [ClosureCase; 13] {
    let decided = |decision: &str, cleanup: &str, workspace: &str, custody: &str| {
        format!(
            r#"{{"kind":"hee3-reconciliation-decided/1","attempt":"{ATTEMPT}","task":"{TASK}","decision":{{"decision":"{decision}"}},"handed":{{"cleanup":{{"cleanup_readback":"{cleanup}"}},"workspace":{{"workspace":"{workspace}"}},"process":{{"custody":"{custody}"}}}}}}"#
        )
    };
    let clean =
        |cleanup: &str, custody: &str| decided("cleanup_candidate", cleanup, "writable", custody);
    let standing = |decision: &str, workspace: &str, custody: &str| {
        decided(decision, "partial", workspace, custody)
    };
    [
        ("no record", None, false),
        (
            "decided, complete, absent",
            Some((RecordKind::Decided, clean("complete", "absent"))),
            true,
        ),
        (
            "decided, complete, pid reused",
            Some((RecordKind::Decided, clean("complete", "pid_reused"))),
            true,
        ),
        (
            "decided, complete, unobserved",
            Some((RecordKind::Decided, clean("complete", "unobserved"))),
            true,
        ),
        (
            "decided, partial",
            Some((RecordKind::Decided, clean("partial", "absent"))),
            false,
        ),
        (
            "decided, complete, live",
            Some((RecordKind::Decided, clean("complete", "live_same_identity"))),
            false,
        ),
        (
            "decided, complete, unreadable",
            Some((RecordKind::Decided, clean("complete", "unreadable"))),
            false,
        ),
        (
            "acceptance stands, workspace retained, absent",
            Some((
                RecordKind::Decided,
                standing("acceptance_stands", "writable", "absent"),
            )),
            true,
        ),
        (
            "cancellation stands, workspace retained, unobserved",
            Some((
                RecordKind::Decided,
                standing("cancellation_stands", "writable", "unobserved"),
            )),
            true,
        ),
        (
            "not a standing decision, workspace retained",
            Some((
                RecordKind::Decided,
                standing("cleanup_candidate", "writable", "absent"),
            )),
            false,
        ),
        (
            "acceptance stands, workspace released, obligations remain",
            Some((
                RecordKind::Decided,
                standing("acceptance_stands", "released", "absent"),
            )),
            false,
        ),
        (
            "acceptance stands, workspace not read",
            Some((
                RecordKind::Decided,
                standing("acceptance_stands", "not_read", "pid_reused"),
            )),
            false,
        ),
        (
            "acceptance stands, workspace retained, live",
            Some((
                RecordKind::Decided,
                standing("acceptance_stands", "writable", "live_same_identity"),
            )),
            false,
        ),
    ]
}

/// B03b: whether a terminal task's settled attempt is closed is decided in ONE place, from the
/// engine's own readback in a startup record, never from a worker's cleanup claim. Each record
/// kind and each liveness classification is pinned: only a positively non-live holder with a
/// complete cleanup readback closes an attempt, and unknown liveness never does. The one other
/// closure, "workspace retained", is pinned conjunct by conjunct: a STANDING decision, a workspace
/// read back still writable, and the same non-live custody -- each case below drops exactly one.
#[test]
fn an_attempt_is_closed_only_by_the_engines_own_complete_readback() {
    let readback = format!(
        r#"{{"kind":"hee3-reconciliation-readback/1","attempt":"{ATTEMPT}","effect":"cleanup","readback":{{"cleanup_readback":"complete"}}}}"#
    );
    let cases = closure_cases();
    for (case, record, closed) in cases {
        let area = Area::new();
        let mut store = area.open();
        accepted(&mut store);
        if let Some((kind, body)) = &record {
            store
                .record_reconciliation(
                    &ReconciliationRecord {
                        attempt: uuid(ATTEMPT),
                        kind: *kind,
                        body: body.as_bytes(),
                        settle_cleanup: false,
                    },
                    deadline(),
                )
                .unwrap();
        }
        assert_eq!(
            store.attempt_closed(uuid(ATTEMPT), deadline()).unwrap(),
            closed,
            "{case}"
        );
    }
    // The readback kind closes on its own cleanup effect.
    let area = Area::new();
    let mut store = area.open();
    accepted(&mut store);
    store
        .record_reconciliation(
            &ReconciliationRecord {
                attempt: uuid(ATTEMPT),
                kind: RecordKind::Readback,
                body: readback.as_bytes(),
                settle_cleanup: false,
            },
            deadline(),
        )
        .unwrap();
    assert!(
        store.attempt_closed(uuid(ATTEMPT), deadline()).unwrap(),
        "readback, cleanup complete"
    );
}

/// A transaction SQLite already rolled back (autocommit restored) needs no second rollback; an
/// open one is rolled back for real.
#[test]
fn roll_back_accepts_a_rollback_sqlite_already_made() {
    let mut db = Connection::open_in_memory().unwrap();
    db.execute_batch("CREATE TABLE t(x);").unwrap();
    let tx = db.transaction().unwrap();
    tx.execute("INSERT INTO t VALUES(1)", []).unwrap();
    tx.execute_batch("ROLLBACK;").unwrap();
    assert!(tx.is_autocommit());
    assert!(roll_back(tx).is_ok());
    let tx = db.transaction().unwrap();
    tx.execute("INSERT INTO t VALUES(2)", []).unwrap();
    assert!(!tx.is_autocommit());
    assert!(roll_back(tx).is_ok());
    let rows: i64 = db
        .query_row("SELECT count(*) FROM t", [], |row| row.get(0))
        .unwrap();
    assert_eq!(rows, 0);
}

/// Constraint and no-space failures produced by SQLite and the OS are classified by name.
#[test]
fn constraint_and_no_space_failures_are_named() {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch("CREATE TABLE t(x INTEGER NOT NULL CHECK(x > 0));")
        .unwrap();
    let refused = db.execute("INSERT INTO t VALUES(0)", []).unwrap_err();
    assert!(matches!(Error::from(refused), Error::Constraint));
    let unique = Connection::open_in_memory().unwrap();
    unique
        .execute_batch("CREATE TABLE u(x PRIMARY KEY); INSERT INTO u VALUES(1);")
        .unwrap();
    let duplicate = unique.execute("INSERT INTO u VALUES(1)", []).unwrap_err();
    assert!(matches!(Error::from(duplicate), Error::Constraint));
    assert!(matches!(
        Error::from(std::io::Error::from_raw_os_error(
            rustix::io::Errno::NOSPC.raw_os_error()
        )),
        Error::Full
    ));
    assert!(matches!(Error::from(rustix::io::Errno::NOSPC), Error::Full));
    let other = db.execute("INSERT INTO missing VALUES(1)", []).unwrap_err();
    assert!(matches!(Error::from(other), Error::Sqlite(_)));
    assert!(matches!(
        Error::from(rustix::io::Errno::ACCESS),
        Error::Os(_)
    ));
}

/// A publication rewrites only the anchor block (d6cd92c did, with no DDL change); the recorded
/// migration identity must not move, while one more byte of migration body must move it.
#[test]
fn anchor_block_rewrite_keeps_migration_identity()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let id = |sql: &str| schema::identity(sql).map_err(|error| format!("{error:?}"));
    let sql = include_str!("../migrations/001.sql");
    let (block, body) = sql
        .split_once("-- HEE3-ANCHORS-END\n")
        .ok_or("migration 1 has no anchor end marker")?;
    let rewritten = format!(
        "{}-- a later publication rewrote this block\n-- HEE3-ANCHORS-END\n{body}",
        block.replace("Readiness binding", "Readiness rebinding")
    );
    assert_ne!(
        rewritten, sql,
        "the fixture must differ inside the anchor block"
    );
    assert_eq!(id(sql)?, MIGRATION_1_BODY);
    assert_eq!(id(&rewritten)?, MIGRATION_1_BODY);
    assert_ne!(
        id(&format!("{sql}-- one more byte of migration\n"))?,
        MIGRATION_1_BODY
    );
    Ok(())
}

/// Without exactly one end marker there is no body to name; a second marker could hide DDL.
#[test]
fn migration_identity_refuses_missing_or_repeated_anchor_end() {
    let sql = include_str!("../migrations/001.sql");
    assert!(matches!(
        schema::identity(&sql.replace("-- HEE3-ANCHORS-END\n", "")),
        Err(Error::UnsupportedSchema)
    ));
    assert!(matches!(
        schema::identity(&format!("{sql}-- HEE3-ANCHORS-END\n")),
        Err(Error::UnsupportedSchema)
    ));
}

/// The whole file executes but only the body is hashed, so SQL inside the excluded block would
/// run unnamed. A statement in the block, or a marker trailing a statement, is refused.
#[test]
fn executable_sql_in_the_excluded_anchor_block_is_refused() {
    let sql = include_str!("../migrations/001.sql");
    assert!(schema::identity(sql).is_ok());
    let planted = sql.replacen(
        "-- HEE3-ANCHORS-BEGIN\n",
        "-- HEE3-ANCHORS-BEGIN\nCREATE TABLE unnamed(x);\n",
        1,
    );
    assert!(matches!(
        schema::identity(&planted),
        Err(Error::UnsupportedSchema)
    ));
    let trailing = sql.replacen(
        "-- HEE3-ANCHORS-END\n",
        "CREATE TABLE unnamed(x); -- HEE3-ANCHORS-END\n",
        1,
    );
    assert!(matches!(
        schema::identity(&trailing),
        Err(Error::UnsupportedSchema)
    ));
}

/// A ledger that recorded the whole-file digest (the rule before the freeze) is refused, not
/// silently re-blessed: no live ledger exists (RC02), so there is nothing to migrate.
#[test]
fn whole_file_checksum_recorded_before_the_freeze_is_refused() {
    let area = Area::new();
    drop(area.open());
    area.edit_closed(&format!(
        "UPDATE migration_history SET checksum='{}';",
        digest(include_bytes!("../migrations/001.sql"))
    ));
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()),
        Err(Error::UnsupportedSchema)
    ));
}

#[test]
fn missing_migration_history_cannot_be_reinferred_from_schema() {
    let area = Area::new();
    drop(area.open());
    area.edit_closed("DELETE FROM migration_history;");
    assert!(Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()).is_err());
    assert_eq!(count(&area, "migration_history"), 0);
}

#[test]
fn unexpected_trigger_refuses_exact_schema_compatibility() {
    let area = Area::new();
    drop(area.open());
    area.edit_closed("CREATE TRIGGER surprise AFTER INSERT ON events BEGIN SELECT 1; END;");
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()),
        Err(Error::UnsupportedSchema)
    ));
}

#[test]
fn foreign_key_corruption_is_detected_beyond_integrity_check() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    drop(store);
    area.edit_closed(&format!(
        "PRAGMA foreign_keys=OFF; UPDATE operations SET resource_id='{OTHER}';"
    ));
    assert_eq!(
        area.inspect()
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()),
        Err(Error::Corrupt)
    ));
}

#[test]
fn accepted_history_without_delivery_obligation_refuses_reopen() {
    let area = Area::new();
    let mut store = area.open();
    accepted(&mut store);
    drop(store);
    area.edit_closed("DELETE FROM outbox;");
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()),
        Err(Error::Corrupt)
    ));
    assert_eq!(count(&area, "acceptances"), 1);
}

#[test]
fn migration_failure_rolls_back_schema_history_and_version_together() {
    let area = Area::new();
    injected(
        Store::open_inner(
            &area.path,
            uuid(GEN),
            uuid(EPOCH),
            true,
            deadline(),
            Some(CutPoint::MigrationWrite),
        ),
        CutPoint::MigrationWrite,
    );
    let db = area.inspect();
    assert_eq!(
        db.query_row("SELECT count(*) FROM sqlite_schema", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        0
    );
    drop(db);
    assert!(Store::open(&area.path, uuid(GEN), uuid(EPOCH), false, deadline()).is_err());
}

#[test]
fn exact_request_replay_returns_original_result_without_new_event() {
    let area = Area::new();
    let mut store = area.open();
    let original = admit(&mut store);
    let owner = principal();
    let mut replay = submission(&owner);
    replay.task = uuid(OTHER);
    replay.event = uuid(CANCELLED);
    assert_eq!(store.submit(replay, deadline()).unwrap(), original);
    for table in ["tasks", "operations", "events"] {
        assert_eq!(count(&area, table), 1);
    }
}

#[test]
fn same_request_key_with_different_exact_bytes_conflicts_atomically() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    let owner = principal();
    let mut changed = submission(&owner);
    changed.request_bytes = b"canonical request\n";
    assert!(matches!(
        store.submit(changed, deadline()),
        Err(Error::Conflict)
    ));
    assert_eq!(count(&area, "tasks"), 1);
    assert_eq!(count(&area, "events"), 1);
    assert_eq!(
        area.inspect()
            .query_row("SELECT spec FROM tasks", [], |row| row.get::<_, Vec<u8>>(0))
            .unwrap(),
        b"canonical request"
    );
}

#[test]
fn request_keys_and_task_visibility_are_scoped_by_uid_and_role() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    for stranger in [
        Principal::new(1001, "operator").unwrap(),
        Principal::new(1000, "observer").unwrap(),
    ] {
        assert!(matches!(
            store.get_by_key(&stranger, uuid(KEY), deadline()),
            Err(Error::NotFound)
        ));
        assert!(matches!(
            store.get(&stranger, uuid(TASK), deadline()),
            Err(Error::NotFound)
        ));
    }
    let stranger = Principal::new(1001, "operator").unwrap();
    let mut second = submission(&stranger);
    second.task = uuid(OTHER);
    second.event = uuid(CANCELLED);
    store.submit(second, deadline()).unwrap();
    assert_eq!(count(&area, "operations"), 2);
    assert_eq!(
        store
            .get_by_key(&stranger, uuid(KEY), deadline())
            .unwrap()
            .id,
        OTHER
    );
}

#[test]
fn lost_admission_reply_is_uncertain_and_recovers_by_stable_key_once() {
    let area = Area::new();
    let mut store = area.open();
    store.fault = Some(CutPoint::AfterCommit);
    assert!(matches!(
        store.submit(submission(&principal()), deadline()),
        Err(Error::UncertainCommit)
    ));
    assert_eq!(
        store
            .get_by_key(&principal(), uuid(KEY), deadline())
            .unwrap()
            .id,
        TASK
    );
    assert!(matches!(
        store.cancel(uuid(TASK), revision(1), uuid(CANCELLED), deadline()),
        Err(Error::UncertainCommit)
    ));
    drop(store);
    let mut reopened = area.reopen();
    let replay = admit(&mut reopened);
    assert_eq!(replay.sequence, 1);
    assert_eq!(count(&area, "tasks"), 1);
    assert_eq!(count(&area, "events"), 1);
}

#[test]
fn task_write_failure_rolls_back_admission_reservations_and_event() {
    let area = Area::new();
    let mut store = area.open();
    store.fault = Some(CutPoint::TaskWrite);
    injected(
        store.submit(submission(&principal()), deadline()),
        CutPoint::TaskWrite,
    );
    no_admission(&area);
    store.fault = None;
    admit(&mut store);
    assert_eq!(head_of(&store).reserved_verify_ms, 200);
}

#[test]
fn before_commit_failure_leaves_no_partial_operation_or_admission() {
    let area = Area::new();
    let mut store = area.open();
    store.fault = Some(CutPoint::BeforeCommit);
    injected(
        store.submit(submission(&principal()), deadline()),
        CutPoint::BeforeCommit,
    );
    drop(store);
    let reopened = area.reopen();
    no_admission(&area);
    assert!(matches!(
        reopened.get_by_key(&principal(), uuid(KEY), deadline()),
        Err(Error::NotFound)
    ));
}

#[test]
fn conflicting_event_identity_rolls_back_new_task_and_reservations() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    let owner = principal();
    let mut duplicate = submission(&owner);
    duplicate.key = uuid(OTHER);
    duplicate.task = uuid(CANCELLED);
    assert!(store.submit(duplicate, deadline()).is_err());
    assert_eq!(count(&area, "tasks"), 1);
    assert_eq!(count(&area, "operations"), 1);
    assert_eq!(count(&area, "events"), 1);
}

/// The frozen migration's own CHECK on `tasks.limit_ms` is a second door on RC01's task limit that
/// cannot import it; pin it to the one definition at the boundary, on a real ledger row.
#[test]
fn frozen_limit_check_agrees_with_the_rc01_task_limit() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    drop(store);
    let limit = u64::try_from(crate::contracts::rc01::TASK_LIMIT.as_millis()).unwrap();
    let db = Connection::open_with_flags(
        area.database(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .unwrap();
    let set = |value: u64| {
        db.execute(
            "UPDATE tasks SET limit_ms=? WHERE id=?",
            params![i64::try_from(value).unwrap(), TASK],
        )
    };
    assert_eq!(set(limit).unwrap(), 1);
    let over = set(limit + 1).unwrap_err();
    assert!(matches!(Error::from(over), Error::Constraint));
    let stored: i64 = db
        .query_row("SELECT limit_ms FROM tasks", [], |row| row.get(0))
        .unwrap();
    assert_eq!(u64::try_from(stored).unwrap(), limit);
}

/// RC01's task bound is inclusive at the ledger's own door: exactly 1,200,000 ms (the published
/// figure, written here as a literal) is admitted; one more is `Bound` (pinned below).
#[test]
fn admission_admits_exactly_the_published_task_limit() {
    let area = Area::new();
    let mut store = area.open();
    let owner = principal();
    let mut request = submission(&owner);
    request.allocation = Allocation {
        limit_ms: 1_200_000,
        work_ms: 900_000,
        verify_ms: 300_000,
    };
    assert_eq!(store.submit(request, deadline()).unwrap().sequence, 1);
    assert_eq!(count(&area, "tasks"), 1);
}

/// An admission may reserve no work time (only verification); such a task is refused an attempt
/// for lack of work budget, while its verification reservation alone does not open one.
#[test]
fn zero_work_reservation_refuses_an_attempt_on_budget() {
    let area = Area::new();
    let mut store = area.open();
    let owner = principal();
    let mut request = submission(&owner);
    request.allocation = Allocation {
        limit_ms: 1000,
        work_ms: 0,
        verify_ms: 200,
    };
    store.submit(request, deadline()).unwrap();
    assert!(matches!(
        store.begin_attempt(
            uuid(TASK),
            revision(1),
            uuid(ATTEMPT),
            uuid(STARTED),
            deadline()
        ),
        Err(Error::Budget)
    ));
    assert_eq!(count(&area, "attempts"), 0);
}

#[test]
fn admission_allocation_bounds_preserve_verification_reservation() {
    let area = Area::new();
    let mut store = area.open();
    let owner = principal();
    for bad in [
        Allocation {
            limit_ms: 0,
            work_ms: 0,
            verify_ms: 1,
        },
        Allocation {
            limit_ms: 1000,
            work_ms: 1000,
            verify_ms: 0,
        },
        Allocation {
            limit_ms: 1000,
            work_ms: 900,
            verify_ms: 200,
        },
        Allocation {
            limit_ms: 1_200_001,
            work_ms: 1,
            verify_ms: 1,
        },
        Allocation {
            limit_ms: 1000,
            work_ms: u64::MAX,
            verify_ms: 1,
        },
    ] {
        let mut value = submission(&owner);
        value.allocation = bad;
        assert!(matches!(store.submit(value, deadline()), Err(Error::Bound)));
        no_admission(&area);
    }
    admit(&mut store);
    assert_eq!(head_of(&store).reserved_verify_ms, 200);
}

#[test]
fn request_blob_bounds_count_bytes_and_do_not_truncate() {
    let area = Area::new();
    let mut store = area.open();
    let owner = principal();
    let oversized = vec![b'x'; 1_048_577];
    for bytes in [b"".as_slice(), oversized.as_slice()] {
        let mut value = submission(&owner);
        value.request_bytes = bytes;
        assert!(matches!(store.submit(value, deadline()), Err(Error::Bound)));
    }
    no_admission(&area);
    let exact = vec![b'x'; 1_048_576];
    let mut value = submission(&owner);
    value.request_bytes = &exact;
    store.submit(value, deadline()).unwrap();
    assert_eq!(
        area.inspect()
            .query_row("SELECT length(spec) FROM tasks", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1_048_576
    );
}

#[test]
fn attempt_start_commits_distinct_task_and_attempt_generations_with_event() {
    let area = Area::new();
    let mut store = area.open();
    running(&mut store);
    assert_eq!(head_of(&store).generation, "2");
    assert_eq!(count(&area, "attempts"), 1);
    assert_eq!(count(&area, "events"), 2);
    let row: (String, String, String) = area
        .inspect()
        .query_row(
            "SELECT generation,effect,cleanup FROM attempts",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(row, ("1".into(), "pending".into(), "pending".into()));
}

#[test]
fn attempt_write_failure_rolls_back_attempt_task_revision_and_event() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    store.fault = Some(CutPoint::AttemptWrite);
    injected(
        store.begin_attempt(
            uuid(TASK),
            revision(1),
            uuid(ATTEMPT),
            uuid(STARTED),
            deadline(),
        ),
        CutPoint::AttemptWrite,
    );
    assert_eq!(head_of(&store).state, "admitted");
    assert_eq!(head_of(&store).generation, "1");
    assert_eq!(count(&area, "attempts"), 0);
    assert_eq!(count(&area, "events"), 1);
}

#[test]
fn stale_task_revision_cannot_cancel_or_start_another_attempt() {
    let area = Area::new();
    let mut store = area.open();
    running(&mut store);
    assert!(matches!(
        store.cancel(uuid(TASK), revision(1), uuid(CANCELLED), deadline()),
        Err(Error::Conflict)
    ));
    assert!(matches!(
        store.begin_attempt(
            uuid(TASK),
            revision(1),
            uuid(OTHER),
            uuid(CANCELLED),
            deadline()
        ),
        Err(Error::Conflict)
    ));
    assert_eq!(count(&area, "events"), 2);
    assert!(!head_of(&store).cancellation);
}

#[test]
fn settlement_requires_exact_attempt_generation_and_identity() {
    let area = Area::new();
    let mut store = area.open();
    running(&mut store);
    for wrong in [
        expected(2, 2),
        Expected {
            attempt: uuid(OTHER),
            ..expected(2, 1)
        },
    ] {
        assert!(matches!(
            store.settle_attempt(
                &wrong,
                settled(Effect::None, Some(1), true, true),
                uuid(SETTLED),
                deadline()
            ),
            Err(Error::Outstanding)
        ));
    }
    assert_eq!(head_of(&store).state, "running");
    assert_eq!(count(&area, "events"), 2);
}

#[test]
fn unresolved_attempt_prevents_reuse_without_releasing_reservations() {
    let area = Area::new();
    let mut store = area.open();
    let active = running(&mut store);
    store
        .settle_attempt(
            &active,
            settled(Effect::Unknown, None, false, false),
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    assert!(
        store
            .begin_attempt(
                uuid(TASK),
                revision(3),
                uuid(OTHER),
                uuid(CANCELLED),
                deadline()
            )
            .is_err()
    );
    assert_eq!(count(&area, "attempts"), 1);
    assert_eq!(head_of(&store).reserved_work_ms, 800);
}

#[test]
fn three_attempt_limit_counts_settled_attempts_without_reusing_identity() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    let mut task_revision = 1;
    for index in 1..=3_u32 {
        let attempt = format!("00000000-0000-4000-8000-{:012x}", 100 + index);
        let start = format!("00000000-0000-4000-8000-{:012x}", 200 + index);
        let end = format!("00000000-0000-4000-8000-{:012x}", 300 + index);
        let started = store
            .begin_attempt(
                uuid(TASK),
                revision(task_revision),
                uuid(&attempt),
                uuid(&start),
                deadline(),
            )
            .unwrap();
        assert_eq!(started.generation, index.to_string());
        task_revision += 1;
        let active = Expected {
            task: uuid(TASK),
            task_generation: revision(task_revision),
            attempt: uuid(&attempt),
            attempt_generation: revision(u64::from(index)),
        };
        store
            .settle_attempt(
                &active,
                settled(Effect::None, Some(1), true, false),
                uuid(&end),
                deadline(),
            )
            .unwrap();
        task_revision += 1;
    }
    assert!(matches!(
        store.begin_attempt(
            uuid(TASK),
            revision(task_revision),
            uuid(OTHER),
            uuid(CANCELLED),
            deadline()
        ),
        Err(Error::Bound)
    ));
    assert_eq!(count(&area, "attempts"), 3);
    assert_eq!(head_of(&store).spent_ms, 3);
}

#[test]
fn known_settlement_consumes_work_once_and_retains_verification_budget() {
    let area = Area::new();
    let mut store = area.open();
    verifying(&mut store);
    let head = head_of(&store);
    assert_eq!(
        (
            head.spent_ms,
            head.reserved_work_ms,
            head.reserved_verify_ms
        ),
        (30, 770, 200)
    );
    assert_eq!(head.state, "verifying");
    assert!(matches!(
        store.settle_attempt(
            &expected(3, 1),
            settled(Effect::None, Some(30), true, true),
            uuid(OTHER),
            deadline()
        ),
        Err(Error::Outstanding)
    ));
    assert_eq!(head_of(&store), head);
}

#[test]
fn over_allocation_usage_refuses_without_partial_settlement() {
    let area = Area::new();
    let mut store = area.open();
    let active = running(&mut store);
    assert!(matches!(
        store.settle_attempt(
            &active,
            settled(Effect::None, Some(801), true, true),
            uuid(SETTLED),
            deadline()
        ),
        Err(Error::Budget)
    ));
    assert_eq!(head_of(&store).state, "running");
    assert_eq!(count(&area, "events"), 2);
    assert_eq!(head_of(&store).reserved_work_ms, 800);
}

#[test]
fn uncertain_effect_retains_measured_usage_and_full_conservative_liability() {
    let area = Area::new();
    let mut store = area.open();
    let active = running(&mut store);
    store
        .settle_attempt(
            &active,
            settled(Effect::Unknown, Some(30), true, false),
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    let head = head_of(&store);
    assert_eq!(
        (
            head.spent_ms,
            head.reserved_work_ms,
            head.reserved_verify_ms
        ),
        (0, 800, 200)
    );
    assert_eq!(head.state, "effect_unknown");
    assert_eq!(
        area.inspect()
            .query_row("SELECT used_ms FROM attempts", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        30
    );
}

#[test]
fn unknown_settlement_cannot_erase_or_reduce_known_usage() {
    let area = Area::new();
    let mut store = area.open();
    let active = running(&mut store);
    store
        .settle_attempt(
            &active,
            settled(Effect::Unknown, Some(30), false, false),
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    for later in [None, Some(29)] {
        assert!(matches!(
            store.settle_attempt(
                &expected(3, 1),
                settled(Effect::Unknown, later, false, false),
                uuid(OTHER),
                deadline()
            ),
            Err(Error::Conflict)
        ));
    }
    assert_eq!(
        area.inspect()
            .query_row("SELECT used_ms FROM attempts", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        30
    );
    assert_eq!(count(&area, "events"), 3);
}

#[test]
fn settled_cleanup_cannot_regress_while_effect_is_unknown() {
    let area = Area::new();
    let mut store = area.open();
    let active = running(&mut store);
    store
        .settle_attempt(
            &active,
            settled(Effect::Unknown, None, true, false),
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    assert!(matches!(
        store.settle_attempt(
            &expected(3, 1),
            settled(Effect::Unknown, None, false, false),
            uuid(OTHER),
            deadline()
        ),
        Err(Error::Conflict)
    ));
    assert_eq!(
        area.inspect()
            .query_row("SELECT cleanup FROM attempts", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "settled"
    );
}

#[test]
fn known_committed_effect_cannot_be_reclassified_unknown() {
    let area = Area::new();
    let mut store = area.open();
    let active = running(&mut store);
    store
        .settle_attempt(
            &active,
            settled(Effect::Committed, None, false, false),
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    assert!(matches!(
        store.settle_attempt(
            &expected(3, 1),
            settled(Effect::Unknown, None, false, false),
            uuid(OTHER),
            deadline()
        ),
        Err(Error::Conflict)
    ));
    assert_eq!(
        area.inspect()
            .query_row("SELECT effect FROM attempts", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "committed"
    );
}

#[test]
fn resolving_uncertainty_charges_only_the_cumulative_measurement_once() {
    let area = Area::new();
    let mut store = area.open();
    let active = running(&mut store);
    store
        .settle_attempt(
            &active,
            settled(Effect::Unknown, Some(20), true, false),
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    store
        .settle_attempt(
            &expected(3, 1),
            settled(Effect::None, Some(30), true, true),
            uuid(OTHER),
            deadline(),
        )
        .unwrap();
    assert_eq!(
        (head_of(&store).spent_ms, head_of(&store).reserved_work_ms),
        (30, 770)
    );
    assert_eq!(head_of(&store).state, "verifying");
}

#[test]
fn cancellation_before_dispatch_blocks_attempt_and_keeps_allocations() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    assert_eq!(
        store
            .cancel(uuid(TASK), revision(1), uuid(CANCELLED), deadline())
            .unwrap(),
        "2"
    );
    assert!(matches!(
        store.begin_attempt(
            uuid(TASK),
            revision(2),
            uuid(ATTEMPT),
            uuid(STARTED),
            deadline()
        ),
        Err(Error::Cancelled)
    ));
    assert_eq!(count(&area, "attempts"), 0);
    assert_eq!(head_of(&store).reserved_verify_ms, 200);
}

#[test]
fn cancellation_does_not_settle_unknown_effect_or_release_usage() {
    let area = Area::new();
    let mut store = area.open();
    let active = running(&mut store);
    store
        .settle_attempt(
            &active,
            settled(Effect::Unknown, None, false, false),
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    store
        .cancel(uuid(TASK), revision(3), uuid(CANCELLED), deadline())
        .unwrap();
    assert!(head_of(&store).cancellation);
    assert_eq!(head_of(&store).reserved_work_ms, 800);
    assert_eq!(
        area.inspect()
            .query_row("SELECT state FROM attempts", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "unknown"
    );
}

#[test]
fn cancellation_first_prevents_acceptance_even_for_prepared_durable_proof() {
    let area = Area::new();
    let mut store = area.open();
    let active = verifying(&mut store);
    let published = proof(&store, &active);
    store
        .cancel(uuid(TASK), revision(3), uuid(CANCELLED), deadline())
        .unwrap();
    assert!(store.accept(&published, 20, deadline()).is_err());
    no_acceptance(&area);
    assert!(store.read_object(published.object(), deadline()).is_ok());
    match store.prepare_acceptance(
        &expected(4, 1),
        uuid(OTHER),
        &published.data.objects,
        deadline(),
    ) {
        Ok(fresh) => assert!(matches!(
            store.accept(&fresh, 20, deadline()),
            Err(Error::Cancelled)
        )),
        Err(Error::Cancelled) => (),
        other => panic!("unexpected cancelled preparation: {other:?}"),
    }
    no_acceptance(&area);
}

#[test]
fn acceptance_first_remains_historical_after_cancel_request() {
    let area = Area::new();
    let mut store = area.open();
    accepted(&mut store);
    let before = head_of(&store);
    assert_eq!(
        store
            .cancel(uuid(TASK), revision(4), uuid(CANCELLED), deadline())
            .unwrap(),
        "4"
    );
    assert_eq!(head_of(&store), before);
    assert_eq!(count(&area, "events"), 4);
    assert_eq!(count(&area, "outbox"), 1);
}

#[test]
fn missing_current_proof_does_not_erase_historical_acceptance() {
    let area = Area::new();
    let mut store = area.open();
    let published = accepted(&mut store);
    let head = head_of(&store);
    fs::remove_file(area.object_path(&published.data.objects[0])).unwrap();
    assert!(
        store
            .read_object(&published.data.objects[0], deadline())
            .is_err()
    );
    assert_eq!(head_of(&store), head);
    drop(store);
    let reopened = area.reopen();
    assert_eq!(head_of(&reopened), head);
    assert_eq!(reopened.pending_delivery(10, deadline()).unwrap().len(), 1);
}

#[test]
fn publication_uses_exact_content_identity_and_verified_inode_reuse() {
    let area = Area::new();
    let store = area.open();
    let object = store.publish(b"hello", uuid(STAGE), deadline()).unwrap();
    assert_eq!(object.digest(), HELLO);
    assert_eq!(object.size(), 5);
    assert_eq!(store.read_object(&object, deadline()).unwrap(), b"hello");
    let path = area.object_path(&object);
    let inode = fs::metadata(&path).unwrap().ino();
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o400);
    assert_eq!(
        store.publish(b"hello", uuid(OTHER), deadline()).unwrap(),
        object
    );
    assert_eq!(fs::metadata(path).unwrap().ino(), inode);
    assert_eq!(count(&area, "artifacts"), 0);
}

#[test]
fn corrupt_existing_object_is_refused_and_never_overwritten() {
    let area = Area::new();
    let store = area.open();
    let object = store.publish(b"hello", uuid(STAGE), deadline()).unwrap();
    let path = area.object_path(&object);
    fs::set_permissions(&path, Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"jello").unwrap();
    assert!(matches!(
        store.publish(b"hello", uuid(OTHER), deadline()),
        Err(Error::Corrupt)
    ));
    assert_eq!(fs::read(path).unwrap(), b"jello");
}

#[test]
fn object_read_checks_declared_size_and_refuses_symlink_substitution() {
    let area = Area::new();
    let store = area.open();
    let object = store.publish(b"hello", uuid(STAGE), deadline()).unwrap();
    let wrong = Object {
        digest: object.digest().into(),
        size: 4,
    };
    assert!(matches!(
        store.read_object(&wrong, deadline()),
        Err(Error::Corrupt)
    ));
    let path = area.object_path(&object);
    let retained = area.path.join("original");
    fs::rename(&path, &retained).unwrap();
    symlink(&retained, &path).unwrap();
    assert!(store.read_object(&object, deadline()).is_err());
    assert_eq!(fs::read(retained).unwrap(), b"hello");
}

#[test]
fn object_size_bound_refuses_before_staging_and_empty_bytes_are_valid() {
    let area = Area::new();
    let store = area.open();
    assert!(matches!(
        store.publish(&vec![0; 16 * 1024 * 1024 + 1], uuid(STAGE), deadline()),
        Err(Error::Bound)
    ));
    let empty = store.publish(b"", uuid(STAGE), deadline()).unwrap();
    assert_eq!(empty.size(), 0);
    assert_eq!(
        empty.digest(),
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert!(store.read_object(&empty, deadline()).unwrap().is_empty());
}

#[test]
fn prepublication_failures_remove_only_the_owned_stage() {
    for point in [CutPoint::ObjectWrite, CutPoint::ObjectSync] {
        let area = Area::new();
        let mut store = area.open();
        let shard = store.objects.child("2c", true).unwrap();
        let neighbor = shard.path.join(".stage-unrelated");
        fs::write(&neighbor, b"retained").unwrap();
        store.fault = Some(point);
        injected(store.publish(b"hello", uuid(STAGE), deadline()), point);
        assert!(!shard.path.join(format!(".stage-{STAGE}")).exists());
        assert!(!shard.path.join(&HELLO[7..]).exists());
        assert_eq!(fs::read(neighbor).unwrap(), b"retained");
        no_acceptance(&area);
    }
}

#[test]
fn postrename_failures_retain_published_object_without_claiming_ledger_commit() {
    for point in [CutPoint::ObjectRename, CutPoint::ObjectDirectorySync] {
        let area = Area::new();
        let mut store = area.open();
        store.fault = Some(point);
        injected(store.publish(b"hello", uuid(STAGE), deadline()), point);
        let object = Object {
            digest: HELLO.into(),
            size: 5,
        };
        assert_eq!(store.read_object(&object, deadline()).unwrap(), b"hello");
        no_acceptance(&area);
        store.fault = None;
        assert_eq!(
            store.publish(b"hello", uuid(OTHER), deadline()).unwrap(),
            object
        );
    }
}

#[test]
fn acceptance_manifest_binds_preallocated_event_subject_and_criteria_before_sql() {
    let area = Area::new();
    let mut store = area.open();
    let active = verifying(&mut store);
    let published = proof(&store, &active);
    let value: serde_json::Value =
        serde_json::from_slice(&store.read_object(published.object(), deadline()).unwrap())
            .unwrap();
    assert_eq!(value["event"], ACCEPTED);
    assert_eq!(value["task"], TASK);
    assert_eq!(value["task_generation"], "3");
    assert_eq!(value["attempt"], ATTEMPT);
    assert_eq!(value["attempt_generation"], "1");
    assert_eq!(value["criteria"], CRITERIA);
    assert_eq!(value["objects"][0]["digest"], HELLO);
    no_acceptance(&area);
    assert_eq!(count(&area, "events"), 3);
}

#[test]
fn manifest_requires_nonempty_unique_available_proof_objects() {
    let area = Area::new();
    let mut store = area.open();
    let active = verifying(&mut store);
    let object = store.publish(b"hello", uuid(STAGE), deadline()).unwrap();
    assert!(matches!(
        store.prepare_acceptance(&active, uuid(ACCEPTED), &[], deadline()),
        Err(Error::Bound)
    ));
    assert!(matches!(
        store.prepare_acceptance(
            &active,
            uuid(ACCEPTED),
            &[object.clone(), object.clone()],
            deadline()
        ),
        Err(Error::Invalid)
    ));
    fs::remove_file(area.object_path(&object)).unwrap();
    assert!(
        store
            .prepare_acceptance(&active, uuid(ACCEPTED), &[object], deadline())
            .is_err()
    );
    no_acceptance(&area);
}

#[test]
fn manifest_publication_failure_retains_durable_unreferenced_bytes() {
    let area = Area::new();
    let mut store = area.open();
    let active = verifying(&mut store);
    let object = store.publish(b"hello", uuid(STAGE), deadline()).unwrap();
    store.fault = Some(CutPoint::ManifestPublished);
    injected(
        store.prepare_acceptance(&active, uuid(ACCEPTED), &[object], deadline()),
        CutPoint::ManifestPublished,
    );
    no_acceptance(&area);
    store.fault = None;
    let published = proof(&store, &active);
    assert!(store.read_object(published.object(), deadline()).is_ok());
    store.accept(&published, 20, deadline()).unwrap();
}

/// A stored role that no longer validates is a corrupt ledger: acceptance refuses instead of
/// addressing the outbox to a principal built around `Principal::new`'s checks (A24).
#[test]
fn acceptance_refuses_a_stored_principal_role_that_no_longer_validates() {
    let area = Area::new();
    let mut store = area.open();
    let active = verifying(&mut store);
    let published = proof(&store, &active);
    area.edit_closed(&format!(
        "UPDATE tasks SET principal_role='bad role' WHERE id='{TASK}';"
    ));
    assert!(matches!(
        store.accept(&published, 20, deadline()),
        Err(Error::Corrupt)
    ));
    assert_eq!(count(&area, "acceptances"), 0);
    assert_eq!(count(&area, "outbox"), 0);
}

#[test]
fn acceptance_commits_objects_history_budget_and_outbox_as_one_unit() {
    let area = Area::new();
    let mut store = area.open();
    let published = accepted(&mut store);
    let head = head_of(&store);
    assert_eq!(head.accepted_event.as_deref(), Some(ACCEPTED));
    assert_eq!(head.state, "accepted");
    assert_eq!(
        (
            head.spent_ms,
            head.reserved_work_ms,
            head.reserved_verify_ms
        ),
        (50, 0, 0)
    );
    assert_eq!(count(&area, "artifacts"), 2);
    assert_eq!(count(&area, "acceptances"), 1);
    assert_eq!(count(&area, "acceptance_objects"), 1);
    assert_eq!(count(&area, "outbox"), 1);
    assert_eq!(
        store.pending_delivery(10, deadline()).unwrap(),
        vec![(ACCEPTED.into(), "1000:operator".into(), 4)]
    );
    let stored: String = area
        .inspect()
        .query_row("SELECT manifest_digest FROM acceptances", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(stored, published.object().digest());
}

#[test]
fn acceptance_write_failure_rolls_back_all_sql_but_retains_published_proof() {
    let area = Area::new();
    let mut store = area.open();
    let active = verifying(&mut store);
    let published = proof(&store, &active);
    store.fault = Some(CutPoint::AcceptanceWrite);
    injected(
        store.accept(&published, 20, deadline()),
        CutPoint::AcceptanceWrite,
    );
    no_acceptance(&area);
    assert_eq!(head_of(&store).state, "verifying");
    assert_eq!(head_of(&store).spent_ms, 30);
    assert!(store.read_object(published.object(), deadline()).is_ok());
    drop(store);
    let mut reopened = area.reopen();
    no_acceptance(&area);
    reopened.accept(&published, 20, deadline()).unwrap();
}

#[test]
fn lost_acceptance_reply_recovers_one_historical_event_and_delivery_obligation() {
    let area = Area::new();
    let mut store = area.open();
    let active = verifying(&mut store);
    let published = proof(&store, &active);
    store.fault = Some(CutPoint::AfterCommit);
    assert!(matches!(
        store.accept(&published, 20, deadline()),
        Err(Error::UncertainCommit)
    ));
    drop(store);
    let mut reopened = area.reopen();
    assert_eq!(head_of(&reopened).accepted_event.as_deref(), Some(ACCEPTED));
    assert_eq!(reopened.pending_delivery(10, deadline()).unwrap().len(), 1);
    assert!(matches!(
        reopened.accept(&published, 20, deadline()),
        Err(Error::Conflict)
    ));
    assert_eq!(count(&area, "events"), 4);
    assert_eq!(head_of(&reopened).spent_ms, 50);
}

#[test]
fn acceptance_rechecks_verification_budget_and_current_object_availability() {
    let area = Area::new();
    let mut store = area.open();
    let active = verifying(&mut store);
    let published = proof(&store, &active);
    assert!(matches!(
        store.accept(&published, 201, deadline()),
        Err(Error::Budget)
    ));
    no_acceptance(&area);
    fs::remove_file(area.object_path(&published.data.objects[0])).unwrap();
    assert!(store.accept(&published, 20, deadline()).is_err());
    no_acceptance(&area);
    assert_eq!(head_of(&store).reserved_verify_ms, 200);
}

#[test]
fn delivery_ack_is_recipient_scoped_idempotent_and_never_reexecutes_task() {
    let area = Area::new();
    let mut store = area.open();
    accepted(&mut store);
    let before = head_of(&store);
    assert!(matches!(
        store.acknowledge_delivery(uuid(ACCEPTED), "1001:operator", deadline()),
        Err(Error::NotFound)
    ));
    assert_eq!(store.pending_delivery(1, deadline()).unwrap().len(), 1);
    assert!(matches!(
        store.pending_delivery(0, deadline()),
        Err(Error::Bound)
    ));
    assert!(matches!(
        store.pending_delivery(257, deadline()),
        Err(Error::Bound)
    ));
    for _ in 0..2 {
        store
            .acknowledge_delivery(uuid(ACCEPTED), "1000:operator", deadline())
            .unwrap();
    }
    assert!(store.pending_delivery(256, deadline()).unwrap().is_empty());
    assert_eq!(head_of(&store), before);
    drop(store);
    let reopened = area.reopen();
    assert!(reopened.pending_delivery(1, deadline()).unwrap().is_empty());
    assert_eq!(count(&area, "outbox"), 1);
}

#[test]
fn expired_operation_deadlines_create_no_mutation_or_object() {
    let area = Area::new();
    assert!(matches!(
        Store::open(&area.path, uuid(GEN), uuid(EPOCH), true, Instant::now()),
        Err(Error::Deadline)
    ));
    assert!(!area.path.join("store.lock").exists());
    let mut store = area.open();
    assert!(matches!(
        store.submit(submission(&principal()), Instant::now()),
        Err(Error::Deadline)
    ));
    no_admission(&area);
    assert!(matches!(
        store.publish(b"hello", uuid(STAGE), Instant::now()),
        Err(Error::Deadline)
    ));
    assert_eq!(fs::read_dir(&store.objects.path).unwrap().count(), 0);
}

#[test]
fn sqlite_writer_contention_obeys_one_short_deadline_without_semantic_retry() {
    let area = Area::new();
    let mut store = area.open();
    let blocker = Connection::open(area.database()).unwrap();
    blocker.execute_batch("BEGIN IMMEDIATE;").unwrap();
    let started = Instant::now();
    assert!(
        store
            .submit(
                submission(&principal()),
                started + Duration::from_millis(50)
            )
            .is_err()
    );
    assert!(started.elapsed() < Duration::from_secs(3));
    blocker.execute_batch("ROLLBACK;").unwrap();
    blocker.close().unwrap();
    no_admission(&area);
    assert_eq!(admit(&mut store).sequence, 1);
}

#[test]
fn quiesced_backup_independently_reopens_with_all_objects_and_pending_delivery() {
    let area = Area::new();
    let destination = Area::new();
    let mut store = area.open();
    accepted(&mut store);
    // A new backup must install its own caller deadline before its first SQL.
    // This stale callback deterministically represents the completed operation.
    store.connection.progress_handler(1, Some(|| true)).unwrap();
    let report = store.backup(&destination.path, deadline()).unwrap();
    assert_eq!(
        report.operational_status,
        backup::RestoreStatus::Unqualified
    );
    assert!(report.step_done);
    assert!(report.finish_call_via_drop);
    assert!(!report.finish_return_observed);
    assert_eq!(report.epoch, EPOCH);
    assert_eq!(report.generation, GEN);
    assert_eq!(report.cutoff, 4);
    assert_eq!(report.counts["outbox"], 1);
    assert_eq!(report.objects.len(), 2);
    assert_eq!(inspect_backup(&destination).unwrap(), report);
    let db = Connection::open_with_flags(
        destination.path.join("ledger.sqlite3"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert_eq!(
        db.query_row("SELECT delivered FROM outbox", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        db.query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "delete"
    );
    assert!(!destination.path.join("ledger.sqlite3-wal").exists());
}

#[test]
fn backup_refuses_running_or_unknown_custody_before_destination_mutation() {
    let area = Area::new();
    let destination = Area::new();
    let mut store = area.open();
    let active = running(&mut store);
    assert!(matches!(
        store.backup(&destination.path, deadline()),
        Err(Error::Outstanding)
    ));
    assert!(!destination.path.join("ledger.sqlite3").exists());
    store
        .settle_attempt(
            &active,
            settled(Effect::Unknown, None, false, false),
            uuid(SETTLED),
            deadline(),
        )
        .unwrap();
    assert!(matches!(
        store.backup(&destination.path, deadline()),
        Err(Error::Outstanding)
    ));
    assert!(!destination.path.join("store-backup.json").exists());
}

#[test]
fn backup_of_missing_registered_proof_is_incomplete_and_retains_source_history() {
    let area = Area::new();
    let destination = Area::new();
    let mut store = area.open();
    let published = accepted(&mut store);
    fs::remove_file(area.object_path(&published.data.objects[0])).unwrap();
    assert!(store.backup(&destination.path, deadline()).is_err());
    assert!(!destination.path.join("store-backup.json").exists());
    assert_eq!(head_of(&store).accepted_event.as_deref(), Some(ACCEPTED));
}

#[test]
fn backup_cut_points_never_publish_a_completion_manifest_early() {
    for point in [
        CutPoint::BackupCopied,
        CutPoint::BackupObject,
        CutPoint::BackupManifest,
    ] {
        let area = Area::new();
        let destination = Area::new();
        let mut store = area.open();
        accepted(&mut store);
        store.fault = Some(point);
        injected(store.backup(&destination.path, deadline()), point);
        assert!(destination.path.join("ledger.sqlite3").exists());
        assert!(!destination.path.join("store-backup.json").exists());
        assert!(
            Store::inspect_backup(
                &destination.path,
                Sha256Digest::parse(CRITERIA).unwrap(),
                deadline()
            )
            .is_err()
        );
        assert_eq!(count(&area, "outbox"), 1);
        assert_eq!(head_of(&store).state, "accepted");
    }
}

#[test]
fn backup_inspection_rejects_changed_manifest_and_database_bytes() {
    let area = Area::new();
    let destination = Area::new();
    let mut store = area.open();
    accepted(&mut store);
    store.backup(&destination.path, deadline()).unwrap();
    let identity = backup_digest(&destination);
    let manifest = destination.path.join("store-backup.json");
    let original = fs::read(&manifest).unwrap();
    fs::write(&manifest, b"{}").unwrap();
    assert!(matches!(
        Store::inspect_backup(
            &destination.path,
            Sha256Digest::parse(&identity).unwrap(),
            deadline()
        ),
        Err(Error::Corrupt)
    ));
    fs::write(&manifest, &original).unwrap();
    let database = destination.path.join("ledger.sqlite3");
    let mut bytes = fs::read(&database).unwrap();
    bytes[0] ^= 1;
    fs::write(database, bytes).unwrap();
    assert!(matches!(inspect_backup(&destination), Err(Error::Corrupt)));
}

#[test]
fn backup_inspection_reconciles_logical_inventory_not_only_manifest_hash() {
    let area = Area::new();
    let destination = Area::new();
    let mut store = area.open();
    accepted(&mut store);
    store.backup(&destination.path, deadline()).unwrap();
    let manifest = destination.path.join("store-backup.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    value["counts"]["outbox"] = serde_json::json!(0);
    fs::write(manifest, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(matches!(inspect_backup(&destination), Err(Error::Corrupt)));
    assert_eq!(store.pending_delivery(10, deadline()).unwrap().len(), 1);
}

#[test]
fn backup_inspection_rehashes_each_object_without_mutating_source() {
    let area = Area::new();
    let destination = Area::new();
    let mut store = area.open();
    let published = accepted(&mut store);
    store.backup(&destination.path, deadline()).unwrap();
    let object = &published.data.objects[0];
    let hex = &object.digest()[7..];
    let target = destination
        .path
        .join("objects/sha256")
        .join(&hex[..2])
        .join(hex);
    fs::set_permissions(&target, Permissions::from_mode(0o600)).unwrap();
    fs::write(target, b"jello").unwrap();
    assert!(matches!(inspect_backup(&destination), Err(Error::Corrupt)));
    assert_eq!(store.read_object(object, deadline()).unwrap(), b"hello");
}

#[test]
fn backup_refuses_existing_destination_without_overwriting_its_bytes() {
    let area = Area::new();
    let destination = Area::new();
    let mut store = area.open();
    admit(&mut store);
    let existing = destination.path.join("ledger.sqlite3");
    fs::write(&existing, b"retained predecessor").unwrap();
    assert!(store.backup(&destination.path, deadline()).is_err());
    assert_eq!(fs::read(existing).unwrap(), b"retained predecessor");
    assert!(!destination.path.join("store-backup.json").exists());
    assert_eq!(head_of(&store).reserved_work_ms, 800);
}

/// Expired reads cannot bypass the caller deadline or mutate retained facts.
#[test]
fn read_apis_enforce_the_same_expired_caller_deadline() {
    let area = Area::new();
    let mut store = area.open();
    admit(&mut store);
    let object = store.publish(b"hello", uuid(STAGE), deadline()).unwrap();
    assert!(matches!(
        store.get(&principal(), uuid(TASK), Instant::now()),
        Err(Error::Deadline)
    ));
    assert!(matches!(
        store.get_by_key(&principal(), uuid(KEY), Instant::now()),
        Err(Error::Deadline)
    ));
    assert!(matches!(
        store.read_object(&object, Instant::now()),
        Err(Error::Deadline)
    ));
    assert!(matches!(
        store.pending_delivery(1, Instant::now()),
        Err(Error::Deadline)
    ));
    assert_eq!(head_of(&store).state, "admitted");
    assert_eq!(store.read_object(&object, deadline()).unwrap(), b"hello");
    assert_eq!(count(&area, "events"), 1);
}
