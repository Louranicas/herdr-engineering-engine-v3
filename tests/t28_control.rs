//! T28 control receiver cases (a module of `t28_actions`): HEE3-Control/1 from bytes to one reply (`contracts::control`,
//! `actions::control`), against RC03 and against an oracle that shares nothing with the code.
//!
//! The load-bearing case is the first: `fixtures/native/control-v1/receiver-oracle.py` decides
//! what each of its cases must produce with Python's parser, `jsonschema` over the published
//! schema and `hashlib`, and validates every reply this receiver writes against the schema's
//! result and error definitions. The rest pin what an oracle over whole requests cannot reach:
//! the stream reader's acquisition bound, the grant seam, the catalogue door, and each rendered
//! record whole.
use habitat_engine::actions::control::{
    self, CURSOR_LIFETIME_MS, Grants, MAX_PAGE_LIMIT, NoGrants, Reply, filter_sha256,
};
use habitat_engine::actions::{Caller, Catalogue, Effect, Owner, PreconditionRule};
use habitat_engine::contracts::control::{
    ErrorCode, Fault, FrameFault, FrameReader, MAX_FRAME_BYTES, ReadError, Received, ResourceKind,
    Retry, admit_object, read_result_frame, receive, request_sha256,
};
use habitat_engine::store::Principal;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

type Outcome = Result<(), Box<dyn Error>>;

const ORACLE: &str = "tests/fixtures/native/control-v1/receiver-oracle.py";
const NOW: u64 = 1_769_999_995_000;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(relative: &str) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&std::fs::read(
        root().join(relative),
    )?)?)
}

fn principal() -> Result<Principal, Box<dyn Error>> {
    Principal::new(1000, "operator").map_err(|error| format!("{error:?}").into())
}

/// Sees every owner and holds every effect: the oracle's cases are about the wire, not grants.
fn everything() -> Caller {
    let caller = Owner::ALL.into_iter().fold(Caller::new(), Caller::seeing);
    Effect::ALL.into_iter().fold(caller, Caller::granted)
}

/// A grant store that records what it was handed (F101) and answers with a fixed caller.
struct Recording {
    caller: Option<Caller>,
    asked: RefCell<Vec<(String, String, String, u64)>>,
}

impl Recording {
    fn answering(caller: Option<Caller>) -> Self {
        Self {
            caller,
            asked: RefCell::new(Vec::new()),
        }
    }
}

impl Grants for Recording {
    fn resolve(
        &self,
        principal: &Principal,
        grant_id: &str,
        scope_sha256: &str,
        now_unix_ms: u64,
    ) -> Option<Caller> {
        self.asked.borrow_mut().push((
            format!("{principal:?}"),
            grant_id.to_owned(),
            scope_sha256.to_owned(),
            now_unix_ms,
        ));
        self.caller.clone()
    }
}

fn oracle(argument: &str, input: Option<&[u8]>) -> Result<Value, Box<dyn Error>> {
    let mut child = Command::new("python3")
        .args(["-W", "error", ORACLE, argument])
        .current_dir(root())
        .env_remove("FORCE_COLOR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(bytes) = input {
        child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    }
    drop(child.stdin.take());
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "oracle {argument} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn hex(text: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    (0..text.len())
        .step_by(2)
        .map(|at| Ok(u8::from_str_radix(&text[at..at + 2], 16)?))
        .collect()
}

fn serve(payload: &[u8], grants: &impl Grants) -> Result<Reply, Box<dyn Error>> {
    Ok(control::serve(payload, NOW, &principal()?, grants))
}

fn reply(payload: &[u8], grants: &impl Grants) -> Result<Value, Box<dyn Error>> {
    match serve(payload, grants)? {
        Reply::Frame(bytes) => {
            // One record: its only LF is its last byte.
            assert_eq!(
                bytes.iter().position(|&b| b == b'\n'),
                bytes.len().checked_sub(1),
                "exactly one LF, at the end"
            );
            Ok(serde_json::from_slice(&bytes)?)
        }
        Reply::Close(fault) => Err(format!("closed: {}", fault.name()).into()),
    }
}

fn category(reply: &Value) -> String {
    match (reply["kind"].as_str(), reply["code"].as_str()) {
        (Some("result"), _) => "served".into(),
        (
            Some("error"),
            Some(
                "invalid_argument"
                | "unsupported_protocol"
                | "unsupported_version"
                | "unknown_action"
                | "unsupported_action_version",
            ),
        ) => "refused".into(),
        (Some("error"), Some("deadline_exceeded")) => "deadline".into(),
        (Some("error"), Some("resync_required")) => "resync".into(),
        (Some("error"), Some("unavailable")) => "unavailable".into(),
        (kind, code) => format!("unexpected {kind:?}/{code:?}"),
    }
}

fn listing(limit: u64, query: Option<&str>, cursor: &Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "protocol": "hee3.control", "version": 1, "kind": "request",
        "request_id": "123e4567-e89b-42d3-a456-00000000000a",
        "action": "tools.list", "action_version": 1, "idempotency_key": null,
        "deadline_unix_ms": (NOW + 1_000).to_string(),
        "authority": {"grant_id": "123e4567-e89b-42d3-a456-00000000000b",
                      "scope_sha256": format!("sha256:{}", "2".repeat(64))},
        "precondition": null,
        "body": {"query": query, "page": {"limit": limit, "cursor": cursor}},
    }))
    .unwrap_or_default()
}

#[test]
fn the_receiver_agrees_with_the_independent_oracle_on_every_case() -> Outcome {
    let generated = oracle("cases", None)?;
    assert_eq!(generated["receive_unix_ms"], json!(NOW));
    let cases = generated["cases"].as_array().ok_or("cases")?;
    let grants = Recording::answering(Some(everything()));
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut replies = Vec::new();
    let mut disagreements = Vec::new();
    for case in cases {
        let name = case["name"].as_str().ok_or("name")?;
        let expected = case["expect"].as_str().ok_or("expect")?;
        let payload = hex(case["payload_hex"].as_str().ok_or("payload")?)?;
        *seen.entry(expected.to_owned()).or_default() += 1;
        let actual = match serve(&payload, &grants)? {
            Reply::Close(_) => {
                if expected != "close" {
                    disagreements.push(format!("{name}: expected {expected}, closed"));
                }
                continue;
            }
            Reply::Frame(bytes) => bytes,
        };
        let parsed: Value = serde_json::from_slice(&actual)?;
        let got = category(&parsed);
        if got != expected {
            disagreements.push(format!(
                "{name}: expected {expected}, got {got} {}",
                String::from_utf8_lossy(&actual)
            ));
            continue;
        }
        if parsed["request_sha256"] != case["request_sha256"] {
            disagreements.push(format!("{name}: digest {}", parsed["request_sha256"]));
        }
        if expected == "served" {
            let ids: Vec<&Value> = parsed["body"]["page"]["items"]
                .as_array()
                .ok_or("items")?
                .iter()
                .map(|item| &item["id"])
                .collect();
            let wanted: Vec<&Value> = case["ids"].as_array().ok_or("ids")?.iter().collect();
            if ids != wanted || parsed["body"]["page"]["next_cursor"] != case["next_cursor"] {
                disagreements.push(format!("{name}: page {}", parsed["body"]["page"]));
            }
        }
        replies.push(json!({
            "name": name,
            "action": case["action"],
            "reply": String::from_utf8(actual)?,
        }));
    }
    assert!(
        disagreements.is_empty(),
        "{} of {} cases disagree:\n{}",
        disagreements.len(),
        cases.len(),
        disagreements.join("\n")
    );
    // Every category the oracle decides must be exercised, or a shrunken oracle reads green.
    for wanted in [
        "close",
        "refused",
        "deadline",
        "resync",
        "unavailable",
        "served",
    ] {
        assert!(
            seen.get(wanted).copied().unwrap_or(0) > 0,
            "no {wanted} case in {seen:?}"
        );
    }
    let checked = oracle("validate", Some(&serde_json::to_vec(&replies)?))?;
    assert_eq!(
        checked["checked"],
        json!(replies.len()),
        "every reply validated"
    );
    assert_eq!(
        checked["invalid"],
        json!([]),
        "replies outside the schema: {checked}"
    );
    // The grant store was consulted only for requests that reached it, and always with the
    // transport's principal and the request's own grant and scope.
    let asked = grants.asked.borrow();
    assert!(!asked.is_empty(), "the grant store was never consulted");
    let principal = format!("{:?}", principal()?);
    for (who, grant, scope, now) in asked.iter() {
        assert_eq!(
            (who, grant.as_str(), scope.as_str(), *now),
            (
                &principal,
                "123e4567-e89b-42d3-a456-000000000003",
                "sha256:1111111111111111111111111111111111111111111111111111111111111111",
                NOW
            )
        );
    }
    Ok(())
}

#[test]
fn the_digest_is_the_published_rc03_known_answer() -> Outcome {
    // C01 retains the exact published RC03 example bytes and hash; neither was produced here.
    let metadata = read_json("tests/fixtures/native/control-v1/C01-metadata.json")?;
    let record = std::fs::read(
        root().join("tests/fixtures/native/control-v1/C01-control-valid-cancel.jsonl"),
    )?;
    let payload = record.strip_suffix(b"\n").ok_or("one terminal LF")?;
    assert_eq!(json!(payload.len()), metadata["payload_bytes_excluding_lf"]);
    let published = metadata["payload_sha256"].as_str().ok_or("digest")?;
    assert_eq!(request_sha256(payload), format!("sha256:{published}"));
    // And the receiver echoes that digest: C01's deadline is inside the fixture's window.
    let receive: u64 = metadata["fixture_receive_unix_ms"]
        .as_str()
        .ok_or("receive")?
        .parse()?;
    let grants = Recording::answering(Some(everything()));
    let Reply::Frame(bytes) = control::serve(payload, receive, &principal()?, &grants) else {
        return Err("C01 closed".into());
    };
    let reply: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(
        reply["request_sha256"],
        json!(format!("sha256:{published}"))
    );
    assert_eq!(
        reply["request_id"],
        json!("123e4567-e89b-42d3-a456-000000000001")
    );
    Ok(())
}

/// A source handing out its bytes a few at a time, counting what was taken.
struct Trickle {
    bytes: Vec<u8>,
    at: usize,
    step: usize,
    interrupt_first: bool,
}

impl Read for Trickle {
    fn read(&mut self, into: &mut [u8]) -> io::Result<usize> {
        if self.interrupt_first {
            self.interrupt_first = false;
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        let take = self.step.min(into.len()).min(self.bytes.len() - self.at);
        into[..take].copy_from_slice(&self.bytes[self.at..self.at + take]);
        self.at += take;
        Ok(take)
    }
}

/// The frames a reader produced, and the fault that stopped it, if one did.
type Frames = (Vec<Vec<u8>>, Option<FrameFault>);

fn frames(source: impl Read) -> Result<Frames, Box<dyn Error>> {
    let mut reader = FrameReader::new(source);
    let mut out = Vec::new();
    for _ in 0..16 {
        match reader.next_frame() {
            Ok(Some(frame)) => out.push(frame),
            Ok(None) => return Ok((out, None)),
            Err(ReadError::Fault(fault)) => return Ok((out, Some(fault))),
            Err(ReadError::Io(error)) => return Err(error.into()),
        }
    }
    Err("more than 16 frames from a bounded fixture".into())
}

#[test]
fn frames_split_across_partial_reads_and_coalesced_records_are_the_same_frames() -> Outcome {
    let stream = b"{\"a\":1}\n{\"b\":[2]}\n\n{\"c\":\"x\\ny\"}\n".to_vec();
    let expected = vec![
        b"{\"a\":1}".to_vec(),
        b"{\"b\":[2]}".to_vec(),
        Vec::new(),
        b"{\"c\":\"x\\ny\"}".to_vec(),
    ];
    for step in [1, 3, 7, stream.len()] {
        let source = Trickle {
            bytes: stream.clone(),
            at: 0,
            step,
            interrupt_first: step == 3,
        };
        assert_eq!(frames(source)?, (expected.clone(), None), "step {step}");
    }
    Ok(())
}

/// An endless record: every byte `a`, counting how many were ever read.
struct Endless {
    taken: usize,
    then: &'static [u8],
}

impl Read for Endless {
    fn read(&mut self, into: &mut [u8]) -> io::Result<usize> {
        if self.taken > MAX_FRAME_BYTES * 2 {
            let take = self.then.len().min(into.len());
            into[..take].copy_from_slice(&self.then[..take]);
            return Ok(take);
        }
        into.fill(b'a');
        self.taken += into.len();
        Ok(into.len())
    }
}

#[test]
fn an_oversize_record_is_refused_having_read_one_byte_past_the_bound() {
    let mut source = Endless {
        taken: 0,
        then: b"{}\n",
    };
    let mut reader = FrameReader::new(&mut source);
    assert!(matches!(
        reader.next_frame(),
        Err(ReadError::Fault(FrameFault::Oversize))
    ));
    // The acquisition is the bound: exactly one byte beyond the largest payload was taken.
    drop(reader);
    assert_eq!(source.taken, MAX_FRAME_BYTES + 1);
    // And a closed reader does not resynchronize on whatever follows.
    let mut oversized = vec![b'a'; MAX_FRAME_BYTES + 1];
    oversized.extend_from_slice(b"\n{}\n");
    let mut reader = FrameReader::new(oversized.as_slice());
    for _ in 0..3 {
        assert!(matches!(
            reader.next_frame(),
            Err(ReadError::Fault(FrameFault::Oversize))
        ));
    }
}

#[test]
fn a_record_of_exactly_the_bound_is_delivered_whole() -> Outcome {
    let mut stream = vec![b'a'; MAX_FRAME_BYTES];
    stream.extend_from_slice(b"\n{}\n");
    let (got, fault) = frames(stream.as_slice())?;
    assert_eq!(fault, None);
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].len(), MAX_FRAME_BYTES);
    assert_eq!(got[1], b"{}");
    Ok(())
}

#[test]
fn a_stream_ending_inside_a_record_is_truncated_and_a_clean_end_is_none() -> Outcome {
    assert_eq!(
        frames(&b"{}\n{\"a\""[..])?,
        (vec![b"{}".to_vec()], Some(FrameFault::Truncated))
    );
    assert_eq!(frames(&b""[..])?, (Vec::new(), None));
    assert_eq!(frames(&b"{}\n"[..])?, (vec![b"{}".to_vec()], None));
    Ok(())
}

#[test]
fn the_grant_store_decides_and_sees_the_transport_principal() -> Outcome {
    let payload = listing(1, None, &Value::Null);
    let refused = reply(&payload, &NoGrants)?;
    assert_eq!(refused["code"], json!("forbidden"));
    assert_eq!(refused["retry"], json!("after_condition"));
    assert_eq!(refused["details"]["field"], json!("/authority/grant_id"));
    let grants = Recording::answering(None);
    let other = Principal::new(4242, "reviewer").map_err(|error| format!("{error:?}"))?;
    let Reply::Frame(_) = control::serve(&payload, NOW + 7, &other, &grants) else {
        return Err("closed".into());
    };
    assert_eq!(
        grants.asked.borrow().as_slice(),
        &[(
            format!("{other:?}"),
            "123e4567-e89b-42d3-a456-00000000000b".to_owned(),
            format!("sha256:{}", "2".repeat(64)),
            NOW + 7,
        )]
    );
    Ok(())
}

#[test]
fn visibility_and_effect_grants_go_through_the_catalogue_door() -> Outcome {
    let payload = listing(MAX_PAGE_LIMIT, None, &Value::Null);
    // The catalogue's own owner unseen: tools.list reads as absent, never as forbidden.
    let hidden = Recording::answering(Some(
        Caller::new().seeing(Owner::Task).granted(Effect::Read),
    ));
    let absent = reply(&payload, &hidden)?;
    assert_eq!(absent["code"], json!("unknown_action"));
    assert_eq!(absent["details"]["field"], json!("/action"));
    // Seen but not granted: forbidden, at the action.
    let unread = Recording::answering(Some(
        Caller::new()
            .seeing(Owner::Actions)
            .granted(Effect::DurableAdmission),
    ));
    let forbidden = reply(&payload, &unread)?;
    assert_eq!(forbidden["code"], json!("forbidden"));
    assert_eq!(forbidden["details"]["field"], json!("/action"));
    // Seen and granted: only what the caller sees is listed.
    let partial = Recording::answering(Some(
        Caller::new()
            .seeing(Owner::Actions)
            .seeing(Owner::Roster)
            .granted(Effect::Read),
    ));
    let listed = reply(&payload, &partial)?;
    let ids: Vec<&str> = listed["body"]["page"]["items"]
        .as_array()
        .ok_or("items")?
        .iter()
        .filter_map(|item| item["id"].as_str())
        .collect();
    let expected: Vec<&str> = Catalogue::all()
        .iter()
        .filter(|action| matches!(action.owner, Owner::Actions | Owner::Roster))
        .map(|action| action.id)
        .collect();
    assert_eq!(ids, expected);
    assert_eq!(ids.len(), 6, "tools.* and roster.*: {ids:?}");
    Ok(())
}

#[test]
fn each_action_requires_of_the_envelope_what_the_schema_requires() -> Outcome {
    let schema = read_json("schemas/actions/control-v1.schema.json")?;
    let definitions = &schema["$defs"];
    let mut compared = 0;
    for action in Catalogue::all() {
        let request =
            &definitions[format!("Request_{}", action.id.replace('.', "_"))]["properties"];
        let idempotency_required = request["idempotency_key"] == json!({"$ref": "#/$defs/UuidV4"});
        assert_eq!(
            idempotency_required,
            action.effect.mutates(),
            "{}: idempotency",
            action.id
        );
        let rule = &request["precondition"];
        let kind = |shape: &Value| -> Result<ResourceKind, Box<dyn Error>> {
            let name = shape["allOf"][1]["properties"]["resource"]["const"]
                .as_str()
                .ok_or("resource const")?;
            ResourceKind::ALL
                .into_iter()
                .find(|kind| kind.name() == name)
                .ok_or_else(|| format!("unknown kind {name}").into())
        };
        let declared = if *rule == json!({"type": "null"}) {
            PreconditionRule::Forbidden
        } else if let Some(options) = rule["oneOf"].as_array() {
            assert_eq!(options[0], json!({"type": "null"}), "{}", action.id);
            PreconditionRule::Optional(kind(&options[1])?)
        } else {
            PreconditionRule::Required(kind(rule)?)
        };
        assert_eq!(action.precondition, declared, "{}: precondition", action.id);
        compared += 1;
    }
    assert_eq!(compared, 21);
    let kinds: Vec<&str> = ResourceKind::ALL
        .into_iter()
        .map(ResourceKind::name)
        .collect();
    assert_eq!(json!(kinds), definitions["ResourceKind"]["enum"]);
    Ok(())
}

#[test]
fn error_codes_are_the_schema_enum_in_order() -> Outcome {
    let schema = read_json("schemas/actions/control-v1.schema.json")?;
    let names: Vec<&str> = ErrorCode::ALL.into_iter().map(ErrorCode::name).collect();
    assert_eq!(json!(names), schema["$defs"]["ErrorCodeV1"]["enum"]);
    Ok(())
}

#[test]
fn every_purpose_is_a_distinct_description_within_its_bound() {
    let mut seen = Vec::new();
    for action in Catalogue::all() {
        assert!(
            (1..=256).contains(&action.purpose.len()),
            "{}: {} bytes",
            action.id,
            action.purpose.len()
        );
        assert!(
            !seen.contains(&action.purpose),
            "{} repeats a purpose",
            action.id
        );
        seen.push(action.purpose);
    }
}

#[test]
fn refusals_render_whole() {
    // Two refusals differing in every field, each asserted as the exact record (F124).
    let first = Fault::invalid("/body/page/limit", "integer 1..100")
        .frame("123e4567-e89b-42d3-a456-000000000001", "sha256:aa");
    assert_eq!(
        String::from_utf8_lossy(&first),
        "{\"code\":\"invalid_argument\",\"details\":{\"constraint\":\"integer 1..100\",\
         \"current_generation\":null,\"field\":\"/body/page/limit\"},\"effect\":\"none\",\
         \"kind\":\"error\",\"message\":\"request does not satisfy HEE3-Control/1\",\
         \"protocol\":\"hee3.control\",\"readback\":null,\
         \"request_id\":\"123e4567-e89b-42d3-a456-000000000001\",\"request_sha256\":\"sha256:aa\",\
         \"retry\":\"never\",\"version\":1}\n"
    );
    let second = Fault::of(ErrorCode::Unavailable, Retry::AfterCondition, "later")
        .frame("123e4567-e89b-42d3-a456-000000000002", "sha256:bb");
    assert_eq!(
        String::from_utf8_lossy(&second),
        "{\"code\":\"unavailable\",\"details\":{\"constraint\":null,\
         \"current_generation\":null,\"field\":null},\"effect\":\"none\",\
         \"kind\":\"error\",\"message\":\"later\",\
         \"protocol\":\"hee3.control\",\"readback\":null,\
         \"request_id\":\"123e4567-e89b-42d3-a456-000000000002\",\"request_sha256\":\"sha256:bb\",\
         \"retry\":\"after_condition\",\"version\":1}\n"
    );
    let result = read_result_frame(
        "123e4567-e89b-42d3-a456-000000000003",
        "sha256:cc",
        &json!({"k": 2}),
    );
    assert_eq!(
        String::from_utf8_lossy(&result),
        "{\"body\":{\"k\":2},\"effect\":\"none\",\"kind\":\"result\",\"observed_generation\":null,\
         \"operation_id\":null,\"protocol\":\"hee3.control\",\"readback\":null,\"replayed\":false,\
         \"request_id\":\"123e4567-e89b-42d3-a456-000000000003\",\"request_sha256\":\"sha256:cc\",\
         \"version\":1}\n"
    );
}

#[test]
fn following_cursors_pages_the_whole_catalogue_once() -> Outcome {
    let grants = Recording::answering(Some(everything()));
    let mut cursor = Value::Null;
    let mut ids = Vec::new();
    let budget = Catalogue::all().len();
    for page in 0..=budget {
        assert!(
            page < budget,
            "{page} pages did not finish {budget} actions"
        );
        let listed = reply(&listing(2, None, &cursor), &grants)?;
        let items = listed["body"]["page"]["items"].as_array().ok_or("items")?;
        assert!(items.len() <= 2);
        ids.extend(
            items
                .iter()
                .filter_map(|item| item["id"].as_str().map(str::to_owned)),
        );
        cursor = listed["body"]["page"]["next_cursor"].clone();
        if cursor.is_null() {
            break;
        }
        // Off the origin: the expiry is this receipt's clock plus the lifetime, not a constant.
        assert_eq!(
            cursor["expires_unix_ms"],
            json!((NOW + CURSOR_LIFETIME_MS).to_string())
        );
        assert_eq!(cursor["filter_sha256"], json!(filter_sha256(None)));
    }
    let all: Vec<String> = Catalogue::all()
        .iter()
        .map(|action| action.id.to_owned())
        .collect();
    assert_eq!(ids, all);
    Ok(())
}

/// `{"a":` nested `depth` objects deep, counting the outermost.
fn objects(depth: usize) -> String {
    format!(
        "{}{{}}{}",
        "{\"a\":".repeat(depth - 1),
        "}".repeat(depth - 1)
    )
}

/// An object holding arrays nested so the whole is `depth` deep.
fn arrays(depth: usize) -> String {
    format!(
        "{{\"a\":{}{}}}",
        "[".repeat(depth - 1),
        "]".repeat(depth - 1)
    )
}

#[test]
fn each_frame_rule_refuses_by_its_own_name() {
    // Several rules are also caught later by serde's parser, so "the frame closed" cannot tell
    // them apart; each case names the fault only its own rule produces (F140).
    let cases: Vec<(&str, Vec<u8>, FrameFault)> = vec![
        ("empty", b"".to_vec(), FrameFault::Empty),
        ("bom", b"\xef\xbb\xbf{}".to_vec(), FrameFault::ByteOrderMark),
        (
            "invalid utf-8",
            b"{\"a\":\"\xff\"}".to_vec(),
            FrameFault::InvalidUtf8,
        ),
        ("leading space", b" {}".to_vec(), FrameFault::Whitespace),
        ("inner space", b"{ }".to_vec(), FrameFault::Whitespace),
        (
            "space after colon",
            b"{\"a\": 1}".to_vec(),
            FrameFault::Whitespace,
        ),
        ("trailing cr", b"{}\r".to_vec(), FrameFault::Whitespace),
        ("array", b"[]".to_vec(), FrameFault::NotObject),
        ("number", b"1".to_vec(), FrameFault::NotObject),
        (
            "duplicate",
            b"{\"a\":1,\"a\":2}".to_vec(),
            FrameFault::DuplicateName,
        ),
        (
            "escaped duplicate",
            b"{\"a\":1,\"\\u0061\":2}".to_vec(),
            FrameFault::DuplicateName,
        ),
        (
            "pair duplicate",
            "{\"\\ud83d\\ude00\":1,\"\u{1f600}\":2}".as_bytes().to_vec(),
            FrameFault::DuplicateName,
        ),
        (
            "nested duplicate",
            b"{\"a\":{\"b\":1,\"b\":1}}".to_vec(),
            FrameFault::DuplicateName,
        ),
        ("objects 33", objects(33).into_bytes(), FrameFault::TooDeep),
        ("arrays 33", arrays(33).into_bytes(), FrameFault::TooDeep),
        (
            "leading zero",
            b"{\"a\":01}".to_vec(),
            FrameFault::NumberToken,
        ),
        ("fraction", b"{\"a\":1.0}".to_vec(), FrameFault::NumberToken),
        ("exponent", b"{\"a\":1e0}".to_vec(), FrameFault::NumberToken),
        ("negative", b"{\"a\":-1}".to_vec(), FrameFault::NumberToken),
        (
            "raw tab",
            b"{\"a\":\"\t\"}".to_vec(),
            FrameFault::ControlCharacter,
        ),
        (
            "lone high",
            b"{\"a\":\"\\ud800\"}".to_vec(),
            FrameFault::UnpairedSurrogate,
        ),
        (
            "high then text",
            b"{\"a\":\"\\ud800x\"}".to_vec(),
            FrameFault::UnpairedSurrogate,
        ),
        (
            "high then high",
            b"{\"a\":\"\\ud800\\ud800\"}".to_vec(),
            FrameFault::UnpairedSurrogate,
        ),
        (
            "lone low",
            b"{\"a\":\"\\udc00\"}".to_vec(),
            FrameFault::UnpairedSurrogate,
        ),
        ("literal typo", b"{\"a\":tru}".to_vec(), FrameFault::Syntax),
        ("trailing comma", b"{\"a\":1,}".to_vec(), FrameFault::Syntax),
        ("trailing data", b"{}x".to_vec(), FrameFault::Syntax),
        (
            "unknown escape",
            b"{\"a\":\"\\x\"}".to_vec(),
            FrameFault::Syntax,
        ),
        ("unterminated", b"{\"a\":\"x".to_vec(), FrameFault::Syntax),
    ];
    for (name, payload, fault) in cases {
        assert_eq!(admit_object(&payload).err(), Some(fault), "{name}");
    }
}

#[test]
fn the_frame_rules_admit_their_boundaries_from_the_other_side() {
    // The refusing half alone leaves each bound free to move inward.
    assert!(admit_object(objects(32).as_bytes()).is_ok(), "objects 32");
    assert!(admit_object(arrays(32).as_bytes()).is_ok(), "arrays 32");
    let admitted = admit_object("{\"\\ud83d\\ude00\":\"a\\tb\",\"n\":0,\"m\":10}".as_bytes());
    assert_eq!(
        admitted.ok().map(serde_json::Value::Object),
        Some(json!({"\u{1f600}": "a\tb", "n": 0, "m": 10}))
    );
    assert_eq!(
        receive(b"{\"a\":1}", NOW),
        Received::Closed(FrameFault::Uncorrelated)
    );
}

/// One object filled to within one member of [`MAX_FRAME_BYTES`] with distinct four-letter names
/// (`aaaa`, `aaab`, ...), closed by a member named `last`. Returns the payload and how many
/// members it holds.
fn wide_object(last: &str) -> (Vec<u8>, usize) {
    let closing = format!("\"{last}\":0}}");
    let mut payload = b"{".to_vec();
    let mut members = 0_usize;
    while payload.len() + 9 + closing.len() <= MAX_FRAME_BYTES {
        let mut name = [b'a'; 4];
        let mut rest = members;
        for slot in name.iter_mut().rev() {
            *slot = b"abcdefghijklmnopqrstuvwxyz"[rest % 26];
            rest /= 26;
        }
        payload.push(b'"');
        payload.extend_from_slice(&name);
        payload.extend_from_slice(b"\":0,");
        members += 1;
    }
    payload.extend_from_slice(closing.as_bytes());
    (payload, members + 1)
}

#[test]
fn a_bound_sized_object_is_judged_on_its_last_member_against_its_first() -> Outcome {
    // CON-04: one frame of about 116 000 distinct names is admissible before any grant check,
    // so duplicate detection must not rescan every earlier name per member. The duplicate is the
    // last member repeating the first, so only a check over the whole object can see it, and the
    // mirror differs only in that name. Both judge results only; there is no timing budget.
    let (duplicate, members) = wide_object("aaaa");
    assert!(
        duplicate.len() > MAX_FRAME_BYTES - 9 && duplicate.len() <= MAX_FRAME_BYTES,
        "frame {} bytes against a bound of {MAX_FRAME_BYTES}",
        duplicate.len()
    );
    assert_eq!(
        admit_object(&duplicate).err(),
        Some(FrameFault::DuplicateName)
    );
    let (distinct, same) = wide_object("zzzz");
    assert_eq!((distinct.len(), same), (duplicate.len(), members));
    let admitted = admit_object(&distinct).map_err(|fault| format!("{fault:?}"))?;
    assert_eq!(admitted.len(), members, "every distinct member admitted");
    assert_eq!(admitted.get("zzzz"), Some(&json!(0)));
    Ok(())
}

#[test]
fn a_version_outside_its_domain_is_invalid_and_one_inside_it_is_unsupported() -> Outcome {
    // RC03 §2: a version is an integer in 1..=65535. Zero and 65536 are not versions at all;
    // 2 is a version this receiver does not speak. The oracle groups all of them as refusals,
    // so the code each one earns is pinned here.
    let grants = Recording::answering(Some(everything()));
    for (member, value, code, field) in [
        ("version", json!(0), "invalid_argument", "/version"),
        ("version", json!(65_536), "invalid_argument", "/version"),
        ("version", json!(2), "unsupported_version", "/version"),
        ("version", json!(65_535), "unsupported_version", "/version"),
        (
            "action_version",
            json!(0),
            "invalid_argument",
            "/action_version",
        ),
        (
            "action_version",
            json!(65_536),
            "invalid_argument",
            "/action_version",
        ),
        (
            "action_version",
            json!(2),
            "unsupported_action_version",
            "/action_version",
        ),
    ] {
        let mut request: Value = serde_json::from_slice(&listing(1, None, &Value::Null))?;
        request[member] = value.clone();
        let refused = reply(&serde_json::to_vec(&request)?, &grants)?;
        assert_eq!(
            (
                refused["code"].as_str(),
                refused["details"]["field"].as_str()
            ),
            (Some(code), Some(field)),
            "{member}={value}"
        );
    }
    Ok(())
}
