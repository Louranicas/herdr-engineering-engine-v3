//! Test-only PID-reuse fixture. Each process makes one literal workspace write,
//! reports its actual `PID/start_ticks/namespace`, and blocks on an owned release
//! pipe. Old mode binds that identity into a running Store attempt and releases
//! Store custody (`store.lock`) before it blocks, as an observed worker is not
//! the Store custodian; replacement mode opens no Store. No signal, resume,
//! dispatch or provider call.
use habitat_engine::{
    contracts::{Sha256Digest, UuidV4},
    store::{Allocation, Principal, Store, Submission},
};
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const GEN: &str = "07000000-0000-4000-8000-000000000001";
const EPOCH: &str = "07000000-0000-4000-8000-000000000002";
const TASK: &str = "07000000-0000-4000-8000-000000000003";
const KEY: &str = "07000000-0000-4000-8000-000000000004";
const ADMIT: &str = "07000000-0000-4000-8000-000000000005";
const ATTEMPT: &str = "07000000-0000-4000-8000-000000000006";
const START: &str = "07000000-0000-4000-8000-000000000007";
const STAGE: &str = "07000000-0000-4000-8000-00000000000d";
const OTHER: &str = "07000000-0000-4000-8000-00000000000e";
const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn id(v: &str) -> UuidV4<'_> {
    UuidV4::parse(v).unwrap()
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn principal() -> Principal {
    Principal::new(rustix::process::geteuid().as_raw(), "operator").unwrap()
}
fn retained(path: &Path, value: &serde_json::Value) {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    f.write_all(&serde_json::to_vec(value).unwrap()).unwrap();
    f.sync_all().unwrap();
    File::open(path.parent().unwrap())
        .unwrap()
        .sync_all()
        .unwrap();
}
fn rostered(path: &Path, identity: &str) -> Store {
    use habitat_engine::contracts::roster::{
        Availability, Kind, Locality, ObservationInput, ObservationSource, RosterDefinitionV1,
        Selection, Update,
    };
    use habitat_engine::store::{RequestSource, RosterStart};
    fs::DirBuilder::new()
        .mode(0o700)
        .create(path.join("store"))
        .unwrap();
    let mut store = Store::open(&path.join("store"), id(GEN), id(EPOCH), true, end()).unwrap();
    store
        .submit(
            Submission {
                principal: &principal(),
                key: id(KEY),
                task: id(TASK),
                event: id(ADMIT),
                request_bytes: b"literal PID-reuse fixture",
                workspace_id: id("28f00000-0000-4000-8000-00000000000a"),
                criteria: Sha256Digest::parse(DIGEST).unwrap(),
                allocation: Allocation {
                    limit_ms: 1_200_000,
                    work_ms: 900_000,
                    verify_ms: 300_000,
                },
            },
            end(),
        )
        .unwrap();
    let input = Update {
        idempotency_key: OTHER.into(),
        record_id: None,
        expected_revision: None,
        definition: RosterDefinitionV1 {
            kind: Kind::Agent,
            display_name: "recovery-pid-fixture".into(),
            owner_id: "fixture-worker".into(),
            version: "v1".into(),
            capabilities: vec!["text".into()],
            locality: Locality::Local,
            endpoint_ref: Some(OTHER.into()),
            limitations: "test-only literal local adapter; no runtime grant".into(),
        },
        audit_reason: "bounded PID reuse fixture".into(),
    };
    let raw=serde_json::to_vec(&json!({"protocol":"hee3.control","version":1,"kind":"request","request_id":OTHER,"action":"roster.update","action_version":1,"idempotency_key":OTHER,"deadline_unix_ms":"1030000","authority":{"grant_id":OTHER,"scope_sha256":DIGEST},"precondition":null,"body":{"record_id":null,"definition":input.definition,"audit_reason":input.audit_reason}})).unwrap();
    let head = store
        .roster_apply(&principal(), &[input], RequestSource::Native(&raw), end())
        .unwrap()
        .remove(0)
        .head;
    store
        .roster_observe_worker(
            &principal(),
            &ObservationInput {
                record_id: head.record_id.clone(),
                record_version: head.record_version.clone(),
                owner_id: head.definition.owner_id.clone(),
                endpoint_ref: head.definition.endpoint_ref.clone(),
                instance_id: None,
                instance_generation: None,
                source: ObservationSource::Worker,
                observed_unix_ms: None,
                availability: Availability::Available,
                actual_identity: Some(identity.into()),
                immutable_revision: None,
                capabilities: vec!["text".into()],
                evidence_ref: OTHER.into(),
            },
            end(),
        )
        .unwrap();
    let selections = [Selection {
        record_id: head.record_id.clone(),
        expected_revision: head.record_version,
        capabilities: vec!["text".into()],
        local_only: true,
        version: Some("v1".into()),
        ttl_ms: 60_000,
    }];
    store
        .begin_rostered_attempt(
            RosterStart {
                principal: &principal(),
                task: id(TASK),
                expected: "1".parse().unwrap(),
                attempt: id(ATTEMPT),
                event: id(START),
                agent_record_id: &head.record_id,
                session: id(KEY),
                workspace: id(STAGE),
                selections: &selections,
                lease_ms: 1,
            },
            end(),
        )
        .unwrap();
    store
}

/// Actual identity of this process as the kernel describes it: field 22 of
/// `/proc/self/stat` (start time in clock ticks) and the PID namespace link.
fn identity() -> serde_json::Value {
    let stat = fs::read_to_string("/proc/self/stat").unwrap();
    let tail = &stat[stat.rfind(')').unwrap() + 2..];
    let start_ticks: u64 = tail.split_whitespace().nth(19).unwrap().parse().unwrap();
    let namespace = fs::read_link("/proc/self/ns/pid").unwrap();
    json!({
        "pid": std::process::id(),
        "start_ticks": start_ticks,
        "namespace": namespace.to_str().unwrap(),
    })
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 3, "usage: <workspace> old|replacement");
    let path = PathBuf::from(&args[1]);
    let mode = args[2].as_str();
    let observed = identity();
    let store = match mode {
        "old" => Some(rostered(&path, &serde_json::to_string(&observed).unwrap())),
        "replacement" => None,
        _ => panic!("unknown mode"),
    };
    retained(&path.join(format!("{mode}-identity.json")), &observed);
    retained(
        &path.join(format!("{mode}-writer.json")),
        &json!({"identity": observed, "literal": "owned finite local write"}),
    );
    drop(store);
    let mut out = std::io::stdout().lock();
    out.write_all(&serde_json::to_vec(&observed).unwrap())
        .unwrap();
    out.write_all(b"\n").unwrap();
    out.flush().unwrap();
    drop(out);
    let mut input = std::io::stdin().lock();
    let mut release = [0];
    input.read_exact(&mut release).unwrap();
    assert_eq!(release, [b'x'], "owned release byte");
    let mut trailing = Vec::new();
    input.read_to_end(&mut trailing).unwrap();
    retained(
        &path.join(format!("{mode}-release.json")),
        &json!({"identity": observed, "release": "x", "trailing_bytes": trailing.len()}),
    );
    assert!(trailing.is_empty(), "unexpected bytes after release");
}
