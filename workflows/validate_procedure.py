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
"""

import ast
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA_PATH = HERE / "procedure-v1.schema.json"


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


def refusal_sites():
    """Every `refuse(...)` code this file can raise, read from its own source.

    Enumerated from the syntax tree rather than from a list an author maintains: a list would
    agree with the code only while somebody kept it agreeing, and the absent one is exactly
    the one a list cannot see.
    """
    tree = ast.parse(Path(__file__).read_text())
    found = []
    for node in ast.walk(tree):
        if (isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                and node.func.id == "refuse" and node.args
                and isinstance(node.args[0], ast.Constant)):
            found.append(node.args[0].value)
    return found


def validate(procedure):
    """Validate one procedure's structure and budget, returning a topological step order.

    The order is the deliverable: a caller that receives one has a graph proven acyclic and
    complete, which is not a state a caller can construct for itself.
    """
    limits = bounds()
    if procedure.get("schema") != "hee3.workflows.procedure.v1":
        refuse("stale_procedure_version", f"schema is {procedure.get('schema')!r}")
    steps = procedure.get("steps") or []
    if not 1 <= len(steps) <= limits["max_steps"]:
        refuse("fanout_exceeded",
               f"{len(steps)} steps against a limit of {limits['max_steps']}")
    admitted = set(schema()["$defs"]["step"]["properties"]["action"]["enum"])

    by_id = {}
    for step in steps:
        identity = step["step_id"]
        if identity in by_id:
            refuse("duplicate_step", f"step_id {identity!r} appears twice")
        by_id[identity] = step
        if step["action"] not in admitted:
            refuse("unknown_action", f"{identity}: action {step['action']!r} is not admitted")
        if step["action_version"] < 1:
            refuse("unsupported_action_version",
                   f"{identity}: action_version {step['action_version']}")
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


def resume(procedure, committed):
    """The steps a composition may start now, given what is already committed.

    `committed` maps step_id to one of the schema's `hee3.step_states`. Two rules are
    structural rather than advisory:

    * A step recorded `effect_unknown` is never returned. Its effect may already have landed,
      and repeating it is the harm the state exists to name. It is refused until a
      reconciliation record replaces the state.
    * A committed identity that this procedure does not contain is `identity_mismatch`, not a
      value to ignore: it means the record and the procedure are describing different things.
    """
    order = validate(procedure)
    known = set(order)
    for identity, state in committed.items():
        if identity not in known:
            refuse("identity_mismatch",
                   f"committed step {identity!r} is not in procedure "
                   f"{procedure['procedure_id']!r} version {procedure['procedure_version']}")
        if state not in schema()["hee3"]["step_states"]:
            refuse("identity_mismatch", f"{identity}: unknown committed state {state!r}")
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


def join(procedure, outcomes, verified):
    """Close a procedure, returning `(disposition, reasons)`.

    A completion CANDIDATE, never an acceptance: whether the parent is accepted is decided
    independently of this module, which is why no branch here returns `accepted`.
    """
    order = validate(procedure)
    by_id = {step["step_id"]: step for step in procedure["steps"]}
    reasons = []
    for identity in order:
        if by_id[identity].get("required", True):
            state = outcomes.get(identity)
            if state is None:
                reasons.append("missing_child")
            elif state == "effect_unknown":
                reasons.append("unreconciled_effect")
            elif state != "done":
                reasons.append("unmet_acceptance")
    if not verified:
        reasons.append("unmet_acceptance")
    ordered = [r for r in schema()["hee3"]["block_reasons"] if r in reasons]
    if not ordered:
        return "completion_candidate", []
    if ordered == ["unmet_acceptance"]:
        return "repair", ordered
    return "blocked", ordered
