//! Fixed pre-execution metadata for two independently reviewed u64 attempts.
use crate::{support, u64_receipt};
use habitat_engine::app::{evidence::Evidence, subjects, workload};
use habitat_engine::check::{
    collector::{self, Publisher},
    consistency::{CasePlan, Prepared},
    graph::Graph,
};
use habitat_engine::contracts::receipt::{self as r, Validate};
use habitat_engine::worker::{resources::Scope, workspace::Snapshot};
use std::collections::BTreeSet;
use std::time::Instant;

pub const EXPECTATION_SHA: &str =
    "sha256:3a7faa5510790c20322ad5829091211eb8c04ab6016e16ec8391433dae3392b9";
pub const REVIEW_SHA: &str =
    "sha256:f288225476120254f5c3a93266fc8f2a8763810462fbddd7c62161107cb39adb";
pub const RECIPE_SHA: &str =
    "sha256:8585e9e9d2e193e46545a6710a221dd9165285b055e7d1f1ec805674bcbcba1f";
pub const PATCH_SHA: &str =
    "sha256:12db53d34f3a388d19d1b971d2bd5dfeb67f8c0e1a459d25e1012954c7b22f8e";

#[derive(Debug)]
pub enum ErrorKind {
    Deadline,
    Missing,
    Binding,
    Bound,
    Encoding,
    Io,
    Publication(collector::Error),
    Subject(subjects::Error),
    Receipt(u64_receipt::Error),
}
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub attempted: Vec<r::Ref>,
}
pub(crate) fn fail(kind: ErrorKind) -> Error {
    Error {
        kind,
        attempted: Vec::new(),
    }
}
pub(crate) fn budget(deadline: Instant) -> Result<(), Error> {
    if Instant::now() >= deadline {
        Err(fail(ErrorKind::Deadline))
    } else {
        Ok(())
    }
}
pub(crate) fn scalar<T>(value: Result<T, r::Error>) -> Result<T, Error> {
    value.map_err(|_| fail(ErrorKind::Encoding))
}
pub(crate) fn name(value: &str) -> Result<r::Name, Error> {
    scalar(r::Name::new(value))
}
pub(crate) fn text(value: impl Into<String>) -> Result<r::Text, Error> {
    scalar(r::Text::new(value))
}
pub(crate) fn count(value: u64) -> Result<r::U64, Error> {
    scalar(r::U64::new(value.to_string()))
}
pub(crate) fn list<T: Validate>(value: Vec<T>) -> Result<r::List<T>, Error> {
    scalar(r::List::new(value))
}
pub(crate) fn none<T>(why: &str) -> Result<r::Maybe<T>, Error> {
    Ok(r::Maybe::unavailable(text(why)?))
}
pub(crate) fn record<T: r::ReceiptRecord>(
    e: &mut Evidence<'_>,
    value: &T,
) -> Result<r::TypedRef<T>, Error> {
    let mut p = Publisher::new(e);
    p.record(value).map_err(|error| Error {
        kind: ErrorKind::Publication(error),
        attempted: p.attempted_refs().to_vec(),
    })
}
pub(crate) fn raw(e: &mut Evidence<'_>, bytes: &[u8], media: &str) -> Result<r::Payload, Error> {
    if bytes.is_empty() {
        return Err(fail(ErrorKind::Missing));
    }
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(fail(ErrorKind::Bound));
    }
    e.payload(bytes, media)
        .map_err(|error| fail(ErrorKind::Publication(collector::Error::Sink(error))))
}
pub(crate) fn json_raw(
    e: &mut Evidence<'_>,
    value: &impl serde::Serialize,
) -> Result<r::Payload, Error> {
    raw(
        e,
        &serde_json::to_vec(value).map_err(|_| fail(ErrorKind::Encoding))?,
        "application/json",
    )
}
macro_rules! page {
    ($e:expr,$ty:ident,$rows:expr) => {{
        let rows = $rows;
        let n = u32::try_from(rows.len()).map_err(|_| fail(ErrorKind::Bound))?;
        record(
            $e,
            &r::$ty {
                page_index: 0,
                page_count: 1,
                row_count: n,
                total_rows: n,
                rows: list(rows)?,
                next: none("end_of_inventory")?,
            },
        )?
    }};
}
pub(crate) use page;

pub struct Sources<'a> {
    pub baseline: &'a Snapshot,
    pub repaired: &'a Snapshot,
    pub protected: &'a Snapshot,
    pub fixtures: &'a Snapshot,
    pub oracle: &'a Snapshot,
    pub harness: &'a Snapshot,
    pub collector: &'a Snapshot,
    pub launcher: &'a Snapshot,
    pub reference_patch: &'a [u8],
}
pub struct Reviewed {
    pub expectation: r::TypedRef<r::ExpectationV1>,
    pub recipe: r::Payload,
    pub review: r::TypedRef<r::ReviewV1>,
    pub provenance: r::Payload,
}
pub struct AttemptIds {
    pub run: r::Id,
    pub attempt: r::Id,
    pub session: r::Id,
    pub workspace: r::Id,
    pub scopes: [Scope; 3],
    pub argv: r::List<r::Text>,
}
pub struct Inputs<'a> {
    pub task: r::Id,
    pub attempts: [AttemptIds; 2],
    pub sources: Sources<'a>,
    pub tools: &'a workload::Tools,
    pub reviewed: Reviewed,
    pub support: support::Inputs<'a>,
}
pub struct PreparedAttempt {
    pub prepared: Prepared,
    pub recipe: r::Payload,
    pub host: r::TypedRef<r::HostV1>,
    pub review_provenance: r::Payload,
    pub session: String,
    pub workspace: String,
    pub scopes: [Scope; 3],
}

/// Publish two frozen plans. Caller retains Evidence and all publication effects.
/// # Errors
/// Refuses absent/currently inconsistent evidence, identities, source or tool pins.
pub fn prepare_pair(
    e: &mut Evidence<'_>,
    inputs: &Inputs<'_>,
    deadline: Instant,
) -> Result<[PreparedAttempt; 2], Error> {
    budget(deadline)?;
    validate_ids(&inputs.task, &inputs.attempts)?;
    validate_scope_tools(&inputs.attempts, inputs.support.tools)?;
    validate_review(e, &inputs.reviewed)?;
    if !u64_receipt::repair_pair(inputs.sources.reference_patch, inputs.sources.repaired) {
        return Err(fail(ErrorKind::Binding));
    }
    let shared = support::publish(e, &inputs.support, inputs.tools, &inputs.sources, deadline)?;
    let baseline = publish_subject(e, inputs.sources.baseline, deadline)?;
    let repaired = publish_subject(e, inputs.sources.repaired, deadline)?;
    let fixtures = publish_subject(e, inputs.sources.fixtures, deadline)?;
    let oracle = publish_subject(e, inputs.sources.oracle, deadline)?;
    let harness = publish_subject(e, inputs.sources.harness, deadline)?;
    let collector = publish_subject(e, inputs.sources.collector, deadline)?;
    let launcher = publish_subject(e, inputs.sources.launcher, deadline)?;
    let empty = e
        .payload(b"", "application/octet-stream")
        .map_err(|error| fail(ErrorKind::Publication(collector::Error::Sink(error))))?;
    let patch = raw(e, inputs.sources.reference_patch, "text/x-diff")?;
    let common = Roles {
        baseline,
        repaired,
        fixtures,
        oracle,
        harness,
        collector,
        launcher,
        empty,
        patch,
    };
    let first = attempt(e, inputs, 0, &shared, &common, deadline)?;
    let second = attempt(e, inputs, 1, &shared, &common, deadline)?;
    budget(deadline)?;
    Ok([first, second])
}
fn publish_subject(
    e: &mut Evidence<'_>,
    source: &Snapshot,
    deadline: Instant,
) -> Result<r::TypedRef<r::SubjectV1>, Error> {
    subjects::publish(e, source, r::SubjectFileV1Origin::Authored, deadline).map_err(|error| {
        Error {
            attempted: error.attempted.clone(),
            kind: ErrorKind::Subject(error),
        }
    })
}
struct Roles {
    baseline: r::TypedRef<r::SubjectV1>,
    repaired: r::TypedRef<r::SubjectV1>,
    fixtures: r::TypedRef<r::SubjectV1>,
    oracle: r::TypedRef<r::SubjectV1>,
    harness: r::TypedRef<r::SubjectV1>,
    collector: r::TypedRef<r::SubjectV1>,
    launcher: r::TypedRef<r::SubjectV1>,
    empty: r::Payload,
    patch: r::Payload,
}
fn attempt(
    e: &Evidence<'_>,
    input: &Inputs<'_>,
    index: usize,
    shared: &support::Published,
    roles: &Roles,
    deadline: Instant,
) -> Result<PreparedAttempt, Error> {
    let ids = &input.attempts[index];
    let criteria = list(vec![name("u64-frozen-exact-output")?])?;
    let identity = r::IdentityV1 {
        run_id: ids.run.clone(),
        task_id: input.task.clone(),
        attempt_id: ids.attempt.clone(),
        generation: scalar(r::Generation::new((index + 1).to_string()))?,
        module_id: name("check")?,
        criterion_ids: criteria.clone(),
        profile_id: name("T06-u64-fixed-runtime-THDEV")?,
        parent_run: none("separate_attempt_same_task")?,
    };
    let subjects = r::SubjectsV1 {
        seed_subject: roles.baseline.clone(),
        result_subject: r::Maybe::present(if index == 0 {
            roles.baseline.clone()
        } else {
            roles.repaired.clone()
        }),
        seed_to_result_patch: r::Maybe::present(if index == 0 {
            roles.empty.clone()
        } else {
            roles.patch.clone()
        }),
        fixtures: roles.fixtures.clone(),
        oracle: roles.oracle.clone(),
        harness: roles.harness.clone(),
        collector: roles.collector.clone(),
        launcher: roles.launcher.clone(),
        locks: shared.locks.clone(),
        toolchain: shared.toolchain.clone(),
        target_features_build_profile: shared.build.clone(),
        standards: shared.standards.clone(),
        isolation_profile: shared.isolation.clone(),
    };
    let invocation = r::InvocationV1 {
        argv: ids.argv.clone(),
        cwd_logical: name("work")?,
        environment: shared.environment.clone(),
        grants: shared.grants.clone(),
        expected: input.reviewed.expectation.clone(),
        oracle_id: name("ORACLE-U64-001/v1")?,
        limits: shared.limits.clone(),
        allowed_effects: shared.effects.clone(),
        cleanup_contract: shared.cleanup.clone(),
    };
    let prepared = Prepared {
        schema_sha256: shared.schema.as_ref().sha256.clone(),
        identity,
        subjects,
        invocation,
        cases: vec![CasePlan {
            case_id: name("WL-U64-PARSE-001-v1")?,
            primary_module_id: name("check")?,
            criterion_ids: criteria,
            fixture_sha256: roles.fixtures.as_ref().sha256.clone(),
            oracle_id: name("ORACLE-U64-001/v1")?,
            expected: input.reviewed.expectation.clone(),
            mandatory: true,
            selected: true,
            excluded: false,
            reviewed_design: Some(input.reviewed.review.clone()),
        }],
    };
    u64_receipt::prepare(
        e,
        u64_receipt::Preparation {
            prepared: &prepared,
            sources: source_bindings(&input.sources, index),
            protected: input.sources.protected,
            tools: input.tools,
            patch: if index == 0 {
                u64_receipt::Patch::Baseline
            } else {
                u64_receipt::Patch::Repair(input.sources.reference_patch)
            },
            recipe: input.reviewed.recipe.clone(),
            host: shared.host.clone(),
            review_provenance: input.reviewed.provenance.clone(),
            deadline,
        },
    )
    .map_err(|error| fail(ErrorKind::Receipt(error)))?;
    Ok(PreparedAttempt {
        prepared,
        recipe: input.reviewed.recipe.clone(),
        host: shared.host.clone(),
        review_provenance: input.reviewed.provenance.clone(),
        session: ids.session.as_str().to_owned(),
        workspace: ids.workspace.as_str().to_owned(),
        scopes: ids.scopes.clone(),
    })
}
fn validate_ids(task: &r::Id, attempts: &[AttemptIds; 2]) -> Result<(), Error> {
    scalar(task.validate())?;
    let mut used = BTreeSet::from([task.as_str()]);
    for attempt in attempts {
        for id in [
            &attempt.run,
            &attempt.attempt,
            &attempt.session,
            &attempt.workspace,
        ] {
            scalar(id.validate())?;
            if !used.insert(id.as_str()) {
                return Err(fail(ErrorKind::Binding));
            }
        }
        if attempt.argv.as_slice().is_empty() {
            return Err(fail(ErrorKind::Missing));
        }
        let aggregate = format!(
            "hee3aggregate{}.slice",
            attempt.run.as_str().replace('-', "")
        );
        for scope in &attempt.scopes {
            scalar(r::Id::new(&scope.run_id))?;
            if !used.insert(&scope.run_id)
                || scope.aggregate != aggregate
                || !scope.systemd_run.is_absolute()
                || !scope.runtime_dir.is_absolute()
            {
                return Err(fail(ErrorKind::Binding));
            }
            // Scope uses the same canonical prefixed digest as the worker launcher.
            scalar(r::Sha::new(&scope.systemd_run_sha256))?;
        }
    }
    Ok(())
}
fn validate_review(e: &Evidence<'_>, input: &Reviewed) -> Result<(), Error> {
    if input.expectation.as_ref().sha256.as_str() != EXPECTATION_SHA
        || input.review.as_ref().sha256.as_str() != REVIEW_SHA
        || input.recipe.as_ref().sha256.as_str() != RECIPE_SHA
    {
        return Err(fail(ErrorKind::Binding));
    }
    let graph = Graph::resolve(e, input.review.as_ref()).map_err(|_| fail(ErrorKind::Binding))?;
    for reference in [
        input.expectation.as_ref(),
        input.recipe.as_ref(),
        input.provenance.as_ref(),
    ] {
        if graph
            .get(reference)
            .map_err(|_| fail(ErrorKind::Binding))?
            .bytes()
            .is_empty()
        {
            return Err(fail(ErrorKind::Missing));
        }
    }
    let review: r::ReviewV1 = scalar(r::decode(
        graph
            .get(input.review.as_ref())
            .map_err(|_| fail(ErrorKind::Binding))?
            .bytes(),
    ))?;
    if review.action.as_str() != "case_design_review"
        || review.disposition.as_str() != "accepted"
        || review.subject_sha256 != input.expectation.as_ref().sha256
    {
        return Err(fail(ErrorKind::Binding));
    }
    let recipe: u64_receipt::Recipe = serde_json::from_slice(
        graph
            .get(input.recipe.as_ref())
            .map_err(|_| fail(ErrorKind::Binding))?
            .bytes(),
    )
    .map_err(|_| fail(ErrorKind::Encoding))?;
    if (
        recipe.candidate_cutoff_ms,
        recipe.verification_cutoff_ms,
        recipe.total_wall_ms,
    ) != (900_000, 1_190_000, 1_200_000)
    {
        return Err(fail(ErrorKind::Binding));
    }
    Ok(())
}

fn validate_scope_tools(
    attempts: &[AttemptIds; 2],
    tools: &[support::ToolInput<'_>],
) -> Result<(), Error> {
    let tool = tools
        .iter()
        .find(|tool| tool.id == "systemd-run")
        .ok_or_else(|| fail(ErrorKind::Missing))?;
    let digest = format!("sha256:{}", support::hex(&tool.sha256));
    for attempt in attempts {
        for scope in &attempt.scopes {
            if scope.systemd_run != tool.path || digest != scope.systemd_run_sha256 {
                return Err(fail(ErrorKind::Binding));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/prepare_controls.rs"]
mod tests;

fn source_bindings<'a>(sources: &Sources<'a>, index: usize) -> u64_receipt::SourceBindings<'a> {
    let binding = |snapshot| u64_receipt::SubjectBinding {
        snapshot,
        origin: r::SubjectFileV1Origin::Authored,
    };
    u64_receipt::SourceBindings {
        seed: binding(sources.baseline),
        result: binding(if index == 0 {
            sources.baseline
        } else {
            sources.repaired
        }),
        fixtures: binding(sources.fixtures),
        oracle: binding(sources.oracle),
        harness: binding(sources.harness),
        collector: binding(sources.collector),
        launcher: binding(sources.launcher),
    }
}
