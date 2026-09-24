//! T28 IPC01 cases (a module of `t28_actions`): the control socket, its custody, the peer
//! principal, the file grant store, and the engine binary serving the bash wrapper end to end.
//!
//! Every case builds its own private directory under the system temporary directory (a Unix
//! socket path is limited to 108 bytes, which a deep target directory can exceed) and removes it.
use habitat_engine::actions::Effect;
use habitat_engine::actions::Owner;
use habitat_engine::actions::control::{self, Composed, Grants, Reply};
use habitat_engine::app::control_socket::{
    self, Ended, Error as SocketError, OPERATOR_ROLE, RUNTIME_DIRECTORY, SOCKET_NAME,
    serve_connection,
};
use habitat_engine::app::grants::{Error as GrantError, FileGrants, GRANT_SCHEMA, MAX_GRANT_BYTES};
use habitat_engine::app::tasks::{StoreTasks, submit_readback};
use habitat_engine::contracts::UuidV4;
use habitat_engine::contracts::control::{FrameFault, FrameReader, ReadError, request_sha256};
use habitat_engine::store::{Principal, Store};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
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

fn now_unix_ms() -> Result<u64, Box<dyn Error>> {
    Ok(u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis(),
    )?)
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
    let prepared = control_socket::prepare(&root)?;
    let socket = prepared.socket().to_owned();
    assert_eq!(socket, root.join(RUNTIME_DIRECTORY).join(SOCKET_NAME));
    let listener = control_socket::bind(&prepared)?;
    assert_eq!(
        fs::symlink_metadata(&socket)?.permissions().mode() & 0o777,
        0o600
    );
    assert!(matches!(
        control_socket::prepare(&root),
        Err(SocketError::Live)
    ));
    drop(listener);
    drop(prepared);
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
    assert_eq!(prepared.socket(), socket);
    drop(prepared);
    assert!(
        fs::symlink_metadata(&socket).is_err(),
        "the stale socket was removed"
    );
    // Something that is not a socket at that path is never removed.
    fs::write(&socket, b"not a socket")?;
    assert!(matches!(
        prepare_settled(&root),
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
        &control_socket::Admission::new(),
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
            &control_socket::Admission::new(),
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
        Self::start_with(run, home, Stdio::null())
    }

    /// Start with standard error written to `log`, which the case reads once the socket accepts:
    /// every line `serve` writes before it binds is there by then.
    fn start_logged(run: &Path, home: &Path, log: &Path) -> Result<Self, Box<dyn Error>> {
        Self::start_with(run, home, Stdio::from(fs::File::create_new(log)?))
    }

    fn start_with(run: &Path, home: &Path, stderr: Stdio) -> Result<Self, Box<dyn Error>> {
        let child = Command::new(env!("CARGO_BIN_EXE_habitat-engine"))
            .arg("serve")
            .env("XDG_RUNTIME_DIR", run)
            .env("HOME", home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(stderr)
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
        Self::granting(owners, &["read"])
    }

    /// A world whose one grant sees `owners` and holds `effects` (their plan-spine names).
    fn granting(owners: &[&str], effects: &[&str]) -> Result<Self, Box<dyn Error>> {
        let scratch = Scratch::new()?;
        let run = scratch.private("run")?;
        let home = scratch.private("home")?;
        let grants = scratch.private("home/.config/herdr-engineering-engine-v3/grants")?;
        let now = u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis(),
        )?;
        let bytes = record(euid(), OPERATOR_ROLE, owners, effects, now + 600_000);
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

/// The generation and epoch every commissioned scratch ledger here is created with.
const GENERATION: &str = "28c00000-0000-4000-8000-0000000000a1";
const EPOCH: &str = "28c00000-0000-4000-8000-0000000000a2";

/// Commission an empty ledger and its active-generation manifest under `home`'s state root, as the
/// operator's commissioning would (RC02); a scratch home, never the operator's own. Returns the
/// ledger file.
fn commission(home: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let state = home.join(".local/state/herdr-engineering-engine-v3");
    DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(&state)?;
    drop(ledger(&state)?);
    let body = serde_json::to_vec(
        &json!({"schema": "hee3.active-generation/1", "generation": GENERATION, "epoch": EPOCH}),
    )?;
    write_grant(&state, "active.json", &body, 0o600)?;
    Ok(state
        .join("generations")
        .join(GENERATION)
        .join("ledger.sqlite3"))
}

/// The ledger at `root` for [`GENERATION`] and [`EPOCH`], created when absent.
fn ledger(root: &Path) -> Result<Store, Box<dyn Error>> {
    Store::open(
        root,
        UuidV4::parse(GENERATION)?,
        UuidV4::parse(EPOCH)?,
        true,
        Instant::now() + Duration::from_secs(10),
    )
    .map_err(|error| format!("{error:?}").into())
}

/// The producer door the wrapper execs: `habitat-engine <action> < request`, sending `request`'s
/// exact bytes as one frame, with its standard output going to `stdout`.
fn producer(
    run: &Path,
    action: &str,
    request: &[u8],
    stdout: Stdio,
) -> Result<Output, Box<dyn Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_habitat-engine"))
        .arg(action)
        .env("XDG_RUNTIME_DIR", run)
        .stdin(Stdio::piped())
        .stdout(stdout)
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(request)?;
    exits_within(child, Duration::from_secs(20))
}

/// The request the wrapper would send, built once so its exact bytes can be sent again (RC03 §3:
/// senders serialise once and retain the bytes; a replay is a byte-exact resend).
fn built(run: &Path, scope: &str, argv: &[&str]) -> Result<Vec<u8>, Box<dyn Error>> {
    let checked = wrapper(run, scope, &[&["--check"], argv].concat())?;
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    Ok(checked.stdout)
}

/// One reply record from a door's output, which must have exited 0.
fn reply_of(output: &Output) -> Result<Value, Box<dyn Error>> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

/// The producer's exit code for a reply the engine sent as a typed error record (BASH-G1).
const EXIT_REFUSED: i32 = 7;

/// One engine error record from a door's output: exit [`EXIT_REFUSED`], the record on stdout.
fn error_of(output: &Output) -> Result<Value, Box<dyn Error>> {
    assert_eq!(
        output.status.code(),
        Some(EXIT_REFUSED),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let record: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(record["kind"], json!("error"), "{record}");
    Ok(record)
}

/// `reply` with the values at `pointers` set to null; every pointer must name a value, so a mask
/// can never pass because what it masks is missing.
fn without(mut reply: Value, pointers: &[&str]) -> Result<Value, Box<dyn Error>> {
    for pointer in pointers {
        *reply
            .pointer_mut(pointer)
            .ok_or_else(|| format!("{pointer} is absent"))? = Value::Null;
    }
    Ok(reply)
}

/// What the receiver's own clock stamps on a submit's result: the cursor's issue and expiry.
const CLOCK: [&str; 2] = [
    "/body/engine_cursor/issued_unix_ms",
    "/body/engine_cursor/expires_unix_ms",
];

fn submitting(key: &str, intent: &str) -> Vec<String> {
    let mut spec = super::tasks::spec();
    spec["intent"] = json!(intent);
    vec![
        "task.submit".to_owned(),
        format!("@idempotency_key={key}"),
        format!("spec:={spec}"),
    ]
}

fn getting(selector: &Value) -> Vec<String> {
    vec![
        "task.get".to_owned(),
        format!("selector:={selector}"),
        "evidence=none".to_owned(),
    ]
}

fn strs(argv: &[String]) -> Vec<&str> {
    argv.iter().map(String::as_str).collect()
}

/// `task.get` through the wrapper for `selector`.
fn get_through(run: &Path, scope: &str, selector: &Value) -> Result<Value, Box<dyn Error>> {
    reply_of(&wrapper(run, scope, &strs(&getting(selector)))?)
}

/// The selector a caller holds before sending: the submit's own idempotency key (RC03 §6).
fn key_selector(key: &str) -> Value {
    json!({"source_action": "task.submit", "idempotency_key": key})
}

#[test]
fn the_engine_admits_and_reads_back_a_task_through_the_wrapper() -> Outcome {
    const KEY: &str = "28c00000-0000-4000-8000-0000000000b1";
    let world = World::granting(&["task"], &["read", "durable admission"])?;
    let (run, scope) = (&world.run, &world.scope);
    commission(&world.home)?;
    let _engine = Engine::start(run, &world.home)?;

    let submitted = reply_of(&wrapper(
        run,
        scope,
        &strs(&submitting(KEY, "Add a strict decimal parser.")),
    )?)?;
    assert_eq!(
        (
            &submitted["kind"],
            &submitted["effect"],
            &submitted["replayed"],
            &submitted["observed_generation"]
        ),
        (
            &json!("result"),
            &json!("committed"),
            &json!(false),
            &json!("1")
        ),
        "{submitted}"
    );
    assert_eq!(submitted["readback"], submit_readback(KEY));
    let task = submitted["body"]["task"]["task_id"]
        .as_str()
        .ok_or("task id")?
        .to_owned();
    UuidV4::parse(&task)?;
    assert_eq!(
        submitted["body"]["task"],
        json!({"task_id": task, "generation": "1", "state": "admitted", "current_attempt_id": null, "unresolved_obligations": 0})
    );
    let filter = format!("{{\"resource_ids\":[\"{task}\"],\"topics\":[\"task\"]}}");
    assert_eq!(
        (
            &submitted["body"]["engine_cursor"]["epoch"],
            &submitted["body"]["engine_cursor"]["filter_sha256"]
        ),
        (
            &json!(EPOCH),
            &json!(super::tasks::digest(Sha256::digest(filter.as_bytes())))
        )
    );

    // Read back by the key the caller held before sending, and by the identity it was given.
    let by_key = get_through(run, scope, &key_selector(KEY))?;
    let by_id = get_through(run, scope, &json!({"task_id": task}))?;
    assert_eq!(by_key["body"]["task"], submitted["body"]["task"]);
    assert_eq!(by_id["body"]["task"], submitted["body"]["task"]);
    let criteria = super::tasks::digest(Sha256::digest(
        br#"["rejects a leading zero","round-trips the maximum"]"#,
    ));
    for read in [&by_key, &by_id] {
        assert_eq!(
            (
                &read["effect"],
                &read["observed_generation"],
                &read["body"]["criteria_sha256"],
                &read["body"]["attempts"],
                &read["body"]["cleanup"],
                &read["body"]["delivery"],
                &read["body"]["evidence"]
            ),
            (
                &json!("none"),
                &json!("1"),
                &json!(criteria),
                &json!([]),
                &json!("none"),
                &json!("none"),
                &json!([])
            ),
            "{read}"
        );
    }

    // Calling the wrapper again builds a new request (its own request_id and deadline), and a
    // replay is byte-exact (RC03 §3): the key conflicts, and nothing new is admitted.
    let retried = error_of(&wrapper(
        run,
        scope,
        &strs(&submitting(KEY, "Add a strict decimal parser.")),
    )?)?;
    assert_eq!(
        (
            &retried["code"],
            &retried["effect"],
            &retried["details"]["field"]
        ),
        (
            &json!("conflict"),
            &json!("none"),
            &json!("/idempotency_key")
        ),
        "{retried}"
    );
    let after = get_through(run, scope, &key_selector(KEY))?;
    assert_eq!(after["body"]["task"], submitted["body"]["task"]);
    Ok(())
}

#[test]
fn an_admission_survives_a_kill_after_commit_and_its_exact_bytes_replay() -> Outcome {
    const KEY: &str = "28c00000-0000-4000-8000-0000000000b2";
    let world = World::granting(&["task"], &["read", "durable admission"])?;
    let (run, home, scope) = (&world.run, &world.home, &world.scope);
    commission(home)?;
    let request = built(
        run,
        scope,
        &strs(&submitting(KEY, "Add a strict decimal parser.")),
    )?;
    let engine = Engine::start(run, home)?;
    let first = reply_of(&producer(run, "task.submit", &request, Stdio::piped())?)?;
    assert_eq!(
        (&first["effect"], &first["replayed"]),
        (&json!("committed"), &json!(false)),
        "{first}"
    );
    assert_eq!(
        first["request_sha256"],
        json!(request_sha256(request.strip_suffix(b"\n").ok_or("LF")?))
    );
    let task = first["body"]["task"]["task_id"].clone();

    // CLI/UDS parity: the same bytes through the library receiver, over a ledger of the same
    // generation and epoch, answer the same record but for what each ledger draws or stamps
    // for itself: the task identity, the filter digest over it, and the receiver's clock.
    let scratch = Scratch::new()?;
    let local = StoreTasks::new(ledger(&scratch.private("state")?)?, EPOCH.to_owned());
    let Reply::Frame(bytes) = control::serve_composed(
        request.strip_suffix(b"\n").ok_or("LF")?,
        now_unix_ms()?,
        &operator()?,
        Composed {
            grants: &Open,
            health: None,
            tasks: Some(&local),
        },
    ) else {
        return Err("the library receiver closed the connection".into());
    };
    let drawn = [
        "/body/task/task_id",
        "/body/engine_cursor/filter_sha256",
        CLOCK[0],
        CLOCK[1],
    ];
    assert_eq!(
        without(serde_json::from_slice(&bytes)?, &drawn)?,
        without(first.clone(), &drawn)?
    );

    // Killed after the commit was acknowledged (SIGKILL: nothing runs on the way down).
    drop(engine);
    let _restarted = Engine::start(run, home)?;
    let found = get_through(run, scope, &key_selector(KEY))?;
    assert_eq!(found["body"]["task"], first["body"]["task"]);

    // The retained bytes, resent: the stored admission, identical but for `replayed` and the
    // receiver's clock.
    let again = reply_of(&producer(run, "task.submit", &request, Stdio::piped())?)?;
    assert_eq!(again["replayed"], json!(true), "{again}");
    let mut expected = without(first.clone(), &CLOCK)?;
    expected["replayed"] = json!(true);
    assert_eq!(without(again, &CLOCK)?, expected);

    // The same key under one changed field: a conflict, and the admission is untouched.
    let changed = built(
        run,
        scope,
        &strs(&submitting(KEY, "Add a strict decimal parser!")),
    )?;
    let conflict = error_of(&producer(run, "task.submit", &changed, Stdio::piped())?)?;
    assert_eq!(
        (&conflict["code"], &conflict["details"]["field"]),
        (&json!("conflict"), &json!("/idempotency_key")),
        "{conflict}"
    );
    let still = get_through(run, scope, &json!({"task_id": task}))?;
    assert_eq!(still["body"]["task"], first["body"]["task"]);
    Ok(())
}

#[test]
fn an_engine_error_reply_is_its_own_exit_code_and_is_named() -> Outcome {
    let world = World::new()?;
    let (run, scope) = (&world.run, &world.scope);
    let _engine = Engine::start(run, &world.home)?;
    let argv = [
        "tools.list",
        "query:=null",
        r#"page:={"limit":1,"cursor":null}"#,
    ];
    // Benign mirror: a result record exits 0.
    let served = producer(
        run,
        "tools.list",
        &built(run, scope, &argv)?,
        Stdio::piped(),
    )?;
    assert_eq!(reply_of(&served)?["kind"], json!("result"));
    // The same request under a substituted scope: the engine answers with an error record. The
    // record is printed unchanged, the exit code alone says it was a refusal, and stderr names it.
    let substituted = request_sha256(b"{}");
    let refused = producer(
        run,
        "tools.list",
        &built(run, &substituted, &argv)?,
        Stdio::piped(),
    )?;
    let record = error_of(&refused)?;
    assert_eq!(record["code"], json!("forbidden"));
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        "habitat-engine: the engine refused the request: forbidden\n"
    );
    // Through the wrapper the producer's code passes unchanged.
    let wrapped = wrapper(run, &substituted, &argv)?;
    assert_eq!(error_of(&wrapped)?["code"], json!("forbidden"));
    Ok(())
}

/// Run a `hee3 chain` spec through the wrapper against the engine in `run`.
fn chain(run: &Path, scope: &str, steps: &Value) -> Result<Output, Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let spec = scratch.0.join("chain.json");
    fs::write(
        &spec,
        serde_json::to_vec(
            &json!({"protocol": "hee3.chain", "version": 1, "timeout_ms": 20000, "steps": steps}),
        )?,
    )?;
    wrapper(run, scope, &["chain", spec.to_str().ok_or("path")?])
}

/// BASH-G3: T29 composition (a) as a `hee3 chain` through the real engine. The spec's intent
/// carries spaces, quotes, `$(...)` and `;` as data; the submit's task id travels by JSON pointer
/// into the read-back; and a refusing step stops the chain with its own status.
#[test]
fn a_chain_submits_and_reads_back_through_the_engine_and_stops_at_a_refusal() -> Outcome {
    const KEY: &str = "28c00000-0000-4000-8000-0000000000c1";
    let world = World::granting(&["task"], &["read", "durable admission"])?;
    let (run, scope) = (&world.run, &world.scope);
    commission(&world.home)?;
    let _engine = Engine::start(run, &world.home)?;
    let mut spec = super::tasks::spec();
    spec["intent"] = json!("keep  spaces; 'single' \"double\" $(touch pwned) and ; as data");
    let submit = json!({"id": "submit", "action": "task.submit",
                        "arguments": {"@idempotency_key": KEY, "spec": spec},
                        "output": "json", "provides": ["/body/task/task_id"]});
    let chained = chain(
        run,
        scope,
        &json!([submit, {"id": "get", "action": "task.get",
                         "arguments": {"evidence": "none"}, "output": "json",
                         "inputs": {"/selector/task_id": {"step": "submit",
                                                          "field": "/body/task/task_id"}}}]),
    )?;
    let records: Vec<Value> = chained
        .stdout
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .map(serde_json::from_slice)
        .collect::<Result<_, _>>()?;
    assert_eq!(chained.status.code(), Some(0), "{records:?}");
    let task = records[0]["result"]["body"]["task"]["task_id"].clone();
    UuidV4::parse(task.as_str().ok_or("task id")?)?;
    let ordered: Vec<(&Value, &Value, &Value)> = records
        .iter()
        .map(|record| (&record["kind"], &record["step"], &record["outcome"]))
        .collect();
    assert_eq!(
        ordered,
        [
            (&json!("step"), &json!("submit"), &json!("ok")),
            (&json!("step"), &json!("get"), &json!("ok")),
            (&json!("summary"), &Value::Null, &json!("ok")),
        ]
    );
    assert_eq!(records[1]["result"]["body"]["task"]["task_id"], task);
    // The spec's `$(touch pwned)` was data end to end: nothing ran it, here or in the wrapper.
    assert!(!Path::new(env!("CARGO_MANIFEST_DIR")).join("pwned").exists());
    assert!(!Path::new("pwned").exists());
    assert_eq!(
        get_through(run, scope, &key_selector(KEY))?["body"]["task"]["task_id"],
        task
    );

    // The same key with other bytes: the engine refuses, the step fails with the producer's own
    // status (7, passed through), and the step after it never runs.
    let mut changed = spec.clone();
    changed["intent"] = json!("a different intent under the same key");
    let refused = chain(
        run,
        scope,
        &json!([{"id": "again", "action": "task.submit",
                 "arguments": {"@idempotency_key": KEY, "spec": changed}, "output": "json"},
                {"id": "never", "action": "task.get",
                 "arguments": {"selector": {"task_id": task}, "evidence": "none"}}]),
    )?;
    let summary: Value = serde_json::from_slice(
        refused
            .stdout
            .split(|&b| b == b'\n')
            .rfind(|line| !line.is_empty())
            .ok_or("no summary")?,
    )?;
    assert_eq!(
        (
            refused.status.code(),
            &summary["outcome"],
            &summary["status"],
            &summary["failed_step"],
            &summary["not_run"]
        ),
        (
            Some(EXIT_REFUSED),
            &json!("failed"),
            &json!(EXIT_REFUSED),
            &json!("again"),
            &json!(["never"])
        ),
        "{summary}"
    );
    Ok(())
}

#[test]
fn a_producer_that_cannot_write_its_reply_says_so() -> Outcome {
    let world = World::new()?;
    let (run, scope) = (&world.run, &world.scope);
    let _engine = Engine::start(run, &world.home)?;
    let request = built(
        run,
        scope,
        &[
            "tools.list",
            "query:=null",
            r#"page:={"limit":1,"cursor":null}"#,
        ],
    )?;
    // Someone reads the reply: it is written, and the producer succeeds.
    let read = producer(run, "tools.list", &request, Stdio::piped())?;
    assert_eq!(reply_of(&read)?["kind"], json!("result"));
    // Nobody can: the producer fails with the wrapper's contract code, and says which step failed.
    let (reader, writer) = std::io::pipe()?;
    drop(reader);
    let closed = producer(run, "tools.list", &request, Stdio::from(writer))?;
    assert_eq!(
        (
            closed.status.code(),
            String::from_utf8_lossy(&closed.stderr).as_ref()
        ),
        (
            Some(6),
            "habitat-engine: the reply could not be written: Broken pipe (os error 32)\n"
        )
    );
    Ok(())
}

#[test]
fn without_a_grant_directory_the_engine_serves_and_refuses_every_request() -> Outcome {
    let world = World::seeing(&["app"])?;
    let (run, scope) = (&world.run, &world.scope);
    let grants = world
        .home
        .join(".config/herdr-engineering-engine-v3/grants");
    fs::remove_dir_all(&grants)?;
    let log = world.home.join("engine.log");
    let _engine = Engine::start_logged(run, &world.home, &log)?;
    for argv in [
        vec!["health"],
        vec![
            "tools.list",
            "query:=null",
            r#"page:={"limit":1,"cursor":null}"#,
        ],
    ] {
        let refused = error_of(&wrapper(run, scope, &argv)?)?;
        assert_eq!(
            (
                &refused["code"],
                &refused["effect"],
                &refused["details"]["field"]
            ),
            (
                &json!("forbidden"),
                &json!("none"),
                &json!("/authority/grant_id")
            ),
            "{argv:?}: {refused}"
        );
    }
    let first = fs::read_to_string(&log)?
        .lines()
        .next()
        .map(str::to_owned)
        .ok_or("no line")?;
    assert_eq!(
        first,
        format!(
            "habitat-engine: no grant directory at {}; every request is refused forbidden",
            grants.display()
        )
    );
    Ok(())
}

#[test]
fn an_unwritable_ledger_leaves_task_actions_unavailable() -> Outcome {
    let world = World::granting(&["app", "task"], &["read", "durable admission"])?;
    let (run, scope) = (&world.run, &world.scope);
    let file = commission(&world.home)?;
    fs::set_permissions(&file, fs::Permissions::from_mode(0o400))?;
    let log = world.home.join("engine.log");
    let _engine = Engine::start_logged(run, &world.home, &log)?;
    let health = reply_of(&wrapper(run, scope, &["health"])?)?;
    let refused = error_of(&wrapper(
        run,
        scope,
        &strs(&submitting(
            "28c00000-0000-4000-8000-0000000000b3",
            "Add a strict decimal parser.",
        )),
    )?)?;
    let log = fs::read_to_string(&log)?;
    assert_eq!(
        (
            &health["body"]["ready"],
            &health["body"]["database"],
            &refused["code"],
            &refused["retry"]
        ),
        (
            &json!(false),
            &json!("unavailable"),
            &json!("unavailable"),
            &json!("after_condition")
        ),
        "{health}\n{refused}\n{log}"
    );
    // The operator reads why, in the engine's first two lines.
    assert_eq!(
        log.lines().take(2).collect::<Vec<_>>(),
        [
            "habitat-engine: startup refused: Store(NotWritable)",
            "habitat-engine: task actions unavailable: no generation was reconciled",
        ]
    );
    Ok(())
}

/// A skill package that needs `task.submit` and `task.get`, with one reviewed reference whose hash
/// is computed here.
fn skill_package(directory: &Path) -> Outcome {
    let references = directory.join("references");
    DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(&references)?;
    let notes = b"Read the task back by the key it was submitted under.\n";
    fs::write(references.join("notes.md"), notes)?;
    let sha = super::tasks::digest(Sha256::digest(notes));
    let manifest = json!({
        "schema": "hee3.skills.skill.v1", "skill_id": "submit-and-read-back", "skill_version": 1,
        "lifecycle": "published", "purpose": "Admit a task and read it back by its own key.",
        "supersedes": [], "requires_actions": ["task.submit", "task.get"], "requires_skills": [],
        "entry": "Submit under the step's derived key; read back by that key before any retry.",
        "references": [{"reference_id": "notes", "path": "references/notes.md",
                        "scope": ["operator"], "depth": 1,
                        "sha256": sha.strip_prefix("sha256:").ok_or("digest prefix")?}],
    });
    fs::write(directory.join("skill.json"), serde_json::to_vec(&manifest)?)?;
    Ok(())
}

/// The composition's procedure: submit, then read back; `probe` adds a root `health` step.
fn procedure_file(path: &Path, probe: bool) -> Outcome {
    let step = |id: &str, action: &str, depends_on: &[&str]| {
        json!({"step_id": id, "action": action, "action_version": 1, "depends_on": depends_on,
               "required": true, "retry": {"max_attempts": 1, "retry_on": []}})
    };
    let mut steps = vec![
        step("submit", "task.submit", &[]),
        step("verify", "task.get", &["submit"]),
    ];
    if probe {
        steps.push(step("probe", "health", &[]));
    }
    let procedure = json!({
        "schema": "hee3.workflows.procedure.v1", "procedure_id": "submit-and-read-back",
        "procedure_version": 1, "purpose": "Admit a task, then read it back by its own key.",
        "budget": {"max_fanout": 2, "max_seconds": 60, "max_tokens": 1000}, "steps": steps,
        "acceptance": [{"criterion_id": "read-back", "statement": "task.get finds the admission."}],
    });
    fs::write(path, serde_json::to_vec(&procedure)?)?;
    Ok(())
}

/// A step's idempotency key, derived here from the same canonical tuple the workflow package
/// hashes, with `sha2` rather than Python's `hashlib`: two implementations that must agree on the
/// exact key the ledger holds.
fn step_key(procedure: &str, version: u64, step: &str) -> Result<String, Box<dyn Error>> {
    let material = serde_json::to_string(&json!([
        "hee3.workflows.step-key/1",
        procedure,
        version,
        step,
        null
    ]))?;
    let mut raw: Vec<u8> = Sha256::digest(material.as_bytes())[..16].to_vec();
    raw[6] = (raw[6] & 0x0F) | 0x40;
    raw[8] = (raw[8] & 0x3F) | 0x80;
    let digest = super::tasks::digest(&raw);
    let hex = digest.strip_prefix("sha256:").ok_or("digest prefix")?;
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    ))
}

/// Run the T29 composition consumer against the engine in `world`.
fn compose(
    world: &World,
    skill: &Path,
    procedure: &Path,
    held: &str,
    injected: &[&str],
) -> Result<Output, Box<dyn Error>> {
    let mut spec = super::tasks::spec();
    spec["intent"] = json!("keep spaces, 'single', \"double\", $(touch x); and `ticks` as data");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(Command::new("python3")
        .args(["-W", "error"])
        .arg(root.join("tests/fixtures/t29/compose.py"))
        .arg(skill)
        .arg(procedure)
        .arg(held)
        .arg(spec.to_string())
        .args(injected)
        .env("XDG_RUNTIME_DIR", &world.run)
        .env("HEE3_PRODUCER", env!("CARGO_BIN_EXE_habitat-engine"))
        .env("HEE3_GRANT_ID", GRANT)
        .env("HEE3_SCOPE_SHA256", &world.scope)
        .env_remove("FORCE_COLOR")
        .env_remove("HEE3_CATALOGUE")
        .output()?)
}

#[test]
fn a_skill_bounded_procedure_runs_through_the_engine_and_closes_its_join() -> Outcome {
    let world = World::granting(&["task"], &["read", "durable admission"])?;
    commission(&world.home)?;
    let _engine = Engine::start(&world.run, &world.home)?;
    let scratch = Scratch::new()?;
    let skill = scratch.private("skill")?;
    skill_package(&skill)?;
    let procedure = scratch.0.join("procedure.json");
    procedure_file(&procedure, false)?;

    let composed = compose(&world, &skill, &procedure, "task.submit,task.get", &[])?;
    let document: Value = reply_of(&composed)?;
    let submit_key = step_key("submit-and-read-back", 1, "submit")?;
    let task = document["task_ids"]["submit"].clone();
    UuidV4::parse(task.as_str().ok_or("task id")?)?;
    assert_eq!(
        document,
        json!({
            "skill": {"skill_id": "submit-and-read-back", "skill_version": 1,
                      "actions_in_effect": ["task.get", "task.submit"], "complete": true,
                      "omissions": []},
            "order": ["submit", "verify"],
            "record": {"procedure_id": "submit-and-read-back", "procedure_version": 1,
                       "steps": {"submit": "done", "verify": "done"}},
            "verified": true, "disposition": "completion_candidate", "reasons": [],
            "keys": {"submit": submit_key, "verify": step_key("submit-and-read-back", 1, "verify")?},
            "calls": [["submit", "task.submit"], ["verify", "task.get"]], "licences": {},
            "task_ids": {"submit": task, "verify": task},
        })
    );
    // The ledger, read back under this test's own derivation of the key.
    let found = get_through(&world.run, &world.scope, &key_selector(&submit_key))?;
    assert_eq!(found["body"]["task"]["task_id"], task);
    Ok(())
}

/// WF-14, through the engine: a step whose reply never reached the composition is settled by a
/// readback under its own key before anything moves on, and the effect is requested once.
///
/// The honest limit: the loss is the consumer's, not the wire's. `--lose` sends the request and,
/// after the engine has committed and answered, DISCARDS the reply; `--unsent` records the step
/// as sent without sending it. Both leave the composition exactly where a lost reply or a lost
/// request would, which is what reconciliation has to decide between, but no socket is cut.
#[test]
fn a_lost_reply_is_reconciled_by_key_and_the_effect_is_requested_once() -> Outcome {
    let world = World::granting(&["task"], &["read", "durable admission"])?;
    commission(&world.home)?;
    let _engine = Engine::start(&world.run, &world.home)?;
    let scratch = Scratch::new()?;
    let skill = scratch.private("skill")?;
    skill_package(&skill)?;
    let procedure = scratch.0.join("procedure.json");
    procedure_file(&procedure, false)?;
    let submit_key = step_key("submit-and-read-back", 1, "submit")?;
    let stamps = [
        "/request_id",
        "/request_sha256",
        "/body/cursor/issued_unix_ms",
        "/body/cursor/expires_unix_ms",
    ];

    // The reply is lost after the commit: the readback finds the admission, and it settles.
    let lost = reply_of(&compose(
        &world,
        &skill,
        &procedure,
        "task.submit,task.get",
        &["--lose", "submit"],
    )?)?;
    let task = lost["task_ids"]["submit"].clone();
    assert_eq!(
        (
            &lost["record"]["steps"],
            &lost["disposition"],
            &lost["calls"]
        ),
        (
            &json!({"submit": "done", "verify": "done"}),
            &json!("completion_candidate"),
            &json!([
                ["submit", "task.submit"],
                ["submit", "task.get"],
                ["verify", "task.get"]
            ])
        ),
        "{lost}"
    );
    // The reply that licensed `done`, whole but for what the receiver stamps: the ledger's own
    // answer to the key the composition held before it sent anything.
    let licence = &lost["licences"]["submit"];
    let found = get_through(&world.run, &world.scope, &key_selector(&submit_key))?;
    assert_eq!(
        without(licence.clone(), &stamps)?,
        without(found.clone(), &stamps)?
    );
    assert_eq!(
        (&licence["kind"], &licence["body"]["task"]["task_id"]),
        (&json!("result"), &task)
    );

    // The request is lost before it lands: the readback finds nothing, which releases the step
    // to run again under the same key; `calls` shows the submit sent once. A fresh ledger, so the
    // first run's admission under this same key cannot answer.
    let fresh = World::granting(&["task"], &["read", "durable admission"])?;
    commission(&fresh.home)?;
    let _second = Engine::start(&fresh.run, &fresh.home)?;
    let unsent = reply_of(&compose(
        &fresh,
        &skill,
        &procedure,
        "task.submit,task.get",
        &["--unsent", "submit"],
    )?)?;
    assert_eq!(
        (
            &unsent["record"]["steps"],
            &unsent["disposition"],
            &unsent["calls"]
        ),
        (
            &json!({"submit": "done", "verify": "done"}),
            &json!("completion_candidate"),
            &json!([
                ["submit", "task.get"],
                ["submit", "task.submit"],
                ["verify", "task.get"]
            ])
        ),
        "{unsent}"
    );
    let released = &unsent["licences"]["submit"];
    assert_eq!(
        (&released["kind"], &released["code"], &released["effect"]),
        (&json!("error"), &json!("not_found"), &json!("none")),
        "{released}"
    );
    Ok(())
}

#[test]
fn a_composition_outside_its_skills_actions_is_refused_before_any_request() -> Outcome {
    let world = World::granting(&["task"], &["read", "durable admission"])?;
    commission(&world.home)?;
    let _engine = Engine::start(&world.run, &world.home)?;
    let scratch = Scratch::new()?;
    let skill = scratch.private("skill")?;
    skill_package(&skill)?;
    let plain = scratch.0.join("plain.json");
    procedure_file(&plain, false)?;
    let probing = scratch.0.join("probing.json");
    procedure_file(&probing, true)?;
    for (case, procedure, held, refusal) in [
        // The caller does not hold what the skill needs: refused at load.
        (
            "skill",
            &plain,
            "task.get",
            json!({"refused": "authority_widening",
                   "detail": "submit-and-read-back requires 'task.submit', which the caller does not hold"}),
        ),
        // The caller holds `health`, the skill does not need it: the packet narrows the procedure,
        // and the probe (first in order) is refused before the submit is ever sent.
        (
            "procedure",
            &probing,
            "task.submit,task.get,health",
            json!({"refused": "authority_widening",
                   "detail": "probe: 'health' is not among the actions the caller holds"}),
        ),
    ] {
        let composed = compose(&world, &skill, procedure, held, &[])?;
        assert_eq!(composed.status.code(), Some(1), "{case}");
        assert_eq!(
            serde_json::from_slice::<Value>(&composed.stdout)?,
            refusal,
            "{case}"
        );
    }
    // Nothing reached the ledger.
    let absent = error_of(&wrapper(
        &world.run,
        &world.scope,
        &strs(&getting(&key_selector(&step_key(
            "submit-and-read-back",
            1,
            "submit",
        )?))),
    )?)?;
    assert_eq!(absent["code"], json!("not_found"), "{absent}");
    Ok(())
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
    commission(home)?;
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

/// `prepare` once no process still holds a copy of a dropped custody. A sibling case forking at
/// the instant this process held the lock keeps a duplicate of its descriptor until the child's
/// exec closes it (CLOEXEC), and `prepare` says Live meanwhile, which is the safe direction. So
/// retry on Live alone, with a budget.
fn prepare_settled(root: &Path) -> Result<control_socket::Prepared, SocketError> {
    let started = Instant::now();
    let budget = Duration::from_secs(5);
    loop {
        match control_socket::prepare(root) {
            Err(SocketError::Live) if started.elapsed() < budget => {
                std::thread::sleep(Duration::from_millis(10));
            }
            other => return other,
        }
    }
}

/// Prepare and bind as `serve` does, keeping custody with the listener.
fn start_at(
    root: &Path,
) -> Result<(control_socket::Prepared, std::os::unix::net::UnixListener), SocketError> {
    control_socket::prepare(root)
        .and_then(|prepared| control_socket::bind(&prepared).map(|listener| (prepared, listener)))
}

/// A socket file nothing listens on and nothing ever listened on: bound, never `listen`ed, closed.
/// A forked sibling that inherits the descriptor cannot make it accept.
fn stale_socket(path: &Path) -> Outcome {
    let fd = rustix::net::socket_with(
        rustix::net::AddressFamily::UNIX,
        rustix::net::SocketType::STREAM,
        rustix::net::SocketFlags::CLOEXEC,
        None,
    )?;
    rustix::net::bind(&fd, &rustix::net::SocketAddrUnix::new(path)?)?;
    Ok(())
}

#[test]
fn custody_is_held_from_prepare_so_a_second_start_is_refused_before_any_bind() -> Outcome {
    let scratch = Scratch::new()?;
    let root = scratch.private("run")?;
    let first = control_socket::prepare(&root)?;
    let socket = root.join(RUNTIME_DIRECTORY).join(SOCKET_NAME);
    assert!(
        fs::symlink_metadata(&socket).is_err(),
        "nothing is bound yet"
    );
    // A stale socket appears while the first start has not bound: the second start must not
    // judge it, let alone unlink it.
    stale_socket(&socket)?;
    assert!(matches!(
        control_socket::prepare(&root),
        Err(SocketError::Live)
    ));
    assert!(
        fs::symlink_metadata(&socket).is_ok(),
        "the refused start unlinked nothing"
    );
    drop(first);
    // Custody ends with the prepared value: the next start clears the stale socket.
    let again = prepare_settled(&root)?;
    assert!(
        fs::symlink_metadata(&socket).is_err(),
        "the stale socket was removed"
    );
    drop(again);
    // The lock is custody only when it is this user's private regular file, reached directly.
    let lock = root.join(RUNTIME_DIRECTORY).join(control_socket::LOCK_NAME);
    assert_eq!(
        fs::symlink_metadata(&lock)?.permissions().mode() & 0o777,
        0o600
    );
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o644))?;
    assert!(matches!(
        control_socket::prepare(&root),
        Err(SocketError::Custody("control lock"))
    ));
    fs::remove_file(&lock)?;
    let elsewhere = scratch.0.join("elsewhere.lock");
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&elsewhere)?;
    symlink(&elsewhere, &lock)?;
    assert!(matches!(
        control_socket::prepare(&root),
        Err(SocketError::Custody("control lock"))
    ));
    // A private FIFO opens read-write and takes a lock; it is still not the lock file.
    fs::remove_file(&lock)?;
    rustix::fs::mknodat(
        rustix::fs::CWD,
        &lock,
        rustix::fs::FileType::Fifo,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        0,
    )?;
    assert!(matches!(
        control_socket::prepare(&root),
        Err(SocketError::Custody("control lock"))
    ));
    Ok(())
}

#[test]
fn racing_starts_over_a_stale_socket_leave_exactly_one_live_engine() -> Outcome {
    const ROUNDS: usize = 16;
    const ATTEMPTS: usize = 64;
    let scratch = Scratch::new()?;
    let root = scratch.private("run")?;
    let socket = root.join(RUNTIME_DIRECTORY).join(SOCKET_NAME);
    let mut won = 0;
    for attempt in 0..ATTEMPTS {
        if won == ROUNDS {
            break;
        }
        drop(prepare_settled(&root)?);
        stale_socket(&socket)?;
        let barrier = std::sync::Barrier::new(2);
        let (a, b) = std::thread::scope(|scope| {
            let race = || {
                barrier.wait();
                start_at(&root)
            };
            let a = scope.spawn(race);
            let b = scope.spawn(race);
            (a.join(), b.join())
        });
        let (a, b) = (
            a.map_err(|_| "a racer panicked")?,
            b.map_err(|_| "a racer panicked")?,
        );
        match (a, b) {
            (Ok(winner), Err(loser)) | (Err(loser), Ok(winner)) => {
                assert!(
                    matches!(loser, SocketError::Live),
                    "attempt {attempt}: {loser:?}"
                );
                // The winner's socket is still the one at the path: it accepts.
                UnixStream::connect(&socket)?;
                drop(winner);
                won += 1;
            }
            (Ok(_), Ok(_)) => return Err(format!("attempt {attempt}: both starts bound").into()),
            (Err(a), Err(b)) => {
                // A forked sibling still held a copy of the last custody: both refused, which is
                // the safe direction, and neither may have unlinked anything.
                assert!(
                    matches!((&a, &b), (SocketError::Live, SocketError::Live)),
                    "attempt {attempt}: {a:?} / {b:?}"
                );
                assert!(
                    fs::symlink_metadata(&socket).is_ok(),
                    "attempt {attempt}: a refused start unlinked the stale socket"
                );
            }
        }
        fs::remove_file(&socket)?;
    }
    assert_eq!(
        won, ROUNDS,
        "only {won} of {ROUNDS} rounds had a winner in {ATTEMPTS} attempts"
    );
    Ok(())
}

#[test]
fn a_start_that_cannot_write_is_not_yet_bound() -> Outcome {
    // `serve` writes its first line to standard error at a known point. Give it a standard error
    // that is already full: it blocks there, and the case reads the engine's state at that point.
    // With grants, the first line is the reconciliation's: the engine has reconciled, holds
    // custody (IPC01: custody before recovery) and has not bound (IPC01: bind after ready).
    // Without grants, the first line says so before the manifest is read: custody is already held.
    for grants in [true, false] {
        let world = World::seeing(&["app"])?;
        if !grants {
            fs::remove_dir_all(
                world
                    .home
                    .join(".config/herdr-engineering-engine-v3/grants"),
            )?;
        }
        let (engine, filled) = blocked_on_stderr(&world)?;
        assert!(
            matches!(control_socket::prepare(&world.run), Err(SocketError::Live)),
            "grants={grants}: the engine blocked at its first line without custody"
        );
        let socket = world.run.join(RUNTIME_DIRECTORY).join(SOCKET_NAME);
        assert!(
            fs::symlink_metadata(&socket).is_err(),
            "grants={grants}: the socket was bound before the engine finished starting \
             (pipe held {filled} bytes)"
        );
        drop(engine);
    }
    Ok(())
}

/// Start `serve` in `world` with a full standard error and return once the kernel says it sleeps
/// writing to it, with the bytes the pipe held.
fn blocked_on_stderr(world: &World) -> Result<(Blocked, usize), Box<dyn Error>> {
    let (drain, writer) = std::io::pipe()?;
    let flags = rustix::fs::fcntl_getfl(&writer)?;
    rustix::fs::fcntl_setfl(&writer, flags | rustix::fs::OFlags::NONBLOCK)?;
    let mut filled = 0_usize;
    for chunk in [4096_usize, 1] {
        // A pipe holds at most 1 MiB (fs.pipe-max-size's default): 1 << 20 writes is the budget.
        let mut attempts = 0_usize;
        loop {
            assert!(
                attempts < 1 << 20,
                "the pipe never filled after {attempts} writes"
            );
            attempts += 1;
            match (&writer).write(&vec![b'x'; chunk]) {
                Ok(written) => filled += written,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error.into()),
            }
        }
    }
    rustix::fs::fcntl_setfl(&writer, flags)?;
    let child = Command::new(env!("CARGO_BIN_EXE_habitat-engine"))
        .arg("serve")
        .env("XDG_RUNTIME_DIR", &world.run)
        .env("HOME", &world.home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(writer)
        .spawn()?;
    let blocked = Blocked {
        engine: Engine { child },
        _drain: drain,
    };
    let wchan = PathBuf::from(format!("/proc/{}/wchan", blocked.engine.child.id()));
    let started = Instant::now();
    let budget = Duration::from_secs(20);
    // Wait on the kernel's account of where the engine sleeps, with a budget.
    while !fs::read_to_string(&wchan)?.contains("pipe_write") {
        assert!(
            started.elapsed() < budget,
            "the engine never blocked on its full standard error within {budget:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok((blocked, filled))
}

/// An engine blocked on its standard error; the engine is killed before the pipe's read end
/// closes (fields drop in declaration order).
struct Blocked {
    engine: Engine,
    _drain: std::io::PipeReader,
}

/// How long a case waits on one reply from a live engine before calling it a hang.
const REPLY_BUDGET: Duration = Duration::from_secs(20);

/// A raw peer of a serving engine: sends exact frames and reads replies one record at a time.
struct Peer {
    stream: UnixStream,
    reader: FrameReader<UnixStream>,
}

impl Peer {
    fn connect(run: &Path) -> Result<Self, Box<dyn Error>> {
        let stream = UnixStream::connect(run.join(RUNTIME_DIRECTORY).join(SOCKET_NAME))?;
        stream.set_read_timeout(Some(REPLY_BUDGET))?;
        stream.set_write_timeout(Some(REPLY_BUDGET))?;
        let reader = FrameReader::new(stream.try_clone()?);
        Ok(Self { stream, reader })
    }

    fn send(&mut self, frame: &[u8]) -> Outcome {
        self.stream.write_all(frame)?;
        Ok(())
    }

    /// The next reply, or `None` when the engine closed the connection.
    fn reply(&mut self) -> Result<Option<Value>, Box<dyn Error>> {
        match self.reader.next_frame() {
            Ok(Some(record)) => Ok(Some(serde_json::from_slice(&record)?)),
            Ok(None) => Ok(None),
            Err(ReadError::Fault(fault)) => Err(fault.name().into()),
            Err(ReadError::Io(error)) => {
                Err(format!("no reply within {REPLY_BUDGET:?}: {error}").into())
            }
        }
    }

    fn ask(&mut self, frame: &[u8]) -> Result<Value, Box<dyn Error>> {
        self.send(frame)?;
        self.reply()?
            .ok_or_else(|| "the engine closed the connection".into())
    }
}

/// One `tools.list` request frame the wrapper builds, LF-terminated, and its `request_id`.
fn listing_frame(world: &World) -> Result<(Vec<u8>, Value), Box<dyn Error>> {
    let mut frame = built(
        &world.run,
        &world.scope,
        &[
            "tools.list",
            "query:=null",
            r#"page:={"limit":1,"cursor":null}"#,
        ],
    )?;
    if frame.last() != Some(&b'\n') {
        frame.push(b'\n');
    }
    let id = serde_json::from_slice::<Value>(&frame)?["request_id"].clone();
    Ok((frame, id))
}

/// The whole `resource_exhausted` record the engine answers `request` with, written from the
/// `ControlErrorV1` schema rather than from the engine's code.
fn exhausted(request: &[u8], constraint: &str) -> Result<Value, Box<dyn Error>> {
    let payload = request.strip_suffix(b"\n").unwrap_or(request);
    let id = serde_json::from_slice::<Value>(payload)?["request_id"].clone();
    Ok(json!({
        "protocol": "hee3.control", "version": 1, "kind": "error",
        "request_id": id, "request_sha256": request_sha256(payload),
        "code": "resource_exhausted", "effect": "none", "retry": "same_exact_request",
        "readback": null,
        "message": "capacity is exhausted; nothing was done",
        "details": {"field": null, "constraint": constraint, "current_generation": null},
    }))
}

#[test]
fn a_held_connection_does_not_keep_another_peer_waiting() -> Outcome {
    let world = World::new()?;
    let _engine = Engine::start(&world.run, &world.home)?;
    let (frame, id) = listing_frame(&world)?;
    // The first peer is admitted, answered, and then holds its connection open.
    let mut held = Peer::connect(&world.run)?;
    assert_eq!(held.ask(&frame)?["request_id"], id);
    // A second peer is answered while the first is still connected.
    let mut second = Peer::connect(&world.run)?;
    let reply = second.ask(&frame)?;
    assert_eq!(
        (&reply["kind"], &reply["request_id"]),
        (&json!("result"), &id)
    );
    // And the first is still served.
    assert_eq!(held.ask(&frame)?["kind"], json!("result"));
    Ok(())
}

#[test]
fn a_ninth_connection_is_refused_whole_and_the_eight_admitted_are_untouched() -> Outcome {
    let world = World::new()?;
    let _engine = Engine::start(&world.run, &world.home)?;
    let (frame, id) = listing_frame(&world)?;
    let mut admitted = Vec::new();
    for _ in 0..8 {
        let mut peer = Peer::connect(&world.run)?;
        let reply = peer.ask(&frame)?;
        assert_eq!(
            (&reply["kind"], &reply["request_id"]),
            (&json!("result"), &id)
        );
        admitted.push(peer);
    }
    let mut ninth = Peer::connect(&world.run)?;
    assert_eq!(
        ninth.ask(&frame)?,
        exhausted(&frame, "at most 8 simultaneous connections")?
    );
    assert!(ninth.reply()?.is_none(), "the refused connection is closed");
    for peer in &mut admitted {
        assert_eq!(
            peer.ask(&frame)?["kind"],
            json!("result"),
            "an admitted peer is served"
        );
    }
    // A closed peer frees its place: a new connection is admitted once the engine has seen the
    // close, which it learns asynchronously, so wait for it with a budget.
    drop(admitted.pop());
    let started = Instant::now();
    loop {
        let reply = Peer::connect(&world.run)?.ask(&frame)?;
        if reply["kind"] == json!("result") {
            break;
        }
        assert_eq!(reply["code"], json!("resource_exhausted"), "{reply}");
        assert!(
            started.elapsed() < REPLY_BUDGET,
            "the closed peer's place was not freed within {REPLY_BUDGET:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

#[test]
fn a_principal_past_its_burst_is_refused_before_any_effect_at_the_contract_rate() -> Outcome {
    // Two connections of one principal, each pipelining this many frames at once.
    const SENT: usize = 100;
    let world = World::new()?;
    let _engine = Engine::start(&world.run, &world.home)?;
    let (frame, _) = listing_frame(&world)?;
    let refused = exhausted(&frame, "100 requests/second, burst 32")?;
    let mut peers = [Peer::connect(&world.run)?, Peer::connect(&world.run)?];
    let started = Instant::now();
    let mut senders = Vec::new();
    for peer in &peers {
        let mut writer = peer.stream.try_clone()?;
        let pipelined = frame.repeat(SENT);
        senders.push(std::thread::spawn(move || writer.write_all(&pipelined)));
    }
    let mut served = 0_usize;
    for peer in &mut peers {
        for _ in 0..SENT {
            let reply = peer.reply()?.ok_or("closed before every reply")?;
            if reply["kind"] == json!("result") {
                served += 1;
            } else {
                assert_eq!(reply, refused);
            }
        }
    }
    let elapsed_ms = started.elapsed().as_millis();
    for sender in senders {
        sender.join().map_err(|_| "a sender panicked")??;
    }
    // The principal's one burst plus what 100 per second refills while the replies arrived:
    // shared by both connections, not one burst each.
    let bound = 32 + usize::try_from(elapsed_ms / 10)? + 1;
    assert!(
        served >= 32 && served <= bound,
        "served={served} bound={bound} elapsed_ms={elapsed_ms}"
    );
    assert!(
        bound < 64,
        "elapsed_ms={elapsed_ms}: too slow to tell one burst from two"
    );
    Ok(())
}

#[test]
fn the_admission_bounds_are_the_contracts_own_figures() -> Outcome {
    use control_socket::{
        AGGREGATE_PENDING, BURST, CONNECTION_CAP, CONNECTION_RULE, RATE_PER_SECOND, RATE_RULE,
    };
    let contract = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/contract-decisions.md"),
    )?;
    let line = contract
        .lines()
        .find(|line| line.starts_with("IPC01 admits"))
        .ok_or("the contract's connection-bound line")?;
    for clause in [
        format!("IPC01 admits at most{CONNECTION_CAP} simultaneous connections total"),
        format!(
            "capped at{RATE_PER_SECOND} requests/second with burst{BURST} and an \
             aggregate{AGGREGATE_PENDING} pending control requests"
        ),
    ] {
        assert!(line.contains(&clause), "{clause:?} is not in {line:?}");
    }
    assert_eq!(
        (CONNECTION_RULE, RATE_RULE),
        (
            "at most 8 simultaneous connections",
            "100 requests/second, burst 32"
        )
    );
    Ok(())
}

#[test]
fn a_principals_burst_refills_at_one_token_per_ten_milliseconds_on_the_given_clock() -> Outcome {
    let admission = control_socket::Admission::new();
    let operator = operator()?;
    let t0 = NOW;
    for spent in 0..32 {
        assert!(admission.admit(&operator, t0), "token {spent} of the burst");
    }
    assert!(
        !admission.admit(&operator, t0),
        "the 33rd at the same instant"
    );
    assert!(
        !admission.admit(&operator, t0 + 9),
        "nine tenths of a token"
    );
    assert!(admission.admit(&operator, t0 + 10), "one whole token");
    assert!(!admission.admit(&operator, t0 + 10), "and only one");
    // A clock that steps back refills nothing, and the ground it lost is not earned again.
    assert!(!admission.admit(&operator, t0));
    assert!(
        !admission.admit(&operator, t0 + 10),
        "t0 + 10 was already judged"
    );
    assert!(admission.admit(&operator, t0 + 20));
    assert!(!admission.admit(&operator, t0 + 20));
    // A long silence refills the burst and no more.
    let later = t0 + 10_000;
    let admitted = (0..100)
        .filter(|_| admission.admit(&operator, later))
        .count();
    assert_eq!(admitted, 32);
    // A clock at the end of time neither overflows nor refills past the burst.
    let end = (0..40)
        .filter(|_| admission.admit(&operator, u64::MAX))
        .count();
    assert_eq!(end, 32);
    Ok(())
}

#[test]
fn each_principal_has_its_own_bucket_and_the_table_holds_eight_in_use() -> Outcome {
    let admission = control_socket::Admission::new();
    let principals = (0..9)
        .map(|uid| Principal::new(50_000 + uid, OPERATOR_ROLE))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{error:?}"))?;
    let t0 = NOW;
    // One principal's exhaustion leaves another's burst whole.
    let shared = control_socket::Admission::new();
    for _ in 0..32 {
        assert!(shared.admit(&principals[0], t0));
    }
    assert!(!shared.admit(&principals[0], t0));
    assert_eq!(
        (0..40).filter(|_| shared.admit(&principals[1], t0)).count(),
        32
    );
    // Eight principals with a spent token fill the table; a ninth is refused while they are.
    for principal in &principals[..8] {
        assert!(admission.admit(principal, t0));
    }
    assert!(
        !admission.admit(&principals[8], t0),
        "a ninth principal while eight are in use"
    );
    assert!(
        !admission.admit(&principals[8], t0 + 9),
        "every bucket is a tenth of a token short of full"
    );
    // Once every bucket has refilled, a full bucket is an absent one, and the ninth is admitted.
    assert!(admission.admit(&principals[8], t0 + 10));
    Ok(())
}

/// The replies a `serve_connection` wrote, as records.
fn records(written: &[u8]) -> Result<Vec<Value>, Box<dyn Error>> {
    Ok(written
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .map(serde_json::from_slice)
        .collect::<Result<_, _>>()?)
}

#[test]
fn a_frame_past_the_burst_is_answered_resource_exhausted_and_never_dispatched() -> Outcome {
    let admission = control_socket::Admission::new();
    let composed = Composed {
        grants: &Open,
        health: None,
        tasks: None,
    };
    let frames = |ids: std::ops::Range<u8>| {
        ids.flat_map(|id| {
            let mut frame = listing(id);
            frame.push(b'\n');
            frame
        })
        .collect::<Vec<u8>>()
    };
    // Two connections of one principal share one burst: 20 then 20 frames at one instant.
    let mut first = Vec::new();
    let ended = serve_connection(
        frames(0..20).as_slice(),
        &mut first,
        &operator()?,
        composed,
        &admission,
        &|| NOW,
    )?;
    assert_eq!(ended, Ended::Clean { served: 20 });
    assert!(
        records(&first)?
            .iter()
            .all(|reply| reply["kind"] == json!("result"))
    );
    let mut second = Vec::new();
    let ended = serve_connection(
        frames(20..40).as_slice(),
        &mut second,
        &operator()?,
        composed,
        &admission,
        &|| NOW,
    )?;
    assert_eq!(
        ended,
        Ended::Clean { served: 20 },
        "a refusal does not close the connection"
    );
    let replies = records(&second)?;
    assert!(
        replies[..12]
            .iter()
            .all(|reply| reply["kind"] == json!("result"))
    );
    for (id, reply) in (32..40).zip(&replies[12..]) {
        assert_eq!(
            reply,
            &exhausted(&listing(id), "100 requests/second, burst 32")?
        );
    }
    // The clock the caller passes is the one judged: ten milliseconds later, one more is served.
    let mut third = Vec::new();
    serve_connection(
        frames(40..42).as_slice(),
        &mut third,
        &operator()?,
        composed,
        &admission,
        &|| NOW + 10,
    )?;
    let kinds: Vec<Value> = records(&third)?
        .iter()
        .map(|reply| reply["kind"].clone())
        .collect();
    assert_eq!(kinds, [json!("result"), json!("error")]);
    // A clock that advances a token's worth per frame refills as fast as a connection spends.
    let clock = std::cell::Cell::new(NOW);
    let advancing = || {
        clock.set(clock.get() + 10);
        clock.get()
    };
    let mut fourth = Vec::new();
    serve_connection(
        frames(0..40).as_slice(),
        &mut fourth,
        &operator()?,
        composed,
        &control_socket::Admission::new(),
        &advancing,
    )?;
    assert!(
        records(&fourth)?
            .iter()
            .all(|reply| reply["kind"] == json!("result"))
    );
    assert_eq!(records(&fourth)?.len(), 40);
    Ok(())
}
