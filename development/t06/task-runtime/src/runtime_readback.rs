//! Retained application observations. This projection grants no acceptance.
use super::{Execution, TaskRuntime, durable_control, workload};
use habitat_engine::worker::{namespace, process, workspace::Snapshot};
use serde_json::{Value, json};
use std::time::Instant;

fn time(origin: Instant, instant: Instant) -> Option<u64> {
    super::clock::cost_ms(origin, instant).ok()
}

fn stream(stream: &process::Stream) -> Value {
    json!({"bytes":stream.bytes, "observed_bytes":stream.observed_bytes,
        "eof":stream.eof, "truncated":stream.truncated, "failed":stream.failed})
}

fn process(report: &process::ProcessReport, origin: Instant) -> Value {
    json!({"started_ms":time(origin, report.started_at),
        "elapsed_ns":report.elapsed.as_nanos().to_string(), "leader_pid":report.leader_pid,
        "exit_code":report.exit_code,"signal":report.signal,
        "interruption":format!("{:?}",report.interruption),
        "interruption_observed_ms":report.interruption_observed_at.and_then(|value| time(origin,value)),
        "stdout":stream(&report.stdout),"stderr":stream(&report.stderr),
        "signals":format!("{:?}",report.signals), "leader_reaped":report.leader_reaped,
        "process_group_settled":report.process_group_settled,
        "observer_ready":report.observer_ready,"pending":format!("{:?}",report.pending)})
}

fn namespace(report: &namespace::NamespaceReport, origin: Instant) -> Value {
    let facts = &report.observer.facts;
    json!({"process":process(&report.process,origin),
        "candidate_stdout":stream(&report.observer.candidate_stdout),
        "candidate_stderr":stream(&report.observer.candidate_stderr),
        "failure":format!("{:?}",report.observer.failure()),
        "postrun_sources_match":report.postrun_sources_match,
        "json_exit_matches":report.json_exit_matches,
        "source_sha256_before":facts.source_sha256_before,
        "source_sha256_after":facts.source_sha256_after,
        "source_sha256_postrun_host":facts.source_sha256_postrun_host,
        "initial_namespace_ids":facts.initial_namespace_ids,
        "before_mount_sha256":facts.before_mount_sha256,"after_mount_sha256":facts.after_mount_sha256,
        "root_device_inode":facts.root_device_inode,
        "pid1":facts.pid1,"shim_pid":facts.shim_pid,"children":facts.child_snapshot,
        "public_files_verified":facts.public_files_verified,"protected_paths_absent":facts.protected_paths_absent,
        "descriptors_verified":facts.descriptors_verified,"protection_verified":facts.protection_verified,
        "release_sent":facts.release_sent,"terminal_json":facts.terminal_json,
        "json_exit_code":facts.json_exit_code,"native_status":format!("{:?}",facts.native_status),
        "native_status_eof":facts.native_status_eof,"native_status_consistent":facts.native_status_consistent,
        "pid1_terminal":facts.pid1_terminal,"shim_terminal":facts.shim_terminal,
        "channel_cleanup_complete":facts.channel_cleanup_complete,
        "term_sent":facts.namespace_term_sent,"kill_sent":facts.namespace_kill_sent,
        "adversarial_isolation":facts.adversarial_isolation,"unresolved":facts.unresolved})
}

fn control(report: &durable_control::Report, origin: Instant) -> Value {
    let run = report.run.as_ref().map(|run| match run {
        Err(error) => json!({"error":format!("{error:?}")}),
        Ok(run) => {
            let steps: Vec<_> = run.steps.iter().map(|step| match step {
                workload::Step::Completed { label, report } =>
                    json!({"label":label,"observed":namespace(report,origin)}),
                workload::Step::Refused { label, error } =>
                    json!({"label":label,"refused":format!("{error:?}")}),
            }).collect();
            json!({"outcome":format!("{:?}",run.outcome),"steps":steps,
                "process_cleanup_complete":run.process_cleanup_complete,
                "subjects_unchanged":run.subjects_unchanged,"cancellation_observed":run.cancellation_observed,
                "scratch_released":run.scratch_released,"retained_paths":run.retained_paths,
                "output_roots":run.outputs.iter().map(Snapshot::root).collect::<Vec<_>>()})
        }
    });
    json!({"started_ms":time(origin,report.started_at),"cause":format!("{:?}",report.cause),
        "first_observed_ms":report.first_observed_after.and_then(|after|
            report.started_at.checked_add(after)).and_then(|at|time(origin,at)),
        "observation_error":format!("{:?}",report.observation_error),
        "worker_panicked":report.worker_panicked,"polls":report.polls,
        "cancellations":format!("{:?}",report.cancellations),
        "pending_cancel":format!("{:?}",report.pending_cancel),
        "queue_disconnected":report.queue_disconnected,"run":run})
}

fn execution(execution: &Execution, origin: Instant) -> Value {
    let aggregate = execution.aggregate.as_ref().map(|aggregate| {
        json!({"unit":aggregate.unit(),"coordinator_unit":aggregate.coordinator_unit(),
            "phase":format!("{:?}",aggregate.phase()),
            "calls":aggregate.calls().iter().map(|call|json!({
                "argv":call.argv,"report":match &call.report {
                    Ok(report)=>process(report,origin),
                    Err(error)=>json!({"refusal":format!("{error:?}")}),
                }})).collect::<Vec<_>>(),
            "started":format!("{:?}",execution.aggregate_start),
            "stopped":execution.aggregate_stop.as_ref().map(|stopped|json!({
                "direct_empty_ms":time(origin,stopped.direct_empty_observed_at),
                "manager_observed_ms":time(origin,stopped.manager.observed_at),
                "name":stopped.manager.name,"load_state":stopped.manager.load_state,
                "active_state":stopped.manager.active_state,"sub_state":stopped.manager.sub_state,
                "control_group":stopped.manager.control_group})),
            "cleanup_error":format!("{:?}",execution.aggregate_cleanup_error)})
    });
    json!({"attempt":execution.roster.attempt.id,"ordinal":execution.roster.attempt.generation,
        "executor_observation":format!("{:?}",execution.executor_observation),
        "began_ms":time(origin,execution.began),
        "source":execution.source.as_ref().map(Snapshot::root),"job":execution.job,
        "aggregate":aggregate,"control":execution.control.as_ref().map(|report|control(report,origin)),
        "undispatched_observed_ms":execution.undispatched_at.and_then(|at|time(origin,at)),
        "work_settled":execution.work_settled,
        "verification_started":execution.verification_started,
        "verification_phases":execution.verification_phases.iter().map(|phase|json!({
            "name":phase.name,"began_ms":time(origin,phase.began),"ended_ms":time(origin,phase.ended),
            "elapsed_ns":phase.ended.checked_duration_since(phase.began).map(|elapsed|elapsed.as_nanos().to_string())
        })).collect::<Vec<_>>(),
        "verification_recorded":execution.verification_recorded,
        "receipt":execution.receipt.as_ref().map(|receipt|json!({
            "reference":receipt.receipt.reference,"state":receipt.decision.state(),
            "decision_inputs":receipt.decision_inputs,"oracle_observation":receipt.oracle_observation})),
        "collected_evidence":execution.collected_evidence.as_ref().map(|collected|json!({
            "registered":collected.registered,"pending":collected.pending,
            "publication_error":collected.publication_error})),
        "imported":execution.imported.as_ref().map(|imported|json!({
            "registered":imported.registered,"pending":imported.pending,
            "publication_error":imported.publication_error}))})
}

impl TaskRuntime<'_> {
    /// Snapshot retained observations without releasing any process, descriptor,
    /// artifact or Store ownership. Errors must also be retained by the caller.
    #[must_use]
    pub fn observation(&self) -> Value {
        json!({"kind":"fixed-u64-runtime-observation/1",
            "scope":"development execution; no module admission",
            "task":self.task.as_str(),"head":format!("{:?}",self.head),
            "clock":{"start_unix_ms":self.clock.unix_ms,
                "candidate_cutoff_ms":time(self.clock.origin,self.clock.candidate_deadline),
                "verification_cutoff_ms":time(self.clock.origin,self.clock.verification_deadline),
                "deadline_ms":time(self.clock.origin,self.clock.deadline)},
            "boundaries":self.boundary_observations.iter().map(|boundary|json!({
                "observed_ms":boundary.observed_after.as_millis().to_string(),
                "observed_after_ns":boundary.observed_after.as_nanos().to_string(),
                "head":format!("{:?}",boundary.head),"command":format!("{:?}",boundary.command)})).collect::<Vec<_>>(),
            "pending_cancel":format!("{:?}",self.pending_cancel),
            "pending_executor_observation":format!("{:?}",self.pending_executor_observation),
            "executions":self.executions.iter().map(|value|execution(value,self.clock.origin)).collect::<Vec<_>>(),
            "staged_registry":self.evidence.registered(),
            "staged_pending":self.evidence.pending_publications(),
            "staged_publication_error":format!("{:?}",self.evidence.last_publication_error())})
    }
}
