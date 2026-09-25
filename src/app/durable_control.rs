//! Parent-owned durable cancellation observation during one bounded workload.

use crate::app::workload::{self, Plan, Run};
use crate::contracts::{Generation, ScalarError, UuidV4};
use crate::store::{self, Principal, Store, TaskHead};
use crate::task::TASK_LIMIT;
use crate::worker::resources::Scope;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{
    Receiver, RecvTimeoutError, SyncSender, TryRecvError, TrySendError, sync_channel,
};
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(100);
const MAX_COMMANDS: usize = 12_001;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventId(String);
impl EventId {
    /// # Errors
    /// Refuses any noncanonical `UUIDv4` spelling.
    pub fn parse(value: &str) -> Result<Self, ScalarError> {
        UuidV4::parse(value)?;
        Ok(Self(value.to_owned()))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    fn borrowed(&self) -> UuidV4<'_> {
        UuidV4::parse(&self.0).expect("EventId is validated at construction")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskId(String);
impl TaskId {
    /// # Errors
    /// Refuses any noncanonical `UUIDv4` spelling.
    pub fn parse(value: &str) -> Result<Self, ScalarError> {
        UuidV4::parse(value)?;
        Ok(Self(value.to_owned()))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingCancel {
    pub task: TaskId,
    pub expected: Generation,
    pub event: EventId,
}

struct Command(PendingCancel);

#[derive(Clone)]
pub struct CancellationHandle {
    sender: SyncSender<Command>,
    task: TaskId,
}
pub struct CancellationQueue {
    receiver: Receiver<Command>,
    task: TaskId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnqueueFailure {
    Full,
    Disconnected,
}

#[derive(Debug)]
pub struct EnqueueError {
    pub failure: EnqueueFailure,
    pub command: PendingCancel,
}

impl CancellationHandle {
    #[must_use]
    pub fn task(&self) -> &TaskId {
        &self.task
    }
    /// Enqueue exactly one task-bound intent without waiting.
    /// # Errors
    /// Returns the unqueued command on full or disconnected capacity.
    pub fn try_cancel(&self, expected: Generation, event: EventId) -> Result<(), EnqueueError> {
        let command = Command(PendingCancel {
            task: self.task.clone(),
            expected,
            event,
        });
        self.sender
            .try_send(command)
            .map_err(|failure| match failure {
                TrySendError::Full(Command(command)) => EnqueueError {
                    failure: EnqueueFailure::Full,
                    command,
                },
                TrySendError::Disconnected(Command(command)) => EnqueueError {
                    failure: EnqueueFailure::Disconnected,
                    command,
                },
            })
    }
}

#[must_use]
pub fn cancellation_queue(task: TaskId) -> (CancellationHandle, CancellationQueue) {
    let (sender, receiver) = sync_channel(1);
    (
        CancellationHandle {
            sender,
            task: task.clone(),
        },
        CancellationQueue { receiver, task },
    )
}

#[derive(Debug)]
pub struct CancelObservation {
    pub command: PendingCancel,
    pub observed_after: Duration,
    pub result: Result<String, store::Error>,
}

/// One effect-boundary observation, measured from the caller's task origin.
pub struct Boundary {
    pub head: TaskHead,
    pub observed_after: Duration,
    pub command: Option<CancelObservation>,
}

impl CancellationQueue {
    #[must_use]
    pub fn task(&self) -> &TaskId {
        &self.task
    }

    /// Authorize the task read, service at most one intent, then read durable state.
    /// A command retained by `collect` is serviced before another queue entry.
    /// Neither an authorization failure nor an invalid clock consumes either input.
    /// Post-command read failures retain the exact command result in the error.
    /// The caller retains every returned observation, including stale CAS failures.
    ///
    /// # Errors
    /// Refuses a future origin, an expired or overlong task horizon, or failed readback.
    pub fn observe(
        &self,
        store: &mut Store,
        principal: &Principal,
        retained: &mut Option<PendingCancel>,
        origin: Instant,
        deadline: Instant,
    ) -> Result<Boundary, Error> {
        if retained
            .as_ref()
            .is_some_and(|command| command.task != self.task)
        {
            return Err(Error::TaskMismatch);
        }
        let now = Instant::now();
        if origin > now
            || now >= deadline
            || deadline
                .checked_duration_since(origin)
                .is_none_or(|value| value > TASK_LIMIT)
        {
            return Err(Error::InvalidDeadline);
        }
        let task = UuidV4::parse(self.task.as_str()).map_err(|_| Error::TaskMismatch)?;
        let read_deadline = poll_deadline(now, deadline);
        let initial =
            store
                .get(principal, task, read_deadline)
                .map_err(|error| Error::InitialRead {
                    error: Box::new(error),
                    command: None,
                })?;
        // An authorized read may itself exhaust the remaining task horizon.
        // Do not transfer command custody after that horizon has closed.
        let command_deadline = boundary_deadline(Instant::now(), deadline)?;
        let command = if let Some(command) = retained.take() {
            Some(commit_command(
                command,
                store,
                task,
                command_deadline,
                origin,
            ))
        } else {
            service_command(self, store, task, command_deadline, origin)
        };
        let head = if command.is_some() {
            match store.get(principal, task, command_deadline) {
                Ok(head) => head,
                Err(error) => {
                    return Err(Error::InitialRead {
                        error: Box::new(error),
                        command: command.map(Box::new),
                    });
                }
            }
        } else {
            initial
        };
        Ok(Boundary {
            head,
            observed_after: Instant::now().saturating_duration_since(origin),
            command,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cause {
    None,
    DurableCancellation,
    ObservationFailure,
    Deadline,
    WorkerPanic,
}

#[derive(Debug)]
pub enum Error {
    InitialRead {
        error: Box<store::Error>,
        command: Option<Box<CancelObservation>>,
    },
    CancellationFlagAlreadySet {
        command: Option<Box<CancelObservation>>,
    },
    TaskMismatch,
    InvalidDeadline,
}

pub struct Report {
    /// `None` means no workload was launched or its thread panicked.
    pub run: Option<Result<Run, workload::Error>>,
    /// The exact monotonic origin for every `observed_after` offset in this report.
    pub started_at: Instant,
    pub cause: Cause,
    pub last_head: TaskHead,
    pub first_observed_after: Option<Duration>,
    pub observation_error: Option<store::Error>,
    pub worker_panicked: bool,
    pub polls: u64,
    pub cancellations: Vec<CancelObservation>,
    pub pending_cancel: Option<PendingCancel>,
    pub queue_disconnected: bool,
}

enum Worker<T> {
    Returned(T),
    Panicked,
}

fn retain_observation<H, E>(
    observation: Result<(H, bool), E>,
    cancelled: &std::sync::atomic::AtomicBool,
    started: Instant,
    last: &mut H,
    cause: &mut Cause,
    observed_after: &mut Option<Duration>,
    observation_error: &mut Option<E>,
) {
    match observation {
        Ok((head, durable_cancelled)) => {
            *last = head;
            if durable_cancelled {
                *cause = Cause::DurableCancellation;
                *observed_after = Some(Instant::now().saturating_duration_since(started));
                cancelled.store(true, Ordering::Release);
            }
        }
        Err(error) => {
            *cause = Cause::ObservationFailure;
            *observed_after = Some(Instant::now().saturating_duration_since(started));
            *observation_error = Some(error);
            cancelled.store(true, Ordering::Release);
        }
    }
}

fn drive<T: Send, E, H>(
    started: Instant,
    deadline: Instant,
    cancelled: &std::sync::atomic::AtomicBool,
    initial: H,
    initial_cancelled: bool,
    mut observe: impl FnMut(Instant) -> Result<(H, bool), E>,
    work: impl FnOnce() -> T + Send,
) -> (Option<T>, Cause, H, Option<Duration>, Option<E>, bool, u64) {
    if initial_cancelled {
        cancelled.store(true, Ordering::Release);
        return (
            None,
            Cause::DurableCancellation,
            initial,
            Some(Duration::ZERO),
            None,
            false,
            0,
        );
    }
    let mut last = initial;
    let mut cause = Cause::None;
    let mut observed_after = None;
    let mut observation_error = None;
    let mut polls = 0_u64;
    let (sender, receiver) = sync_channel(1);
    let mut next_poll = started;
    let (result, panicked) = std::thread::scope(|scope| {
        scope.spawn(move || {
            let message = match catch_unwind(AssertUnwindSafe(work)) {
                Ok(value) => Worker::Returned(value),
                Err(_) => Worker::Panicked,
            };
            let _ = sender.send(message);
        });
        loop {
            let now = Instant::now();
            let wait = if cause == Cause::None {
                next_poll.min(deadline).saturating_duration_since(now)
            } else {
                POLL
            };
            match receiver.recv_timeout(wait) {
                Ok(Worker::Returned(value)) => {
                    if cause == Cause::None {
                        if Instant::now() < deadline {
                            polls = polls.saturating_add(1);
                            retain_observation(
                                observe(deadline),
                                cancelled,
                                started,
                                &mut last,
                                &mut cause,
                                &mut observed_after,
                                &mut observation_error,
                            );
                        } else {
                            cause = Cause::Deadline;
                            observed_after =
                                Some(Instant::now().saturating_duration_since(started));
                        }
                    }
                    return (Some(value), false);
                }
                Ok(Worker::Panicked) | Err(RecvTimeoutError::Disconnected) => {
                    return (None, true);
                }
                Err(RecvTimeoutError::Timeout) => {}
            }
            if cause != Cause::None {
                continue;
            }
            let now = Instant::now();
            if now >= deadline {
                // The workload owns timeout and cleanup. Deadline alone is not cancellation.
                cause = Cause::Deadline;
                observed_after = Some(now.saturating_duration_since(started));
                continue;
            }
            polls = polls.saturating_add(1);
            retain_observation(
                observe(deadline),
                cancelled,
                started,
                &mut last,
                &mut cause,
                &mut observed_after,
                &mut observation_error,
            );
            next_poll = next_poll.checked_add(POLL).unwrap_or(deadline);
        }
    });
    if panicked && cause == Cause::None {
        cause = Cause::WorkerPanic;
        observed_after = Some(Instant::now().saturating_duration_since(started));
    }
    (
        result,
        cause,
        last,
        observed_after,
        observation_error,
        panicked,
        polls,
    )
}

fn poll_deadline(now: Instant, deadline: Instant) -> Instant {
    now.checked_add(POLL).unwrap_or(deadline).min(deadline)
}

fn boundary_deadline(now: Instant, deadline: Instant) -> Result<Instant, Error> {
    if now >= deadline {
        Err(Error::InvalidDeadline)
    } else {
        Ok(poll_deadline(now, deadline))
    }
}

fn service_command(
    queue: &CancellationQueue,
    store: &mut Store,
    task: UuidV4<'_>,
    deadline: Instant,
    started: Instant,
) -> Option<CancelObservation> {
    let Command(command) = match queue.receiver.try_recv() {
        Ok(command) => command,
        Err(TryRecvError::Empty | TryRecvError::Disconnected) => return None,
    };
    Some(commit_command(command, store, task, deadline, started))
}

fn commit_command(
    command: PendingCancel,
    store: &mut Store,
    task: UuidV4<'_>,
    deadline: Instant,
    started: Instant,
) -> CancelObservation {
    let result = store.cancel(task, command.expected, command.event.borrowed(), deadline);
    CancelObservation {
        command,
        observed_after: Instant::now().saturating_duration_since(started),
        result,
    }
}

fn pending(queue: &CancellationQueue) -> (Option<PendingCancel>, bool) {
    match queue.receiver.try_recv() {
        Ok(Command(command)) => (Some(command), false),
        Err(TryRecvError::Empty) => (None, false),
        Err(TryRecvError::Disconnected) => (None, true),
    }
}

/// Run one bounded workload on an owned scoped thread while the parent and its
/// existing Store connection retain durable cancellation observation.
///
/// # Errors
/// Refuses a failed initial read or a flag already asserted without durable intent.
pub fn collect(
    store: &mut Store,
    principal: &Principal,
    task: UuidV4<'_>,
    plan: &Plan<'_>,
    scopes: &[Scope; 3],
    queue: &CancellationQueue,
) -> Result<Report, Error> {
    if queue.task.as_str() != task.as_str() {
        return Err(Error::TaskMismatch);
    }
    let started = Instant::now();
    if plan
        .deadline
        .checked_duration_since(started)
        .is_none_or(|remaining| remaining > TASK_LIMIT)
    {
        return Err(Error::InvalidDeadline);
    }
    let initial = match store.get(
        principal,
        task,
        poll_deadline(Instant::now(), plan.deadline),
    ) {
        Ok(head) => head,
        Err(error) => {
            return Err(Error::InitialRead {
                error: Box::new(error),
                command: None,
            });
        }
    };
    let command_deadline = boundary_deadline(Instant::now(), plan.deadline)?;
    let initial_command = service_command(queue, store, task, command_deadline, started);
    let initial = if initial_command.is_some() {
        match store.get(principal, task, command_deadline) {
            Ok(head) => head,
            Err(error) => {
                return Err(Error::InitialRead {
                    error: Box::new(error),
                    command: initial_command.map(Box::new),
                });
            }
        }
    } else {
        initial
    };
    let initial_observed_after = Instant::now().saturating_duration_since(started);
    let mut cancellations = initial_command.into_iter().collect::<Vec<_>>();
    if plan.cancelled.load(Ordering::Acquire) && !initial.cancellation {
        return Err(Error::CancellationFlagAlreadySet {
            command: cancellations.pop().map(Box::new),
        });
    }
    let (
        run,
        cause,
        last_head,
        mut first_observed_after,
        observation_error,
        worker_panicked,
        polls,
    ) = drive(
        started,
        plan.deadline,
        plan.cancelled,
        initial.clone(),
        initial.cancellation,
        |deadline| {
            let now = Instant::now();
            if now >= deadline {
                return Err(store::Error::Deadline);
            }
            let deadline = poll_deadline(now, deadline);
            if cancellations.len() < MAX_COMMANDS
                && let Some(observation) = service_command(queue, store, task, deadline, started)
            {
                cancellations.push(observation);
            }
            store
                .get(principal, task, deadline)
                .map(|head| (head.clone(), head.cancellation))
        },
        || workload::collect_bounded(plan, scopes),
    );
    if initial.cancellation {
        first_observed_after = Some(initial_observed_after);
    }
    let (pending_cancel, queue_disconnected) = pending(queue);
    Ok(Report {
        run,
        started_at: started,
        cause,
        last_head,
        first_observed_after,
        observation_error,
        worker_panicked,
        polls,
        cancellations,
        pending_cancel,
        queue_disconnected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::Sha256Digest;
    use crate::store::{Allocation, Submission};
    use std::fs;
    use std::os::unix::fs::DirBuilderExt;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::time::SystemTime;

    fn id(value: &str) -> UuidV4<'_> {
        UuidV4::parse(value).unwrap()
    }

    fn admitted_store() -> (PathBuf, Store, Principal, UuidV4<'static>) {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "hee3-durable-parent-{}-{nonce}",
            std::process::id()
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let principal = Principal::new(1000, "operator").unwrap();
        let task = id("33333333-3333-4333-8333-333333333333");
        let mut store = Store::open(
            &path,
            id("11111111-1111-4111-8111-111111111111"),
            id("22222222-2222-4222-8222-222222222222"),
            true,
            deadline,
        )
        .unwrap();
        store
            .submit(
                Submission {
                    principal: &principal,
                    key: id("44444444-4444-4444-8444-444444444444"),
                    task,
                    event: id("55555555-5555-4555-8555-555555555555"),
                    request_bytes: b"fixed request",
                    workspace_id: id("66666666-6666-4666-8666-666666666666"),
                    criteria: Sha256Digest::parse(
                        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                    )
                    .unwrap(),
                    allocation: Allocation {
                        limit_ms: 1_000,
                        work_ms: 800,
                        verify_ms: 200,
                    },
                },
                deadline,
            )
            .unwrap();
        (path, store, principal, task)
    }

    #[test]
    fn durable_cancel_and_observation_failure_are_distinct() {
        let flag = AtomicBool::new(false);
        let (value, cause, head, _, error, panicked, _) = drive(
            Instant::now(),
            Instant::now() + Duration::from_secs(1),
            &flag,
            1_u8,
            false,
            |_| Ok::<_, ()>((2, true)),
            || {
                while !flag.load(Ordering::Acquire) {
                    std::thread::yield_now();
                }
                7
            },
        );
        assert_eq!(
            (value, cause, head, error, panicked),
            (Some(7), Cause::DurableCancellation, 2, None, false)
        );

        let flag = AtomicBool::new(false);
        let (value, cause, head, _, error, panicked, _) = drive(
            Instant::now(),
            Instant::now() + Duration::from_secs(1),
            &flag,
            1_u8,
            false,
            |_| Err::<(u8, bool), _>("read-failed"),
            || {
                while !flag.load(Ordering::Acquire) {
                    std::thread::yield_now();
                }
                8
            },
        );
        assert_eq!(
            (value, cause, head, error, panicked),
            (
                Some(8),
                Cause::ObservationFailure,
                1,
                Some("read-failed"),
                false
            )
        );
    }

    #[test]
    fn deadline_does_not_assert_cancellation_and_panic_is_unknown() {
        let flag = AtomicBool::new(false);
        let (value, cause, _, _, _, panicked, _) = drive(
            Instant::now(),
            Instant::now() + Duration::from_millis(20),
            &flag,
            1_u8,
            false,
            |_| Ok::<_, ()>((1, false)),
            || {
                std::thread::sleep(Duration::from_millis(120));
                9
            },
        );
        assert_eq!((value, cause, panicked), (Some(9), Cause::Deadline, false));
        assert!(!flag.load(Ordering::Acquire));

        let flag = AtomicBool::new(false);
        let (value, cause, _, _, _, panicked, _) = drive(
            Instant::now(),
            Instant::now() + Duration::from_secs(1),
            &flag,
            1_u8,
            false,
            |_| Ok::<_, ()>((1, false)),
            || -> u8 { panic!("owned worker fault") },
        );
        assert_eq!(cause, Cause::WorkerPanic);
        assert!(value.is_none() && panicked && !flag.load(Ordering::Acquire));
    }

    #[test]
    fn sole_parent_commits_fresh_intent_and_stale_generation_does_not_cancel() {
        let (path, mut store, principal, task) = admitted_store();
        let initial = store
            .get(&principal, task, Instant::now() + Duration::from_secs(1))
            .unwrap();
        let flag = AtomicBool::new(false);
        let (handle, queue) = cancellation_queue(TaskId::parse(task.as_str()).unwrap());
        let mut commands = Vec::new();
        let (value, cause, head, _, error, panicked, _) = drive(
            Instant::now(),
            Instant::now() + Duration::from_secs(2),
            &flag,
            initial,
            false,
            |deadline| {
                if let Some(observation) =
                    service_command(&queue, &mut store, task, deadline, Instant::now())
                {
                    commands.push(observation);
                }
                store
                    .get(&principal, task, deadline)
                    .map(|head| (head.clone(), head.cancellation))
            },
            || {
                std::thread::sleep(Duration::from_millis(20));
                handle
                    .try_cancel(
                        "1".parse().unwrap(),
                        EventId::parse("66666666-6666-4666-8666-666666666666").unwrap(),
                    )
                    .unwrap();
                while !flag.load(Ordering::Acquire) {
                    std::thread::yield_now();
                }
                10
            },
        );
        assert_eq!(value, Some(10));
        assert_eq!(cause, Cause::DurableCancellation);
        assert!(head.cancellation && error.is_none() && !panicked);
        assert_eq!(commands.len(), 1);
        assert!(commands[0].result.is_ok());
        drop(store);
        fs::remove_dir_all(path).unwrap();

        let (path, mut store, principal, task) = admitted_store();
        let initial = store
            .get(&principal, task, Instant::now() + Duration::from_secs(1))
            .unwrap();
        let flag = AtomicBool::new(false);
        let (handle, queue) = cancellation_queue(TaskId::parse(task.as_str()).unwrap());
        let mut commands = Vec::new();
        let (_, cause, head, _, error, _, _) = drive(
            Instant::now(),
            Instant::now() + Duration::from_secs(2),
            &flag,
            initial,
            false,
            |deadline| {
                if let Some(observation) =
                    service_command(&queue, &mut store, task, deadline, Instant::now())
                {
                    commands.push(observation);
                }
                store
                    .get(&principal, task, deadline)
                    .map(|head| (head.clone(), head.cancellation))
            },
            || {
                handle
                    .try_cancel(
                        "2".parse().unwrap(),
                        EventId::parse("77777777-7777-4777-8777-777777777777").unwrap(),
                    )
                    .unwrap();
                std::thread::sleep(Duration::from_millis(150));
            },
        );
        assert_eq!(cause, Cause::None);
        assert!(!head.cancellation && error.is_none());
        assert!(!flag.load(Ordering::Acquire));
        assert_eq!(commands.len(), 1);
        assert!(commands[0].result.is_err());
        drop(store);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn capacity_and_disconnection_return_the_unqueued_command() {
        let (handle, queue) =
            cancellation_queue(TaskId::parse("33333333-3333-4333-8333-333333333333").unwrap());
        handle
            .try_cancel(
                "1".parse().unwrap(),
                EventId::parse("88888888-8888-4888-8888-888888888888").unwrap(),
            )
            .unwrap();
        let full = handle
            .try_cancel(
                "1".parse().unwrap(),
                EventId::parse("99999999-9999-4999-8999-999999999999").unwrap(),
            )
            .unwrap_err();
        assert_eq!(full.failure, EnqueueFailure::Full);
        let (pending, disconnected) = pending(&queue);
        assert_eq!(
            pending.unwrap().event.as_str(),
            "88888888-8888-4888-8888-888888888888"
        );
        assert!(!disconnected);
        drop(queue);
        let disconnected = handle
            .try_cancel(
                "1".parse().unwrap(),
                EventId::parse("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa").unwrap(),
            )
            .unwrap_err();
        assert_eq!(disconnected.failure, EnqueueFailure::Disconnected);
    }

    #[test]
    fn store_poll_deadline_is_capped_by_interval_and_task_deadline() {
        let before = Instant::now();
        let task_deadline = before + Duration::from_secs(10);
        let bounded = poll_deadline(before, task_deadline);
        assert_eq!(bounded, before + POLL);

        let near = Instant::now() + Duration::from_millis(5);
        assert_eq!(poll_deadline(before, near), near);
        assert!(matches!(
            boundary_deadline(near, near),
            Err(Error::InvalidDeadline)
        ));
        assert!(matches!(
            boundary_deadline(near + POLL, near),
            Err(Error::InvalidDeadline)
        ));
        assert_eq!(boundary_deadline(before, near).unwrap(), near);
    }
    #[test]
    fn boundary_authorizes_before_consuming_retained_or_queued_commands() {
        let (path, mut store, principal, task) = admitted_store();
        let origin = Instant::now();
        let deadline = origin + Duration::from_secs(5);
        let (handle, queue) = cancellation_queue(TaskId::parse(task.as_str()).unwrap());
        let mut retained = Some(PendingCancel {
            task: TaskId::parse(task.as_str()).unwrap(),
            expected: "2".parse().unwrap(),
            event: EventId::parse("66666666-6666-4666-8666-666666666666").unwrap(),
        });
        handle
            .try_cancel(
                "1".parse().unwrap(),
                EventId::parse("77777777-7777-4777-8777-777777777777").unwrap(),
            )
            .unwrap();
        let mut foreign = Some(PendingCancel {
            task: TaskId::parse("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa").unwrap(),
            expected: "1".parse().unwrap(),
            event: EventId::parse("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb").unwrap(),
        });
        assert!(matches!(
            queue.observe(&mut store, &principal, &mut foreign, origin, deadline),
            Err(Error::TaskMismatch)
        ));
        assert!(foreign.is_some());
        assert!(!store.get(&principal, task, deadline).unwrap().cancellation);
        let other = Principal::new(2000, "operator").unwrap();
        assert!(matches!(
            queue.observe(&mut store, &other, &mut retained, origin, deadline),
            Err(Error::InitialRead { command: None, .. })
        ));
        assert!(retained.is_some());
        assert!(matches!(
            queue.observe(&mut store, &principal, &mut retained, deadline, deadline),
            Err(Error::InvalidDeadline)
        ));
        assert!(retained.is_some());
        let first = queue
            .observe(&mut store, &principal, &mut retained, origin, deadline)
            .unwrap();
        assert!(retained.is_none());
        assert!(!first.head.cancellation);
        assert!(matches!(
            first.command.unwrap().result,
            Err(store::Error::Conflict)
        ));
        let second = queue
            .observe(&mut store, &principal, &mut retained, origin, deadline)
            .unwrap();
        assert!(second.head.cancellation);
        assert_eq!(second.head.generation, "2");
        assert!(second.command.unwrap().result.is_ok());
        assert!(second.observed_after >= first.observed_after);
        let third = queue
            .observe(&mut store, &principal, &mut retained, origin, deadline)
            .unwrap();
        assert!(third.head.cancellation && third.command.is_none());
        assert_eq!(queue.task().as_str(), task.as_str());
        drop(store);
        fs::remove_dir_all(path).unwrap();
    }
}
