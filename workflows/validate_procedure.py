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
import json
import sys
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
