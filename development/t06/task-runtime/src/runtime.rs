//! Fixed T06 application composition. Store and `task::driver` keep their existing authority.
use crate::clock::{self, TaskClock};
use habitat_engine::app::{
    durable_control::{self, CancellationQueue, PendingCancel, TaskId},
    evidence::{self, Evidence},
    repair, workload,
};
use habitat_engine::check::consistency::Prepared;
use habitat_engine::contracts::roster::Selection;
use habitat_engine::contracts::{
    Generation, Sha256Digest, UuidV4,
    receipt::{HostV1, Payload, ReceiptV1, Ref, TypedRef},
};
use habitat_engine::store::{self, Principal, RosterAttempt, RosterStart, Store, TaskHead};
use habitat_engine::worker::{
    aggregate,
    resources::Scope,
    workspace::{self, Snapshot},
};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{RecvTimeoutError, SyncSender, TrySendError, sync_channel};
use std::time::Duration;
use std::time::Instant;

// The concrete receipt adapter is developed independently and imported unchanged
// once its exact release is reviewed. No substitute verifier trait is introduced.
use crate::native_executor_observation as native;
use crate::u64_receipt;
#[path = "runtime_readback.rs"]
mod readback;

pub struct Recipe {
    pub prepared: Prepared,
    pub recipe: Payload,
    pub host: TypedRef<HostV1>,
    pub review_provenance: Payload,
    pub session: String,
    pub workspace: String,
    pub scopes: [Scope; 3],
}

pub struct Sources {
    pub baseline: Snapshot,
    pub repaired: Snapshot,
    pub protected: Snapshot,
    pub fixtures: Snapshot,
    pub oracle: Snapshot,
    pub harness: Snapshot,
    pub collector: Snapshot,
    pub launcher: Snapshot,
    pub reference_patch: Vec<u8>,
}

pub struct Config {
    pub sources: Sources,
    pub recipes: [Recipe; 2],
    pub tools: workload::Tools,
    pub root: PathBuf,
    pub agent_record_id: String,
    pub selections: Vec<Selection>,
    pub busctl: PathBuf,
    pub busctl_sha256: String,
    pub runtime_dir: PathBuf,
    pub executor: PathBuf,
    pub executor_sha256: String,
}

#[derive(Debug)]
pub enum Error {
    Identity,
    State,
    Clock(clock::Error),
    Store(store::Error),
    Workspace(workspace::Error),
    Materialize(workspace::MaterializeError),
    Repair(repair::Failure),
    Aggregate(aggregate::Error),
    Control(durable_control::Error),
    Receipt(u64_receipt::Error),
    Import(habitat_engine::app::receipt_import::Error),
    Publication,
    Io(std::io::Error),
    Native(native::Error),
    AttemptPointFull,
    AttemptPointDisconnected,
    VerificationPanicked,
    VerificationCollection {
        work: Option<Box<Error>>,
        observation: Option<durable_control::Error>,
        custody: Option<Box<Error>>,
    },
    VerificationObservation {
        observation: durable_control::Error,
        work: Option<Box<Error>>,
    },
    PreparationPanicked,
    PreparationObservation {
        observation: durable_control::Error,
        work: Option<Box<Error>>,
    },
}
impl From<store::Error> for Error {
    fn from(error: store::Error) -> Self {
        Self::Store(error)
    }
}
impl From<clock::Error> for Error {
    fn from(error: clock::Error) -> Self {
        Self::Clock(error)
    }
}

/// A driver ticket is neither a durable generation nor an acceptance capability.
pub struct Attempt {
    index: usize,
    id: String,
}

struct Execution {
    roster: RosterAttempt,
    executor_observation: native::FreshObservation,
    began: Instant,
    source: Option<Snapshot>,
    frozen: Option<u64_receipt::Frozen>,
    job: PathBuf,
    aggregate: Option<aggregate::Aggregate>,
    aggregate_start: Option<aggregate::Live>,
    aggregate_stop: Option<aggregate::Stopped>,
    aggregate_cleanup_error: Option<aggregate::Error>,
    control: Option<durable_control::Report>,
    undispatched_at: Option<Instant>,
    work_settled: bool,
    verification_started: bool,
    verification_phases: Vec<Phase>,
    receipt: Option<u64_receipt::Collected>,
    collected_evidence: Option<Imported>,
    imported: Option<Imported>,
    verification_recorded: bool,
}

struct Phase {
    name: &'static str,
    began: Instant,
    ended: Instant,
}

struct Imported {
    registered: BTreeMap<String, (Ref, store::Object)>,
    pending: Vec<(Ref, store::Object)>,
    publication_error: Option<String>,
}

// Keep the adapter outside the unwind boundary: completed or pending publications
// remain observable even if an internal verifier defect panics. A panic still
// marks custody unknown; catching it is not settlement or permission to accept.
fn retain_import<T>(
    destination: &mut Evidence<'_>,
    work: impl FnOnce(&mut Evidence<'_>) -> Result<T, Error>,
) -> (Result<T, Error>, Imported) {
    let returned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| work(destination)));
    let mut publication_error = destination
        .last_publication_error()
        .map(|error| format!("{error:?}"));
    let result = if let Ok(result) = returned {
        result
    } else {
        publication_error = Some(format!(
            "verifier worker panicked; publication custody unknown; prior error: {publication_error:?}"
        ));
        Err(Error::VerificationPanicked)
    };
    let imported = Imported {
        registered: destination.registered().clone(),
        pending: destination.pending_publications().to_vec(),
        publication_error,
    };
    (result, imported)
}

pub struct AcceptanceProof {
    root: TypedRef<ReceiptV1>,
    subject: String,
    object: store::Object,
    objects: Vec<store::Object>,
}

/// Observation only: no Store handle or authority to change the task generation.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct AttemptCancellationPoint {
    pub task_id: String,
    pub attempt_id: String,
    pub expected_generation: String,
    pub task_elapsed_ns: String,
    #[serde(skip)]
    pub task_origin: Instant,
    pub run_id: String,
    pub aggregate: String,
    pub scope_units: [String; 3],
}

pub struct TaskRuntime<'a> {
    store: &'a mut Store,
    principal: &'a Principal,
    task: TaskId,
    queue: &'a CancellationQueue,
    evidence: Evidence<'a>,
    config: Config,
    clock: TaskClock,
    head: TaskHead,
    pending_cancel: Option<PendingCancel>,
    boundary_observations: Vec<durable_control::Boundary>,
    executions: Vec<Execution>,
    pending_executor_observation: Option<native::FreshObservation>,
    attempt_points: Option<SyncSender<AttemptCancellationPoint>>,
}

fn uuid(value: &str) -> Result<UuidV4<'_>, Error> {
    UuidV4::parse(value).map_err(|_| Error::Identity)
}
fn generation(value: &str) -> Result<Generation, Error> {
    value.parse().map_err(|_| Error::Identity)
}
fn event(deadline: Instant) -> Result<String, Error> {
    evidence::fresh_id(deadline)
        .map(|id| id.as_str().to_owned())
        .map_err(|_| Error::Identity)
}

impl<'a> TaskRuntime<'a> {
    /// Take the sole Store owner after admission and independently reviewed preparation.
    /// The external `ArtifactStaging` owner backs `evidence`; it opens no second ledger.
    /// # Errors
    /// Refuses identity/clock/budget drift, stale admission, missing custody or failed Store readback.
    pub fn new(
        store: &'a mut Store,
        principal: &'a Principal,
        task: TaskId,
        queue: &'a CancellationQueue,
        evidence: Evidence<'a>,
        config: Config,
        clock: TaskClock,
    ) -> Result<Self, Error> {
        clock.dispatch()?;
        if queue.task() != &task {
            return Err(Error::Identity);
        }
        if config.selections.len() != 1 || config.selections[0].record_id != config.agent_record_id
        {
            return Err(Error::Identity);
        }
        let head = store.get(principal, uuid(task.as_str())?, clock.deadline)?;
        if head.state != "admitted" || head.cancellation || head.accepted_event.is_some() {
            return Err(Error::State);
        }
        if head.spent_ms != 0
            || head.reserved_work_ms != 900_000
            || head.reserved_verify_ms != 300_000
        {
            return Err(Error::Clock(clock::Error::Budget));
        }
        let root =
            Snapshot::capture(&config.root, &[], clock.deadline).map_err(Error::Workspace)?;
        if root.entries().next().is_some() {
            return Err(Error::State);
        }
        for (index, recipe) in config.recipes.iter().enumerate() {
            let identity = &recipe.prepared.identity;
            if identity.task_id.as_str() != task.as_str()
                || identity.generation.as_str() != (index + 1).to_string()
                || recipe.prepared.cases.len() != 1
                || identity.criterion_ids.as_slice().len() != 1
            {
                return Err(Error::Identity);
            }
            uuid(&recipe.session)?;
            uuid(&recipe.workspace)?;
            let aggregate_name = format!(
                "hee3aggregate{}.slice",
                identity.run_id.as_str().replace('-', "")
            );
            if recipe
                .scopes
                .iter()
                .any(|scope| scope.aggregate != aggregate_name)
            {
                return Err(Error::Identity);
            }
        }
        if config.recipes[0].prepared.identity.attempt_id
            == config.recipes[1].prepared.identity.attempt_id
            || config.recipes[0].prepared.identity.run_id
                == config.recipes[1].prepared.identity.run_id
        {
            return Err(Error::Identity);
        }
        Ok(Self {
            store,
            principal,
            task,
            queue,
            evidence,
            config,
            clock,
            head,
            pending_cancel: None,
            boundary_observations: Vec::new(),
            executions: Vec::new(),
            pending_executor_observation: None,
            attempt_points: None,
        })
    }

    /// Install one bounded observation channel before the first attempt.
    /// # Errors
    /// Refuses replacement or installation after attempt ownership has begun.
    pub fn set_attempt_points(
        &mut self,
        sender: SyncSender<AttemptCancellationPoint>,
    ) -> Result<(), Error> {
        if self.attempt_points.is_some() || !self.executions.is_empty() {
            return Err(Error::State);
        }
        self.attempt_points = Some(sender);
        Ok(())
    }

    fn publish_attempt_point(&self, ticket: &Attempt) -> Result<(), Error> {
        self.current(ticket)?;
        let Some(sender) = &self.attempt_points else {
            return Ok(());
        };
        let recipe = &self.config.recipes[ticket.index];
        let point = AttemptCancellationPoint {
            task_id: self.task.as_str().to_owned(),
            attempt_id: ticket.id.clone(),
            expected_generation: self.head.generation.clone(),
            task_elapsed_ns: self.clock.elapsed().as_nanos().to_string(),
            task_origin: self.clock.origin,
            run_id: recipe.prepared.identity.run_id.as_str().to_owned(),
            aggregate: recipe.scopes[0].aggregate.clone(),
            scope_units: [
                recipe.scopes[0].unit().map_err(|_| Error::Identity)?,
                recipe.scopes[1].unit().map_err(|_| Error::Identity)?,
                recipe.scopes[2].unit().map_err(|_| Error::Identity)?,
            ],
        };
        sender.try_send(point).map_err(|error| match error {
            TrySendError::Full(_) => Error::AttemptPointFull,
            TrySendError::Disconnected(_) => Error::AttemptPointDisconnected,
        })
    }

    fn refresh(&mut self) -> Result<bool, Error> {
        let observation = self
            .queue
            .observe(
                self.store,
                self.principal,
                &mut self.pending_cancel,
                self.clock.origin,
                self.clock.deadline,
            )
            .map_err(Error::Control)?;
        self.head = observation.head.clone();
        let cancelled = self.head.cancellation;
        self.boundary_observations.push(observation);
        Ok(cancelled)
    }

    fn current(&self, ticket: &Attempt) -> Result<&Execution, Error> {
        let execution = self.executions.get(ticket.index).ok_or(Error::Identity)?;
        if ticket.index + 1 != self.executions.len() || execution.roster.attempt.id != ticket.id {
            return Err(Error::Identity);
        }
        Ok(execution)
    }

    fn begin(&mut self, ordinal: Generation) -> Result<Attempt, Error> {
        self.clock.dispatch()?;
        if self.refresh()? {
            return Err(Error::Store(store::Error::Cancelled));
        }
        let index = self.executions.len();
        if index >= 2 || ordinal.to_string() != (index + 1).to_string() {
            return Err(Error::Identity);
        }
        if self.pending_executor_observation.is_some() {
            return Err(Error::State);
        }
        // The first work settlement includes admission and recipe preparation.
        // Later attempts start after the previous verification settlement.
        let began = if index == 0 {
            self.clock.origin
        } else {
            Instant::now()
        };
        let observation = native::observe(
            self.store,
            self.principal,
            &mut self.evidence,
            &self.config.selections[0],
            &self.config.executor,
            &self.config.executor_sha256,
            self.clock.deadline,
        )
        .map_err(Error::Native)?;
        self.pending_executor_observation = Some(observation);
        // A fresh owner observation does not replace the task cancellation boundary.
        if self.refresh()? {
            return Err(Error::Store(store::Error::Cancelled));
        }
        self.clock.dispatch()?;
        let recipe = &self.config.recipes[index];
        let start_event = event(self.clock.deadline)?;
        let roster = self.store.begin_rostered_attempt(
            RosterStart {
                principal: self.principal,
                task: uuid(self.task.as_str())?,
                expected: generation(&self.head.generation)?,
                attempt: uuid(recipe.prepared.identity.attempt_id.as_str())?,
                event: uuid(&start_event)?,
                agent_record_id: &self.config.agent_record_id,
                session: uuid(&recipe.session)?,
                workspace: uuid(&recipe.workspace)?,
                selections: &self.config.selections,
                lease_ms: clock::lease_ms(Instant::now(), self.clock.deadline)?,
            },
            self.clock.deadline,
        )?;
        let ticket = Attempt {
            index,
            id: roster.attempt.id.clone(),
        };
        let root = self.config.root.join(&ticket.id);
        let job = root.join("job");
        // Retain durable identity before the first private filesystem effect.
        self.executions.push(Execution {
            roster,
            executor_observation: self
                .pending_executor_observation
                .take()
                .ok_or(Error::State)?,
            began,
            source: None,
            frozen: None,
            job: job.clone(),
            aggregate: None,
            aggregate_start: None,
            aggregate_stop: None,
            aggregate_cleanup_error: None,
            control: None,
            undispatched_at: None,
            work_settled: false,
            verification_started: false,
            verification_phases: Vec::new(),
            receipt: None,
            collected_evidence: None,
            imported: None,
            verification_recorded: false,
        });
        self.head = self.store.get(
            self.principal,
            uuid(self.task.as_str())?,
            self.clock.deadline,
        )?;
        if self.executions[index].roster.attempt.generation != ordinal.to_string() {
            return Err(Error::Identity);
        }
        for path in [&root, &job] {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(path)
                .map_err(Error::Io)?;
        }
        let source = self.materialize(&root, index)?;
        self.executions[index].source = Some(source);
        if self.refresh()? {
            return Ok(ticket);
        }
        self.prepare(index)?;
        Ok(ticket)
    }

    // Only immutable snapshots cross this scoped boundary; Store stays on its sole owner.
    // Cancellation can become durable while the existing bounded copy finishes. No
    // candidate/aggregate is dispatched here and the scoped worker is always joined.
    fn materialize(&mut self, root: &std::path::Path, index: usize) -> Result<Snapshot, Error> {
        let sources = &self.config.sources;
        let deadline = self.clock.deadline;
        let (sender, receiver) = sync_channel(1);
        std::thread::scope(|scope| {
            let worker = scope.spawn(move || {
                let result = materialize_source(sources, root, index, deadline);
                let _ = sender.send(result);
            });
            let mut observation_error = None;
            let result = loop {
                if observation_error.is_none() && Instant::now() < deadline {
                    match self.queue.observe(
                        self.store,
                        self.principal,
                        &mut self.pending_cancel,
                        self.clock.origin,
                        deadline,
                    ) {
                        Ok(boundary) => {
                            self.head = boundary.head.clone();
                            self.boundary_observations.push(boundary);
                        }
                        Err(error) => observation_error = Some(error),
                    }
                }
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(result) => break result,
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break Err(Error::PreparationPanicked),
                }
            };
            let result = if worker.join().is_err() {
                Err(Error::PreparationPanicked)
            } else {
                result
            };
            if let Some(observation) = observation_error {
                return Err(Error::PreparationObservation {
                    observation,
                    work: result.err().map(Box::new),
                });
            }
            result
        })
    }

    fn prepare(&mut self, index: usize) -> Result<(), Error> {
        use habitat_engine::contracts::receipt::SubjectFileV1Origin::Authored;
        use u64_receipt::{Patch, Preparation, SourceBindings, SubjectBinding};
        let sources = &self.config.sources;
        let recipe = &self.config.recipes[index];
        let execution = &mut self.executions[index];
        let result = execution.source.as_ref().ok_or(Error::State)?;
        let binding = |snapshot| SubjectBinding {
            snapshot,
            origin: Authored,
        };
        let staging = self.evidence.staging_owner().ok_or(Error::State)?;
        let registered = self.evidence.registered().clone();
        let deadline = self.clock.deadline;
        let tools = &self.config.tools;
        let (sender, receiver) = sync_channel(1);
        let frozen = std::thread::scope(|scope| {
            let worker = scope.spawn(move || {
                let result = (|| {
                    let mut worker_evidence = Evidence::staged(staging, deadline);
                    for (_, (reference, object)) in registered {
                        worker_evidence
                            .register(reference, object)
                            .map_err(|_| Error::Publication)?;
                    }
                    let frozen = u64_receipt::prepare(
                        &worker_evidence,
                        Preparation {
                            prepared: &recipe.prepared,
                            sources: SourceBindings {
                                seed: binding(&sources.baseline),
                                result: binding(result),
                                fixtures: binding(&sources.fixtures),
                                oracle: binding(&sources.oracle),
                                harness: binding(&sources.harness),
                                collector: binding(&sources.collector),
                                launcher: binding(&sources.launcher),
                            },
                            protected: &sources.protected,
                            tools,
                            patch: if index == 0 {
                                Patch::Baseline
                            } else {
                                Patch::Repair(&sources.reference_patch)
                            },
                            recipe: recipe.recipe.clone(),
                            host: recipe.host.clone(),
                            review_provenance: recipe.review_provenance.clone(),
                            deadline,
                        },
                    )
                    .map_err(Error::Receipt)?;
                    Ok(frozen)
                })();
                let _ = sender.send(result);
            });
            let mut observation_error = None;
            let result = loop {
                if observation_error.is_none() && Instant::now() < deadline {
                    match self.queue.observe(
                        self.store,
                        self.principal,
                        &mut self.pending_cancel,
                        self.clock.origin,
                        deadline,
                    ) {
                        Ok(boundary) => {
                            self.head = boundary.head.clone();
                            self.boundary_observations.push(boundary);
                        }
                        Err(error) => observation_error = Some(error),
                    }
                }
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(result) => break result,
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break Err(Error::PreparationPanicked),
                }
            };
            let result = if worker.join().is_err() {
                Err(Error::PreparationPanicked)
            } else {
                result
            };
            if let Some(observation) = observation_error {
                return Err(Error::PreparationObservation {
                    observation,
                    work: result.err().map(Box::new),
                });
            }
            result
        })?;
        execution.frozen = Some(frozen);
        Ok(())
    }

    fn execute(&mut self, ticket: &Attempt) -> Result<habitat_engine::task::driver::Work, Error> {
        let execution = self.current(ticket)?;
        if execution.control.is_some() || execution.work_settled {
            return Err(Error::State);
        }
        if self.refresh()? {
            self.settle_undispatched(ticket.index)?;
            // The driver rechecks durable cancellation before calling the verifier.
            return Ok(habitat_engine::task::driver::Work::ReadyForCheck);
        }
        if self.current(ticket)?.frozen.is_none() {
            return Err(Error::State);
        }
        self.clock.dispatch()?;
        // Preparation was part of this work phase. Do not give that elapsed time back.
        let already_used = clock::cost_ms(self.executions[ticket.index].began, Instant::now())?;
        let remaining = self
            .head
            .reserved_work_ms
            .checked_sub(already_used)
            .ok_or(Error::Clock(clock::Error::Budget))?;
        let deadline = self.clock.work_deadline(remaining)?;
        self.publish_attempt_point(ticket)?;
        let recipe = &self.config.recipes[ticket.index];
        let execution = &mut self.executions[ticket.index];
        let aggregate = aggregate::Aggregate::prepare(
            aggregate::Config {
                busctl: self.config.busctl.clone(),
                busctl_sha256: self.config.busctl_sha256.clone(),
                runtime_dir: self.config.runtime_dir.clone(),
                run_id: recipe.prepared.identity.run_id.as_str().to_owned(),
            },
            deadline,
        )
        .map_err(Error::Aggregate)?;
        execution.aggregate = Some(aggregate);
        let cancelled = AtomicBool::new(false);
        let owner = execution.aggregate.as_mut().ok_or(Error::State)?;
        // Failed manager calls and their owned phase remain in execution on error.
        match owner.start(deadline, &cancelled) {
            Ok(started) => execution.aggregate_start = Some(started),
            Err(error) => {
                // Retain the failed start and attempt only the existing owner's
                // bounded restore/empty/stop sequence. An unsupported phase stays
                // unresolved, with both original and cleanup errors observable.
                cleanup_aggregate(execution, self.clock.deadline);
                return Err(Error::Aggregate(error));
            }
        }
        let source = execution.source.as_ref().ok_or(Error::State)?;
        let result = durable_control::collect(
            self.store,
            self.principal,
            uuid(self.task.as_str())?,
            &workload::Plan {
                source,
                protected: &self.config.sources.protected,
                job_root: &execution.job,
                tools: &self.config.tools,
                deadline,
                cancelled: &cancelled,
            },
            &recipe.scopes,
            self.queue,
        );
        // Aggregate cleanup uses the task's retained reserve, never a fresh horizon.
        cleanup_aggregate(execution, self.clock.deadline);
        let mut control = result.map_err(Error::Control)?;
        self.head = control.last_head.clone();
        self.pending_cancel = control.pending_cancel.take();
        execution.control = Some(control);
        self.settle_work(ticket.index)
    }

    fn settle_work(&mut self, index: usize) -> Result<habitat_engine::task::driver::Work, Error> {
        use habitat_engine::task::driver::Work;
        let execution = &mut self.executions[index];
        let control = execution.control.as_ref().ok_or(Error::State)?;
        let run = control.run.as_ref().and_then(|value| value.as_ref().ok());
        let undispatched = control.run.is_none()
            && !control.worker_panicked
            && control.cause == durable_control::Cause::DurableCancellation
            && control.first_observed_after == Some(std::time::Duration::ZERO);
        if undispatched {
            execution.undispatched_at = Some(control.started_at);
        }
        let clean = execution.aggregate_stop.is_some()
            && (undispatched
                || run
                    .is_some_and(|value| value.process_cleanup_complete && value.scratch_released))
            && !control.worker_panicked;
        let observed = run.is_some_and(|value| {
            !value.steps.is_empty()
                && value
                    .steps
                    .iter()
                    .all(|step| matches!(step, workload::Step::Completed { .. }))
        });
        let cost = clock::cost_ms(execution.began, Instant::now())?;
        // Unknown/over-reservation usage cannot be charged as a smaller known value.
        let used_ms = (cost <= self.head.reserved_work_ms).then_some(cost);
        let settled = clean && used_ms.is_some();
        let observed_event = event(self.clock.deadline)?;
        let task_generation = self.store.settle_attempt(
            &store::Expected {
                task: uuid(self.task.as_str())?,
                task_generation: generation(&self.head.generation)?,
                attempt: uuid(&execution.roster.attempt.id)?,
                attempt_generation: generation(&execution.roster.attempt.generation)?,
            },
            store::Settlement {
                effect: if clean {
                    store::Effect::None
                } else {
                    store::Effect::Unknown
                },
                used_ms,
                cleanup_settled: clean,
                ready_to_verify: settled && (observed || undispatched),
            },
            uuid(&observed_event)?,
            self.clock.deadline,
        )?;
        self.head.generation = task_generation;
        execution.work_settled = settled;
        self.head = self.store.get(
            self.principal,
            uuid(self.task.as_str())?,
            self.clock.deadline,
        )?;
        if settled && (observed || undispatched) {
            Ok(Work::ReadyForCheck)
        } else if settled {
            Ok(Work::Failed)
        } else {
            Ok(Work::Unsettled)
        }
    }

    fn settle_undispatched(&mut self, index: usize) -> Result<(), Error> {
        let execution = &mut self.executions[index];
        if execution.aggregate.is_some()
            || execution.control.is_some()
            || execution.work_settled
            || !self.head.cancellation
        {
            return Err(Error::State);
        }
        let observed = Instant::now();
        execution.undispatched_at = Some(observed);
        let cost = clock::cost_ms(execution.began, observed)?;
        let used_ms = (cost <= self.head.reserved_work_ms).then_some(cost);
        let observed_event = event(self.clock.deadline)?;
        self.head.generation = self.store.settle_attempt(
            &store::Expected {
                task: uuid(self.task.as_str())?,
                task_generation: generation(&self.head.generation)?,
                attempt: uuid(&execution.roster.attempt.id)?,
                attempt_generation: generation(&execution.roster.attempt.generation)?,
            },
            store::Settlement {
                effect: store::Effect::None,
                used_ms,
                cleanup_settled: true,
                ready_to_verify: used_ms.is_some(),
            },
            uuid(&observed_event)?,
            self.clock.deadline,
        )?;
        execution.work_settled = used_ms.is_some();
        Ok(())
    }

    fn verify(
        &mut self,
        ticket: &Attempt,
    ) -> Result<habitat_engine::task::driver::Checked<AcceptanceProof>, Error> {
        use habitat_engine::contracts::receipt::VerdictV1State;
        use habitat_engine::task::driver::Checked;
        let execution = self.current(ticket)?;
        if !execution.work_settled || execution.receipt.is_some() {
            return Err(Error::State);
        }
        if self.refresh()? {
            return Ok(Checked::Unsettled);
        }
        let began = Instant::now();
        if began >= self.clock.verification_deadline {
            return Err(Error::Clock(clock::Error::Deadline));
        }
        let (root, state) = self.collect_current_receipt(ticket.index)?;
        // The receipt's completed observation remains historical if cancellation
        // arrives now. Make intent durable before importing its retained proof.
        self.refresh()?;
        self.import_current_receipt(ticket.index, &root)?;
        // No immutable Store borrow spans this final queue/CAS checkpoint.
        self.refresh()?;
        let execution = &mut self.executions[ticket.index];
        let frozen = execution.frozen.as_ref().ok_or(Error::State)?;
        let imported = execution.imported.as_ref().ok_or(Error::State)?;
        if !imported.pending.is_empty() || imported.publication_error.is_some() {
            return Err(Error::Publication);
        }
        let object = imported
            .registered
            .get(root.as_ref().artifact_id.as_str())
            .ok_or(Error::Identity)?
            .1
            .clone();
        let subject = frozen
            .prepared()
            .subjects
            .result_subject
            .value
            .as_ref()
            .ok_or(Error::Identity)?
            .as_ref()
            .sha256
            .as_str()
            .to_owned();
        let used = clock::cost_ms(began, Instant::now())?;
        let used_ms = (used <= self.head.reserved_verify_ms).then_some(used);
        let verdict = store_verdict(state);
        let observation_event = event(self.clock.deadline)?;
        self.head.generation = self.store.record_verification(
            &store::Expected {
                task: uuid(self.task.as_str())?,
                task_generation: generation(&self.head.generation)?,
                attempt: uuid(&execution.roster.attempt.id)?,
                attempt_generation: generation(&execution.roster.attempt.generation)?,
            },
            &store::Verification {
                verdict,
                subject: Sha256Digest::parse(&subject).map_err(|_| Error::Identity)?,
                evidence: object.clone(),
                used_ms,
                cleanup_settled: true,
            },
            uuid(&observation_event)?,
            self.clock.deadline,
        )?;
        execution.verification_recorded = true;
        if used_ms.is_none() {
            return Ok(Checked::Unsettled);
        }
        Ok(match state {
            VerdictV1State::PassCandidate => Checked::Passed {
                evidence: AcceptanceProof {
                    root,
                    subject,
                    object,
                    objects: unique_objects(&imported.registered)?,
                },
                criteria: 1,
            },
            VerdictV1State::Fail => Checked::Failed { criteria: 0 },
            VerdictV1State::Invalid | VerdictV1State::Unmeasured => Checked::Invalid,
            VerdictV1State::Error => Checked::Error,
            VerdictV1State::Timeout => Checked::Timeout,
            VerdictV1State::Cancelled => Checked::Unsettled,
        })
    }

    fn collect_current_receipt(
        &mut self,
        index: usize,
    ) -> Result<
        (
            TypedRef<ReceiptV1>,
            habitat_engine::contracts::receipt::VerdictV1State,
        ),
        Error,
    > {
        self.executions[index].verification_started = true;
        let (result, observation) = self.collection_worker(index, collect_receipt);
        // Retain the exact historical receipt before any later observation/rebind
        // error can refuse progress. Cancellation does not rewrite its verdict.
        let work = match result {
            Ok(collected) => {
                self.executions[index].receipt = Some(collected);
                None
            }
            Err(error) => Some(Box::new(error)),
        };
        let custody = self.rebind_collection(index).err().map(Box::new);
        if work.is_some() || observation.is_some() || custody.is_some() {
            return Err(Error::VerificationCollection {
                work,
                observation,
                custody,
            });
        }
        let collected = self.executions[index]
            .receipt
            .as_ref()
            .ok_or(Error::State)?;
        Ok((
            collected.receipt.reference.clone(),
            collected.decision.state,
        ))
    }

    fn collection_worker<T: Send>(
        &mut self,
        index: usize,
        work: impl FnOnce(&mut Evidence<'_>, &Execution, TaskClock) -> Result<T, Error> + Send,
    ) -> (Result<T, Error>, Option<durable_control::Error>) {
        if !self.evidence.pending_publications().is_empty()
            || self.evidence.last_publication_error().is_some()
        {
            return (Err(Error::Publication), None);
        }
        let Some(staging) = self.evidence.staging_owner() else {
            return (Err(Error::State), None);
        };
        let registry = self.evidence.registered().clone();
        let execution = &mut self.executions[index];
        let clock = self.clock;
        let began = Instant::now();
        let (result, observation_error, retained) = std::thread::scope(|scope| {
            let (sender, receiver) = sync_channel(1);
            let source = &*execution;
            let worker = scope.spawn(move || {
                // Store never crosses this boundary. Keep Evidence outside the
                // unwind guard so every known publication survives a collector panic.
                let mut evidence = Evidence::staged(staging, clock.deadline);
                let returned = retain_import(&mut evidence, |evidence| {
                    for (_, (reference, object)) in registry {
                        evidence
                            .register(reference, object)
                            .map_err(|_| Error::Publication)?;
                    }
                    work(evidence, source, clock)
                });
                let _ = sender.send(returned);
            });
            let mut observation_error = None;
            let returned = loop {
                if observation_error.is_none() && Instant::now() < clock.deadline {
                    match self.queue.observe(
                        self.store,
                        self.principal,
                        &mut self.pending_cancel,
                        clock.origin,
                        clock.deadline,
                    ) {
                        Ok(boundary) => {
                            self.head = boundary.head.clone();
                            self.boundary_observations.push(boundary);
                        }
                        Err(error) => observation_error = Some(error),
                    }
                }
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(returned) => break Some(returned),
                    Err(RecvTimeoutError::Timeout) => (),
                    Err(RecvTimeoutError::Disconnected) => break None,
                }
            };
            let joined = worker.join();
            let (result, retained) = returned.unwrap_or_else(|| {
                (
                    Err(Error::VerificationPanicked),
                    Imported {
                        registered: BTreeMap::new(),
                        pending: Vec::new(),
                        publication_error: Some(
                            "collection worker disconnected; returned custody unknown".into(),
                        ),
                    },
                )
            });
            (
                if joined.is_err() {
                    Err(Error::VerificationPanicked)
                } else {
                    result
                },
                observation_error,
                retained,
            )
        });
        execution.collected_evidence = Some(retained);
        execution.verification_phases.push(Phase {
            name: "observe_and_finalize",
            began,
            ended: Instant::now(),
        });
        (result, observation_error)
    }

    fn rebind_collection(&mut self, index: usize) -> Result<(), Error> {
        let retained = self.executions[index]
            .collected_evidence
            .as_ref()
            .ok_or(Error::State)?;
        let registry = retained.registered.clone();
        for (id, (reference, object)) in registry {
            // This loop performs bounded immutable readback; service intent for
            // each item instead of blocking through the complete 4096-object map.
            self.refresh()?;
            if let Some(existing) = self.evidence.registered().get(&id) {
                if existing != &(reference, object) {
                    return Err(Error::Identity);
                }
            } else {
                self.evidence
                    .register(reference, object)
                    .map_err(|_| Error::Publication)?;
            }
        }
        let retained = self.executions[index]
            .collected_evidence
            .as_ref()
            .ok_or(Error::State)?;
        if !retained.pending.is_empty() || retained.publication_error.is_some() {
            return Err(Error::Publication);
        }
        Ok(())
    }

    fn import_current_receipt(
        &mut self,
        index: usize,
        root: &TypedRef<ReceiptV1>,
    ) -> Result<(), Error> {
        let prepared = self.executions[index]
            .frozen
            .as_ref()
            .ok_or(Error::State)?
            .prepared()
            .clone();
        self.import_prepared_receipt(index, &prepared, root)
    }

    fn import_prepared_receipt(
        &mut self,
        index: usize,
        prepared: &Prepared,
        root: &TypedRef<ReceiptV1>,
    ) -> Result<(), Error> {
        let execution = &mut self.executions[index];
        let began = Instant::now();
        let source_staging = self.evidence.staging_owner().ok_or(Error::State)?;
        let source_registry = self.evidence.registered().clone();
        let deadline = self.clock.deadline;
        let origin = self.clock.origin;
        let queue = self.queue;
        let principal = self.principal;
        let pending = &mut self.pending_cancel;
        let head = &mut self.head;
        let boundaries = &mut self.boundary_observations;
        let result = self
            .store
            .with_artifact_staging(deadline, |store, destination_staging| {
                std::thread::scope(|scope| {
                    let (sender, receiver) = sync_channel(1);
                    let worker = scope.spawn(move || {
                        // Neither Evidence's Store variant nor the SQLite connection crosses
                        // this boundary. Publication uses the same immutable Store primitives.
                        let mut destination = Evidence::staged(destination_staging, deadline);
                        let (result, imported) = retain_import(&mut destination, |destination| {
                            let mut source = Evidence::staged(source_staging, deadline);
                            for (_, (reference, object)) in source_registry {
                                source
                                    .register(reference, object)
                                    .map_err(|_| Error::Publication)?;
                            }
                            habitat_engine::app::receipt_import::receipt(
                                destination,
                                &source,
                                prepared,
                                root,
                            )
                            .map(|_| ())
                            .map_err(Error::Import)
                        });
                        let _ = sender.send((result, imported));
                    });
                    let mut observation_error = None;
                    let returned = loop {
                        if observation_error.is_none() && Instant::now() < deadline {
                            match queue.observe(store, principal, pending, origin, deadline) {
                                Ok(boundary) => {
                                    *head = boundary.head.clone();
                                    boundaries.push(boundary);
                                }
                                Err(error) => observation_error = Some(error),
                            }
                        }
                        match receiver.recv_timeout(Duration::from_millis(100)) {
                            Ok(value) => break Some(value),
                            Err(RecvTimeoutError::Timeout) => (),
                            Err(RecvTimeoutError::Disconnected) => break None,
                        }
                    };
                    let joined = worker.join();
                    let result = match returned {
                        Some((result, imported)) => {
                            execution.imported = Some(imported);
                            result
                        }
                        None => Err(Error::VerificationPanicked),
                    };
                    let result = if joined.is_err() {
                        Err(Error::VerificationPanicked)
                    } else {
                        result
                    };
                    if let Some(observation) = observation_error {
                        Err(Error::VerificationObservation {
                            observation,
                            work: result.err().map(Box::new),
                        })
                    } else {
                        result
                    }
                })
            });
        execution.verification_phases.push(Phase {
            name: "import",
            began,
            ended: Instant::now(),
        });
        result.map_err(Error::Store)?
    }

    fn accept(&mut self, ticket: &Attempt, proof: &AcceptanceProof) -> Result<(), Error> {
        let execution = self.current(ticket)?;
        if !execution.verification_recorded
            || execution
                .receipt
                .as_ref()
                .is_none_or(|value| value.receipt.reference != proof.root)
        {
            return Err(Error::Identity);
        }
        if self.refresh()? {
            return Err(Error::Store(store::Error::Cancelled));
        }
        let execution = self.current(ticket)?;
        let acceptance_event = event(self.clock.deadline)?;
        let published = self.store.prepare_verified_acceptance(
            &store::Expected {
                task: uuid(self.task.as_str())?,
                task_generation: generation(&self.head.generation)?,
                attempt: uuid(&execution.roster.attempt.id)?,
                attempt_generation: generation(&execution.roster.attempt.generation)?,
            },
            uuid(&acceptance_event)?,
            Sha256Digest::parse(&proof.subject).map_err(|_| Error::Identity)?,
            &proof.object,
            &proof.objects,
            self.clock.deadline,
        )?;
        self.store.accept(&published, 0, self.clock.deadline)?;
        self.head = self.store.get(
            self.principal,
            uuid(self.task.as_str())?,
            self.clock.deadline,
        )?;
        if self.head.accepted_event.as_deref() != Some(&acceptance_event) {
            return Err(Error::State);
        }
        Ok(())
    }

    fn stop(&mut self, reason: habitat_engine::task::driver::StopReason) -> Result<bool, Error> {
        self.refresh()?;
        self.record_cancelled_idle_verifier()?;
        if self
            .executions
            .iter()
            .any(|execution| !execution.work_settled)
            || self.executions.last().is_some_and(|execution| {
                !execution.verification_recorded
                    && (execution.verification_started
                        || !(self.head.state == "failed"
                            || (reason == habitat_engine::task::driver::StopReason::WorkerFailed
                                && self.head.state == "repair_pending")))
            })
        {
            // No inferred verifier idleness or implicit release of pending resources.
            return Ok(false);
        }
        let name = if self.head.cancellation {
            "durable_cancellation"
        } else {
            stop_name(reason)
        };
        let reference = self
            .executions
            .last()
            .and_then(|execution| execution.receipt.as_ref())
            .map(|collected| &collected.receipt.reference);
        let elapsed_ms = clock::cost_ms(self.clock.origin, Instant::now())?;
        let bytes = serde_json::to_vec(&serde_json::json!({
            "kind":"fixed-u64-terminal-request/1", "task":self.task.as_str(),
            "generation":self.head.generation,"reason":name,"receipt":reference,
            "cancellation_observed":self.head.cancellation,
            "elapsed_ms":elapsed_ms,
            "completed_attempts":self.executions.len(),
        }))
        .map_err(|_| Error::Publication)?;
        let publication = event(self.clock.deadline)?;
        let object = self
            .store
            .publish(&bytes, uuid(&publication)?, self.clock.deadline)?;
        let stop_event = event(self.clock.deadline)?;
        let reason =
            habitat_engine::contracts::receipt::Name::new(name).map_err(|_| Error::Identity)?;
        let stop = store::Stop {
            task: uuid(self.task.as_str())?,
            generation: generation(&self.head.generation)?,
            reason: &reason,
            evidence: &object,
            event: uuid(&stop_event)?,
        };
        let stopped = if self.executions.is_empty() {
            // Runtime construction follows completed preparation. With no attempt,
            // no aggregate or worker effect was dispatched; charge preparation once.
            self.store.finish_preparation(
                self.principal,
                stop,
                store::Settlement {
                    used_ms: Some(elapsed_ms),
                    effect: store::Effect::None,
                    cleanup_settled: true,
                    ready_to_verify: false,
                },
                self.clock.deadline,
            )?
        } else {
            self.store
                .finish_unaccepted(self.principal, stop, self.clock.deadline)?
        };
        self.head.generation = stopped.generation;
        self.head = self.store.get(
            self.principal,
            uuid(self.task.as_str())?,
            self.clock.deadline,
        )?;
        Ok(true)
    }

    fn record_cancelled_idle_verifier(&mut self) -> Result<(), Error> {
        let Some(execution) = self.executions.last_mut() else {
            return Ok(());
        };
        if !self.head.cancellation
            || !execution.work_settled
            || execution.verification_started
            || execution.verification_recorded
        {
            return Ok(());
        }
        let (prepared, subject_binding) = if let Some(frozen) = &execution.frozen {
            (frozen.prepared(), "frozen_result")
        } else if execution.undispatched_at.is_some()
            && execution.aggregate.is_none()
            && execution.control.is_none()
            && execution.source.is_some()
        {
            // No verifier ran and no aggregate/worker was started. Retain the
            // planned result identity explicitly; this is never passing proof.
            let recipe = self
                .config
                .recipes
                .iter()
                .find(|recipe| {
                    recipe.prepared.identity.attempt_id.as_str() == execution.roster.attempt.id
                })
                .ok_or(Error::Identity)?;
            (&recipe.prepared, "planned_only_preparation_cancelled")
        } else {
            return Err(Error::State);
        };
        let subject = prepared
            .subjects
            .result_subject
            .value
            .as_ref()
            .ok_or(Error::Identity)?
            .as_ref()
            .sha256
            .as_str();
        let bytes = serde_json::to_vec(&serde_json::json!({
            "kind":"fixed-u64-verification-not-started/1",
            "task":self.task.as_str(), "attempt":execution.roster.attempt.id,
            "generation":self.head.generation, "subject":subject, "subject_binding":subject_binding,
            "observed_ms":clock::cost_ms(self.clock.origin, Instant::now())?,
            "durable_cancellation":true, "verifier_started":false,
            "work_settled":true, "verifier_cost_ms":0,
            "scope":"trusted owner records idleness; no oracle or producer result",
        }))
        .map_err(|_| Error::Publication)?;
        let publication = event(self.clock.deadline)?;
        let object = self
            .store
            .publish(&bytes, uuid(&publication)?, self.clock.deadline)?;
        let observation = event(self.clock.deadline)?;
        self.head.generation = self.store.record_verification(
            &store::Expected {
                task: uuid(self.task.as_str())?,
                task_generation: generation(&self.head.generation)?,
                attempt: uuid(&execution.roster.attempt.id)?,
                attempt_generation: generation(&execution.roster.attempt.generation)?,
            },
            &store::Verification {
                verdict: store::VerificationVerdict::Cancelled,
                subject: Sha256Digest::parse(subject).map_err(|_| Error::Identity)?,
                evidence: object,
                used_ms: Some(0),
                cleanup_settled: true,
            },
            uuid(&observation)?,
            self.clock.deadline,
        )?;
        execution.verification_recorded = true;
        Ok(())
    }
}

fn cleanup_aggregate(execution: &mut Execution, deadline: Instant) {
    let Some(owner) = execution.aggregate.as_mut() else {
        return;
    };
    match owner
        .restore_origin(deadline)
        .and_then(|()| owner.stop_if_empty(deadline))
    {
        Ok(stopped) => execution.aggregate_stop = Some(stopped),
        Err(error) => execution.aggregate_cleanup_error = Some(error),
    }
}

fn stop_name(reason: habitat_engine::task::driver::StopReason) -> &'static str {
    use habitat_engine::task::driver::StopReason;
    match reason {
        StopReason::Policy(_) => "task_policy_stop",
        StopReason::WorkerFailed => "worker_failed",
        StopReason::InvalidCheck => "invalid_check",
        StopReason::VerifierError => "verifier_error",
        StopReason::VerifierTimeout => "verifier_timeout",
        StopReason::Cancelled => "durable_cancellation",
        StopReason::Unsettled => "unsettled_obligation",
    }
}

impl habitat_engine::task::driver::Runtime for TaskRuntime<'_> {
    type Error = Error;
    type Attempt = Attempt;
    type Evidence = AcceptanceProof;
    fn elapsed(&self) -> std::time::Duration {
        self.clock.elapsed()
    }
    fn cancellation_requested(&mut self) -> Result<bool, Self::Error> {
        self.refresh()
    }
    fn begin(&mut self, ordinal: Generation) -> Result<Attempt, Self::Error> {
        self.begin(ordinal)
    }
    fn execute(
        &mut self,
        attempt: &Attempt,
    ) -> Result<habitat_engine::task::driver::Work, Self::Error> {
        self.execute(attempt)
    }
    fn verify(
        &mut self,
        attempt: &Attempt,
    ) -> Result<habitat_engine::task::driver::Checked<AcceptanceProof>, Self::Error> {
        self.verify(attempt)
    }
    fn accept(&mut self, attempt: &Attempt, evidence: AcceptanceProof) -> Result<(), Self::Error> {
        self.accept(attempt, &evidence)
    }
    fn stop(
        &mut self,
        reason: habitat_engine::task::driver::StopReason,
    ) -> Result<bool, Self::Error> {
        self.stop(reason)
    }
}

fn aggregate_evidence(
    evidence: &mut Evidence<'_>,
    execution: &Execution,
    clock: TaskClock,
) -> Result<Payload, Error> {
    use serde_json::json;
    let aggregate = execution.aggregate.as_ref().ok_or(Error::State)?;
    let stopped = execution.aggregate_stop.as_ref().ok_or(Error::State)?;
    let started = execution.aggregate_start.as_ref().ok_or(Error::State)?;
    let mut calls = Vec::new();
    for call in aggregate.calls() {
        let report = match &call.report {
            Ok(report) => {
                // Embed exact manager stream bytes; raw JSON does not confer typed
                // graph edges to nested payload references.
                json!({"stdout_bytes":report.stdout.bytes,"stderr_bytes":report.stderr.bytes,"exit_code":report.exit_code,"signal":report.signal,
                    "interruption":format!("{:?}",report.interruption),"leader_reaped":report.leader_reaped,
                    "process_group_settled":report.process_group_settled,"pending":report.pending.is_some(),
                    "stdout_eof":report.stdout.eof,"stderr_eof":report.stderr.eof,
                    "stdout_truncated":report.stdout.truncated,"stderr_truncated":report.stderr.truncated,
                    "started_ms":clock::cost_ms(clock.origin,report.started_at)?,"elapsed_ms":report.elapsed.as_millis()})
            }
            Err(refusal) => json!({"refusal":format!("{refusal:?}")}),
        };
        calls.push(json!({"argv_bytes":call.argv.iter().map(|value|value.as_bytes()).collect::<Vec<_>>(),"report":report}));
    }
    let bytes = serde_json::to_vec(&json!({"kind":"fixed-u64-aggregate-observation/1",
        "executor_observation":{"reference":execution.executor_observation.evidence,
            "object":execution.executor_observation.object,
            "exact_raw_bytes":execution.executor_observation.raw_bytes,
            "confirmed":execution.executor_observation.observation},
        "phase":format!("{:?}",aggregate.phase()),"coordinator_pid":started.coordinator_pid,
        "origin_cgroup":started.before,"active_cgroup":started.after,"aggregate":started.aggregate,
        "limits":format!("{:?}",started.limits),"calls":calls,
        "direct_empty_ms":clock::cost_ms(clock.origin,stopped.direct_empty_observed_at)?,
        "manager":{"name":stopped.manager.name,"load_state":stopped.manager.load_state,
            "active_state":stopped.manager.active_state,"sub_state":stopped.manager.sub_state,
            "control_group":stopped.manager.control_group,"observed_ms":clock::cost_ms(clock.origin,stopped.manager.observed_at)?},
        "retained_workspace":"private source and immutable output evidence retained; no writable process descendants"}))
        .map_err(|_| Error::Publication)?;
    evidence
        .payload(&bytes, "application/json")
        .map_err(|_| Error::Publication)
}

fn unique_objects(
    registry: &BTreeMap<String, (Ref, store::Object)>,
) -> Result<Vec<store::Object>, Error> {
    let mut objects = BTreeMap::new();
    for (_, object) in registry.values() {
        if let Some(previous) = objects.insert(object.digest().to_owned(), object.clone())
            && previous != *object
        {
            return Err(Error::Identity);
        }
    }
    Ok(objects.into_values().collect())
}

fn materialize_source(
    sources: &Sources,
    root: &std::path::Path,
    index: usize,
    deadline: Instant,
) -> Result<Snapshot, Error> {
    let source = if index == 0 {
        let copied = sources
            .baseline
            .materialize(root, "source", &[], deadline)
            .map_err(Error::Materialize)?;
        Snapshot::capture(
            &copied.path,
            &sources.baseline.source_identities(),
            deadline,
        )
        .map_err(Error::Workspace)?
    } else {
        let replacement = sources
            .repaired
            .entries()
            .find_map(|entry| match &entry.content {
                workspace::Content::File { bytes, .. } if entry.path == "src/lib.rs" => {
                    Some(bytes.as_slice())
                }
                _ => None,
            })
            .ok_or(Error::Identity)?;
        repair::apply(
            &sources.baseline,
            &sources.repaired,
            "src/lib.rs",
            replacement,
            root,
            "source",
            deadline,
        )
        .map_err(Error::Repair)?
    };

    Ok(source)
}

fn collect_receipt(
    evidence: &mut Evidence<'_>,
    execution: &Execution,
    clock: TaskClock,
) -> Result<u64_receipt::Collected, Error> {
    let control = execution.control.as_ref().ok_or(Error::State)?;
    let run = control
        .run
        .as_ref()
        .and_then(|value| value.as_ref().ok())
        .ok_or(Error::State)?;
    let frozen = execution.frozen.as_ref().ok_or(Error::State)?;
    let oracle = u64_receipt::observe(frozen, run).map_err(Error::Receipt)?;
    let cleanup = aggregate_evidence(evidence, execution, clock)?;
    let cancelled = if control.cause == durable_control::Cause::DurableCancellation {
        control
            .first_observed_after
            .and_then(|elapsed| control.started_at.checked_add(elapsed))
    } else {
        None
    };
    let observed_at = Instant::now();
    let collected = u64_receipt::finalize(
        evidence,
        frozen,
        run,
        &oracle,
        u64_receipt::Clock {
            origin: clock.origin,
            start_unix_ms: clock.unix_ms,
            end_unix_ms: clock::unix_ms()?,
            candidate_deadline: clock.candidate_deadline,
            work_deadline: clock.verification_deadline,
            cleanup_deadline: clock.deadline,
            observed_at,
            cancellation_observed_at: cancelled,
        },
        &u64_receipt::Cleanup {
            aggregate: habitat_engine::check::decision::Settlement::Settled,
            obligations: habitat_engine::check::decision::Settlement::Settled,
            evidence: cleanup,
            unresolved: Vec::new(),
        },
        clock.deadline,
    )
    .map_err(Error::Receipt)?;

    Ok(collected)
}

fn store_verdict(
    state: habitat_engine::contracts::receipt::VerdictV1State,
) -> store::VerificationVerdict {
    use habitat_engine::contracts::receipt::VerdictV1State;
    match state {
        VerdictV1State::PassCandidate => store::VerificationVerdict::Passed,
        VerdictV1State::Fail => store::VerificationVerdict::Failed,
        VerdictV1State::Invalid | VerdictV1State::Unmeasured => store::VerificationVerdict::Invalid,
        VerdictV1State::Error => store::VerificationVerdict::Error,
        VerdictV1State::Timeout => store::VerificationVerdict::Timeout,
        VerdictV1State::Cancelled => store::VerificationVerdict::Cancelled,
    }
}

#[cfg(test)]
#[path = "runtime_controls.rs"]
mod cancellation_controls;
