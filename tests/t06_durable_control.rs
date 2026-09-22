#![forbid(unsafe_code)]

#[cfg(test)]
mod actual_store_controls {
    use habitat_engine::{
        app::durable_control::{self, Cause},
        app::workload::{Plan, Tools},
        contracts::{Sha256Digest, UuidV4},
        store::{Allocation, Principal, Store, Submission},
        worker::{namespace::ReadOnlyFile, resources::Scope, workspace::Snapshot},
    };
    use std::fs;
    use std::os::unix::fs::DirBuilderExt;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant, SystemTime};

    fn id(value: &str) -> UuidV4<'_> {
        UuidV4::parse(value).unwrap()
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one Store fixture proves authorization precedes every queued effect"
    )]
    fn already_durable_cancellation_prevents_workload_launch() {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("hee3-durable-store-{}-{nonce}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        for name in ["source", "protected", "job"] {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(root.join(name))
                .unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        let generation = id("11111111-1111-4111-8111-111111111111");
        let epoch = id("22222222-2222-4222-8222-222222222222");
        let task = id("33333333-3333-4333-8333-333333333333");
        let principal = Principal::new(1000, "operator").unwrap();
        let mut store = Store::open(&root, generation, epoch, true, deadline).unwrap();
        store
            .submit(
                Submission {
                    principal: &principal,
                    key: id("44444444-4444-4444-8444-444444444444"),
                    task,
                    event: id("55555555-5555-4555-8555-555555555555"),
                    request_bytes: b"fixed request",
                    criteria: Sha256Digest::parse(
                        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                    )
                    .unwrap(),
                    allocation: Allocation {
                        limit_ms: 1_000,
                        work_ms: 800,
                        verify_ms: 200,
                    },
                },
                deadline,
            )
            .unwrap();
        let (handle, queue) = durable_control::cancellation_queue(
            durable_control::TaskId::parse(task.as_str()).unwrap(),
        );
        handle
            .try_cancel(
                "1".parse().unwrap(),
                durable_control::EventId::parse("66666666-6666-4666-8666-666666666666").unwrap(),
            )
            .unwrap();

        let source = Snapshot::capture(&root.join("source"), &[], deadline).unwrap();
        let protected = Snapshot::capture(&root.join("protected"), &[], deadline).unwrap();
        let cancelled = AtomicBool::new(false);
        let tools = Tools {
            bwrap: PathBuf::from("/absent"),
            compiler: ReadOnlyFile {
                host: PathBuf::from("/absent"),
                namespace: PathBuf::from("/toolchain/bin/rustc"),
                sha256: [0; 32],
            },
            shim: ReadOnlyFile {
                host: PathBuf::from("/absent"),
                namespace: PathBuf::from("/shim/namespace-shim"),
                sha256: [0; 32],
            },
            runtime_files: Vec::new(),
            namespace_directories: Vec::new(),
        };
        let plan = Plan {
            source: &source,
            protected: &protected,
            job_root: &root.join("job"),
            tools: &tools,
            deadline,
            cancelled: &cancelled,
        };
        let scopes: [Scope; 3] = std::array::from_fn(|index| Scope {
            systemd_run: PathBuf::from("/absent"),
            systemd_run_sha256: "sha256:00".into(),
            runtime_dir: PathBuf::from("/absent"),
            run_id: format!("77777777-7777-4777-8777-77777777777{index}"),
            aggregate: "fixed.slice".into(),
        });
        let long_plan = Plan {
            source: &source,
            protected: &protected,
            job_root: &root.join("job"),
            tools: &tools,
            deadline: Instant::now() + Duration::from_secs(1_201),
            cancelled: &cancelled,
        };
        assert!(matches!(
            durable_control::collect(&mut store, &principal, task, &long_plan, &scopes, &queue),
            Err(durable_control::Error::InvalidDeadline)
        ));
        let wrong_principal = Principal::new(2000, "operator").unwrap();
        assert!(matches!(
            durable_control::collect(&mut store, &wrong_principal, task, &plan, &scopes, &queue),
            Err(durable_control::Error::InitialRead { command: None, .. })
        ));
        assert!(!store.get(&principal, task, deadline).unwrap().cancellation);
        assert_eq!(
            handle
                .try_cancel(
                    "1".parse().unwrap(),
                    durable_control::EventId::parse("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb")
                        .unwrap()
                )
                .unwrap_err()
                .failure,
            durable_control::EnqueueFailure::Full
        );
        let (_, wrong_queue) = durable_control::cancellation_queue(
            durable_control::TaskId::parse("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa").unwrap(),
        );
        assert!(matches!(
            durable_control::collect(&mut store, &principal, task, &plan, &scopes, &wrong_queue),
            Err(durable_control::Error::TaskMismatch)
        ));
        assert_eq!(handle.task().as_str(), task.as_str());
        let report =
            durable_control::collect(&mut store, &principal, task, &plan, &scopes, &queue).unwrap();
        assert!(report.run.is_none());
        assert_eq!(report.cause, Cause::DurableCancellation);
        assert!(report.started_at <= Instant::now());
        assert!(report.first_observed_after.is_some());
        assert!(report.last_head.cancellation && cancelled.load(Ordering::Acquire));
        assert_eq!(report.cancellations.len(), 1);
        assert!(report.cancellations[0].result.is_ok());
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
}
