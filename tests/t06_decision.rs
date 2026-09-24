//! Independent supplied-fact RC04 decision oracles. Design preceded bodies.
//! No decision implementation was read. Reviewed/matched/settled values below
//! are fixture inputs, not authenticated process, custody or admission evidence.

use habitat_engine::check::decision::{
    CaseObservation, CasePlan, CheckerFact, CleanupFacts, Counts, Decision, Design, Detector,
    DiagnosticPolicy, DiagnosticState, Diagnostics, EvidenceState, Identity, IdentityFact,
    IdentityState, IncompleteCause, Input, LogState, OracleFact, ProcessFact, ProcessState,
    ReasonKind, Selection, Settlement, Streams, Termination, Timing, decide,
};
use habitat_engine::contracts::receipt::{
    CaseV1Outcome, ExpectedProducerV1, Maybe, Name, VerdictV1State,
};

const IDENTITIES: [Identity; 11] = [
    Identity::Seed,
    Identity::Result,
    Identity::Fixtures,
    Identity::Oracle,
    Identity::Harness,
    Identity::Collector,
    Identity::Launcher,
    Identity::Locks,
    Identity::Toolchain,
    Identity::Profile,
    Identity::Standards,
];

fn expected_exit(code: u32) -> ExpectedProducerV1 {
    serde_json::from_value(serde_json::json!({"status":"exited","exit_code":{"value":code,"unavailable_reason":null},"signal":{"value":null,"unavailable_reason":"ordinary exit"}})).unwrap()
}
fn expected_signal(signal: u32) -> ExpectedProducerV1 {
    serde_json::from_value(serde_json::json!({"status":"signalled","exit_code":{"value":null,"unavailable_reason":"native signal"},"signal":{"value":signal,"unavailable_reason":null}})).unwrap()
}
fn name(index: usize) -> Name {
    Name::new(format!("case-{index:04}")).unwrap()
}

#[derive(Clone)]
struct Fixture {
    identities: Vec<IdentityFact>,
    plans: Vec<CasePlan>,
    cases: Vec<CaseObservation>,
    producer: ProcessFact,
    checker: CheckerFact,
    oracle: OracleFact,
    oracle_unavailable_cause: IncompleteCause,
    logs: Streams,
    diagnostics: Diagnostics,
    cleanup: CleanupFacts,
    evidence: EvidenceState,
    timing: Timing,
}
impl Fixture {
    fn new() -> Self {
        Self {
            identities: IDENTITIES
                .map(|subject| IdentityFact {
                    subject,
                    state: IdentityState::Matched,
                })
                .to_vec(),
            plans: vec![CasePlan {
                case_id: name(0),
                selection: Selection::Required,
                expected_producer: expected_exit(0),
                design: Design::Reviewed,
            }],
            cases: vec![CaseObservation {
                case_id: name(0),
                executed: true,
                outcome: CaseV1Outcome::Passed,
                producer: ProcessState::Exited(0),
                incomplete_cause: IncompleteCause::Unexplained,
            }],
            producer: ProcessFact {
                expected: expected_exit(0),
                actual: ProcessState::Exited(0),
                termination: Termination::Ordinary,
            },
            checker: CheckerFact::InProcessComplete,
            oracle: OracleFact::Satisfied,
            oracle_unavailable_cause: IncompleteCause::Unexplained,
            logs: Streams {
                stdout: LogState::Complete,
                stderr: LogState::Complete,
            },
            diagnostics: Diagnostics {
                policy: DiagnosticPolicy::CleanBaseline,
                state: DiagnosticState::Complete {
                    warnings: 0,
                    errors: 0,
                },
            },
            cleanup: CleanupFacts {
                descendants: Settlement::Settled,
                resources: Settlement::Settled,
                obligations: Settlement::Settled,
            },
            evidence: EvidenceState::FinalizedComplete,
            timing: Timing {
                work_deadline_ms: 100,
                cleanup_deadline_ms: 200,
                observed_ms: 120,
                decisive_ms: Some(99),
                timeout_intent_ms: None,
                cancellation_intent_ms: None,
            },
        }
    }
    fn decision(&self) -> Decision {
        decide(&Input {
            identities: &self.identities,
            plans: &self.plans,
            cases: &self.cases,
            producer: &self.producer,
            checker: &self.checker,
            oracle: self.oracle,
            oracle_unavailable_cause: self.oracle_unavailable_cause,
            logs: self.logs,
            diagnostics: self.diagnostics,
            cleanup: self.cleanup,
            evidence: self.evidence,
            timing: self.timing,
        })
    }
    fn add(
        &mut self,
        selection: Selection,
        outcome: CaseV1Outcome,
        executed: bool,
        cause: IncompleteCause,
    ) {
        let case_id = name(self.plans.len());
        self.plans.push(CasePlan {
            case_id: case_id.clone(),
            selection,
            expected_producer: expected_exit(0),
            design: Design::Reviewed,
        });
        self.cases.push(CaseObservation {
            case_id,
            executed,
            outcome,
            producer: if executed {
                ProcessState::Exited(0)
            } else {
                ProcessState::NotStarted
            },
            incomplete_cause: cause,
        });
    }
    fn producer_failure() -> Self {
        let mut value = Self::new();
        value.producer.actual = ProcessState::Exited(7);
        value.checker = CheckerFact::NotStarted;
        value.oracle = OracleFact::Unavailable;
        value.oracle_unavailable_cause = IncompleteCause::ProducerFailure;
        value.timing.decisive_ms = None;
        value.timing.observed_ms = 50;
        value.cases[0].outcome = CaseV1Outcome::Unmeasured;
        value.cases[0].producer = ProcessState::Exited(7);
        value.cases[0].incomplete_cause = IncompleteCause::ProducerFailure;
        value
    }
    fn timeout() -> Self {
        let mut value = Self::new();
        value.producer.actual = ProcessState::Signalled(9);
        value.producer.termination = Termination::Deadline;
        value.checker = CheckerFact::NotStarted;
        value.oracle = OracleFact::Unavailable;
        value.oracle_unavailable_cause = IncompleteCause::Deadline;
        value.timing.decisive_ms = None;
        value.timing.timeout_intent_ms = Some(100);
        value.timing.observed_ms = 150;
        value.cases[0].outcome = CaseV1Outcome::Timeout;
        value.cases[0].producer = ProcessState::Signalled(9);
        value.cases[0].incomplete_cause = IncompleteCause::Deadline;
        value
    }
}
fn reason(decision: &Decision, kind: ReasonKind) {
    assert!(
        decision.reasons().iter().any(|item| item.kind == kind),
        "missing {kind:?}: {:?}",
        decision.reasons()
    );
}
fn no_reason(decision: &Decision, kind: ReasonKind) {
    assert!(
        decision.reasons().iter().all(|item| item.kind != kind),
        "unexpected {kind:?}: {:?}",
        decision.reasons()
    );
}
fn state(decision: &Decision, expected: VerdictV1State) {
    assert_eq!(
        decision.state(),
        expected,
        "reasons: {:?}",
        decision.reasons()
    );
}

#[test]
fn clean_decisive_result_may_be_captured_later_inside_cleanup_reserve() {
    let decision = Fixture::new().decision();
    state(&decision, VerdictV1State::PassCandidate);
    assert!(decision.reasons().is_empty());
    assert!(decision.detectors().is_empty());
    assert_eq!(
        decision.counts(),
        Counts {
            discovered: 1,
            selected: 1,
            mandatory: 1,
            executed: 1,
            passed: 1,
            ..Counts::default()
        }
    );
}

#[test]
fn exact_frozen_nonzero_exit_and_native_signal_can_qualify() {
    for actual in [ProcessState::Exited(7), ProcessState::Signalled(6)] {
        let mut fixture = Fixture::new();
        let expected = match actual {
            ProcessState::Exited(code) => expected_exit(code),
            ProcessState::Signalled(signal) => expected_signal(signal),
            _ => unreachable!(),
        };
        fixture.producer.expected = expected.clone();
        fixture.producer.actual = actual;
        fixture.plans[0].expected_producer = expected;
        fixture.cases[0].producer = actual;
        state(&fixture.decision(), VerdictV1State::PassCandidate);
        fixture.oracle = OracleFact::Mismatch;
        state(&fixture.decision(), VerdictV1State::Fail);
        let mut checker = Fixture::new();
        let expected = match actual {
            ProcessState::Exited(code) => expected_exit(code),
            ProcessState::Signalled(signal) => expected_signal(signal),
            _ => unreachable!(),
        };
        checker.checker = CheckerFact::Process(ProcessFact {
            expected,
            actual,
            termination: Termination::Ordinary,
        });
        state(&checker.decision(), VerdictV1State::PassCandidate);
    }
}

#[test]
fn observed_unexpected_status_cannot_rewrite_the_frozen_ordinary_expectation() {
    for (actual, kind) in [
        (ProcessState::Exited(7), ReasonKind::ProducerUnexpectedExit),
        (
            ProcessState::Signalled(6),
            ReasonKind::ProducerUnexpectedSignal,
        ),
    ] {
        let mut fixture = Fixture::new();
        fixture.producer.actual = actual;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Error);
        reason(&decision, kind);
    }
    for (actual, kind) in [
        (ProcessState::NotStarted, ReasonKind::ProducerNotStarted),
        (ProcessState::Unknown, ReasonKind::ProducerUnknown),
    ] {
        let mut fixture = Fixture::new();
        fixture.producer.actual = actual;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, kind);
    }
}

#[test]
fn malformed_expected_producer_record_is_invalid_even_when_actual_exit_is_zero() {
    let mut fixture = Fixture::new();
    fixture.producer.expected.signal = Maybe::present(6);
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::InvalidExpectation);
    let mut fixture = Fixture::new();
    fixture.plans[0].expected_producer.signal = Maybe::present(6);
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::InvalidExpectation);
}

#[test]
fn every_required_identity_kind_is_required_exactly_once() {
    for missing in IDENTITIES {
        let mut fixture = Fixture::new();
        fixture.identities.retain(|fact| fact.subject != missing);
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        assert!(
            decision
                .reasons()
                .iter()
                .any(|item| item.kind == ReasonKind::MissingIdentity
                    && item.identity == Some(missing))
        );
    }
}

#[test]
fn repeated_matched_identity_is_not_additional_identity_proof() {
    let mut fixture = Fixture::new();
    fixture.identities.push(fixture.identities[0]);
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::Bound);
    fixture.identities.pop();
    fixture.identities[1] = fixture.identities[0];
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::DuplicateIdentity);
    reason(&decision, ReasonKind::MissingIdentity);
}

#[test]
fn changed_and_unavailable_identities_name_the_exact_failed_subject() {
    for subject in IDENTITIES {
        for (observed, kind) in [
            (IdentityState::Changed, ReasonKind::ChangedIdentity),
            (IdentityState::Unavailable, ReasonKind::UnavailableIdentity),
        ] {
            let mut fixture = Fixture::new();
            fixture
                .identities
                .iter_mut()
                .find(|fact| fact.subject == subject)
                .unwrap()
                .state = observed;
            let decision = fixture.decision();
            state(&decision, VerdictV1State::Invalid);
            assert!(
                decision
                    .reasons()
                    .iter()
                    .any(|item| item.kind == kind && item.identity == Some(subject))
            );
        }
    }
}

#[test]
fn independent_identity_failure_takes_invalid_priority_but_retains_process_crash() {
    let mut fixture = Fixture::producer_failure();
    fixture.identities[0].state = IdentityState::Changed;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::ChangedIdentity);
    reason(&decision, ReasonKind::ProducerUnexpectedExit);
    reason(&decision, ReasonKind::OracleUnavailable);
}

#[test]
fn empty_plan_is_invalid_instead_of_unmeasured_success() {
    let mut fixture = Fixture::new();
    fixture.plans.clear();
    fixture.cases.clear();
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::EmptyPlan);
    assert_eq!(decision.counts().discovered, 0);
}

#[test]
fn valid_zero_selection_and_selected_but_no_mandatory_both_refuse() {
    let mut fixture = Fixture::new();
    fixture.plans[0].selection = Selection::Unselected;
    fixture.cases[0].executed = false;
    fixture.cases[0].outcome = CaseV1Outcome::Unmeasured;
    fixture.cases[0].producer = ProcessState::NotStarted;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::EmptySelection);
    assert_eq!(
        (
            decision.counts().discovered,
            decision.counts().selected,
            decision.counts().executed
        ),
        (1, 0, 0)
    );
    let mut fixture = Fixture::new();
    fixture.plans[0].selection = Selection::Optional;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::NoMandatory);
}

#[test]
fn required_case_design_cannot_be_inferred_reviewed_from_a_passing_result() {
    let mut fixture = Fixture::new();
    fixture.plans[0].design = Design::Unreviewed;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::UnreviewedRequired);
}

#[test]
fn observed_cases_must_form_an_exact_bijection_with_frozen_plans() {
    let mut missing = Fixture::new();
    missing.cases.clear();
    let decision = missing.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::MissingObservation);
    let mut extra = Fixture::new();
    let mut observation = extra.cases[0].clone();
    observation.case_id = Name::new("unplanned").unwrap();
    extra.cases.push(observation);
    let decision = extra.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::UnexpectedObservation);
    let mut duplicate = Fixture::new();
    duplicate.cases.push(duplicate.cases[0].clone());
    let decision = duplicate.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::DuplicateObservation);
}

#[test]
fn duplicate_plan_ids_cannot_double_count_a_single_observation() {
    let mut fixture = Fixture::new();
    fixture.plans.push(fixture.plans[0].clone());
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::DuplicatePlan);
}

#[test]
fn a_passed_label_without_execution_does_not_satisfy_required_coverage() {
    let mut fixture = Fixture::new();
    fixture.cases[0].executed = false;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::RequiredNotExecuted);
    assert_eq!(decision.counts().executed, 0);
}

#[test]
fn mandatory_nondecisive_outcomes_are_not_successful_negative_controls() {
    for (outcome, kind) in [
        (CaseV1Outcome::Skipped, ReasonKind::RequiredSkipped),
        (CaseV1Outcome::Ignored, ReasonKind::RequiredIgnored),
        (CaseV1Outcome::Broken, ReasonKind::RequiredBroken),
        (CaseV1Outcome::Invalid, ReasonKind::RequiredInvalid),
        (CaseV1Outcome::Unmeasured, ReasonKind::RequiredUnmeasured),
    ] {
        let mut fixture = Fixture::new();
        fixture.cases[0].outcome = outcome;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, kind);
    }
}

#[test]
fn valid_case_and_oracle_mismatches_are_failures_with_both_reasons_retained() {
    let mut fixture = Fixture::new();
    fixture.cases[0].outcome = CaseV1Outcome::Failed;
    fixture.oracle = OracleFact::Mismatch;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Fail);
    reason(&decision, ReasonKind::CaseMismatch);
    reason(&decision, ReasonKind::OracleMismatch);
}

#[test]
fn unselected_and_excluded_inventory_is_retained_without_blanket_failure() {
    let mut fixture = Fixture::new();
    fixture.add(
        Selection::Unselected,
        CaseV1Outcome::Unmeasured,
        false,
        IncompleteCause::Unexplained,
    );
    fixture.add(
        Selection::Excluded,
        CaseV1Outcome::Unmeasured,
        false,
        IncompleteCause::Unexplained,
    );
    let decision = fixture.decision();
    state(&decision, VerdictV1State::PassCandidate);
    assert_eq!(
        decision.counts(),
        Counts {
            discovered: 3,
            selected: 1,
            mandatory: 1,
            executed: 1,
            passed: 1,
            unmeasured: 1,
            excluded: 1,
            ..Counts::default()
        }
    );
}

#[test]
fn unselected_or_excluded_rows_cannot_execute_or_claim_decisive_outcomes() {
    for selection in [Selection::Unselected, Selection::Excluded] {
        let mut fixture = Fixture::new();
        fixture.add(
            selection,
            CaseV1Outcome::Passed,
            true,
            IncompleteCause::Unexplained,
        );
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::InvalidAccounting);
        fixture.cases[1].executed = false;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::InvalidAccounting);
    }
}

#[test]
fn an_actually_selected_optional_mismatch_still_contributes_fail() {
    let mut fixture = Fixture::new();
    fixture.add(
        Selection::Optional,
        CaseV1Outcome::Failed,
        true,
        IncompleteCause::Unexplained,
    );
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Fail);
    reason(&decision, ReasonKind::CaseMismatch);
    assert_eq!(
        (
            decision.counts().mandatory,
            decision.counts().selected,
            decision.counts().failed
        ),
        (1, 2, 1)
    );
}

#[test]
fn a_passed_case_with_wrong_frozen_producer_status_is_error() {
    let mut fixture = Fixture::new();
    fixture.cases[0].producer = ProcessState::Exited(4);
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Error);
    reason(&decision, ReasonKind::CaseUnexpectedStatus);
    fixture.plans[0].expected_producer = expected_exit(4);
    state(&fixture.decision(), VerdictV1State::PassCandidate);
}

#[test]
fn complete_disposition_counts_partition_all_rows_without_excluded_double_counting() {
    let mut fixture = Fixture::new();
    for outcome in [
        CaseV1Outcome::Failed,
        CaseV1Outcome::Skipped,
        CaseV1Outcome::Ignored,
        CaseV1Outcome::Broken,
        CaseV1Outcome::Invalid,
        CaseV1Outcome::Unmeasured,
    ] {
        fixture.add(
            Selection::Optional,
            outcome,
            true,
            IncompleteCause::Unexplained,
        );
    }
    fixture.add(
        Selection::Optional,
        CaseV1Outcome::Timeout,
        true,
        IncompleteCause::Deadline,
    );
    fixture.add(
        Selection::Excluded,
        CaseV1Outcome::Unmeasured,
        false,
        IncompleteCause::Unexplained,
    );
    fixture.timing.timeout_intent_ms = Some(100);
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Timeout);
    assert_eq!(
        decision.counts(),
        Counts {
            discovered: 9,
            selected: 8,
            mandatory: 1,
            executed: 8,
            passed: 1,
            failed: 1,
            skipped: 1,
            ignored: 1,
            broken: 1,
            timed_out: 1,
            invalid: 1,
            excluded: 1,
            unmeasured: 1
        }
    );
}

#[test]
fn inventory_bound_is_inclusive_at_4096_and_refuses_4097_in_either_slice() {
    let mut fixture = Fixture::new();
    for _ in 1..4096 {
        fixture.add(
            Selection::Required,
            CaseV1Outcome::Passed,
            true,
            IncompleteCause::Unexplained,
        );
    }
    let decision = fixture.decision();
    state(&decision, VerdictV1State::PassCandidate);
    assert_eq!(
        (decision.counts().discovered, decision.counts().executed),
        (4096, 4096)
    );
    let mut plans = fixture.clone();
    plans.plans.push(CasePlan {
        case_id: name(4096),
        selection: Selection::Required,
        expected_producer: expected_exit(0),
        design: Design::Reviewed,
    });
    let decision = plans.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::Bound);
    let mut rows = fixture;
    let mut extra = rows.cases[0].clone();
    extra.case_id = name(4096);
    rows.cases.push(extra);
    let decision = rows.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::Bound);
}

#[test]
fn required_stdout_and_stderr_each_preserve_their_distinct_failure_reason() {
    for (log, stdout_reason, stderr_reason) in [
        (
            LogState::Truncated,
            ReasonKind::StdoutTruncated,
            ReasonKind::StderrTruncated,
        ),
        (
            LogState::Missing,
            ReasonKind::StdoutMissing,
            ReasonKind::StderrMissing,
        ),
        (
            LogState::ReadError,
            ReasonKind::StdoutReadError,
            ReasonKind::StderrReadError,
        ),
    ] {
        let mut fixture = Fixture::new();
        fixture.logs.stdout = log;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, stdout_reason);
        let mut fixture = Fixture::new();
        fixture.logs.stderr = log;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, stderr_reason);
    }
}

#[test]
fn expected_fault_diagnostics_require_a_frozen_policy_and_do_not_waive_oracle() {
    let mut fixture = Fixture::new();
    fixture.diagnostics.state = DiagnosticState::Complete {
        warnings: 2,
        errors: 1,
    };
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::BaselineWarnings);
    reason(&decision, ReasonKind::BaselineErrors);
    fixture.diagnostics.policy = DiagnosticPolicy::ExpectedFault;
    state(&fixture.decision(), VerdictV1State::PassCandidate);
    fixture.oracle = OracleFact::Mismatch;
    state(&fixture.decision(), VerdictV1State::Fail);
}

#[test]
fn unavailable_diagnostics_cannot_be_relabelled_zero_under_either_policy() {
    for policy in [
        DiagnosticPolicy::CleanBaseline,
        DiagnosticPolicy::ExpectedFault,
    ] {
        let mut fixture = Fixture::new();
        fixture.diagnostics = Diagnostics {
            policy,
            state: DiagnosticState::Unavailable,
        };
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::DiagnosticsUnavailable);
    }
}

#[test]
fn required_artifact_evidence_must_be_finalized_and_complete() {
    for (evidence, kind) in [
        (EvidenceState::Unfinalized, ReasonKind::EvidenceUnfinalized),
        (EvidenceState::Incomplete, ReasonKind::EvidenceIncomplete),
        (EvidenceState::Missing, ReasonKind::EvidenceMissing),
    ] {
        let mut fixture = Fixture::new();
        fixture.evidence = evidence;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, kind);
    }
}

#[test]
fn descendants_resources_and_obligations_are_independent_cleanup_gates() {
    for pending in [Settlement::Pending, Settlement::Failed, Settlement::Unknown] {
        let mut descendants = Fixture::new();
        descendants.cleanup.descendants = pending;
        let decision = descendants.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::DescendantsUnsettled);
        let mut resources = Fixture::new();
        resources.cleanup.resources = pending;
        let decision = resources.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::ResourcesUnsettled);
        let mut obligations = Fixture::new();
        obligations.cleanup.obligations = pending;
        let decision = obligations.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::ObligationsUnsettled);
    }
}

#[test]
fn malformed_missing_mismatching_and_in_process_error_oracles_are_distinct() {
    for (oracle, wanted, kind) in [
        (
            OracleFact::Mismatch,
            VerdictV1State::Fail,
            ReasonKind::OracleMismatch,
        ),
        (
            OracleFact::Malformed,
            VerdictV1State::Invalid,
            ReasonKind::OracleMalformed,
        ),
        (
            OracleFact::Unavailable,
            VerdictV1State::Invalid,
            ReasonKind::OracleUnavailable,
        ),
        (
            OracleFact::Error,
            VerdictV1State::Error,
            ReasonKind::OracleError,
        ),
    ] {
        let mut fixture = Fixture::new();
        fixture.oracle = oracle;
        if oracle != OracleFact::Mismatch {
            fixture.timing.decisive_ms = None;
            fixture.timing.observed_ms = 50;
        }
        let decision = fixture.decision();
        state(&decision, wanted);
        reason(&decision, kind);
        no_reason(&decision, ReasonKind::CheckerUnexpectedSignal);
    }
}

#[test]
fn reported_checker_process_failure_can_explain_only_its_derivative_gaps() {
    let mut fixture = Fixture::new();
    fixture.checker = CheckerFact::Process(ProcessFact {
        expected: expected_exit(0),
        actual: ProcessState::Signalled(6),
        termination: Termination::Ordinary,
    });
    fixture.oracle = OracleFact::Unavailable;
    fixture.oracle_unavailable_cause = IncompleteCause::CheckerFailure;
    fixture.timing.decisive_ms = None;
    fixture.timing.observed_ms = 50;
    fixture.cases[0].outcome = CaseV1Outcome::Unmeasured;
    fixture.cases[0].incomplete_cause = IncompleteCause::CheckerFailure;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Error);
    reason(&decision, ReasonKind::CheckerUnexpectedSignal);
    reason(&decision, ReasonKind::OracleUnavailable);
    fixture.oracle_unavailable_cause = IncompleteCause::Unexplained;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::CheckerUnexpectedSignal);
    for (checker, kind) in [
        (CheckerFact::NotStarted, ReasonKind::CheckerNotStarted),
        (CheckerFact::Unavailable, ReasonKind::CheckerUnavailable),
    ] {
        let mut fixture = Fixture::new();
        fixture.checker = checker;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, kind);
    }
}

#[test]
fn producer_failure_cannot_mask_an_unrelated_missing_oracle_or_case() {
    let fixture = Fixture::producer_failure();
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Error);
    reason(&decision, ReasonKind::ProducerUnexpectedExit);
    reason(&decision, ReasonKind::RequiredUnmeasured);
    let mut oracle = fixture.clone();
    oracle.oracle_unavailable_cause = IncompleteCause::Unexplained;
    state(&oracle.decision(), VerdictV1State::Invalid);
    let mut case = fixture;
    case.cases[0].incomplete_cause = IncompleteCause::Unexplained;
    state(&case.decision(), VerdictV1State::Invalid);
}

#[test]
fn explicit_incomplete_causes_must_have_independent_supporting_facts() {
    for cause in [
        IncompleteCause::ProducerFailure,
        IncompleteCause::CheckerFailure,
        IncompleteCause::Deadline,
        IncompleteCause::Cancellation,
    ] {
        let mut fixture = Fixture::new();
        fixture.cases[0].outcome = CaseV1Outcome::Unmeasured;
        fixture.cases[0].incomplete_cause = cause;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::UnboundCause);
        let mut fixture = Fixture::new();
        fixture.oracle = OracleFact::Unavailable;
        fixture.oracle_unavailable_cause = cause;
        fixture.timing.decisive_ms = None;
        fixture.timing.observed_ms = 50;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::UnboundCause);
    }
    let mut timeout = Fixture::timeout();
    timeout.cases[0].incomplete_cause = IncompleteCause::Unexplained;
    let decision = timeout.decision();
    state(&decision, VerdictV1State::Timeout);
    reason(&decision, ReasonKind::UnboundCause);
}

#[test]
fn decisive_skip_invalid_rows_and_available_oracles_cannot_carry_incomplete_causes() {
    for outcome in [
        CaseV1Outcome::Passed,
        CaseV1Outcome::Failed,
        CaseV1Outcome::Skipped,
        CaseV1Outcome::Ignored,
        CaseV1Outcome::Invalid,
    ] {
        let mut fixture = Fixture::producer_failure();
        fixture.cases[0].outcome = outcome;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        assert!(decision.reasons().iter().any(|reason| matches!(
            reason.kind,
            ReasonKind::UnboundCause | ReasonKind::InvalidAccounting
        )));
    }
    let mut fixture = Fixture::new();
    fixture.oracle_unavailable_cause = IncompleteCause::ProducerFailure;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::UnboundCause);
}

#[test]
fn timeout_caused_native_signal_is_timeout_with_all_derivative_reasons_retained() {
    let decision = Fixture::timeout().decision();
    state(&decision, VerdictV1State::Timeout);
    reason(&decision, ReasonKind::TimeoutIntent);
    reason(&decision, ReasonKind::CaseTimeout);
    reason(&decision, ReasonKind::OracleUnavailable);
    no_reason(&decision, ReasonKind::ProducerUnexpectedSignal);
    no_reason(&decision, ReasonKind::UnboundCause);
}

#[test]
fn cancellation_retains_independent_identity_log_and_cleanup_failures() {
    let mut fixture = Fixture::timeout();
    fixture.producer.termination = Termination::Cancellation;
    fixture.timing.timeout_intent_ms = None;
    fixture.timing.cancellation_intent_ms = Some(90);
    fixture.cases[0].outcome = CaseV1Outcome::Unmeasured;
    fixture.cases[0].incomplete_cause = IncompleteCause::Cancellation;
    fixture.oracle_unavailable_cause = IncompleteCause::Cancellation;
    fixture.identities[0].state = IdentityState::Changed;
    fixture.logs.stderr = LogState::Missing;
    fixture.cleanup.resources = Settlement::Failed;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Cancelled);
    for kind in [
        ReasonKind::CancellationIntent,
        ReasonKind::ChangedIdentity,
        ReasonKind::StderrMissing,
        ReasonKind::ResourcesUnsettled,
    ] {
        reason(&decision, kind);
    }
}

#[test]
fn earliest_stop_intent_wins_and_equal_timestamps_choose_cancellation() {
    for (cancel, wanted) in [
        (90, VerdictV1State::Cancelled),
        (100, VerdictV1State::Cancelled),
        (110, VerdictV1State::Timeout),
    ] {
        let mut fixture = Fixture::timeout();
        fixture.timing.cancellation_intent_ms = Some(cancel);
        let decision = fixture.decision();
        state(&decision, wanted);
        reason(&decision, ReasonKind::TimeoutIntent);
        reason(&decision, ReasonKind::CancellationIntent);
    }
}

#[test]
fn future_early_unknown_and_unbound_timing_facts_cannot_authorize_a_stop() {
    let mut future = Fixture::new();
    future.timing.cancellation_intent_ms = Some(121);
    let decision = future.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::InvalidTiming);
    let mut early = Fixture::new();
    early.timing.timeout_intent_ms = Some(99);
    let decision = early.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::InvalidTiming);
    let mut unknown = Fixture::new();
    unknown.producer.termination = Termination::Unknown;
    let decision = unknown.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::UnknownTermination);
    for termination in [Termination::Deadline, Termination::Cancellation] {
        let mut fixture = Fixture::new();
        fixture.producer.termination = termination;
        fixture.producer.actual = ProcessState::Signalled(9);
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::UnboundTermination);
    }
    let mut reversed = Fixture::new();
    reversed.timing.cleanup_deadline_ms = 99;
    let decision = reversed.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::InvalidTiming);
}

#[test]
fn decisive_observation_requires_a_known_time_strictly_before_work_cutoff() {
    let baseline = Fixture::new();
    state(&baseline.decision(), VerdictV1State::PassCandidate);
    for decisive in [Some(100), Some(101)] {
        let mut fixture = baseline.clone();
        fixture.timing.decisive_ms = decisive;
        let decision = fixture.decision();
        state(&decision, VerdictV1State::Invalid);
        reason(&decision, ReasonKind::DecisiveLate);
    }
    let mut absent = baseline.clone();
    absent.timing.decisive_ms = None;
    state(&absent.decision(), VerdictV1State::Invalid);
    let mut future = baseline;
    future.timing.decisive_ms = Some(121);
    let decision = future.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::InvalidTiming);
}

#[test]
fn missing_result_after_work_cutoff_does_not_invent_an_observed_timeout() {
    let mut fixture = Fixture::new();
    fixture.oracle = OracleFact::Unavailable;
    fixture.timing.decisive_ms = None;
    fixture.timing.observed_ms = 101;
    let decision = fixture.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::DeadlineWithoutIntent);
    reason(&decision, ReasonKind::OracleUnavailable);
    no_reason(&decision, ReasonKind::TimeoutIntent);
}

#[test]
fn original_and_ten_second_cleanup_cutoffs_cannot_be_renewed() {
    let mut boundary = Fixture::new();
    boundary.timing.observed_ms = 200;
    state(&boundary.decision(), VerdictV1State::PassCandidate);
    let mut original = Fixture::new();
    original.timing.observed_ms = 201;
    let decision = original.decision();
    state(&decision, VerdictV1State::Invalid);
    reason(&decision, ReasonKind::CleanupLate);
    let mut stopped = Fixture::timeout();
    stopped.timing.cleanup_deadline_ms = 20_000;
    stopped.timing.observed_ms = 10_101;
    let decision = stopped.decision();
    state(&decision, VerdictV1State::Timeout);
    reason(&decision, ReasonKind::CleanupLate);
    stopped.timing.observed_ms = 10_100;
    let decision = stopped.decision();
    state(&decision, VerdictV1State::Timeout);
    no_reason(&decision, ReasonKind::CleanupLate);
}

#[test]
fn reasons_follow_frozen_plan_order_and_detectors_are_deduplicated_in_declaration_order() {
    let mut fixture = Fixture::new();
    fixture.add(
        Selection::Required,
        CaseV1Outcome::Skipped,
        false,
        IncompleteCause::Unexplained,
    );
    fixture.add(
        Selection::Required,
        CaseV1Outcome::Ignored,
        false,
        IncompleteCause::Unexplained,
    );
    fixture.plans[1].case_id = Name::new("z-before-a").unwrap();
    fixture.cases[1].case_id = fixture.plans[1].case_id.clone();
    fixture.plans[2].case_id = Name::new("a-after-z").unwrap();
    fixture.cases[2].case_id = fixture.plans[2].case_id.clone();
    fixture.cases.reverse();
    fixture.identities[0].state = IdentityState::Changed;
    fixture.logs = Streams {
        stdout: LogState::Missing,
        stderr: LogState::Missing,
    };
    let first = fixture.decision();
    let second = fixture.decision();
    state(&first, VerdictV1State::Invalid);
    assert_eq!(first.reasons(), second.reasons());
    assert_eq!(first.detectors(), second.detectors());
    assert_eq!(first.counts(), second.counts());
    let indexes: Vec<_> = first
        .reasons()
        .iter()
        .filter_map(|reason| reason.case_index)
        .collect();
    assert!(!indexes.is_empty());
    assert!(indexes.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(
        first
            .reasons()
            .iter()
            .any(|item| item.kind == ReasonKind::RequiredSkipped && item.case_index == Some(1))
    );
    assert!(
        first
            .reasons()
            .iter()
            .any(|item| item.kind == ReasonKind::RequiredIgnored && item.case_index == Some(2))
    );
    assert!(first.detectors().windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        first
            .detectors()
            .iter()
            .filter(|value| **value == Detector::Logs)
            .count(),
        1
    );
}
