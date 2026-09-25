//! B07 · `task.preview` (task-G06): validation plus route eligibility with no write.
//!
//! The owner composes route's policy (`config/routes.toml`, one parse) with the caller's roster
//! snapshot: each declared recipe joined with the roster record it names, screened by route's own
//! filters, reported as `eligible` / `exclusions` with the design's codes (B07 DESIGN P7, R1-R3).
//! Known answers come from `tests/fixtures/preview/make-known-answers.py`, a second
//! implementation of the adapter in Python over the T09 route oracle's `screen` (F94/F113). The
//! roster is seeded through the store's own doors; ledger facts are read by SQL from the file.

use super::tasks::{
    GENERATION, KEY, NOW, Open, Scratch, conforms, raw_store, request, serve, serve_composed_at,
    spec,
};
use habitat_engine::actions::control as receiver;
use habitat_engine::app::routing::{self, MAX_ROUTE_CONFIG_BYTES, ROUTES_FILE, Unready};
use habitat_engine::app::tasks::StoreTasks;
use habitat_engine::contracts::roster::{
    ActiveAttemptPolicy, Availability, Disable, Kind, Locality, ObservationInput,
    ObservationSource, RosterDefinitionV1, Update,
};
use habitat_engine::store::{Principal, RequestSource, Store};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type Outcome = Result<(), Box<dyn Error>>;

const CONFIG: &str = include_str!("../config/routes.toml");
const FIXTURE: &str = include_str!("fixtures/preview/fixture.json");
const ANSWERS: &str = include_str!("fixtures/preview/known-answers.json");
/// The baseline identity the shipped configuration declares (no recipe stands behind it).
const DECLARED_BASELINE: &str = "09000000-0000-4000-8000-00000000000b";
const CLASS: &str = "rust-library-change/1";
/// The cursor epoch the task owner is composed with; preview issues no cursor.
const EPOCH: &str = "07b00000-0000-4000-8000-0000000000e0";
/// A roster record identity no seeded record has.
const GHOST: &str = "07b00000-0000-4000-8000-0000000000ef";
const FIGURES: [&str; 4] = [
    "context_limit_tokens",
    "cost_microunits",
    "quality_basis_points",
    "latency_ms",
];

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}

fn principal(uid: u32) -> Result<Principal, Box<dyn Error>> {
    Ok(Principal::new(uid, "operator").map_err(|error| format!("{error:?}"))?)
}

fn wall_ms() -> Result<u64, Box<dyn Error>> {
    Ok(u64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}

fn strings(value: &Value) -> Result<Vec<String>, Box<dyn Error>> {
    value
        .as_array()
        .ok_or("array")?
        .iter()
        .map(|item| Ok(item.as_str().ok_or("string")?.to_owned()))
        .collect()
}

/// A preview body over `spec`, at brief revision 0 (no parent under RC01) and catalogue 1.
fn body(spec: &Value) -> Value {
    json!({"spec": spec, "brief_revision": "0", "catalogue_revision": "1"})
}

fn preview(request_byte: u8, key: Option<&str>) -> Vec<u8> {
    request("task.preview", request_byte, key, &body(&spec()))
}

/// The seeded roster: the fixture's records, created, observed and disabled through the store's
/// own doors, by name.
struct World {
    scratch: Scratch,
    records: BTreeMap<String, String>,
    fixture: Value,
}

/// Seed one record: `roster_apply` (the operator's door), then `roster_observe_worker` for its
/// observation, then `roster_disable` when the fixture disables it. Returns its record id.
fn seed(
    store: &mut Store,
    owner: &Principal,
    row: &Value,
    serial: &mut u32,
) -> Result<String, Box<dyn Error>> {
    let mut next = || {
        *serial += 1;
        format!("07b00000-0000-4000-8000-{:012x}", 0x100 + *serial)
    };
    let update = Update {
        idempotency_key: next(),
        record_id: None,
        expected_revision: None,
        definition: RosterDefinitionV1 {
            kind: Kind::Agent,
            display_name: format!("preview {}", row["name"].as_str().ok_or("name")?),
            owner_id: "preview-fixture".into(),
            version: "v1".into(),
            capabilities: strings(&row["capabilities"])?,
            locality: match row["locality"].as_str() {
                Some("local") => Locality::Local,
                Some("remote") => Locality::Remote,
                _ => Locality::Hybrid,
            },
            endpoint_ref: None,
            limitations: "test-only preview fixture; no runtime grant".into(),
        },
        audit_reason: "B07 preview fixture".into(),
    };
    let raw = serde_json::to_vec(&json!({
        "protocol": "hee3.control", "version": 1, "kind": "request",
        "request_id": update.idempotency_key, "action": "roster.update", "action_version": 1,
        "idempotency_key": update.idempotency_key, "deadline_unix_ms": "1030000",
        "authority": {"grant_id": KEY, "scope_sha256": format!("sha256:{}", "4".repeat(64))},
        "precondition": null,
        "body": {"record_id": null, "definition": update.definition, "audit_reason": update.audit_reason},
    }))?;
    let head = store
        .roster_apply(owner, &[update], RequestSource::Native(&raw), deadline())
        .map_err(|error| format!("{error:?}"))?
        .remove(0)
        .head;
    if let Some(observed) = row["observation"].as_object() {
        store
            .roster_observe_worker(
                owner,
                &ObservationInput {
                    record_id: head.record_id.clone(),
                    record_version: head.record_version.clone(),
                    owner_id: head.definition.owner_id.clone(),
                    endpoint_ref: None,
                    instance_id: None,
                    instance_generation: None,
                    source: ObservationSource::Worker,
                    observed_unix_ms: None,
                    availability: match observed["availability"].as_str() {
                        Some("available") => Availability::Available,
                        Some("unavailable") => Availability::Unavailable,
                        _ => Availability::Unknown,
                    },
                    actual_identity: None,
                    immutable_revision: None,
                    capabilities: strings(&observed["capabilities"])?,
                    evidence_ref: next(),
                },
                deadline(),
            )
            .map_err(|error| format!("{error:?}"))?;
    }
    if row["disabled"] == json!(true) {
        let disable = Disable {
            idempotency_key: next(),
            record_id: head.record_id.clone(),
            expected_revision: head.record_version.clone(),
            active_attempt_policy: ActiveAttemptPolicy::LeaveRunning,
            audit_reason: "B07 preview fixture".into(),
        };
        store
            .roster_disable(owner, &disable, b"{\"b07\":\"disable\"}", deadline())
            .map_err(|error| format!("{error:?}"))?;
    }
    Ok(head.record_id)
}

/// The fixture's world, seeded into a fresh ledger, and that same store (its receiver epoch is
/// the one the observations were stamped in).
fn world() -> Result<(World, Store), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let mut store = raw_store(&scratch)?;
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    let (operator, stranger) = (principal(1000)?, principal(1001)?);
    let mut records = BTreeMap::new();
    let mut serial = 0;
    for row in fixture["records"].as_array().ok_or("records")? {
        let name = row["name"].as_str().ok_or("name")?;
        let owner = if name == "stranger" {
            &stranger
        } else {
            &operator
        };
        records.insert(name.to_owned(), seed(&mut store, owner, row, &mut serial)?);
    }
    Ok((
        World {
            scratch,
            records,
            fixture,
        },
        store,
    ))
}

/// The shipped configuration with the fixture's recipes declared (each naming its seeded record,
/// or `GHOST`) and the fixture's baseline in place of the shipped one.
fn configuration(world: &World) -> Result<String, Box<dyn Error>> {
    let rows = world.fixture["recipes"]
        .as_array()
        .ok_or("recipes")?
        .iter()
        .map(|recipe| {
            let named = recipe["record"].as_str().ok_or("record")?;
            let record = world.records.get(named).map_or(GHOST, String::as_str);
            let mut members = vec![
                format!("id = {}", recipe["id"]),
                format!("version = {}", recipe["version"]),
                format!("adapter = {}", recipe["adapter"]),
                format!(
                    "actual_model_required = {}",
                    recipe["actual_model_required"]
                ),
                format!("roster_record = \"{record}\""),
                format!("serves = [\"{CLASS}\"]"),
            ];
            for figure in FIGURES {
                if let Some(value) = recipe.get(figure).and_then(Value::as_u64) {
                    members.push(format!("{figure} = {value}"));
                }
            }
            Ok(format!("{{ {} }}", members.join(", ")))
        })
        .collect::<Result<Vec<String>, Box<dyn Error>>>()?;
    assert_eq!(CONFIG.matches("recipes = []").count(), 1);
    assert_eq!(CONFIG.matches(DECLARED_BASELINE).count(), 1);
    let declared = CONFIG.replacen(
        "recipes = []",
        &format!("recipes = [\n  {},\n]", rows.join(",\n  ")),
        1,
    );
    Ok(declared.replacen(
        DECLARED_BASELINE,
        world.fixture["baseline"].as_str().ok_or("baseline")?,
        1,
    ))
}

/// The owner over `store`, with the world's routing composed from its configuration bytes.
fn owner(world: &World, store: Store) -> Result<StoreTasks, Box<dyn Error>> {
    Ok(StoreTasks::new(store, EPOCH.to_owned())
        .with_routing(routing::compose(configuration(world)?.as_bytes())))
}

/// The ledger file's own counts: events high-water, operations, tasks, roster revisions.
fn ledger_counts(scratch: &Scratch) -> Result<[u64; 4], Box<dyn Error>> {
    let file = scratch
        .0
        .join("state/generations")
        .join(GENERATION)
        .join("ledger.sqlite3");
    let db =
        rusqlite::Connection::open_with_flags(file, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut counts = [0; 4];
    for (slot, sql) in counts.iter_mut().zip([
        "SELECT coalesce(max(sequence),0) FROM events",
        "SELECT count(*) FROM operations",
        "SELECT count(*) FROM tasks",
        "SELECT count(*) FROM roster_revisions",
    ]) {
        *slot = u64::try_from(db.query_row(sql, [], |row| row.get::<_, i64>(0))?)?;
    }
    Ok(counts)
}

/// The expected body for `answers` with the reply's own cutoff, which must be the receiver's wall
/// time at the read: inside `[before, after]`.
fn expected(
    reply: &Value,
    answers: &Value,
    before: u64,
    after: u64,
) -> Result<Value, Box<dyn Error>> {
    let cutoff = reply["body"]["observations_cutoff_unix_ms"]
        .as_str()
        .ok_or("cutoff")?;
    let cutoff: u64 = cutoff.parse()?;
    assert!(
        (before..=after).contains(&cutoff),
        "the cutoff {cutoff} is the read's receiver instant, inside [{before}, {after}]"
    );
    let mut body = answers.clone();
    body["observations_cutoff_unix_ms"] = json!(cutoff.to_string());
    Ok(body)
}

/// B07-P1 · one preview over eighteen declared recipes: every exclusion code and both eligibility
/// paths (every filter passed; every filter passed but a ranking figure missing), the equality
/// boundaries (context 32,768 passes, 32,767 does not; latency at the work share passes, one past
/// it does not), a record whose roster labels repeat and hold a control byte (roster labels are not
/// read under RC01 — review G1 — so it screens as any local recipe does), an absent, a disabled and
/// another principal's record — whole, against the independent oracle's
/// table, through the schema oracle.
#[test]
fn a_preview_screens_every_declared_recipe_as_the_independent_oracle_does() -> Outcome {
    let before = wall_ms()?;
    let (world, store) = world()?;
    let tasks = owner(&world, store)?;
    let reply = serve(&tasks, &principal(1000)?, &preview(1, None))?;
    let after = wall_ms()?;
    let answers: Value = serde_json::from_str(ANSWERS)?;
    // The oracle screened the fixture's wall budget; the preview sent spec()'s: they must agree.
    assert_eq!(
        world.fixture["wall_ms"].to_string(),
        spec()["budget"]["wall_ms"].as_str().ok_or("wall")?
    );
    assert_eq!(reply["kind"], json!("result"), "{reply}");
    assert_eq!(reply["effect"], json!("none"));
    assert_eq!(reply["readback"], Value::Null);
    assert_eq!(
        reply["body"],
        expected(&reply, &answers["answers"], before, after)?
    );
    conforms(&[("task.preview", &reply)])
}

/// B07-P2 · the route's proof: after a preview under key K, `task.get` by K finds nothing, the
/// ledger file's event high-water, operations, tasks and roster revisions are unchanged, and K is
/// not consumed — a submission under it then admits.
#[test]
fn a_preview_writes_nothing_and_consumes_no_key() -> Outcome {
    let (world, store) = world()?;
    let tasks = owner(&world, store)?;
    let operator = principal(1000)?;
    let counts = ledger_counts(&world.scratch)?;
    let reply = serve(&tasks, &operator, &preview(2, Some(KEY)))?;
    assert_eq!(reply["kind"], json!("result"), "{reply}");
    assert_eq!(ledger_counts(&world.scratch)?, counts);
    let get = serve(
        &tasks,
        &operator,
        &request(
            "task.get",
            3,
            None,
            &json!({"selector": {"source_action": "task.submit", "idempotency_key": KEY}, "evidence": "none"}),
        ),
    )?;
    assert_eq!(get["code"], json!("not_found"), "{get}");
    let submit = serve(
        &tasks,
        &operator,
        &request("task.submit", 4, Some(KEY), &json!({"spec": spec()})),
    )?;
    assert_eq!(
        (&submit["effect"], &submit["body"]["task"]["state"]),
        (&json!("committed"), &json!("admitted")),
        "{submit}"
    );
    conforms(&[
        ("task.preview", &reply),
        ("task.get", &get),
        ("task.submit", &submit),
    ])
}

/// B07-P3 · after the ledger is reopened, the receiver epoch is new and no observation is current
/// (`docs/roster-contract.md`: reopen invalidates current freshness): every recipe that reaches
/// R05 is `stale`, nothing is eligible and the cost is `unknown` — the oracle's `after_reopen`.
#[test]
fn after_a_reopen_no_observation_is_current() -> Outcome {
    let (world, store) = world()?;
    drop(store);
    let before = wall_ms()?;
    let tasks = owner(&world, raw_store(&world.scratch)?)?;
    let reply = serve(&tasks, &principal(1000)?, &preview(5, None))?;
    let after = wall_ms()?;
    let answers: Value = serde_json::from_str(ANSWERS)?;
    assert_eq!(
        reply["body"],
        expected(&reply, &answers["after_reopen"], before, after)?
    );
    conforms(&[("task.preview", &reply)])
}

fn refusal(reply: &Value) -> (Value, Value, Value) {
    (
        reply["code"].clone(),
        reply["details"]["field"].clone(),
        reply["details"]["constraint"].clone(),
    )
}

/// B07-P4 · the body: a spec defect is refused exactly as `task.submit` refuses it (one door,
/// `task::control::spec`); the two revisions are checked member by member; a precondition is
/// refused as a parent is.
#[test]
fn a_preview_body_is_checked_as_a_submission_is() -> Outcome {
    let (world, store) = world()?;
    let tasks = owner(&world, store)?;
    let operator = principal(1000)?;
    let mut replies = Vec::new();
    for (index, (path, value)) in [
        (&["budget", "tokens"][..], json!("1")),
        (&["budget", "currency_microunits"][..], json!("7")),
        (&["privacy"][..], json!("remote_allowed")),
        (&["task_class"][..], json!("rust-library-change/2")),
        (&["criteria"][..], json!([])),
        (
            &["parent"][..],
            json!({"task_id": KEY, "allocation_id": KEY, "brief_revision": "1"}),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut defective = spec();
        let mut target = &mut defective;
        for step in &path[..path.len() - 1] {
            target = &mut target[*step];
        }
        target[path[path.len() - 1]] = value;
        let byte = 0x10 + u8::try_from(index)? * 2;
        let submitted = serve(
            &tasks,
            &operator,
            &request("task.submit", byte, Some(KEY), &json!({"spec": defective})),
        )?;
        let previewed = serve(
            &tasks,
            &operator,
            &request("task.preview", byte + 1, None, &body(&defective)),
        )?;
        assert_eq!(refusal(&previewed), refusal(&submitted), "{path:?}");
        assert_eq!(previewed["kind"], json!("error"), "{previewed}");
        replies.push(previewed);
    }
    let with = |member: &str, value: Value| {
        let mut preview = body(&spec());
        preview[member] = value;
        preview
    };
    let mut members = body(&spec());
    members["extra"] = json!(1);
    let cases = [
        (
            with("catalogue_revision", json!("2")),
            ("resync_required", "/body/catalogue_revision"),
        ),
        (
            with("catalogue_revision", json!("01")),
            ("invalid_argument", "/body/catalogue_revision"),
        ),
        (
            with("brief_revision", json!("1")),
            ("invalid_argument", "/body/brief_revision"),
        ),
        (
            with("brief_revision", json!(0)),
            ("invalid_argument", "/body/brief_revision"),
        ),
        (members, ("invalid_argument", "/body")),
    ];
    for (index, (case, (code, field))) in cases.into_iter().enumerate() {
        let reply = serve(
            &tasks,
            &operator,
            &request("task.preview", 0x30 + u8::try_from(index)?, None, &case),
        )?;
        assert_eq!(
            (&reply["code"], &reply["details"]["field"]),
            (&json!(code), &json!(field)),
            "{reply}"
        );
        replies.push(reply);
    }
    let mut frame: Value = serde_json::from_slice(&preview(0x40, None))?;
    frame["precondition"] = json!({"resource": "task", "id": KEY, "generation": "1"});
    let reply = serve(&tasks, &operator, &serde_json::to_vec(&frame)?)?;
    assert_eq!(
        refusal(&reply),
        (
            json!("unavailable"),
            json!("/precondition"),
            json!("child allocations are not composed behind this receiver")
        )
    );
    replies.push(reply);
    let rows: Vec<(&str, &Value)> = replies
        .iter()
        .map(|reply| ("task.preview", reply))
        .collect();
    conforms(&rows)
}

/// B07-P5 · the order every task action keeps: shape, then owner, then the catalogue revision,
/// then routing. With no owner, a valid RC01 body is `owner not composed` (and a bad catalogue
/// revision too: the owner is checked first); a spec defect is refused before the owner.
#[test]
fn shape_comes_before_the_owner_and_the_owner_before_the_revision() -> Outcome {
    let operator = principal(1000)?;
    let unowned = |payload: &[u8]| -> Result<Value, Box<dyn Error>> {
        match receiver::serve(payload, NOW, &operator, &Open) {
            receiver::Reply::Frame(bytes) => Ok(serde_json::from_slice(&bytes)?),
            receiver::Reply::Close(fault) => Err(format!("closed: {}", fault.name()).into()),
        }
    };
    // The shared receiver fixture's own base case (t28_control's oracle battery) is refused for
    // its shape — its spec asks for tokens RC01 does not admit — before the owner (review G2).
    let cases: Value =
        serde_json::from_str(include_str!("fixtures/native/control-v1/actions.json"))?;
    let base = cases["cases"]
        .as_array()
        .ok_or("cases")?
        .iter()
        .find(|case| case["action"] == json!("task.preview"))
        .ok_or("base case")?;
    let at: u64 = base["request"]["deadline_unix_ms"]
        .as_str()
        .ok_or("deadline")?
        .parse()?;
    let fixture = match receiver::serve(
        &serde_json::to_vec(&base["request"])?,
        at - 1_000,
        &operator,
        &Open,
    ) {
        receiver::Reply::Frame(bytes) => serde_json::from_slice::<Value>(&bytes)?,
        receiver::Reply::Close(fault) => return Err(format!("closed: {}", fault.name()).into()),
    };
    assert_eq!(
        (&fixture["code"], &fixture["details"]["field"]),
        (&json!("unavailable"), &json!("/body/spec/budget/tokens")),
        "{fixture}"
    );
    let valid = unowned(&preview(0x50, None))?;
    assert_eq!(
        (&valid["code"], &valid["details"]["constraint"]),
        (&json!("unavailable"), &json!("owner not composed")),
        "{valid}"
    );
    let mut stale = body(&spec());
    stale["catalogue_revision"] = json!("2");
    let stale = unowned(&request("task.preview", 0x51, None, &stale))?;
    assert_eq!(
        stale["details"]["constraint"],
        json!("owner not composed"),
        "{stale}"
    );
    let mut tokens = spec();
    tokens["budget"]["tokens"] = json!("1");
    let shape = unowned(&request("task.preview", 0x52, None, &body(&tokens)))?;
    assert_eq!(
        (&shape["code"], &shape["details"]["field"]),
        (&json!("unavailable"), &json!("/body/spec/budget/tokens")),
        "{shape}"
    );
    conforms(&[
        ("task.preview", &valid),
        ("task.preview", &stale),
        ("task.preview", &shape),
    ])
}

/// B07-P6 · each reason preview cannot answer is named, statically: no routing installed, a
/// refused configuration (the shipped one: its baseline names no declared recipe), the baseline's
/// record invisible to this caller (records are scoped by uid and role: the design's named
/// limitation until cross-principal visibility), and a baseline no longer usable — decided per
/// preview from the roster, over one routing value composed once (R3.5).
#[test]
fn a_preview_names_why_it_cannot_answer() -> Outcome {
    let (world, store) = world()?;
    let operator = principal(1000)?;
    let constraint = |tasks: &StoreTasks, principal: &Principal, byte: u8| {
        serve(tasks, principal, &preview(byte, None)).map(|reply| {
            assert_eq!(reply["code"], json!("unavailable"), "{reply}");
            reply["details"]["constraint"].clone()
        })
    };
    let bare = StoreTasks::new(store, EPOCH.to_owned());
    assert_eq!(
        constraint(&bare, &operator, 0x60)?,
        json!("route configuration not installed")
    );
    drop(bare);
    let shipped = StoreTasks::new(raw_store(&world.scratch)?, EPOCH.to_owned())
        .with_routing(routing::compose(CONFIG.as_bytes()));
    assert_eq!(
        constraint(&shipped, &operator, 0x61)?,
        json!("route configuration refused")
    );
    drop(shipped);
    let composed = routing::compose(configuration(&world)?.as_bytes());
    let tasks = StoreTasks::new(raw_store(&world.scratch)?, EPOCH.to_owned())
        .with_routing(composed.clone());
    assert_eq!(
        constraint(&tasks, &principal(1001)?, 0x62)?,
        json!("the route baseline's roster record is not available to this caller")
    );
    let answered = serve(&tasks, &operator, &preview(0x63, None))?;
    assert_eq!(answered["kind"], json!("result"), "{answered}");
    drop(tasks);
    // The baseline's record is updated to remote: the same routing value now cannot serve.
    let mut store = raw_store(&world.scratch)?;
    let baseline = world.records.get("B").ok_or("B")?;
    let head = store
        .roster_get(
            &operator,
            habitat_engine::contracts::UuidV4::parse(baseline)?,
            deadline(),
        )
        .map_err(|error| format!("{error:?}"))?
        .head;
    let mut definition = head.definition.clone();
    definition.locality = Locality::Remote;
    let update = Update {
        idempotency_key: "07b00000-0000-4000-8000-0000000000d1".into(),
        record_id: Some(head.record_id.clone()),
        expected_revision: Some(head.record_version.clone()),
        definition,
        audit_reason: "B07 baseline moved remote".into(),
    };
    let raw = serde_json::to_vec(&json!({
        "protocol": "hee3.control", "version": 1, "kind": "request",
        "request_id": update.idempotency_key, "action": "roster.update", "action_version": 1,
        "idempotency_key": update.idempotency_key, "deadline_unix_ms": "1030000",
        "authority": {"grant_id": KEY, "scope_sha256": format!("sha256:{}", "4".repeat(64))},
        "precondition": {"resource": "roster", "id": head.record_id, "generation": head.record_version},
        "body": {"record_id": head.record_id, "definition": update.definition, "audit_reason": update.audit_reason},
    }))?;
    store
        .roster_apply(
            &operator,
            &[update],
            RequestSource::Native(&raw),
            deadline(),
        )
        .map_err(|error| format!("{error:?}"))?;
    let moved = StoreTasks::new(store, EPOCH.to_owned()).with_routing(composed);
    assert_eq!(
        constraint(&moved, &operator, 0x64)?,
        json!("the route baseline is not usable")
    );
    Ok(())
}

/// The routing directory as the operator installs it: private, owned, 0700; `routes.toml` 0600.
fn installed(scratch: &Scratch, source: &[u8]) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let directory = scratch.0.join("routing");
    DirBuilder::new().mode(0o700).create(&directory)?;
    let file = directory.join(ROUTES_FILE);
    fs::write(&file, source)?;
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600))?;
    Ok(directory)
}

/// B07-P7 · the configuration is read under the grants' custody (R1.4, R3.2, R3.3): only an
/// absent directory or file reads "not installed"; a group-readable directory, a 0644 file, a
/// symlink and an oversize file are refused; a private file loads.
#[test]
fn routing_is_read_under_custody() -> Outcome {
    let (world, store) = world()?;
    drop(store);
    let source = configuration(&world)?;
    let scratch = Scratch::new()?;
    assert_eq!(
        routing::read(&scratch.0.join("absent")).err(),
        Some(Unready::NotInstalled)
    );
    let directory = installed(&scratch, source.as_bytes())?;
    assert!(routing::read(&directory).is_ok());
    let file = directory.join(ROUTES_FILE);
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644))?;
    assert_eq!(routing::read(&directory).err(), Some(Unready::Refused));
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600))?;
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o755))?;
    assert_eq!(routing::read(&directory).err(), Some(Unready::Refused));
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
    let aside = directory.join("aside.toml");
    fs::rename(&file, &aside)?;
    assert_eq!(routing::read(&directory).err(), Some(Unready::NotInstalled));
    std::os::unix::fs::symlink(&aside, &file)?;
    assert_eq!(routing::read(&directory).err(), Some(Unready::Refused));
    fs::remove_file(&file)?;
    let limit = usize::try_from(MAX_ROUTE_CONFIG_BYTES)?;
    let mut oversize = source.into_bytes();
    oversize.resize(limit + 1, b'\n');
    fs::write(&file, &oversize)?;
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600))?;
    assert_eq!(routing::read(&directory).err(), Some(Unready::Refused));
    oversize.truncate(limit);
    fs::write(&file, &oversize)?;
    assert!(
        routing::read(&directory).is_ok(),
        "exactly the bound is admitted"
    );
    Ok(())
}

/// B07-P8 · composition refuses what preview could not serve: bytes that are not UTF-8, the
/// shipped configuration (its baseline names no declared recipe), a recipe serving a class the
/// catalogue does not admit, a baseline lacking a ranking figure. The bound is derived from the
/// widest values, not guessed: the shipped file (anchor block included) plus 128 recipe rows with
/// every value at its widest composes under `MAX_ROUTE_CONFIG_BYTES` (formatting padding is not
/// bounded by it, and is refused past it). A
/// figure's widest declarable value is `i64::MAX`: TOML integers are 64-bit signed (the spec
/// requires an error for one not representable losslessly), so `u64::MAX` refuses the file.
#[test]
fn composition_refuses_what_preview_could_not_serve() -> Outcome {
    assert_eq!(
        routing::compose(&[0xff, 0xfe]).err(),
        Some(Unready::Refused)
    );
    assert_eq!(
        routing::compose(CONFIG.as_bytes()).err(),
        Some(Unready::Refused)
    );
    let row_of = |id: &str, class: &str, figure: &str| {
        format!(
            "{{ id = \"{id}\", version = 65535, adapter = \"07b00000-0000-4000-8000-0000000000a0\", \
             actual_model_required = false, roster_record = \"07b00000-0000-4000-8000-0000000000a1\", \
             serves = [\"{class}\"], context_limit_tokens = {figure}, \
             cost_microunits = {figure}, quality_basis_points = 10000, \
             latency_ms = {figure} }}"
        )
    };
    let widest_figure = i64::MAX.to_string();
    let row = |id: &str, class: &str| row_of(id, class, &widest_figure);
    let declare = |rows: &[String], baseline: &str| {
        CONFIG
            .replacen(
                "recipes = []",
                &format!("recipes = [\n  {},\n]", rows.join(",\n  ")),
                1,
            )
            .replacen(DECLARED_BASELINE, baseline, 1)
    };
    let unknown = declare(&[row("b", "rust-library-change/2")], "b");
    assert_eq!(
        routing::compose(unknown.as_bytes()).err(),
        Some(Unready::Refused)
    );
    let undeclared = declare(&[row("b", CLASS)], "c");
    assert_eq!(
        routing::compose(undeclared.as_bytes()).err(),
        Some(Unready::Refused)
    );
    let unsigned = declare(&[row_of("b", CLASS, &u64::MAX.to_string())], "b");
    assert_eq!(
        habitat_engine::route::Routing::parse(&unsigned).err(),
        Some(habitat_engine::route::ConfigError::Syntax)
    );
    assert!(routing::compose(declare(&[row("b", CLASS)], "b").as_bytes()).is_ok());
    // A baseline lacking a figure the ranking reads is a configuration fact no roster change can
    // mend: refused here, at composition (review D1), each figure in turn; a candidate may lack one.
    for figure in ["cost_microunits", "quality_basis_points", "latency_ms"] {
        let full = row("b", CLASS);
        let start = full.find(&format!(", {figure} = ")).ok_or(figure)?;
        let end = full[start + 2..]
            .find([',', ' '])
            .map_or(full.len(), |at| start + 2 + at);
        let end = full[end..]
            .find([',', '}'])
            .map_or(full.len(), |at| end + at);
        let lacking = format!("{}{}", &full[..start], &full[end..]);
        assert!(!lacking.contains(figure), "{lacking}");
        let baseline = declare(std::slice::from_ref(&lacking), "b");
        assert_eq!(
            routing::compose(baseline.as_bytes()).err(),
            Some(Unready::Refused),
            "{figure}"
        );
        let candidate = declare(
            &[row("b", CLASS), lacking.replacen("\"b\"", "\"c\"", 1)],
            "b",
        );
        assert!(routing::compose(candidate.as_bytes()).is_ok(), "{figure}");
    }
    let widest: Vec<String> = (0..128)
        .map(|index| row(&format!("{index:0>128}"), CLASS))
        .collect();
    let full = declare(&widest, &format!("{:0>128}", 0));
    assert!(
        full.len() < usize::try_from(MAX_ROUTE_CONFIG_BYTES)?,
        "{} bytes exceed the bound",
        full.len()
    );
    assert!(routing::compose(full.as_bytes()).is_ok());
    Ok(())
}

/// B07-P9 · the owner is handed what it was asked for: the transport's principal, the spec, the
/// deadline and the receipt time (F101: a double that records).
#[test]
fn the_preview_owner_is_handed_the_request() -> Outcome {
    let handed = super::tasks::Handed::default();
    let operator = principal(1000)?;
    let reply = serve_composed_at(&handed, &Open, &operator, &preview(0x70, None), NOW)?;
    assert_eq!(reply["code"], json!("deadline_exceeded"), "{reply}");
    // The spec's own literals: wall 600000 ms; its work share is that less RC01's 5 min
    // verification reserve (docs/contract-decisions.md RC01), 300000; the criteria in order; the
    // catalogue revision the body named.
    assert_eq!(spec()["budget"]["wall_ms"], json!("600000"));
    assert_eq!(
        *handed.0.borrow(),
        [format!(
            "preview {operator:?} {CLASS} 600000 300000 rejects a leading zero|round-trips the maximum 1 {} {NOW}",
            NOW + 5_000
        )]
    );
    Ok(())
}
