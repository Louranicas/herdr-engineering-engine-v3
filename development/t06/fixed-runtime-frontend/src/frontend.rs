//! Fixed driver composition. Desired outcomes are absent from this entrypoint.
use crate::{
    Result, checked, inputs,
    manifest::{self, Manifest},
    probes,
};
use habitat_engine::app::{
    durable_control::{self, CancellationQueue, PendingCancel, TaskId},
    evidence::Evidence,
};
use habitat_engine::check::graph::Graph;
use habitat_engine::contracts::{
    Sha256Digest, UuidV4,
    receipt::Ref,
    roster::{Kind, Locality, RosterDefinitionV1, Selection, Update},
};
use habitat_engine::store::{
    Allocation, ArtifactStaging, Effect, Object, Principal, RequestSource, Settlement, Stop, Store,
    Submission, TaskHead,
};
use habitat_engine::task::driver;
use hee3_fixed_task_runtime::{
    clock::TaskClock,
    runtime::{AttemptCancellationPoint, Config, TaskRuntime},
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::SyncSender,
};
use std::time::{Duration, Instant};

fn uuid(value: &str) -> Result<UuidV4<'_>> {
    checked(UuidV4::parse(value))
}
#[derive(Serialize)]
struct Changes<'a> {
    kind: &'a str,
    version: u32,
    updates: Vec<Change>,
}
#[derive(Serialize)]
struct Change {
    idempotency_key: String,
    audit_reason: String,
    definition: RosterDefinitionV1,
}
pub(crate) fn roster_bytes(key: &str, owner: &str) -> Result<Vec<u8>> {
    uuid(key)?;
    let update = Update {
        idempotency_key: key.into(),
        record_id: None,
        expected_revision: None,
        audit_reason: "Register the explicitly selected fixed local native executor".into(),
        definition: RosterDefinitionV1 {
            kind: Kind::Agent,
            display_name: "Fixed u64 native executor".into(),
            owner_id: owner.into(),
            version: "T06-u64-native-1".into(),
            capabilities: vec!["u64-fixed-workload".into()],
            locality: Locality::Local,
            endpoint_ref: None,
            limitations:
                "Fixed offline u64 workload only; no provider/model identity or module admission"
                    .into(),
        },
    };
    checked(update.validate())?;
    checked(toml::to_string(&Changes {
        kind: "hee3-roster-changes",
        version: 1,
        updates: vec![Change {
            idempotency_key: update.idempotency_key,
            audit_reason: update.audit_reason,
            definition: update.definition,
        }],
    }))
    .map(String::into_bytes)
}
fn agent(
    store: &mut Store,
    principal: &Principal,
    m: &Manifest,
    out: &Path,
    deadline: Instant,
) -> Result<(String, Selection)> {
    let bytes = roster_bytes(&m.identities.roster_key, &m.owner_id)?;
    manifest::write(&out.join("agent-changes.toml"), &bytes)?;
    let rows = checked(habitat_engine::roster::parse_changes(&bytes))?;
    let results =
        checked(store.roster_apply(principal, &rows, RequestSource::Import(&bytes), deadline))?;
    manifest::json(&out.join("agent-import.json"), &results)?;
    if results.len() != 1 {
        return Err("fixed native profile count".into());
    }
    let head = &results[0].head;
    Ok((
        head.record_id.clone(),
        Selection {
            record_id: head.record_id.clone(),
            expected_revision: head.record_version.clone(),
            capabilities: vec!["u64-fixed-workload".into()],
            local_only: true,
            version: Some(head.definition.version.clone()),
            ttl_ms: 60_000,
        },
    ))
}
fn registry(e: &Evidence<'_>) -> Value {
    json!({"registered":e.registered(),"pending":e.pending_publications(),"publication_error":format!("{:?}",e.last_publication_error())})
}
fn head(h: &TaskHead) -> Value {
    json!({"id":h.id,"generation":h.generation,"state":h.state,"cancellation":h.cancellation,"accepted_event":h.accepted_event,
    "criteria":h.criteria,"spent_ms":h.spent_ms,"reserved_work_ms":h.reserved_work_ms,"reserved_verify_ms":h.reserved_verify_ms})
}
pub(crate) fn readback(
    store: &Store,
    principal: &Principal,
    task: &str,
    out: &Path,
    deadline: Instant,
    accepted: bool,
) -> Result<()> {
    let current = store
        .get(principal, uuid(task)?, deadline)
        .map(|h| head(&h));
    let delivery = store.pending_delivery(256, deadline);
    let persisted = manifest::json(
        &out.join("store-readback.json"),
        &json!({"head":current.as_ref().ok(),"head_error":current.as_ref().err().map(|e|format!("{e:?}")),
        "outbox":delivery.as_ref().ok(),"outbox_error":delivery.as_ref().err().map(|e|format!("{e:?}")),"deadline":"original task deadline; no renewed read allowance","delivery_acknowledged":false}),
    );
    let validation = (|| {
        let actual = checked(current)?;
        let outbox = checked(delivery)?;
        if accepted
            && (actual["id"] != task
                || actual["state"] != "accepted"
                || actual["accepted_event"].is_null()
                || actual["cancellation"] != false
                || actual["reserved_work_ms"] != 0
                || actual["reserved_verify_ms"] != 0
                || outbox.len() != 1
                || Some(outbox[0].0.as_str()) != actual["accepted_event"].as_str()
                || outbox[0].1 != format!("{}:operator", rustix::process::geteuid().as_raw()))
        {
            return Err("accepted driver result lacks exact durable accepted head/outbox".into());
        }
        Ok(())
    })();
    retain(validation, "store readback projection", persisted)
}

pub(crate) fn graphs(
    staging: &ArtifactStaging,
    observation: &Value,
    out: &Path,
    deadline: Instant,
) -> Result<()> {
    let registered: BTreeMap<String, (Ref, Object)> = checked(serde_json::from_value(
        observation["staged_registry"].clone(),
    ))?;
    let mut evidence = Evidence::staged(staging, deadline);
    for (_, (reference, object)) in registered {
        checked(evidence.register(reference, object))?;
    }
    let mut summaries = Vec::new();
    let executions = observation["executions"]
        .as_array()
        .ok_or("runtime executions absent")?;
    let mut failed = executions.is_empty();
    for execution in observation["executions"]
        .as_array()
        .ok_or("runtime executions absent")?
    {
        if execution["receipt"].is_null() {
            failed = true;
            continue;
        }
        let reference: Ref = checked(serde_json::from_value(
            execution["receipt"]["reference"].clone(),
        ))?;
        checked(habitat_engine::contracts::receipt::TypedRef::<
            habitat_engine::contracts::receipt::ReceiptV1,
        >::new(reference.clone()))?;
        let resolved = Graph::resolve(&evidence, &reference);
        failed |= resolved.is_err();
        summaries.push(match resolved {
            Ok(g) => json!({"root":reference,"objects":g.object_count(),"bytes":g.total_bytes()}),
            Err(error) => json!({"root":reference,"error":format!("{error:?}")}),
        });
    }
    manifest::json(
        &out.join("retained-graphs.json"),
        &json!({"scope":"independent exact current staged closure readback; no admission from this summary","roots":summaries}),
    )?;
    if failed {
        Err("incomplete or unresolved retained receipt closure".into())
    } else {
        Ok(())
    }
}
/// Run the one fixed offline task with its original clock and retained observations.
/// # Errors
/// Refuses input drift, unavailable observations, preparation/runtime failure or failed readback.
/// Every created output directory is retained; errors do not imply settled runtime custody.
pub fn run(manifest_path: &Path, out: &Path, argv: &[String]) -> Result<()> {
    run_controlled(manifest_path, out, argv, None)
}

/// Exact admission generation supplied by the sole Store owner.
#[derive(Debug, Clone, Serialize)]
pub struct AdmissionCancellationPoint {
    #[serde(skip)]
    pub task_origin: Instant,
    pub task_elapsed_ns: String,
    pub task_id: String,
    pub expected_generation: String,
}

/// Trusted in-process input. It grants no Store connection or external transport.
pub struct InProcessCancellation {
    pub queue: CancellationQueue,
    pub attempt_points: SyncSender<AttemptCancellationPoint>,
    pub admission_points: Option<SyncSender<AdmissionCancellationPoint>>,
}

/// Run with the existing task-bound cancellation queue and nonblocking phase observations.
/// # Errors
/// Refuses mismatched queue identity, unavailable phase delivery and the same failures as `run`.
pub fn run_with_cancellation(
    manifest_path: &Path,
    out: &Path,
    argv: &[String],
    control: InProcessCancellation,
) -> Result<()> {
    run_controlled(manifest_path, out, argv, Some(control))
}

fn run_controlled(
    manifest_path: &Path,
    out: &Path,
    argv: &[String],
    control: Option<InProcessCancellation>,
) -> Result<()> {
    if !out.is_absolute()
        || checked(out.parent().ok_or("output parent")?.canonicalize())?
            != out.parent().ok_or("output parent")?
    {
        return Err("fresh absolute canonical output required".into());
    }
    // Allocate exactly one clock before admission and all preparation. No subsequent reset.
    let clock = checked(TaskClock::start())?;
    manifest::private_dir(out)?;
    let actual = run_inner(manifest_path, out, argv, clock, control);
    report_frontend(actual, manifest_path, out, clock)
}

pub(crate) fn report_frontend(
    actual: Result<()>,
    manifest_path: &Path,
    out: &Path,
    clock: TaskClock,
) -> Result<()> {
    let persisted = manifest::json(
        &out.join("frontend-result.json"),
        &json!({"kind":"fixed-u64-frontend-result/1","returned_error":actual.as_ref().err(),"operation_returned_ok":actual.is_ok(),
        "elapsed_ms":clock.elapsed().as_millis().to_string(),"scope":"actual development run, not module admission; inspect driver and retained custody facts","source_manifest":manifest_path}),
    );
    retain(actual, "frontend result", persisted)
}

fn run_inner(
    manifest_path: &Path,
    out: &Path,
    argv: &[String],
    clock: TaskClock,
    control: Option<InProcessCancellation>,
) -> Result<()> {
    let m = Manifest::load(
        manifest_path,
        &out.join("source-manifest.raw.json"),
        clock.deadline(),
    )?;
    manifest::json(&out.join("frozen-manifest.json"), &m)?;
    let executor = manifest::read(&m.executor, 16 * 1024 * 1024, clock.deadline())?;
    if checked(std::env::current_exe())? != m.executor.path
        || manifest::digest(&executor) != m.executor.sha256
    {
        return Err("run the exact frozen collector executable".into());
    }
    for dir in ["store", "staging", "runtime"] {
        manifest::private_dir(&out.join(dir))?;
    }
    let principal = checked(Principal::new(
        rustix::process::geteuid().as_raw(),
        "operator",
    ))?;
    let mut store = checked(Store::open(
        &out.join("store"),
        uuid(&m.identities.store_generation)?,
        uuid(&m.identities.store_epoch)?,
        true,
        clock.deadline(),
    ))?;
    let staged = checked(ArtifactStaging::open(
        &out.join("staging"),
        true,
        clock.deadline(),
    ))?;
    let task = checked(TaskId::parse(&m.identities.task))?;
    let (_private_handle, private_queue) = durable_control::cancellation_queue(task.clone());
    let (queue, attempt_points, admission_points) = match control {
        Some(control) => (
            control.queue,
            Some(control.attempt_points),
            control.admission_points,
        ),
        None => (private_queue, None, None),
    };
    if queue.task() != &task {
        return Err("cancellation queue task mismatch before admission".into());
    }
    let mut preparation_registry = json!({"registered":{},"pending":[],"publication_error":"None"});
    let execution = prepare_and_run(
        &m,
        (out, argv),
        clock,
        (&mut store, &principal),
        &mut preparation_registry,
        &staged,
        (&queue, attempt_points, admission_points),
    );
    report_execution(
        execution,
        (&store, &principal, &m.identities.task),
        out,
        clock,
        &preparation_registry,
    )
}

pub(crate) fn report_execution(
    execution: Result<()>,
    owner: (&Store, &Principal, &str),
    out: &Path,
    clock: TaskClock,
    registry: &Value,
) -> Result<()> {
    let (store, principal, task) = owner;
    let accepted = execution.is_ok();
    let execution = retain(
        execution,
        "initial staging projection",
        manifest::json(&out.join("initial-staging-final.json"), registry),
    );
    let persisted = manifest::json(
        &out.join("execution-return.json"),
        &json!({"error":execution.as_ref().err(),"ok":execution.is_ok()}),
    );
    let execution = retain(execution, "execution return projection", persisted);
    let readback = readback(store, principal, task, out, clock.deadline(), accepted);
    match (execution, readback) {
        (Ok(()), result) => result,
        (Err(original), Ok(())) => Err(original),
        (Err(original), Err(readback)) => {
            Err(format!("execution: {original}; readback: {readback}"))
        }
    }
}

fn retain<T>(original: Result<T>, label: &str, persistence: Result<()>) -> Result<T> {
    match (original, persistence) {
        (result, Ok(())) => result,
        (Ok(_), Err(error)) => Err(format!("{label}: {error}")),
        (Err(original), Err(error)) => Err(format!("{original}; {label}: {error}")),
    }
}

/// Handle the actual preparation return, preserving both original and settlement errors.
pub(crate) fn preparation_result<T>(
    result: Result<T>,
    owner: (&mut Store, &Principal, &str),
    out: &Path,
    clock: TaskClock,
    admitted: bool,
    cleanup: &probes::Cleanup,
    registry: &Value,
) -> Result<T> {
    let result = retain(
        result,
        "preparation registry",
        manifest::json(&out.join("preparation-registry.json"), &registry),
    );
    let original = match result {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };
    preparation_failure(&original, owner, out, clock, admitted, cleanup, registry)
}

fn preparation_failure<T>(
    original: &str,
    owner: (&mut Store, &Principal, &str),
    out: &Path,
    clock: TaskClock,
    admitted: bool,
    cleanup: &probes::Cleanup,
    registry: &Value,
) -> Result<T> {
    let (store, principal, task) = owner;
    let elapsed = clock.elapsed();
    let used_ms = u64::try_from(elapsed.as_nanos().div_ceil(1_000_000)).ok();
    let mut observation = json!({
        "kind":"preparation-failure-settlement/1","original_error":original,
        "admission_observed":admitted,"cleanup":cleanup.observation(),
        "task_elapsed_ns":elapsed.as_nanos().to_string(),"used_ms":used_ms,
        "clock":"original task clock; conservative ceiling milliseconds; sampled before terminal persistence",
        "registry":registry,"settled":false,"task":task,
    });
    let settlement: Result<Value> = (|| {
        if !admitted {
            return Err("no observed admission commit; no preparation stop attempted".into());
        }
        let publication_settled = registry["pending"].as_array().is_some_and(Vec::is_empty)
            && registry["publication_error"] == "None";
        if !cleanup.settled() || !publication_settled || used_ms.is_none() {
            return Err(
                "preparation custody or accounting unresolved; reservations retained".into(),
            );
        }
        let current = checked(store.get(principal, uuid(task)?, clock.deadline()))?;
        let event = checked(habitat_engine::app::evidence::fresh_id(clock.deadline()))?;
        let stage = checked(habitat_engine::app::evidence::fresh_id(clock.deadline()))?;
        observation["event_id"] = json!(event.as_str());
        observation["expected_generation"] = json!(current.generation);
        let bytes = checked(serde_json::to_vec(&observation))?;
        let evidence = checked(store.publish(&bytes, uuid(stage.as_str())?, clock.deadline()))?;
        observation["evidence"] = json!(evidence);
        let stopped = checked(store.finish_preparation(
            principal,
            Stop {
                task: uuid(task)?,
                generation: checked(current.generation.parse())?,
                reason: &checked(habitat_engine::contracts::receipt::Name::new(
                    if current.cancellation {
                        "cancelled"
                    } else {
                        "preparation_failed"
                    },
                ))?,
                evidence: &evidence,
                event: uuid(event.as_str())?,
            },
            Settlement {
                effect: Effect::None,
                used_ms,
                cleanup_settled: true,
                ready_to_verify: false,
            },
            clock.deadline(),
        ))?;
        Ok(json!({"generation":stopped.generation,"cancelled":stopped.cancelled}))
    })();
    observation["settled"] = json!(settlement.is_ok());
    observation["settlement"] = json!(settlement.as_ref().ok());
    observation["settlement_error"] = json!(settlement.as_ref().err());
    let returned = match settlement {
        Ok(_) => format!("{original}; preparation terminal settlement committed"),
        Err(error) => format!("{original}; preparation settlement unresolved: {error}"),
    };
    retain(
        Err(returned),
        "preparation settlement report",
        manifest::json(&out.join("preparation-settlement.json"), &observation),
    )
}

fn admit(
    store: &mut Store,
    principal: &Principal,
    m: &Manifest,
    out: &Path,
    clock: TaskClock,
    committed: &mut bool,
) -> Result<()> {
    let criteria = manifest::digest(b"u64-frozen-exact-output");
    let request = checked(serde_json::to_vec(
        &json!({"kind":"fixed-u64-task/1","task":m.identities.task,"workload":"WL-U64-PARSE-001/v1","criteria_sha256":criteria,
        "work_ms":900_000,"verify_ms":300_000,"limit_ms":1_200_000,"maximum_attempts":3,"provider_calls":0}),
    ))?;
    manifest::write(&out.join("submission.json"), &request)?;
    let admission = checked(store.submit(
        Submission {
            principal,
            key: uuid(&m.identities.submit_key)?,
            task: uuid(&m.identities.task)?,
            event: uuid(&m.identities.submit_event)?,
            request_bytes: &request,
            criteria: checked(Sha256Digest::parse(&criteria))?,
            allocation: Allocation {
                limit_ms: 1_200_000,
                work_ms: 900_000,
                verify_ms: 300_000,
            },
        },
        clock.deadline(),
    ))?;
    *committed = true;
    manifest::json(&out.join("admission.json"), &admission)?;
    Ok(())
}

/// Service durable intent on the only Store owner while Store-free preparation runs.
struct PreparationControl<'a> {
    queue: &'a CancellationQueue,
    store: &'a mut Store,
    principal: &'a Principal,
    clock: TaskClock,
    pending: Option<PendingCancel>,
    boundaries: &'a mut Vec<Value>,
    error: Option<String>,
}
impl PreparationControl<'_> {
    fn prepare<'a>(
        &mut self,
        m: &Manifest,
        invocation: (&Path, &[String]),
        staged: &'a ArtifactStaging,
        preparation_registry: &mut Value,
        agent: (String, Selection),
        control: (&mut probes::Cleanup, &AtomicBool),
    ) -> Result<(Config, Evidence<'a>)> {
        let clock = self.clock;
        let (cleanup, cancelled) = control;
        self.poll(cancelled);
        preparing(cancelled, clock)?;
        let worker = std::thread::scope(|scope| {
            let worker = scope.spawn(|| {
                // Construct the staging-only adapter on its worker. Neither Evidence's Store
                // variant nor the coordinator Store crosses this boundary.
                let mut evidence = Evidence::staged(staged, clock.deadline());
                let config = prepare_config(
                    m,
                    invocation,
                    clock,
                    &mut evidence,
                    agent,
                    cleanup,
                    cancelled,
                );
                (config, registry(&evidence), evidence.into_registered())
            });
            while !worker.is_finished() {
                self.poll(cancelled);
                std::thread::sleep(Duration::from_millis(100));
            }
            let result = worker.join();
            self.poll(cancelled);
            result
        });
        let Ok((config, observed_registry, registered)) = worker else {
            *preparation_registry =
                json!({"worker_panicked":true,"pending":null,"publication_error":"unknown"});
            return Err("preparation worker panicked; staging custody unknown".into());
        };
        *preparation_registry = observed_registry;
        let config = retain(
            config,
            "preparation durable observation",
            self.error.clone().map_or(Ok(()), Err),
        )?;
        preparing(cancelled, clock)?;
        let mut runtime_evidence = Evidence::staged(staged, clock.deadline());
        for (_, (reference, object)) in registered {
            self.poll(cancelled);
            preparing(cancelled, clock)?;
            checked(runtime_evidence.register(reference, object))?;
        }
        self.poll(cancelled);
        if let Some(error) = self.error.clone() {
            return Err(error);
        }
        preparing(cancelled, clock)?;
        Ok((config, runtime_evidence))
    }
    fn poll(&mut self, cancelled: &AtomicBool) {
        if self.error.is_some() {
            return;
        }
        match self.queue.observe(
            self.store,
            self.principal,
            &mut self.pending,
            self.clock.origin(),
            self.clock.deadline(),
        ) {
            Ok(boundary) => {
                if boundary.head.cancellation {
                    cancelled.store(true, Ordering::Release);
                }
                self.boundaries.push(json!({"head":head(&boundary.head),
                    "observed_after_ns":boundary.observed_after.as_nanos().to_string(),
                    "command":boundary.command.map(|command|json!({
                        "task":command.command.task.as_str(),"expected":command.command.expected.to_string(),
                        "event":command.command.event.as_str(),
                        "observed_after_ns":command.observed_after.as_nanos().to_string(),
                        "result":command.result.as_ref().ok(),
                        "error":command.result.as_ref().err().map(|e|format!("{e:?}"))}))}));
            }
            Err(error) => {
                self.boundaries.push(json!({"observation_error":format!("{error:?}"), "task_elapsed_ns":self.clock.elapsed().as_nanos().to_string()}));
                self.error = Some(format!("{error:?}"));
                cancelled.store(true, Ordering::Release);
            }
        }
    }
}
fn preparing(cancelled: &AtomicBool, clock: TaskClock) -> Result<()> {
    checked(clock.dispatch())?;
    if cancelled.load(Ordering::Acquire) {
        Err("preparation interrupted; inspect durable observations".into())
    } else {
        Ok(())
    }
}

fn prepare_config(
    m: &Manifest,
    invocation: (&Path, &[String]),
    clock: TaskClock,
    evidence: &mut Evidence<'_>,
    agent: (String, Selection),
    cleanup: &mut probes::Cleanup,
    cancelled: &AtomicBool,
) -> Result<Config> {
    let (out, argv) = invocation;
    let (agent_record_id, selection) = agent;
    preparing(cancelled, clock)?;
    let sources = inputs::sources(m, clock.deadline())?;
    preparing(cancelled, clock)?;
    let assets = inputs::Assets::load(m, clock.deadline())?;
    let tools = manifest::tools(&m.tools, clock.deadline())?;
    let observations = probes::capture(m, &out.join("probes"), clock, cleanup, cancelled)?;
    preparing(cancelled, clock)?;
    let recipes = inputs::pair(
        evidence,
        &inputs::Preparation {
            manifest: m,
            assets: &assets,
            sources: &sources,
            tools: &tools,
            observations: &observations,
            argv,
            deadline: clock.deadline(),
        },
    )?;
    preparing(cancelled, clock)?;
    let plans:Vec<_>=recipes.iter().map(|p|json!({"identity":p.prepared.identity,"subjects":p.prepared.subjects,"invocation":p.prepared.invocation,
                        "host":p.host,"recipe":p.recipe,"review_provenance":p.review_provenance,"session":p.session,"workspace":p.workspace})).collect();
    manifest::json(&out.join("prepared-attempts.json"), &plans)?;
    Ok(Config {
        sources,
        recipes,
        tools,
        root: out.join("runtime"),
        agent_record_id,
        selections: vec![selection],
        busctl: m.tools.busctl.path.clone(),
        busctl_sha256: m.tools.busctl.sha256.clone(),
        runtime_dir: m.runtime_dir.clone(),
        executor: m.executor.path.clone(),
        executor_sha256: m.executor.sha256.clone(),
    })
}

fn prepare_and_run(
    m: &Manifest,
    invocation: (&Path, &[String]),
    clock: TaskClock,
    store_owner: (&mut Store, &Principal),
    preparation_registry: &mut Value,
    staged: &ArtifactStaging,
    control: (
        &CancellationQueue,
        Option<SyncSender<AttemptCancellationPoint>>,
        Option<SyncSender<AdmissionCancellationPoint>>,
    ),
) -> Result<()> {
    let (store, principal) = store_owner;
    let (out, _) = invocation;
    let (queue, attempt_points, admission_points) = control;
    let mut admitted = false;
    let mut cleanup = probes::Cleanup::default();
    let cancelled = AtomicBool::new(false);
    let mut boundaries = Vec::new();
    let result: Result<(Config, Evidence<'_>, TaskId)> = (|| {
        let (agent_record_id, selection) = agent(store, principal, m, out, clock.deadline())?;
        admit(store, principal, m, out, clock, &mut admitted)?;
        if let Some(sender) = admission_points {
            let current =
                checked(store.get(principal, uuid(&m.identities.task)?, clock.deadline()))?;
            checked(sender.try_send(AdmissionCancellationPoint {
                task_origin: clock.origin(),
                task_elapsed_ns: clock.elapsed().as_nanos().to_string(),
                task_id: m.identities.task.clone(),
                expected_generation: current.generation,
            }))?;
        }
        let mut owner = PreparationControl {
            queue,
            store,
            principal,
            clock,
            pending: None,
            boundaries: &mut boundaries,
            error: None,
        };
        let (config, runtime_evidence) = owner.prepare(
            m,
            invocation,
            staged,
            preparation_registry,
            (agent_record_id, selection),
            (&mut cleanup, &cancelled),
        )?;
        Ok((
            config,
            runtime_evidence,
            checked(TaskId::parse(&m.identities.task))?,
        ))
    })();
    let result = retain(
        result,
        "preparation control projection",
        manifest::json(
            &out.join("preparation-control.json"),
            &json!({"boundaries":boundaries,"cancel_signal":cancelled.load(Ordering::Acquire),"poll_interval_ms":100,"sole_store_owner":true}),
        ),
    );
    let (config, runtime_evidence, task) = preparation_result(
        result,
        (store, principal, &m.identities.task),
        out,
        clock,
        admitted,
        &cleanup,
        preparation_registry,
    )?;
    let mut runtime_entered = false;
    let execution = (|| {
        let mut runtime = checked(TaskRuntime::new(
            store,
            principal,
            task,
            queue,
            runtime_evidence,
            config,
            clock,
        ))?;
        runtime_entered = true;
        if let Some(sender) = attempt_points {
            checked(runtime.set_attempt_points(sender))?;
        }
        run_runtime(runtime, staged, out, clock)
    })();
    match execution {
        Err(original) if !runtime_entered => preparation_failure(
            &original,
            (store, principal, &m.identities.task),
            out,
            clock,
            admitted,
            &cleanup,
            preparation_registry,
        ),
        result => result,
    }
}

fn run_runtime(
    mut runtime: TaskRuntime<'_>,
    staged: &ArtifactStaging,
    out: &Path,
    clock: TaskClock,
) -> Result<()> {
    let result = driver::run(&mut runtime, 1);
    let observation = runtime.observation();
    manifest::json(&out.join("runtime-observation.json"), &observation)?;
    manifest::json(
        &out.join("driver-result.json"),
        &json!({"ok":result.is_ok(),"outcome":result.as_ref().ok().map(|v|format!("{v:?}")),"error":result.as_ref().err().map(|v|format!("{v:?}"))}),
    )?;
    drop(runtime);
    let graph_result = graphs(staged, &observation, out, clock.deadline());
    if let Err(error) = &graph_result {
        manifest::json(
            &out.join("graph-readback-error.json"),
            &json!({"error":error,"deadline_renewed":false}),
        )?;
    }
    accepted_outcome(checked(result)?)?;
    graph_result
}
pub(crate) fn accepted_outcome(outcome: driver::Outcome) -> Result<()> {
    match outcome {
        driver::Outcome::Accepted => Ok(()),
        other => Err(format!("driver returned nonacceptance: {other:?}")),
    }
}

#[cfg(test)]
mod cancellation_controls {
    use super::*;
    use habitat_engine::app::durable_control::{EventId, cancellation_queue};
    use habitat_engine::contracts::Generation;
    const TASK: &str = "c0000000-0000-4000-8000-000000000003";
    fn area() -> std::path::PathBuf {
        let id = habitat_engine::app::evidence::fresh_id(Instant::now() + Duration::from_secs(5))
            .unwrap();
        let path = std::env::temp_dir().join(format!("hee3-preparation-control-{}", id.as_str()));
        manifest::private_dir(&path).unwrap();
        path
    }
    fn fixture(path: &Path, clock: TaskClock) -> (Store, Principal, String) {
        let mut store = Store::open(
            path,
            uuid("c0000000-0000-4000-8000-000000000001").unwrap(),
            uuid("c0000000-0000-4000-8000-000000000002").unwrap(),
            true,
            clock.deadline(),
        )
        .unwrap();
        let principal = Principal::new(rustix::process::geteuid().as_raw(), "operator").unwrap();
        store
            .submit(
                Submission {
                    principal: &principal,
                    key: uuid("c0000000-0000-4000-8000-000000000004").unwrap(),
                    task: uuid(TASK).unwrap(),
                    event: uuid("c0000000-0000-4000-8000-000000000005").unwrap(),
                    request_bytes: b"preparation cancellation fixture",
                    criteria: Sha256Digest::parse(&format!("sha256:{}", "a".repeat(64))).unwrap(),
                    allocation: Allocation {
                        limit_ms: 1_200_000,
                        work_ms: 900_000,
                        verify_ms: 300_000,
                    },
                },
                clock.deadline(),
            )
            .unwrap();
        let generation = store
            .get(&principal, uuid(TASK).unwrap(), clock.deadline())
            .unwrap()
            .generation;
        (store, principal, generation)
    }
    #[test]
    fn owner_commits_exact_intent_and_preserves_stale_counterevidence() {
        for stale in [false, true] {
            let root = area();
            let out = area();
            let clock = TaskClock::start().unwrap();
            let (mut store, principal, generation) = fixture(&root, clock);
            let (handle, queue) = cancellation_queue(TaskId::parse(TASK).unwrap());
            let expected: Generation = if stale { "999" } else { &generation }.parse().unwrap();
            handle
                .try_cancel(
                    expected,
                    EventId::parse("c0000000-0000-4000-8000-000000000006").unwrap(),
                )
                .unwrap();
            let cancelled = AtomicBool::new(false);
            let mut boundaries = Vec::new();
            let mut owner = PreparationControl {
                queue: &queue,
                store: &mut store,
                principal: &principal,
                clock,
                pending: None,
                boundaries: &mut boundaries,
                error: None,
            };
            owner.poll(&cancelled);
            assert!(owner.error.is_none());
            drop(owner);
            assert_eq!(cancelled.load(Ordering::Acquire), !stale);
            assert_eq!(boundaries.len(), 1);
            assert_eq!(
                boundaries[0]["command"]["event"],
                "c0000000-0000-4000-8000-000000000006"
            );
            assert_eq!(boundaries[0]["command"]["error"].is_null(), !stale);
            let head = store
                .get(&principal, uuid(TASK).unwrap(), clock.deadline())
                .unwrap();
            assert_eq!(head.cancellation, !stale);
            assert_eq!(head.accepted_event, None);
            if !stale {
                let result: Result<()> = preparation_failure(
                    "durable cancellation during preparation",
                    (&mut store, &principal, TASK),
                    &out,
                    clock,
                    true,
                    &probes::Cleanup::default(),
                    &json!({"registered":{},"pending":[],"publication_error":"None"}),
                );
                assert!(
                    result
                        .unwrap_err()
                        .contains("terminal settlement committed")
                );
                let head = store
                    .get(&principal, uuid(TASK).unwrap(), clock.deadline())
                    .unwrap();
                assert_eq!(head.state, "cancelled");
                assert_eq!(head.reserved_work_ms, 0);
                assert_eq!(head.reserved_verify_ms, 0);
                let settlement: Value = serde_json::from_slice(
                    &std::fs::read(out.join("preparation-settlement.json")).unwrap(),
                )
                .unwrap();
                assert_eq!(settlement["settled"], true);
                assert_eq!(settlement["used_ms"], head.spent_ms);
                assert_eq!(
                    store.pending_delivery(256, clock.deadline()).unwrap().len(),
                    1
                );
            }
            drop(store);
            std::fs::remove_dir_all(root).unwrap();
            std::fs::remove_dir_all(out).unwrap();
        }
    }
    #[test]
    fn preset_signal_refuses_probe_before_input_or_process_effect() {
        let root = area();
        let mut cleanup = probes::Cleanup::default();
        let pin = manifest::Pin {
            path: root.join("missing-must-not-open"),
            sha256: format!("sha256:{}", "a".repeat(64)),
            bytes: 1,
        };
        let result = probes::command(
            &pin,
            &[],
            &root,
            &root,
            "never",
            TaskClock::start().unwrap(),
            (&mut cleanup, &AtomicBool::new(true)),
        );
        assert_eq!(
            result.unwrap_err(),
            "preparation stopped before probe dispatch"
        );
        assert!(cleanup.settled());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir_all(root).unwrap();
    }
}
