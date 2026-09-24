use super::{
    Cleanup, Clock, Collected, Error, ErrorKind, Evidence, Frozen, Instant, LABELS,
    NamespaceReport, OracleObservation, budget, bytes, c, capture, d, fail, graph, hash, json_raw,
    list, name, none, number, preparation, r, record, subjects, text, u64_oracle, workload,
};
use habitat_engine::worker::{namespace::NativeStatus, process::Interruption};
use serde_json::{Value, json};

pub(super) fn reports(run: &workload::Run) -> Result<Vec<&NamespaceReport>, Error> {
    if run.steps.is_empty() || run.steps.len() > 3 {
        return Err(fail(ErrorKind::UnsupportedRun));
    }
    run.steps
        .iter()
        .zip(LABELS)
        .map(|(step, expected)| match step {
            workload::Step::Completed { label, report } if *label == expected => {
                Ok(report.as_ref())
            }
            _ => Err(fail(ErrorKind::UnsupportedRun)),
        })
        .collect()
}
pub(super) fn normal(report: &NamespaceReport) -> bool {
    matches!(
        report.observer.facts.native_status,
        Some(NativeStatus::Exit { code: 0, .. })
    ) && report.observer.facts.native_status_consistent
        && report.json_exit_matches
        && report.process.interruption.is_none()
        && report.observer.failure().is_none()
}
fn state(producer: &r::ProducerV1) -> d::ProcessState {
    match (
        producer.status,
        producer.exit_code.value,
        producer.signal.value,
    ) {
        (r::ProducerV1Status::Exited, Some(code), _) => d::ProcessState::Exited(code),
        (r::ProducerV1Status::Signalled, _, Some(signal)) => d::ProcessState::Signalled(signal),
        (r::ProducerV1Status::NotStarted, _, _) => d::ProcessState::NotStarted,
        _ => d::ProcessState::Unknown,
    }
}
fn elapsed(origin: Instant, at: Instant) -> Result<u64, Error> {
    u64::try_from(
        at.checked_duration_since(origin)
            .ok_or_else(|| fail(ErrorKind::Binding))?
            .as_millis(),
    )
    .map_err(|_| fail(ErrorKind::Binding))
}
fn ns(origin: Instant, at: Instant) -> Result<u64, Error> {
    u64::try_from(
        at.checked_duration_since(origin)
            .ok_or_else(|| fail(ErrorKind::Binding))?
            .as_nanos(),
    )
    .map_err(|_| fail(ErrorKind::Binding))
}
fn log(complete: bool) -> d::LogState {
    if complete {
        d::LogState::Complete
    } else {
        d::LogState::ReadError
    }
}
fn settled(value: bool) -> d::Settlement {
    if value {
        d::Settlement::Settled
    } else {
        d::Settlement::Unknown
    }
}
fn artifact(reference: &r::Ref, role: &str) -> Result<r::ArtifactV1, Error> {
    Ok(r::ArtifactV1 {
        object: reference.clone(),
        role: name(role)?,
        required: true,
        truncated: false,
        availability: r::ArtifactV1Availability::Available,
        reason: text("trusted_application_observation")?,
    })
}
// Retain the exact predeclared references, including empty baseline patches.
// Re-publishing identical bytes would allocate different artifact identities.
fn subject_artifacts(
    evidence: &Evidence<'_>,
    subjects: &r::SubjectsV1,
    deadline: Instant,
) -> Result<Vec<r::ArtifactV1>, Error> {
    budget(deadline)?;
    let result = subjects
        .result_subject
        .value
        .as_ref()
        .ok_or_else(|| fail(ErrorKind::Binding))?;
    let patch = subjects
        .seed_to_result_patch
        .value
        .as_ref()
        .ok_or_else(|| fail(ErrorKind::Binding))?;
    graph::Graph::resolve(evidence, patch.as_ref())
        .map_err(|error| fail(ErrorKind::Graph(error)))?;
    let closure = graph::Graph::resolve(evidence, result.as_ref())
        .map_err(|error| fail(ErrorKind::Graph(error)))?;
    let subject: r::SubjectV1 = super::scalar(r::decode(
        closure
            .get(result.as_ref())
            .map_err(|error| fail(ErrorKind::Graph(error)))?
            .bytes(),
    ))?;
    let mut artifacts = vec![artifact(patch.as_ref(), "seed_to_result_patch")?];
    for value in closure
        .rows(subject.files.as_ref())
        .map_err(|error| fail(ErrorKind::Graph(error)))?
    {
        budget(deadline)?;
        let file: r::SubjectFileV1 =
            serde_json::from_value(value.clone()).map_err(|_| fail(ErrorKind::Encoding))?;
        if let Some(content) = file.content.value {
            artifacts.push(artifact(content.as_ref(), "result_source_content")?);
        }
    }
    budget(deadline)?;
    Ok(artifacts)
}

#[cfg(test)]
#[path = "../../tests/receipt_inventory_controls.rs"]
mod receipt_inventory_controls;

fn oracle_value(observation: &OracleObservation) -> Value {
    match &observation.result {
        Some(Ok(value)) => {
            json!({"matched":value.matched,"failed":value.failed,"vectors":value.vectors.iter().map(|v|json!({"index":v.index,"id":v.id,"expected":format!("{:?}",v.expected),"actual":format!("{:?}",v.actual),"matched":v.matched})).collect::<Vec<_>>()})
        }
        Some(Err(error)) => json!({"malformed":format!("{error:?}")}),
        None => json!({"unavailable":"no complete normal driver execution"}),
    }
}
fn readbacks(
    evidence: &Evidence<'_>,
    frozen: &Frozen,
    run: &workload::Run,
    deadline: Instant,
) -> (Vec<d::IdentityFact>, Vec<Value>) {
    let mut facts = Vec::new();
    let mut rows = Vec::new();
    for source in &frozen.sources {
        let read = subjects::verify(
            evidence,
            &source.snapshot,
            source.origin,
            &source.reference,
            deadline,
        );
        facts.push(d::IdentityFact {
            subject: source.identity,
            state: if read.is_ok() {
                d::IdentityState::Matched
            } else {
                d::IdentityState::Changed
            },
        });
        rows.push(json!({"identity":format!("{:?}",source.identity),"reference":source.reference,"matched":read.is_ok(),"failure":read.err().map(|e|format!("{e:?}"))}));
    }
    for (reference, subject) in preparation::support_refs(&frozen.prepared)
        .into_iter()
        .zip([
            d::Identity::Locks,
            d::Identity::Toolchain,
            d::Identity::Profile,
            d::Identity::Standards,
        ])
    {
        let ok = graph::Graph::resolve(evidence, reference).is_ok();
        facts.push(d::IdentityFact {
            subject,
            state: if ok {
                d::IdentityState::Matched
            } else {
                d::IdentityState::Unavailable
            },
        });
        rows.push(json!({"identity":format!("{subject:?}"),"reference":reference,"matched":ok,"scope":"immutable prepared object closure; actual launcher tool pins remain separate captured facts"}));
    }
    let protected_ok = frozen.protected.readback_source(deadline).is_ok();
    if !protected_ok || !run.subjects_unchanged {
        for fact in &mut facts {
            if matches!(fact.subject, d::Identity::Oracle | d::Identity::Result) {
                fact.state = d::IdentityState::Changed;
            }
        }
    }
    rows.push(json!({"protected_snapshot_matched":protected_ok,"workload_subjects_unchanged":run.subjects_unchanged}));
    (facts, rows)
}
fn timing(
    frozen: &Frozen,
    reports: &[&NamespaceReport],
    oracle: &OracleObservation,
    clock: &Clock,
) -> Result<(d::Timing, bool), Error> {
    if clock.observed_at > Instant::now()
        || oracle.checked_at > clock.observed_at
        || oracle.terminal_end > oracle.checked_at
        || clock.end_unix_ms < clock.start_unix_ms
        || elapsed(clock.origin, clock.candidate_deadline)? != frozen.recipe.candidate_cutoff_ms
        || elapsed(clock.origin, clock.work_deadline)? != frozen.recipe.verification_cutoff_ms
        || elapsed(clock.origin, clock.cleanup_deadline)? != frozen.recipe.total_wall_ms
    {
        return Err(fail(ErrorKind::Binding));
    }
    let mut timeout = None;
    let mut cancel = clock.cancellation_observed_at;
    let mut candidate_bound = true;
    let mut previous = clock.origin;
    for report in reports {
        let process = &report.process;
        let end = process
            .started_at
            .checked_add(process.elapsed)
            .ok_or_else(|| fail(ErrorKind::Binding))?;
        if process.started_at < previous || end > oracle.checked_at {
            return Err(fail(ErrorKind::Binding));
        }
        previous = end;
        if let Some(intent) = process.interruption_observed_at {
            if intent < process.started_at || intent > end {
                return Err(fail(ErrorKind::Binding));
            }
            match process.interruption {
                Some(Interruption::Timeout) => {
                    timeout = Some(timeout.map_or(intent, |prior: Instant| prior.min(intent)));
                }
                Some(Interruption::Cancelled) => {
                    cancel = Some(cancel.map_or(intent, |prior| prior.min(intent)));
                }
                _ => {}
            }
        }
        candidate_bound &= process.started_at <= clock.candidate_deadline
            && (end <= clock.candidate_deadline
                || matches!(
                    process.interruption,
                    Some(Interruption::Timeout | Interruption::Cancelled)
                ));
    }
    if cancel.is_some_and(|at| at > clock.observed_at)
        || timeout.is_some_and(|at| at > clock.observed_at)
    {
        return Err(fail(ErrorKind::Binding));
    }
    Ok((
        d::Timing {
            work_deadline_ms: if timeout.is_some() {
                frozen.recipe.candidate_cutoff_ms
            } else {
                frozen.recipe.verification_cutoff_ms
            },
            cleanup_deadline_ms: frozen.recipe.total_wall_ms,
            observed_ms: elapsed(clock.origin, clock.observed_at)?,
            decisive_ms: oracle
                .result
                .as_ref()
                .filter(|r| r.is_ok())
                .map(|_| elapsed(clock.origin, oracle.checked_at))
                .transpose()?,
            timeout_intent_ms: timeout.map(|at| elapsed(clock.origin, at)).transpose()?,
            cancellation_intent_ms: cancel.map(|at| elapsed(clock.origin, at)).transpose()?,
        },
        candidate_bound,
    ))
}
struct Measurement {
    captures: Vec<capture::Captured>,
    artifacts: Vec<r::ArtifactV1>,
    diagnostics: Vec<r::DiagnosticV1>,
    clean: bool,
    stdout: bool,
    stderr: bool,
}
fn capture_all(
    evidence: &mut Evidence<'_>,
    reports: &[&NamespaceReport],
) -> Result<Measurement, Error> {
    let mut value = Measurement {
        captures: Vec::new(),
        artifacts: Vec::new(),
        diagnostics: Vec::new(),
        clean: true,
        stdout: true,
        stderr: true,
    };
    for (report, label) in reports.iter().zip(LABELS) {
        let captured = capture::capture(evidence, report)
            .map_err(|e| fail(ErrorKind::Capture(Box::new(e))))?;
        let f = &captured.flags;
        value.stdout &=
            f.candidate_stdout_complete && f.launcher_stdout_complete && f.native_stream_complete;
        value.stderr &= f.candidate_stderr_complete && f.launcher_stderr_complete;
        let empty = f.candidate_stderr_complete
            && f.launcher_stderr_complete
            && !f.candidate_stderr_nonempty
            && !f.launcher_stderr_nonempty
            && (label == "execute-driver" || report.observer.candidate_stdout.bytes.is_empty());
        value.clean &= empty;
        if empty {
            value.diagnostics.push(r::DiagnosticV1 {
                tool_id: name(label)?,
                baseline: true,
                warning_count: 0,
                error_count: 0,
                stdout: captured.payloads.candidate_stdout.clone(),
                stderr: captured.payloads.candidate_stderr.clone(),
                stdout_truncated: !f.candidate_stdout_complete,
                stderr_truncated: false,
                producer: captured.producer.clone(),
            });
        }
        value.artifacts.extend(captured.artifacts.clone());
        value.artifacts.push(artifact(
            captured.producer_ref.as_ref(),
            "observed_native_producer",
        )?);
        value.captures.push(captured);
    }
    if !value.clean {
        value.diagnostics.clear();
    }
    Ok(value)
}

struct Context {
    timing: d::Timing,
    identities: Vec<d::IdentityFact>,
    readbacks: Vec<Value>,
    candidate_bound: bool,
    bounded_profile: bool,
    measured: Measurement,
}
fn context(
    evidence: &mut Evidence<'_>,
    frozen: &Frozen,
    run: &workload::Run,
    oracle: &OracleObservation,
    clock: &Clock,
    deadline: Instant,
) -> Result<Context, Error> {
    budget(deadline)?;
    let reports = reports(run)?;
    let last = *reports
        .last()
        .ok_or_else(|| fail(ErrorKind::UnsupportedRun))?;
    if oracle.stdout_sha256 != hash(&last.observer.candidate_stdout.bytes)
        || oracle.terminal_end
            != last
                .process
                .started_at
                .checked_add(last.process.elapsed)
                .ok_or_else(|| fail(ErrorKind::Binding))?
    {
        return Err(fail(ErrorKind::Binding));
    }
    let (timing, candidate_bound) = timing(frozen, &reports, oracle, clock)?;
    let (mut identities, mut readbacks) = readbacks(evidence, frozen, run, deadline);
    let bounded_profile = reports.iter().all(|r| {
        r.observer.resource_scope().is_some()
            && r.observer
                .scratch_facts()
                .is_some_and(|f| f.capacity_bytes == 4_294_967_296)
            && r.observer.facts.protection_verified
            && r.observer.facts.descriptors_verified
    });
    if !candidate_bound || !bounded_profile {
        identities
            .iter_mut()
            .filter(|f| f.subject == d::Identity::Profile)
            .for_each(|f| f.state = d::IdentityState::Changed);
    }
    let mut measured = capture_all(evidence, &reports)?;
    let (input_binding, output_artifacts) =
        super::binding::capture_outputs(evidence, frozen, run, &reports, deadline)?;
    if !input_binding {
        for fact in &mut identities {
            if matches!(
                fact.subject,
                d::Identity::Result | d::Identity::Toolchain | d::Identity::Harness
            ) {
                fact.state = d::IdentityState::Changed;
            }
        }
    }
    measured.artifacts.extend(output_artifacts);
    readbacks.push(json!({"actual_namespace_inputs_and_compiled_outputs_bound":input_binding}));
    Ok(Context {
        timing,
        identities,
        readbacks,
        candidate_bound,
        bounded_profile,
        measured,
    })
}
fn cleanup_facts(
    run: &workload::Run,
    reports: &[&NamespaceReport],
    cleanup: &Cleanup,
) -> d::CleanupFacts {
    let settled_process = run.process_cleanup_complete
        && reports.iter().all(|r| {
            r.process.pending.is_none()
                && r.process.leader_reaped
                && r.observer.facts.pid1_terminal
                && r.observer.facts.shim_terminal
                && r.observer.facts.channel_cleanup_complete
        });
    let obligations = if cleanup
        .unresolved
        .iter()
        .any(|o| o.material && o.state != r::ObligationV1State::Settled)
    {
        d::Settlement::Pending
    } else {
        cleanup.obligations
    };
    d::CleanupFacts {
        descendants: settled(settled_process),
        resources: if run.scratch_released {
            cleanup.aggregate
        } else {
            d::Settlement::Pending
        },
        obligations,
    }
}
struct Assessment {
    producer: r::ProducerV1,
    checker: d::CheckerFact,
    cause: d::IncompleteCause,
    oracle_fact: d::OracleFact,
    outcome: r::CaseV1Outcome,
    cleanup_facts: d::CleanupFacts,
    decision: d::Decision,
}
fn assess(
    evidence: &Evidence<'_>,
    frozen: &Frozen,
    run: &workload::Run,
    oracle: &OracleObservation,
    cleanup: &Cleanup,
    context: &Context,
) -> Result<Assessment, Error> {
    let reports = reports(run)?;
    let last = *reports
        .last()
        .ok_or_else(|| fail(ErrorKind::UnsupportedRun))?;
    let capture = context
        .measured
        .captures
        .last()
        .ok_or_else(|| fail(ErrorKind::UnsupportedRun))?;
    let producer = capture.producer.clone();
    let termination = match last.process.interruption {
        Some(Interruption::Timeout) => d::Termination::Deadline,
        Some(Interruption::Cancelled) => d::Termination::Cancellation,
        Some(_) => d::Termination::Unknown,
        None => d::Termination::Ordinary,
    };
    let cause = if oracle.result.is_some() {
        d::IncompleteCause::Unexplained
    } else {
        match termination {
            d::Termination::Deadline => d::IncompleteCause::Deadline,
            d::Termination::Cancellation => d::IncompleteCause::Cancellation,
            _ if reports.iter().any(|r| !normal(r)) => d::IncompleteCause::ProducerFailure,
            _ => d::IncompleteCause::Unexplained,
        }
    };
    let (oracle_fact, outcome) = match &oracle.result {
        Some(Ok(e)) if e.failed == 0 => (d::OracleFact::Satisfied, r::CaseV1Outcome::Passed),
        Some(Ok(_)) => (d::OracleFact::Mismatch, r::CaseV1Outcome::Failed),
        Some(Err(_)) => (d::OracleFact::Malformed, r::CaseV1Outcome::Invalid),
        None if cause == d::IncompleteCause::Deadline => {
            (d::OracleFact::Unavailable, r::CaseV1Outcome::Timeout)
        }
        None => (d::OracleFact::Unavailable, r::CaseV1Outcome::Unmeasured),
    };
    let plan = &frozen.prepared.cases[0];
    let plans = [d::CasePlan {
        case_id: plan.case_id.clone(),
        selection: d::Selection::Required,
        expected_producer: frozen.expectation.expected_producer.clone(),
        design: d::Design::Reviewed,
    }];
    let cases = [d::CaseObservation {
        case_id: plan.case_id.clone(),
        executed: reports.len() == 3,
        outcome,
        incomplete_cause: cause,
        producer: state(&producer),
    }];
    let producer_fact = d::ProcessFact {
        expected: frozen.expectation.expected_producer.clone(),
        actual: state(&producer),
        termination,
    };
    let checker = if oracle.result.is_some() {
        d::CheckerFact::InProcessComplete
    } else {
        d::CheckerFact::NotStarted
    };
    let cleanup_facts = cleanup_facts(run, &reports, cleanup);
    if bytes(evidence, cleanup.evidence.as_ref())?.is_empty() {
        return Err(fail(ErrorKind::Binding));
    }
    let input = d::Input {
        identities: &context.identities,
        plans: &plans,
        cases: &cases,
        producer: &producer_fact,
        checker: &checker,
        oracle: oracle_fact,
        oracle_unavailable_cause: cause,
        logs: d::Streams {
            stdout: log(context.measured.stdout),
            stderr: log(context.measured.stderr),
        },
        diagnostics: d::Diagnostics {
            policy: d::DiagnosticPolicy::CleanBaseline,
            state: if context.measured.clean {
                d::DiagnosticState::Complete {
                    warnings: 0,
                    errors: 0,
                }
            } else {
                d::DiagnosticState::Unavailable
            },
        },
        cleanup: cleanup_facts,
        evidence: d::EvidenceState::FinalizedComplete,
        timing: context.timing,
    };
    let decision = d::decide(&input);
    Ok(Assessment {
        producer,
        checker,
        cause,
        oracle_fact,
        outcome,
        cleanup_facts,
        decision,
    })
}
struct Proofs {
    oracle_observation: r::Payload,
    decision_inputs: r::Payload,
    oracle_result: r::TypedRef<r::OracleResultV1>,
}
fn proofs(
    evidence: &mut Evidence<'_>,
    frozen: &Frozen,
    oracle: &OracleObservation,
    clock: &Clock,
    cleanup: &Cleanup,
    context: &Context,
    assessment: &Assessment,
) -> Result<Proofs, Error> {
    let capture = context
        .measured
        .captures
        .last()
        .ok_or_else(|| fail(ErrorKind::UnsupportedRun))?;
    let readbacks = &context.readbacks;
    let candidate_bound = context.candidate_bound;
    let bounded_profile = context.bounded_profile;
    let measured = &context.measured;
    let timing = context.timing;
    let producer = &assessment.producer;
    let checker = &assessment.checker;
    let cause = assessment.cause;
    let oracle_fact = assessment.oracle_fact;
    let cleanup_facts = assessment.cleanup_facts;
    let decision = &assessment.decision;
    let oracle_observation = json_raw(
        evidence,
        &json!({"workload":"WL-U64-PARSE-001/v1","checker":"actual trusted coordinator-local FrozenOracle evaluation; no checker process exit exists","checked_at_offset_ms":elapsed(clock.origin,oracle.checked_at)?,"terminal_report_end_offset_ms":elapsed(clock.origin,oracle.terminal_end)?,"terminal_time_scope":"conservative process report sample, not exact native exit timestamp","stdout_sha256":oracle.stdout_sha256,"oracle_sha256":u64_oracle::ORACLE_SHA256,"vector_count_is_not_module_credit":true,"observation":oracle_value(oracle)}),
    )?;
    let decision_inputs = json_raw(
        evidence,
        &json!({"identity_readback":readbacks,"candidate_bound_observed":candidate_bound,"bounded_namespace_resource_profile_observed":bounded_profile,"recipe":frozen.recipe,"recipe_reference":frozen.recipe_ref,"producer":producer,"checker":format!("{checker:?}"),"oracle":format!("{oracle_fact:?}"),"cause":format!("{cause:?}"),"clock":format!("{timing:?}"),"selected_deadline_phase":if timing.timeout_intent_ms.is_some(){"candidate timeout"}else{"coordinator verification"},"cleanup":format!("{cleanup_facts:?}"),"cleanup_evidence":cleanup.evidence,"diagnostics":{"strict_empty_baseline_measured":measured.clean,"stdout_complete":measured.stdout,"stderr_complete":measured.stderr},"derived_state":decision.state(),"reasons":format!("{:?}",decision.reasons()),"detectors":format!("{:?}",decision.detectors())}),
    )?;
    let oracle_result = record(
        evidence,
        &r::OracleResultV1 {
            oracle_id: frozen.expectation.oracle_id.clone(),
            expected: frozen.prepared.invocation.expected.clone(),
            result: match oracle_fact {
                d::OracleFact::Satisfied => r::OracleResultV1Result::Satisfied,
                d::OracleFact::Mismatch => r::OracleResultV1Result::Violated,
                d::OracleFact::Unavailable => r::OracleResultV1Result::Unavailable,
                _ => r::OracleResultV1Result::Error,
            },
            detector_id: if oracle.result.is_some() {
                r::Maybe::present(frozen.expectation.intended_detector.clone())
            } else {
                none("checker_not_started")?
            },
            raw_evidence_refs: list(vec![
                oracle_observation.as_ref().clone(),
                capture.payloads.candidate_stdout.as_ref().clone(),
                capture.facts_payload.as_ref().clone(),
                frozen.expectation.specification.as_ref().clone(),
            ])?,
            reason: text(
                "Exact retained stdout evaluated in process against independently frozen 335-vector expectations; finite agreement is not execution authentication or module admission.",
            )?,
        },
    )?;
    Ok(Proofs {
        oracle_observation,
        decision_inputs,
        oracle_result,
    })
}
struct Pages {
    resources: r::TypedRef<r::ResourcePageV1>,
    obligations: r::TypedRef<r::ObligationPageV1>,
    missing: r::TypedRef<r::MissingObjectPageV1>,
}
fn pages(evidence: &mut Evidence<'_>, cleanup: &Cleanup) -> Result<Pages, Error> {
    let (resources, obligations, missing) = {
        let mut publisher = c::Publisher::new(evidence);
        let result = (|| {
            Ok((
                publisher.resource_pages(&[])?,
                publisher.obligation_pages(&cleanup.unresolved)?,
                publisher.missing_object_pages(&[])?,
            ))
        })();
        result.map_err(|e| Error {
            kind: ErrorKind::Publication(e),
            attempted_refs: publisher.attempted_refs().to_vec(),
        })?
    };
    Ok(Pages {
        resources,
        obligations,
        missing,
    })
}
fn case_row(
    frozen: &Frozen,
    executed: bool,
    producer: &r::ProducerV1,
    outcome: r::CaseV1Outcome,
    proof: &Proofs,
) -> Result<c::CaseObservation, Error> {
    let plan = &frozen.prepared.cases[0];
    let oracle_observation = &proof.oracle_observation;
    let decision_inputs = &proof.decision_inputs;
    let observation = c::CaseObservation {
        case_id: plan.case_id.clone(),
        executed,
        outcome,
        producer: producer.clone(),
        detector_id: frozen.expectation.intended_detector.clone(),
        benign_pair_id: none("single_workload_case")?,
        raw_evidence_refs: list(vec![
            oracle_observation.as_ref().clone(),
            decision_inputs.as_ref().clone(),
            plan.reviewed_design
                .as_ref()
                .ok_or_else(|| fail(ErrorKind::Review))?
                .as_ref()
                .clone(),
            frozen.provenance.as_ref().clone(),
        ])?,
        reason: if outcome == r::CaseV1Outcome::Passed {
            text("")?
        } else {
            text(
                "Trusted workload observation retained; see exact oracle, native status, timing and decision reasons.",
            )?
        },
    };
    Ok(observation)
}
fn observation_row(
    frozen: &Frozen,
    clock: &Clock,
    timing: d::Timing,
    producer: r::ProducerV1,
    pages: Pages,
    all_settled: bool,
) -> Result<r::ObservationsV1, Error> {
    let Pages {
        resources,
        obligations,
        missing: _,
    } = pages;
    Ok(r::ObservationsV1 {
        host: frozen.host.clone(),
        start_unix_ms: number(clock.start_unix_ms)?,
        end_unix_ms: number(clock.end_unix_ms)?,
        start_monotonic_ns: number(0)?,
        end_monotonic_ns: number(ns(clock.origin, clock.observed_at)?)?,
        cutoff_unix_ms: number(
            clock
                .start_unix_ms
                .checked_add(frozen.recipe.verification_cutoff_ms)
                .ok_or_else(|| fail(ErrorKind::Binding))?,
        )?,
        resources,
        producer,
        cancellation: if timing.cancellation_intent_ms.is_some() {
            r::ObservationsV1Cancellation::Requested
        } else {
            r::ObservationsV1Cancellation::NotRequested
        },
        cleanup: if all_settled {
            r::ObservationsV1Cleanup::Settled
        } else {
            r::ObservationsV1Cleanup::Pending
        },
        unresolved_obligations: obligations,
    })
}
fn assemble(
    evidence: &mut Evidence<'_>,
    frozen: &Frozen,
    clock: &Clock,
    cleanup: &Cleanup,
    parts: (Context, Assessment, Proofs),
    deadline: Instant,
) -> Result<Collected, Error> {
    let (context, assessment, proof) = parts;
    let mut measured = context.measured;
    let executed = measured.captures.len() == 3;
    let timing = context.timing;
    let producer = assessment.producer;
    let outcome = assessment.outcome;
    let cleanup_facts = assessment.cleanup_facts;
    let decision = assessment.decision;
    let oracle_observation = proof.oracle_observation.clone();
    let decision_inputs = proof.decision_inputs.clone();
    let oracle_result = proof.oracle_result.clone();
    for (reference, role) in [
        (oracle_observation.as_ref(), "u64_oracle_observation"),
        (decision_inputs.as_ref(), "decision_inputs"),
        (cleanup.evidence.as_ref(), "aggregate_cleanup"),
        (frozen.recipe_ref.as_ref(), "frozen_recipe"),
        (frozen.provenance.as_ref(), "review_provenance"),
    ] {
        measured.artifacts.push(artifact(reference, role)?);
    }
    measured.artifacts.extend(subject_artifacts(
        evidence,
        &frozen.prepared.subjects,
        deadline,
    )?);
    let pages = pages(evidence, cleanup)?;
    let missing = pages.missing.clone();
    let all_settled = cleanup_facts.descendants == d::Settlement::Settled
        && cleanup_facts.resources == d::Settlement::Settled
        && cleanup_facts.obligations == d::Settlement::Settled;
    let observation = case_row(frozen, executed, &producer, outcome, &proof)?;
    let observed = c::Observed {
        observations: observation_row(frozen, clock, timing, producer, pages, all_settled)?,
        cases: vec![observation],
        diagnostic_baseline: measured.clean,
        diagnostics: measured.diagnostics,
        diagnostic_mismatch: if measured.clean {
            none("strict_empty_diagnostics_observed")?
        } else {
            r::Maybe::present(text(
                "Nonempty or incomplete diagnostics: warning/error counts are unmeasured.",
            )?)
        },
        artifacts: measured.artifacts,
        artifacts_finalized: true,
        campaigns: Vec::new(),
        verdict: c::VerdictBinding {
            oracle_result,
            intended_detector: frozen.expectation.intended_detector.clone(),
            benign_pair: none("single_workload_case")?,
        },
        availability: r::AvailabilityV1 {
            observed_unix_ms: number(clock.end_unix_ms)?,
            state: r::AvailabilityV1State::Complete,
            missing_objects: missing,
            retention_policy: r::AvailabilityV1RetentionPolicy::RetainV1,
        },
    };
    budget(deadline)?;
    let mut publisher = c::Publisher::new(evidence);
    let receipt = publisher
        .finalize(&frozen.prepared, &decision, observed)
        .map_err(|e| Error {
            kind: ErrorKind::Publication(e),
            attempted_refs: publisher.attempted_refs().to_vec(),
        })?;
    Ok(Collected {
        receipt,
        decision,
        decision_inputs,
        oracle_observation,
    })
}
pub(super) fn finalize(
    evidence: &mut Evidence<'_>,
    frozen: &Frozen,
    run: &workload::Run,
    oracle: &OracleObservation,
    clock: &Clock,
    cleanup: &Cleanup,
    deadline: Instant,
) -> Result<Collected, Error> {
    let context = context(evidence, frozen, run, oracle, clock, deadline)?;
    let assessment = assess(evidence, frozen, run, oracle, cleanup, &context)?;
    let proof = proofs(
        evidence,
        frozen,
        oracle,
        clock,
        cleanup,
        &context,
        &assessment,
    )?;
    assemble(
        evidence,
        frozen,
        clock,
        cleanup,
        (context, assessment, proof),
        deadline,
    )
}
