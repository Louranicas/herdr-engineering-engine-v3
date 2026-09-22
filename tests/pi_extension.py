#!/usr/bin/env python3
"""HEE3-IF-pi_extension package contract and lifecycle tests.

Run: python3 -W error tests/pi_extension.py
Dependency: installed jsonschema (test tooling only). The author of the contract also
implemented these tests; independent oracle authorship is NOT claimed.

**No host is exercised.** The contract's *"Host fixtures"* proof obligation is NOT met and is
not claimed: host registration signatures are unselected until host compatibility is
qualified, so there is nothing to run a fixture against. What these cases do cover is the
part the contract states without reference to a host signature — *"reload/stale callback;
parallel calls; cancel races; rendering failure without verdict loss; schema/version ...
parity"*. The gap is named in `integrations/pi/README.md` and in the schema's
`hee3.unqualified`, rather than left to be inferred from a count.
"""

import copy
import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "integrations/pi/pi-extension-v1.schema.json"
EXAMPLE = ROOT / "integrations/pi/examples/hee3-tools.json"

_spec = importlib.util.spec_from_file_location("hee3_pi", ROOT / "integrations/pi/reconcile.py")
PI = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(PI)

SCHEMA = json.loads(SCHEMA_PATH.read_text())
BOUNDS = SCHEMA["hee3"]["bounds"]
BRIDGES = {1}
EXERCISED_CODES = set()


def package(**overrides):
    body = json.loads(EXAMPLE.read_text())
    body.update(overrides)
    return body


class SchemaShape(unittest.TestCase):
    def test_schema_is_itself_valid(self):
        Draft202012Validator.check_schema(SCHEMA)

    def test_generator_reproduces_the_artifact(self):
        result = subprocess.run(
            [sys.executable, str(ROOT / "integrations/pi/generate_extension_schema.py"), "--check"],
            capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_the_example_validates(self):
        Draft202012Validator(SCHEMA).validate(package())

    def test_action_enum_is_the_action_catalogue(self):
        catalogue = json.loads((ROOT / "schemas/actions/control-v1.schema.json").read_text())
        self.assertEqual(SCHEMA["$defs"]["tool"]["properties"]["action"]["enum"],
                         catalogue["$defs"]["ActionId"]["enum"])

    def test_host_binding_is_sealed_and_empty(self):
        binding = SCHEMA["properties"]["host_binding"]
        self.assertEqual(binding["properties"], {})
        self.assertFalse(binding["additionalProperties"])

    def test_any_host_binding_field_is_unspellable(self):
        with self.assertRaises(Exception):
            Draft202012Validator(SCHEMA).validate(
                package(host_binding={"register": "pi.register_tool"}))

    def test_the_schema_names_what_is_unqualified(self):
        # A gap stated in the artifact, so a reader of the schema alone cannot miss it.
        self.assertTrue(any("unselected" in line for line in SCHEMA["hee3"]["unqualified"]))
        self.assertTrue(any("host fixture" in line.lower() for line in SCHEMA["hee3"]["unqualified"]))


class Registration(unittest.TestCase):
    def refused(self, code, call):
        EXERCISED_CODES.add(code)
        with self.assertRaises(PI.ExtensionRefusal) as caught:
            call()
        self.assertEqual(caught.exception.code, code, caught.exception.detail)
        return caught.exception.detail

    def test_the_example_registers_but_not_with_a_host(self):
        result = PI.register(package(), BRIDGES)
        self.assertEqual(len(result["tools"]), 3)
        self.assertFalse(result["registered_with_host"])
        self.assertIn("unqualified", result["reason_not_registered"])

    def test_a_declared_host_binding_is_refused(self):
        detail = self.refused(
            "host_binding_unqualified",
            lambda: PI.register(package(host_binding={"register": "x"}), BRIDGES))
        self.assertIn("unselected", detail)

    def test_an_empty_host_binding_is_accepted(self):
        # The sealed-but-present case: declaring nothing is not declaring something.
        self.assertTrue(PI.register(package(host_binding={}), BRIDGES)["tools"])

    def test_a_wrong_schema_string(self):
        self.refused("unknown_bridge_version",
                     lambda: PI.register(package(schema="hee3.integrations.pi_extension.v2"),
                                         BRIDGES))

    def test_an_unsupported_bridge_version(self):
        detail = self.refused("unknown_bridge_version",
                              lambda: PI.register(package(bridge_version=9), BRIDGES))
        self.assertIn("9", detail)

    def test_more_tools_than_the_limit(self):
        tool = package()["tools"][0]
        many = [dict(tool, tool_id=f"t{i}") for i in range(BOUNDS["max_tools"] + 1)]
        self.refused("tool_limit", lambda: PI.register(package(tools=many), BRIDGES))

    def test_a_duplicate_tool_identity(self):
        tools = package()["tools"]
        self.refused("duplicate_call",
                     lambda: PI.register(package(tools=[tools[0], dict(tools[0])]), BRIDGES))

    def test_a_tool_exposing_an_action_outside_the_catalogue(self):
        tools = package()["tools"]
        tools[0]["action"] = "task.forge"
        self.refused("unknown_action", lambda: PI.register(package(tools=tools), BRIDGES))


class Lifecycle(unittest.TestCase):
    def setUp(self):
        self.calls = PI.Calls(generation=3)

    def refused(self, code, call):
        EXERCISED_CODES.add(code)
        with self.assertRaises(PI.ExtensionRefusal) as caught:
            call()
        self.assertEqual(caught.exception.code, code, caught.exception.detail)
        return caught.exception.detail

    def test_a_call_opens_and_renders(self):
        self.calls.opened("c1", "get-task")
        settled = self.calls.render("c1", "get-task", 3, b"{}")
        self.assertEqual(settled["state"], "rendered")
        self.assertEqual(settled["bytes"], 2)

    def test_a_result_under_an_old_generation_is_refused(self):
        self.calls.opened("c1", "get-task")
        self.calls.reload(4)
        # The call was abandoned by the reload; under the new generation it is unknown.
        self.refused("unknown_call", lambda: self.calls.render("c1", "get-task", 4, b"{}"))

    def test_a_reload_names_what_it_abandoned(self):
        self.calls.opened("c1", "get-task")
        self.calls.opened("c2", "engine-health")
        self.assertEqual(self.calls.reload(4), ["c1", "c2"])
        self.assertEqual(self.calls.open, {})

    def test_a_reload_must_advance_the_generation(self):
        self.refused("stale_generation", lambda: self.calls.reload(3))
        self.refused("stale_generation", lambda: self.calls.reload(2))

    def test_a_callback_under_a_stale_generation_is_refused(self):
        self.calls.opened("c1", "get-task")
        detail = self.refused("stale_generation",
                              lambda: self.calls.render("c1", "get-task", 2, b"{}"))
        self.assertIn("generation 3", detail)

    def test_a_result_for_the_wrong_tool_is_refused(self):
        # The parallel-call case: two calls are live, and one's result claims the other's
        # tool. A reconciler keyed only on call id would accept it.
        self.calls.opened("c1", "get-task")
        self.calls.opened("c2", "engine-health")
        detail = self.refused("call_identity_mismatch",
                              lambda: self.calls.render("c1", "engine-health", 3, b"{}"))
        self.assertIn("get-task", detail)

    def test_parallel_calls_settle_independently(self):
        for index in range(4):
            self.calls.opened(f"c{index}", "get-task")
        self.calls.render("c2", "get-task", 3, b"two")
        self.assertEqual(sorted(self.calls.open), ["c0", "c1", "c3"])
        self.assertEqual(self.calls.settled["c2"]["bytes"], 3)

    def test_an_unknown_call_is_refused(self):
        self.refused("unknown_call", lambda: self.calls.render("ghost", "get-task", 3, b"{}"))

    def test_a_call_cannot_open_twice(self):
        self.calls.opened("c1", "get-task")
        self.refused("duplicate_call", lambda: self.calls.opened("c1", "get-task"))

    def test_a_settled_call_cannot_reopen(self):
        self.calls.opened("c1", "get-task")
        self.calls.render("c1", "get-task", 3, b"{}")
        self.refused("duplicate_call", lambda: self.calls.opened("c1", "get-task"))

    def test_more_parallel_calls_than_the_limit(self):
        for index in range(BOUNDS["max_parallel_calls"]):
            self.calls.opened(f"c{index}", "get-task")
        self.refused("parallel_call_limit", lambda: self.calls.opened("overflow", "get-task"))

    def test_the_last_allowed_parallel_call_opens(self):
        for index in range(BOUNDS["max_parallel_calls"] - 1):
            self.calls.opened(f"c{index}", "get-task")
        self.assertEqual(self.calls.opened("last", "get-task")["state"], "running")

    def test_a_render_over_the_bound_is_refused_without_settling_the_call(self):
        self.calls.opened("c1", "get-task")
        self.refused("render_limit",
                     lambda: self.calls.render("c1", "get-task", 3,
                                               b"x" * (BOUNDS["max_render_bytes"] + 1)))
        # Rendering failure without verdict loss: the call is still open and still its own.
        self.assertIn("c1", self.calls.open)
        self.assertEqual(self.calls.render("c1", "get-task", 3, b"{}")["state"], "rendered")

    def test_cancellation_wins_a_race_against_a_late_result(self):
        self.calls.opened("c1", "get-task")
        self.assertEqual(self.calls.cancel("c1", "get-task", 3)["state"], "cancelled")
        detail = self.refused("terminal_call",
                              lambda: self.calls.render("c1", "get-task", 3, b"{}"))
        self.assertIn("cancelled", detail)
        self.assertEqual(self.calls.settled["c1"]["state"], "cancelled")

    def test_a_result_wins_a_race_against_a_late_cancellation(self):
        # The same rule from the other side: whichever lands first is the outcome, and the
        # second is refused rather than overwriting it.
        self.calls.opened("c1", "get-task")
        self.calls.render("c1", "get-task", 3, b"{}")
        self.refused("terminal_call", lambda: self.calls.cancel("c1", "get-task", 3))
        self.assertEqual(self.calls.settled["c1"]["state"], "rendered")

    def test_an_unobserved_effect_stays_open_and_is_not_done(self):
        self.calls.opened("c1", "get-task")
        self.assertEqual(self.calls.unknown_effect("c1", "get-task", 3)["state"],
                         "effect_unknown")
        self.assertIn("c1", self.calls.open)
        self.assertNotIn("c1", self.calls.settled)

    def test_an_unobserved_effect_can_still_be_reconciled(self):
        self.calls.opened("c1", "get-task")
        self.calls.unknown_effect("c1", "get-task", 3)
        self.assertEqual(self.calls.render("c1", "get-task", 3, b"{}")["state"], "rendered")


class SiteCoverage(unittest.TestCase):
    def test_every_refusal_site_has_a_case(self):
        self.assertEqual(set(PI.refusal_sites()) - EXERCISED_CODES, set())

    def test_the_schema_declares_exactly_what_the_module_raises(self):
        self.assertEqual(set(PI.refusal_sites()), set(SCHEMA["hee3"]["refusals"]))


if __name__ == "__main__":
    loader = unittest.TestLoader()
    loader.sortTestMethodsUsing = None
    suite = unittest.TestSuite(loader.loadTestsFromTestCase(cls) for cls in
                               (SchemaShape, Registration, Lifecycle, SiteCoverage))
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    print(f"pi extension tests: run={result.testsRun} failures={len(result.failures)} "
          f"errors={len(result.errors)} refusal_codes_exercised={len(EXERCISED_CODES)} "
          f"host_fixtures_run=0")
    sys.exit(0 if result.wasSuccessful() else 1)
