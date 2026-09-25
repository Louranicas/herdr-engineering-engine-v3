//! Actual `TaskRuntime` stop/queue controls. State setup is synthetic; no worker or collector ran.
use super::*;
use habitat_engine::check::consistency::CasePlan;
use habitat_engine::contracts::receipt as r;
use habitat_engine::contracts::roster::{Kind, Locality};
use habitat_engine::store::{Effect, Expected, Settlement, Verification, VerificationVerdict};
use habitat_engine::task::driver::{self, Runtime, StopReason};
use habitat_engine::worker::namespace::ReadOnlyFile;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
#[path = "runtime_fixture.rs"]
mod fixture;
const TASK: &str = "71000000-0000-4000-8000-000000000005";
const ATTEMPT: &str = "71000000-0000-4000-8000-000000000007";
const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn id(n: u32) -> String {
    format!("72000000-0000-4000-8000-{n:012x}")
}
fn parse<T: DeserializeOwned>(v: &Value) -> T {
    serde_json::from_value(v.clone()).unwrap()
}
fn reference(schema: &str) -> Value {
    json!({"artifact_id":id(90),"sha256":SHA,"byte_length":1,"media_type":"application/json","schema_id":schema})
}
fn recipe(index: u32) -> Recipe {
    let p: Value = serde_json::from_str(include_str!("runtime-preparation.json")).unwrap();
    let c = &p["cases"][0];
    let mut identity: r::IdentityV1 = parse(&p["identity"]);
    identity.task_id = r::Id::new(TASK).unwrap();
    identity.run_id = r::Id::new(id(10 + index)).unwrap();
    identity.attempt_id = r::Id::new(if index == 0 {
        ATTEMPT.to_owned()
    } else {
        id(12)
    })
    .unwrap();
    identity.generation = r::Generation::new((index + 1).to_string()).unwrap();
    let aggregate = format!(
        "hee3aggregate{}.slice",
        identity.run_id.as_str().replace('-', "")
    );
    Recipe {
        prepared: Prepared {
            schema_sha256: parse(&p["schema_sha256"]),
            identity,
            subjects: parse(&p["subjects"]),
            invocation: parse(&p["invocation"]),
            cases: vec![CasePlan {
                case_id: parse(&c["case_id"]),
                primary_module_id: parse(&c["primary_module_id"]),
                criterion_ids: parse(&c["criterion_ids"]),
                fixture_sha256: parse(&c["fixture_sha256"]),
                oracle_id: parse(&c["oracle_id"]),
                expected: parse(&c["expected"]),
                mandatory: true,
                selected: true,
                excluded: false,
                reviewed_design: None,
            }],
            // The WL-U64 lane's one editable file and bounds, as `prepare_u64` pins them.
            editable: habitat_engine::check::consistency::Editable {
                path: r::RelPath::new("src/lib.rs").unwrap(),
                bounds: habitat_engine::check::patch::CandidateBounds {
                    bytes: 65_536,
                    changed_lines: 200,
                },
            },
        },
        recipe: parse(&reference("hee3.raw/1")),
        host: parse(&reference("hee3.receipt/1:HostV1")),
        review_provenance: parse(&reference("hee3.raw/1")),
        session: id(20 + index),
        workspace: id(22 + index),
        scopes: [0, 1, 2].map(|n| Scope {
            systemd_run: "/usr/bin/systemd-run".into(),
            systemd_run_sha256: SHA.into(),
            runtime_dir: "/run/user/1000".into(),
            run_id: id(30 + index * 3 + n),
            aggregate: aggregate.clone(),
        }),
    }
}
fn config(area: &fixture::Area, selected: Selection) -> Config {
    let parent = area.root.parent().unwrap();
    let source = parent.join("source");
    let root = parent.join("runtime");
    fixture::private(&source);
    fixture::private(&root);
    let snapshot = || Snapshot::capture(&source, &[], fixture::deadline()).unwrap();
    let file = || ReadOnlyFile {
        host: "/never-dispatch".into(),
        namespace: "/toolchain/never-dispatch".into(),
        sha256: [0; 32],
    };
    Config {
        sources: Sources {
            baseline: snapshot(),
            repaired: snapshot(),
            protected: snapshot(),
            fixtures: snapshot(),
            oracle: snapshot(),
            harness: snapshot(),
            collector: snapshot(),
            launcher: snapshot(),
            reference_patch: Vec::new(),
        },
        recipes: [recipe(0), recipe(1)],
        tools: workload::Tools {
            bwrap: "/never-dispatch".into(),
            compiler: file(),
            shim: file(),
            runtime_files: Vec::new(),
            namespace_directories: Vec::new(),
        },
        root,
        agent_record_id: selected.record_id.clone(),
        selections: vec![selected],
        busctl: "/never-dispatch".into(),
        busctl_sha256: SHA.into(),
        runtime_dir: "/run/user/1000".into(),
        executor: std::env::current_exe().unwrap(),
        executor_sha256: fixture::executable_sha(),
    }
}
#[derive(Clone, Copy, Debug)]
enum Case {
    QueuedCancel,
    CleanRefusal,
    UnknownWork,
    VerifierError,
    Exhaustion,
    UnrecordedVerifier,
}
fn seed_verification(runtime: &mut TaskRuntime<'_>, attempt: &RosterAttempt, case: Case) {
    let head = runtime
        .store
        .get(
            runtime.principal,
            fixture::uuid(TASK),
            runtime.clock.deadline(),
        )
        .unwrap();
    let object = runtime
        .store
        .publish(
            b"synthetic verifier outcome: no collector execution",
            fixture::uuid(&id(72)),
            runtime.clock.deadline(),
        )
        .unwrap();
    runtime
        .store
        .record_verification(
            &Expected {
                task: fixture::uuid(TASK),
                task_generation: fixture::generation(&head.generation),
                attempt: fixture::uuid(ATTEMPT),
                attempt_generation: fixture::generation(&attempt.attempt.generation),
            },
            &Verification {
                verdict: if matches!(case, Case::VerifierError) {
                    VerificationVerdict::Error
                } else {
                    VerificationVerdict::Failed
                },
                subject: Sha256Digest::parse(SHA).unwrap(),
                evidence: object,
                used_ms: Some(3),
                cleanup_settled: true,
            },
            fixture::uuid(&id(73)),
            runtime.clock.deadline(),
        )
        .unwrap();
}

fn seeded_stop(
    runtime: &mut TaskRuntime<'_>,
    area: &fixture::Area,
    roster: &habitat_engine::contracts::roster::RosterHeadV1,
    selected: &Selection,
    case: Case,
) -> bool {
    let observation = native::observe(
        runtime.store,
        runtime.principal,
        &mut runtime.evidence,
        selected,
        &std::env::current_exe().unwrap(),
        &fixture::executable_sha(),
        runtime.clock.deadline(),
    )
    .unwrap();
    let attempt = fixture::begin(runtime.store, roster, selected).unwrap();
    let head = runtime
        .store
        .get(
            runtime.principal,
            fixture::uuid(TASK),
            runtime.clock.deadline(),
        )
        .unwrap();
    let expected = Expected {
        task: fixture::uuid(TASK),
        task_generation: fixture::generation(&head.generation),
        attempt: fixture::uuid(ATTEMPT),
        attempt_generation: fixture::generation(&attempt.attempt.generation),
    };
    let unknown = matches!(case, Case::UnknownWork);
    let verifier = matches!(
        case,
        Case::VerifierError | Case::Exhaustion | Case::UnrecordedVerifier
    );
    runtime
        .store
        .settle_attempt(
            &expected,
            Settlement {
                effect: if unknown {
                    Effect::Unknown
                } else {
                    Effect::None
                },
                used_ms: if unknown { None } else { Some(17) },
                cleanup_settled: !unknown,
                ready_to_verify: verifier,
            },
            fixture::uuid(&id(71)),
            runtime.clock.deadline(),
        )
        .unwrap();
    let recorded = matches!(case, Case::VerifierError | Case::Exhaustion);
    if recorded {
        seed_verification(runtime, &attempt, case);
    }

    runtime.executions.push(Execution {
        roster: attempt,
        executor_observation: observation,
        began: runtime.clock.origin,
        source: None,
        frozen: None,
        job: area.root.parent().unwrap().join("unused-job"),
        aggregate: None,
        aggregate_start: None,
        aggregate_stop: None,
        aggregate_cleanup_error: None,
        control: None,
        undispatched_at: None,
        work_settled: !unknown,
        verification_started: verifier,
        verification_phases: Vec::new(),
        receipt: None,
        collected_evidence: None,
        imported: None,
        verification_recorded: recorded,
    });
    let reason = match case {
        Case::CleanRefusal => StopReason::WorkerFailed,
        Case::UnknownWork => StopReason::Unsettled,
        Case::VerifierError | Case::UnrecordedVerifier => StopReason::VerifierError,
        Case::Exhaustion => {
            StopReason::Policy(habitat_engine::task::LoopRefusal::AttemptsExhausted)
        }
        Case::QueuedCancel => unreachable!(),
    };
    Runtime::stop(runtime, reason).unwrap()
}

fn control(case: Case) {
    let area = fixture::Area::new();
    let mut store = area.store();
    let staging = area.staging();
    let principal = fixture::principal();
    let roster = fixture::roster(
        &mut store,
        Kind::Agent,
        Locality::Local,
        &["u64-fixed-workload"],
    );
    let selected = fixture::selection(&roster);
    fixture::admit(&mut store);
    let task = TaskId::parse(TASK).unwrap();
    let (handle, queue) = durable_control::cancellation_queue(task.clone());
    let cfg = config(&area, selected.clone());
    let mut clock = TaskClock::start().unwrap();
    // Advance the test origin, preserving the original total horizon without sleeping.
    if matches!(case, Case::QueuedCancel) {
        let prep = std::time::Duration::from_millis(37);
        clock.origin -= prep;
        clock.candidate_deadline -= prep;
        clock.verification_deadline -= prep;
        clock.deadline -= prep;
        clock.unix_ms -= 37;
    }
    let evidence = Evidence::staged(&staging, clock.deadline());
    let mut runtime =
        TaskRuntime::new(&mut store, &principal, task, &queue, evidence, cfg, clock).unwrap();
    let result = if matches!(case, Case::QueuedCancel) {
        handle
            .try_cancel(
                fixture::generation("1"),
                durable_control::EventId::parse(&id(70)).unwrap(),
            )
            .unwrap();
        let outcome = driver::run(&mut runtime, 1).unwrap();
        assert_eq!(outcome, driver::Outcome::Stopped(StopReason::Cancelled));
        true
    } else {
        seeded_stop(&mut runtime, &area, &roster, &selected, case)
    };
    let observation = runtime.observation();
    drop(runtime);
    drop(store);
    let store = area.store();
    let head = store
        .get(&principal, fixture::uuid(TASK), fixture::deadline())
        .unwrap();
    let outbox = store.pending_delivery(256, fixture::deadline()).unwrap();
    assert!(head.accepted_event.is_none());
    if matches!(case, Case::UnknownWork | Case::UnrecordedVerifier) {
        assert!(!result);
        assert!(outbox.is_empty());
        assert!(head.reserved_work_ms + head.reserved_verify_ms > 0);
    } else {
        assert!(result);
        assert_eq!(outbox.len(), 1);
        assert_eq!(head.reserved_work_ms, 0);
        assert_eq!(head.reserved_verify_ms, 0);
    }
    if matches!(case, Case::QueuedCancel) {
        assert!(head.cancellation);
        assert_eq!(head.state, "cancelled");
        assert!(
            head.spent_ms >= 37,
            "pre-attempt preparation must be charged"
        );
    }
    if matches!(case, Case::UnknownWork) {
        assert_eq!(head.state, "effect_unknown");
    }
    println!(
        "CONTROL {}",
        json!({"case":format!("{case:?}"),"actual_runtime_stop":true,"synthetic_state_setup":true,"producer_execution":false,"head":{"state":head.state,"generation":head.generation,"cancellation":head.cancellation,"accepted_event":head.accepted_event,"spent_ms":head.spent_ms,"reserved_work_ms":head.reserved_work_ms,"reserved_verify_ms":head.reserved_verify_ms},"outbox":outbox,"observation":observation})
    );
}
#[test]
fn actual_runtime_queued_predispatch_cancellation() {
    control(Case::QueuedCancel);
}
#[test]
fn actual_runtime_clean_refusal_stops() {
    control(Case::CleanRefusal);
}
#[test]
fn actual_runtime_unknown_work_retains_obligations() {
    control(Case::UnknownWork);
}
#[test]
fn actual_runtime_recorded_verifier_error_stops() {
    control(Case::VerifierError);
}
#[test]
fn actual_runtime_exhaustion_stops_after_failed_verification() {
    control(Case::Exhaustion);
}
#[test]
fn actual_runtime_unrecorded_verifier_retains_obligations() {
    control(Case::UnrecordedVerifier);
}

fn point_runtime(control: impl FnOnce(&mut TaskRuntime<'_>, &durable_control::CancellationHandle)) {
    let area = fixture::Area::new();
    let mut store = area.store();
    let staging = area.staging();
    let principal = fixture::principal();
    let roster = fixture::roster(
        &mut store,
        Kind::Agent,
        Locality::Local,
        &["u64-fixed-workload"],
    );
    let selected = fixture::selection(&roster);
    fixture::admit(&mut store);
    let task = TaskId::parse(TASK).unwrap();
    let (handle, queue) = durable_control::cancellation_queue(task.clone());
    let mut runtime = TaskRuntime::new(
        &mut store,
        &principal,
        task,
        &queue,
        Evidence::staged(&staging, fixture::deadline()),
        config(&area, selected),
        TaskClock::start().unwrap(),
    )
    .unwrap();
    control(&mut runtime, &handle);
}

// Real begin commits the attempt and materializes an empty synthetic source before
// its deliberately unavailable receipt graph refuses. Only the actual notification
// method is tested here; no successful preparation or native dispatch is claimed.
fn retained_point_ticket(runtime: &mut TaskRuntime<'_>) -> Attempt {
    assert!(matches!(
        runtime.begin("1".parse().unwrap()),
        Err(Error::Receipt(_))
    ));
    assert_eq!(runtime.head.generation, "2");
    assert_eq!(runtime.executions.len(), 1);
    Attempt {
        index: 0,
        id: ATTEMPT.to_owned(),
    }
}

#[test]
fn actual_runtime_point_reports_exact_owner_generation_and_scopes() {
    point_runtime(|runtime, _| {
        let (sender, receiver) = sync_channel(1);
        runtime.set_attempt_points(sender).unwrap();
        let ticket = retained_point_ticket(runtime);
        runtime.publish_attempt_point(&ticket).unwrap();
        let point = receiver.try_recv().unwrap();
        assert_eq!(point.task_id, TASK);
        assert_eq!(point.attempt_id, ATTEMPT);
        assert_eq!(point.expected_generation, "2");
        assert_eq!(point.task_origin, runtime.clock.origin());
        assert!(
            serde_json::to_value(&point)
                .unwrap()
                .get("task_origin")
                .is_none()
        );
        assert!(
            point.task_elapsed_ns.parse::<u128>().unwrap() <= runtime.clock.elapsed().as_nanos()
        );
        assert_eq!(point.run_id, id(10));
        assert_eq!(
            point.aggregate,
            format!("hee3aggregate{}.slice", id(10).replace('-', ""))
        );
        assert_eq!(
            point.scope_units,
            [30, 31, 32].map(|n| format!("hee3-resource-{}.scope", id(n).replace('-', "")))
        );
        assert!(runtime.executions[0].aggregate.is_none());
    });
}
#[test]
fn actual_runtime_point_full_and_disconnected_refuse_without_dispatch() {
    for disconnected in [false, true] {
        point_runtime(|runtime, _| {
            let (sender, receiver) = sync_channel(1);
            runtime.set_attempt_points(sender).unwrap();
            let ticket = retained_point_ticket(runtime);
            runtime.publish_attempt_point(&ticket).unwrap();
            if disconnected {
                drop(receiver);
            }
            let error = runtime.publish_attempt_point(&ticket).unwrap_err();
            assert!(matches!(
                (disconnected, error),
                (true, Error::AttemptPointDisconnected) | (false, Error::AttemptPointFull)
            ));
            assert!(runtime.executions[0].aggregate.is_none());
            assert!(runtime.executions[0].control.is_none());
            assert!(!runtime.head.cancellation);
            assert!(runtime.head.accepted_event.is_none());
        });
    }
}
#[test]
fn actual_runtime_point_late_replacement_and_foreign_ticket_refuse() {
    point_runtime(|runtime, _| {
        let (sender, _receiver) = sync_channel(1);
        runtime.set_attempt_points(sender).unwrap();
        let (other, _receiver2) = sync_channel(1);
        assert!(matches!(
            runtime.set_attempt_points(other),
            Err(Error::State)
        ));
        let ticket = retained_point_ticket(runtime);
        assert!(matches!(
            runtime.publish_attempt_point(&Attempt {
                index: ticket.index,
                id: id(99)
            }),
            Err(Error::Identity)
        ));
    });
    point_runtime(|runtime, _| {
        let _ticket = retained_point_ticket(runtime);
        let (sender, _receiver) = sync_channel(1);
        assert!(matches!(
            runtime.set_attempt_points(sender),
            Err(Error::State)
        ));
    });
}
#[test]
fn actual_runtime_materialize_services_queued_intent_with_same_owner() {
    point_runtime(|runtime, handle| {
        let root = runtime.config.root.join("copy-control");
        fixture::private(&root);
        handle
            .try_cancel(
                "1".parse().unwrap(),
                durable_control::EventId::parse(&id(98)).unwrap(),
            )
            .unwrap();
        let source = runtime.materialize(&root, 0).unwrap();
        assert!(source.entries().next().is_none());
        assert!(runtime.head.cancellation);
        assert_eq!(runtime.head.state, "cancellation_requested");
        assert_eq!(runtime.head.generation, "2");
        assert!(runtime.executions.is_empty());
        assert!(runtime.head.accepted_event.is_none());
        assert!(!runtime.boundary_observations.is_empty());
    });
}
#[test]
fn actual_runtime_materialize_benign_and_failed_copy_preserve_task() {
    for preexisting in [false, true] {
        point_runtime(|runtime, _| {
            let root = runtime.config.root.join("copy-control");
            fixture::private(&root);
            if preexisting {
                fixture::private(&root.join("source"));
            }
            let result = runtime.materialize(&root, 0);
            assert_eq!(result.is_err(), preexisting);
            assert!(!runtime.head.cancellation);
            assert_eq!(runtime.head.generation, "1");
            assert!(runtime.executions.is_empty());
        });
    }
}

#[test]
fn actual_runtime_cancel_before_frozen_preparation_settles_without_dispatch() {
    point_runtime(|runtime, handle| {
        // This control names a planned result explicitly; it is not a frozen/verified subject.
        runtime.config.recipes[0].prepared.subjects.result_subject = r::Maybe::present(
            runtime.config.recipes[0]
                .prepared
                .subjects
                .seed_subject
                .clone(),
        );
        let ticket = retained_point_ticket(runtime);
        assert!(runtime.executions[0].frozen.is_none());
        assert!(matches!(runtime.execute(&ticket), Err(Error::State)));
        handle
            .try_cancel(
                "2".parse().unwrap(),
                durable_control::EventId::parse(&id(97)).unwrap(),
            )
            .unwrap();
        assert!(matches!(
            runtime.execute(&ticket).unwrap(),
            driver::Work::ReadyForCheck
        ));
        assert!(runtime.executions[0].aggregate.is_none());
        assert!(runtime.executions[0].work_settled);
        assert!(Runtime::stop(runtime, StopReason::Cancelled).unwrap());
        let head = runtime
            .store
            .get(
                runtime.principal,
                fixture::uuid(TASK),
                runtime.clock.deadline(),
            )
            .unwrap();
        assert_eq!(head.state, "cancelled");
        assert_eq!(head.reserved_work_ms, 0);
        assert_eq!(head.reserved_verify_ms, 0);
        assert!(head.accepted_event.is_none());
        assert_eq!(
            runtime
                .store
                .pending_delivery(16, runtime.clock.deadline())
                .unwrap()
                .len(),
            1
        );
    });
}

#[test]
fn actual_runtime_freeze_services_intent_and_preserves_preparation_failure() {
    point_runtime(|runtime, handle| {
        let _ticket = retained_point_ticket(runtime);
        handle
            .try_cancel(
                "2".parse().unwrap(),
                durable_control::EventId::parse(&id(96)).unwrap(),
            )
            .unwrap();
        assert!(matches!(runtime.prepare(0), Err(Error::Receipt(_))));
        assert!(runtime.head.cancellation);
        assert!(runtime.executions[0].frozen.is_none());
        assert!(runtime.executions[0].aggregate.is_none());
        assert!(runtime.head.accepted_event.is_none());
    });
}

// These three controls import the existing tiny UNMEASURED representation fixture.
// They test real receipt transfer and concurrent Store ownership, not candidate
// execution or acceptance. Preparation is loaded independently of the receipt.
fn import_representation(cancel: bool, corrupt_root: bool) {
    use habitat_engine::check::collector::Sink;
    point_runtime(|runtime, handle| {
        retained_point_ticket(runtime);
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/receipt-import/nonpass.json"
        ))
        .unwrap();
        let plan: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/receipt-import/preparation.json"
        ))
        .unwrap();
        let mut prepared = recipe(0).prepared;
        prepared.schema_sha256 = parse(&plan["schema_sha256"]);
        prepared.identity = parse(&plan["identity"]);
        prepared.subjects = parse(&plan["subjects"]);
        prepared.invocation = parse(&plan["invocation"]);
        let case = &plan["cases"][0];
        prepared.cases[0].mandatory = case["mandatory"].as_bool().unwrap();
        prepared.cases[0].selected = case["selected"].as_bool().unwrap();
        prepared.cases[0].excluded = case["excluded"].as_bool().unwrap();
        for row in fixture["objects"].as_array().unwrap() {
            let reference: r::Ref = parse(&row["reference"]);
            runtime
                .evidence
                .publish(&reference, row["bytes"].as_str().unwrap().as_bytes())
                .unwrap();
        }
        let mut root: r::Ref = parse(&fixture["root"]);
        if corrupt_root {
            root.sha256 = r::Sha::new(SHA).unwrap();
        }
        let root = r::TypedRef::<r::ReceiptV1>::new(root).unwrap();
        if cancel {
            handle
                .try_cancel(
                    runtime.head.generation.parse().unwrap(),
                    durable_control::EventId::parse(&id(130)).unwrap(),
                )
                .unwrap();
        }
        let before = runtime.boundary_observations.len();
        let result = runtime.import_prepared_receipt(0, &prepared, &root);
        assert_eq!(result.is_ok(), !corrupt_root);
        if corrupt_root {
            assert!(matches!(result, Err(Error::Import(_))));
        }
        if cancel {
            assert!(runtime.boundary_observations.len() > before);
        }
        assert_eq!(runtime.head.cancellation, cancel);
        let imported = runtime.executions[0].imported.as_ref().unwrap();
        assert!(imported.pending.is_empty());
        assert!(imported.publication_error.is_none());
        assert_eq!(imported.registered.len(), if corrupt_root { 0 } else { 28 });
        for (reference, object) in imported.registered.values() {
            let bytes = runtime
                .store
                .read_object(object, runtime.clock.deadline())
                .unwrap();
            assert_eq!(bytes.len(), reference.byte_length as usize);
        }
        assert!(runtime.head.accepted_event.is_none());
        assert!(
            runtime
                .store
                .pending_delivery(256, runtime.clock.deadline())
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            runtime.executions[0]
                .verification_phases
                .last()
                .unwrap()
                .name,
            "import"
        );
    });
}
#[test]
fn actual_import_services_queued_intent_and_retains_real_transfer() {
    import_representation(true, false);
}
#[test]
fn actual_import_benign_transfers_exact_unmeasured_representation() {
    import_representation(false, false);
}
#[test]
fn actual_import_failure_preserves_error_and_cancel_observation() {
    import_representation(true, true);
}

#[test]
fn import_panic_retains_published_identity_and_marks_custody_unknown() {
    let area = fixture::Area::new();
    let staging = area.staging();
    let mut evidence = Evidence::staged(&staging, fixture::deadline());
    let (result, retained): (Result<(), Error>, Imported) =
        retain_import(&mut evidence, |destination| {
            destination
                .payload(
                    b"actual immutable object before panic",
                    "application/octet-stream",
                )
                .unwrap();
            panic!("intentional verifier fault after real publication");
        });
    assert!(matches!(result, Err(Error::VerificationPanicked)));
    assert_eq!(retained.registered.len(), 1);
    assert!(
        retained
            .publication_error
            .as_ref()
            .unwrap()
            .contains("custody unknown")
    );
    for (reference, object) in retained.registered.values() {
        let bytes = staging.read_object(object, fixture::deadline()).unwrap();
        assert_eq!(bytes, b"actual immutable object before panic");
        assert_eq!(reference.byte_length as usize, bytes.len());
    }
}

// These controls run the real scoped collection servicing and actual immutable
// publication, over the synthetic admitted fixture. They do not execute a native
// workload or claim a real completed oracle receipt.
fn collection_publication(cancel: bool) {
    point_runtime(|runtime, handle| {
        retained_point_ticket(runtime);
        let initial = runtime.evidence.registered().clone();
        runtime
            .evidence
            .payload(b"existing collection input", "text/plain")
            .unwrap();
        if cancel {
            handle
                .try_cancel(
                    runtime.head.generation.parse().unwrap(),
                    durable_control::EventId::parse(&id(190)).unwrap(),
                )
                .unwrap();
        }
        let before = runtime.boundary_observations.len();
        let (result, observation) = runtime.collection_worker(0, |evidence, _, _| {
            let reference = evidence
                .payload(b"actual scoped collector output", "text/plain")
                .map_err(|_| Error::Publication)?;
            std::thread::sleep(Duration::from_millis(220));
            Ok(reference)
        });
        assert!(observation.is_none());
        let output = result.unwrap();
        assert_eq!(runtime.head.cancellation, cancel);
        if cancel {
            assert!(runtime.boundary_observations.len() >= before + 2);
        }
        let phase = runtime.executions[0].verification_phases.last().unwrap();
        assert_eq!(phase.name, "observe_and_finalize");
        if cancel {
            assert!(
                runtime.boundary_observations[before].observed_after
                    < phase.ended.duration_since(runtime.clock.origin)
            );
        }
        let retained = runtime.executions[0].collected_evidence.as_ref().unwrap();
        assert_eq!(retained.registered.len(), initial.len() + 2);
        for (id, original) in initial {
            assert_eq!(retained.registered.get(&id), Some(&original));
        }
        assert!(retained.pending.is_empty());
        assert!(retained.publication_error.is_none());
        runtime.rebind_collection(0).unwrap();
        let object = &runtime.evidence.registered()[output.as_ref().artifact_id.as_str()].1;
        assert_eq!(
            runtime
                .evidence
                .staging_owner()
                .unwrap()
                .read_object(object, runtime.clock.deadline())
                .unwrap(),
            b"actual scoped collector output"
        );
        assert!(runtime.head.accepted_event.is_none());
        assert!(!runtime.executions[0].verification_recorded);
        assert!(
            runtime
                .store
                .pending_delivery(256, runtime.clock.deadline())
                .unwrap()
                .is_empty()
        );
    });
}
#[test]
fn collection_benign_rebinds_exact_real_publications() {
    collection_publication(false);
}
#[test]
fn collection_services_durable_cancellation_before_worker_completion() {
    collection_publication(true);
}
#[test]
fn collection_panic_preserves_known_objects_and_unknown_custody() {
    point_runtime(|runtime, _| {
        retained_point_ticket(runtime);
        let initial = runtime.evidence.registered().clone();
        let (result, observation): (Result<(), Error>, _) =
            runtime.collection_worker(0, |evidence, _, _| {
                evidence
                    .payload(b"retained before real collection unwind", "text/plain")
                    .unwrap();
                panic!("intentional collection unwind");
            });
        assert!(matches!(result, Err(Error::VerificationPanicked)));
        assert!(observation.is_none());
        let retained = runtime.executions[0].collected_evidence.as_ref().unwrap();
        assert_eq!(retained.registered.len(), initial.len() + 1);
        assert!(
            retained
                .publication_error
                .as_ref()
                .unwrap()
                .contains("custody unknown")
        );
        assert!(matches!(
            runtime.rebind_collection(0),
            Err(Error::Publication)
        ));
        assert_eq!(runtime.evidence.registered().len(), initial.len() + 1);
        assert!(runtime.head.accepted_event.is_none());
        assert!(!runtime.executions[0].verification_recorded);
        assert!(
            runtime.observation()["executions"][0]["collected_evidence"]["publication_error"]
                .as_str()
                .unwrap()
                .contains("custody unknown")
        );
    });
}
#[test]
fn actual_collection_error_retains_input_registry_and_never_records_a_verdict() {
    point_runtime(|runtime, _| {
        retained_point_ticket(runtime);
        let input = runtime
            .evidence
            .payload(b"retained actual input", "text/plain")
            .unwrap();
        // Missing actual worker report is a real collector State error, not an
        // oracle mismatch, semantic verifier error, or synthetic completed case.
        let result = runtime.collect_current_receipt(0);
        assert!(matches!(
            result,
            Err(Error::VerificationCollection {
                work: Some(_),
                observation: None,
                custody: None
            })
        ));
        assert!(runtime.executions[0].verification_started);
        assert!(!runtime.executions[0].verification_recorded);
        assert!(runtime.executions[0].receipt.is_none());
        assert!(
            runtime.executions[0]
                .collected_evidence
                .as_ref()
                .unwrap()
                .registered
                .contains_key(input.as_ref().artifact_id.as_str())
        );
        assert!(runtime.head.accepted_event.is_none());
    });
}
#[test]
fn collection_publication_failure_is_retained_after_real_filesystem_refusal() {
    use std::os::unix::fs::PermissionsExt;
    point_runtime(|runtime, _| {
        retained_point_ticket(runtime);
        let initial = runtime.evidence.registered().clone();
        let objects = runtime.config.root.parent().unwrap().join("staging/sha256");
        let (result, observation): (Result<(), Error>, _) =
            runtime.collection_worker(0, |evidence, _, _| {
                std::fs::set_permissions(&objects, std::fs::Permissions::from_mode(0o500)).unwrap();
                let result =
                    evidence.payload(b"publication refused by real permissions", "text/plain");
                std::fs::set_permissions(&objects, std::fs::Permissions::from_mode(0o700)).unwrap();
                assert!(result.is_err());
                result.map(|_| ()).map_err(|_| Error::Publication)
            });
        assert!(matches!(result, Err(Error::Publication)));
        assert!(observation.is_none());
        let retained = runtime.executions[0].collected_evidence.as_ref().unwrap();
        assert_eq!(retained.registered, initial);
        assert!(retained.publication_error.is_some());
        assert!(matches!(
            runtime.rebind_collection(0),
            Err(Error::Publication)
        ));
        assert!(runtime.head.accepted_event.is_none());
    });
}
#[test]
fn collection_rebind_services_intent_for_each_returned_item() {
    point_runtime(|runtime, handle| {
        retained_point_ticket(runtime);
        let initial_count = runtime.evidence.registered().len();
        let (result, observation) = runtime.collection_worker(0, |evidence, _, _| {
            for i in 0..3 {
                evidence
                    .payload(format!("actual rebind object {i}").as_bytes(), "text/plain")
                    .unwrap();
            }
            Ok(())
        });
        result.unwrap();
        assert!(observation.is_none());
        handle
            .try_cancel(
                runtime.head.generation.parse().unwrap(),
                durable_control::EventId::parse(&id(191)).unwrap(),
            )
            .unwrap();
        let before = runtime.boundary_observations.len();
        runtime.rebind_collection(0).unwrap();
        assert!(runtime.head.cancellation);
        assert_eq!(
            runtime.boundary_observations.len(),
            before + initial_count + 3
        );
        assert_eq!(runtime.evidence.registered().len(), initial_count + 3);
    });
}
#[test]
fn collection_seeded_pending_custody_cannot_be_erased_by_successful_rebind() {
    point_runtime(|runtime, _| {
        retained_point_ticket(runtime);
        let (result, _) = runtime.collection_worker(0, |evidence, _, _| {
            evidence
                .payload(
                    b"actual object; synthetic pending marker below",
                    "text/plain",
                )
                .unwrap();
            Ok(())
        });
        result.unwrap();
        let retained = runtime.executions[0].collected_evidence.as_mut().unwrap();
        // Explicitly synthetic pending marker tests refusal, not production of a
        // partial publication. No unknown result is turned into successful cleanup.
        retained
            .pending
            .push(retained.registered.values().next().unwrap().clone());
        assert!(matches!(
            runtime.rebind_collection(0),
            Err(Error::Publication)
        ));
        assert_eq!(
            runtime.executions[0]
                .collected_evidence
                .as_ref()
                .unwrap()
                .pending
                .len(),
            1
        );
        assert!(!runtime.executions[0].verification_recorded);
    });
}
