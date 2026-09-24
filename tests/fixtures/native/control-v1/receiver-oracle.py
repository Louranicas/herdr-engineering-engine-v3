#!/usr/bin/env python3
"""Independent oracle for the HEE3-Control/1 receiver (`tests/t28_control.rs`).

The Rust receiver (`contracts::control`, `actions::control`) and this file share a specification,
RC03, and nothing else: no code, no table, no generated artefact. What each case should produce
is decided here by other machinery -- Python's `json` module with hooks for the rules JSON Schema
cannot see, `jsonschema` over the published `schemas/actions/control-v1.schema.json`, `hashlib`
for the exact-byte digest -- so a green run is agreement between two implementations rather than
one implementation agreeing with itself.

    receiver-oracle.py cases       print the cases as one JSON document
    receiver-oracle.py validate    read [{name, action, reply}] on stdin; validate every reply
    receiver-oracle.py digests     print each published per-action schema file's SHA-256, and
                                   whether each file is its bundle definition plus exactly the
                                   closure it references (decision D-4)

Categories: close (no reply may be written) · refused (invalid_argument and the protocol,
version and action codes) · deadline · resync · unavailable · served.

jsonschema is installed test tooling only, as for tests/skill_schema.py.
"""
import copy
import hashlib
import json
import re
import sys
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[4]
SCHEMAS = ROOT / "schemas/actions"
SCHEMA = json.loads((SCHEMAS / "control-v1.schema.json").read_text(encoding="utf-8"))
FIXTURES = json.loads((Path(__file__).parent / "actions.json").read_text(encoding="utf-8"))
RECEIVE_UNIX_MS = 1_769_999_995_000
MAX_FRAME = 1_048_576
MAX_DEPTH = 32
# The continuation-cursor lifetime the receiver promises (decision record T28/control-receiver).
CURSOR_LIFETIME_MS = 300_000
UUID = re.compile(r"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}")
ACTIONS = SCHEMA["$defs"]["ActionId"]["enum"]
GRANT = "123e4567-e89b-42d3-a456-000000000003"
SCOPE = "sha256:" + "1" * 64


def validator(definition):
    return Draft202012Validator({**SCHEMA, "oneOf": [{"$ref": "#/$defs/" + definition}]})


REQUEST = validator("ControlRequestV1")


def envelope_validator(action):
    """Request_<action> without its cross-field rules that read the body.

    RC03 section 1: the receiving module owns its action body, and section 5 validates the
    body only after the grant. A rule that reads the body -- roster.update's "create requires
    precondition null" is the only one -- is therefore the owner's, and an action whose owner is
    not composed behind the receiver is refused `unavailable` before any body rule runs.
    """
    definitions = copy.deepcopy(SCHEMA["$defs"])
    definitions["Request_" + action.replace(".", "_")].pop("allOf", None)
    return Draft202012Validator({**SCHEMA, "$defs": definitions,
                                 "oneOf": [{"$ref": "#/$defs/Request_" + action.replace(".", "_")}]})


def compact(value):
    return json.dumps(value, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def filter_sha256(query):
    return "sha256:" + hashlib.sha256(compact({"query": query})).hexdigest()


# ---- tools.inspect: what the published files and the bundle say -------------------------

def schema_file(kind, action=None):
    return f"control-v1.{kind}.schema.json" if action is None else f"control-v1.{kind}.{action}.schema.json"


def file_sha256(name):
    return "sha256:" + hashlib.sha256((SCHEMAS / name).read_bytes()).hexdigest()


def pinned_readback(action):
    """The readback action the bundle's Result_<action> pins, or None for the generic selector."""
    result = SCHEMA["$defs"]["Result_" + action.replace(".", "_")]
    for alternative in result["properties"]["readback"]["oneOf"]:
        for part in alternative.get("allOf", []):
            pinned = part.get("properties", {}).get("action", {}).get("const")
            if pinned is not None:
                return pinned
    return None


def inspection(action):
    return {"action": action, "version": 1,
            "request_schema_sha256": file_sha256(schema_file("request", action)),
            "result_schema_sha256": file_sha256(schema_file("result", action)),
            "error_schema_sha256": file_sha256(schema_file("error")),
            "max_request_bytes": MAX_FRAME, "max_deadline_ms": 60_000,
            "readback_action": pinned_readback(action)}


def refs(value):
    if isinstance(value, dict):
        for key, member in value.items():
            if key == "$ref":
                yield member
            else:
                yield from refs(member)
    elif isinstance(value, list):
        for member in value:
            yield from refs(member)


def standalone_faults(name, definition):
    """How a published file departs from `definition` plus exactly the closure it references."""
    published = json.loads((SCHEMAS / name).read_text(encoding="utf-8"))
    faults = []
    own = published.pop("$defs", {})
    for key in ("$schema", "$id", "title"):
        published.pop(key, None)
    if published != SCHEMA["$defs"][definition]:
        faults.append(f"{name}: root is not the bundle's {definition}")
    wanted, frontier = set(), [definition]
    while frontier:
        for target in refs(SCHEMA["$defs"][frontier.pop()]):
            referenced = target.removeprefix("#/$defs/")
            if referenced not in wanted and referenced != definition:
                wanted.add(referenced)
                frontier.append(referenced)
    if set(own) != wanted:
        faults.append(f"{name}: $defs {sorted(set(own) ^ wanted)} differ from the closure")
    faults += [f"{name}: {key} differs from the bundle" for key in sorted(set(own) & wanted)
               if own[key] != SCHEMA["$defs"][key]]
    return faults


def digests():
    faults = standalone_faults(schema_file("error"), "ControlErrorV1")
    actions = {}
    for action in ACTIONS:
        stem = action.replace(".", "_")
        faults += standalone_faults(schema_file("request", action), "Request_" + stem)
        faults += standalone_faults(schema_file("result", action), "Result_" + stem)
        actions[action] = {"request": file_sha256(schema_file("request", action)),
                           "result": file_sha256(schema_file("result", action)),
                           "readback_action": pinned_readback(action)}
    return {"error": file_sha256(schema_file("error")), "actions": actions, "faults": faults}


# ---- the frame rules, decided by Python's parser plus explicit hooks ----------------------

class Refuse(Exception):
    pass


def unique(pairs):
    names = [name for name, _ in pairs]
    if len(names) != len(set(names)):
        raise Refuse("duplicate name")
    return dict(pairs)


def integer_token(text):
    if text.startswith("-"):
        raise Refuse("negative")
    return int(text)


def no_float(_text):
    raise Refuse("float")


def depth(value):
    if isinstance(value, dict):
        return 1 + max((depth(v) for v in value.values()), default=0)
    if isinstance(value, list):
        return 1 + max((depth(v) for v in value), default=0)
    return 0


def strings(value):
    if isinstance(value, dict):
        for name, member in value.items():
            yield name
            yield from strings(member)
    elif isinstance(value, list):
        for member in value:
            yield from strings(member)
    elif isinstance(value, str):
        yield value


def admitted(payload):
    """The parsed object, or None when RC03 section 3 forbids a reply."""
    if len(payload) > MAX_FRAME or payload.startswith(b"\xef\xbb\xbf"):
        return None
    try:
        text = payload.decode("utf-8")
    except UnicodeDecodeError:
        return None
    # Whitespace outside strings: blank every string literal, then look.
    if re.search(r"[ \t\r\n]", re.sub(r'"(?:[^"\\\x00-\x1f]|\\.)*"', '""', text)):
        return None
    try:
        value = json.loads(text, object_pairs_hook=unique, parse_int=integer_token,
                           parse_float=no_float, parse_constant=no_float)
    except (Refuse, ValueError):
        return None
    if not isinstance(value, dict) or depth(value) > MAX_DEPTH:
        return None
    try:
        for text_value in strings(value):
            text_value.encode("utf-8")
    except UnicodeEncodeError:
        return None
    request_id = value.get("request_id")
    if not isinstance(request_id, str) or not UUID.fullmatch(request_id):
        return None
    return value


def listing(query, after, limit):
    """The ids a fully visible tools.list page holds, from the schema's own ActionId order."""
    matching = [name for name in ACTIONS if query is None or query in name]
    start = 0 if after is None else matching.index(after) + 1
    width = min(limit, 32)
    return matching[start:start + width], start + width < len(matching)


def expect(payload):
    request = admitted(payload)
    if request is None:
        return {"expect": "close"}
    verdict = {"request_sha256": "sha256:" + hashlib.sha256(payload).hexdigest()}
    if not REQUEST.is_valid(request):
        action = request.get("action")
        unserved = action in ACTIONS and action not in ("tools.list", "tools.inspect")
        if not (unserved and envelope_validator(action).is_valid(request)):
            return {**verdict, "expect": "refused"}
    deadline = int(request["deadline_unix_ms"])
    if deadline <= RECEIVE_UNIX_MS:
        return {**verdict, "expect": "deadline"}
    if deadline - RECEIVE_UNIX_MS > 60_000:
        return {**verdict, "expect": "refused"}
    if request["action"] == "tools.inspect":
        return {**verdict, "expect": "served", "inspection": inspection(request["body"]["action"])}
    if request["action"] != "tools.list":
        return {**verdict, "expect": "unavailable"}
    body = request["body"]
    query = body["query"]
    if query is not None and len(query.encode("utf-8")) > 256:
        return {**verdict, "expect": "refused"}
    cursor = body["page"]["cursor"]
    after = None
    if cursor is not None:
        if cursor["snapshot_revision"] != "1" or int(cursor["expires_unix_ms"]) <= RECEIVE_UNIX_MS:
            return {**verdict, "expect": "resync"}
        matching = [name for name in ACTIONS if query is None or query in name]
        if cursor["filter_sha256"] != filter_sha256(query) or cursor["after_key"] not in matching:
            return {**verdict, "expect": "refused"}
        after = cursor["after_key"]
    ids, more = listing(query, after, body["page"]["limit"])
    next_cursor = None
    if more:
        next_cursor = {"snapshot_revision": "1", "after_key": ids[-1],
                       "filter_sha256": filter_sha256(query),
                       "expires_unix_ms": str(RECEIVE_UNIX_MS + CURSOR_LIFETIME_MS)}
    return {**verdict, "expect": "served", "ids": ids, "next_cursor": next_cursor}


# ---- the cases -----------------------------------------------------------------------------

def bases():
    """Each fixture request, re-dated into the receive window, tools.list without a cursor."""
    for case in FIXTURES["cases"]:
        request = copy.deepcopy(case["request"])
        request["deadline_unix_ms"] = str(RECEIVE_UNIX_MS + 5_000)
        request["authority"] = {"grant_id": GRANT, "scope_sha256": SCOPE}
        if request["action"] == "tools.list":
            request["body"] = {"query": None, "page": {"limit": 100, "cursor": None}}
        yield request["action"], request


def mutated(base, path, value):
    request = copy.deepcopy(base)
    target = request
    for step in path[:-1]:
        target = target[step]
    if value is DELETE:
        del target[path[-1]]
    else:
        target[path[-1]] = value
    return request


DELETE = object()
MEMBERS = ["protocol", "version", "kind", "request_id", "action", "action_version",
           "idempotency_key", "deadline_unix_ms", "authority", "precondition", "body"]


def cases():
    out = []

    def add(name, payload, action):
        out.append({"name": name, "action": action, "payload_hex": payload.hex(), **expect(payload)})

    by_action = dict(bases())
    for action, request in by_action.items():
        add(f"base/{action}", compact(request), action)
    listing_base = by_action["tools.list"]
    # Envelope rules, on one read, one creating, one mutating and one forbidding action.
    for action in ("tools.list", "health", "task.submit", "task.cancel", "roster.update"):
        base = by_action[action]
        for member in MEMBERS:
            add(f"drop/{action}/{member}", compact(mutated(base, [member], DELETE)), action)
        for name, path, value in [
            ("extra-member", ["extra"], 1),
            ("protocol-other", ["protocol"], "hee3.other"),
            ("protocol-number", ["protocol"], 1),
            ("version-2", ["version"], 2),
            ("version-0", ["version"], 0),
            ("version-string", ["version"], "1"),
            ("version-bool", ["version"], True),
            ("version-65536", ["version"], 65536),
            ("kind-result", ["kind"], "result"),
            ("request-id-upper", ["request_id"], base["request_id"].upper()),
            ("request-id-v1", ["request_id"], "123e4567-e89b-12d3-a456-000000000001"),
            ("action-unknown", ["action"], "tools.nope"),
            ("action-version-2", ["action_version"], 2),
            ("action-version-string", ["action_version"], "1"),
            ("idempotency-malformed", ["idempotency_key"], "not-a-uuid"),
            ("idempotency-null", ["idempotency_key"], None),
            ("deadline-leading-zero", ["deadline_unix_ms"], "0" + str(RECEIVE_UNIX_MS + 5_000)),
            ("deadline-number", ["deadline_unix_ms"], RECEIVE_UNIX_MS + 5_000),
            ("deadline-now", ["deadline_unix_ms"], str(RECEIVE_UNIX_MS)),
            ("deadline-past", ["deadline_unix_ms"], str(RECEIVE_UNIX_MS - 1)),
            ("deadline-at-horizon", ["deadline_unix_ms"], str(RECEIVE_UNIX_MS + 60_000)),
            ("deadline-beyond-horizon", ["deadline_unix_ms"], str(RECEIVE_UNIX_MS + 60_001)),
            ("deadline-overflow", ["deadline_unix_ms"], "18446744073709551616"),
            ("authority-missing-scope", ["authority"], {"grant_id": GRANT}),
            ("authority-extra", ["authority"], {"grant_id": GRANT, "scope_sha256": SCOPE, "x": 1}),
            ("authority-grant-upper", ["authority"], {"grant_id": GRANT.upper(), "scope_sha256": SCOPE}),
            ("authority-scope-short", ["authority"], {"grant_id": GRANT, "scope_sha256": SCOPE[:-1]}),
            ("precondition-task", ["precondition"],
             {"resource": "task", "id": GRANT, "generation": "7"}),
            ("precondition-roster", ["precondition"],
             {"resource": "roster", "id": GRANT, "generation": "7"}),
            ("precondition-generation-zero", ["precondition"],
             {"resource": "task", "id": GRANT, "generation": "0"}),
            ("precondition-null", ["precondition"], None),
            ("body-array", ["body"], []),
        ]:
            add(f"{name}/{action}", compact(mutated(base, path, value)), action)
    # The tools.list body, which this receiver materializes.
    for name, path, value in [
        ("limit-0", ["body", "page", "limit"], 0),
        ("limit-1", ["body", "page", "limit"], 1),
        ("limit-5", ["body", "page", "limit"], 5),
        ("limit-100", ["body", "page", "limit"], 100),
        ("limit-101", ["body", "page", "limit"], 101),
        ("limit-string", ["body", "page", "limit"], "5"),
        ("query-task", ["body", "query"], "task"),
        ("query-none-match", ["body", "query"], "zzz"),
        ("query-infix", ["body", "query"], "list"),
        ("query-separator", ["body", "query"], "."),
        ("query-empty", ["body", "query"], ""),
        ("query-256-bytes", ["body", "query"], "a" * 256),
        ("query-257-bytes", ["body", "query"], "a" * 257),
        # 129 code points, 258 bytes: within the schema's maxLength, beyond RC03's byte bound.
        ("query-258-bytes-129-points", ["body", "query"], "é" * 129),
        ("query-astral", ["body", "query"], "\U0001f600"),
        ("body-extra", ["body", "extra"], 1),
        ("body-no-query", ["body", "query"], DELETE),
        ("page-no-cursor", ["body", "page", "cursor"], DELETE),
        ("page-extra", ["body", "page", "extra"], 1),
    ]:
        add(f"list/{name}", compact(mutated(listing_base, path, value)), "tools.list")

    def cursor(**changes):
        value = {"snapshot_revision": "1", "after_key": "task.submit",
                 "filter_sha256": filter_sha256(None), "expires_unix_ms": str(RECEIVE_UNIX_MS + 1)}
        value.update(changes)
        return value

    for name, value, query in [
        ("resume", cursor(), None),
        ("resume-last", cursor(after_key="health"), None),
        ("resume-query", cursor(after_key="task.get", filter_sha256=filter_sha256("task")), "task"),
        ("resume-foreign-filter", cursor(filter_sha256=SCOPE), None),
        ("resume-query-changed", cursor(), "task"),
        ("resume-unknown-key", cursor(after_key="tools.nope"), None),
        ("resume-key-outside-filter", cursor(after_key="health", filter_sha256=filter_sha256("task")), "task"),
        ("resume-expired", cursor(expires_unix_ms=str(RECEIVE_UNIX_MS)), None),
        ("resume-other-snapshot", cursor(snapshot_revision="2"), None),
        ("resume-snapshot-leading-zero", cursor(snapshot_revision="01"), None),
        ("resume-extra", {**cursor(), "x": "1"}, None),
    ]:
        request = mutated(listing_base, ["body", "page", "cursor"], value)
        request["body"]["query"] = query
        request["body"]["page"]["limit"] = 3
        add(f"cursor/{name}", compact(request), "tools.list")

    # The tools.inspect body: every action inspected, then each way the body can be wrong.
    inspect_base = by_action["tools.inspect"]
    for target in ACTIONS:
        add(f"inspect/{target}", compact(mutated(inspect_base, ["body", "action"], target)),
            "tools.inspect")
    for name, path, value in [
        ("action-unknown", ["body", "action"], "tools.nope"),
        ("action-number", ["body", "action"], 7),
        ("action-missing", ["body", "action"], DELETE),
        ("version-2", ["body", "version"], 2),
        ("version-0", ["body", "version"], 0),
        ("version-string", ["body", "version"], "1"),
        ("version-missing", ["body", "version"], DELETE),
        ("body-extra", ["body", "extra"], 1),
    ]:
        add(f"inspect/{name}", compact(mutated(inspect_base, path, value)), "tools.inspect")

    # Frame rules on a request that is otherwise served.
    good = compact(listing_base)
    text = good.decode("utf-8")
    deep = "[" * 31 + "]" * 31
    for name, payload in [
        ("frame/pretty", json.dumps(listing_base, indent=1).encode()),
        ("frame/space-after-colon", text.replace('"kind":', '"kind": ', 1).encode()),
        ("frame/leading-space", b" " + good),
        ("frame/trailing-space", good + b" "),
        ("frame/trailing-cr", good + b"\r"),
        ("frame/bom", b"\xef\xbb\xbf" + good),
        ("frame/empty", b""),
        ("frame/array", b"[" + good + b"]"),
        ("frame/trailing-data", good + b"x"),
        ("frame/two-objects", good + good),
        ("frame/duplicate", text.replace('"kind":"request"', '"kind":"request","kind":"request"', 1).encode()),
        ("frame/duplicate-escaped", text.replace('"kind":"request"', '"kind":"request","\\u006bind":"request"', 1).encode()),
        ("frame/duplicate-nested", text.replace('"limit":100', '"limit":100,"limit":100', 1).encode()),
        ("frame/float-version", text.replace('"version":1', '"version":1.0', 1).encode()),
        ("frame/exponent-version", text.replace('"version":1', '"version":1e0', 1).encode()),
        ("frame/negative-limit", text.replace('"limit":100', '"limit":-1', 1).encode()),
        ("frame/leading-zero-limit", text.replace('"limit":100', '"limit":0100', 1).encode()),
        ("frame/lone-high-surrogate", text.replace('"query":null', '"query":"\\ud800"', 1).encode()),
        ("frame/lone-low-surrogate", text.replace('"query":null', '"query":"\\udc00"', 1).encode()),
        ("frame/reversed-pair", text.replace('"query":null', '"query":"\\ude00\\ud83d"', 1).encode()),
        ("frame/escaped-pair", text.replace('"query":null', '"query":"\\ud83d\\ude00"', 1).encode()),
        ("frame/raw-tab-in-string", text.replace('"query":null', '"query":"a\tb"', 1).encode()),
        ("frame/escaped-tab-in-string", text.replace('"query":null', '"query":"a\\tb"', 1).encode()),
        ("frame/invalid-utf8", text.replace('"query":null', '"query":"a\x00b"', 1).encode().replace(b"\x00", b"\xff")),
        ("frame/depth-32", text.replace('"body":', '"extra":' + deep + ',"body":', 1).encode()),
        ("frame/depth-33", text.replace('"body":', '"extra":[' + deep + '],"body":', 1).encode()),
        ("frame/object-depth-32", text.replace('"body":', '"extra":' + '{"a":' * 30 + '{}' + '}' * 30 + ',"body":', 1).encode()),
        ("frame/object-depth-33", text.replace('"body":', '"extra":' + '{"a":' * 31 + '{}' + '}' * 31 + ',"body":', 1).encode()),
        ("frame/no-request-id", compact(mutated(listing_base, ["request_id"], DELETE))),
        ("frame/request-id-number", compact(mutated(listing_base, ["request_id"], 7))),
        ("frame/nan", text.replace('"limit":100', '"limit":NaN', 1).encode()),
        ("frame/true-literal-typo", text.replace('"precondition":null', '"precondition":nul', 1).encode()),
    ]:
        add(name, payload, "tools.list")

    # The acquisition bound, from both sides, padded through a member the envelope refuses.
    head = compact(mutated(listing_base, ["zz"], ""))
    pad = MAX_FRAME - len(head)
    at_bound = head.replace(b'"zz":""', b'"zz":"' + b"a" * pad + b'"', 1)
    assert len(at_bound) == MAX_FRAME, len(at_bound)
    add("bound/at-1048576", at_bound, "tools.list")
    add("bound/over-1048576", at_bound.replace(b'"zz":"', b'"zz":"a', 1), "tools.list")
    names = [case["name"] for case in out]
    assert len(names) == len(set(names)), "case names must be unique"
    return out


def validate(rows):
    invalid = []
    for row in rows:
        reply = json.loads(row["reply"])
        definition = "ControlErrorV1" if reply.get("kind") == "error" else "Result_" + row["action"].replace(".", "_")
        errors = sorted(validator(definition).iter_errors(reply), key=lambda error: list(error.path))
        if errors:
            invalid.append({"name": row["name"], "definition": definition,
                            "first": errors[0].message[:200]})
    return {"checked": len(rows), "invalid": invalid}


def main(argv):
    if argv[1:] == ["cases"]:
        generated = cases()
        json.dump({"receive_unix_ms": RECEIVE_UNIX_MS, "cases": generated}, sys.stdout)
        return 0
    if argv[1:] == ["digests"]:
        json.dump(digests(), sys.stdout)
        return 0
    if argv[1:] == ["validate"]:
        json.dump(validate(json.load(sys.stdin)), sys.stdout)
        return 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
