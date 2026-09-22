//! Fixed identity and real retained review-graph controls, without workload launch.
use super::*;
use habitat_engine::check::collector::Sink;
use habitat_engine::store::ArtifactStaging;
use std::fs;
use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;
use std::time::Duration;
fn id(value: u32) -> r::Id {
    r::Id::new(format!("10000000-0000-4000-8000-{value:012x}")).unwrap()
}
fn attempt_ids(start: u32) -> AttemptIds {
    let run = id(start);
    let aggregate = format!("hee3aggregate{}.slice", run.as_str().replace('-', ""));
    AttemptIds {
        run,
        attempt: id(start + 1),
        session: id(start + 2),
        workspace: id(start + 3),
        scopes: [0, 1, 2].map(|n| Scope {
            systemd_run: "/usr/bin/systemd-run".into(),
            systemd_run_sha256: format!("sha256:{}", "0".repeat(64)),
            runtime_dir: "/run/user/1000".into(),
            run_id: id(start + 4 + n).as_str().to_owned(),
            aggregate: aggregate.clone(),
        }),
        argv: list(vec![text("literal-test-invocation").unwrap()]).unwrap(),
    }
}
#[test]
fn two_attempt_namespace_dimensions_are_distinct_and_order_independent() {
    let first = attempt_ids(10);
    let second = attempt_ids(20);
    assert!(validate_ids(&id(1), &[first, second]).is_ok());
}
#[test]
fn duplicate_attempt_or_stage_identity_cannot_reuse_an_execution_namespace() {
    let first = attempt_ids(10);
    let mut second = attempt_ids(20);
    second.attempt = first.attempt.clone();
    assert!(validate_ids(&id(1), &[first, second]).is_err());
    let first = attempt_ids(10);
    let mut second = attempt_ids(20);
    second.scopes[2].run_id = first.scopes[0].run_id.clone();
    assert!(validate_ids(&id(1), &[first, second]).is_err());
}
#[test]
fn foreign_aggregate_is_refused_before_publication() {
    let first = attempt_ids(10);
    let mut second = attempt_ids(20);
    second.scopes[1].aggregate = "hee3aggregateforeign.slice".into();
    assert!(matches!(
        validate_ids(&id(1), &[first, second]).err().unwrap().kind,
        ErrorKind::Binding
    ));
}
struct Area(PathBuf);
impl Area {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hee3-u64-review-import-{}",
            habitat_engine::app::evidence::fresh_id(Instant::now() + Duration::from_secs(5))
                .unwrap()
                .as_str()
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn unhex(input: &str) -> Vec<u8> {
    input
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn exact_review_import_resolves_and_substituted_recipe_or_provenance_refuses() {
    let area = Area::new();
    let deadline = Instant::now() + Duration::from_secs(20);
    let staging = ArtifactStaging::open(&area.0, true, deadline).unwrap();
    let mut evidence = Evidence::staged(&staging, deadline);
    let bundle: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/review-bundle.json")).unwrap();
    let existing: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/existing-objects.json")).unwrap();
    for object in existing
        .as_array()
        .unwrap()
        .iter()
        .chain(bundle["objects"].as_array().unwrap())
    {
        let reference: r::Ref = serde_json::from_value(object["reference"].clone()).unwrap();
        evidence
            .publish(&reference, &unhex(object["hex"].as_str().unwrap()))
            .unwrap();
    }
    let request: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/review-request.json")).unwrap();
    let mut reviewed = Reviewed {
        expectation: serde_json::from_value(request["expectation"].clone()).unwrap(),
        recipe: serde_json::from_value(request["recipe"].clone()).unwrap(),
        review: serde_json::from_value(bundle["review"].clone()).unwrap(),
        provenance: serde_json::from_value(bundle["provenance"].clone()).unwrap(),
    };
    assert!(validate_review(&evidence, &reviewed).is_ok());
    reviewed.recipe = evidence.payload(b"{}", "application/json").unwrap();
    assert!(validate_review(&evidence, &reviewed).is_err());
    reviewed.recipe = serde_json::from_value(request["recipe"].clone()).unwrap();
    reviewed.provenance = evidence.payload(b"unrelated", "text/plain").unwrap();
    assert!(validate_review(&evidence, &reviewed).is_err());
    drop(evidence);
    drop(staging);
}

#[test]
fn noncanonical_scope_digests_are_refused_before_publication() {
    for digest in ["0".repeat(64), format!("sha256:sha256:{}", "0".repeat(64))] {
        let first = attempt_ids(10);
        let mut second = attempt_ids(20);
        second.scopes[0].systemd_run_sha256 = digest;
        assert!(matches!(
            validate_ids(&id(1), &[first, second]).err().unwrap().kind,
            ErrorKind::Encoding
        ));
    }
}

#[test]
fn selected_launcher_pin_matches_canonical_scope_and_refuses_neighbor() {
    use habitat_engine::worker::process::{ProcessReport, SignalFacts, Stream};
    let report = ProcessReport {
        started_at: Instant::now(),
        leader_pid: 1,
        exit_code: None,
        signal: None,
        interruption: None,
        interruption_observed_at: None,
        stdout: Stream::default(),
        stderr: Stream::default(),
        elapsed: Duration::ZERO,
        signals: SignalFacts::default(),
        leader_reaped: false,
        process_group_settled: false,
        observer_ready: false,
        pending: None,
    };
    // This guard binds the selected tool, independently of later version checks.
    let tools = [support::ToolInput {
        id: "systemd-run",
        path: std::path::Path::new("/usr/bin/systemd-run"),
        sha256: [0; 32],
        version: &report,
    }];
    let mut attempts = [attempt_ids(10), attempt_ids(20)];
    assert!(validate_scope_tools(&attempts, &tools).is_ok());
    attempts[1].scopes[2].systemd_run_sha256 = format!("sha256:{}", "1".repeat(64));
    assert!(matches!(
        validate_scope_tools(&attempts, &tools).err().unwrap().kind,
        ErrorKind::Binding
    ));
    attempts[1].scopes[2].systemd_run_sha256 = format!("sha256:{}", "0".repeat(64));
    attempts[1].scopes[2].systemd_run = "/other/systemd-run".into();
    assert!(matches!(
        validate_scope_tools(&attempts, &tools).err().unwrap().kind,
        ErrorKind::Binding
    ));
}
