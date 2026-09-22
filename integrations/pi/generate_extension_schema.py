#!/usr/bin/env python3
"""Author the HEE3-IF-pi_extension package contract; this is not an engine validator.

Source: `corpus/public-interfaces.json` → `pi_extension`, whose operations are *"Register
supported tools"*, *"Invoke and render"* and *"Cancel and reconcile lifecycle"*, bounded by
*"Host registration signatures remain unselected until host compatibility is qualified.
Reject stale handlers, unknown versions and mismatched call identities."*

**What is deliberately absent.** The host's own registration signatures are NOT modelled here,
because the contract says they are unselected until host compatibility is qualified, and a
schema that guessed them would be a fabricated claim about an external system. `host_binding`
is therefore a sealed, empty object: a package that tries to declare one is refused by name
rather than admitted against an invented shape. The refusal is `host_binding_unqualified`, and
it is the honest state of this integration until that qualification happens.

Everything the contract states WITHOUT reference to host signatures -- generations, stale
handlers, call identity, parallel calls, cancellation -- is modelled and checked.

Run without arguments to write the adjacent artifact, or `--check` to compare exact bytes.
"""

import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ARTIFACT = HERE / "pi-extension-v1.schema.json"
CATALOGUE = ROOT / "schemas/actions/control-v1.schema.json"

MAX_TOOLS = 64
MAX_PARALLEL_CALLS = 32
MAX_RENDER_BYTES = 131_072
MAX_IDENTIFIER = 128

CALL_STATES = ("pending", "running", "rendered", "failed", "cancelled", "effect_unknown")
REFUSALS = (
    "host_binding_unqualified", "unknown_bridge_version", "stale_generation",
    "unknown_call", "call_identity_mismatch", "duplicate_call", "unknown_action",
    "tool_limit", "parallel_call_limit", "render_limit", "terminal_call",
)


def catalogue_actions():
    defs = json.loads(CATALOGUE.read_text()).get("$defs", {})
    node = defs.get("ActionId")
    if isinstance(node, dict) and isinstance(node.get("enum"), list) and node["enum"]:
        return tuple(node["enum"])
    raise SystemExit(f"{CATALOGUE} does not expose an action enum; this contract will not "
                     "invent one")


def ident(description):
    return {"type": "string", "minLength": 1, "maxLength": MAX_IDENTIFIER,
            "pattern": "^[a-z0-9][a-z0-9._-]*$", "description": description}


def build():
    actions = catalogue_actions()
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://hee3.local/schemas/integrations/pi-extension-v1.schema.json",
        "title": "HEE3 Pi host extension package, version 1",
        "description": (
            "A versioned extension package describing which admitted actions it exposes as "
            "host tools, and under which bridge version and generation. It declares no host "
            "registration signature: those are unselected until host compatibility is "
            "qualified, and this document will not invent them."
        ),
        "type": "object",
        "additionalProperties": False,
        "required": ["schema", "extension_id", "extension_generation", "bridge_version", "tools"],
        "properties": {
            "schema": {"type": "string", "const": "hee3.integrations.pi_extension.v1"},
            "extension_id": ident("Stable identity of this extension."),
            "extension_generation": {
                "type": "integer", "minimum": 1,
                "description": (
                    "Bumped whenever the package is reloaded. A callback arriving under an "
                    "older generation is a STALE HANDLER: it is refused, never applied, "
                    "because a reload's whole purpose is that the previous registration no "
                    "longer speaks for this extension."
                ),
            },
            "bridge_version": {
                "type": "integer", "minimum": 1,
                "description": "The bridge contract this package was built against. Pinned exactly.",
            },
            "host_binding": {
                "type": "object", "additionalProperties": False, "properties": {},
                "description": (
                    "SEALED AND EMPTY, deliberately. The host's registration signatures are "
                    "unselected until host compatibility is qualified; a package declaring "
                    "one is refused as `host_binding_unqualified` rather than validated "
                    "against a shape nobody has confirmed."
                ),
            },
            "tools": {
                "type": "array", "minItems": 1, "maxItems": MAX_TOOLS, "uniqueItems": True,
                "items": {"$ref": "#/$defs/tool"},
            },
        },
        "$defs": {
            "tool": {
                "type": "object", "additionalProperties": False,
                "required": ["tool_id", "action", "action_version"],
                "properties": {
                    "tool_id": ident("Stable within this extension."),
                    "action": {
                        "type": "string", "enum": list(actions),
                        "description": (
                            "The admitted action this tool exposes. Read from the action "
                            "catalogue, never retyped: a tool cannot expose an action the "
                            "engine does not have."
                        ),
                    },
                    "action_version": {"type": "integer", "minimum": 1},
                    "summary": {"type": "string", "minLength": 1, "maxLength": 512},
                },
            },
        },
        "hee3": {
            "call_states": list(CALL_STATES),
            "refusals": list(REFUSALS),
            "bounds": {
                "max_tools": MAX_TOOLS, "max_parallel_calls": MAX_PARALLEL_CALLS,
                "max_render_bytes": MAX_RENDER_BYTES,
            },
            "action_catalogue": {"source": "schemas/actions/control-v1.schema.json",
                                 "count": len(actions)},
            "unqualified": [
                "Host registration signatures are unselected until host compatibility is "
                "qualified. No shape for them appears in this document.",
                "No host fixture is executed by this package; nothing here proves the "
                "extension loads in a real host.",
            ],
            "scope": (
                "Describes an extension package and reconciles call lifecycle. Registers "
                "nothing with a host, grants no action and admits no module."
            ),
        },
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    if parser.parse_args().check:
        if not ARTIFACT.exists():
            raise SystemExit(f"{ARTIFACT} is absent")
        if ARTIFACT.read_text() != json.dumps(build(), indent=2) + "\n":
            raise SystemExit(f"{ARTIFACT.name} differs from this generator")
        print(f"pi extension schema: matches_generator=yes actions={len(catalogue_actions())}")
        return
    ARTIFACT.write_text(json.dumps(build(), indent=2) + "\n")
    print(f"wrote {ARTIFACT.name}: actions={len(catalogue_actions())}")


main()
