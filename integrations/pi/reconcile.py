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

Nothing here registers with a host, invokes an action, grants an action or admits a module.
"""

import ast
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA_PATH = HERE / "pi-extension-v1.schema.json"

TERMINAL = ("rendered", "failed", "cancelled")


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


def register(package, supported_bridge_versions):
    """Admit a package's tool definitions, or refuse them visibly.

    Returns the tool definitions a host COULD be offered. It does not offer them: that step
    needs the registration signature this integration does not have.
    """
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
    seen = set()
    for tool in tools:
        if tool["tool_id"] in seen:
            refuse("duplicate_call", f"tool {tool['tool_id']!r} is defined twice")
        seen.add(tool["tool_id"])
        if tool["action"] not in admitted:
            refuse("unknown_action",
                   f"{tool['tool_id']}: {tool['action']!r} is not in the action catalogue")
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
    """The open calls of one extension generation, and their terminal outcomes."""

    def __init__(self, generation):
        self.generation = generation
        self.open = {}
        self.settled = {}

    def reload(self, generation):
        """Advance to a new generation, abandoning callbacks from the previous one."""
        if generation <= self.generation:
            refuse("stale_generation",
                   f"generation {generation} does not follow {self.generation}")
        abandoned = sorted(self.open)
        self.generation = generation
        self.open = {}
        return abandoned

    def opened(self, call_id, tool_id):
        if call_id in self.open or call_id in self.settled:
            refuse("duplicate_call", f"call {call_id!r} is already open or settled")
        if len(self.open) >= bounds()["max_parallel_calls"]:
            refuse("parallel_call_limit",
                   f"{len(self.open)} calls open against a limit of "
                   f"{bounds()['max_parallel_calls']}")
        self.open[call_id] = {"tool_id": tool_id, "generation": self.generation,
                              "state": "running"}
        return dict(self.open[call_id])

    def _matched(self, call_id, tool_id, generation):
        if call_id in self.settled:
            refuse("terminal_call",
                   f"call {call_id!r} already settled as {self.settled[call_id]['state']!r}")
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

    def render(self, call_id, tool_id, generation, body):
        """Settle a call with a rendered result, refusing anything that is not truly its own."""
        self._matched(call_id, tool_id, generation)
        if len(body) > bounds()["max_render_bytes"]:
            refuse("render_limit",
                   f"call {call_id!r} rendered {len(body)} bytes against a limit of "
                   f"{bounds()['max_render_bytes']}")
        del self.open[call_id]
        self.settled[call_id] = {"tool_id": tool_id, "generation": generation,
                                 "state": "rendered", "bytes": len(body)}
        return dict(self.settled[call_id])

    def cancel(self, call_id, tool_id, generation):
        """Settle a call as cancelled. The first terminal transition wins."""
        self._matched(call_id, tool_id, generation)
        del self.open[call_id]
        self.settled[call_id] = {"tool_id": tool_id, "generation": generation,
                                 "state": "cancelled"}
        return dict(self.settled[call_id])

    def unknown_effect(self, call_id, tool_id, generation):
        """Record that a call's effect could not be observed.

        Not terminal: an unobserved effect is reconciled, never guessed at. It stays open so
        that nothing reports it as done, and it is not retried.
        """
        record = self._matched(call_id, tool_id, generation)
        record["state"] = "effect_unknown"
        return dict(record)
