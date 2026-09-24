#!/usr/bin/env python3
"""Register tools and reconcile call lifecycle for HEE3-IF-pi_extension.

The half of this integration that does NOT depend on host registration signatures. Those are
unselected until host compatibility is qualified, so a package declaring one is refused by
name (`host_binding_unqualified`) rather than validated against an invented shape, and no host
fixture is executed anywhere here.

What IS decided, because the contract states it without reference to a host signature:

* **A stale handler cannot return a result.** Every call carries the generation it was opened
  under. After a reload the generation advances, and a callback arriving under the old one is
  refused. A reload's whole purpose is that the previous registration no longer speaks for the
  extension.
* **A parallel call cannot return another call's success.** A result is applied only when the
  call id, the tool id and the generation all match what was recorded at open. Two calls to
  one tool have separate identities and separate limits.
* **A cancel race has one winner.** A call reaches a terminal state once; a second transition
  is refused, so a result racing a cancellation cannot overwrite the cancellation, and a
  cancellation racing a result cannot erase the result.
* **A cancellation request is acknowledged, not final.** The host's signal records the
  acknowledgement on the same call and leaves it open; the call settles on the engine's
  disposition -- a result, a failure, or a confirmed cancellation.
* **A reload revokes; it does not forget.** Calls open at a reload keep the package/protocol
  tuple they were admitted under, are never settled by a later handler, and their ids cannot
  be reused. An unobserved effect stays an obligation across the reload.

Nothing here registers with a host, invokes an action, grants an action or admits a module.
"""

import ast
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA_PATH = HERE / "pi-extension-v1.schema.json"

class ExtensionRefusal(Exception):
    """An extension operation was refused. `code` is one of the schema's `hee3.refusals`."""

    def __init__(self, code, detail):
        super().__init__(f"{code}: {detail}")
        self.code = code
        self.detail = detail


def refuse(code, detail):
    raise ExtensionRefusal(code, detail)


def schema():
    return json.loads(SCHEMA_PATH.read_text())


def bounds():
    return schema()["hee3"]["bounds"]


def refusal_sites():
    """Every `refuse(...)` code this file can raise, read from its own syntax tree."""
    return [
        node.args[0].value
        for node in ast.walk(ast.parse(Path(__file__).read_text()))
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
        and node.func.id == "refuse" and node.args and isinstance(node.args[0], ast.Constant)
    ]


def assigned_states():
    """Every call state this file assigns, read from its own syntax tree.

    A `"state": <constant>` entry in a dict literal or a `record["state"] = <constant>`
    assignment. Compared with the schema's `call_states`, so a declared state nothing
    produces cannot stand as vocabulary.
    """
    found = []
    for node in ast.walk(ast.parse(Path(__file__).read_text())):
        if isinstance(node, ast.Dict):
            found += [value.value for key, value in zip(node.keys, node.values, strict=True)
                      if isinstance(key, ast.Constant) and key.value == "state"
                      and isinstance(value, ast.Constant)]
        elif isinstance(node, ast.Assign) and isinstance(node.value, ast.Constant):
            found += [node.value.value for target in node.targets
                      if isinstance(target, ast.Subscript)
                      and isinstance(target.slice, ast.Constant) and target.slice.value == "state"]
        elif isinstance(node, ast.keyword) and node.arg == "state" \
                and isinstance(node.value, ast.Constant):
            found.append(node.value.value)
    return found


JSON_TYPES = {"string": str, "integer": int, "array": list, "object": dict}


def _shape_problem(value, spec, where):
    """The first structural rule `value` breaks against an object schema `spec`, or None.

    A hand check of the few structural rules -- object, required keys, no undeclared keys,
    the declared JSON type of each present key -- read from the published schema itself, so
    the contract and this check cannot disagree. `jsonschema` stays a test-only dependency.
    """
    if not isinstance(value, dict):
        return f"{where} is not an object"
    missing = [key for key in spec["required"] if key not in value]
    if missing:
        return f"{where} is missing {', '.join(missing)}"
    unknown = sorted(set(value) - set(spec["properties"]))
    if unknown:
        return f"{where} has undeclared key {', '.join(unknown)}"
    for key, item in value.items():
        expected = JSON_TYPES.get(spec["properties"][key].get("type"))
        # `bool` is an `int` in Python; JSON's `true` is not an integer.
        if expected is not None and (not isinstance(item, expected) or isinstance(item, bool)):
            return f"{where}.{key} is not {spec['properties'][key]['type']}"
    return None


def _malformed(package):
    contract = schema()
    problem = _shape_problem(package, contract, "package")
    if problem is None and len(package["tools"]) < contract["properties"]["tools"]["minItems"]:
        problem = "package.tools is empty; a package exposes at least one tool"
    for index, tool in enumerate(package["tools"] if problem is None else ()):
        problem = problem or _shape_problem(tool, contract["$defs"]["tool"], f"tools[{index}]")
    return problem


def register(package, supported_bridge_versions):
    """Admit a package's tool definitions, or refuse them visibly.

    Returns the tool definitions a host COULD be offered. It does not offer them: that step
    needs the registration signature this integration does not have.
    """
    problem = _malformed(package)
    if problem is not None:
        refuse("malformed_package", problem)
    if package.get("schema") != "hee3.integrations.pi_extension.v1":
        refuse("unknown_bridge_version", f"schema is {package.get('schema')!r}")
    if package["bridge_version"] not in supported_bridge_versions:
        refuse("unknown_bridge_version",
               f"bridge version {package['bridge_version']} is not among "
               f"{sorted(supported_bridge_versions)}")
    if package.get("host_binding"):
        refuse("host_binding_unqualified",
               "this package declares a host binding; host registration signatures are "
               "unselected until host compatibility is qualified")
    tools = package["tools"]
    if len(tools) > bounds()["max_tools"]:
        refuse("tool_limit", f"{len(tools)} tools against a limit of {bounds()['max_tools']}")
    admitted = set(schema()["$defs"]["tool"]["properties"]["action"]["enum"])
    versions = schema()["hee3"]["action_versions"]
    seen = set()
    for tool in tools:
        if tool["tool_id"] in seen:
            refuse("duplicate_call", f"tool {tool['tool_id']!r} is defined twice")
        seen.add(tool["tool_id"])
        if tool["action"] not in admitted:
            refuse("unknown_action",
                   f"{tool['tool_id']}: {tool['action']!r} is not in the action catalogue")
        if tool["action_version"] != versions[tool["action"]]:
            refuse("unknown_action_version",
                   f"{tool['tool_id']}: {tool['action']} version {tool['action_version']} is "
                   f"not the catalogue's version {versions[tool['action']]}")
    return {
        "extension_id": package["extension_id"],
        "generation": package["extension_generation"],
        "bridge_version": package["bridge_version"],
        "tools": [{"tool_id": t["tool_id"], "action": t["action"],
                   "action_version": t["action_version"]} for t in tools],
        "registered_with_host": False,
        "reason_not_registered": "Host registration signatures are unqualified.",
    }


class Calls:
    """The open calls of one extension, and their outcomes.

    Built from `register()`'s admitted result, never from a bare generation: every call is
    pinned at open to the admitted tuple (extension, generation, bridge version, its tool's
    action version), and a reload takes the next admitted result.
    """

    def __init__(self, admitted):
        self.open = {}
        self.settled = {}
        self.revoked = {}
        self._admit(admitted)

    def _admit(self, admitted):
        self.extension_id = admitted["extension_id"]
        self.generation = admitted["generation"]
        self.bridge_version = admitted["bridge_version"]
        self.tools = {tool["tool_id"]: tool for tool in admitted["tools"]}

    def reload(self, admitted):
        """Advance to the next admitted generation, revoking the previous one's calls.

        A revoked call keeps its admitted tuple and its state -- `running` or
        `effect_unknown` -- and is never settled here: no later handler speaks for it, and a
        reload is not a reconciliation. Returns the ids it revoked.
        """
        if admitted["generation"] <= self.generation:
            refuse("stale_generation",
                   f"generation {admitted['generation']} does not follow {self.generation}")
        abandoned = sorted(self.open)
        self.revoked.update(self.open)
        self.open = {}
        self._admit(admitted)
        return abandoned

    def opened(self, call_id, tool_id):
        for where, calls in (("open", self.open), ("settled", self.settled),
                             ("revoked", self.revoked)):
            if call_id in calls:
                refuse("duplicate_call", f"call {call_id!r} is already {where}")
        tool = self.tools.get(tool_id)
        if tool is None:
            refuse("unknown_tool",
                   f"tool {tool_id!r} is not admitted at generation {self.generation}")
        if len(self.open) >= bounds()["max_parallel_calls"]:
            refuse("parallel_call_limit",
                   f"{len(self.open)} calls open against a limit of "
                   f"{bounds()['max_parallel_calls']}")
        self.open[call_id] = {"tool_id": tool_id, "extension_id": self.extension_id,
                              "generation": self.generation,
                              "bridge_version": self.bridge_version,
                              "action_version": tool["action_version"], "state": "running"}
        return dict(self.open[call_id])

    def _matched(self, call_id, tool_id, generation):
        if call_id in self.settled:
            refuse("terminal_call",
                   f"call {call_id!r} already settled as {self.settled[call_id]['state']!r}")
        if call_id in self.revoked:
            refuse("stale_generation",
                   f"call {call_id!r} was revoked by a reload from generation "
                   f"{self.revoked[call_id]['generation']}; no handler speaks for it now")
        record = self.open.get(call_id)
        if record is None:
            refuse("unknown_call", f"call {call_id!r} is not open")
        if record["generation"] != generation:
            refuse("stale_generation",
                   f"call {call_id!r} was opened under generation {record['generation']}, "
                   f"not {generation}")
        if record["tool_id"] != tool_id:
            refuse("call_identity_mismatch",
                   f"call {call_id!r} belongs to tool {record['tool_id']!r}, not {tool_id!r}")
        return record

    def _settle(self, call_id, **outcome):
        record = self.open.pop(call_id)
        record.update(outcome)
        self.settled[call_id] = record
        return dict(record)

    def render(self, call_id, tool_id, generation, body):
        """Settle a call with a rendered result, refusing anything that is not truly its own."""
        self._matched(call_id, tool_id, generation)
        if len(body) > bounds()["max_render_bytes"]:
            refuse("render_limit",
                   f"call {call_id!r} rendered {len(body)} bytes against a limit of "
                   f"{bounds()['max_render_bytes']}")
        return self._settle(call_id, state="rendered", bytes=len(body))

    def fail(self, call_id, tool_id, generation, error_code):
        """Settle a call with the action's error, named in the control-v1 error vocabulary."""
        self._matched(call_id, tool_id, generation)
        if error_code not in schema()["hee3"]["error_codes"]:
            refuse("unknown_error_code",
                   f"call {call_id!r} failed with {error_code!r}, which is not a control-v1 "
                   "error code")
        return self._settle(call_id, state="failed", error_code=error_code)

    def cancel_requested(self, call_id, tool_id, generation):
        """Acknowledge the host's cancellation signal on the same call, without settling it.

        The call stays open until the engine's disposition arrives: a result, a failure or a
        confirmed cancellation. An effectful action may complete anyway, and an
        acknowledgement presented as final would hide that. An `effect_unknown` call keeps
        that state; the request is recorded beside it, never over it.
        """
        record = self._matched(call_id, tool_id, generation)
        record["cancel_acknowledged"] = True
        if record["state"] == "running":
            record["state"] = "cancellation_requested"
        return dict(record)

    def cancel(self, call_id, tool_id, generation):
        """Settle a call on the engine's confirmed cancellation. The first terminal wins."""
        self._matched(call_id, tool_id, generation)
        return self._settle(call_id, state="cancelled")

    def unknown_effect(self, call_id, tool_id, generation):
        """Record that a call's effect could not be observed.

        Not terminal: an unobserved effect is reconciled, never guessed at. It stays open so
        that nothing reports it as done, and it is not retried.
        """
        record = self._matched(call_id, tool_id, generation)
        record["state"] = "effect_unknown"
        return dict(record)
