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

    def test_a_tool_at_an_action_version_the_catalogue_does_not_pin(self):
        tools = package()["tools"]
        tools[1]["action_version"] = 9
        detail = self.refused("unknown_action_version",
                              lambda: PI.register(package(tools=tools), BRIDGES))
        self.assertIn("get-task", detail)
        self.assertIn("9", detail)

    def test_a_tool_below_the_pinned_action_version(self):
        # The pin is equality, not a ceiling: a version under the catalogue's is as unknown as
        # one over it. `_shape_problem` checks type only, so 0 reaches this check.
        tools = package()["tools"]
        tools[1]["action_version"] = 0
        detail = self.refused("unknown_action_version",
                              lambda: PI.register(package(tools=tools), BRIDGES))
        self.assertIn("get-task", detail)
        self.assertIn("version 0", detail)

    def test_the_catalogue_pins_every_action_version_it_admits(self):
        # Independent source: the control-v1 catalogue's own Request_<action> definitions,
        # read here rather than through the generator's table.
        catalogue = json.loads((ROOT / "schemas/actions/control-v1.schema.json").read_text())
        expected = {}
        for name, node in catalogue["$defs"].items():
            if name.startswith("Request_"):
                props = node["properties"]
                expected[props["action"]["const"]] = props["action_version"]["const"]
        self.assertEqual(SCHEMA["hee3"]["action_versions"], expected)

    def test_a_tool_at_the_current_action_version_registers(self):
        # Moved off the identity element: the admitted version is read back per tool, and
        # every one of the example's three tools carries it through.
        result = PI.register(package(), BRIDGES)
        self.assertEqual([(t["tool_id"], t["action_version"]) for t in result["tools"]],
                         [("list-tasks", 1), ("get-task", 1), ("engine-health", 1)])

    def malformed(self, body):
        return self.refused("malformed_package", lambda: PI.register(body, BRIDGES))

    def test_a_package_missing_its_bridge_version(self):
        body = package()
        del body["bridge_version"]
        self.assertIn("bridge_version", self.malformed(body))

    def test_a_package_with_no_tools(self):
        self.assertIn("tools", self.malformed(package(tools=[])))

    def test_a_package_with_an_unknown_key(self):
        self.assertIn("handlers", self.malformed(package(handlers=[])))

    def test_a_tool_with_an_unknown_key(self):
        tools = package()["tools"]
        tools[2]["callback"] = "pi.on_result"
        self.assertIn("callback", self.malformed(package(tools=tools)))

    def test_a_tool_missing_its_action(self):
        tools = package()["tools"]
        del tools[0]["action"]
        self.assertIn("action", self.malformed(package(tools=tools)))

    def test_a_package_that_is_not_an_object(self):
        self.assertIn("object", self.malformed([package()]))

    def test_a_tool_that_is_not_an_object(self):
        self.assertIn("object", self.malformed(package(tools=["get-task"])))

    def test_tools_that_are_not_a_list(self):
        # Exact: a dict of tools must fail as the wrong TYPE, not later as a non-object entry
        # reached by iterating its keys.
        self.assertEqual(self.malformed(package(tools={"tool_id": "x"})),
                         "package.tools is not array")

    def test_a_version_of_the_wrong_type(self):
        # A string and a boolean are both refused: `True == 1` in Python, so a check by
        # equality alone would admit `true` as bridge version 1.
        self.assertIn("bridge_version", self.malformed(package(bridge_version="1")))
        self.assertIn("bridge_version", self.malformed(package(bridge_version=True)))

    def test_a_tool_version_of_the_wrong_type(self):
        tools = package()["tools"]
        tools[1]["action_version"] = "1"
        self.assertIn("action_version", self.malformed(package(tools=tools)))

    def test_an_identifier_of_the_wrong_type(self):
        self.assertIn("extension_id", self.malformed(package(extension_id=7)))


def admitted(generation=3, bridge_version=1):
    return PI.register(package(extension_generation=generation, bridge_version=bridge_version),
                       {1, 2})


class Lifecycle(unittest.TestCase):
    def setUp(self):
        self.calls = PI.Calls(admitted())

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
        self.calls.reload(admitted(4))
        # The call was revoked by the reload, not forgotten: a result for it is stale from
        # either generation, and it is never settled by one.
        detail = self.refused("stale_generation",
                              lambda: self.calls.render("c1", "get-task", 4, b"{}"))
        self.assertIn("revoked", detail)
        self.refused("stale_generation", lambda: self.calls.render("c1", "get-task", 3, b"{}"))
        self.assertNotIn("c1", self.calls.settled)

    def test_a_reload_names_what_it_abandoned(self):
        self.calls.opened("c1", "get-task")
        self.calls.opened("c2", "engine-health")
        self.assertEqual(self.calls.reload(admitted(4)), ["c1", "c2"])
        self.assertEqual(self.calls.open, {})
        self.assertEqual(sorted(self.calls.revoked), ["c1", "c2"])

    def test_a_reload_must_advance_the_generation(self):
        self.refused("stale_generation", lambda: self.calls.reload(admitted(3)))
        self.refused("stale_generation", lambda: self.calls.reload(admitted(2)))

    def test_an_unobserved_effect_survives_a_reload(self):
        # G07: an unresolved obligation is retained until actual reconciliation. A reload
        # is not a reconciliation, so it must not erase what nobody observed.
        self.calls.opened("c1", "get-task")
        self.calls.opened("c2", "engine-health")
        self.calls.unknown_effect("c1", "get-task", 3)
        self.calls.reload(admitted(4))
        self.assertEqual(self.calls.revoked["c1"]["state"], "effect_unknown")
        self.assertEqual(self.calls.revoked["c1"]["generation"], 3)
        self.assertEqual(self.calls.revoked["c2"]["state"], "running")
        self.assertNotIn("c1", self.calls.settled)

    def test_a_revoked_call_id_cannot_be_reopened(self):
        self.calls.opened("c1", "get-task")
        self.calls.reload(admitted(4))
        detail = self.refused("duplicate_call", lambda: self.calls.opened("c1", "get-task"))
        self.assertIn("revoked", detail)
        self.assertEqual(self.calls.revoked["c1"]["generation"], 3)

    def test_a_version_switch_keeps_each_call_on_its_admitted_tuple(self):
        # T29: the admitted package/protocol tuple stays pinned through every active
        # attempt. Two records that differ in every tuple field, so no constant passes.
        self.calls.opened("c1", "get-task")
        self.calls.reload(admitted(generation=4, bridge_version=2))
        fresh = self.calls.opened("c2", "engine-health")
        old = self.calls.revoked["c1"]
        self.assertEqual(
            {k: old[k] for k in ("tool_id", "extension_id", "generation", "bridge_version",
                                 "action_version")},
            {"tool_id": "get-task", "extension_id": "hee3-tools", "generation": 3,
             "bridge_version": 1, "action_version": 1})
        self.assertEqual(
            {k: fresh[k] for k in ("tool_id", "generation", "bridge_version")},
            {"tool_id": "engine-health", "generation": 4, "bridge_version": 2})
        self.refused("stale_generation", lambda: self.calls.render("c1", "get-task", 3, b"{}"))
        self.assertEqual(self.calls.revoked["c1"]["bridge_version"], 1)

    def test_a_call_records_the_admission_it_was_opened_under(self):
        # A second admission differing from the default in every tuple field, so a record
        # built from constants rather than from the admission cannot pass.
        body = package(extension_id="hee3-tools-b", extension_generation=7, bridge_version=2)
        calls = PI.Calls(PI.register(body, {1, 2}))
        record = calls.opened("k1", "list-tasks")
        self.assertEqual(record, {"tool_id": "list-tasks", "extension_id": "hee3-tools-b",
                                  "generation": 7, "bridge_version": 2, "action_version": 1,
                                  "state": "running"})

    def test_a_call_for_a_tool_not_admitted_is_refused(self):
        detail = self.refused("unknown_tool", lambda: self.calls.opened("c1", "delete-task"))
        self.assertIn("delete-task", detail)
        self.assertEqual(self.calls.open, {})

    def test_a_reload_drops_a_tool_the_new_package_does_not_admit(self):
        body = package(extension_generation=4)
        body["tools"] = body["tools"][:2]
        self.calls.reload(PI.register(body, BRIDGES))
        self.refused("unknown_tool", lambda: self.calls.opened("c9", "engine-health"))
        self.assertEqual(self.calls.opened("c8", "get-task")["generation"], 4)

    def test_an_action_error_settles_the_call_as_failed(self):
        self.calls.opened("c1", "get-task")
        self.calls.opened("c2", "list-tasks")
        settled = self.calls.fail("c1", "get-task", 3, "not_found")
        self.assertEqual((settled["state"], settled["error_code"]), ("failed", "not_found"))
        settled = self.calls.fail("c2", "list-tasks", 3, "forbidden")
        self.assertEqual((settled["state"], settled["error_code"]), ("failed", "forbidden"))
        self.assertEqual(self.calls.open, {})

    def test_a_failure_outside_the_control_vocabulary_is_refused(self):
        self.calls.opened("c1", "get-task")
        detail = self.refused("unknown_error_code",
                              lambda: self.calls.fail("c1", "get-task", 3, "oops"))
        self.assertIn("oops", detail)
        self.assertIn("c1", self.calls.open)

    def test_a_failure_goes_through_the_same_identity_check(self):
        self.calls.opened("c1", "get-task")
        self.refused("call_identity_mismatch",
                     lambda: self.calls.fail("c1", "engine-health", 3, "internal"))
        self.refused("stale_generation", lambda: self.calls.fail("c1", "get-task", 2, "internal"))
        self.assertIn("c1", self.calls.open)

    def test_a_failure_wins_a_race_against_a_late_result(self):
        self.calls.opened("c1", "get-task")
        self.calls.fail("c1", "get-task", 3, "unavailable")
        self.assertIn("failed", self.refused(
            "terminal_call", lambda: self.calls.render("c1", "get-task", 3, b"{}")))
        self.refused("terminal_call", lambda: self.calls.cancel("c1", "get-task", 3))
        self.assertEqual(self.calls.settled["c1"]["state"], "failed")

    def test_a_result_wins_a_race_against_a_late_failure(self):
        self.calls.opened("c1", "get-task")
        self.calls.render("c1", "get-task", 3, b"{}")
        self.refused("terminal_call", lambda: self.calls.fail("c1", "get-task", 3, "internal"))
        self.assertEqual(self.calls.settled["c1"]["state"], "rendered")

    def test_a_confirmed_cancel_wins_a_race_against_a_late_failure(self):
        self.calls.opened("c1", "get-task")
        self.calls.cancel("c1", "get-task", 3)
        self.assertIn("cancelled", self.refused(
            "terminal_call", lambda: self.calls.fail("c1", "get-task", 3, "internal")))
        self.assertEqual(self.calls.settled["c1"]["state"], "cancelled")
        self.assertNotIn("error_code", self.calls.settled["c1"])

    def test_a_cancellation_request_is_acknowledged_but_not_final(self):
        self.calls.opened("c1", "get-task")
        acknowledged = self.calls.cancel_requested("c1", "get-task", 3)
        self.assertEqual(acknowledged["state"], "cancellation_requested")
        self.assertIn("c1", self.calls.open)
        self.assertNotIn("c1", self.calls.settled)

    def test_a_result_after_a_cancellation_request_settles_as_rendered(self):
        # The effect completed anyway: the truthful outcome is the result, not "cancelled".
        self.calls.opened("c1", "get-task")
        self.calls.cancel_requested("c1", "get-task", 3)
        self.assertEqual(self.calls.render("c1", "get-task", 3, b"done")["state"], "rendered")

    def test_a_confirmed_cancel_after_a_request_settles_as_cancelled(self):
        self.calls.opened("c1", "get-task")
        self.calls.cancel_requested("c1", "get-task", 3)
        self.assertEqual(self.calls.cancel("c1", "get-task", 3)["state"], "cancelled")
        self.assertNotIn("c1", self.calls.open)

    def test_a_failure_after_a_cancellation_request_settles_as_failed(self):
        self.calls.opened("c1", "get-task")
        self.calls.cancel_requested("c1", "get-task", 3)
        self.assertEqual(self.calls.fail("c1", "get-task", 3, "cancelled")["state"], "failed")

    def test_a_cancellation_request_does_not_hide_an_unobserved_effect(self):
        self.calls.opened("c1", "get-task")
        self.calls.unknown_effect("c1", "get-task", 3)
        acknowledged = self.calls.cancel_requested("c1", "get-task", 3)
        self.assertEqual(acknowledged["state"], "effect_unknown")
        self.assertTrue(acknowledged["cancel_acknowledged"])

    def test_a_cancellation_request_goes_through_the_identity_check(self):
        self.calls.opened("c1", "get-task")
        self.refused("call_identity_mismatch",
                     lambda: self.calls.cancel_requested("c1", "list-tasks", 3))
        self.assertEqual(self.calls.open["c1"]["state"], "running")

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

    def test_every_declared_call_state_is_one_the_module_assigns(self):
        # A declared state nothing produces is a dead path presented as vocabulary (G10).
        self.assertEqual(set(PI.assigned_states()), set(SCHEMA["hee3"]["call_states"]))

    def test_the_error_vocabulary_is_the_control_catalogue(self):
        catalogue = json.loads((ROOT / "schemas/actions/control-v1.schema.json").read_text())
        self.assertEqual(SCHEMA["hee3"]["error_codes"], catalogue["$defs"]["ErrorCodeV1"]["enum"])


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
