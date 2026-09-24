//! T28 IPC01 cases (a module of `t28_actions`): the control socket, its custody, the peer
//! principal, the file grant store, and the engine binary serving the bash wrapper end to end.
//!
//! Every case builds its own private directory under the system temporary directory (a Unix
//! socket path is limited to 108 bytes, which a deep target directory can exceed) and removes it.
use habitat_engine::actions::Effect;
use habitat_engine::actions::Owner;
use habitat_engine::actions::control::{Composed, Grants};
use habitat_engine::app::control_socket::{
    self, Ended, Error as SocketError, OPERATOR_ROLE, RUNTIME_DIRECTORY, SOCKET_NAME,
    serve_connection,
};
use habitat_engine::app::grants::{Error as GrantError, FileGrants, GRANT_SCHEMA, MAX_GRANT_BYTES};
use habitat_engine::contracts::control::{FrameFault, request_sha256};
use habitat_engine::store::Principal;
use serde_json::{Value, json};
use std::error::Error;
use std::fs::{self, DirBuilder};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt, symlink};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

type Outcome = Result<(), Box<dyn Error>>;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const NOW: u64 = 1_769_999_995_000;
const GRANT: &str = "123e4567-e89b-42d3-a456-0000000000aa";

/// A private directory removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Result<Self, Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!(
            "hee3-t28s-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&path)?;
        Ok(Self(path))
    }

    fn private(&self, name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let path = self.0.join(name);
        DirBuilder::new()
            .mode(0o700)
            .recursive(true)
            .create(&path)?;
        Ok(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn euid() -> u32 {
    rustix::process::geteuid().as_raw()
}

fn operator() -> Result<Principal, Box<dyn Error>> {
    Principal::new(euid(), OPERATOR_ROLE).map_err(|error| format!("{error:?}").into())
}

fn record(uid: u32, role: &str, owners: &[&str], effects: &[&str], expires: u64) -> Vec<u8> {
    serde_json::to_vec_pretty(&json!({
        "schema": GRANT_SCHEMA, "grant_id": GRANT, "uid": uid, "role": role,
        "owners": owners, "effects": effects, "expires_unix_ms": expires.to_string(),
    }))
    .unwrap_or_default()
}

fn write_grant(directory: &Path, name: &str, bytes: &[u8], mode: u32) -> Outcome {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(directory.join(name))?;
    file.write_all(bytes)?;
    fs::set_permissions(directory.join(name), fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[test]
fn the_runtime_root_must_be_named_absolute_and_private() -> Outcome {
    let scratch = Scratch::new()?;
    assert!(matches!(
        control_socket::runtime_root(None),
        Err(SocketError::NoRuntimeDirectory)
    ));
    assert!(matches!(
        control_socket::runtime_root(Some("relative/run".as_ref())),
        Err(SocketError::NoRuntimeDirectory)
    ));
    let root = scratch.private("run")?;
    assert_eq!(control_socket::runtime_root(Some(root.as_os_str()))?, root);
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755))?;
    assert!(matches!(
        control_socket::runtime_root(Some(root.as_os_str())),
        Err(SocketError::Custody("runtime root"))
    ));
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let link = scratch.0.join("link");
    symlink(&root, &link)?;
    assert!(
        control_socket::runtime_root(Some(link.as_os_str())).is_err(),
        "a link is not followed"
    );
    Ok(())
}

#[test]
fn a_live_engine_refuses_the_bind_and_a_stale_socket_is_cleared() -> Outcome {
    let scratch = Scratch::new()?;
    let root = scratch.private("run")?;
    let socket = control_socket::prepare(&root)?;
    assert_eq!(socket, root.join(RUNTIME_DIRECTORY).join(SOCKET_NAME));
    let listener = control_socket::bind(&socket)?;
    assert_eq!(
        fs::symlink_metadata(&socket)?.permissions().mode() & 0o777,
        0o600
    );
    assert!(matches!(
        control_socket::prepare(&root),
        Err(SocketError::Live)
    ));
    drop(listener);
    assert!(
        fs::symlink_metadata(&socket).is_ok(),
        "a dropped listener leaves its path"
    );
    // A sibling case forking a child at this instant holds a copy of the listening fd until the
    // child's exec closes it (CLOEXEC), and the socket then still accepts: `prepare` says Live,
    // which is the safe direction. So wait, with a budget, for no process to hold it.
    let started = Instant::now();
    let budget = Duration::from_secs(5);
    let prepared = loop {
        match control_socket::prepare(&root) {
            Err(SocketError::Live) => {
                assert!(started.elapsed() < budget, "still live after {budget:?}");
                std::thread::sleep(Duration::from_millis(10));
            }
            other => break other?,
        }
    };
    assert_eq!(prepared, socket);
    assert!(
        fs::symlink_metadata(&socket).is_err(),
        "the stale socket was removed"
    );
    // Something that is not a socket at that path is never removed.
    fs::write(&socket, b"not a socket")?;
    assert!(matches!(
        control_socket::prepare(&root),
        Err(SocketError::Custody("control socket path"))
    ));
    assert_eq!(fs::read(&socket)?, b"not a socket");
    // An engine directory another mode could share is refused, not repaired.
    fs::remove_file(&socket)?;
    fs::set_permissions(
        root.join(RUNTIME_DIRECTORY),
        fs::Permissions::from_mode(0o750),
    )?;
    assert!(matches!(
        control_socket::prepare(&root),
        Err(SocketError::Custody("engine runtime directory"))
    ));
    Ok(())
}

#[test]
fn only_the_operator_is_admitted_and_as_the_operator_role() -> Outcome {
    let admitted = control_socket::admit_peer(1000, 1000)?;
    assert!(admitted.is(1000, OPERATOR_ROLE));
    assert!(!admitted.is(1001, OPERATOR_ROLE));
    assert_eq!(
        control_socket::admit_peer(1001, 1000).err(),
        Some("peer uid 1001 is not the operator".to_owned())
    );
    assert!(
        control_socket::admit_peer(0, 1000).is_err(),
        "root is not the operator either"
    );
    Ok(())
}

#[test]
fn the_peer_is_the_kernels_account_of_the_other_end() -> Outcome {
    let (left, right) = UnixStream::pair()?;
    assert_eq!(control_socket::peer_uid(&left)?, euid());
    assert_eq!(control_socket::peer_uid(&right)?, euid());
    Ok(())
}

/// Grants every visible effect to anyone: the connection cases are about framing, not grants.
struct Open;

impl Grants for Open {
    fn resolve(
        &self,
        _: &Principal,
        _: &str,
        _: &str,
        _: u64,
    ) -> Option<habitat_engine::actions::Caller> {
        let caller = Owner::ALL.into_iter().fold(
            habitat_engine::actions::Caller::new(),
            habitat_engine::actions::Caller::seeing,
        );
        Some(
            Effect::ALL
                .into_iter()
                .fold(caller, habitat_engine::actions::Caller::granted),
        )
    }
}

fn listing(request: u8) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "protocol": "hee3.control", "version": 1, "kind": "request",
        "request_id": format!("123e4567-e89b-42d3-a456-0000000000{request:02x}"),
        "action": "tools.list", "action_version": 1, "idempotency_key": null,
        "deadline_unix_ms": (NOW + 1_000).to_string(),
        "authority": {"grant_id": GRANT, "scope_sha256": format!("sha256:{}", "3".repeat(64))},
        "precondition": null,
        "body": {"query": null, "page": {"limit": 1, "cursor": null}},
    }))
    .unwrap_or_default()
}

#[test]
fn a_connection_is_answered_in_order_and_closed_at_its_first_unanswerable_frame() -> Outcome {
    let mut stream = Vec::new();
    for request in [1_u8, 2] {
        stream.extend(listing(request));
        stream.push(b'\n');
    }
    stream.extend(b"{ }\n");
    stream.extend(listing(3));
    stream.push(b'\n');
    let mut written = Vec::new();
    let ended = serve_connection(
        stream.as_slice(),
        &mut written,
        &operator()?,
        Composed {
            grants: &Open,
            health: None,
            tasks: None,
        },
        &|| NOW,
    )?;
    assert_eq!(
        ended,
        Ended::Closed {
            served: 2,
            fault: FrameFault::Whitespace
        }
    );
    let replies: Vec<Value> = written
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .map(serde_json::from_slice)
        .collect::<Result<_, _>>()?;
    let ids: Vec<&Value> = replies.iter().map(|reply| &reply["request_id"]).collect();
    assert_eq!(
        ids,
        [
            &json!("123e4567-e89b-42d3-a456-000000000001"),
            &json!("123e4567-e89b-42d3-a456-000000000002")
        ],
        "two replies in request order; the frame after the fault is never answered"
    );
    let mut clean = Vec::new();
    let mut only = listing(4);
    only.push(b'\n');
    assert_eq!(
        serve_connection(
            only.as_slice(),
            &mut clean,
            &operator()?,
            Composed {
                grants: &Open,
                health: None,
                tasks: None
            },
            &|| NOW
        )?,
        Ended::Clean { served: 1 }
    );
    Ok(())
}

#[test]
fn a_grant_resolves_only_as_its_reviewed_record_says() -> Outcome {
    let scratch = Scratch::new()?;
    let directory = scratch.private("grants")?;
    let bytes = record(
        euid(),
        OPERATOR_ROLE,
        &["actions", "task"],
        &["read"],
        NOW + 10,
    );
    write_grant(&directory, &format!("{GRANT}.json"), &bytes, 0o600)?;
    let store = FileGrants::open(&directory)?;
    let scope = request_sha256(&bytes);
    let caller = store
        .resolve(&operator()?, GRANT, &scope, NOW)
        .ok_or("a reviewed grant resolves")?;
    let seen: Vec<Owner> = Owner::ALL
        .into_iter()
        .filter(|owner| caller.sees(*owner))
        .collect();
    let held: Vec<Effect> = Effect::ALL
        .into_iter()
        .filter(|effect| caller.holds(*effect))
        .collect();
    assert_eq!(
        (seen, held),
        (vec![Owner::Actions, Owner::Task], vec![Effect::Read])
    );
    let other_role = Principal::new(euid(), "reviewer").map_err(|error| format!("{error:?}"))?;
    let other_uid =
        Principal::new(euid() + 1, OPERATOR_ROLE).map_err(|error| format!("{error:?}"))?;
    for (case, principal, grant, presented, now) in [
        (
            "another scope",
            operator()?,
            GRANT,
            request_sha256(b"{}"),
            NOW,
        ),
        ("another role", other_role, GRANT, scope.clone(), NOW),
        ("another uid", other_uid, GRANT, scope.clone(), NOW),
        (
            "expired at the instant",
            operator()?,
            GRANT,
            scope.clone(),
            NOW + 10,
        ),
        (
            "unknown grant",
            operator()?,
            "123e4567-e89b-42d3-a456-0000000000bb",
            scope.clone(),
            NOW,
        ),
        ("not a uuid", operator()?, "../grants", scope.clone(), NOW),
    ] {
        assert!(
            store.resolve(&principal, grant, &presented, now).is_none(),
            "{case}"
        );
    }
    // One step off the expiry instant, from the admitted side.
    assert!(
        store
            .resolve(&operator()?, GRANT, &scope, NOW + 9)
            .is_some()
    );
    Ok(())
}

#[test]
fn a_record_the_store_cannot_trust_resolves_to_nothing() -> Outcome {
    let scratch = Scratch::new()?;
    let refusals: Vec<(&str, Vec<u8>, u32)> = vec![
        (
            "readable by others",
            record(euid(), OPERATOR_ROLE, &["actions"], &["read"], NOW + 10),
            0o644,
        ),
        (
            "names another grant",
            {
                let mut value: Value = serde_json::from_slice(&record(
                    euid(),
                    OPERATOR_ROLE,
                    &["actions"],
                    &["read"],
                    NOW + 10,
                ))?;
                value["grant_id"] = json!("123e4567-e89b-42d3-a456-0000000000cc");
                serde_json::to_vec(&value)?
            },
            0o600,
        ),
        (
            "unknown member",
            {
                let mut value: Value = serde_json::from_slice(&record(
                    euid(),
                    OPERATOR_ROLE,
                    &["actions"],
                    &["read"],
                    NOW + 10,
                ))?;
                value["bearer"] = json!(true);
                serde_json::to_vec(&value)?
            },
            0o600,
        ),
        (
            "unknown owner",
            record(euid(), OPERATOR_ROLE, &["everything"], &["read"], NOW + 10),
            0o600,
        ),
        (
            "unknown effect",
            record(euid(), OPERATOR_ROLE, &["actions"], &["root"], NOW + 10),
            0o600,
        ),
        (
            "another schema",
            {
                let mut value: Value = serde_json::from_slice(&record(
                    euid(),
                    OPERATOR_ROLE,
                    &["actions"],
                    &["read"],
                    NOW + 10,
                ))?;
                value["schema"] = json!("hee3.grant/2");
                serde_json::to_vec(&value)?
            },
            0o600,
        ),
        (
            "over the bound",
            {
                let mut bytes = record(euid(), OPERATOR_ROLE, &["actions"], &["read"], NOW + 10);
                bytes.resize(usize::try_from(MAX_GRANT_BYTES)? + 1, b' ');
                bytes
            },
            0o600,
        ),
    ];
    for (case, bytes, mode) in refusals {
        let directory = scratch.private(&case.replace(' ', "-"))?;
        write_grant(&directory, &format!("{GRANT}.json"), &bytes, mode)?;
        let store = FileGrants::open(&directory)?;
        assert!(
            store
                .resolve(&operator()?, GRANT, &request_sha256(&bytes), NOW)
                .is_none(),
            "{case}"
        );
    }
    Ok(())
}

#[test]
fn a_linked_record_or_a_shared_directory_is_not_a_grant() -> Outcome {
    let scratch = Scratch::new()?;
    // A link to a reviewed record is not the record.
    let real = scratch.private("real")?;
    let bytes = record(euid(), OPERATOR_ROLE, &["actions"], &["read"], NOW + 10);
    write_grant(&real, "elsewhere.json", &bytes, 0o600)?;
    let linked = scratch.private("linked")?;
    symlink(
        real.join("elsewhere.json"),
        linked.join(format!("{GRANT}.json")),
    )?;
    assert!(
        FileGrants::open(&linked)?
            .resolve(&operator()?, GRANT, &request_sha256(&bytes), NOW)
            .is_none()
    );
    // And a directory another user could write to is not a store at all.
    let shared = scratch.private("shared")?;
    fs::set_permissions(&shared, fs::Permissions::from_mode(0o755))?;
    assert!(matches!(
        FileGrants::open(&shared),
        Err(GrantError::Custody)
    ));
    Ok(())
}

/// The engine binary under a private runtime root and home, killed by its own handle on drop.
struct Engine {
    child: Child,
}

impl Engine {
    fn start(run: &Path, home: &Path) -> Result<Self, Box<dyn Error>> {
        let child = Command::new(env!("CARGO_BIN_EXE_habitat-engine"))
            .arg("serve")
            .env("XDG_RUNTIME_DIR", run)
            .env("HOME", home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let socket = run.join(RUNTIME_DIRECTORY).join(SOCKET_NAME);
        let started = Instant::now();
        let budget = Duration::from_secs(20);
        // Wait on the artifact the engine produces, with a budget: a connectable socket.
        while UnixStream::connect(&socket).is_err() {
            assert!(started.elapsed() < budget, "no socket within {budget:?}");
            std::thread::sleep(Duration::from_millis(20));
        }
        Ok(Self { child })
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The child's output once it exits, or an error naming the budget after it is killed and reaped.
fn exits_within(mut child: Child, budget: Duration) -> Result<Output, Box<dyn Error>> {
    let started = Instant::now();
    while child.try_wait()?.is_none() {
        if started.elapsed() >= budget {
            child.kill()?;
            child.wait()?;
            return Err(format!("still running after {budget:?}").into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Ok(child.wait_with_output()?)
}

fn wrapper(run: &Path, scope: &str, argv: &[&str]) -> Result<Output, Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(Command::new("bash")
        .arg(root.join("integrations/bash/hee3"))
        .args(argv)
        .env("XDG_RUNTIME_DIR", run)
        .env("HEE3_PRODUCER", env!("CARGO_BIN_EXE_habitat-engine"))
        .env("HEE3_GRANT_ID", GRANT)
        .env("HEE3_SCOPE_SHA256", scope)
        .env_remove("FORCE_COLOR")
        .env_remove("HEE3_CATALOGUE")
        .output()?)
}

/// A private runtime root and home holding one reviewed grant, valid for ten minutes.
struct World {
    /// Held so the directories outlive the case; removed on drop.
    _scratch: Scratch,
    run: PathBuf,
    home: PathBuf,
    scope: String,
}

impl World {
    fn new() -> Result<Self, Box<dyn Error>> {
        Self::seeing(&["actions", "task"])
    }

    fn seeing(owners: &[&str]) -> Result<Self, Box<dyn Error>> {
        let scratch = Scratch::new()?;
        let run = scratch.private("run")?;
        let home = scratch.private("home")?;
        let grants = scratch.private("home/.config/herdr-engineering-engine-v3/grants")?;
        let now = u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis(),
        )?;
        let bytes = record(euid(), OPERATOR_ROLE, owners, &["read"], now + 600_000);
        write_grant(&grants, &format!("{GRANT}.json"), &bytes, 0o600)?;
        let scope = request_sha256(&bytes);
        Ok(Self {
            _scratch: scratch,
            run,
            home,
            scope,
        })
    }
}

#[test]
fn the_engine_serves_the_bash_wrapper_end_to_end() -> Outcome {
    let world = World::new()?;
    let (run, scope) = (&world.run, &world.scope);
    let _engine = Engine::start(run, &world.home)?;

    let listed = wrapper(
        run,
        scope,
        &[
            "tools.list",
            "query=task",
            r#"page:={"limit":2,"cursor":null}"#,
        ],
    )?;
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let reply: Value = serde_json::from_slice(&listed.stdout)?;
    assert_eq!(reply["kind"], json!("result"));
    let ids: Vec<&Value> = reply["body"]["page"]["items"]
        .as_array()
        .ok_or("items")?
        .iter()
        .map(|item| &item["id"])
        .collect();
    assert_eq!(ids, [&json!("task.preview"), &json!("task.submit")]);
    assert_eq!(
        reply["body"]["page"]["next_cursor"]["after_key"],
        json!("task.submit")
    );

    // The same request under a substituted scope: denied by the grant store, over the socket.
    let substituted = wrapper(
        run,
        &request_sha256(b"{}"),
        &[
            "tools.list",
            "query:=null",
            r#"page:={"limit":1,"cursor":null}"#,
        ],
    )?;
    let denied: Value = serde_json::from_slice(&substituted.stdout)?;
    assert_eq!(
        (denied["code"].as_str(), denied["details"]["field"].as_str()),
        (Some("forbidden"), Some("/authority/grant_id"))
    );

    Ok(())
}

#[test]
fn a_second_engine_is_refused_and_a_killed_one_is_replaced() -> Outcome {
    let world = World::new()?;
    let (run, home, scope) = (&world.run, &world.home, &world.scope);
    let engine = Engine::start(run, home)?;
    // A second engine is refused while the first serves; after a kill, the stale socket clears.
    let second = Command::new(env!("CARGO_BIN_EXE_habitat-engine"))
        .arg("serve")
        .env("XDG_RUNTIME_DIR", run)
        .env("HOME", home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    // A second engine that wrongly starts serves forever: wait on it with a budget, or this case
    // would hang instead of fail (a planted mutant found exactly that).
    let second = exits_within(second, Duration::from_secs(10))?;
    assert_eq!(
        second.status.code(),
        Some(6),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert!(String::from_utf8_lossy(&second.stderr).contains("Live"));
    drop(engine);
    let restarted = Engine::start(run, home)?;
    let again = wrapper(
        run,
        scope,
        &[
            "tools.list",
            "query=tools",
            r#"page:={"limit":5,"cursor":null}"#,
        ],
    )?;
    let reply: Value = serde_json::from_slice(&again.stdout)?;
    assert_eq!(
        reply["body"]["page"]["items"].as_array().map(Vec::len),
        Some(2)
    );
    drop(restarted);

    // No engine: the producer says so with the wrapper's own code for it.
    let absent = wrapper(
        run,
        scope,
        &[
            "tools.list",
            "query:=null",
            r#"page:={"limit":1,"cursor":null}"#,
        ],
    )?;
    assert_eq!(
        absent.status.code(),
        Some(3),
        "{}",
        String::from_utf8_lossy(&absent.stderr)
    );
    Ok(())
}

#[test]
fn the_engine_serves_the_health_its_start_left() -> Outcome {
    let world = World::seeing(&["app"])?;
    let (run, home, scope) = (&world.run, &world.home, &world.scope);
    // Nothing commissioned: the engine serves, and says it cannot act.
    let engine = Engine::start(run, home)?;
    let blocked: Value = serde_json::from_slice(&wrapper(run, scope, &["health"])?.stdout)?;
    assert_eq!(
        (
            &blocked["body"]["ready"],
            &blocked["body"]["recovery"],
            &blocked["body"]["database"]
        ),
        (&json!(false), &json!("blocked"), &json!("unavailable"))
    );
    drop(engine);
    // A commissioned, empty ledger: reconciled at start, ready.
    let state = home.join(".local/state/herdr-engineering-engine-v3");
    DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(&state)?;
    let (generation, epoch) = (
        "28c00000-0000-4000-8000-0000000000a1",
        "28c00000-0000-4000-8000-0000000000a2",
    );
    drop(
        habitat_engine::store::Store::open(
            &state,
            habitat_engine::contracts::UuidV4::parse(generation)?,
            habitat_engine::contracts::UuidV4::parse(epoch)?,
            true,
            Instant::now() + Duration::from_secs(10),
        )
        .map_err(|error| format!("{error:?}"))?,
    );
    let body = serde_json::to_vec(
        &json!({"schema": "hee3.active-generation/1", "generation": generation, "epoch": epoch}),
    )?;
    write_grant(&state, "active.json", &body, 0o600)?;
    let _engine = Engine::start(run, home)?;
    let ready: Value = serde_json::from_slice(&wrapper(run, scope, &["health"])?.stdout)?;
    assert_eq!(ready["kind"], json!("result"), "{ready}");
    assert_eq!(
        (
            &ready["body"]["ready"],
            &ready["body"]["recovery"],
            &ready["body"]["database"],
            &ready["body"]["socket"]
        ),
        (
            &json!(true),
            &json!("complete"),
            &json!("ready"),
            &json!("owned")
        )
    );
    Ok(())
}
