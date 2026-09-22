#!/usr/bin/env python3
"""HEE3-IF-skills manifest schema and loader tests.

Run: python3 -W error tests/skill_schema.py
Dependency: installed jsonschema (test tooling only). The schema author also implemented
these tests; independent oracle authorship is NOT claimed. Tests, subtests and fixture counts
confer no module case credits. No transport, dispatch, grant or admission is tested.

`tools/check-skill-sites` neuters each refusal site in `skills/load_skill.py` and requires
this file to notice. A case per refusal CODE would not do: the loader raises ten codes from
fifteen places, so five have a neighbour that could answer for them.
"""

import copy
import hashlib
import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "skills/skill-v1.schema.json"
EXAMPLE = ROOT / "skills/examples/receipt-reading/skill.json"

_spec = importlib.util.spec_from_file_location("hee3_load_skill", ROOT / "skills/load_skill.py")
LS = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(LS)

SCHEMA = json.loads(SCHEMA_PATH.read_text())
BOUNDS = SCHEMA["hee3"]["bounds"]
HELD = {"task.get", "task.list", "health", "task.submit"}
BODY = (ROOT / "skills/examples/receipt-reading/references/reading-a-receipt.md").read_bytes()
EXERCISED_CODES = set()


def base():
    return json.loads(EXAMPLE.read_text())


def minimal(**overrides):
    body = {
        "schema": "hee3.skills.skill.v1",
        "skill_id": "case",
        "skill_version": 1,
        "lifecycle": "published",
        "purpose": "A minimal manifest for one case.",
        "entry": "Do the checkable thing.",
    }
    body.update(overrides)
    return body


def reference(identity="r1", path="references/a.md", scope=("reviewer",), **extra):
    body = {"reference_id": identity, "path": path, "scope": list(scope)}
    body.update(extra)
    return body


class SchemaShape(unittest.TestCase):
    def test_schema_is_itself_valid(self):
        Draft202012Validator.check_schema(SCHEMA)

    def test_generator_reproduces_the_artifact(self):
        result = subprocess.run(
            [sys.executable, str(ROOT / "skills/generate_skill_schema.py"), "--check"],
            capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_action_enum_is_the_action_catalogue(self):
        catalogue = json.loads((ROOT / "schemas/actions/control-v1.schema.json").read_text())
        self.assertEqual(SCHEMA["properties"]["requires_actions"]["items"]["enum"],
                         catalogue["$defs"]["ActionId"]["enum"])

    def test_reference_example_validates(self):
        Draft202012Validator(SCHEMA).validate(base())

    def test_absolute_path_is_unspellable(self):
        body = minimal(references=[reference(path="/etc/passwd")])
        with self.assertRaises(Exception):
            Draft202012Validator(SCHEMA).validate(body)

    def test_unknown_manifest_field_is_refused(self):
        with self.assertRaises(Exception):
            Draft202012Validator(SCHEMA).validate(minimal(run="curl example.invalid"))

    def test_scope_denial_is_an_omission_not_a_refusal(self):
        self.assertIn("denied_scope", SCHEMA["hee3"]["omission_reasons"])
        self.assertNotIn("denied_scope", SCHEMA["hee3"]["refusals"])

    def test_stale_reference_is_an_omission_not_a_refusal(self):
        self.assertIn("stale_reference", SCHEMA["hee3"]["omission_reasons"])
        self.assertNotIn("stale_reference", SCHEMA["hee3"]["refusals"])

    def test_dependency_version_is_exact(self):
        pinned = SCHEMA["$defs"]["dependency"]["properties"]["skill_version"]
        self.assertEqual(pinned["type"], "integer")
        self.assertNotIn("maximum", pinned)


class Refusals(unittest.TestCase):
    def refused(self, code, call):
        EXERCISED_CODES.add(code)
        with self.assertRaises(LS.SkillRefusal) as caught:
            call()
        self.assertEqual(caught.exception.code, code, caught.exception.detail)
        return caught.exception.detail

    def test_reference_example_discovers(self):
        summary = LS.discover(base(), HELD, {})
        self.assertEqual(summary["skill_version"], 2)
        self.assertFalse(summary["deprecated"])

    def test_wrong_schema_string(self):
        self.refused("version_mismatch",
                     lambda: LS.discover(minimal(schema="hee3.skills.skill.v2"), HELD, {}))

    def test_retired_skill_does_not_load(self):
        self.refused("retired_skill",
                     lambda: LS.discover(minimal(lifecycle="retired"), HELD, {}))

    def test_more_required_actions_than_the_limit(self):
        actions = SCHEMA["properties"]["requires_actions"]["items"]["enum"]
        many = (actions * 4)[: BOUNDS["max_actions"] + 1]
        self.refused("incompatible_action",
                     lambda: LS.discover(minimal(requires_actions=many), HELD, {}))

    def test_action_outside_the_catalogue(self):
        self.refused("unknown_action",
                     lambda: LS.discover(minimal(requires_actions=["task.forge"]), HELD, {}))

    def test_action_the_caller_does_not_hold(self):
        detail = self.refused(
            "authority_widening",
            lambda: LS.discover(minimal(requires_actions=["roster.disable"]), HELD, {}))
        self.assertIn("roster.disable", detail)

    def test_more_dependencies_than_the_limit(self):
        # Every dependency is PRESENT, so the per-dependency check below cannot answer this
        # case: only the count bound can. Built the other way round it passed while the
        # bound was neutered, which is what the site sweep is for.
        count = BOUNDS["max_dependencies"] + 1
        many = [{"skill_id": f"d{i}", "skill_version": 1} for i in range(count)]
        available = {f"d{i}": {1} for i in range(count)}
        detail = self.refused("missing_dependency",
                              lambda: LS.discover(minimal(requires_skills=many), HELD, available))
        self.assertIn(str(BOUNDS["max_dependencies"]), detail)

    def test_required_dependency_absent(self):
        body = minimal(requires_skills=[{"skill_id": "other", "skill_version": 3}])
        detail = self.refused("missing_dependency", lambda: LS.discover(body, HELD, {}))
        self.assertIn("other", detail)

    def test_required_dependency_at_a_different_version(self):
        body = minimal(requires_skills=[{"skill_id": "other", "skill_version": 3}])
        self.refused("missing_dependency",
                     lambda: LS.discover(body, HELD, {"other": {1, 2}}))

    def test_duplicate_reference_identity(self):
        body = minimal(references=[reference("r1"), reference("r1", path="references/b.md")])
        self.refused("duplicate_reference", lambda: LS.discover(body, HELD, {}))

    def test_backslash_in_a_path(self):
        body = minimal(references=[reference(path="references\\a.md")])
        self.refused("unsafe_path", lambda: LS.discover(body, HELD, {}))

    def test_parent_segment_in_a_path(self):
        body = minimal(references=[reference(path="references/../../etc/passwd")])
        detail = self.refused("unsafe_path", lambda: LS.discover(body, HELD, {}))
        self.assertIn("parent segment", detail)

    def test_doubled_separator_in_a_path(self):
        body = minimal(references=[reference(path="references//a.md")])
        self.refused("unsafe_path", lambda: LS.discover(body, HELD, {}))

    def test_depth_above_the_limit(self):
        body = minimal(references=[reference(depth=BOUNDS["max_depth"] + 1)])
        self.refused("traversal_budget", lambda: LS.discover(body, HELD, {}))

    def test_entry_larger_than_the_budget(self):
        body = minimal(entry="x" * 500)
        self.refused("context_budget", lambda: LS.load(body, HELD, {}, ["reviewer"], {}, 100))

    def test_revision_of_a_different_skill(self):
        self.refused("version_mismatch",
                     lambda: LS.revise(minimal(skill_id="a"), minimal(skill_id="b",
                                                                     skill_version=2)))

    def test_revision_that_does_not_advance(self):
        self.refused("version_mismatch",
                     lambda: LS.revise(minimal(skill_version=3), minimal(skill_version=3)))

    def test_revision_of_a_retired_skill(self):
        self.refused("retired_skill",
                     lambda: LS.revise(minimal(lifecycle="retired"), minimal(skill_version=2)))


class Packets(unittest.TestCase):
    def packet(self, scopes=("reviewer",), contents=None, budget=100_000, manifest=None):
        return LS.load(manifest or base(), HELD, {}, list(scopes),
                       {"reading-a-receipt": BODY} if contents is None else contents, budget)

    def test_a_carried_reference_is_listed_with_its_size(self):
        carried = self.packet()["references"]
        self.assertEqual(len(carried), 1)
        self.assertEqual(carried[0]["reference_id"], "reading-a-receipt")
        self.assertEqual(carried[0]["bytes"], len(BODY))

    def test_a_reference_outside_scope_is_named_not_dropped(self):
        packet = self.packet()
        self.assertEqual(packet["omissions"],
                         [{"reference_id": "operator-only-notes", "reason": "denied_scope"}])
        self.assertFalse(packet["complete"])

    def test_a_packet_carrying_everything_says_so(self):
        packet = self.packet(scopes=("reviewer", "operator"),
                             contents={"reading-a-receipt": BODY,
                                       "operator-only-notes": b"operator notes\n"})
        self.assertEqual(packet["omissions"], [])
        self.assertTrue(packet["complete"])

    def test_absent_content_is_a_stale_reference_omission(self):
        packet = self.packet(contents={})
        self.assertIn({"reference_id": "reading-a-receipt", "reason": "stale_reference"},
                      packet["omissions"])

    def test_content_that_does_not_match_its_hash_is_omitted(self):
        packet = self.packet(contents={"reading-a-receipt": BODY + b"tampered\n"})
        self.assertIn({"reference_id": "reading-a-receipt", "reason": "stale_reference"},
                      packet["omissions"])

    def test_a_reference_without_a_hash_is_carried(self):
        manifest = minimal(references=[reference("r1", scope=("reviewer",))])
        packet = LS.load(manifest, HELD, {}, ["reviewer"], {"r1": b"free text\n"}, 100_000)
        self.assertEqual([r["reference_id"] for r in packet["references"]], ["r1"])

    def test_a_reference_over_the_per_reference_bound_is_omitted(self):
        manifest = minimal(references=[reference("r1")])
        big = b"x" * (BOUNDS["max_reference_bytes"] + 1)
        packet = LS.load(manifest, HELD, {}, ["reviewer"], {"r1": big}, 10_000_000)
        self.assertEqual(packet["omissions"],
                         [{"reference_id": "r1", "reason": "reference_too_large"}])

    def test_the_budget_cuts_by_name(self):
        manifest = minimal(entry="e", references=[reference("r1"), reference("r2",
                                                                            path="references/b.md")])
        packet = LS.load(manifest, HELD, {}, ["reviewer"],
                         {"r1": b"x" * 50, "r2": b"y" * 50}, 60)
        self.assertEqual([r["reference_id"] for r in packet["references"]], ["r1"])
        self.assertEqual(packet["omissions"],
                         [{"reference_id": "r2", "reason": "context_budget"}])

    def test_cost_is_the_sum_of_what_was_carried(self):
        packet = self.packet()
        self.assertEqual(packet["cost_bytes"],
                         packet["entry_bytes"] + sum(r["bytes"] for r in packet["references"]))

    def test_the_packet_budget_never_exceeds_the_module_bound(self):
        packet = self.packet(budget=10_000_000)
        self.assertEqual(packet["budget_bytes"], BOUNDS["max_packet_bytes"])

    def test_actions_in_effect_are_the_callers_not_the_skills(self):
        # The skill asks for three actions and the caller holds four; the packet states the
        # intersection. No wider set appears anywhere in it.
        packet = self.packet()
        self.assertEqual(packet["actions_in_effect"], ["health", "task.get", "task.list"])
        self.assertNotIn("roster.disable", json.dumps(packet))

    def test_a_deprecated_skill_loads_and_says_so(self):
        manifest = base()
        manifest["lifecycle"] = "deprecated"
        packet = LS.load(manifest, HELD, {}, ["reviewer"], {"reading-a-receipt": BODY}, 100_000)
        self.assertTrue(packet["deprecated"])

    def test_a_packet_carries_no_field_that_could_confer_authority(self):
        # Not a word search: the packet's own scope line contains "no grant", so a substring
        # test for "grant" passes whether or not the rule holds. The checkable claim is that
        # the packet's action set is a SUBSET of what the caller already had, for every
        # manifest, including one asking for more than the caller holds -- which cannot even
        # be loaded, so the strongest reachable statement is the subset over what can.
        for scopes in (["reviewer"], ["operator"], ["reviewer", "operator"], []):
            packet = self.packet(scopes=scopes)
            self.assertLessEqual(set(packet["actions_in_effect"]), HELD)
            self.assertEqual(set(packet) & {"grants", "authority", "permits", "token"}, set())


class Revisions(unittest.TestCase):
    def test_a_forward_revision_reports_its_drift(self):
        previous = base()
        proposed = copy.deepcopy(previous)
        proposed["skill_version"] = 3
        proposed["entry"] = "A different entry."
        proposed["references"] = [proposed["references"][0]]
        proposed["requires_actions"] = ["task.get"]
        record = LS.revise(previous, proposed)
        self.assertEqual((record["from_version"], record["to_version"]), (2, 3))
        self.assertTrue(record["entry_changed"])
        self.assertEqual(record["references_removed"], ["operator-only-notes"])
        self.assertEqual(record["references_added"], [])
        self.assertEqual(sorted(record["actions_removed"]), ["health", "task.list"])
        self.assertIn("v2", record["affected_consumers"])

    def test_a_changed_reference_hash_is_reported(self):
        previous = base()
        proposed = copy.deepcopy(previous)
        proposed["skill_version"] = 3
        proposed["references"][0]["sha256"] = hashlib.sha256(b"other").hexdigest()
        self.assertEqual(LS.revise(previous, proposed)["references_changed"],
                         ["reading-a-receipt"])


class SiteCoverage(unittest.TestCase):
    def test_every_refusal_site_has_a_case(self):
        self.assertEqual(set(LS.refusal_sites()) - EXERCISED_CODES, set(),
                         "these refusal codes are raised by the loader and asserted by no case")

    def test_the_schema_declares_exactly_what_the_loader_raises(self):
        self.assertEqual(set(LS.refusal_sites()), set(SCHEMA["hee3"]["refusals"]))


if __name__ == "__main__":
    loader = unittest.TestLoader()
    loader.sortTestMethodsUsing = None
    suite = unittest.TestSuite(loader.loadTestsFromTestCase(cls) for cls in
                               (SchemaShape, Refusals, Packets, Revisions, SiteCoverage))
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    print(f"skill tests: run={result.testsRun} failures={len(result.failures)} "
          f"errors={len(result.errors)} refusal_codes_exercised={len(EXERCISED_CODES)}")
    sys.exit(0 if result.wasSuccessful() else 1)
