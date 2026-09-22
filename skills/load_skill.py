#!/usr/bin/env python3
"""Discover, load and revise skill packages under HEE3-IF-skills.

The schema states a manifest's shape. It cannot state that a path stays inside its package,
that a dependency is present, that a reference still hashes to what was reviewed, or that a
skill has not asked for authority its caller does not hold -- so those are obligations of
this loader, and each is named in the schema's `hee3.refusals`.

Two rules are structural rather than advisory:

* **A skill cannot widen authority.** `load()` takes the actions the caller ALREADY holds and
  refuses a manifest naming one outside that set. There is no branch that adds an action, so
  "a loaded skill granted itself a capability" is not a state this module can reach.
* **A reference that is not carried is named.** Scope denial, a budget cutoff, a stale hash
  and a traversal bound all produce an OMISSION carrying the reference id and the reason.
  A packet is never quietly short, because a short packet that looks complete is the failure
  that this module's whole bounding exists to make visible.

Nothing here executes instruction text, grants an action, accepts a task or changes a task's
criteria. Text is not execution approval.
"""

import ast
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA_PATH = HERE / "skill-v1.schema.json"


class SkillRefusal(Exception):
    """A skill was refused. `code` is one of the schema's `hee3.refusals`."""

    def __init__(self, code, detail):
        super().__init__(f"{code}: {detail}")
        self.code = code
        self.detail = detail


def refuse(code, detail):
    raise SkillRefusal(code, detail)


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


def safe_path(reference_id, path):
    """Refuse a reference path that could leave its package.

    The schema's pattern refuses a leading slash. Everything a pattern would express badly is
    here, where it can be read: a parent segment, an empty segment from a doubled or trailing
    separator, and a Windows drive or backslash that a forward-slash pattern would let past.
    """
    if "\\" in path or ":" in path:
        refuse("unsafe_path", f"{reference_id}: {path!r} is not a forward-slashed relative path")
    segments = path.split("/")
    if any(segment in ("", ".", "..") for segment in segments):
        refuse("unsafe_path",
               f"{reference_id}: {path!r} contains an empty, current or parent segment")
    return segments


def discover(manifest, held_actions, available_skills):
    """Whether this manifest is loadable here, refusing by symbol when it is not.

    `held_actions` is what the caller already holds; `available_skills` maps skill_id to the
    set of versions present. Returns the manifest's compatibility summary.
    """
    if manifest.get("schema") != "hee3.skills.skill.v1":
        refuse("version_mismatch", f"schema is {manifest.get('schema')!r}")
    if manifest["lifecycle"] == "retired":
        refuse("retired_skill",
               f"{manifest['skill_id']} version {manifest['skill_version']} is retired")

    # The admitted names come from this schema's own enum, which the generator took from
    # the action catalogue. Reading the catalogue again here would be a second door.
    admitted = set(schema()["properties"]["requires_actions"]["items"]["enum"])
    required = manifest.get("requires_actions", [])
    if len(required) > bounds()["max_actions"]:
        refuse("incompatible_action",
               f"{len(required)} required actions against a limit of {bounds()['max_actions']}")
    for action in required:
        if action not in admitted:
            refuse("unknown_action", f"{action!r} is not in the action catalogue")
        if action not in held_actions:
            # The refusal, never a grant: this is the only place a skill's action claim meets
            # the caller's authority, and it can only narrow.
            refuse("authority_widening",
                   f"{manifest['skill_id']} requires {action!r}, which the caller does not hold")

    dependencies = manifest.get("requires_skills", [])
    if len(dependencies) > bounds()["max_dependencies"]:
        refuse("missing_dependency",
               f"{len(dependencies)} dependencies against a limit of "
               f"{bounds()['max_dependencies']}")
    for dependency in dependencies:
        if not dependency.get("required", True):
            continue
        present = available_skills.get(dependency["skill_id"], set())
        if dependency["skill_version"] not in present:
            refuse("missing_dependency",
                   f"{dependency['skill_id']} version {dependency['skill_version']} is "
                   "required and not present")

    seen = set()
    for reference in manifest.get("references", []):
        identity = reference["reference_id"]
        if identity in seen:
            refuse("duplicate_reference", f"{identity} appears twice")
        seen.add(identity)
        segments = safe_path(identity, reference["path"])
        depth = reference.get("depth", len(segments) - 1)
        if depth > bounds()["max_depth"]:
            refuse("traversal_budget",
                   f"{identity}: depth {depth} against a limit of {bounds()['max_depth']}")

    return {
        "skill_id": manifest["skill_id"],
        "skill_version": manifest["skill_version"],
        "lifecycle": manifest["lifecycle"],
        "deprecated": manifest["lifecycle"] == "deprecated",
        "requires_actions": sorted(required),
        "reference_count": len(manifest.get("references", [])),
    }


def load(manifest, held_actions, available_skills, scopes, contents, budget_bytes):
    """Build a bounded instruction packet, stating everything it did not carry.

    `scopes` is what the caller may see; `contents` maps reference_id to bytes. The packet
    records provenance, omissions and cost, so a reader can tell a complete answer from a
    budgeted one without asking.
    """
    summary = discover(manifest, held_actions, available_skills)
    limit = min(budget_bytes, bounds()["max_packet_bytes"])
    entry = manifest["entry"].encode()
    if len(entry) > limit:
        # The entry is the skill; a packet without it would be a different skill that looks
        # like this one.
        refuse("context_budget",
               f"entry instructions are {len(entry)} bytes against a budget of {limit}")

    carried, omissions, used = [], [], len(entry)
    for reference in manifest.get("references", []):
        identity = reference["reference_id"]
        if not set(reference["scope"]) & set(scopes):
            omissions.append({"reference_id": identity, "reason": "denied_scope"})
            continue
        body = contents.get(identity)
        if body is None:
            omissions.append({"reference_id": identity, "reason": "stale_reference"})
            continue
        if len(body) > bounds()["max_reference_bytes"]:
            omissions.append({"reference_id": identity, "reason": "reference_too_large"})
            continue
        expected = reference.get("sha256")
        if expected is not None and hashlib.sha256(body).hexdigest() != expected:
            omissions.append({"reference_id": identity, "reason": "stale_reference"})
            continue
        if used + len(body) > limit:
            omissions.append({"reference_id": identity, "reason": "context_budget"})
            continue
        used += len(body)
        carried.append({"reference_id": identity, "path": reference["path"],
                        "bytes": len(body)})

    return {
        "schema": "hee3.skills.packet.v1",
        "skill_id": summary["skill_id"],
        "skill_version": summary["skill_version"],
        "deprecated": summary["deprecated"],
        # The packet repeats what the CALLER holds, not what the skill asked for. A reader
        # of a packet can never mistake it for a grant, because no wider set appears in it.
        "actions_in_effect": sorted(set(held_actions) & set(summary["requires_actions"])),
        "entry_bytes": len(entry),
        "references": carried,
        "omissions": omissions,
        "cost_bytes": used,
        "budget_bytes": limit,
        "complete": not omissions,
        "scope": "Instructions only. No grant, acceptance, criterion change or effect.",
    }


def revise(previous, proposed):
    """Record a revision, refusing one that is not a forward, immutable step.

    Returns the drift record a consumer needs: what changed, and which pins are affected.
    """
    if proposed["skill_id"] != previous["skill_id"]:
        refuse("version_mismatch",
               f"{proposed['skill_id']!r} is not a revision of {previous['skill_id']!r}")
    if proposed["skill_version"] <= previous["skill_version"]:
        refuse("version_mismatch",
               f"version {proposed['skill_version']} does not follow "
               f"{previous['skill_version']}; a published revision is never edited in place")
    if previous["lifecycle"] == "retired":
        refuse("retired_skill", f"{previous['skill_id']} is retired and takes no revision")
    before = {r["reference_id"]: r.get("sha256") for r in previous.get("references", [])}
    after = {r["reference_id"]: r.get("sha256") for r in proposed.get("references", [])}
    return {
        "skill_id": proposed["skill_id"],
        "from_version": previous["skill_version"],
        "to_version": proposed["skill_version"],
        "entry_changed": previous["entry"] != proposed["entry"],
        "references_added": sorted(set(after) - set(before)),
        "references_removed": sorted(set(before) - set(after)),
        "references_changed": sorted(k for k in set(before) & set(after)
                                     if before[k] != after[k]),
        "actions_added": sorted(set(proposed.get("requires_actions", []))
                                - set(previous.get("requires_actions", []))),
        "actions_removed": sorted(set(previous.get("requires_actions", []))
                                  - set(proposed.get("requires_actions", []))),
        "affected_consumers": "Every consumer pinning "
                              f"{previous['skill_id']} v{previous['skill_version']}",
    }
