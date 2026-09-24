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
import subprocess
import sys
import tempfile
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


def record(steps, procedure_id="case", version=1):
    """A committed record, bound to the procedure identity and version it was written under."""
    return {"procedure_id": procedure_id, "procedure_version": version, "steps": dict(steps)}


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

    def test_action_versions_are_the_catalogue_s(self):
        # Read here by a different path than the generator's: each action's own request
        # definition, named by convention, rather than a scan of every definition.
        catalogue = json.loads((ROOT / "schemas/actions/control-v1.schema.json").read_text())
        actions = catalogue["$defs"]["ActionId"]["enum"]
        expected = {
            action: catalogue["$defs"]["Request_" + action.replace(".", "_")]
            ["properties"]["action_version"]["const"]
            for action in actions
        }
        self.assertEqual(SCHEMA["hee3"]["action_versions"], expected)
        self.assertEqual(len(expected), 21)

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
                VP.join(body, outcomes, verified, False)
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

    def test_action_version_the_catalogue_does_not_serve(self):
        # Above the floor, so only the catalogue comparison can refuse it (WF-03).
        detail = self.refused(procedure([step("a", version=2)]), "unsupported_action_version")
        self.assertIn("action_version 2", detail)
        self.assertIn("serves version 1", detail)

    def test_duplicate_criterion_identity(self):
        body = base()
        body["acceptance"].append(copy.deepcopy(body["acceptance"][0]))
        detail = self.refused(body, "duplicate_criterion")
        self.assertIn(body["acceptance"][0]["criterion_id"], detail)

    def test_every_schema_required_member_is_refused_when_absent(self):
        # The members come from the generated schema, not from the validator, so a member the
        # validator forgot is a red case here rather than a KeyError in production (WF-06).
        defs = SCHEMA["$defs"]
        places = [
            ("procedure", SCHEMA["required"], lambda b: b),
            ("budget", defs["budget"]["required"], lambda b: b["budget"]),
            ("step", defs["step"]["required"], lambda b: b["steps"][1]),
            ("retry", defs["step"]["properties"]["retry"]["required"],
             lambda b: b["steps"][1]["retry"]),
            ("criterion", defs["criterion"]["required"], lambda b: b["acceptance"][1]),
        ]
        members = 0
        for place, required, locate in places:
            for member in required:
                with self.subTest(place=place, member=member):
                    body = base()
                    del locate(body)[member]
                    detail = self.refused(body, "malformed_procedure")
                    self.assertIn(repr(member), detail)
                    members += 1
        self.assertEqual(members, 16, "the schema's required members were not all enumerated")

    def test_member_of_the_wrong_type_is_refused(self):
        cases = [
            (lambda b: b.__setitem__("steps", {"a": 1}), "procedure.steps"),
            (lambda b: b["steps"].__setitem__(2, "threads"), "procedure.steps[2]"),
            (lambda b: b["steps"][3].__setitem__("step_id", 7), "procedure.steps[3].step_id"),
            (lambda b: b["steps"][0]["retry"].__setitem__("max_attempts", "2"),
             "procedure.steps[0].retry.max_attempts"),
            (lambda b: b["steps"][1].__setitem__("depends_on", "preview"),
             "procedure.steps[1].depends_on"),
            (lambda b: b["steps"][1]["depends_on"].__setitem__(0, ["preview"]),
             "procedure.steps[1].depends_on[0]"),
            (lambda b: b.__setitem__("budget", [4, 60, 1000]), "procedure.budget"),
            (lambda b: b["steps"][4].__setitem__("required", "no"), "procedure.steps[4].required"),
        ]
        for plant, where in cases:
            with self.subTest(where=where):
                body = base()
                plant(body)
                detail = self.refused(body, "malformed_procedure")
                self.assertTrue(detail.startswith(where + " "), detail)

    def test_a_boolean_is_not_an_integer(self):
        # JSON distinguishes them and Python's bool is an int; the schema says integer.
        body = base()
        body["steps"][0]["action_version"] = True
        detail = self.refused(body, "malformed_procedure")
        self.assertIn("procedure.steps[0].action_version", detail)

    def test_a_procedure_that_is_not_an_object(self):
        detail = self.refused([base()], "malformed_procedure")
        self.assertIn("procedure is not object", detail)

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
                              committed=record({"b": "done"}))
        self.assertIn("b", detail)

    def test_committed_state_outside_the_vocabulary(self):
        self.refused(procedure([step("a")]), "identity_mismatch",
                     committed=record({"a": "finished"}))

    def test_committed_under_another_procedure_identity(self):
        detail = self.refused(procedure([step("a")]), "identity_mismatch",
                              committed=record({"a": "done"}, procedure_id="other"))
        self.assertIn("'other'", detail)
        self.assertIn("'case'", detail)

    def test_committed_under_another_procedure_version(self):
        # The step ids agree; only the version differs, so no id check can answer this (WF-04).
        detail = self.refused(procedure([step("a"), step("b", depends_on=["a"])]),
                              "stale_procedure_version",
                              committed=record({"a": "done"}, version=2))
        self.assertIn("version 2", detail)
        self.assertIn("version 1", detail)

    def test_committed_record_without_a_step_map(self):
        detail = self.refused(procedure([step("a")]), "identity_mismatch",
                              committed={"procedure_id": "case", "procedure_version": 1})
        self.assertIn("step map", detail)

    def test_join_refuses_an_outcome_for_a_foreign_step(self):
        # resume() already refused this; join() ignored it (WF-05). One door now.
        detail = self.refused(procedure([step("a")]), "identity_mismatch",
                              outcomes=record({"a": "done", "ghost": "done"}))
        self.assertIn("ghost", detail)

    def test_join_refuses_an_outcome_state_outside_the_vocabulary(self):
        detail = self.refused(procedure([step("a")]), "identity_mismatch",
                              outcomes=record({"a": "finished"}))
        self.assertIn("finished", detail)

    def test_join_refuses_outcomes_recorded_under_another_version(self):
        self.refused(procedure([step("a")]), "stale_procedure_version",
                     outcomes=record({"a": "done"}, version=3))

    def test_resume_refuses_an_unreconciled_effect(self):
        detail = self.refused(procedure([step("a"), step("b", depends_on=["a"])]),
                              "unreconciled_effect", committed=record({"a": "effect_unknown"}))
        self.assertIn("a", detail)


class ResumeAndJoin(unittest.TestCase):
    def test_resume_from_nothing_offers_only_roots(self):
        body = procedure([step("a"), step("b", depends_on=["a"]), step("c")])
        self.assertEqual(VP.resume(body, record({})), ["a", "c"])

    def test_resume_offers_a_step_whose_dependency_settled(self):
        body = procedure([step("a"), step("b", depends_on=["a"])])
        self.assertEqual(VP.resume(body, record({"a": "done"})), ["b"])

    def test_resume_does_not_offer_a_step_already_committed(self):
        body = procedure([step("a"), step("b", depends_on=["a"])])
        self.assertEqual(VP.resume(body, record({"a": "done", "b": "running"})), [])

    def test_resume_treats_cancelled_as_settled(self):
        body = procedure([step("a"), step("b", depends_on=["a"])])
        self.assertEqual(VP.resume(body, record({"a": "cancelled"})), ["b"])

    def test_resume_withholds_a_step_whose_dependency_is_still_running(self):
        body = procedure([step("a"), step("b", depends_on=["a"])])
        self.assertEqual(VP.resume(body, record({"a": "running"})), [])

    def test_resume_is_capped_by_fanout(self):
        body = procedure([step(f"s{i}") for i in range(8)])
        body["budget"]["max_fanout"] = 3
        self.assertEqual(len(VP.resume(body, record({}))), 3)

    def test_join_proposes_a_candidate_when_every_required_step_is_done(self):
        body = procedure([step("a"), step("b", required=False)])
        self.assertEqual(VP.join(body, record({"a": "done"}), True, False),
                         ("completion_candidate", []))

    def test_join_blocks_on_a_missing_required_child(self):
        body = procedure([step("a"), step("b")])
        disposition, reasons = VP.join(body, record({"a": "done"}), True, False)
        self.assertEqual(disposition, "blocked")
        self.assertEqual(reasons, ["missing_child"])

    def test_join_blocks_on_an_unreconciled_effect(self):
        body = procedure([step("a")])
        disposition, reasons = VP.join(body, record({"a": "effect_unknown"}), True, False)
        self.assertEqual(disposition, "blocked")
        self.assertEqual(reasons, ["unreconciled_effect"])

    def test_join_asks_for_repair_when_only_acceptance_is_unmet(self):
        body = procedure([step("a")])
        self.assertEqual(VP.join(body, record({"a": "done"}), False, False),
                         ("repair", ["unmet_acceptance"]))

    def test_join_asks_for_repair_on_a_failed_required_step(self):
        body = procedure([step("a")])
        self.assertEqual(VP.join(body, record({"a": "failed"}), True, False),
                         ("repair", ["unmet_acceptance"]))

    def test_join_never_returns_an_acceptance(self):
        body = procedure([step("a")])
        for outcomes in ({"a": "done"}, {"a": "failed"}, {}, {"a": "effect_unknown"}):
            for verified in (True, False):
                disposition, _ = VP.join(body, record(outcomes), verified, False)
                self.assertIn(disposition, SCHEMA["hee3"]["dispositions"])
                self.assertNotEqual(disposition, "accepted")

    def test_an_optional_step_with_an_unreconciled_effect_blocks(self):
        # The reference procedure's optional `record` step is analysis.request, an effect.
        # Optional means the join may close WITHOUT its outcome, not over an unresolved one.
        body = base()
        outcomes = {"preview": "done", "submit": "done", "threads": "done", "verify": "done",
                    "record": "effect_unknown"}
        self.assertEqual(VP.join(body, record(outcomes, "verify-and-close"), True, False),
                         ("blocked", ["unreconciled_effect"]))

    def test_join_blocks_on_budget_exhaustion(self):
        body = procedure([step("a")])
        self.assertEqual(VP.join(body, record({"a": "done"}), True, True),
                         ("blocked", ["resource_exhausted"]))

    def test_exhaustion_is_not_repairable_when_acceptance_is_also_unmet(self):
        body = procedure([step("a")])
        self.assertEqual(VP.join(body, record({"a": "done"}), False, True),
                         ("blocked", ["unmet_acceptance", "resource_exhausted"]))

    def test_an_optional_step_does_not_block(self):
        body = procedure([step("a"), step("b", required=False)])
        self.assertEqual(VP.join(body, record({"a": "done", "b": "failed"}), True, False),
                         ("completion_candidate", []))


class CommandLine(unittest.TestCase):
    """The one projection of a procedure onto a command: the exit status is the verdict."""

    COMMAND = (sys.executable, "-W", "error", str(ROOT / "workflows/validate_procedure.py"))

    def run_on(self, content):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "procedure.json"
            if not isinstance(content, bytes):
                content = json.dumps(content).encode()
            path.write_bytes(content)
            # A hang guard for the test, not a limit of the code under test.
            return subprocess.run([*self.COMMAND, str(path)], capture_output=True, text=True,
                                  check=False, timeout=120)

    def test_a_valid_procedure_prints_its_order_and_exits_zero(self):
        for body, line in ((base(), "order=preview,submit,threads,verify,record\n"),
                           (procedure([step("z"), step("m", depends_on=["z"])]), "order=z,m\n")):
            result = self.run_on(body)
            self.assertEqual((result.returncode, result.stdout, result.stderr), (0, line, ""))

    def test_a_refused_procedure_prints_its_code_and_exits_one(self):
        cases = (
            (procedure([step("a", depends_on=["b"]), step("b", depends_on=["a"])]),
             "refused cycle: steps in a dependency cycle: a, b\n"),
            (procedure([step("q"), step("q")]),
             "refused duplicate_step: step_id 'q' appears twice\n"),
        )
        for body, line in cases:
            result = self.run_on(body)
            self.assertEqual((result.returncode, result.stdout), (1, line))

    def test_a_file_over_the_byte_bound_is_not_parsed(self):
        limit = BOUNDS["max_procedure_bytes"]
        # Well over the bound, so a reader that took the whole file would report its size
        # rather than the bound plus one.
        over = self.run_on(b"{}" + b" " * (limit + 4096))
        self.assertEqual(over.returncode, 2, over.stderr)
        self.assertIn(f"exceeds the bound of {limit} bytes (stopped after {limit + 1})",
                      over.stderr)
        # At the bound exactly, the file is read and judged as a procedure.
        at = self.run_on(b"{}" + b" " * (limit - 2))
        self.assertEqual(
            (at.returncode, at.stdout),
            (1, "refused malformed_procedure: procedure lacks required member 'schema'\n"))

    def test_a_file_that_is_not_json_exits_two(self):
        result = self.run_on(b"steps: [a, b]")
        self.assertEqual((result.returncode, result.stdout), (2, ""))
        self.assertIn("is not JSON", result.stderr)

    def test_a_missing_argument_exits_two(self):
        result = subprocess.run(list(self.COMMAND), capture_output=True, text=True, check=False,
                                timeout=120)
        self.assertEqual((result.returncode, result.stdout), (2, ""))
        self.assertIn("usage:", result.stderr)


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

    def test_the_schema_declares_every_block_reason_join_appends(self):
        self.assertEqual(set(VP.block_reason_sites()) - set(SCHEMA["hee3"]["block_reasons"]), set())

    def test_join_appends_every_block_reason_the_schema_declares(self):
        # The mirror of the refusal check (WF-02): a declared reason no code can emit is a
        # promise to the reader that nothing keeps.
        self.assertEqual(set(SCHEMA["hee3"]["block_reasons"]) - set(VP.block_reason_sites()), set())

    def test_every_declared_block_reason_was_returned_by_a_case(self):
        self.assertEqual(set(SCHEMA["hee3"]["block_reasons"]) - EMITTED_REASONS, set(),
                         "these block reasons are declared and returned by no case")


EXERCISED_CODES = set()
_original_refused = Refusals.refused


def _recording_refused(self, body, code, **kwargs):
    EXERCISED_CODES.add(code)
    return _original_refused(self, body, code, **kwargs)


Refusals.refused = _recording_refused

EMITTED_REASONS = set()
_original_join = VP.join


def _recording_join(*arguments):
    disposition, reasons = _original_join(*arguments)
    EMITTED_REASONS.update(reasons)
    return disposition, reasons


VP.join = _recording_join


if __name__ == "__main__":
    # SiteCoverage reads what Refusals exercised, so it must run after it.
    loader = unittest.TestLoader()
    loader.sortTestMethodsUsing = None
    suite = unittest.TestSuite(
        loader.loadTestsFromTestCase(cls)
        for cls in (SchemaShape, Refusals, ResumeAndJoin, CommandLine, SiteCoverage)
    )
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    print(f"procedure tests: run={result.testsRun} failures={len(result.failures)} "
          f"errors={len(result.errors)} refusal_codes_exercised={len(EXERCISED_CODES)}")
    sys.exit(0 if result.wasSuccessful() else 1)
