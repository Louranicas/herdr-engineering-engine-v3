//! The dispatcher (B14b-1; design R20 round 2 in `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`):
//! the first production caller of [`super::runtime::dispatch`]. One thread in `serve`, one task at a
//! time: it waits for the store's next dispatchable task under the store's own guard
//! ([`StoreTasks::wait_dispatchable`]), runs it through `dispatch`, and classifies EVERY result into
//! one of three steps so that no result can leave the picked task re-pickable at once — the hot
//! loop the round-1 review named is unrepresentable here, because the classification is an
//! exhaustive `match` with no catch-all arm and the read never returns a task that has an attempt row.
//!
//! B14b-2 composes the production provider in `serve` (`native_provider::NativeProvider`, its roster
//! record installed once at start) and records each attempt's root in the ledger at begin. The
//! drain does not reach an in-flight exchange or check — one waits for it under the attempt's own
//! deadline (R21 D8, reversing R20 round 2 D5: the wake is B19/B21's, the engine unit's stop
//! timeout APP-22's). With no provider composed the
//! dispatcher enters a named state that carries why (`unavailable: no native provider (not
//! installed)`, or the compose refusal `serve` said at start — R21 round-1 LOW F8), reports it
//! once, stops picking and leaves the task `admitted` — P2c-R1.5's "stop it
//! `dispatch_unavailable`" revisited: the task is the owner's and the missing configuration the
//! operator's.

use super::backup_target::{BackupTarget, BackupUnready, FreeSpace};
use super::coordinator::RootIdError;
use super::runtime::{
    Admission, Admitted, CandidateSource, Dispatch, Error as RuntimeError, Outcome, Verifier,
    admit, drive,
};
use super::tasks::StoreTasks;
use crate::contracts::UuidV4;
use crate::contracts::roster::Selection;
use crate::store::{Dispatchable, Error as StoreError, ResolveRefusal};
use crate::task::driver;
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

/// Where the dispatcher's candidate source and verifier come from, per dispatch (B14b-1 D7).
pub trait Provider {
    type Source: CandidateSource;
    type Verifier: Verifier;
    /// Open the pair for one dispatch, or say by name why none is available. `admitted` is the
    /// dispatch's own admitted state (R21 N1): its window, baseline, drain and class profile — the
    /// provider computes no window of its own.
    ///
    /// # Errors
    /// [`Unavailable`], the dispatcher's named state.
    fn open(
        &mut self,
        next: &Dispatchable,
        admitted: &Admitted<'_>,
    ) -> Result<(Self::Source, Self::Verifier), Unavailable>;
}

/// Why the dispatcher cannot dispatch: a named dispatcher state, never a task's failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unavailable {
    /// No native provider was composed at `serve` start, by the compose refusal that said so once
    /// (R21 round-1 LOW F8): absent, or refused.
    NoNativeProvider(NoNative),
    /// The task owner has no class profile (not installed, or refused at read).
    NoClassProfile,
    /// The class declares no native model row (R21 D1): the native provider does not serve it.
    ClassNotNative,
    /// The operator's native file does not fit the class, by the declaration that does not.
    Native(NativeWhy),
    /// The daemon could not be resolved (R21 N6, N7): the resolver's refusal.
    Daemon(crate::worker::native::Error),
    /// The reviewed closure did not yield the class's candidate inputs (R21 N22).
    Closure,
    /// The class prompt refused an input, by the class profile's name for it (R21 N22).
    Prompt(super::candidates::ClassPromptError),
    /// RC01's backup-freshness gate refused the dispatch (OPS-2): the whole line, with the reason
    /// and both headroom numbers, is [`BackupWhy::line`]; [`Unavailable::name`] is its fixed prefix.
    Backup(BackupWhy),
}

/// Which of the operator's native declarations does not fit the class (R21 D1, N11).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWhy {
    /// The operator's manifest pin is not the one the class's native row names.
    Manifest,
    /// The class's adapter row is not one this build knows.
    Adapter,
    /// The client's working directory is not canonical, the engine's, and 0700, by its cause
    /// (B14b-2 review round 2, FT-11).
    Directory(crate::worker::native::DirectoryError),
}

/// Why `serve` composed no native provider (R21 round-1 LOW F8): the kind of the compose refusal
/// its startup line names once, kept so the dispatcher's own state tells an absent file from a
/// refused one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoNative {
    /// No operator file (nor its directory): nothing is configured.
    NotInstalled,
    /// No class profile could be read at start, so no adapter row was known.
    ClassProfile,
    /// The custody door refused the operator's directory or file, or it could not be read.
    Read,
    /// The operator's file was read but did not decode as `hee3.native/1`.
    Decode,
    /// The class's adapter row is not one this build knows.
    Adapter,
    /// The engine's own operator principal was refused.
    Principal,
    /// The store refused the agent record's install.
    Install,
}

impl Unavailable {
    /// The state's name, whole: one literal per variant and payload, so a new one is a compile
    /// error here.
    #[must_use]
    pub const fn name(self) -> &'static str {
        use super::candidates::ClassPromptError as Input;
        use crate::worker::aggregate::Error as Manager;
        use crate::worker::native::{DirectoryError as Directory, Error as Native};
        match self {
            Self::NoNativeProvider(NoNative::NotInstalled) => {
                "unavailable: no native provider (not installed)"
            }
            Self::NoNativeProvider(NoNative::ClassProfile) => {
                "unavailable: no native provider (no class profile at start)"
            }
            Self::NoNativeProvider(NoNative::Read) => {
                "unavailable: native provider refused (file read)"
            }
            Self::NoNativeProvider(NoNative::Decode) => {
                "unavailable: native provider refused (file decode)"
            }
            Self::NoNativeProvider(NoNative::Adapter) => {
                "unavailable: native provider refused (adapter unknown)"
            }
            Self::NoNativeProvider(NoNative::Principal) => {
                "unavailable: native provider refused (principal)"
            }
            Self::NoNativeProvider(NoNative::Install) => {
                "unavailable: native provider refused (install)"
            }
            Self::NoClassProfile => "unavailable: no class profile",
            Self::ClassNotNative => "unavailable: class declares no native model",
            Self::Native(NativeWhy::Manifest) => "unavailable: native manifest",
            Self::Native(NativeWhy::Adapter) => "unavailable: native adapter",
            Self::Native(NativeWhy::Directory(Directory::NotCanonical)) => {
                "unavailable: native directory not canonical"
            }
            Self::Native(NativeWhy::Directory(Directory::Unreadable(_))) => {
                "unavailable: native directory unreadable"
            }
            Self::Native(NativeWhy::Directory(Directory::Custody)) => {
                "unavailable: native directory custody"
            }
            Self::Daemon(Native::Profile) => "unavailable: daemon profile",
            Self::Daemon(Native::Subject) => "unavailable: daemon subject",
            Self::Daemon(Native::Deadline) => "unavailable: daemon deadline",
            Self::Daemon(Native::Cancelled) => "unavailable: daemon cancelled",
            Self::Daemon(Native::Json) => "unavailable: daemon json",
            Self::Daemon(Native::Identity) => "unavailable: daemon identity",
            Self::Daemon(Native::Usage) => "unavailable: daemon usage",
            Self::Daemon(Native::Response) => "unavailable: daemon response",
            Self::Daemon(Native::Process) => "unavailable: daemon process",
            Self::Daemon(Native::Contract(_)) => "unavailable: daemon contract",
            Self::Daemon(Native::Census(_)) => "unavailable: daemon census",
            Self::Daemon(Native::Candidates(_)) => "unavailable: daemon candidates",
            Self::Daemon(Native::Matches(_)) => "unavailable: daemon matches",
            Self::Daemon(Native::Manager(Manager::Invalid)) => {
                "unavailable: daemon manager invalid"
            }
            Self::Daemon(Native::Manager(Manager::Bound)) => "unavailable: daemon manager bound",
            Self::Daemon(Native::Manager(Manager::Deadline)) => {
                "unavailable: daemon manager deadline"
            }
            Self::Daemon(Native::Manager(Manager::Cancelled)) => {
                "unavailable: daemon manager cancelled"
            }
            Self::Daemon(Native::Manager(Manager::Identity)) => {
                "unavailable: daemon manager identity"
            }
            Self::Daemon(Native::Manager(Manager::Io)) => "unavailable: daemon manager io",
            Self::Daemon(Native::Manager(Manager::State)) => "unavailable: daemon manager state",
            Self::Daemon(Native::Manager(Manager::Manager)) => {
                "unavailable: daemon manager manager"
            }
            Self::Daemon(Native::Manager(Manager::Process)) => {
                "unavailable: daemon manager process"
            }
            Self::Daemon(Native::Manager(Manager::Limits)) => "unavailable: daemon manager limits",
            Self::Daemon(Native::Manager(Manager::Busy)) => "unavailable: daemon manager busy",
            Self::Closure => "unavailable: reviewed closure",
            Self::Prompt(Input::Task) => "unavailable: prompt task",
            Self::Prompt(Input::Cargo) => "unavailable: prompt cargo",
            Self::Prompt(Input::Base) => "unavailable: prompt base",
            Self::Backup(_) => "unavailable: backup",
        }
    }
}

/// RC01 "Backup freshness" (docs/contract-decisions.md): the latest complete backup must be at most
/// 15 minutes old before any dispatch.
pub(crate) const BACKUP_FRESHNESS: Duration = Duration::from_mins(15);
/// RC01 "Backup freshness": a backup at every batch boundary, at most 8 tasks.
pub(crate) const BATCH_BOUNDARY: u32 = 8;
/// RC01 "Persistent capacity": at least 96 GiB free on the state filesystem before dispatch.
pub(crate) const STATE_RESERVE: u64 = 96 << 30;
/// RC01 "Persistent capacity": at least 256 GiB free on the backup filesystem before dispatch.
pub(crate) const BACKUP_RESERVE: u64 = 256 << 30;

/// Why a backup is due (RC01): none yet in this process, the last one aged past
/// [`BACKUP_FRESHNESS`], or [`BATCH_BOUNDARY`] dispatches since it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Due {
    Never,
    Aged,
    Batch,
}

/// RC01's freshness rule, pure (F95): `last` is this process's last complete backup on the
/// monotonic clock, `dispatched_since` the dispatches since it. Age is checked before the batch.
/// "<=15 minutes" is fresh, so exactly [`BACKUP_FRESHNESS`] old is not due.
pub(crate) fn backup_due(
    last: Option<Instant>,
    dispatched_since: u32,
    now: Instant,
) -> Option<Due> {
    let Some(last) = last else {
        return Some(Due::Never);
    };
    if now.saturating_duration_since(last) > BACKUP_FRESHNESS {
        Some(Due::Aged)
    } else if dispatched_since >= BATCH_BOUNDARY {
        Some(Due::Batch)
    } else {
        None
    }
}

/// Which filesystem RC01's headroom names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fs {
    /// The state root's.
    State,
    /// The backup destination's.
    Backup,
}

impl Fs {
    const fn name(self) -> &'static str {
        match self {
            Self::State => "state",
            Self::Backup => "backup",
        }
    }
}

/// A headroom refusal with both numbers (HO-03 C10 `headroom{free, reserve}`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Headroom {
    pub fs: Fs,
    pub free: u64,
    pub reserve: u64,
}

/// RC01's headroom rule, pure: `free` must be at least `reserve`.
pub(crate) fn headroom(fs: Fs, free: u64, reserve: u64) -> Result<(), Headroom> {
    if free < reserve {
        Err(Headroom { fs, free, reserve })
    } else {
        Ok(())
    }
}

/// RC01's headroom decision, pure (F95; RA1 a): each filesystem in RC01's order (the state root's,
/// then the destination's) with its measured free bytes (`None`: unmeasured) and its reserve. The
/// first that is unmeasured or short refuses, naming the one filesystem its own numbers determine,
/// and nothing after it is consumed: the caller measures lazily, so a short state root leaves the
/// destination unmeasured.
pub(crate) fn headroom_decision(
    measured: impl IntoIterator<Item = (Fs, Option<u64>, u64)>,
) -> Result<(), BackupWhy> {
    for (fs, free, reserve) in measured {
        let free = free.ok_or(BackupWhy::Unmeasured(fs))?;
        headroom(fs, free, reserve).map_err(BackupWhy::Headroom)?;
    }
    Ok(())
}

/// Why RC01's backup-freshness gate refused (OPS-2), each by its own name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackupWhy {
    /// The store's backup door refused, by the reason `store_reason` names (HO-03 C10
    /// `backup_failed{reason}`).
    Store(&'static str),
    /// A filesystem is below its RC01 reserve.
    Headroom(Headroom),
    /// A filesystem's free space could not be measured.
    Unmeasured(Fs),
    /// There is no backup target: `serve` said why once at start.
    Target(BackupUnready),
    /// No backup id could be drawn under the backup's deadline.
    Id,
    /// The backup's own directory could not be made under the destination.
    Destination(&'static str),
}

impl BackupWhy {
    /// The one renderer of every refusal line the gate reports (after `dispatcher: `).
    #[must_use]
    pub fn line(&self) -> String {
        match self {
            Self::Store(reason) => format!("unavailable: backup failed ({reason})"),
            Self::Headroom(Headroom { fs, free, reserve }) => format!(
                "unavailable: headroom ({}: free={free} reserve={reserve})",
                fs.name()
            ),
            Self::Unmeasured(fs) => format!("unavailable: headroom unmeasured ({})", fs.name()),
            Self::Target(why) => format!("unavailable: no backup target ({})", why.line()),
            Self::Id => "unavailable: backup failed (id)".to_owned(),
            Self::Destination(reason) => {
                format!("unavailable: backup failed (destination {reason})")
            }
        }
    }
}

/// The store's refusal as the backup-failed line names it: a total `match` with no catch-all, so a
/// new store error is a compile error here (the `store_step` precedent).
const fn store_reason(error: &StoreError) -> &'static str {
    match error {
        StoreError::Outstanding => "outstanding",
        StoreError::Bound => "bound",
        StoreError::Custody => "custody",
        StoreError::Conflict => "conflict",
        StoreError::Deadline => "deadline",
        StoreError::Corrupt => "corrupt",
        StoreError::UncertainCommit => "uncertain commit",
        StoreError::Full => "full",
        StoreError::Invalid => "invalid",
        StoreError::Forbidden => "forbidden",
        StoreError::NotFound => "not found",
        StoreError::Cancelled => "cancelled",
        StoreError::Budget => "budget",
        StoreError::Locked => "locked",
        StoreError::TaskViewBound { .. } => "task view bound",
        StoreError::StartupBound { .. } => "startup bound",
        StoreError::StaleGeneration { .. } => "stale generation",
        StoreError::SnapshotAhead { .. } => "snapshot ahead",
        StoreError::SnapshotMoved { .. } => "snapshot moved",
        StoreError::Disposition(_) => "disposition",
        StoreError::EvidenceIdentity => "evidence identity",
        StoreError::EvidenceBound { .. } => "evidence bound",
        StoreError::EventsBound { .. } => "events bound",
        StoreError::AlreadyStopped => "already stopped",
        StoreError::NotWritable => "not writable",
        StoreError::UnsupportedSchema => "unsupported schema",
        StoreError::Chain(_) => "chain",
        StoreError::UpgradeRequired { .. } => "upgrade required",
        StoreError::Runtime => "runtime",
        StoreError::Constraint => "constraint",
        StoreError::RecoveryRequired => "recovery required",
        StoreError::InspectionOnly => "inspection only",
        StoreError::Cleanup { .. } => "cleanup",
        StoreError::Rollback { .. } => "rollback",
        StoreError::Io(_) => "io",
        StoreError::Os(_) => "os",
        StoreError::Sqlite(_) => "sqlite",
        StoreError::Encoding(_) => "encoding",
        #[cfg(test)]
        StoreError::Injected(_) => "injected",
    }
}

/// An I/O failure's kind as a static reason: the kinds a directory create meets by name, the rest
/// as `io` (`ErrorKind` is non-exhaustive, so it cannot be matched totally).
const fn io_reason(kind: std::io::ErrorKind) -> &'static str {
    match kind {
        std::io::ErrorKind::NotFound => "io:not_found",
        std::io::ErrorKind::PermissionDenied => "io:permission_denied",
        std::io::ErrorKind::AlreadyExists => "io:already_exists",
        std::io::ErrorKind::StorageFull => "io:storage_full",
        std::io::ErrorKind::ReadOnlyFilesystem => "io:read_only_filesystem",
        _ => "io",
    }
}

/// This process's backup freshness, held on the monotonic clock (HO-03 as amended): set only from a
/// backup the store's door read back. A new process has none, so its first pick backs up.
#[derive(Default)]
struct Freshness {
    last: Option<Instant>,
    dispatched_since: u32,
}

/// Why the gate did not pass: RC01's refusal, or a ledger owner that panicked.
enum Gate {
    Refused(BackupWhy),
    Poisoned,
}

impl Gate {
    /// The exit this gate's refusal stops the dispatcher as, its line reported first.
    fn stop(self, report: &(dyn Fn(&str) + Sync)) -> Exit {
        match self {
            Self::Poisoned => Exit::Poisoned,
            Self::Refused(why) => {
                report(&format!("dispatcher: {}", why.line()));
                Exit::Unavailable(Unavailable::Backup(why))
            }
        }
    }
}

/// RC01's gate (OPS-2): when a backup is due ([`backup_due`]), check both filesystems' headroom,
/// then back up through the store's one door into a fresh 0700 `<destination>/<backup-id>/`, under
/// the operator's deadline, and report the backup's line. Quiesce is the door's own rule (it refuses
/// `Outstanding`). A refused backup's directory is removed when empty; a partial copy is inert,
/// since its manifest is published last and it is never reused (the door refuses a non-empty
/// destination).
fn ensure_fresh(
    tasks: &StoreTasks,
    backup: &Result<BackupTarget, BackupUnready>,
    space: &(dyn FreeSpace + Sync),
    freshness: &mut Freshness,
    report: &(dyn Fn(&str) + Sync),
) -> Result<(), Gate> {
    let Some(due) = backup_due(freshness.last, freshness.dispatched_since, Instant::now()) else {
        return Ok(());
    };
    let target = backup
        .as_ref()
        .map_err(|why| Gate::Refused(BackupWhy::Target(*why)))?;
    // Measured lazily, in RC01's order: the decision consumes the destination's measurement only
    // once the state root's has passed.
    headroom_decision(
        [
            (Fs::State, &target.state_root, STATE_RESERVE),
            (Fs::Backup, &target.destination, BACKUP_RESERVE),
        ]
        .into_iter()
        .map(|(fs, path, reserve)| (fs, space.free(path).ok(), reserve)),
    )
    .map_err(Gate::Refused)?;
    let deadline = Instant::now()
        .checked_add(target.deadline)
        .ok_or(Gate::Refused(BackupWhy::Store("deadline")))?;
    let id = super::evidence::fresh_id(deadline).map_err(|_| Gate::Refused(BackupWhy::Id))?;
    let directory = target.destination.join(id.as_str());
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&directory)
        .map_err(|error| Gate::Refused(BackupWhy::Destination(io_reason(error.kind()))))?;
    let backed = match tasks.with_store(|store| store.backup(&directory, deadline)) {
        Err(super::tasks::Poisoned) => Err(Gate::Poisoned),
        Ok(Err(error)) => Err(Gate::Refused(BackupWhy::Store(store_reason(&error)))),
        Ok(Ok(report)) => Ok(report),
    };
    let backed = match backed {
        Ok(backed) => backed,
        Err(gate) => {
            // Best effort, and only when empty: a partial copy is inert and never reused.
            let _ = std::fs::remove_dir(&directory);
            return Err(gate);
        }
    };
    freshness.last = Some(Instant::now());
    freshness.dispatched_since = 0;
    report(&format!(
        "dispatcher: backup {} complete: objects={}/{} database_bytes={} cutoff={} due={due:?}",
        id.as_str(),
        backed.objects.len(),
        crate::store::OBJECT_INVENTORY_BOUND,
        backed.database_bytes,
        backed.cutoff
    ));
    Ok(())
}

/// The provider when `serve` composed none: every `open` refuses by the compose refusal it holds
/// (R21 round-1 LOW F8).
pub struct NoProvider(pub NoNative);

/// A source that can never be asked: `NoProvider::open` refuses before one is built.
pub struct Never;

impl CandidateSource for Never {
    fn next(&mut self, _ask: &super::runtime::Ask<'_>) -> super::runtime::Answer {
        super::runtime::Answer {
            candidate: super::runtime::Candidate::Exhausted,
            settle: None,
        }
    }
    fn ready(
        &mut self,
        _deadline: Instant,
        _cancelled: &AtomicBool,
    ) -> Result<super::runtime::Readiness, crate::worker::native::Error> {
        Err(crate::worker::native::Error::Profile)
    }
    fn settle_retained(
        &mut self,
        _deadline: Instant,
        _cancelled: &AtomicBool,
    ) -> super::runtime::Custody {
        super::runtime::Custody::default()
    }
}

impl Verifier for Never {
    fn check(&mut self, plan: super::runtime::CheckPlan<'_>) -> super::runtime::Observed {
        super::runtime::Observed {
            run: Err(super::runtime::Unlaunched::Workload(
                super::workload::Error::Layout,
            )),
            observed: plan.window.begun,
            resources: super::runtime::Resources::NotHeld,
        }
    }
}

impl Provider for NoProvider {
    type Source = Never;
    type Verifier = Never;
    fn open(
        &mut self,
        _next: &Dispatchable,
        _admitted: &Admitted<'_>,
    ) -> Result<(Never, Never), Unavailable> {
        Err(Unavailable::NoNativeProvider(self.0))
    }
}

/// What one dispatch came to, for the loop: exactly three, so the loop cannot be written wrong.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Step {
    /// The task is no longer pickable: refused or stopped by name, or accepted.
    TaskDone(&'static str),
    /// An attempt row exists and the task is the recovery's (B17/B21): the read never returns it.
    TaskLeft(&'static str),
    /// The dispatcher stops, by name.
    DispatcherStops(&'static str),
}

/// The exhaustive classification (R20 round 2 D3): every `Outcome`, every `RuntimeError` variant and
/// every `StoreError` kind it can carry — no catch-all arm, so a new variant is a compile error here.
#[must_use]
pub fn classify(result: &Result<Outcome, RuntimeError>) -> Step {
    match result {
        Ok(Outcome::Refused(refusal)) => Step::TaskDone(refusal.name()),
        // Drained at attempt 1 the task is still `admitted` and picked again; at attempt ≥ 2 it is
        // `repair_pending`, which the read never returns — left for recovery (B17; closure item 9).
        Ok(Outcome::Drained) => Step::TaskLeft("drained: left for recovery"),
        // The source refused to be made ready (R21 N4): nothing written for the attempt. At attempt
        // 1 the task stays `admitted` and a next read would return it at once, so the dispatcher
        // stops by name (the `open` refusal's rule, N2); at attempt ≥ 2 the task is the recovery's.
        Ok(Outcome::NotReady(_)) => Step::DispatcherStops("provider not ready"),
        Ok(Outcome::Driven(driver::Outcome::Accepted)) => Step::TaskDone("accepted"),
        Ok(Outcome::Driven(driver::Outcome::Stopped(_))) => Step::TaskDone("stopped"),
        Ok(Outcome::Driven(driver::Outcome::NeedsSettlement(_))) => {
            Step::TaskLeft("needs settlement")
        }
        Err(RuntimeError::Poisoned) => Step::DispatcherStops("the ledger's owner panicked"),
        Err(RuntimeError::PreDispatch(_)) => {
            Step::DispatcherStops("an error before any attempt row: the dispatcher's")
        }
        Err(RuntimeError::ConcurrentWriter) => Step::TaskLeft("concurrent writer"),
        Err(RuntimeError::Identity) => Step::TaskLeft("identity"),
        Err(RuntimeError::Entropy) => Step::TaskLeft("entropy"),
        // Replaced under the running engine (B14b-2 review round 2, D9), told apart from a root
        // whose id cannot be read: every reader's kind is listed, so a new one is a compile error.
        // At attempt 1 it is raised before any attempt row, so it arrives inside `PreDispatch` (the
        // dispatcher's); bare, at attempt ≥ 2, the task is the recovery's — and still no later task
        // can begin under a replaced root, so the dispatcher stops by name there too.
        Err(RuntimeError::AttemptsRoot(RootIdError::Changed)) => {
            Step::DispatcherStops("attempts root changed")
        }
        Err(RuntimeError::AttemptsRoot(
            RootIdError::Absent
            | RootIdError::Custody
            | RootIdError::NotCanonical
            | RootIdError::Unmarked
            | RootIdError::Marker
            | RootIdError::Entropy
            | RootIdError::Deadline
            | RootIdError::Io(_),
        )) => Step::TaskLeft("attempts root unreadable"),
        // Raised only in `admit`, so it reaches here inside `PreDispatch`; bare, it is still before
        // any attempt row: the dispatcher's (R21 closure C13).
        Err(RuntimeError::PlanRoot(_)) => {
            Step::DispatcherStops("a leftover plan root could not be removed")
        }
        Err(RuntimeError::Policy(_)) => Step::TaskLeft("policy"),
        Err(RuntimeError::Store(error)) => store_step(error),
    }
}

/// A store error out of `dispatch` after an attempt row may exist: the task is the recovery's, unless
/// the ledger itself can no longer be written (the dispatcher stops), its storage is full, or the
/// object inventory is full (DS17: `Disposition(Inventory)`, raised by the registration door at the
/// first registration past the bound and by the acceptance door when the accepted objects would
/// cross it — closure H2 and closure 2; `Full` is `SQLITE_FULL`, its own name).
fn store_step(error: &StoreError) -> Step {
    match error {
        StoreError::UncertainCommit => {
            Step::DispatcherStops("uncertain commit: the ledger is poisoned")
        }
        StoreError::Full => Step::DispatcherStops("the ledger's storage is full (SQLITE_FULL)"),
        StoreError::Disposition(ResolveRefusal::Inventory) => {
            Step::DispatcherStops("store object inventory full (DS17)")
        }
        StoreError::NotWritable | StoreError::RecoveryRequired | StoreError::InspectionOnly => {
            Step::DispatcherStops("the ledger is not writable")
        }
        StoreError::Invalid
        | StoreError::Forbidden
        | StoreError::Bound
        | StoreError::Conflict
        | StoreError::NotFound
        | StoreError::Cancelled
        | StoreError::Outstanding
        | StoreError::Budget
        | StoreError::Deadline
        | StoreError::Locked
        | StoreError::Custody
        | StoreError::TaskViewBound { .. }
        | StoreError::StartupBound { .. }
        | StoreError::StaleGeneration { .. }
        | StoreError::SnapshotAhead { .. }
        | StoreError::SnapshotMoved { .. }
        | StoreError::Disposition(_)
        | StoreError::EvidenceIdentity
        | StoreError::EvidenceBound { .. }
        | StoreError::EventsBound { .. }
        | StoreError::AlreadyStopped
        | StoreError::Corrupt
        | StoreError::UnsupportedSchema
        | StoreError::Chain(_)
        | StoreError::UpgradeRequired { .. }
        | StoreError::Runtime
        | StoreError::Constraint
        | StoreError::Cleanup { .. }
        | StoreError::Rollback { .. }
        | StoreError::Io(_)
        | StoreError::Os(_)
        | StoreError::Sqlite(_)
        | StoreError::Encoding(_) => Step::TaskLeft("store refusal after begin"),
        #[cfg(test)]
        StoreError::Injected(_) => Step::TaskLeft("injected"),
    }
}

/// Why the dispatcher stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Exit {
    /// The drain ended the wait, or cancelled a provider's `open` (R21 round-1 LOW F6).
    Drained,
    /// A provider could not be opened.
    Unavailable(Unavailable),
    /// A result the classification named as the dispatcher's own stop.
    Stopped(&'static str),
    /// The ledger's owner panicked.
    Poisoned,
}

/// The teardown share every dispatch is handed (M8: derived, never a new limit): the cleanup grace
/// the check already keeps back. The capture share is the owner's reservation less this, derived in
/// `prepare` (closure M1) — nothing here fixes it.
#[must_use]
pub fn teardown_share() -> u64 {
    u64::try_from(super::runtime::CHECK_TEARDOWN.as_millis()).unwrap_or(u64::MAX)
}

/// What an `open` refusal ends the dispatcher as, and the line that says so (R21 round-1 LOW F6):
/// `open` is handed the drain as its resolve's cancel flag, so `Daemon(Cancelled)` with the drain
/// raised is the drain's exit, `Drained` — never a provider's unavailability. The drain decides it,
/// not the name: the same refusal with no drain raised, and any other refusal, stay the named
/// unavailable state.
fn open_refused(why: Unavailable, drained: bool) -> (Exit, String) {
    match (why, drained) {
        (Unavailable::Daemon(crate::worker::native::Error::Cancelled), true) => (
            Exit::Drained,
            format!(
                "dispatcher: drained while opening the provider ({})",
                why.name()
            ),
        ),
        _ => (
            Exit::Unavailable(why),
            format!("dispatcher: {}", why.name()),
        ),
    }
}

/// One dispatcher over one task owner: what every dispatch is handed, held together so the loop
/// takes one value (the eight loose arguments the first cut had were the lint's point).
pub struct Dispatcher<'a, P> {
    /// The task owner the socket serves: the one ledger and the one class profile (A11 — the
    /// profile is read from it inside `run`, never a second value).
    pub tasks: &'a StoreTasks,
    /// The attempts root under the state root (`coordinator::attempts_root`).
    pub attempts: &'a Path,
    /// The attempts root's id as `serve` prepared it at its start (B14b-2 review round 2, D9):
    /// carried into every dispatch, so a begin compares the marker it reads against it.
    pub root_id: &'a str,
    pub provider: &'a mut P,
    /// The roster inputs a begin needs (B14a-1c): the rig's in the proofs; in production the native
    /// install's record and selection (R21 N3), fixed at `serve` start — empty with no provider,
    /// when no task reaches `begin`.
    pub agent_record_id: &'a str,
    pub selections: &'a [Selection],
    /// The engine's drain (`Drain::flag`): ends the wait, and is read by the runtime between attempts.
    pub drain: &'a AtomicBool,
    /// Where RC01's backups go, or why there is none (OPS-2): read once at `serve` start.
    pub backup: &'a Result<BackupTarget, BackupUnready>,
    /// How RC01's headroom is measured (`backup_target::Statvfs` in `serve`).
    pub space: &'a (dyn FreeSpace + Sync),
}

impl<P: Provider> Dispatcher<'_, P> {
    /// Run until the dispatcher stops; `report` receives one line per step and the exit.
    pub fn run(self, report: &(dyn Fn(&str) + Sync)) -> Exit {
        run(self, report)
    }
}

fn run<P: Provider>(dispatcher: Dispatcher<'_, P>, report: &(dyn Fn(&str) + Sync)) -> Exit {
    let Dispatcher {
        tasks,
        attempts,
        root_id,
        provider,
        agent_record_id,
        selections,
        drain,
        backup,
        space,
    } = dispatcher;
    let Ok(profile) = tasks.class_profile() else {
        report(&format!(
            "dispatcher: {}",
            Unavailable::NoClassProfile.name()
        ));
        return Exit::Unavailable(Unavailable::NoClassProfile);
    };
    let teardown_ms = teardown_share();
    let stopped = || drain.load(std::sync::atomic::Ordering::SeqCst);
    // The task the last step handled: a read that returns it again is a defect in the rules above
    // (a handled task is stopped, accepted, or has an attempt row), stopped by name, never looped.
    let mut handled: Option<String> = None;
    let mut freshness = Freshness::default();
    loop {
        let next = match tasks.wait_dispatchable(&stopped) {
            Err(super::tasks::Poisoned) => return Exit::Poisoned,
            Ok(Err(error)) => {
                report(&format!("dispatcher: the read refused ({error:?})"));
                return Exit::Stopped("the dispatch read refused");
            }
            Ok(Ok(None)) => return Exit::Drained,
            Ok(Ok(Some(next))) => next,
        };
        if handled.as_deref() == Some(next.task.as_str()) {
            report(&format!(
                "dispatcher: the read returned the task just handled ({}): stopping",
                next.task
            ));
            return Exit::Stopped("the read returned the task just handled");
        }
        let Ok(task) = UuidV4::parse(&next.task) else {
            report(&format!(
                "dispatcher: a stored task id is not a UuidV4 ({})",
                next.task
            ));
            return Exit::Stopped("a stored task id is malformed");
        };
        // RC01 (OPS-2 point a): before any dispatch, the latest complete backup is fresh — or the
        // dispatcher stops by name and the task stays `admitted`.
        if let Err(exit) = ensure_fresh(tasks, backup, space, &mut freshness, report) {
            return exit.stop(report);
        }
        // Phase one before any provider (R20 round 2 A2/A4): the free checks and the captures; a
        // refusal stops the task by name here, and a provider is opened only for an admitted task.
        let admitted = admit(
            tasks,
            profile,
            Dispatch {
                principal: &next.owner,
                task,
                agent_record_id,
                selections,
                attempts,
                root_id,
                forbidden: &[],
                teardown_ms,
                capture_ms: None,
                drain,
            },
        );
        // The custody a driven dispatch's retained children came to (R21 N18), reported by name on
        // every exit of `drive` (closure C4).
        let (result, custody) = match admitted {
            Err(error) => (Err(error), None),
            Ok(Admission::Refused(refusal)) => (Ok(Outcome::Refused(refusal)), None),
            Ok(Admission::Ready(ready)) => {
                // A dispatch that reached a provider: what RC01's batch boundary counts.
                freshness.dispatched_since = freshness.dispatched_since.saturating_add(1);
                match provider.open(&next, &ready) {
                    Err(why) => {
                        let (exit, line) = open_refused(why, stopped());
                        report(&line);
                        return exit;
                    }
                    Ok((mut source, mut verifier)) => {
                        match drive(tasks, *ready, &mut source, &mut verifier) {
                            Ok(driven) => (Ok(driven.outcome), Some(driven.custody)),
                            Err(undispatched) => {
                                (Err(undispatched.error), Some(undispatched.custody))
                            }
                        }
                    }
                }
            }
        };
        let step = classify(&result);
        report(&step_line(&next.task, step, &result, custody));
        handled = Some(next.task.clone());
        if let Step::DispatcherStops(why) = step {
            return Exit::Stopped(why);
        }
        // RC01 (OPS-2 point b): after each task, back up if freshness expired or a batch ended.
        // A refusal is only reported: point (a) refuses the next dispatch if it is still due.
        if let Err(gate) = ensure_fresh(tasks, backup, space, &mut freshness, report)
            && let Exit::Poisoned = gate.stop(report)
        {
            return Exit::Poisoned;
        }
    }
}

/// The line one dispatch's step is reported by: the task, its step, a provider not ready by its
/// refusal's name (closure C4), an error whole, and the custody a driven dispatch came to.
fn step_line(
    task: &str,
    step: Step,
    result: &Result<Outcome, RuntimeError>,
    custody: Option<super::runtime::Custody>,
) -> String {
    let cause = match result {
        Ok(Outcome::NotReady(error)) => format!(": {}", error.name()),
        _ => String::new(),
    };
    format!(
        "dispatcher: task {task} -> {step:?}{cause}{}{}",
        result
            .as_ref()
            .err()
            .map(|error| format!(" ({error:?})"))
            .unwrap_or_default(),
        custody
            .map(|custody| format!(
                ", custody: settled={} pending={}",
                custody.settled, custody.pending
            ))
            .unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BACKUP_FRESHNESS, BACKUP_RESERVE, BATCH_BOUNDARY, BackupWhy, Due, Fs, Headroom, NativeWhy,
        NoNative, STATE_RESERVE, Step, Unavailable, backup_due, classify, headroom,
        headroom_decision, teardown_share,
    };
    use crate::app::candidates::ClassPromptError;
    use crate::app::coordinator::RootIdError;
    use crate::app::runtime::{Error as RuntimeError, Outcome, Refusal};
    use crate::store::{Error as StoreError, ResolveRefusal};
    use crate::task::LoopRefusal;
    use crate::task::driver::{self, StopReason};
    use crate::worker::aggregate::Error as Manager;
    use crate::worker::native::{DirectoryError as Directory, Error as Native};
    use std::time::{Duration, Instant};

    /// Every result the classification is pinned over, with its step.
    fn cases() -> Vec<(Result<Outcome, RuntimeError>, Step)> {
        vec![
            (
                Ok(Outcome::Refused(Refusal::CancelledBeforeDispatch)),
                Step::TaskDone("cancelled_before_dispatch"),
            ),
            (
                Ok(Outcome::Drained),
                Step::TaskLeft("drained: left for recovery"),
            ),
            (
                Ok(Outcome::NotReady(crate::worker::native::Error::Identity)),
                Step::DispatcherStops("provider not ready"),
            ),
            (
                Ok(Outcome::Driven(driver::Outcome::Accepted)),
                Step::TaskDone("accepted"),
            ),
            (
                Ok(Outcome::Driven(driver::Outcome::Stopped(
                    StopReason::WorkerFailed,
                ))),
                Step::TaskDone("stopped"),
            ),
            (
                Ok(Outcome::Driven(driver::Outcome::NeedsSettlement(
                    StopReason::Unsettled,
                ))),
                Step::TaskLeft("needs settlement"),
            ),
            (
                Err(RuntimeError::Poisoned),
                Step::DispatcherStops("the ledger's owner panicked"),
            ),
            (
                Err(RuntimeError::PreDispatch(Box::new(RuntimeError::Store(
                    StoreError::NotFound,
                )))),
                Step::DispatcherStops("an error before any attempt row: the dispatcher's"),
            ),
            (
                Err(RuntimeError::PreDispatch(Box::new(RuntimeError::Entropy))),
                Step::DispatcherStops("an error before any attempt row: the dispatcher's"),
            ),
            (
                Err(RuntimeError::ConcurrentWriter),
                Step::TaskLeft("concurrent writer"),
            ),
            (Err(RuntimeError::Identity), Step::TaskLeft("identity")),
            (Err(RuntimeError::Entropy), Step::TaskLeft("entropy")),
            (
                Err(RuntimeError::AttemptsRoot(RootIdError::Changed)),
                Step::DispatcherStops("attempts root changed"),
            ),
            (
                Err(RuntimeError::AttemptsRoot(RootIdError::Unmarked)),
                Step::TaskLeft("attempts root unreadable"),
            ),
            (
                Err(RuntimeError::AttemptsRoot(RootIdError::Io(
                    std::io::ErrorKind::PermissionDenied,
                ))),
                Step::TaskLeft("attempts root unreadable"),
            ),
            (
                Err(RuntimeError::PlanRoot(
                    crate::worker::workspace::Error::Custody,
                )),
                Step::DispatcherStops("a leftover plan root could not be removed"),
            ),
            (
                Err(RuntimeError::Policy(LoopRefusal::GenerationExhausted)),
                Step::TaskLeft("policy"),
            ),
        ]
        .into_iter()
        .chain(store_cases())
        .collect()
    }

    /// The store kinds the classification is pinned over: those that stop the dispatcher against
    /// those that leave the task to recovery.
    fn store_cases() -> Vec<(Result<Outcome, RuntimeError>, Step)> {
        vec![
            (
                Err(RuntimeError::Store(StoreError::UncertainCommit)),
                Step::DispatcherStops("uncertain commit: the ledger is poisoned"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Full)),
                Step::DispatcherStops("the ledger's storage is full (SQLITE_FULL)"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Disposition(
                    ResolveRefusal::Inventory,
                ))),
                Step::DispatcherStops("store object inventory full (DS17)"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Disposition(
                    ResolveRefusal::Resolved,
                ))),
                Step::TaskLeft("store refusal after begin"),
            ),
            (
                Err(RuntimeError::Store(StoreError::NotWritable)),
                Step::DispatcherStops("the ledger is not writable"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Conflict)),
                Step::TaskLeft("store refusal after begin"),
            ),
            (
                Err(RuntimeError::Store(StoreError::Corrupt)),
                Step::TaskLeft("store refusal after begin"),
            ),
        ]
    }

    /// The native provider's named states (R21 D2), every variant and payload with its whole name.
    fn native_names() -> Vec<(Unavailable, &'static str)> {
        let native = |why| Unavailable::Daemon(why);
        vec![
            (
                Unavailable::ClassNotNative,
                "unavailable: class declares no native model",
            ),
            (
                Unavailable::Native(NativeWhy::Manifest),
                "unavailable: native manifest",
            ),
            (
                Unavailable::Native(NativeWhy::Adapter),
                "unavailable: native adapter",
            ),
            (
                Unavailable::Native(NativeWhy::Directory(Directory::NotCanonical)),
                "unavailable: native directory not canonical",
            ),
            (
                Unavailable::Native(NativeWhy::Directory(Directory::Unreadable(
                    std::io::ErrorKind::NotFound,
                ))),
                "unavailable: native directory unreadable",
            ),
            (
                Unavailable::Native(NativeWhy::Directory(Directory::Custody)),
                "unavailable: native directory custody",
            ),
            (native(Native::Profile), "unavailable: daemon profile"),
            (native(Native::Subject), "unavailable: daemon subject"),
            (native(Native::Deadline), "unavailable: daemon deadline"),
            (native(Native::Cancelled), "unavailable: daemon cancelled"),
            (native(Native::Json), "unavailable: daemon json"),
            (native(Native::Identity), "unavailable: daemon identity"),
            (native(Native::Usage), "unavailable: daemon usage"),
            (native(Native::Response), "unavailable: daemon response"),
            (native(Native::Process), "unavailable: daemon process"),
            (
                native(Native::Contract(crate::worker::ContractError::Identity)),
                "unavailable: daemon contract",
            ),
            (
                native(Native::Census(
                    crate::worker::process::CensusError::Deadline,
                )),
                "unavailable: daemon census",
            ),
            (
                native(Native::Candidates(
                    crate::worker::process::DescendantBound {
                        found: 65,
                        limit: 64,
                    },
                )),
                "unavailable: daemon candidates",
            ),
            (
                native(Native::Matches(crate::worker::native::DaemonMatches {
                    matched: 2,
                    candidates: 3,
                })),
                "unavailable: daemon matches",
            ),
            (Unavailable::Closure, "unavailable: reviewed closure"),
            (
                Unavailable::Prompt(ClassPromptError::Task),
                "unavailable: prompt task",
            ),
            (
                Unavailable::Prompt(ClassPromptError::Cargo),
                "unavailable: prompt cargo",
            ),
            (
                Unavailable::Prompt(ClassPromptError::Base),
                "unavailable: prompt base",
            ),
        ]
    }

    /// R20 round 2 D3 · every result lands in exactly one of three steps, by a table: a refusal and
    /// every driven outcome; each runtime error; the store kinds that stop the dispatcher against
    /// the kinds that leave the task to recovery. A new variant is a compile error in `classify`.
    #[test]
    fn every_dispatch_result_is_classified_into_one_of_three_steps() {
        let cases = cases();
        for (result, expected) in &cases {
            assert_eq!(classify(result), *expected, "{result:?}");
        }
        // The teardown share is the check's grace, derived; the capture share is not fixed here.
        assert_eq!(
            u128::from(teardown_share()),
            crate::app::runtime::CHECK_TEARDOWN.as_millis()
        );
        assert_eq!(
            Unavailable::NoClassProfile.name(),
            "unavailable: no class profile"
        );
        // R21 D2 · the native provider's states, every variant and payload by its whole name.
        let native = |why| Unavailable::Daemon(why);
        for (state, name) in native_names() {
            assert_eq!(state.name(), name, "{state:?}");
        }
        // R21 closure C6 · the user manager's refusal, every kind by its whole name.
        for (kind, name) in [
            (Manager::Invalid, "unavailable: daemon manager invalid"),
            (Manager::Bound, "unavailable: daemon manager bound"),
            (Manager::Deadline, "unavailable: daemon manager deadline"),
            (Manager::Cancelled, "unavailable: daemon manager cancelled"),
            (Manager::Identity, "unavailable: daemon manager identity"),
            (Manager::Io, "unavailable: daemon manager io"),
            (Manager::State, "unavailable: daemon manager state"),
            (Manager::Manager, "unavailable: daemon manager manager"),
            (Manager::Process, "unavailable: daemon manager process"),
            (Manager::Limits, "unavailable: daemon manager limits"),
            (Manager::Busy, "unavailable: daemon manager busy"),
        ] {
            assert_eq!(native(Native::Manager(kind)).name(), name, "{kind:?}");
        }
    }

    /// R21 round-1 LOW F8 · no provider composed, by the compose refusal's kind, each name whole:
    /// an absent file is told apart from every refusal, and each refusal from the others.
    #[test]
    fn each_no_native_kind_has_its_whole_name() {
        for (kind, name) in [
            (
                NoNative::NotInstalled,
                "unavailable: no native provider (not installed)",
            ),
            (
                NoNative::ClassProfile,
                "unavailable: no native provider (no class profile at start)",
            ),
            (
                NoNative::Read,
                "unavailable: native provider refused (file read)",
            ),
            (
                NoNative::Decode,
                "unavailable: native provider refused (file decode)",
            ),
            (
                NoNative::Adapter,
                "unavailable: native provider refused (adapter unknown)",
            ),
            (
                NoNative::Principal,
                "unavailable: native provider refused (principal)",
            ),
            (
                NoNative::Install,
                "unavailable: native provider refused (install)",
            ),
        ] {
            assert_eq!(Unavailable::NoNativeProvider(kind).name(), name, "{kind:?}");
        }
    }

    /// OPS-2 · RC01's freshness rule at its boundaries (F103), asserted as one whole table: no
    /// backup yet is due; exactly 15 minutes old is fresh ("<=15 minutes") and one nanosecond more
    /// is aged; seven dispatches since are not a batch and eight are; age is named before the batch;
    /// a `now` before `last` (never on a monotonic clock) saturates to fresh.
    #[test]
    fn backup_due_is_rc01_s_rule_at_its_boundaries() {
        let t = Instant::now();
        let fifteen = Duration::from_mins(15);
        let cases = [
            ("none yet", None, 0, t),
            ("none yet, a batch", None, 8, t),
            ("exactly 15 min", Some(t), 0, t + fifteen),
            (
                "15 min + 1 ns",
                Some(t),
                0,
                t + fifteen + Duration::from_nanos(1),
            ),
            ("seven since", Some(t), 7, t + Duration::from_secs(1)),
            ("eight since", Some(t), 8, t + Duration::from_secs(1)),
            ("nine since", Some(t), 9, t + Duration::from_secs(1)),
            ("aged and a batch", Some(t), 8, t + Duration::from_secs(901)),
            ("now before last", Some(t + Duration::from_secs(5)), 0, t),
        ];
        let found: Vec<(&str, Option<Due>)> = cases
            .iter()
            .map(|(case, last, since, now)| (*case, backup_due(*last, *since, *now)))
            .collect();
        assert_eq!(
            found,
            vec![
                ("none yet", Some(Due::Never)),
                ("none yet, a batch", Some(Due::Never)),
                ("exactly 15 min", None),
                ("15 min + 1 ns", Some(Due::Aged)),
                ("seven since", None),
                ("eight since", Some(Due::Batch)),
                ("nine since", Some(Due::Batch)),
                ("aged and a batch", Some(Due::Aged)),
                ("now before last", None),
            ]
        );
    }

    /// OPS-2 · RC01's headroom rule refuses below the reserve with both numbers (HO-03 C10), and
    /// admits exactly the reserve and above: each filesystem at its own reserve, off the origin.
    #[test]
    fn headroom_refuses_below_the_reserve_with_both_numbers() {
        assert_eq!(
            [
                headroom(Fs::Backup, 274_877_906_943, BACKUP_RESERVE),
                headroom(Fs::Backup, 274_877_906_944, BACKUP_RESERVE),
                headroom(Fs::State, 103_079_215_103, STATE_RESERVE),
                headroom(Fs::State, 103_079_215_104, STATE_RESERVE),
                headroom(Fs::State, u64::MAX, STATE_RESERVE),
            ],
            [
                Err(Headroom {
                    fs: Fs::Backup,
                    free: 274_877_906_943,
                    reserve: 274_877_906_944
                }),
                Ok(()),
                Err(Headroom {
                    fs: Fs::State,
                    free: 103_079_215_103,
                    reserve: 103_079_215_104
                }),
                Ok(()),
                Ok(()),
            ]
        );
    }

    /// RA1 (a) · RC01's headroom decision is pure: each case's numbers determine the one filesystem
    /// that refuses, asserted whole. Short on the state root alone, then on the destination alone
    /// (two fixtures differing in every field), both short (the state root is named: RC01's order),
    /// each unmeasured, both exactly at their reserves, and reserves that are not RC01's (the
    /// reserve is the argument's, never the constant's).
    #[test]
    fn the_headroom_decision_names_the_filesystem_its_numbers_determine() {
        let state = |free: Option<u64>| (Fs::State, free, STATE_RESERVE);
        let backup = |free: Option<u64>| (Fs::Backup, free, BACKUP_RESERVE);
        let short = |fs, free, reserve| Err(BackupWhy::Headroom(Headroom { fs, free, reserve }));
        assert_eq!(
            [
                headroom_decision([state(Some(103_079_215_103)), backup(Some(u64::MAX))]),
                headroom_decision([state(Some(103_079_215_104)), backup(Some(274_877_906_937))]),
                headroom_decision([state(Some(5)), backup(Some(9))]),
                headroom_decision([state(None), backup(Some(3))]),
                headroom_decision([state(Some(1_099_511_627_776)), backup(None)]),
                headroom_decision([state(Some(103_079_215_104)), backup(Some(274_877_906_944))]),
                headroom_decision([(Fs::State, Some(10), 11), (Fs::Backup, Some(12), 12)]),
                headroom_decision([(Fs::State, Some(11), 11), (Fs::Backup, Some(12), 13)]),
            ],
            [
                short(Fs::State, 103_079_215_103, 103_079_215_104),
                short(Fs::Backup, 274_877_906_937, 274_877_906_944),
                short(Fs::State, 5, 103_079_215_104),
                Err(BackupWhy::Unmeasured(Fs::State)),
                Err(BackupWhy::Unmeasured(Fs::Backup)),
                Ok(()),
                short(Fs::State, 10, 11),
                short(Fs::Backup, 12, 13),
            ]
        );
    }

    /// RA1 (a) · the decision consumes no measurement after the one that refuses (so the caller's
    /// lazy measurement leaves the destination unasked when the state root is short), and consumes
    /// every one when all pass.
    #[test]
    fn the_headroom_decision_consumes_nothing_after_its_refusal() {
        let decided = |measured: [(Fs, Option<u64>, u64); 2]| {
            let consumed = std::cell::Cell::new(0_u32);
            let verdict = headroom_decision(
                measured
                    .into_iter()
                    .inspect(|_| consumed.set(consumed.get() + 1)),
            );
            (verdict, consumed.get())
        };
        assert_eq!(
            [
                decided([
                    (Fs::State, Some(7), STATE_RESERVE),
                    (Fs::Backup, Some(u64::MAX), BACKUP_RESERVE),
                ]),
                decided([
                    (Fs::State, None, STATE_RESERVE),
                    (Fs::Backup, None, BACKUP_RESERVE),
                ]),
                decided([
                    (Fs::State, Some(u64::MAX), STATE_RESERVE),
                    (Fs::Backup, Some(u64::MAX), BACKUP_RESERVE),
                ]),
            ],
            [
                (
                    Err(BackupWhy::Headroom(Headroom {
                        fs: Fs::State,
                        free: 7,
                        reserve: 103_079_215_104
                    })),
                    1
                ),
                (Err(BackupWhy::Unmeasured(Fs::State)), 1),
                (Ok(()), 2),
            ]
        );
    }

    /// The decimal number written immediately before `suffix` in `text`.
    fn number_before(text: &str, suffix: &str) -> Option<u64> {
        let head = &text[..text.find(suffix)?];
        let start = head
            .rfind(|c: char| !c.is_ascii_digit())
            .map_or(0, |index| index + 1);
        head[start..].parse().ok()
    }

    /// OPS-2 · the independent source for RC01's constants (F122): the published contract text,
    /// read at run time from the repository (not compiled in, so a missing document fails this case
    /// by name rather than the build). Each number is parsed out of its row and compared with the
    /// constant the gate enforces.
    #[test]
    fn rc01_s_numbers_are_the_contract_s() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/contract-decisions.md");
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !text.is_empty(),
            "the RC01 contract text is unreadable at {}",
            path.display()
        );
        let row = |name: &str| {
            text.lines()
                .find(|line| line.starts_with(&format!("| {name} |")))
                .unwrap_or_default()
        };
        let freshness = row("Backup freshness");
        let capacity = row("Persistent capacity");
        assert!(
            freshness.contains("latest complete backup must be <=15 minutes old"),
            "{freshness}"
        );
        assert!(
            freshness.contains("at every batch boundary (at most 8 tasks)"),
            "{freshness}"
        );
        assert!(
            capacity.contains(
                "reserve at least 96 GiB free on the state filesystem and 256 GiB on backup \
                 filesystem before dispatch"
            ),
            "{capacity}"
        );
        let minutes = number_before(freshness, " minutes old");
        let tasks = number_before(freshness, " tasks)");
        let state = number_before(capacity, " GiB free on the state filesystem");
        let backup = number_before(capacity, " GiB on backup filesystem");
        assert_eq!(
            (
                minutes.map(|minutes| Duration::from_secs(minutes * 60)),
                tasks,
                state.map(|gib| gib << 30),
                backup.map(|gib| gib << 30),
            ),
            (
                Some(BACKUP_FRESHNESS),
                Some(u64::from(BATCH_BOUNDARY)),
                Some(STATE_RESERVE),
                Some(BACKUP_RESERVE),
            )
        );
    }
}
