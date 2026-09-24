#!/usr/bin/env python3
"""Validate a finite procedure against HEE3-IF-workflows.

A JSON Schema states the shape of a procedure. It cannot state that dependencies name steps
that exist, that the graph is acyclic, or that a resume does not repeat an effect nobody
observed -- so those are obligations of this validator, and each one is named in the schema's
`hee3.refusals` list. That list is the world this module is checked against: `refusal_sites()`
enumerates what this file can actually raise, and the tests require the two to agree, so a
refusal added to one and forgotten in the other is a red test rather than a silent gap.

Refusals are raised by symbol, never as prose, and every one names the step it is about.

This module decides nothing about acceptance. `join()` returns a disposition -- a completion
CANDIDATE, a repair or a block -- and the parent's acceptance remains an independent decision
made elsewhere. Nothing here schedules, sleeps, retries or performs an effect.

Run as a command, it validates one procedure file: `python3 workflows/validate_procedure.py
<procedure.json>` prints `order=<steps>` and exits 0, prints `refused <code>: <detail>` and
exits 1, or names why the file could not be read as JSON and exits 2.
"""

import ast
import hashlib
import json
import sys
import uuid
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA_PATH = HERE / "procedure-v1.schema.json"
SCHEMA_STRING = "hee3.workflows.procedure.v1"


class ProcedureRefusal(Exception):
    """A procedure was refused. `code` is one of the schema's `hee3.refusals`."""

    def __init__(self, code, detail):
        super().__init__(f"{code}: {detail}")
        self.code = code
        self.detail = detail


def refuse(code, detail):
    raise ProcedureRefusal(code, detail)


def schema():
    return json.loads(SCHEMA_PATH.read_text())


def bounds():
    return schema()["hee3"]["bounds"]


def _constant_calls(matches):
    """The constant first argument of every call in this file whose callee `matches`."""
    tree = ast.parse(Path(__file__).read_text())
    return [
        node.args[0].value for node in ast.walk(tree)
        if isinstance(node, ast.Call) and matches(node.func) and node.args
        and isinstance(node.args[0], ast.Constant)
    ]


def refusal_sites():
    """Every `refuse(...)` code this file can raise, read from its own source.

    Enumerated from the syntax tree rather than from a list an author maintains: a list would
    agree with the code only while somebody kept it agreeing, and the absent one is exactly
    the one a list cannot see.
    """
    return _constant_calls(lambda f: isinstance(f, ast.Name) and f.id == "refuse")


def block_reason_sites():
    """Every block reason `join()` can append, read from its own source, as refusals are."""
    return _constant_calls(lambda f: isinstance(f, ast.Attribute) and f.attr == "append"
                           and isinstance(f.value, ast.Name) and f.value.id == "reasons")


_JSON_TYPES = {"object": dict, "array": list, "string": str, "boolean": bool}


def _is(value, kind):
    if kind == "integer":
        # Python's bool is an int; JSON's boolean is not an integer.
        return isinstance(value, int) and not isinstance(value, bool)
    return isinstance(value, _JSON_TYPES[kind])


def _shape(value, node, where, definitions):
    """Refuse a member that is absent or of the wrong JSON type, as the schema declares them.

    The schema is the one statement of shape; this reads its `type`, `required`, `properties`
    and `items` so every index the validator makes afterwards is onto a member known to exist
    with a known type. Ranges, enums and limits are not checked here: each has its own named
    refusal below. Depth follows the schema, never the input.
    """
    if "$ref" in node:
        node = definitions[node["$ref"].rsplit("/", 1)[1]]
    kind = node.get("type")
    if kind is not None and not _is(value, kind):
        refuse("malformed_procedure", f"{where} is not {kind}")
    if kind == "object":
        for member in node.get("required", []):
            if member not in value:
                refuse("malformed_procedure", f"{where} lacks required member {member!r}")
        for member, child in node.get("properties", {}).items():
            if member in value:
                _shape(value[member], child, f"{where}.{member}", definitions)
    elif kind == "array":
        for index, item in enumerate(value):
            _shape(item, node.get("items", {}), f"{where}[{index}]", definitions)


def validate(procedure):
    """Validate one procedure's structure and budget, returning a topological step order.

    The order is the deliverable: a caller that receives one has a graph proven acyclic and
    complete, which is not a state a caller can construct for itself.
    """
    document = schema()
    limits = document["hee3"]["bounds"]
    # Another schema string is another version, whose shape this one cannot judge; so the
    # version is read first and the shape only once the version is known to be this one.
    if (isinstance(procedure, dict) and "schema" in procedure
            and procedure["schema"] != SCHEMA_STRING):
        refuse("stale_procedure_version", f"schema is {procedure['schema']!r}")
    _shape(procedure, document, "procedure", document["$defs"])
    steps = procedure["steps"]
    if not 1 <= len(steps) <= limits["max_steps"]:
        refuse("fanout_exceeded",
               f"{len(steps)} steps against a limit of {limits['max_steps']}")
    admitted = set(document["$defs"]["step"]["properties"]["action"]["enum"])
    served = document["hee3"]["action_versions"]

    by_id = {}
    for step in steps:
        identity = step["step_id"]
        if identity in by_id:
            refuse("duplicate_step", f"step_id {identity!r} appears twice")
        by_id[identity] = step
        if step["action"] not in admitted:
            refuse("unknown_action", f"{identity}: action {step['action']!r} is not admitted")
        if step["action_version"] != served[step["action"]]:
            refuse("unsupported_action_version",
                   f"{identity}: action_version {step['action_version']}; the catalogue "
                   f"serves version {served[step['action']]} of {step['action']!r}")
        attempts = step["retry"]["max_attempts"]
        if not 1 <= attempts <= limits["max_retries"]:
            refuse("retry_limit",
                   f"{identity}: {attempts} attempts against a limit of {limits['max_retries']}")
        if "effect_unknown" in step["retry"].get("retry_on", []):
            refuse("unreconciled_effect",
                   f"{identity}: effect_unknown is not retryable; it is reconciled, never repeated")
        dependencies = step.get("depends_on", [])
        if len(dependencies) > limits["max_dependencies"]:
            refuse("dependency_limit",
                   f"{identity}: {len(dependencies)} dependencies against a limit of "
                   f"{limits['max_dependencies']}")

    for step in steps:
        for dependency in step.get("depends_on", []):
            if dependency not in by_id:
                refuse("missing_step",
                       f"{step['step_id']} depends on {dependency!r}, which this procedure "
                       "does not contain")

    criteria = set()
    for criterion in procedure["acceptance"]:
        identity = criterion["criterion_id"]
        if identity in criteria:
            refuse("duplicate_criterion", f"criterion_id {identity!r} appears twice")
        criteria.add(identity)

    budget = procedure["budget"]
    if not 1 <= budget["max_fanout"] <= limits["max_fanout"]:
        refuse("fanout_exceeded",
               f"max_fanout {budget['max_fanout']} against a limit of {limits['max_fanout']}")
    if budget["max_seconds"] > limits["max_total_seconds"]:
        refuse("time_limit",
               f"max_seconds {budget['max_seconds']} against a limit of "
               f"{limits['max_total_seconds']}")
    if budget["max_tokens"] > limits["max_total_tokens"]:
        refuse("resource_limit",
               f"max_tokens {budget['max_tokens']} against a limit of "
               f"{limits['max_total_tokens']}")

    # Kahn's algorithm. The remainder after it settles IS the cycle, so the refusal names the
    # steps involved rather than reporting that one exists somewhere.
    remaining = {identity: set(step.get("depends_on", [])) for identity, step in by_id.items()}
    order = []
    while True:
        ready = sorted(i for i, deps in remaining.items() if not deps)
        if not ready:
            break
        for identity in ready:
            order.append(identity)
            del remaining[identity]
        for deps in remaining.values():
            deps.difference_update(ready)
    if remaining:
        refuse("cycle", "steps in a dependency cycle: " + ", ".join(sorted(remaining)))
    return order


def _committed_steps(procedure, record, known):
    """The step states a record carries, once it is shown to describe THIS procedure.

    One door for `resume()` and `join()` alike, so the two cannot disagree about what a
    foreign record is. `record` is `{procedure_id, procedure_version, steps}`, where `steps`
    maps step_id to one of the schema's `hee3.step_states`. A record written under another
    procedure is `identity_mismatch`; under another version of this one, whose step ids may
    coincide while meaning different things, it is `stale_procedure_version`; a step id the
    procedure does not contain, or a state outside the vocabulary, is `identity_mismatch`.
    """
    if not isinstance(record, dict) or not isinstance(record.get("steps"), dict):
        refuse("identity_mismatch", "the committed record carries no step map")
    if record.get("procedure_id") != procedure["procedure_id"]:
        refuse("identity_mismatch",
               f"record was committed under procedure {record.get('procedure_id')!r}, "
               f"not {procedure['procedure_id']!r}")
    # `type(...) is not int` first: True == 1 and 1.0 == 1 in Python, and the schema this
    # module keeps says a version is an integer, not a boolean or a float (WF-06).
    version = record.get("procedure_version")
    if type(version) is not int or version != procedure["procedure_version"]:
        refuse("stale_procedure_version",
               f"record was committed under version {version!r}; "
               f"the procedure is version {procedure['procedure_version']}")
    states = schema()["hee3"]["step_states"]
    for identity, state in record["steps"].items():
        if identity not in known:
            refuse("identity_mismatch",
                   f"committed step {identity!r} is not in procedure "
                   f"{procedure['procedure_id']!r} version {procedure['procedure_version']}")
        if state not in states:
            refuse("identity_mismatch", f"{identity}: unknown committed state {state!r}")
    return record["steps"]


def resume(procedure, committed):
    """The steps a composition may start now, given what is already committed.

    `committed` is a record as `_committed_steps` reads it. Two rules are structural rather
    than advisory:

    * A step recorded `effect_unknown` is never returned. Its effect may already have landed,
      and repeating it is the harm the state exists to name. It is refused until a
      reconciliation record replaces the state.
    * A record that describes another procedure, another version or a step this procedure
      does not contain is refused, not a value to ignore.
    """
    order = validate(procedure)
    committed = _committed_steps(procedure, committed, set(order))
    for identity, state in committed.items():
        if state == "effect_unknown":
            refuse("unreconciled_effect",
                   f"{identity} is effect_unknown; resume will not repeat it")
    settled = {i for i, s in committed.items() if s in ("done", "cancelled")}
    by_id = {step["step_id"]: step for step in procedure["steps"]}
    ready = [
        identity for identity in order
        if identity not in committed
        and set(by_id[identity].get("depends_on", [])) <= settled
    ]
    return ready[: procedure["budget"]["max_fanout"]]


def join(procedure, outcomes, verified, exhausted):
    """Close a procedure, returning `(disposition, reasons)`.

    `outcomes` is a record as `_committed_steps` reads it. `verified` is the final integrated
    verifier's verdict and `exhausted` whether the procedure's budget ran out; both are
    observed by the caller that runs the procedure, since nothing here runs or meters one.
    Neither has a default, so a caller cannot close a join by forgetting to say.

    A completion CANDIDATE, never an acceptance: whether the parent is accepted is decided
    independently of this module, which is why no branch here returns `accepted`.
    """
    order = validate(procedure)
    outcomes = _committed_steps(procedure, outcomes, set(order))
    by_id = {step["step_id"]: step for step in procedure["steps"]}
    reasons = []
    for identity in order:
        state = outcomes.get(identity)
        # Examined for every step: optional means the join may close without the outcome,
        # not over an effect that may have landed and was never reconciled.
        if state == "effect_unknown":
            reasons.append("unreconciled_effect")
        elif by_id[identity].get("required", True):
            if state is None:
                reasons.append("missing_child")
            elif state != "done":
                reasons.append("unmet_acceptance")
    if not verified:
        reasons.append("unmet_acceptance")
    if exhausted:
        reasons.append("resource_exhausted")
    ordered = [r for r in schema()["hee3"]["block_reasons"] if r in reasons]
    if not ordered:
        return "completion_candidate", []
    if ordered == ["unmet_acceptance"]:
        return "repair", ordered
    return "blocked", ordered


#: The domain a step key is derived under. A new derivation is a new domain, never an edit of this
#: one: keys already in a ledger name steps under this derivation.
STEP_KEY_DOMAIN = "hee3.workflows.step-key/1"


def step_key(procedure, step_id, parent):
    """The idempotency key a step is submitted under: a UUIDv4 fixed by the procedure's identity
    and version, the step and the parent task (or None).

    Derived, never drawn: a composition that loses a reply and dispatches the same step again
    sends the same key, so the ledger's replay/conflict rule, not this module, is what stops a
    second effect. The first 16 bytes of SHA-256 over the canonical tuple, with the UUID version
    and variant bits set.
    """
    material = json.dumps(
        [STEP_KEY_DOMAIN, procedure["procedure_id"], procedure["procedure_version"],
         step_id, parent],
        separators=(",", ":"), ensure_ascii=False).encode()
    raw = bytearray(hashlib.sha256(material).digest()[:16])
    raw[6] = (raw[6] & 0x0F) | 0x40
    raw[8] = (raw[8] & 0x3F) | 0x80
    return str(uuid.UUID(bytes=bytes(raw)))


def _compact(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def dispatch(procedure, committed, step_id, held, spec=None, parent=None):
    """The `hee3` wrapper arguments that start `step_id` now (WF-11). Sends nothing.

    Only a step `resume()` offers may start, and only under an action the caller already holds
    (`held`: a grant's actions, or a skill packet's `actions_in_effect`) -- this can narrow what
    a procedure does, never widen it. Two actions have a dispatch arm: `task.submit`, under the
    step's derived key with the caller's spec, and `task.get`, which reads back the one
    `task.submit` step it depends on by that step's key. Every other action is refused by name
    (WF-13) rather than sent as a request no arm composes.
    """
    ready = resume(procedure, committed)
    if step_id not in ready:
        refuse("not_ready", f"{step_id}: resume offers {ready}")
    step = {s["step_id"]: s for s in procedure["steps"]}[step_id]
    action = step["action"]
    if action not in held:
        refuse("authority_widening",
               f"{step_id}: {action!r} is not among the actions the caller holds")
    if action == "task.submit":
        if not isinstance(spec, dict):
            refuse("undispatchable_action", f"{step_id}: task.submit needs the caller's task spec")
        return ["task.submit", "@idempotency_key=" + step_key(procedure, step_id, parent),
                "spec:=" + _compact(spec)]
    if action == "task.get":
        by_id = {s["step_id"]: s for s in procedure["steps"]}
        sources = [d for d in step["depends_on"] if by_id[d]["action"] == "task.submit"]
        if len(sources) != 1:
            refuse("undispatchable_action",
                   f"{step_id}: task.get reads back exactly one task.submit dependency; "
                   f"it has {len(sources)}")
        selector = {"source_action": "task.submit",
                    "idempotency_key": step_key(procedure, sources[0], parent)}
        return ["task.get", "selector:=" + _compact(selector), "evidence=none"]
    refuse("undispatchable_action", f"{step_id}: {action!r} has no dispatch arm")


def observe(procedure, committed, step_id, reply, parent=None):
    """A new committed record with `step_id`'s state read from the engine's `reply` (WF-11).

    A result is `done`; `effect_unknown` stays `effect_unknown`, so `resume()` will not repeat
    it; any other refusal is `failed`. A reply whose readback names another step's key answers
    a different request and is refused, as is anything that is not a control record. The record
    given is not changed.
    """
    steps = dict(_committed_steps(procedure, committed, set(validate(procedure))))
    if step_id not in {s["step_id"] for s in procedure["steps"]}:
        refuse("identity_mismatch", f"{step_id} is not a step of {procedure['procedure_id']!r}")
    if step_id in steps:
        refuse("not_ready", f"{step_id} is already committed as {steps[step_id]!r}")
    if not isinstance(reply, dict) or reply.get("kind") not in ("result", "error"):
        refuse("malformed_reply", f"{step_id}: the reply is not a control result or error")
    if reply["kind"] == "error" and not isinstance(reply.get("code"), str):
        refuse("malformed_reply", f"{step_id}: an error reply carries no code")
    if reply["kind"] == "result" and not isinstance(reply.get("body"), dict):
        refuse("malformed_reply", f"{step_id}: a result reply carries no body")
    readback = reply.get("readback")
    if isinstance(readback, dict):
        answered = readback.get("body", {}).get("selector", {}).get("idempotency_key")
        expected = step_key(procedure, step_id, parent)
        if answered is not None and answered != expected:
            refuse("identity_mismatch",
                   f"{step_id}: the reply answers key {answered}, not this step's {expected}")
    if reply["kind"] == "result":
        steps[step_id] = "done"
    elif reply["code"] == "effect_unknown":
        steps[step_id] = "effect_unknown"
    else:
        steps[step_id] = "failed"
    return {"procedure_id": procedure["procedure_id"],
            "procedure_version": procedure["procedure_version"], "steps": steps}


def main(arguments):
    """Validate one procedure file. The exit status is the verdict: 0 valid, 1 refused, 2 unread."""
    if len(arguments) != 1:
        print("usage: validate_procedure.py <procedure.json>", file=sys.stderr)
        return 2
    limit = bounds()["max_procedure_bytes"]
    try:
        with open(arguments[0], "rb") as handle:
            raw = handle.read(limit + 1)
    except OSError as error:
        print(f"unreadable: {error}", file=sys.stderr)
        return 2
    if len(raw) > limit:
        print(f"unreadable: {arguments[0]} exceeds the bound of {limit} bytes "
              f"(stopped after {len(raw)})", file=sys.stderr)
        return 2
    try:
        body = json.loads(raw)
    except (ValueError, RecursionError) as error:
        print(f"unreadable: {arguments[0]} is not JSON: {error}", file=sys.stderr)
        return 2
    try:
        order = validate(body)
    except ProcedureRefusal as refusal:
        print(f"refused {refusal}")
        return 1
    print("order=" + ",".join(order))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
