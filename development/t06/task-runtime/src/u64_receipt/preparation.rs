use super::{
    BoundSource, Error, ErrorKind, Evidence, Frozen, Instant, Patch, Preparation, Recipe, Snapshot,
    WRAPPER_SHA, budget, bytes, consistency, d, fail, file, graph, hash, json_raw, list, name,
    none, r, raw, record, scalar, subjects, typed, u64_oracle,
};

pub struct FrozenExpectation {
    pub reference: r::TypedRef<r::ExpectationV1>,
    pub specification: r::Payload,
    pub wrapper: r::Payload,
    pub recipe: r::Payload,
}
fn valid_recipe(value: &Recipe) -> bool {
    value.candidate_cutoff_ms > 0
        && value.candidate_cutoff_ms < value.verification_cutoff_ms
        && value.verification_cutoff_ms < value.total_wall_ms
}
/// Publish the exact independently frozen oracle and explicit black-box assumptions.
/// This returns a subject for independent review; it creates no approval.
/// # Errors
/// Refuses altered oracle/wrapper, invalid cutoffs, changed snapshots or failed publication.
pub fn publish_expectation(
    evidence: &mut Evidence<'_>,
    protected: &Snapshot,
    recipe: &Recipe,
    deadline: Instant,
) -> Result<FrozenExpectation, Error> {
    budget(deadline)?;
    protected
        .readback_source(deadline)
        .map_err(|_| fail(ErrorKind::Binding))?;
    let oracle_bytes = file(protected, "oracle.json")?;
    u64_oracle::FrozenOracle::from_bytes(oracle_bytes).map_err(|_| fail(ErrorKind::Binding))?;
    let wrapper_bytes = file(protected, "public-wrapper.rs")?;
    if !valid_recipe(recipe) || hash(wrapper_bytes) != WRAPPER_SHA {
        return Err(fail(ErrorKind::Recipe));
    }
    let specification = raw(evidence, oracle_bytes)?;
    let wrapper = raw(evidence, wrapper_bytes)?;
    let recipe = json_raw(evidence, recipe)?;
    let assumption = r::AssumptionV1 {
        assumption_id: name("u64-fixed-black-box-protocol")?,
        shared_with: list(vec![
            name("candidate-public-wrapper")?,
            name("trusted-in-process-oracle")?,
        ])?,
        statement: scalar(r::Reason::new(
            "The 335 literal/arbitrary-precision vectors are finite black-box workload observations, not engine case credit. The public wrapper contains no expected answers but shares a process with the candidate library and cannot authenticate hostile same-process output. The coordinator evaluates exact ordered UTF-8 protocol bytes against this independently frozen oracle; this does not qualify hostile containment or general correctness.",
        ))?,
        evidence: list(vec![
            specification.as_ref().clone(),
            wrapper.as_ref().clone(),
            recipe.as_ref().clone(),
        ])?,
    };
    let shared_assumptions = record(
        evidence,
        &r::AssumptionPageV1 {
            page_index: 0,
            page_count: 1,
            row_count: 1,
            total_rows: 1,
            rows: list(vec![assumption])?,
            next: none("end_of_inventory")?,
        },
    )?;
    let expectation = r::ExpectationV1 {
        oracle_id: name("ORACLE-U64-001/v1")?,
        oracle_class: r::ExpectationV1OracleClass::Reference,
        specification: specification.clone(),
        expected_producer: r::ExpectedProducerV1 {
            status: r::ExpectedProducerV1Status::Exited,
            exit_code: r::Maybe::present(0),
            signal: none("ordinary_exit_expected")?,
        },
        expected_oracle: r::ExpectationV1ExpectedOracle::Satisfied,
        intended_detector: name("u64-exact-output-agreement")?,
        shared_assumptions,
    };
    let reference = record(evidence, &expectation)?;
    protected
        .readback_source(deadline)
        .map_err(|_| fail(ErrorKind::Binding))?;
    Ok(FrozenExpectation {
        reference,
        specification,
        wrapper,
        recipe,
    })
}

pub(super) fn prepare(evidence: &Evidence<'_>, input: Preparation<'_>) -> Result<Frozen, Error> {
    budget(input.deadline)?;
    let p = input.prepared;
    let (expectation, oracle_bytes, recipe) = expected_inputs(evidence, &input)?;
    let review = p.cases[0]
        .reviewed_design
        .as_ref()
        .ok_or_else(|| fail(ErrorKind::Review))?;
    validate_review(
        evidence,
        review,
        &p.invocation.expected,
        &input.review_provenance,
    )?;
    typed(evidence, &input.host)?;
    validate_patch(evidence, &input)?;
    let refs = [
        p.subjects.seed_subject.clone(),
        p.subjects
            .result_subject
            .value
            .clone()
            .ok_or_else(|| fail(ErrorKind::Binding))?,
        p.subjects.fixtures.clone(),
        p.subjects.oracle.clone(),
        p.subjects.harness.clone(),
        p.subjects.collector.clone(),
        p.subjects.launcher.clone(),
    ];
    let identities = [
        d::Identity::Seed,
        d::Identity::Result,
        d::Identity::Fixtures,
        d::Identity::Oracle,
        d::Identity::Harness,
        d::Identity::Collector,
        d::Identity::Launcher,
    ];
    let bindings = [
        input.sources.seed,
        input.sources.result,
        input.sources.fixtures,
        input.sources.oracle,
        input.sources.harness,
        input.sources.collector,
        input.sources.launcher,
    ];
    let mut sources = Vec::with_capacity(7);
    for ((binding, reference), identity) in bindings.into_iter().zip(refs).zip(identities) {
        subjects::verify(
            evidence,
            binding.snapshot,
            binding.origin,
            &reference,
            input.deadline,
        )
        .map_err(|e| fail(ErrorKind::Source(e)))?;
        sources.push(BoundSource {
            snapshot: binding.snapshot.clone(),
            reference,
            origin: binding.origin,
            identity,
        });
    }
    input
        .protected
        .readback_source(input.deadline)
        .map_err(|_| fail(ErrorKind::Binding))?;
    for reference in support_refs(p) {
        graph::Graph::resolve(evidence, reference).map_err(|e| fail(ErrorKind::Graph(e)))?;
    }
    let tool_pins = super::binding::tool_pins(input.tools)?;
    Ok(Frozen {
        prepared: p.clone(),
        sources,
        protected: input.protected.clone(),
        oracle_bytes,
        recipe,
        recipe_ref: input.recipe,
        host: input.host,
        provenance: input.review_provenance,
        expectation,
        tool_pins,
        compiler_pin: (
            input.tools.compiler.namespace.clone(),
            input.tools.compiler.sha256,
        ),
    })
}
fn same_content(left: &Snapshot, right: &Snapshot) -> bool {
    left.entries()
        .map(|e| (&e.path, &e.content))
        .eq(right.entries().map(|e| (&e.path, &e.content)))
}
pub(super) fn support_refs(p: &consistency::Prepared) -> [&r::Ref; 4] {
    [
        p.subjects.locks.as_ref(),
        p.subjects.toolchain.as_ref(),
        p.subjects.target_features_build_profile.as_ref(),
        p.subjects.standards.as_ref(),
    ]
}
fn validate_review(
    evidence: &Evidence<'_>,
    reference: &r::TypedRef<r::ReviewV1>,
    expected: &r::TypedRef<r::ExpectationV1>,
    provenance: &r::Payload,
) -> Result<(), Error> {
    let closure = graph::Graph::resolve(evidence, reference.as_ref())
        .map_err(|e| fail(ErrorKind::Graph(e)))?;
    let review: r::ReviewV1 = scalar(r::decode(
        closure
            .get(reference.as_ref())
            .map_err(|e| fail(ErrorKind::Graph(e)))?
            .bytes(),
    ))?;
    if review.action.as_str() != "case_design_review"
        || review.disposition.as_str() != "accepted"
        || review.subject_sha256 != expected.as_ref().sha256
        || review.reviewer.as_str().trim().is_empty()
        || closure
            .get(provenance.as_ref())
            .map_err(|_| fail(ErrorKind::Review))?
            .bytes()
            .is_empty()
    {
        return Err(fail(ErrorKind::Review));
    }
    for row in closure
        .rows(review.findings.as_ref())
        .map_err(|e| fail(ErrorKind::Graph(e)))?
    {
        let finding: r::FindingV1 =
            serde_json::from_value(row.clone()).map_err(|_| fail(ErrorKind::Encoding))?;
        if finding.material
            && matches!(
                finding.disposition,
                r::FindingV1Disposition::Open | r::FindingV1Disposition::AcceptedResidual
            )
        {
            return Err(fail(ErrorKind::Review));
        }
    }
    for row in closure
        .rows(review.residual_obligations.as_ref())
        .map_err(|e| fail(ErrorKind::Graph(e)))?
    {
        let value: r::ObligationV1 =
            serde_json::from_value(row.clone()).map_err(|_| fail(ErrorKind::Encoding))?;
        if value.material && value.state != r::ObligationV1State::Settled {
            return Err(fail(ErrorKind::Review));
        }
    }
    Ok(())
}

fn expected_inputs(
    evidence: &Evidence<'_>,
    input: &Preparation<'_>,
) -> Result<(r::ExpectationV1, Vec<u8>, Recipe), Error> {
    let p = input.prepared;
    if p.cases.len() != 1
        || !p.cases[0].mandatory
        || !p.cases[0].selected
        || p.cases[0].excluded
        || p.cases[0].expected != p.invocation.expected
        || p.cases[0].oracle_id != p.invocation.oracle_id
    {
        return Err(fail(ErrorKind::Binding));
    }
    let expectation: r::ExpectationV1 = typed(evidence, &p.invocation.expected)?;
    if expectation.oracle_class != r::ExpectationV1OracleClass::Reference
        || expectation.oracle_id.as_str() != "ORACLE-U64-001/v1"
        || expectation.expected_producer.status != r::ExpectedProducerV1Status::Exited
        || expectation.expected_producer.exit_code.value != Some(0)
        || expectation.expected_producer.signal.value.is_some()
    {
        return Err(fail(ErrorKind::Binding));
    }
    let oracle_bytes = file(input.protected, "oracle.json")?.to_vec();
    u64_oracle::FrozenOracle::from_bytes(&oracle_bytes).map_err(|_| fail(ErrorKind::Binding))?;
    if bytes(evidence, expectation.specification.as_ref())? != oracle_bytes
        || hash(file(input.protected, "public-wrapper.rs")?) != WRAPPER_SHA
    {
        return Err(fail(ErrorKind::Binding));
    }
    let recipe: Recipe = serde_json::from_slice(&bytes(evidence, input.recipe.as_ref())?)
        .map_err(|_| fail(ErrorKind::Recipe))?;
    let limits: r::LimitsV1 = typed(evidence, &p.invocation.limits)?;
    if !valid_recipe(&recipe)
        || limits.wall_ms.get() != recipe.total_wall_ms
        || limits.cleanup_deadline_ms.get() != recipe.total_wall_ms
    {
        return Err(fail(ErrorKind::Recipe));
    }
    let expectation_graph = graph::Graph::resolve(evidence, p.invocation.expected.as_ref())
        .map_err(|e| fail(ErrorKind::Graph(e)))?;
    expectation_graph
        .get(input.recipe.as_ref())
        .map_err(|_| fail(ErrorKind::Binding))?;
    let mut wrapper_bound = false;
    for value in expectation_graph
        .rows(expectation.shared_assumptions.as_ref())
        .map_err(|e| fail(ErrorKind::Graph(e)))?
    {
        let assumption: r::AssumptionV1 =
            serde_json::from_value(value.clone()).map_err(|_| fail(ErrorKind::Encoding))?;
        if assumption.assumption_id.as_str() == "u64-fixed-black-box-protocol"
            && assumption
                .evidence
                .as_slice()
                .contains(input.recipe.as_ref())
        {
            wrapper_bound = assumption.evidence.as_slice().iter().any(|reference| {
                expectation_graph
                    .get(reference)
                    .is_ok_and(|node| hash(node.bytes()) == WRAPPER_SHA)
            });
        }
    }
    if !wrapper_bound {
        return Err(fail(ErrorKind::Binding));
    }
    Ok((expectation, oracle_bytes, recipe))
}

fn validate_patch(evidence: &Evidence<'_>, input: &Preparation<'_>) -> Result<(), Error> {
    let p = input.prepared;
    if hash(file(input.sources.seed.snapshot, "src/lib.rs")?)
        != "6fd50c63a83a1a9cae3b88bcc11ebad723cb5b86c443148d112567cc002cb5df"
    {
        return Err(fail(ErrorKind::Binding));
    }

    let patch = p
        .subjects
        .seed_to_result_patch
        .value
        .as_ref()
        .ok_or_else(|| fail(ErrorKind::Binding))?;
    let actual = bytes(evidence, patch.as_ref())?;
    match &input.patch {
        Patch::Baseline
            if actual.is_empty()
                && same_content(input.sources.seed.snapshot, input.sources.result.snapshot) => {}
        Patch::Repair(expected)
            if actual.as_slice() == *expected
                && super::repair_pair(expected, input.sources.result.snapshot)
                && hash(file(input.sources.seed.snapshot, "src/lib.rs")?)
                    == "6fd50c63a83a1a9cae3b88bcc11ebad723cb5b86c443148d112567cc002cb5df"
                && input
                    .sources
                    .seed
                    .snapshot
                    .entries()
                    .filter(|e| e.path != "src/lib.rs")
                    .map(|e| (&e.path, &e.content))
                    .eq(input
                        .sources
                        .result
                        .snapshot
                        .entries()
                        .filter(|e| e.path != "src/lib.rs")
                        .map(|e| (&e.path, &e.content))) => {}
        _ => return Err(fail(ErrorKind::Binding)),
    }
    Ok(())
}
