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


def _load_generator_module():
    spec = importlib.util.spec_from_file_location(
        "hee3_generate_procedure_schema", ROOT / "workflows/generate_procedure_schema.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def catalogue_of(*pins):
    """A catalogue document whose request definitions pin each `(action, version)` given."""
    return {"$defs": {
        f"Request_{index}": {"properties": {"action": {"const": action},
                                            "action_version": {"const": version}}}
        for index, (action, version) in enumerate(pins)
    }}
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

    def test_generator_stops_on_two_pins_that_disagree(self):
        # WF-03: keeping the first pin would admit a version the catalogue also refuses.
        generator = _load_generator_module()
        catalogue = catalogue_of(("health", 1), ("task.get", 1), ("health", 2))
        with self.assertRaises(SystemExit) as caught:
            generator.catalogue_action_versions(catalogue, ("health", "task.get"))
        self.assertIn("pins 'health' at two versions", str(caught.exception))

    def test_generator_stops_on_an_admitted_action_with_no_pin(self):
        generator = _load_generator_module()
        catalogue = catalogue_of(("health", 1))
        with self.assertRaises(SystemExit) as caught:
            generator.catalogue_action_versions(catalogue, ("health", "task.get"))
        self.assertIn("pins no action_version for ['task.get']", str(caught.exception))

    def test_generator_reads_each_pin_as_the_catalogue_states_it(self):
        # Off the identity element: agreeing duplicate pins of 3 and 2 are kept, not reset.
        generator = _load_generator_module()
        catalogue = catalogue_of(("health", 3), ("task.get", 2), ("health", 3))
        self.assertEqual(generator.catalogue_action_versions(catalogue, ("task.get", "health")),
                         {"task.get": 2, "health": 3})

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
    def refused(self, body, code, committed=None, outcomes=None, verified=True, call=None):
        with self.assertRaises(VP.ProcedureRefusal) as caught:
            if call is not None:
                call()
            elif committed is not None:
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

    def test_committed_under_a_boolean_procedure_version(self):
        # True == 1 in Python; the record's version is an integer or it is not this version.
        detail = self.refused(procedure([step("a")]), "stale_procedure_version",
                              committed=record({}, version=True))
        self.assertIn("version True", detail)

    def test_join_refuses_outcomes_recorded_under_a_float_version(self):
        detail = self.refused(procedure([step("a")]), "stale_procedure_version",
                              outcomes=record({"a": "done"}, version=1.0))
        self.assertIn("version 1.0", detail)

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


def submit_and_read_back():
    """The composition's procedure: admit a task, then read it back by the submit's own key."""
    return procedure([step("submit", action="task.submit"),
                      step("verify", action="task.get", depends_on=["submit"])],
                     procedure_id="submit-and-read-back")


HELD = {"task.submit", "task.get"}
SPEC = {"task_class": "rust-library-change/1", "intent": "a spec with spaces, 'quotes' and $(x)"}


def submitted(key, effect="committed"):
    """A task.submit result as the engine answers one, carrying the readback by `key`."""
    return {"kind": "result", "effect": effect, "replayed": False,
            "readback": {"action": "task.get", "action_version": 1, "body": {
                "selector": {"source_action": "task.submit", "idempotency_key": key},
                "evidence": "none"}},
            "body": {"task": {"task_id": "28f00000-0000-4000-8000-000000000001",
                              "generation": "1", "state": "admitted"}}}


class Dispatch(unittest.TestCase):
    """WF-11: a ready step becomes a request; a reply becomes a committed record. No loop."""

    def test_a_step_key_is_a_uuid4_fixed_by_procedure_version_step_and_parent(self):
        body = submit_and_read_back()
        key = VP.step_key(body, "submit", None)
        self.assertRegex(key, r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
        self.assertEqual(key, VP.step_key(copy.deepcopy(body), "submit", None))
        other_version = copy.deepcopy(body)
        other_version["procedure_version"] = 2
        variants = {VP.step_key(body, "verify", None),
                    VP.step_key(body, "submit", "28f00000-0000-4000-8000-0000000000aa"),
                    VP.step_key(other_version, "submit", None)}
        self.assertEqual(len(variants | {key}), 4, "each coordinate moves the key")

    def test_a_ready_submit_step_is_the_wrapper_argv_with_its_key(self):
        body = submit_and_read_back()
        argv = VP.dispatch(body, record({}, "submit-and-read-back"), "submit", HELD, SPEC)
        self.assertEqual(argv, ["task.submit",
                                "@idempotency_key=" + VP.step_key(body, "submit", None),
                                "spec:=" + json.dumps(SPEC, sort_keys=True, separators=(",", ":"))])

    def test_a_read_step_reads_back_its_submit_dependency_by_key(self):
        body = submit_and_read_back()
        argv = VP.dispatch(body, record({"submit": "done"}, "submit-and-read-back"), "verify", HELD)
        selector = {"idempotency_key": VP.step_key(body, "submit", None),
                    "source_action": "task.submit"}
        self.assertEqual(argv, ["task.get",
                                "selector:=" + json.dumps(selector, sort_keys=True, separators=(",", ":")),
                                "evidence=none"])

    def test_a_step_resume_does_not_offer_is_not_dispatched(self):
        body = submit_and_read_back()
        detail = Refusals.refused(self, body, "not_ready", call=lambda: VP.dispatch(
            body, record({}, "submit-and-read-back"), "verify", HELD))
        self.assertIn("resume offers ['submit']", detail)

    def test_a_step_outside_the_held_actions_is_not_dispatched(self):
        body = submit_and_read_back()
        detail = Refusals.refused(self, body, "authority_widening", call=lambda: VP.dispatch(
            body, record({}, "submit-and-read-back"), "submit", {"task.get"}, SPEC))
        self.assertIn("'task.submit'", detail)

    def test_an_action_with_no_dispatch_arm_is_named(self):
        body = procedure([step("probe", action="health")])
        detail = Refusals.refused(self, body, "undispatchable_action", call=lambda: VP.dispatch(
            body, record({}), "probe", {"health"}))
        self.assertIn("no dispatch arm", detail)

    def test_a_submit_step_without_a_spec_is_not_dispatched(self):
        body = submit_and_read_back()
        detail = Refusals.refused(self, body, "undispatchable_action", call=lambda: VP.dispatch(
            body, record({}, "submit-and-read-back"), "submit", HELD))
        self.assertIn("needs the caller's task spec", detail)

    def test_a_read_step_needs_exactly_one_submit_dependency(self):
        body = procedure([step("verify", action="task.get")])
        detail = Refusals.refused(self, body, "undispatchable_action", call=lambda: VP.dispatch(
            body, record({}), "verify", HELD))
        self.assertIn("has 0", detail)

    def test_observing_a_committed_submit_records_it_done(self):
        body = submit_and_read_back()
        key = VP.step_key(body, "submit", None)
        after = VP.observe(body, record({}, "submit-and-read-back"), "submit", submitted(key))
        self.assertEqual(after, record({"submit": "done"}, "submit-and-read-back"))

    def test_an_uncertain_commit_is_recorded_effect_unknown_and_resume_stops(self):
        body = submit_and_read_back()
        key = VP.step_key(body, "submit", None)
        lost = {"kind": "error", "code": "effect_unknown", "effect": "unknown",
                "readback": submitted(key)["readback"]}
        after = VP.observe(body, record({}, "submit-and-read-back"), "submit", lost)
        self.assertEqual(after["steps"], {"submit": "effect_unknown"})
        Refusals.refused(self, body, "unreconciled_effect", committed=after)

    def test_a_refused_request_is_recorded_failed(self):
        body = submit_and_read_back()
        conflict = {"kind": "error", "code": "conflict", "effect": "none", "readback": None}
        after = VP.observe(body, record({}, "submit-and-read-back"), "submit", conflict)
        self.assertEqual(after["steps"], {"submit": "failed"})

    def test_a_reply_for_another_steps_key_is_refused(self):
        body = submit_and_read_back()
        foreign = submitted(VP.step_key(body, "verify", None))
        detail = Refusals.refused(self, body, "identity_mismatch", call=lambda: VP.observe(
            body, record({}, "submit-and-read-back"), "submit", foreign))
        self.assertIn("answers key", detail)

    def test_a_reply_that_is_not_a_control_record_is_refused(self):
        body = submit_and_read_back()
        for reply in ([], {"kind": "result"}, {"kind": "error"}, {"kind": "other", "effect": "none"}):
            with self.subTest(reply=reply):
                Refusals.refused(self, body, "malformed_reply", call=lambda: VP.observe(
                    body, record({}, "submit-and-read-back"), "submit", reply))

    def test_a_reply_for_a_step_the_procedure_lacks_is_refused(self):
        body = submit_and_read_back()
        detail = Refusals.refused(self, body, "identity_mismatch", call=lambda: VP.observe(
            body, record({}, "submit-and-read-back"), "ghost", submitted("x")))
        self.assertIn("ghost is not a step of 'submit-and-read-back'", detail)

    def test_a_step_already_committed_is_not_observed_again(self):
        body = submit_and_read_back()
        key = VP.step_key(body, "submit", None)
        detail = Refusals.refused(self, body, "not_ready", call=lambda: VP.observe(
            body, record({"submit": "done"}, "submit-and-read-back"), "submit", submitted(key)))
        self.assertIn("already committed", detail)

    def test_observing_does_not_change_the_record_it_was_given(self):
        body = submit_and_read_back()
        given = record({}, "submit-and-read-back")
        VP.observe(body, given, "submit", submitted(VP.step_key(body, "submit", None)))
        self.assertEqual(given, record({}, "submit-and-read-back"))


class Reconcile(unittest.TestCase):
    """WF-14: a reply lost after its effect is `effect_unknown`; a readback by the step's own key
    settles it, and only a settled step lets the procedure move on."""

    def lost(self):
        body = submit_and_read_back()
        return body, VP.unanswered(body, record({}, "submit-and-read-back"), "submit")

    def test_a_request_with_no_reply_is_effect_unknown_and_resume_stops(self):
        body, after = self.lost()
        self.assertEqual(after, record({"submit": "effect_unknown"}, "submit-and-read-back"))
        Refusals.refused(self, body, "unreconciled_effect", committed=after)

    def test_the_readback_that_settles_it_is_by_the_steps_own_key(self):
        body, after = self.lost()
        selector = {"idempotency_key": VP.step_key(body, "submit", None),
                    "source_action": "task.submit"}
        self.assertEqual(VP.reconcile_argv(body, after, "submit"),
                         ["task.get", "selector:=" + json.dumps(selector, sort_keys=True,
                                                                separators=(",", ":")),
                          "evidence=none"])

    def test_a_found_admission_settles_the_step_done(self):
        body, after = self.lost()
        found = {"kind": "result", "effect": "none",
                 "body": {"task": {"task_id": "28f00000-0000-4000-8000-000000000001"}}}
        settled = VP.reconcile(body, after, "submit", found)
        self.assertEqual(settled, record({"submit": "done"}, "submit-and-read-back"))
        self.assertEqual(VP.resume(body, settled), ["verify"])

    def test_an_absent_admission_releases_the_step_to_run_again_under_the_same_key(self):
        body, after = self.lost()
        absent = {"kind": "error", "code": "not_found", "effect": "none", "readback": None}
        released = VP.reconcile(body, after, "submit", absent)
        self.assertEqual(released, record({}, "submit-and-read-back"))
        self.assertEqual(VP.resume(body, released), ["submit"])

    def test_a_readback_that_cannot_answer_leaves_the_step_unknown(self):
        body, after = self.lost()
        busy = {"kind": "error", "code": "unavailable", "effect": "none", "readback": None}
        self.assertEqual(VP.reconcile(body, after, "submit", busy), after)

    def test_only_an_unknown_step_is_reconciled(self):
        body = submit_and_read_back()
        done = record({"submit": "done"}, "submit-and-read-back")
        for call in (lambda: VP.reconcile_argv(body, done, "submit"),
                     lambda: VP.reconcile(body, done, "submit", {"kind": "result", "body": {}})):
            detail = Refusals.refused(self, body, "not_ready", call=call)
            self.assertIn("is 'done', not effect_unknown", detail)

    def test_only_a_submit_step_has_a_readback_to_settle_it(self):
        body = procedure([step("probe", action="health")])
        unknown = record({"probe": "effect_unknown"})
        detail = Refusals.refused(self, body, "undispatchable_action",
                                  call=lambda: VP.reconcile_argv(body, unknown, "probe"))
        self.assertIn("no readback settles 'health'", detail)

    def test_a_readback_that_is_not_a_control_record_is_refused(self):
        body, after = self.lost()
        for reply in ([], {"kind": "error"}, {"kind": "result"}):
            with self.subTest(reply=reply):
                Refusals.refused(self, body, "malformed_reply",
                                 call=lambda: VP.reconcile(body, after, "submit", reply))

    def test_an_answered_step_is_not_marked_unanswered(self):
        body = submit_and_read_back()
        detail = Refusals.refused(self, body, "not_ready", call=lambda: VP.unanswered(
            body, record({"submit": "done"}, "submit-and-read-back"), "submit"))
        self.assertIn("already committed", detail)

    def test_an_unknown_step_name_is_refused(self):
        body = submit_and_read_back()
        detail = Refusals.refused(self, body, "identity_mismatch", call=lambda: VP.unanswered(
            body, record({}, "submit-and-read-back"), "ghost"))
        self.assertIn("ghost is not a step", detail)


class VersionSwitch(unittest.TestCase):
    """WF-16: v2 published while a v1 run is active. The v1 run resumes on v1; v2 cannot resume
    v1's record; and v2's steps derive other keys, so v2 never replays or collides with an
    effect v1 admitted."""

    def test_an_active_v1_run_resumes_on_v1_after_v2_is_published(self):
        v1 = submit_and_read_back()
        v2 = copy.deepcopy(v1)
        v2["procedure_version"] = 2
        running = VP.observe(v1, record({}, "submit-and-read-back"), "submit",
                             submitted(VP.step_key(v1, "submit", None)))
        self.assertEqual(VP.resume(v1, running), ["verify"])
        # The v1 read step reads back v1's admission, by v1's key.
        self.assertIn(VP.step_key(v1, "submit", None), VP.dispatch(v1, running, "verify", HELD)[1])
        detail = Refusals.refused(self, v2, "stale_procedure_version", committed=running)
        self.assertIn("committed under version 1; the procedure is version 2", detail)
        for step_id in ("submit", "verify"):
            with self.subTest(step=step_id):
                self.assertNotEqual(VP.step_key(v1, step_id, None), VP.step_key(v2, step_id, None))

    def test_a_v2_readback_does_not_settle_a_v1_step(self):
        v1 = submit_and_read_back()
        v2 = copy.deepcopy(v1)
        v2["procedure_version"] = 2
        lost = VP.unanswered(v1, record({}, "submit-and-read-back"), "submit")
        v2_answer = submitted(VP.step_key(v2, "submit", None))
        detail = Refusals.refused(self, v1, "identity_mismatch", call=lambda: VP.observe(
            v1, record({}, "submit-and-read-back"), "submit", v2_answer))
        self.assertIn("answers key", detail)
        Refusals.refused(self, v2, "stale_procedure_version",
                         call=lambda: VP.reconcile_argv(v2, lost, "submit"))


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
        for cls in (SchemaShape, Refusals, ResumeAndJoin, CommandLine, Dispatch, Reconcile,
                    VersionSwitch, SiteCoverage)
    )
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    print(f"procedure tests: run={result.testsRun} failures={len(result.failures)} "
          f"errors={len(result.errors)} refusal_codes_exercised={len(EXERCISED_CODES)}")
    sys.exit(0 if result.wasSuccessful() else 1)
