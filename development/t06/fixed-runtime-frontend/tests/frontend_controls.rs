use super::*;
use habitat_engine::contracts::UuidV4;
use habitat_engine::store::{Principal, RequestSource, Store};
use std::time::Duration;
struct Area(PathBuf);
impl Area {
    fn new() -> Self {
        let d = std::env::temp_dir().join(format!(
            "hee3-frontend-{}",
            habitat_engine::app::evidence::fresh_id(Instant::now() + Duration::from_secs(5))
                .unwrap()
                .as_str()
        ));
        private_dir(&d).unwrap();
        Self(d)
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn pin(path: &Path, bytes: &[u8]) -> Pin {
    write(path, bytes).unwrap();
    Pin {
        path: path.into(),
        sha256: digest(bytes),
        bytes: bytes.len() as u64,
    }
}
#[test]
fn exact_fixed_digest_parser_refuses_noncanonical_encodings() {
    assert_eq!(
        digest_array(&format!("sha256:{}", "ab".repeat(32))).unwrap(),
        [0xab; 32]
    );
    for wrong in [
        "ab".repeat(32),
        format!("sha256:{}", "AB".repeat(32)),
        format!("sha256:{}", "a".repeat(63)),
    ] {
        assert!(digest_array(&wrong).is_err());
    }
}
#[test]
fn actual_owned_input_readback_and_substitution_refusal() {
    let a = Area::new();
    let p = pin(&a.0.join("input"), b"known exact bytes");
    assert_eq!(read(&p, 32, deadline()).unwrap(), b"known exact bytes");
    fs::write(&p.path, b"altered same len!").unwrap();
    assert!(read(&p, 32, deadline()).is_err());
}
#[test]
fn symbolic_tool_alias_is_refused_even_when_target_bytes_match() {
    let a = Area::new();
    let mut p = pin(&a.0.join("input"), b"abc");
    std::os::unix::fs::symlink(&p.path, a.0.join("alias")).unwrap();
    p.path = a.0.join("alias");
    assert!(read(&p, 3, deadline()).is_err());
}
#[test]
fn inclusive_byte_bound_and_expired_missing_path_are_distinct() {
    let a = Area::new();
    let p = pin(&a.0.join("input"), b"abc");
    assert!(read(&p, 3, deadline()).is_ok());
    assert!(read(&p, 2, deadline()).is_err());
    let missing = Pin {
        path: a.0.join("never-open"),
        ..p
    };
    assert_eq!(
        read(&missing, 3, Instant::now()).unwrap_err(),
        "original deadline expired"
    );
    assert!(!missing.path.exists());
}
#[test]
fn fixed_agent_toml_is_decoded_and_applied_by_real_single_store() {
    let a = Area::new();
    let d = deadline();
    let principal = Principal::new(rustix::process::geteuid().as_raw(), "operator").unwrap();
    let parse = |s| UuidV4::parse(s).unwrap();
    let mut store = Store::open(
        &a.0,
        parse("10000000-0000-4000-8000-000000000001"),
        parse("10000000-0000-4000-8000-000000000002"),
        true,
        d,
    )
    .unwrap();
    let bytes =
        crate::frontend::roster_bytes("10000000-0000-4000-8000-000000000003", "Luke").unwrap();
    let updates = habitat_engine::roster::parse_changes(&bytes).unwrap();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].definition.capabilities, ["u64-fixed-workload"]);
    assert!(updates[0].definition.endpoint_ref.is_none());
    let applied = store
        .roster_apply(&principal, &updates, RequestSource::Import(&bytes), d)
        .unwrap();
    let read = store
        .roster_get(&principal, parse(&applied[0].head.record_id), d)
        .unwrap();
    assert_eq!(read.head, applied[0].head);
    assert!(read.observation.is_none());
    let replay = store
        .roster_apply(&principal, &updates, RequestSource::Import(&bytes), d)
        .unwrap();
    assert_eq!(replay, applied);
    drop(store);
}
#[test]
fn closed_pin_rejects_unknown_and_duplicate_fields() {
    let baseline = r#"{"path":"/tmp/x","sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","bytes":1}"#;
    assert!(serde_json::from_str::<Pin>(baseline).is_ok());
    for tail in [",\"desired_verdict\":\"PASS\"}", ",\"bytes\":1}"] {
        let bad = format!("{}{}", &baseline[..baseline.len() - 1], tail);
        assert!(serde_json::from_str::<Pin>(&bad).is_err());
    }
}
#[test]
fn source_snapshot_detects_content_and_inventory_changes() {
    let a = Area::new();
    let p = pin(&a.0.join("lib.rs"), b"source");
    let tree = Tree {
        path: a.0.clone(),
        directories: vec![],
        files: vec![TreeFile {
            path: "lib.rs".into(),
            sha256: p.sha256,
            bytes: 6,
            executable: false,
        }],
    };
    assert!(snapshot(&tree, deadline()).is_ok());
    write(&a.0.join("unexpected"), b"x").unwrap();
    assert!(snapshot(&tree, deadline()).is_err());
}

#[test]
fn stopped_and_unsettled_driver_outcomes_cannot_report_frontend_success() {
    use habitat_engine::task::driver::{Outcome, StopReason};
    assert!(crate::frontend::accepted_outcome(Outcome::Accepted).is_ok());
    assert!(crate::frontend::accepted_outcome(Outcome::Stopped(StopReason::WorkerFailed)).is_err());
    assert!(
        crate::frontend::accepted_outcome(Outcome::NeedsSettlement(StopReason::Cancelled)).is_err()
    );
}
#[test]
fn store_readback_error_is_retained_and_propagated() {
    let a = Area::new();
    let out = Area::new();
    let principal = Principal::new(rustix::process::geteuid().as_raw(), "operator").unwrap();
    let parse = |v| UuidV4::parse(v).unwrap();
    let store = Store::open(
        &a.0,
        parse("10000000-0000-4000-8000-000000000001"),
        parse("10000000-0000-4000-8000-000000000002"),
        true,
        deadline(),
    )
    .unwrap();
    let result = crate::frontend::readback(
        &store,
        &principal,
        "10000000-0000-4000-8000-000000000003",
        &out.0,
        Instant::now(),
        false,
    );
    assert!(result.is_err());
    let raw: serde_json::Value =
        serde_json::from_slice(&fs::read(out.0.join("store-readback.json")).unwrap()).unwrap();
    assert!(raw["head"].is_null());
    assert!(raw["head_error"].as_str().unwrap().contains("Deadline"));
    assert!(raw["outbox_error"].is_string());
    drop(store);
}
#[test]
fn actual_unresolved_receipt_graph_is_retained_and_propagated() {
    use habitat_engine::app::evidence::Evidence;
    use habitat_engine::contracts::receipt::Name;
    use habitat_engine::store::ArtifactStaging;
    let a = Area::new();
    let out = Area::new();
    let stage = ArtifactStaging::open(&a.0, true, deadline()).unwrap();
    let mut evidence = Evidence::staged(&stage, deadline());
    let payload = evidence.payload(b"{}", "application/json").unwrap();
    let mut reference = payload.as_ref().clone();
    reference.schema_id = Name::new("hee3.receipt/1:ReceiptV1").unwrap();
    let object = evidence.registered().values().next().unwrap().1.clone();
    let observed = serde_json::json!({"staged_registry":{reference.artifact_id.as_str():[reference.clone(),object]},"executions":[{"receipt":{"reference":reference}}]});
    assert!(crate::frontend::graphs(&stage, &observed, &out.0, deadline()).is_err());
    let raw: serde_json::Value =
        serde_json::from_slice(&fs::read(out.0.join("retained-graphs.json")).unwrap()).unwrap();
    assert!(raw["roots"][0]["error"].is_string());
}
#[test]
fn accepted_path_requires_a_nonempty_receipt_inventory() {
    let a = Area::new();
    let out = Area::new();
    let stage = habitat_engine::store::ArtifactStaging::open(&a.0, true, deadline()).unwrap();
    assert!(
        crate::frontend::graphs(
            &stage,
            &serde_json::json!({"staged_registry":{},"executions":[]}),
            &out.0,
            deadline()
        )
        .is_err()
    );
}
#[test]
fn malformed_manifest_preserves_exact_input_bytes_before_decode_refusal() {
    let a = Area::new();
    let source = a.0.join("manifest.json");
    write(&source, b" \n{\"desired_verdict\":\"PASS\"}\n").unwrap();
    let retained = a.0.join("raw");
    assert!(Manifest::load(&source, &retained, deadline()).is_err());
    assert_eq!(fs::read(&source).unwrap(), fs::read(retained).unwrap());
}

const PREP_TASK: &str = "20000000-0000-4000-8000-000000000003";
const PREP_GENERATION: &str = "20000000-0000-4000-8000-000000000001";
struct PrepRig {
    store: Store,
    area: Area,
    out: Area,
    principal: Principal,
    clock: hee3_fixed_task_runtime::clock::TaskClock,
}
impl PrepRig {
    fn new(admit: bool) -> Self {
        use habitat_engine::{
            contracts::Sha256Digest,
            store::{Allocation, Submission},
        };
        let clock = hee3_fixed_task_runtime::clock::TaskClock::start().unwrap();
        let area = Area::new();
        let out = Area::new();
        let principal = Principal::new(rustix::process::geteuid().as_raw(), "operator").unwrap();
        let mut store = Store::open(
            &area.0,
            UuidV4::parse(PREP_GENERATION).unwrap(),
            UuidV4::parse("20000000-0000-4000-8000-000000000002").unwrap(),
            true,
            clock.deadline(),
        )
        .unwrap();
        if admit {
            store
                .submit(
                    Submission {
                        principal: &principal,
                        key: UuidV4::parse("20000000-0000-4000-8000-000000000004").unwrap(),
                        task: UuidV4::parse(PREP_TASK).unwrap(),
                        event: UuidV4::parse("20000000-0000-4000-8000-000000000005").unwrap(),
                        request_bytes: b"frontend preparation failure fixture",
                        workspace_id: UuidV4::parse("28f00000-0000-4000-8000-00000000000a").unwrap(),
                        criteria: Sha256Digest::parse(&format!("sha256:{}", "a".repeat(64)))
                            .unwrap(),
                        allocation: Allocation {
                            limit_ms: 1_200_000,
                            work_ms: 900_000,
                            verify_ms: 300_000,
                        },
                    },
                    clock.deadline(),
                )
                .unwrap();
        }
        Self {
            store,
            area,
            out,
            principal,
            clock,
        }
    }
    fn result(
        &mut self,
        original: crate::Result<()>,
        admitted: bool,
        cleanup: &crate::probes::Cleanup,
    ) -> crate::Result<()> {
        crate::frontend::preparation_result(
            original,
            (&mut self.store, &self.principal, PREP_TASK),
            &self.out.0,
            self.clock,
            admitted,
            cleanup,
            &serde_json::json!({"test_registry":"exact fixture registry","registered":{},"pending":[],"publication_error":"None"}),
        )
    }
    fn head(&self) -> habitat_engine::store::TaskHead {
        self.store
            .get(
                &self.principal,
                UuidV4::parse(PREP_TASK).unwrap(),
                self.clock.deadline(),
            )
            .unwrap()
    }
    fn report(&self) -> serde_json::Value {
        serde_json::from_slice(&fs::read(self.out.0.join("preparation-settlement.json")).unwrap())
            .unwrap()
    }
}
#[test]
fn actual_preparation_error_handler_charges_original_clock_and_retains_exact_error() {
    let mut rig = PrepRig::new(true);
    let returned = rig
        .result(
            Err("original Encoding failure".into()),
            true,
            &crate::probes::Cleanup::default(),
        )
        .unwrap_err();
    assert!(returned.contains("original Encoding failure"));
    assert!(returned.contains("terminal settlement committed"));
    let report = rig.report();
    assert_eq!(report["original_error"], "original Encoding failure");
    assert_eq!(
        report["registry"]["test_registry"],
        "exact fixture registry"
    );
    assert_eq!(report["settled"], true);
    let elapsed: u128 = report["task_elapsed_ns"].as_str().unwrap().parse().unwrap();
    let whole_ms = elapsed / 1_000_000;
    let fractional_ms = u128::from(!elapsed.is_multiple_of(1_000_000));
    let expected = u64::try_from(whole_ms + fractional_ms).unwrap();
    assert_eq!(report["used_ms"], expected);
    let head = rig.head();
    assert_eq!(
        (
            head.spent_ms,
            head.reserved_work_ms,
            head.reserved_verify_ms
        ),
        (expected, 0, 0)
    );
    assert_eq!(head.state, "failed");
    assert!(head.accepted_event.is_none());
    assert_eq!(
        rig.store
            .pending_delivery(256, rig.clock.deadline())
            .unwrap()
            .len(),
        1
    );
    let evidence: habitat_engine::store::Object =
        serde_json::from_value(report["evidence"].clone()).unwrap();
    let durable: serde_json::Value = serde_json::from_slice(
        &rig.store
            .read_object(&evidence, rig.clock.deadline())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(durable["original_error"], report["original_error"]);
    assert_eq!(durable["used_ms"], expected);
}
#[test]
fn successful_preparation_handler_preserves_admission_and_does_not_stop() {
    let mut rig = PrepRig::new(true);
    let before = rig.head();
    rig.result(Ok(()), true, &crate::probes::Cleanup::default())
        .unwrap();
    assert_eq!(rig.head(), before);
    assert!(
        rig.store
            .pending_delivery(256, rig.clock.deadline())
            .unwrap()
            .is_empty()
    );
    assert!(!rig.out.0.join("preparation-settlement.json").exists());
}
#[test]
fn unknown_probe_custody_prevents_preparation_terminal_release() {
    let mut rig = PrepRig::new(true);
    let before = rig.head();
    let mut cleanup = crate::probes::Cleanup::default();
    cleanup.begin("unreturned fixture probe").unwrap();
    let error = rig
        .result(Err("original probe error".into()), true, &cleanup)
        .unwrap_err();
    assert!(error.contains("original probe error"));
    assert!(error.contains("custody or accounting unresolved"));
    assert_eq!(rig.head(), before);
    assert_eq!(rig.report()["settled"], false);
}
#[test]
fn error_before_observed_admission_does_not_attempt_a_terminal_commit() {
    let mut rig = PrepRig::new(false);
    let error = rig
        .result(
            Err("admission failed".into()),
            false,
            &crate::probes::Cleanup::default(),
        )
        .unwrap_err();
    assert!(error.contains("no observed admission commit"));
    assert_eq!(rig.report()["admission_observed"], false);
    assert!(
        rig.store
            .pending_delivery(256, rig.clock.deadline())
            .unwrap()
            .is_empty()
    );
    assert!(
        rig.store
            .get(
                &rig.principal,
                UuidV4::parse(PREP_TASK).unwrap(),
                rig.clock.deadline()
            )
            .is_err()
    );
}
#[test]
fn cancellation_before_preparation_failure_keeps_cancellation_precedence() {
    let mut rig = PrepRig::new(true);
    rig.store
        .cancel(
            UuidV4::parse(PREP_TASK).unwrap(),
            "1".parse().unwrap(),
            UuidV4::parse("20000000-0000-4000-8000-000000000006").unwrap(),
            rig.clock.deadline(),
        )
        .unwrap();
    rig.result(
        Err("preparation interrupted".into()),
        true,
        &crate::probes::Cleanup::default(),
    )
    .unwrap_err();
    assert_eq!(rig.head().state, "cancelled");
    assert_eq!(rig.report()["settlement"]["cancelled"], true);
    assert_eq!(
        rig.store
            .pending_delivery(256, rig.clock.deadline())
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn failed_failure_evidence_publication_keeps_original_and_reservations() {
    use std::os::unix::fs::PermissionsExt;
    let mut rig = PrepRig::new(true);
    let before = rig.head();
    let objects = rig
        .area
        .0
        .join("generations")
        .join(PREP_GENERATION)
        .join("objects/sha256");
    fs::set_permissions(&objects, fs::Permissions::from_mode(0o500)).unwrap();
    let result = rig.result(
        Err("original preparation fault".into()),
        true,
        &crate::probes::Cleanup::default(),
    );
    fs::set_permissions(&objects, fs::Permissions::from_mode(0o700)).unwrap();
    let error = result.unwrap_err();
    assert!(error.contains("original preparation fault"));
    assert!(error.contains("settlement unresolved"));
    assert_eq!(rig.head(), before);
    assert_eq!(rig.report()["settled"], false);
}
#[test]
fn registry_and_report_persistence_failures_do_not_discard_original_error() {
    let mut rig = PrepRig::new(true);
    private_dir(&rig.out.0.join("preparation-registry.json")).unwrap();
    private_dir(&rig.out.0.join("preparation-settlement.json")).unwrap();
    let error = rig
        .result(
            Err("keep this original".into()),
            true,
            &crate::probes::Cleanup::default(),
        )
        .unwrap_err();
    assert!(error.contains("keep this original"));
    assert!(error.contains("preparation registry"));
    assert!(error.contains("preparation settlement report"));
    assert_eq!(rig.head().state, "failed");
    assert_eq!(
        rig.store
            .pending_delivery(256, rig.clock.deadline())
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn actual_probe_cleanup_is_retained_when_later_stdout_persistence_fails() {
    let area = Area::new();
    private_dir(&area.0.join("true.stdout")).unwrap();
    let path = PathBuf::from("/usr/bin/true").canonicalize().unwrap();
    let bytes = fs::read(&path).unwrap();
    let pin = Pin {
        path,
        sha256: digest(&bytes),
        bytes: bytes.len() as u64,
    };
    let mut cleanup = crate::probes::Cleanup::default();
    let result = crate::probes::command(
        &pin,
        &[],
        &area.0,
        &area.0,
        "true",
        hee3_fixed_task_runtime::clock::TaskClock::start().unwrap(),
        (&mut cleanup, &std::sync::atomic::AtomicBool::new(false)),
    );
    assert!(result.is_err());
    assert!(cleanup.settled());
    let observed = cleanup.observation();
    assert_eq!(observed["reports"][0]["report"]["exit_code"], 0);
    assert_eq!(observed["reports"][0]["report"]["leader_reaped"], true);
    assert_eq!(observed["reports"][0]["report"]["group_settled"], true);
}
#[test]
fn actual_probe_spawn_refusal_cannot_masquerade_as_observed_cleanup() {
    let area = Area::new();
    let pin = pin(&area.0.join("nonexecuting"), b"not executable");
    let mut cleanup = crate::probes::Cleanup::default();
    let result = crate::probes::command(
        &pin,
        &[],
        &area.0,
        &area.0,
        "refused",
        hee3_fixed_task_runtime::clock::TaskClock::start().unwrap(),
        (&mut cleanup, &std::sync::atomic::AtomicBool::new(false)),
    );
    assert!(result.is_err());
    assert!(!cleanup.settled());
    assert!(cleanup.begin("must not clear preceding custody").is_err());
}

#[test]
fn uncertain_staged_publication_cannot_release_preparation_reservations() {
    for registry in [
        serde_json::json!({"registered":{},"pending":["retained object awaiting readback"],"publication_error":"None"}),
        serde_json::json!({"registered":{},"pending":[],"publication_error":"Some(uncertain publication)"}),
    ] {
        let mut rig = PrepRig::new(true);
        let before = rig.head();
        let result: crate::Result<()> = crate::frontend::preparation_result(
            Err("original artifact error".into()),
            (&mut rig.store, &rig.principal, PREP_TASK),
            &rig.out.0,
            rig.clock,
            true,
            &crate::probes::Cleanup::default(),
            &registry,
        );
        assert!(result.unwrap_err().contains("original artifact error"));
        assert_eq!(rig.head(), before);
        assert_eq!(rig.report()["settled"], false);
        assert_eq!(rig.report()["registry"], registry);
    }
}

#[test]
fn actual_outer_reporting_retains_original_and_all_write_errors_and_attempts_readback() {
    let mut rig = PrepRig::new(true);
    let original = rig
        .result(
            Err("original preparation fault".into()),
            true,
            &crate::probes::Cleanup::default(),
        )
        .unwrap_err();
    for file in [
        "initial-staging-final.json",
        "execution-return.json",
        "frontend-result.json",
    ] {
        private_dir(&rig.out.0.join(file)).unwrap();
    }
    let reported = crate::frontend::report_execution(
        Err(original.clone()),
        (&rig.store, &rig.principal, PREP_TASK),
        &rig.out.0,
        rig.clock,
        &serde_json::json!({"exact_registry":"kept in original Store evidence"}),
    );
    let readback: serde_json::Value =
        serde_json::from_slice(&fs::read(rig.out.0.join("store-readback.json")).unwrap()).unwrap();
    assert_eq!(readback["head"]["state"], "failed");
    let error = crate::frontend::report_frontend(
        reported,
        Path::new("/fixture/source-manifest"),
        &rig.out.0,
        rig.clock,
    )
    .unwrap_err();
    assert!(error.contains(&original));
    for label in [
        "initial staging projection",
        "execution return projection",
        "frontend result",
    ] {
        assert!(error.contains(label));
    }
}
#[test]
fn actual_outer_reporting_keeps_readback_failure_with_original_and_projection_failure() {
    let rig = PrepRig::new(true);
    private_dir(&rig.out.0.join("initial-staging-final.json")).unwrap();
    let foreign = Principal::new(rustix::process::geteuid().as_raw(), "viewer").unwrap();
    let error = crate::frontend::report_execution(
        Err("original plus settlement details".into()),
        (&rig.store, &foreign, PREP_TASK),
        &rig.out.0,
        rig.clock,
        &serde_json::json!({}),
    )
    .unwrap_err();
    assert!(error.contains("original plus settlement details"));
    assert!(error.contains("initial staging projection"));
    assert!(error.contains("readback:"));
    let readback: serde_json::Value =
        serde_json::from_slice(&fs::read(rig.out.0.join("store-readback.json")).unwrap()).unwrap();
    assert!(readback["head_error"].is_string());
}
#[test]
fn accepted_execution_still_requires_accepted_readback_after_projection_failure() {
    let rig = PrepRig::new(true);
    private_dir(&rig.out.0.join("initial-staging-final.json")).unwrap();
    let error = crate::frontend::report_execution(
        Ok(()),
        (&rig.store, &rig.principal, PREP_TASK),
        &rig.out.0,
        rig.clock,
        &serde_json::json!({}),
    )
    .unwrap_err();
    assert!(error.contains("initial staging projection"));
    assert!(error.contains("accepted driver result lacks exact durable accepted head/outbox"));
}

#[test]
fn store_readback_projection_failure_does_not_erase_observed_store_error() {
    let rig = PrepRig::new(true);
    private_dir(&rig.out.0.join("store-readback.json")).unwrap();
    let foreign = Principal::new(rustix::process::geteuid().as_raw(), "viewer").unwrap();
    let error = crate::frontend::report_execution(
        Err("original settlement evidence".into()),
        (&rig.store, &foreign, PREP_TASK),
        &rig.out.0,
        rig.clock,
        &serde_json::json!({}),
    )
    .unwrap_err();
    assert!(error.contains("original settlement evidence"));
    assert!(error.contains("NotFound"));
    assert!(error.contains("store readback projection"));
}
