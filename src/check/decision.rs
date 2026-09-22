//! Pure RC04 verdict derivation from trusted, frozen-plan-bound observations.
//! This module authenticates no observations and grants no admission or case credit.

use crate::contracts::receipt::{
    CaseV1Outcome, ExpectedProducerV1, ExpectedProducerV1Status, Name, Validate, VerdictV1State,
};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_CASES: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Identity {
    Seed,
    Result,
    Fixtures,
    Oracle,
    Harness,
    Collector,
    Launcher,
    Locks,
    Toolchain,
    Profile,
    Standards,
}
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityState {
    Matched,
    Changed,
    Unavailable,
}
#[derive(Clone, Copy, Debug)]
pub struct IdentityFact {
    pub subject: Identity,
    pub state: IdentityState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Selection {
    Required,
    Optional,
    Unselected,
    Excluded,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Design {
    Reviewed,
    Unreviewed,
}
#[derive(Clone, Debug)]
pub struct CasePlan {
    pub case_id: Name,
    pub selection: Selection,
    pub expected_producer: ExpectedProducerV1,
    pub design: Design,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessState {
    Exited(u32),
    Signalled(u32),
    NotStarted,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Termination {
    Ordinary,
    Deadline,
    Cancellation,
    Unknown,
}
#[derive(Clone, Debug)]
pub struct ProcessFact {
    pub expected: ExpectedProducerV1,
    pub actual: ProcessState,
    pub termination: Termination,
}
#[derive(Clone, Debug)]
pub enum CheckerFact {
    InProcessComplete,
    Process(ProcessFact),
    NotStarted,
    Unavailable,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IncompleteCause {
    Unexplained,
    ProducerFailure,
    CheckerFailure,
    Deadline,
    Cancellation,
}
#[derive(Clone, Debug)]
pub struct CaseObservation {
    pub case_id: Name,
    pub executed: bool,
    pub outcome: CaseV1Outcome,
    pub incomplete_cause: IncompleteCause,
    pub producer: ProcessState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleFact {
    Satisfied,
    Mismatch,
    Malformed,
    Unavailable,
    Error,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogState {
    Complete,
    Truncated,
    Missing,
    ReadError,
}
#[derive(Clone, Copy, Debug)]
pub struct Streams {
    pub stdout: LogState,
    pub stderr: LogState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticPolicy {
    CleanBaseline,
    ExpectedFault,
}
#[derive(Clone, Copy, Debug)]
pub enum DiagnosticState {
    Complete { warnings: u32, errors: u32 },
    Unavailable,
}
#[derive(Clone, Copy, Debug)]
pub struct Diagnostics {
    pub policy: DiagnosticPolicy,
    pub state: DiagnosticState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Settlement {
    Settled,
    Pending,
    Failed,
    Unknown,
}
#[derive(Clone, Copy, Debug)]
pub struct CleanupFacts {
    pub descendants: Settlement,
    pub resources: Settlement,
    pub obligations: Settlement,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceState {
    FinalizedComplete,
    Unfinalized,
    Incomplete,
    Missing,
}
#[derive(Clone, Copy, Debug)]
pub struct Timing {
    pub work_deadline_ms: u64,
    pub cleanup_deadline_ms: u64,
    pub observed_ms: u64,
    pub decisive_ms: Option<u64>,
    pub timeout_intent_ms: Option<u64>,
    pub cancellation_intent_ms: Option<u64>,
}
pub struct Input<'a> {
    pub identities: &'a [IdentityFact],
    pub plans: &'a [CasePlan],
    pub cases: &'a [CaseObservation],
    pub producer: &'a ProcessFact,
    pub checker: &'a CheckerFact,
    pub oracle: OracleFact,
    pub oracle_unavailable_cause: IncompleteCause,
    pub logs: Streams,
    pub diagnostics: Diagnostics,
    pub cleanup: CleanupFacts,
    pub evidence: EvidenceState,
    pub timing: Timing,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counts {
    pub discovered: u32,
    pub selected: u32,
    pub mandatory: u32,
    pub executed: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub ignored: u32,
    pub broken: u32,
    pub timed_out: u32,
    pub invalid: u32,
    pub excluded: u32,
    pub unmeasured: u32,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Detector {
    Input,
    Identity,
    Selection,
    Execution,
    RequiredOutcome,
    Producer,
    Checker,
    Oracle,
    Logs,
    Diagnostics,
    Cleanup,
    Evidence,
    Timing,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasonKind {
    Bound,
    MissingIdentity,
    DuplicateIdentity,
    ChangedIdentity,
    UnavailableIdentity,
    EmptyPlan,
    DuplicatePlan,
    DuplicateObservation,
    MissingObservation,
    UnexpectedObservation,
    InvalidExpectation,
    InvalidAccounting,
    UnreviewedRequired,
    EmptySelection,
    NoMandatory,
    NoExecution,
    RequiredNotExecuted,
    RequiredSkipped,
    RequiredIgnored,
    RequiredBroken,
    RequiredInvalid,
    RequiredUnmeasured,
    CaseMismatch,
    CaseUnexpectedStatus,
    CaseTimeout,
    ProducerNotStarted,
    ProducerUnknown,
    ProducerUnexpectedExit,
    ProducerUnexpectedSignal,
    CheckerNotStarted,
    CheckerUnavailable,
    CheckerUnexpectedExit,
    CheckerUnexpectedSignal,
    OracleMismatch,
    OracleMalformed,
    OracleUnavailable,
    OracleError,
    StdoutTruncated,
    StdoutMissing,
    StdoutReadError,
    StderrTruncated,
    StderrMissing,
    StderrReadError,
    BaselineWarnings,
    BaselineErrors,
    DiagnosticsUnavailable,
    DescendantsUnsettled,
    ResourcesUnsettled,
    ObligationsUnsettled,
    EvidenceUnfinalized,
    EvidenceIncomplete,
    EvidenceMissing,
    InvalidTiming,
    UnknownTermination,
    UnboundTermination,
    UnboundCause,
    DeadlineWithoutIntent,
    DecisiveLate,
    CleanupLate,
    TimeoutIntent,
    CancellationIntent,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Reason {
    pub kind: ReasonKind,
    /// Index in the frozen input plan, never a candidate-controlled diagnostic string.
    pub case_index: Option<usize>,
    pub identity: Option<Identity>,
}
#[derive(Clone, Debug)]
pub struct Decision {
    pub state: VerdictV1State,
    /// Diagnostic summary only when input accounting was invalid; never a receipt inventory.
    pub counts: Counts,
    pub reasons: Vec<Reason>,
    pub detectors: Vec<Detector>,
}
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum Severity {
    Invalid,
    Derivative,
    Error,
    Fail,
    Intent,
}
struct Assessment {
    reasons: Vec<Reason>,
    detectors: BTreeSet<Detector>,
    classes: BTreeSet<Severity>,
}
impl Assessment {
    fn new() -> Self {
        Self {
            reasons: Vec::new(),
            detectors: BTreeSet::new(),
            classes: BTreeSet::new(),
        }
    }
    fn add(&mut self, kind: ReasonKind, detector: Detector, severity: Severity) {
        self.at(kind, detector, severity, None, None);
    }
    fn at(
        &mut self,
        kind: ReasonKind,
        detector: Detector,
        severity: Severity,
        case_index: Option<usize>,
        identity: Option<Identity>,
    ) {
        self.reasons.push(Reason {
            kind,
            case_index,
            identity,
        });
        self.detectors.insert(detector);
        self.classes.insert(severity);
    }
}

/// Derive a bounded decision without executing effects or accepting a desired verdict.
/// Missing evidence is fail-closed; all observed gate reasons remain in the result.
#[must_use]
pub fn decide(input: &Input<'_>) -> Decision {
    let mut a = Assessment::new();
    let causal = timing(input, &mut a);
    let causes = causes(input, causal.is_some());
    identities(input.identities, &mut a);
    let counts = cases(input.plans, input.cases, &causes, &mut a);
    process(input.producer, false, &input.timing, &mut a);
    match input.checker {
        CheckerFact::Process(fact) => process(fact, true, &input.timing, &mut a),
        CheckerFact::NotStarted => a.add(
            ReasonKind::CheckerNotStarted,
            Detector::Checker,
            Severity::Derivative,
        ),
        CheckerFact::Unavailable => a.add(
            ReasonKind::CheckerUnavailable,
            Detector::Checker,
            Severity::Invalid,
        ),
        CheckerFact::InProcessComplete => {}
    }
    oracle(
        input.oracle,
        input.oracle_unavailable_cause,
        &causes,
        &mut a,
    );
    evidence(input, &mut a);
    let state = if let Some(state) = causal {
        state
    } else if a.classes.contains(&Severity::Invalid)
        || (a.classes.contains(&Severity::Derivative) && !a.classes.contains(&Severity::Error))
    {
        VerdictV1State::Invalid
    } else if a.classes.contains(&Severity::Error) {
        VerdictV1State::Error
    } else if a.classes.contains(&Severity::Fail) {
        VerdictV1State::Fail
    } else {
        VerdictV1State::PassCandidate
    };
    Decision {
        state,
        counts,
        reasons: a.reasons,
        detectors: a.detectors.into_iter().collect(),
    }
}

fn causes(input: &Input<'_>, coherent_stop: bool) -> BTreeSet<IncompleteCause> {
    let mut causes = BTreeSet::new();
    let unexpected = |p: &ProcessFact| {
        p.expected.validate().is_ok()
            && p.termination == Termination::Ordinary
            && matches!(
                p.actual,
                ProcessState::Exited(_) | ProcessState::Signalled(_)
            )
            && !matches(&p.expected, p.actual)
    };
    if unexpected(input.producer) {
        causes.insert(IncompleteCause::ProducerFailure);
    }
    if input.oracle == OracleFact::Error
        || matches!(input.checker, CheckerFact::Process(p) if unexpected(p))
    {
        causes.insert(IncompleteCause::CheckerFailure);
    }
    if coherent_stop {
        if input.timing.timeout_intent_ms.is_some() {
            causes.insert(IncompleteCause::Deadline);
        }
        if input.timing.cancellation_intent_ms.is_some() {
            causes.insert(IncompleteCause::Cancellation);
        }
    }
    causes
}
fn incomplete(
    cause: IncompleteCause,
    causes: &BTreeSet<IncompleteCause>,
    index: Option<usize>,
    a: &mut Assessment,
) -> Severity {
    if cause == IncompleteCause::Unexplained {
        return Severity::Invalid;
    }
    if causes.contains(&cause) {
        return Severity::Derivative;
    }
    a.at(
        ReasonKind::UnboundCause,
        Detector::Input,
        Severity::Invalid,
        index,
        None,
    );
    Severity::Invalid
}

fn identities(facts: &[IdentityFact], a: &mut Assessment) {
    if facts.len() > IDENTITIES.len() {
        a.add(ReasonKind::Bound, Detector::Input, Severity::Invalid);
        return;
    }
    for identity in IDENTITIES {
        let mut found = facts.iter().filter(|fact| fact.subject == identity);
        let Some(fact) = found.next() else {
            a.at(
                ReasonKind::MissingIdentity,
                Detector::Identity,
                Severity::Invalid,
                None,
                Some(identity),
            );
            continue;
        };
        if found.next().is_some() {
            a.at(
                ReasonKind::DuplicateIdentity,
                Detector::Identity,
                Severity::Invalid,
                None,
                Some(identity),
            );
        }
        let kind = match fact.state {
            IdentityState::Matched => continue,
            IdentityState::Changed => ReasonKind::ChangedIdentity,
            IdentityState::Unavailable => ReasonKind::UnavailableIdentity,
        };
        a.at(
            kind,
            Detector::Identity,
            Severity::Invalid,
            None,
            Some(identity),
        );
    }
}

fn matches(expected: &ExpectedProducerV1, actual: ProcessState) -> bool {
    match (expected.status, actual) {
        (ExpectedProducerV1Status::Exited, ProcessState::Exited(code)) => {
            expected.exit_code.value == Some(code)
        }
        (ExpectedProducerV1Status::Signalled, ProcessState::Signalled(signal)) => {
            expected.signal.value == Some(signal)
        }
        _ => false,
    }
}

fn cases(
    plans: &[CasePlan],
    observations: &[CaseObservation],
    causes: &BTreeSet<IncompleteCause>,
    a: &mut Assessment,
) -> Counts {
    let mut counts = Counts::default();
    if plans.len() > MAX_CASES || observations.len() > MAX_CASES {
        a.add(ReasonKind::Bound, Detector::Input, Severity::Invalid);
        return counts;
    }
    if plans.is_empty() {
        a.add(
            ReasonKind::EmptyPlan,
            Detector::Selection,
            Severity::Invalid,
        );
    }
    let mut plan_ids = BTreeSet::new();
    let mut observed = BTreeMap::new();
    for row in observations {
        if observed.insert(row.case_id.as_str(), row).is_some() {
            a.add(
                ReasonKind::DuplicateObservation,
                Detector::Input,
                Severity::Invalid,
            );
        }
    }
    for (index, plan) in plans.iter().enumerate() {
        counts.discovered += 1;
        if !plan_ids.insert(plan.case_id.as_str()) {
            a.at(
                ReasonKind::DuplicatePlan,
                Detector::Input,
                Severity::Invalid,
                Some(index),
                None,
            );
        }
        let selected = matches!(plan.selection, Selection::Required | Selection::Optional);
        counts.selected += u32::from(selected);
        counts.mandatory += u32::from(plan.selection == Selection::Required);
        if plan.expected_producer.validate().is_err() {
            a.at(
                ReasonKind::InvalidExpectation,
                Detector::Input,
                Severity::Invalid,
                Some(index),
                None,
            );
        }
        if plan.selection == Selection::Required && plan.design != Design::Reviewed {
            a.at(
                ReasonKind::UnreviewedRequired,
                Detector::RequiredOutcome,
                Severity::Invalid,
                Some(index),
                None,
            );
        }
        let Some(row) = observed.remove(plan.case_id.as_str()) else {
            a.at(
                ReasonKind::MissingObservation,
                Detector::Input,
                Severity::Invalid,
                Some(index),
                None,
            );
            continue;
        };
        case_row(plan, row, index, &mut counts, causes, a);
    }
    if !observed.is_empty() {
        a.add(
            ReasonKind::UnexpectedObservation,
            Detector::Input,
            Severity::Invalid,
        );
    }
    if counts.selected == 0 {
        a.add(
            ReasonKind::EmptySelection,
            Detector::Selection,
            Severity::Invalid,
        );
    }
    if counts.mandatory == 0 {
        a.add(
            ReasonKind::NoMandatory,
            Detector::Selection,
            Severity::Invalid,
        );
    }
    if counts.executed == 0 {
        a.add(
            ReasonKind::NoExecution,
            Detector::Execution,
            Severity::Derivative,
        );
    }
    counts
}
fn case_row(
    plan: &CasePlan,
    row: &CaseObservation,
    index: usize,
    counts: &mut Counts,
    causes: &BTreeSet<IncompleteCause>,
    a: &mut Assessment,
) {
    let selected = matches!(plan.selection, Selection::Required | Selection::Optional);
    let unfinished = incomplete(row.incomplete_cause, causes, Some(index), a);
    if row.incomplete_cause != IncompleteCause::Unexplained
        && matches!(
            row.outcome,
            CaseV1Outcome::Passed
                | CaseV1Outcome::Failed
                | CaseV1Outcome::Skipped
                | CaseV1Outcome::Ignored
                | CaseV1Outcome::Invalid
        )
    {
        a.at(
            ReasonKind::InvalidAccounting,
            Detector::Input,
            Severity::Invalid,
            Some(index),
            None,
        );
    }
    counts.executed += u32::from(row.executed);
    if plan.selection == Selection::Excluded {
        counts.excluded += 1;
    } else {
        count_outcome(counts, row.outcome);
    }
    if (!selected && (row.executed || row.outcome != CaseV1Outcome::Unmeasured))
        || (!row.executed && matches!(row.outcome, CaseV1Outcome::Passed | CaseV1Outcome::Failed))
        || (row.executed && row.producer == ProcessState::NotStarted)
    {
        a.at(
            ReasonKind::InvalidAccounting,
            Detector::Input,
            Severity::Invalid,
            Some(index),
            None,
        );
    }
    if row.executed
        && row.outcome == CaseV1Outcome::Passed
        && !matches(&plan.expected_producer, row.producer)
    {
        a.at(
            ReasonKind::CaseUnexpectedStatus,
            Detector::Producer,
            Severity::Error,
            Some(index),
            None,
        );
    }
    if selected {
        match row.outcome {
            CaseV1Outcome::Failed => a.at(
                ReasonKind::CaseMismatch,
                Detector::Oracle,
                Severity::Fail,
                Some(index),
                None,
            ),
            CaseV1Outcome::Timeout => {
                if row.incomplete_cause != IncompleteCause::Deadline {
                    a.at(
                        ReasonKind::UnboundCause,
                        Detector::Input,
                        Severity::Invalid,
                        Some(index),
                        None,
                    );
                }
                a.at(
                    ReasonKind::CaseTimeout,
                    Detector::Timing,
                    if row.incomplete_cause == IncompleteCause::Deadline {
                        unfinished
                    } else {
                        Severity::Invalid
                    },
                    Some(index),
                    None,
                );
            }
            _ => {}
        }
    }
    if plan.selection == Selection::Required {
        required(row, index, unfinished, a);
    }
}

fn count_outcome(c: &mut Counts, outcome: CaseV1Outcome) {
    match outcome {
        CaseV1Outcome::Passed => c.passed += 1,
        CaseV1Outcome::Failed => c.failed += 1,
        CaseV1Outcome::Skipped => c.skipped += 1,
        CaseV1Outcome::Ignored => c.ignored += 1,
        CaseV1Outcome::Broken => c.broken += 1,
        CaseV1Outcome::Timeout => c.timed_out += 1,
        CaseV1Outcome::Invalid => c.invalid += 1,
        CaseV1Outcome::Unmeasured => c.unmeasured += 1,
    }
}
fn required(row: &CaseObservation, index: usize, unfinished: Severity, a: &mut Assessment) {
    if !row.executed {
        a.at(
            ReasonKind::RequiredNotExecuted,
            Detector::Execution,
            unfinished,
            Some(index),
            None,
        );
    }
    let (kind, severity) = match row.outcome {
        CaseV1Outcome::Skipped => (ReasonKind::RequiredSkipped, Severity::Invalid),
        CaseV1Outcome::Ignored => (ReasonKind::RequiredIgnored, Severity::Invalid),
        CaseV1Outcome::Broken => (ReasonKind::RequiredBroken, unfinished),
        CaseV1Outcome::Invalid => (ReasonKind::RequiredInvalid, Severity::Invalid),
        CaseV1Outcome::Unmeasured => (ReasonKind::RequiredUnmeasured, unfinished),
        _ => return,
    };
    a.at(kind, Detector::RequiredOutcome, severity, Some(index), None);
}
fn process(fact: &ProcessFact, checker: bool, timing: &Timing, a: &mut Assessment) {
    let detector = if checker {
        Detector::Checker
    } else {
        Detector::Producer
    };
    if fact.expected.validate().is_err() {
        a.add(
            ReasonKind::InvalidExpectation,
            Detector::Input,
            Severity::Invalid,
        );
    }
    match fact.termination {
        Termination::Unknown => a.add(ReasonKind::UnknownTermination, detector, Severity::Invalid),
        Termination::Deadline if timing.timeout_intent_ms.is_none() => {
            a.add(ReasonKind::UnboundTermination, detector, Severity::Invalid);
        }
        Termination::Cancellation if timing.cancellation_intent_ms.is_none() => {
            a.add(ReasonKind::UnboundTermination, detector, Severity::Invalid);
        }
        _ => {}
    }
    if matches(&fact.expected, fact.actual) {
        return;
    }
    // A native status caused by this process's coherent stop intent is retained
    // in the observation. It is not an independent ordinary producer crash.
    let coherent_clock = timing.work_deadline_ms <= timing.cleanup_deadline_ms
        && timing
            .timeout_intent_ms
            .is_none_or(|at| at >= timing.work_deadline_ms && at <= timing.observed_ms)
        && timing
            .cancellation_intent_ms
            .is_none_or(|at| at <= timing.observed_ms);
    let caused_stop = coherent_clock
        && match fact.termination {
            Termination::Deadline => timing.timeout_intent_ms.is_some(),
            Termination::Cancellation => timing.cancellation_intent_ms.is_some(),
            Termination::Ordinary | Termination::Unknown => false,
        };
    if caused_stop
        && matches!(
            fact.actual,
            ProcessState::Exited(_) | ProcessState::Signalled(_)
        )
    {
        return;
    }
    let (kind, severity) = match (checker, fact.actual) {
        (false, ProcessState::NotStarted) => (ReasonKind::ProducerNotStarted, Severity::Derivative),
        (true, ProcessState::NotStarted) => (ReasonKind::CheckerNotStarted, Severity::Derivative),
        (false, ProcessState::Unknown) => (ReasonKind::ProducerUnknown, Severity::Invalid),
        (true, ProcessState::Unknown) => (ReasonKind::CheckerUnavailable, Severity::Invalid),
        (false, ProcessState::Exited(_)) => (ReasonKind::ProducerUnexpectedExit, Severity::Error),
        (false, ProcessState::Signalled(_)) => {
            (ReasonKind::ProducerUnexpectedSignal, Severity::Error)
        }
        (true, ProcessState::Exited(_)) => (ReasonKind::CheckerUnexpectedExit, Severity::Error),
        (true, ProcessState::Signalled(_)) => {
            (ReasonKind::CheckerUnexpectedSignal, Severity::Error)
        }
    };
    a.add(kind, detector, severity);
}
fn oracle(
    fact: OracleFact,
    cause: IncompleteCause,
    causes: &BTreeSet<IncompleteCause>,
    a: &mut Assessment,
) {
    let unavailable = incomplete(cause, causes, None, a);
    if fact != OracleFact::Unavailable && cause != IncompleteCause::Unexplained {
        a.add(ReasonKind::UnboundCause, Detector::Input, Severity::Invalid);
    }
    let (kind, severity) = match fact {
        OracleFact::Satisfied => return,
        OracleFact::Mismatch => (ReasonKind::OracleMismatch, Severity::Fail),
        OracleFact::Malformed => (ReasonKind::OracleMalformed, Severity::Invalid),
        OracleFact::Unavailable => (ReasonKind::OracleUnavailable, unavailable),
        OracleFact::Error => (ReasonKind::OracleError, Severity::Error),
    };
    a.add(kind, Detector::Oracle, severity);
}
fn evidence(input: &Input<'_>, a: &mut Assessment) {
    for (state, stdout) in [(input.logs.stdout, true), (input.logs.stderr, false)] {
        let kind = match (stdout, state) {
            (_, LogState::Complete) => continue,
            (true, LogState::Truncated) => ReasonKind::StdoutTruncated,
            (true, LogState::Missing) => ReasonKind::StdoutMissing,
            (true, LogState::ReadError) => ReasonKind::StdoutReadError,
            (false, LogState::Truncated) => ReasonKind::StderrTruncated,
            (false, LogState::Missing) => ReasonKind::StderrMissing,
            (false, LogState::ReadError) => ReasonKind::StderrReadError,
        };
        a.add(kind, Detector::Logs, Severity::Invalid);
    }
    match input.diagnostics.state {
        DiagnosticState::Unavailable => a.add(
            ReasonKind::DiagnosticsUnavailable,
            Detector::Diagnostics,
            Severity::Invalid,
        ),
        DiagnosticState::Complete { warnings, errors }
            if input.diagnostics.policy == DiagnosticPolicy::CleanBaseline =>
        {
            if warnings > 0 {
                a.add(
                    ReasonKind::BaselineWarnings,
                    Detector::Diagnostics,
                    Severity::Invalid,
                );
            }
            if errors > 0 {
                a.add(
                    ReasonKind::BaselineErrors,
                    Detector::Diagnostics,
                    Severity::Invalid,
                );
            }
        }
        DiagnosticState::Complete { .. } => {}
    }
    for (state, kind) in [
        (input.cleanup.descendants, ReasonKind::DescendantsUnsettled),
        (input.cleanup.resources, ReasonKind::ResourcesUnsettled),
        (input.cleanup.obligations, ReasonKind::ObligationsUnsettled),
    ] {
        if state != Settlement::Settled {
            a.add(kind, Detector::Cleanup, Severity::Invalid);
        }
    }
    let kind = match input.evidence {
        EvidenceState::FinalizedComplete => return,
        EvidenceState::Unfinalized => ReasonKind::EvidenceUnfinalized,
        EvidenceState::Incomplete => ReasonKind::EvidenceIncomplete,
        EvidenceState::Missing => ReasonKind::EvidenceMissing,
    };
    a.add(kind, Detector::Evidence, Severity::Invalid);
}
fn timing(input: &Input<'_>, a: &mut Assessment) -> Option<VerdictV1State> {
    let t = input.timing;
    let invalid = t.work_deadline_ms > t.cleanup_deadline_ms
        || t.timeout_intent_ms
            .is_some_and(|at| at < t.work_deadline_ms || at > t.observed_ms)
        || t.cancellation_intent_ms
            .is_some_and(|at| at > t.observed_ms);
    if invalid {
        a.add(
            ReasonKind::InvalidTiming,
            Detector::Timing,
            Severity::Invalid,
        );
    }
    if matches!(input.oracle, OracleFact::Satisfied | OracleFact::Mismatch) {
        match t.decisive_ms {
            None => a.add(
                ReasonKind::InvalidTiming,
                Detector::Timing,
                Severity::Invalid,
            ),
            Some(at) if at > t.observed_ms => a.add(
                ReasonKind::InvalidTiming,
                Detector::Timing,
                Severity::Invalid,
            ),
            Some(at) if at >= t.work_deadline_ms => a.add(
                ReasonKind::DecisiveLate,
                Detector::Timing,
                Severity::Invalid,
            ),
            Some(_) => {}
        }
    } else if t.decisive_ms.is_some() {
        a.add(
            ReasonKind::InvalidTiming,
            Detector::Timing,
            Severity::Invalid,
        );
    }
    if t.observed_ms > t.cleanup_deadline_ms
        || [t.timeout_intent_ms, t.cancellation_intent_ms]
            .into_iter()
            .flatten()
            .any(|at| t.observed_ms.saturating_sub(at) > 10_000)
    {
        a.add(
            ReasonKind::CleanupLate,
            Detector::Cleanup,
            Severity::Invalid,
        );
    }
    if t.timeout_intent_ms.is_some() {
        a.add(
            ReasonKind::TimeoutIntent,
            Detector::Timing,
            Severity::Intent,
        );
    }
    if t.cancellation_intent_ms.is_some() {
        a.add(
            ReasonKind::CancellationIntent,
            Detector::Timing,
            Severity::Intent,
        );
    }
    if invalid {
        return None;
    }
    if t.observed_ms >= t.work_deadline_ms
        && t.timeout_intent_ms.is_none()
        && !matches!(input.oracle, OracleFact::Satisfied | OracleFact::Mismatch)
    {
        a.add(
            ReasonKind::DeadlineWithoutIntent,
            Detector::Timing,
            Severity::Invalid,
        );
    }
    match (t.timeout_intent_ms, t.cancellation_intent_ms) {
        (Some(timeout), Some(cancel)) if cancel <= timeout => Some(VerdictV1State::Cancelled),
        (Some(_), _) => Some(VerdictV1State::Timeout),
        (None, Some(_)) => Some(VerdictV1State::Cancelled),
        (None, None) => None,
    }
}
