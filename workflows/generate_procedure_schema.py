#!/usr/bin/env python3
"""Author the HEE3-IF-workflows finite procedure schema; this is not an engine validator.

Source: `corpus/public-interfaces.json` → `workflows`, whose three operations are *"Validate
procedure"*, *"Start or resume composition"* and *"Join and close"*, bounded by *"Bound
fan-out, retries, time and total resources; dynamic text cannot insert unapproved actions.
Refuse stale versions or missing steps."*

Run without arguments to write the adjacent artifact, or `--check` to compare exact bytes.
Only Python's standard library is needed. No transport or effects are started.

The action vocabulary is READ FROM `schemas/actions/control-v1.schema.json`, never retyped
here. A list typed in a second place is a promise that the two agree; taking it from the
artifact that decides it makes an action added there visible here without anyone remembering
(F147). `--check` therefore also fails when the catalogue changes, which is correct: a
procedure schema that admits a stale action vocabulary is the stale-version refusal this
module exists to make.
"""

import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
ARTIFACT = HERE / "procedure-v1.schema.json"
CATALOGUE = ROOT / "schemas/actions/control-v1.schema.json"

# Bounds. Each is a refusal point, and each is stated once so a procedure cannot be admitted
# by one limit and scheduled against another.
MAX_STEPS = 64
MAX_FANOUT = 16
MAX_DEPENDENCIES = 32
MAX_RETRIES = 8
MAX_TOTAL_SECONDS = 86_400
MAX_TOTAL_TOKENS = 10_000_000
MAX_IDENTIFIER = 128
MAX_TEXT = 4096
# The command line reads at most this many bytes of a procedure file before parsing it. The
# largest schema-valid procedure is about 1.1 MB unescaped (64 steps of notes and 32
# dependencies, 64 criteria of two texts); \u-escaping can grow text sixfold, so 8 MiB admits
# every valid procedure and still refuses a file before it is read whole.
MAX_PROCEDURE_BYTES = 8 * 1024 * 1024

STEP_STATES = (
    # `effect_unknown` is a state, not an error: a step whose service effect could not be
    # observed must be reconciled explicitly. Resume may not repeat it, and there is no
    # transition from it to `done` that does not pass through a reconciliation record.
    "planned", "assigned", "running", "blocked", "done", "failed",
    "cancelled", "effect_unknown",
)
DISPOSITIONS = ("completion_candidate", "repair", "blocked")
# Only reasons `join()` can emit. `dissent` and `stale_version` were declared here with no
# input that could produce them: a join against another procedure version is REFUSED as
# `stale_procedure_version`, not blocked, and this module takes no dissent input. A declared
# reason nothing emits is a promise to the reader that nothing keeps (WF-02).
BLOCK_REASONS = (
    "missing_child", "unmet_acceptance", "resource_exhausted", "unreconciled_effect",
)
REFUSALS = (
    "cycle", "missing_step", "duplicate_step", "unknown_action",
    "unsupported_action_version", "stale_procedure_version", "fanout_exceeded",
    "dependency_limit", "retry_limit", "time_limit", "resource_limit",
    "unreconciled_effect", "identity_mismatch", "duplicate_criterion",
    "malformed_procedure",
    # WF-11: dispatch() and observe(), the request builder and readback mapper.
    "not_ready", "authority_widening", "undispatchable_action", "malformed_reply",
)


def read_catalogue():
    """The action catalogue document, read once and handed to whatever derives from it."""
    return json.loads(CATALOGUE.read_text())


def catalogue_actions(schema):
    """The admitted action names, taken from the catalogue document that defines them."""
    defs = schema.get("$defs", {})
    node = defs.get("ActionId")
    if isinstance(node, dict) and isinstance(node.get("enum"), list) and node["enum"]:
        return tuple(node["enum"])
    # Degrade to a narrower world rather than an empty one: refuse instead of admitting
    # every string, which is what an empty enum would mean here.
    raise SystemExit(
        f"{CATALOGUE} does not expose an action enum; the procedure schema will not "
        "invent one, because a procedure that admits any action name is the hole this "
        "schema exists to close"
    )


def catalogue_action_versions(schema, actions):
    """The version the catalogue serves of each admitted action, from its request definition.

    Every request definition pins `action` and `action_version` as constants. The map is read
    from those rather than assumed to be 1, and an admitted action with no pinned version, or
    with two that disagree, stops the generator instead of being guessed. `schema` is the
    catalogue document, passed in rather than re-read, so both guards are reachable by
    choosing an argument.
    """
    versions = {}
    for definition in schema.get("$defs", {}).values():
        properties = definition.get("properties", {}) if isinstance(definition, dict) else {}
        action = properties.get("action", {}).get("const")
        version = properties.get("action_version", {}).get("const")
        if action is None or version is None:
            continue
        if versions.setdefault(action, version) != version:
            raise SystemExit(f"{CATALOGUE} pins {action!r} at two versions")
    missing = [action for action in actions if action not in versions]
    if missing:
        raise SystemExit(f"{CATALOGUE} pins no action_version for {missing}")
    return {action: versions[action] for action in actions}


def ident(description):
    return {
        "type": "string", "minLength": 1, "maxLength": MAX_IDENTIFIER,
        "pattern": "^[a-z0-9][a-z0-9._-]*$", "description": description,
    }


def text(description, maximum=MAX_TEXT):
    return {"type": "string", "minLength": 1, "maxLength": maximum, "description": description}


def count(maximum, description):
    return {"type": "integer", "minimum": 0, "maximum": maximum, "description": description}


def build(catalogue):
    actions = catalogue_actions(catalogue)
    versions = catalogue_action_versions(catalogue, actions)
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://hee3.local/schemas/workflows/procedure-v1.schema.json",
        "title": "HEE3 finite procedure, version 1",
        "description": (
            "A finite, versioned procedure over already-admitted actions. The schema "
            "describes a procedure; it grants nothing, schedules nothing, and admits no "
            "module. Final acceptance of a parent remains a separate, independent decision."
        ),
        "type": "object",
        "additionalProperties": False,
        "required": [
            "schema", "procedure_id", "procedure_version", "budget", "steps", "acceptance",
        ],
        "properties": {
            "schema": {"type": "string", "const": "hee3.workflows.procedure.v1"},
            "procedure_id": ident("Stable identity of this procedure across revisions."),
            "procedure_version": {
                "type": "integer", "minimum": 1,
                "description": (
                    "Monotonic revision. A composition carries the version it started "
                    "under; a join against a different version is refused as "
                    "`stale_procedure_version` rather than silently re-planned."
                ),
            },
            "purpose": text("What this procedure is for, in the author's words."),
            "budget": {"$ref": "#/$defs/budget"},
            "steps": {
                "type": "array", "minItems": 1, "maxItems": MAX_STEPS,
                "items": {"$ref": "#/$defs/step"},
                "description": (
                    f"At most {MAX_STEPS} steps. Finite by construction: there is no loop "
                    "or expansion form, so the step count is known before anything runs."
                ),
            },
            "acceptance": {
                "type": "array", "minItems": 1, "maxItems": MAX_STEPS,
                "items": {"$ref": "#/$defs/criterion"},
                "description": (
                    "What the parent must show at close. Criteria are stated before the "
                    "procedure runs, so a failing run cannot be closed by weakening them."
                ),
            },
        },
        "$defs": {
            "budget": {
                "type": "object", "additionalProperties": False,
                "required": ["max_fanout", "max_seconds", "max_tokens"],
                "properties": {
                    "max_fanout": {
                        "type": "integer", "minimum": 1, "maximum": MAX_FANOUT,
                        "description": (
                            f"Most steps runnable at once, at most {MAX_FANOUT}. The bound "
                            "is on the procedure, not on one wave of it."
                        ),
                    },
                    "max_seconds": count(MAX_TOTAL_SECONDS, "Wall-clock budget for the whole procedure."),
                    "max_tokens": count(MAX_TOTAL_TOKENS, "Usage budget for the whole procedure."),
                },
            },
            "step": {
                "type": "object", "additionalProperties": False,
                "required": ["step_id", "action", "action_version", "retry"],
                "properties": {
                    "step_id": ident("Stable within the procedure; a composition commits it."),
                    "action": {
                        "type": "string", "enum": list(actions),
                        "description": (
                            "One already-admitted action. The closed enum is the mechanism "
                            "behind *dynamic text cannot insert unapproved actions*: a name "
                            "not in the catalogue cannot be spelled here at all."
                        ),
                    },
                    "action_version": {
                        "type": "integer", "minimum": 1,
                        "description": (
                            "Must equal the version the catalogue serves of this action, "
                            "listed under `hee3.action_versions`; any other pin is "
                            "`unsupported_action_version` here rather than a refusal at the "
                            "receiver."
                        ),
                    },
                    "depends_on": {
                        "type": "array", "maxItems": MAX_DEPENDENCIES, "uniqueItems": True,
                        "items": ident("Another step_id in this procedure."),
                        "description": (
                            "Every entry must name a step present in this procedure; a "
                            "dangling name is `missing_step`, and a cycle is `cycle`. "
                            "Neither is representable as a schema constraint, so both are "
                            "validator obligations listed under `refusals`."
                        ),
                    },
                    "required": {
                        "type": "boolean", "default": True,
                        "description": (
                            "Whether the join may close without this step's outcome. An "
                            "optional step may be absent or failed; one in `effect_unknown` "
                            "still blocks the join, because optional means the parent can "
                            "close without the effect, not over an unresolved one."
                        ),
                    },
                    "retry": {
                        "type": "object", "additionalProperties": False,
                        "required": ["max_attempts"],
                        "properties": {
                            "max_attempts": {
                                "type": "integer", "minimum": 1, "maximum": MAX_RETRIES,
                                "description": (
                                    f"Total attempts, at most {MAX_RETRIES}. A step whose "
                                    "effect could not be observed is never retried by this "
                                    "count: it becomes `effect_unknown` and waits for a "
                                    "reconciliation record."
                                ),
                            },
                            "retry_on": {
                                "type": "array", "uniqueItems": True, "maxItems": 8,
                                "items": {"type": "string", "enum": [
                                    "unavailable", "deadline_exceeded", "resource_exhausted",
                                    "conflict", "internal",
                                ]},
                                "description": (
                                    "Only these are retryable. `effect_unknown` is "
                                    "deliberately absent: repeating an effect that may "
                                    "already have landed is the harm this bound exists for."
                                ),
                            },
                        },
                    },
                    "notes": text("Author's notes. Never read as an instruction or a grant."),
                },
            },
            "criterion": {
                "type": "object", "additionalProperties": False,
                "required": ["criterion_id", "statement"],
                "properties": {
                    "criterion_id": ident("Stable identity of this acceptance criterion."),
                    "statement": text("What must be shown, stated so a reader can check it."),
                    "evidence": text("Where the evidence for it is to be found."),
                },
            },
        },
        "hee3": {
            "step_states": list(STEP_STATES),
            "dispositions": list(DISPOSITIONS),
            "block_reasons": list(BLOCK_REASONS),
            "refusals": list(REFUSALS),
            "bounds": {
                "max_steps": MAX_STEPS, "max_fanout": MAX_FANOUT,
                "max_dependencies": MAX_DEPENDENCIES, "max_retries": MAX_RETRIES,
                "max_total_seconds": MAX_TOTAL_SECONDS, "max_total_tokens": MAX_TOTAL_TOKENS,
                "max_procedure_bytes": MAX_PROCEDURE_BYTES,
            },
            "action_versions": versions,
            "action_catalogue": {
                "source": "schemas/actions/control-v1.schema.json",
                "count": len(actions),
            },
            "scope": (
                "Describes a procedure. Starts no scheduler, grants no action, accepts no "
                "parent and admits no module."
            ),
        },
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true",
                        help="compare the adjacent artifact byte-for-byte instead of writing it")
    arguments = parser.parse_args()
    catalogue = read_catalogue()
    actions = catalogue_actions(catalogue)
    text_out = json.dumps(build(catalogue), indent=2) + "\n"
    if arguments.check:
        if not ARTIFACT.exists():
            raise SystemExit(f"{ARTIFACT} is absent")
        actual = ARTIFACT.read_text()
        if actual != text_out:
            raise SystemExit(
                f"{ARTIFACT.name} differs from this generator by "
                f"{sum(1 for a, b in zip(actual, text_out, strict=False) if a != b) + abs(len(actual) - len(text_out))} "
                "characters; regenerate it or fix the generator"
            )
        print(f"procedure schema: matches_generator=yes actions={len(actions)}")
        return
    ARTIFACT.write_text(text_out)
    print(f"wrote {ARTIFACT.name}: bytes={len(text_out.encode())} actions={len(actions)}")


if __name__ == "__main__":
    main()
