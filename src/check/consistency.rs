//! Recompute receipt accounting against immutable pre-execution expectations.
//! Consistency is necessary, but cannot authenticate candidate or collector custody.

use super::graph::{Error as GraphError, Graph};
use super::{patch, u64_oracle};
use crate::contracts::receipt::{
    Address, ArtifactV1, ArtifactV1Availability, AvailabilityReceiptV1, AvailabilityV1State,
    CampaignV1, CaseV1, CaseV1Outcome, CleanupContractV1, DiagnosticV1, EffectPageV1,
    EnvironmentPageV1, ExpectationV1, ExpectationV1ExpectedOracle, ExpectedProducerV1,
    ExpectedProducerV1Status, FindingV1, FindingV1Disposition, Generation, GrantPageV1, Id,
    IdentityV1, InvocationV1, LimitsV1, List, Maybe, MutantV1, MutantV1Outcome, Name, ObligationV1,
    ObligationV1State, OracleResultV1, OracleResultV1Result, Payload, ProducerV1, ProducerV1Status,
    ReceiptRecord, ReceiptV1, Ref, RelPath, ReviewReceiptV1, ReviewV1, Sha, SubjectFileV1,
    SubjectFileV1Kind, SubjectV1, SubjectsV1, Text, TypedRef, Validate, VerdictV1State, decode,
};
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet};

/// Case selection and independent expectations frozen before producer execution.
#[derive(Clone, Debug)]
pub struct CasePlan {
    pub case_id: Name,
    pub primary_module_id: Name,
    pub criterion_ids: List<Name>,
    pub fixture_sha256: Sha,
    pub oracle_id: Name,
    pub expected: TypedRef<ExpectationV1>,
    pub mandatory: bool,
    pub selected: bool,
    pub excluded: bool,
    /// An independently reviewed case-design record, not a candidate claim.
    pub reviewed_design: Option<TypedRef<ReviewV1>>,
}

/// Trusted check-owner preparation. Candidate JSON cannot supply this value.
#[derive(Clone, Debug)]
pub struct Prepared {
    pub schema_sha256: Sha,
    pub identity: IdentityV1,
    pub subjects: SubjectsV1,
    pub invocation: InvocationV1,
    pub cases: Vec<CasePlan>,
    /// The one file a candidate may change, and how much: the class's, fixed before execution. The
    /// receipt's patch binding refuses a change anywhere else or past these bounds (B14-P4 R2).
    pub editable: Editable,
}

/// A class's one editable file and its candidate bounds (B14-P2's profile will supply both).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Editable {
    pub path: RelPath,
    pub bounds: patch::CandidateBounds,
}

/// The one frozen WL-U64 case and the criterion it credits.
pub const U64_CASE_ID: &str = "WL-U64-PARSE-001-v1";
pub const U64_CRITERION_ID: &str = "u64-frozen-exact-output";
/// The class's whole criteria list, in the order a task binds it by (`criteria_digest`): the one
/// credited criterion, so the class's criteria are this check's, never a second description.
pub const U64_CRITERIA: [&str; 1] = [U64_CRITERION_ID];
const U64_MODULE_ID: &str = "check";
/// The WL-U64 class's one editable file (`TASK.md`: "Change only `src/lib.rs`") and its bounds:
/// R1's 200 changed lines, and 64 KiB against a 751-byte reference. B14-P2's profile takes them over.
pub const U64_EDITABLE: &str = "src/lib.rs";
pub const U64_BOUNDS: patch::CandidateBounds = patch::CandidateBounds {
    bytes: 65_536,
    changed_lines: 200,
};
const U64_CWD: &str = "work";

/// Typed, already published inputs for one WL-U64 attempt. Nothing here is a
/// candidate claim: the trusted preparer published every referenced object.
#[derive(Clone, Debug)]
pub struct U64Attempt {
    pub schema_sha256: Sha,
    pub run_id: Id,
    pub task_id: Id,
    pub attempt_id: Id,
    pub generation: Generation,
    pub profile_id: Name,
    pub parent_run: Maybe<Id>,
    pub subjects: SubjectsV1,
    pub argv: List<Text>,
    pub environment: TypedRef<EnvironmentPageV1>,
    pub grants: TypedRef<GrantPageV1>,
    pub limits: TypedRef<LimitsV1>,
    pub allowed_effects: TypedRef<EffectPageV1>,
    pub cleanup_contract: TypedRef<CleanupContractV1>,
    pub expectation: TypedRef<ExpectationV1>,
    pub case_design_review: TypedRef<ReviewV1>,
}

/// Op1 for the one admitted workload: compose the frozen verification plan.
/// The oracle is `u64_oracle`'s pinned identity, the fixture digest is the
/// fixtures subject's, and one reviewed mandatory case carries the criterion.
///
/// # Errors
/// `Binding` when the run, task and attempt identities are not pairwise
/// distinct or the invocation has no argv.
pub fn prepare_u64(attempt: U64Attempt) -> Result<Prepared, Error> {
    let identities = [&attempt.run_id, &attempt.task_id, &attempt.attempt_id]
        .map(Id::as_str)
        .into_iter()
        .collect::<BTreeSet<_>>();
    if identities.len() != 3 || attempt.argv.as_slice().is_empty() {
        return Err(Error::Binding);
    }
    let name = |value: &str| Name::new(value).map_err(|_| Error::Encoding);
    let criteria = List::new(
        U64_CRITERIA
            .iter()
            .map(|criterion| name(criterion))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .map_err(|_| Error::Encoding)?;
    let case = CasePlan {
        case_id: name(U64_CASE_ID)?,
        primary_module_id: name(U64_MODULE_ID)?,
        criterion_ids: criteria.clone(),
        fixture_sha256: attempt.subjects.fixtures.as_ref().sha256.clone(),
        oracle_id: name(u64_oracle::ORACLE_ID)?,
        expected: attempt.expectation.clone(),
        mandatory: true,
        selected: true,
        excluded: false,
        reviewed_design: Some(attempt.case_design_review),
    };
    Ok(Prepared {
        schema_sha256: attempt.schema_sha256,
        identity: IdentityV1 {
            run_id: attempt.run_id,
            task_id: attempt.task_id,
            attempt_id: attempt.attempt_id,
            generation: attempt.generation,
            module_id: name(U64_MODULE_ID)?,
            criterion_ids: criteria,
            profile_id: attempt.profile_id,
            parent_run: attempt.parent_run,
        },
        subjects: attempt.subjects,
        invocation: InvocationV1 {
            argv: attempt.argv,
            cwd_logical: name(U64_CWD)?,
            environment: attempt.environment,
            grants: attempt.grants,
            expected: attempt.expectation,
            oracle_id: name(u64_oracle::ORACLE_ID)?,
            limits: attempt.limits,
            allowed_effects: attempt.allowed_effects,
            cleanup_contract: attempt.cleanup_contract,
        },
        cases: vec![case],
        editable: Editable {
            path: RelPath::new(U64_EDITABLE).map_err(|_| Error::Encoding)?,
            bounds: U64_BOUNDS,
        },
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Graph(GraphError),
    Encoding,
    Binding,
    CasePlan,
    CaseCounts,
    RequiredCase,
    Producer,
    Oracle,
    Diagnostic,
    Artifact,
    Mutation,
    Clock,
    Obligation,
    Availability,
    /// The seed → result patch is not the one its subjects derive (B14-P4).
    Patch(PatchRefusal),
}

/// Why a receipt's `seed_to_result_patch` is not bound to its seed and result subjects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PatchRefusal {
    /// The two subjects list different numbers of entries.
    Count,
    /// An entry differs in something other than its content (path, kind, mode, link, origin,
    /// exclusion).
    Entry,
    /// More than one entry's content differs.
    TwoChanges,
    /// The one changed entry is not a regular file with content on both sides.
    Editable,
    /// The one changed entry is not the preparation's editable file.
    Path,
    /// The two editable texts derive no patch within the patch's own edit count.
    Derivation(patch::Error),
    /// The patch object's bytes are not the derived patch.
    Mismatch,
}
impl From<GraphError> for Error {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

/// Summary of internally checked observations; never an acceptance capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Summary {
    pub cases: u32,
    pub primary_credit: u32,
    pub mandatory_cases: u32,
}

/// Validate exact plan bindings and recompute the complete receipt inventories.
/// Nonpass receipts may retain failed producer/cleanup observations. A declared
/// PASS must satisfy all locally and transitively checkable pass predicates too.
/// Actual protected process/filesystem/oracle observations remain the collector's
/// separate duty; metadata alone cannot establish them.
///
/// # Errors
/// Refuses changed plans, contradictory counts/expectations, incomplete proof or
/// any unsupported PASS claim. Never rewrites a receipt to make it consistent.
pub fn validate(
    graph: &Graph,
    reference: &TypedRef<ReceiptV1>,
    prepared: &Prepared,
) -> Result<Summary, Error> {
    let receipt: ReceiptV1 = typed(graph, reference)?;
    let receipt = &receipt;
    receipt.validate().map_err(|_| Error::Encoding)?;
    if receipt.schema_sha256 != prepared.schema_sha256
        || receipt.identity != prepared.identity
        || receipt.subjects != prepared.subjects
        || receipt.invocation != prepared.invocation
    {
        return Err(Error::Binding);
    }
    patch_binding(graph, &receipt.subjects, &prepared.editable)?;
    let pass = receipt.verdict.state == VerdictV1State::PassCandidate;
    let summary = cases(graph, receipt, prepared, pass)?;
    diagnostics(graph, receipt, pass)?;
    artifacts(graph, receipt, pass)?;
    mutations(graph, receipt)?;
    clocks(graph, receipt, pass)?;
    let expectation: ExpectationV1 = typed(graph, &receipt.invocation.expected)?;
    let oracle: OracleResultV1 = typed(graph, &receipt.verdict.oracle_result)?;
    if expectation.oracle_id != receipt.invocation.oracle_id
        || oracle.oracle_id != expectation.oracle_id
        || oracle.expected != receipt.invocation.expected
    {
        return Err(Error::Oracle);
    }
    if pass {
        if !producer_matches(
            &receipt.observations.producer,
            &expectation.expected_producer,
        ) {
            return Err(Error::Producer);
        }
        let expected = match expectation.expected_oracle {
            ExpectationV1ExpectedOracle::Satisfied => OracleResultV1Result::Satisfied,
        };
        if oracle.result != expected
            || oracle.detector_id.value.as_ref() != Some(&expectation.intended_detector)
            || receipt.verdict.intended_detector != expectation.intended_detector
        {
            return Err(Error::Oracle);
        }
        let obligations: Vec<ObligationV1> =
            rows(graph, receipt.observations.unresolved_obligations.as_ref())?;
        if obligations
            .iter()
            .any(|obligation| obligation.material && obligation.state != ObligationV1State::Settled)
        {
            return Err(Error::Obligation);
        }
    }
    let missing = graph.rows(receipt.availability.missing_objects.as_ref())?;
    if receipt.availability.state == AvailabilityV1State::Complete && !missing.is_empty() {
        return Err(Error::Availability);
    }
    Ok(summary)
}

/// B14-P4 · a receipt that names both a result and a seed → result patch names the patch its
/// subjects derive, whoever composed it. The seed's and the result's files are read back from the
/// receipt's own graph and must agree entry for entry (path, kind, mode, link, origin, exclusion);
/// at most one entry's content may differ, and it must be a regular file on both sides. The patch
/// object's bytes must then be exactly the unified diff of that file's two texts under its path
/// — or empty when no content differs. That file must be the preparation's editable one, and its
/// result text at most the class's bytes (the seed, the class's own baseline, only `MAX_TEXT`). The
/// search runs at most `min(edit_count(claimed), changed_lines + 2)` rounds, so its work is
/// `O((seed + result lines) · (changed_lines + 2))`, bounded by the preparation, never by a clock
/// (review P4-2). A receipt missing either side claims no patch and is left to the pass predicates.
fn patch_binding(graph: &Graph, subjects: &SubjectsV1, editable: &Editable) -> Result<(), Error> {
    let (Some(result), Some(claimed)) = (
        subjects.result_subject.value.as_ref(),
        subjects.seed_to_result_patch.value.as_ref(),
    ) else {
        return Ok(());
    };
    let refuse = |refusal| Error::Patch(refusal);
    let seed: SubjectV1 = typed(graph, &subjects.seed_subject)?;
    let result: SubjectV1 = typed(graph, result)?;
    let seed: Vec<SubjectFileV1> = rows(graph, seed.files.as_ref())?;
    let result: Vec<SubjectFileV1> = rows(graph, result.files.as_ref())?;
    if seed.len() != result.len() {
        return Err(refuse(PatchRefusal::Count));
    }
    let identity = |content: &Maybe<Payload>| {
        content.value.as_ref().map(|payload| {
            let reference: &Ref = payload.as_ref();
            (reference.sha256.clone(), reference.byte_length)
        })
    };
    let mut changed = None;
    for (before, after) in seed.iter().zip(&result) {
        if before.path != after.path
            || before.kind != after.kind
            || before.executable != after.executable
            || before.link_target != after.link_target
            || before.origin != after.origin
            || before.exclusion_reason != after.exclusion_reason
        {
            return Err(refuse(PatchRefusal::Entry));
        }
        if identity(&before.content) != identity(&after.content) {
            if changed.is_some() {
                return Err(refuse(PatchRefusal::TwoChanges));
            }
            if before.path != editable.path {
                return Err(refuse(PatchRefusal::Path));
            }
            changed = Some((changed_text(before)?, changed_text(after)?));
        }
    }
    let claimed = graph.get(claimed.as_ref())?.bytes();
    let derived = match changed {
        None => Vec::new(),
        Some((before, after)) => {
            let after = graph.get(after)?.bytes();
            if after.len() > editable.bounds.bytes.min(patch::MAX_TEXT) {
                return Err(refuse(PatchRefusal::Derivation(patch::Error::Bound)));
            }
            let class = editable
                .bounds
                .changed_lines
                .min(patch::MAX_CHANGED_LINES)
                .saturating_add(2);
            patch::unified(
                graph.get(before)?.bytes(),
                after,
                editable.path.as_str(),
                patch::edit_count(claimed).min(class),
                None,
            )
            .map_err(|error| refuse(PatchRefusal::Derivation(error)))?
        }
    };
    if derived != claimed {
        return Err(refuse(PatchRefusal::Mismatch));
    }
    Ok(())
}

/// The changed entry's content reference: it must be a regular file with content.
fn changed_text(file: &SubjectFileV1) -> Result<&Ref, Error> {
    match (&file.kind, &file.content.value) {
        (SubjectFileV1Kind::File, Some(content)) => Ok(content.as_ref()),
        _ => Err(Error::Patch(PatchRefusal::Editable)),
    }
}

fn typed<T: ReceiptRecord>(graph: &Graph, reference: &TypedRef<T>) -> Result<T, Error> {
    decode(graph.get(reference.as_ref())?.bytes()).map_err(|_| Error::Encoding)
}
fn rows<T: DeserializeOwned>(graph: &Graph, first: &Ref) -> Result<Vec<T>, Error> {
    graph
        .rows(first)?
        .into_iter()
        .map(|row| serde_json::from_value(row.clone()).map_err(|_| Error::Encoding))
        .collect()
}

fn cases(
    graph: &Graph,
    root: &ReceiptV1,
    prepared: &Prepared,
    pass: bool,
) -> Result<Summary, Error> {
    let plans: BTreeMap<&str, &CasePlan> = prepared
        .cases
        .iter()
        .map(|case| (case.case_id.as_str(), case))
        .collect();
    if plans.len() != prepared.cases.len() || plans.is_empty() {
        return Err(Error::CasePlan);
    }
    let observed: Vec<CaseV1> = rows(graph, root.cases.inventory.as_ref())?;
    if observed.len() != plans.len() {
        return Err(Error::CasePlan);
    }
    let mut counts = [0_u32; 12];
    let mut mandatory = 0_u32;
    let mut credit = 0_u32;
    let mut criteria = BTreeSet::new();
    for case in &observed {
        let plan = plans.get(case.case_id.as_str()).ok_or(Error::CasePlan)?;
        if !matches_plan(case, plan) {
            return Err(Error::CasePlan);
        }
        counts[0] += 1;
        counts[1] += u32::from(case.selected);
        counts[2] += u32::from(case.executed);
        let outcome = if case.excluded {
            10
        } else {
            match case.outcome {
                CaseV1Outcome::Passed => 3,
                CaseV1Outcome::Failed => 4,
                CaseV1Outcome::Skipped => 5,
                CaseV1Outcome::Ignored => 6,
                CaseV1Outcome::Broken => 7,
                CaseV1Outcome::Timeout => 8,
                CaseV1Outcome::Invalid => 9,
                CaseV1Outcome::Unmeasured => 11,
            }
        };
        counts[outcome] += 1;
        let reviewed = reviewed_case(graph, case, plan)?;
        if case.mandatory {
            mandatory += 1;
            if pass && (!case.executed || case.outcome != CaseV1Outcome::Passed || !reviewed) {
                return Err(Error::RequiredCase);
            }
        }
        if case.executed && case.outcome == CaseV1Outcome::Passed {
            let expected: ExpectationV1 = typed(graph, &case.expected)?;
            if expected.oracle_id != case.oracle_id
                || case.detector_id != expected.intended_detector
                || !producer_matches(&case.producer_exit_or_signal, &expected.expected_producer)
            {
                return Err(Error::Producer);
            }
            if case.primary_module_id == root.identity.module_id && reviewed {
                credit += 1;
            }
            if case.mandatory {
                criteria.extend(case.criterion_ids.as_slice().iter().map(Name::as_str));
            }
        }
    }
    let c = &root.cases;
    if counts
        != [
            c.discovered,
            c.selected,
            c.executed,
            c.passed,
            c.failed,
            c.skipped,
            c.ignored,
            c.broken,
            c.timed_out,
            c.invalid,
            c.excluded,
            c.unmeasured,
        ]
        || credit != c.primary_credit
    {
        return Err(Error::CaseCounts);
    }
    if pass
        && (mandatory == 0
            || root
                .identity
                .criterion_ids
                .as_slice()
                .iter()
                .any(|id| !criteria.contains(id.as_str())))
    {
        return Err(Error::RequiredCase);
    }
    Ok(Summary {
        cases: counts[0],
        primary_credit: credit,
        mandatory_cases: mandatory,
    })
}

fn matches_plan(case: &CaseV1, plan: &CasePlan) -> bool {
    case.primary_module_id == plan.primary_module_id
        && case.criterion_ids == plan.criterion_ids
        && case.fixture_sha256 == plan.fixture_sha256
        && case.oracle_id == plan.oracle_id
        && case.expected == plan.expected
        && case.mandatory == plan.mandatory
        && case.selected == plan.selected
        && case.excluded == plan.excluded
}

pub(crate) fn reviewed_case(graph: &Graph, case: &CaseV1, plan: &CasePlan) -> Result<bool, Error> {
    let Some(reference) = &plan.reviewed_design else {
        return Ok(false);
    };
    if !case
        .raw_evidence_refs
        .as_slice()
        .contains(reference.as_ref())
    {
        return Err(Error::CasePlan);
    }
    let review: ReviewV1 = typed(graph, reference)?;
    if review.subject_sha256 != plan.expected.as_ref().sha256
        || review.action.as_str() != "case_design_review"
        || review.disposition.as_str() != "accepted"
    {
        return Err(Error::CasePlan);
    }
    let findings: Vec<FindingV1> = rows(graph, review.findings.as_ref())?;
    let obligations: Vec<ObligationV1> = rows(graph, review.residual_obligations.as_ref())?;
    Ok(!findings.iter().any(|finding| {
        finding.material
            && matches!(
                finding.disposition,
                FindingV1Disposition::Open | FindingV1Disposition::AcceptedResidual
            )
    }) && !obligations
        .iter()
        .any(|obligation| obligation.material && obligation.state != ObligationV1State::Settled))
}

fn producer_matches(actual: &ProducerV1, expected: &ExpectedProducerV1) -> bool {
    let status = match expected.status {
        ExpectedProducerV1Status::Exited => ProducerV1Status::Exited,
        ExpectedProducerV1Status::Signalled => ProducerV1Status::Signalled,
    };
    actual.status == status
        && actual.exit_code.value == expected.exit_code.value
        && actual.signal.value == expected.signal.value
        && !actual.timeout
}

fn diagnostics(graph: &Graph, root: &ReceiptV1, pass: bool) -> Result<(), Error> {
    let rows: Vec<DiagnosticV1> = rows(graph, root.diagnostics.by_tool.as_ref())?;
    let mut warnings = 0_u32;
    let mut errors = 0_u32;
    for row in rows {
        if row.baseline != root.diagnostics.baseline {
            return Err(Error::Diagnostic);
        }
        warnings = warnings
            .checked_add(row.warning_count)
            .ok_or(Error::Diagnostic)?;
        errors = errors
            .checked_add(row.error_count)
            .ok_or(Error::Diagnostic)?;
        if pass
            && (row.stdout_truncated
                || row.stderr_truncated
                || row.baseline
                    && (row.producer.status != ProducerV1Status::Exited
                        || row.producer.exit_code.value != Some(0)
                        || row.producer.timeout))
        {
            return Err(Error::Diagnostic);
        }
    }
    if warnings != root.diagnostics.warning_count || errors != root.diagnostics.error_count {
        return Err(Error::Diagnostic);
    }
    Ok(())
}

fn artifacts(graph: &Graph, root: &ReceiptV1, pass: bool) -> Result<(), Error> {
    let inventory: Vec<ArtifactV1> = rows(graph, root.artifacts.inventory.as_ref())?;
    let mut bytes = 0_u64;
    for row in &inventory {
        let schema = row.object.schema_id.as_str();
        if schema.ends_with("PageV1")
            || [
                ReceiptV1::SCHEMA_ID,
                ReviewReceiptV1::SCHEMA_ID,
                AvailabilityReceiptV1::SCHEMA_ID,
            ]
            .contains(&schema)
        {
            return Err(Error::Artifact);
        }
        bytes = bytes
            .checked_add(u64::from(row.object.byte_length))
            .ok_or(Error::Artifact)?;
        if pass
            && row.required
            && (row.truncated || row.availability != ArtifactV1Availability::Available)
        {
            return Err(Error::Artifact);
        }
    }
    if pass {
        let mut required = Vec::new();
        required.extend(
            root.observations
                .producer
                .stdout
                .value
                .iter()
                .map(AsRef::as_ref),
        );
        required.extend(
            root.observations
                .producer
                .stderr
                .value
                .iter()
                .map(AsRef::as_ref),
        );
        required.extend(
            root.subjects
                .seed_to_result_patch
                .value
                .iter()
                .map(AsRef::as_ref),
        );
        let result = root
            .subjects
            .result_subject
            .value
            .as_ref()
            .ok_or(Error::Artifact)?;
        let subject: SubjectV1 = typed(graph, result)?;
        let files: Vec<SubjectFileV1> = rows(graph, subject.files.as_ref())?;
        required.extend(
            files
                .iter()
                .filter_map(|file| file.content.value.as_ref().map(AsRef::as_ref)),
        );
        if required.iter().any(|reference| {
            !inventory.iter().any(|row| {
                &row.object == *reference
                    && row.required
                    && !row.truncated
                    && row.availability == ArtifactV1Availability::Available
            })
        }) {
            return Err(Error::Artifact);
        }
    }
    if inventory.len() != root.artifacts.count as usize || bytes != root.artifacts.total_bytes.get()
    {
        return Err(Error::Artifact);
    }
    Ok(())
}

fn mutations(graph: &Graph, root: &ReceiptV1) -> Result<(), Error> {
    let campaigns: Vec<CampaignV1> = rows(graph, root.mutation.campaigns.as_ref())?;
    let mut counts = [0_u32; 8];
    let mut ids = BTreeSet::new();
    for campaign in &campaigns {
        let mutants: Vec<MutantV1> = rows(graph, campaign.mutants.as_ref())?;
        if mutants.len() != campaign.planned_mutants as usize {
            return Err(Error::Mutation);
        }
        let baseline: ReceiptV1 = typed(graph, &campaign.baseline_receipt)?;
        if baseline.identity.run_id == root.identity.run_id
            || !baseline.diagnostics.baseline
            || baseline.verdict.state != VerdictV1State::PassCandidate
        {
            return Err(Error::Mutation);
        }
        for mutant in mutants {
            if !ids.insert(mutant.mutant_id.as_str().to_owned())
                || mutant.campaign_id != campaign.campaign_id
                || baseline
                    .subjects
                    .result_subject
                    .value
                    .as_ref()
                    .map(|subject| &subject.as_ref().sha256)
                    != Some(&mutant.baseline_subject_sha256)
            {
                return Err(Error::Mutation);
            }
            counts[0] += 1;
            let index = match mutant.outcome {
                MutantV1Outcome::Caught => 1,
                MutantV1Outcome::Survived => 2,
                MutantV1Outcome::Timeout => 3,
                MutantV1Outcome::Unviable => 4,
                MutantV1Outcome::ReviewedEquivalent => 5,
                MutantV1Outcome::Excluded => 6,
                MutantV1Outcome::Unmeasured => 7,
            };
            counts[index] += 1;
            if let Some(review) = &mutant.review_ref.value {
                let review: ReviewV1 = typed(graph, review)?;
                if review.action.as_str() != "mutation_disposition"
                    || review.subject_sha256 != mutant.diff.as_ref().sha256
                {
                    return Err(Error::Mutation);
                }
            }
        }
    }
    let m = &root.mutation;
    if campaigns.len() != m.campaign_count as usize
        || counts
            != [
                m.distinct_mutants,
                m.caught,
                m.survived,
                m.timed_out,
                m.unviable,
                m.equivalent,
                m.excluded,
                m.unmeasured,
            ]
    {
        return Err(Error::Mutation);
    }
    Ok(())
}

fn clocks(graph: &Graph, root: &ReceiptV1, pass: bool) -> Result<(), Error> {
    let limits: LimitsV1 = typed(graph, &root.invocation.limits)?;
    let cleanup: CleanupContractV1 = typed(graph, &root.invocation.cleanup_contract)?;
    if limits.cleanup_deadline_ms != cleanup.deadline_ms
        || limits.term_grace_ms != cleanup.term_grace_ms
        || limits.cleanup_deadline_ms.get() > limits.wall_ms.get()
    {
        return Err(Error::Clock);
    }
    if pass {
        let elapsed = root
            .observations
            .end_monotonic_ns
            .get()
            .checked_sub(root.observations.start_monotonic_ns.get())
            .ok_or(Error::Clock)?;
        let limit = limits
            .wall_ms
            .get()
            .checked_mul(1_000_000)
            .ok_or(Error::Clock)?;
        if elapsed > limit {
            return Err(Error::Clock);
        }
    }
    Ok(())
}
