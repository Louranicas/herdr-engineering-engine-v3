#!/usr/bin/env python3
"""HEE3-IF-workflows procedure schema and validator tests.

Run: python3 -W error tests/procedure_schema.py
Dependency: installed jsonschema (test tooling only). The schema author also implemented
these tests; independent oracle authorship is NOT claimed. Tests, subtests and fixture counts
confer no module case credits. No transport, dispatch, scheduler or admission is tested.

Two controls make the case list mean something rather than merely be long:

* `test_every_refusal_site_has_a_case` takes the denominator from the validator's own syntax
  tree, so a refusal added there without a case here is a red test. Counting refusal NAMES
  would hide the four codes this validator raises from more than one place.
* `tools/check-workflow-sites` neuters each site in turn and requires this file to notice.
  A case per site is not the same as a case that pins the site, and only the sweep can tell
  the two apart.
"""

import copy
import importlib.util
import json
import sys
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "workflows/procedure-v1.schema.json"
EXAMPLE = ROOT / "workflows/examples/verify-and-close-v1.json"


def _load_validator_module():
    spec = importlib.util.spec_from_file_location(
        "hee3_validate_procedure", ROOT / "workflows/validate_procedure.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


VP = _load_validator_module()
SCHEMA = json.loads(SCHEMA_PATH.read_text())
BOUNDS = SCHEMA["hee3"]["bounds"]


def base():
    return json.loads(EXAMPLE.read_text())


def step(identity, action="health", depends_on=(), attempts=1, required=True, version=1):
    return {
        "step_id": identity,
        "action": action,
        "action_version": version,
        "depends_on": list(depends_on),
        "required": required,
        "retry": {"max_attempts": attempts, "retry_on": []},
    }


def procedure(steps, **overrides):
    body = {
        "schema": "hee3.workflows.procedure.v1",
        "procedure_id": "case",
        "procedure_version": 1,
        "budget": {"max_fanout": 4, "max_seconds": 60, "max_tokens": 1000},
        "steps": steps,
        "acceptance": [{"criterion_id": "c1", "statement": "something checkable"}],
    }
    body.update(overrides)
    return body


class SchemaShape(unittest.TestCase):
    def test_schema_is_itself_valid(self):
        Draft202012Validator.check_schema(SCHEMA)

    def test_generator_reproduces_the_artifact(self):
        # The artifact is a build output. If it can drift from its generator, the enum it
        # pins is a claim nobody re-derives.
        import subprocess

        result = subprocess.run(
            [sys.executable, str(ROOT / "workflows/generate_procedure_schema.py"), "--check"],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_action_enum_is_the_action_catalogue(self):
        catalogue = json.loads((ROOT / "schemas/actions/control-v1.schema.json").read_text())
        self.assertEqual(
            SCHEMA["$defs"]["step"]["properties"]["action"]["enum"],
            catalogue["$defs"]["ActionId"]["enum"],
            "the procedure schema's action vocabulary must be the catalogue's, not a copy",
        )

    def test_reference_procedure_validates(self):
        Draft202012Validator(SCHEMA).validate(base())

    def test_unapproved_action_name_is_unspellable(self):
        body = base()
        body["steps"][0]["action"] = "task.forge"
        with self.assertRaises(Exception):
            Draft202012Validator(SCHEMA).validate(body)

    def test_unknown_step_field_is_refused(self):
        body = base()
        body["steps"][0]["shell"] = "rm -rf /"
        with self.assertRaises(Exception):
            Draft202012Validator(SCHEMA).validate(body)

    def test_effect_unknown_is_not_a_retryable_reason_in_the_schema(self):
        retryable = SCHEMA["$defs"]["step"]["properties"]["retry"]["properties"]["retry_on"]["items"]["enum"]
        self.assertNotIn("effect_unknown", retryable)

    def test_no_disposition_is_an_acceptance(self):
        self.assertNotIn("accepted", SCHEMA["hee3"]["dispositions"])


class Refusals(unittest.TestCase):
    def refused(self, body, code, committed=None, outcomes=None, verified=True):
        with self.assertRaises(VP.ProcedureRefusal) as caught:
            if committed is not None:
                VP.resume(body, committed)
            elif outcomes is not None:
                VP.join(body, outcomes, verified)
            else:
                VP.validate(body)
        self.assertEqual(caught.exception.code, code, caught.exception.detail)
        return caught.exception.detail

    def test_reference_procedure_has_a_topological_order(self):
        order = VP.validate(base())
        self.assertEqual(order[0], "preview")
        self.assertEqual(len(order), 5)

    # One case per refusal SITE. Where a code is raised from several sites, each case is
    # built so that only its own site can answer it.
    def test_stale_schema_string(self):
        self.refused(procedure([step("a")], schema="hee3.workflows.procedure.v2"),
                     "stale_procedure_version")

    def test_empty_step_list(self):
        self.refused(procedure([]), "fanout_exceeded")

    def test_more_steps_than_the_limit(self):
        steps = [step(f"s{i}") for i in range(BOUNDS["max_steps"] + 1)]
        detail = self.refused(procedure(steps), "fanout_exceeded")
        self.assertIn(str(BOUNDS["max_steps"]), detail)

    def test_duplicate_step_identity(self):
        self.refused(procedure([step("a"), step("a")]), "duplicate_step")

    def test_unknown_action_reaches_the_validator(self):
        # The schema makes this unspellable; the validator refuses it anyway, because a
        # caller that skipped schema validation must not get further than one that did.
        body = procedure([step("a")])
        body["steps"][0]["action"] = "task.forge"
        self.refused(body, "unknown_action")

    def test_action_version_below_one(self):
        self.refused(procedure([step("a", version=0)]), "unsupported_action_version")

    def test_zero_attempts(self):
        self.refused(procedure([step("a", attempts=0)]), "retry_limit")

    def test_more_attempts_than_the_limit(self):
        detail = self.refused(procedure([step("a", attempts=BOUNDS["max_retries"] + 1)]),
                              "retry_limit")
        self.assertIn(str(BOUNDS["max_retries"]), detail)

    def test_effect_unknown_listed_as_retryable(self):
        body = procedure([step("a")])
        body["steps"][0]["retry"]["retry_on"] = ["effect_unknown"]
        self.refused(body, "unreconciled_effect")

    def test_more_dependencies_than_the_limit(self):
        names = [f"d{i}" for i in range(BOUNDS["max_dependencies"] + 1)]
        steps = [step(n) for n in names] + [step("a", depends_on=names)]
        self.refused(procedure(steps), "dependency_limit")

    def test_dependency_naming_an_absent_step(self):
        detail = self.refused(procedure([step("a", depends_on=["ghost"])]), "missing_step")
        self.assertIn("ghost", detail)

    def test_step_depending_on_itself(self):
        # A cycle of length one, caught by the same remainder that catches longer ones --
        # there is no separate self-dependency check to keep in agreement with this.
        detail = self.refused(procedure([step("a", depends_on=["a"])]), "cycle")
        self.assertIn("a", detail)

    def test_two_step_cycle(self):
        # Neither step depends on itself, so the self-dependency site cannot answer this;
        # only Kahn's remainder can.
        detail = self.refused(
            procedure([step("a", depends_on=["b"]), step("b", depends_on=["a"])]), "cycle")
        self.assertIn("a", detail)
        self.assertIn("b", detail)

    def test_three_step_cycle_with_an_acyclic_prefix(self):
        steps = [step("head"),
                 step("a", depends_on=["head", "c"]),
                 step("b", depends_on=["a"]),
                 step("c", depends_on=["b"])]
        detail = self.refused(procedure(steps), "cycle")
        self.assertNotIn("head", detail)

    def test_fanout_above_the_limit(self):
        body = procedure([step("a")])
        body["budget"]["max_fanout"] = BOUNDS["max_fanout"] + 1
        self.refused(body, "fanout_exceeded")

    def test_fanout_of_zero(self):
        body = procedure([step("a")])
        body["budget"]["max_fanout"] = 0
        self.refused(body, "fanout_exceeded")

    def test_seconds_above_the_limit(self):
        body = procedure([step("a")])
        body["budget"]["max_seconds"] = BOUNDS["max_total_seconds"] + 1
        self.refused(body, "time_limit")

    def test_tokens_above_the_limit(self):
        body = procedure([step("a")])
        body["budget"]["max_tokens"] = BOUNDS["max_total_tokens"] + 1
        self.refused(body, "resource_limit")

    def test_committed_step_not_in_the_procedure(self):
        detail = self.refused(procedure([step("a")]), "identity_mismatch",
                              committed={"b": "done"})
        self.assertIn("b", detail)

    def test_committed_state_outside_the_vocabulary(self):
        self.refused(procedure([step("a")]), "identity_mismatch", committed={"a": "finished"})

    def test_resume_refuses_an_unreconciled_effect(self):
        detail = self.refused(procedure([step("a"), step("b", depends_on=["a"])]),
                              "unreconciled_effect", committed={"a": "effect_unknown"})
        self.assertIn("a", detail)


class ResumeAndJoin(unittest.TestCase):
    def test_resume_from_nothing_offers_only_roots(self):
        body = procedure([step("a"), step("b", depends_on=["a"]), step("c")])
        self.assertEqual(VP.resume(body, {}), ["a", "c"])

    def test_resume_offers_a_step_whose_dependency_settled(self):
        body = procedure([step("a"), step("b", depends_on=["a"])])
        self.assertEqual(VP.resume(body, {"a": "done"}), ["b"])

    def test_resume_does_not_offer_a_step_already_committed(self):
        body = procedure([step("a"), step("b", depends_on=["a"])])
        self.assertEqual(VP.resume(body, {"a": "done", "b": "running"}), [])

    def test_resume_treats_cancelled_as_settled(self):
        body = procedure([step("a"), step("b", depends_on=["a"])])
        self.assertEqual(VP.resume(body, {"a": "cancelled"}), ["b"])

    def test_resume_withholds_a_step_whose_dependency_is_still_running(self):
        body = procedure([step("a"), step("b", depends_on=["a"])])
        self.assertEqual(VP.resume(body, {"a": "running"}), [])

    def test_resume_is_capped_by_fanout(self):
        body = procedure([step(f"s{i}") for i in range(8)])
        body["budget"]["max_fanout"] = 3
        self.assertEqual(len(VP.resume(body, {})), 3)

    def test_join_proposes_a_candidate_when_every_required_step_is_done(self):
        body = procedure([step("a"), step("b", required=False)])
        self.assertEqual(VP.join(body, {"a": "done"}, True), ("completion_candidate", []))

    def test_join_blocks_on_a_missing_required_child(self):
        body = procedure([step("a"), step("b")])
        disposition, reasons = VP.join(body, {"a": "done"}, True)
        self.assertEqual(disposition, "blocked")
        self.assertEqual(reasons, ["missing_child"])

    def test_join_blocks_on_an_unreconciled_effect(self):
        body = procedure([step("a")])
        disposition, reasons = VP.join(body, {"a": "effect_unknown"}, True)
        self.assertEqual(disposition, "blocked")
        self.assertEqual(reasons, ["unreconciled_effect"])

    def test_join_asks_for_repair_when_only_acceptance_is_unmet(self):
        body = procedure([step("a")])
        self.assertEqual(VP.join(body, {"a": "done"}, False), ("repair", ["unmet_acceptance"]))

    def test_join_asks_for_repair_on_a_failed_required_step(self):
        body = procedure([step("a")])
        self.assertEqual(VP.join(body, {"a": "failed"}, True), ("repair", ["unmet_acceptance"]))

    def test_join_never_returns_an_acceptance(self):
        body = procedure([step("a")])
        for outcomes in ({"a": "done"}, {"a": "failed"}, {}, {"a": "effect_unknown"}):
            for verified in (True, False):
                disposition, _ = VP.join(body, outcomes, verified)
                self.assertIn(disposition, SCHEMA["hee3"]["dispositions"])
                self.assertNotEqual(disposition, "accepted")

    def test_an_optional_step_does_not_block(self):
        body = procedure([step("a"), step("b", required=False)])
        self.assertEqual(VP.join(body, {"a": "done", "b": "failed"}, True),
                         ("completion_candidate", []))


class SiteCoverage(unittest.TestCase):
    def test_every_refusal_site_has_a_case(self):
        # The denominator comes from the validator's syntax tree; the numerator is what the
        # Refusals cases actually asserted, recorded as they ran. Neither is a list here.
        self.assertEqual(
            set(VP.refusal_sites()) - EXERCISED_CODES, set(),
            "these refusal codes are raised by the validator and asserted by no case",
        )
        self.assertGreaterEqual(len(EXERCISED_CODES), 13)

    def test_the_schema_declares_every_code_the_validator_raises(self):
        self.assertEqual(set(VP.refusal_sites()) - set(SCHEMA["hee3"]["refusals"]), set())

    def test_the_validator_raises_every_code_the_schema_declares(self):
        self.assertEqual(set(SCHEMA["hee3"]["refusals"]) - set(VP.refusal_sites()), set())


EXERCISED_CODES = set()
_original_refused = Refusals.refused


def _recording_refused(self, body, code, **kwargs):
    EXERCISED_CODES.add(code)
    return _original_refused(self, body, code, **kwargs)


Refusals.refused = _recording_refused


if __name__ == "__main__":
    # SiteCoverage reads what Refusals exercised, so it must run after it.
    loader = unittest.TestLoader()
    loader.sortTestMethodsUsing = None
    suite = unittest.TestSuite(
        loader.loadTestsFromTestCase(cls) for cls in (SchemaShape, Refusals, ResumeAndJoin, SiteCoverage)
    )
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    print(f"procedure tests: run={result.testsRun} failures={len(result.failures)} "
          f"errors={len(result.errors)} refusal_codes_exercised={len(EXERCISED_CODES)}")
    sys.exit(0 if result.wasSuccessful() else 1)
