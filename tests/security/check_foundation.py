"""T26 design/identity checks, not hostile execution or module admission."""
import copy
import hashlib
import json
from pathlib import Path
import tomllib
import unittest

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[2]


class Foundation(unittest.TestCase):
    def schema_part(self, name):
        schema = json.loads((ROOT / "tests/security/review-record-v1.schema.json").read_text())
        return Draft202012Validator({"$defs": schema["$defs"], "$ref": "#/$defs/" + name})

    def test_development_or_unqualified_profile_cannot_claim_adversarial_isolation(self):
        validator = self.schema_part("profile")
        # Fictional digest exercises structure, never qualification or custody.
        profile = dict(profile_id="TH-DEV/1", profile_sha256="sha256:" + "a" * 64,
                       adversarial_isolation=False, qualification_state="qualified")
        validator.validate(profile)
        profile["adversarial_isolation"] = True
        self.assertFalse(validator.is_valid(profile))
        profile["profile_id"] = "ADV-BWRAP/1"
        validator.validate(profile)
        profile["qualification_state"] = "unqualified"
        self.assertFalse(validator.is_valid(profile))
        profile["adversarial_isolation"] = False
        validator.validate(profile)

    def test_finding_closure_requires_repair_and_both_control_results(self):
        validator = self.schema_part("finding")
        # Deliberately fictional refs; a validator cannot authenticate evidence.
        reference = dict(schema_id="fictional-test-ref", sha256="sha256:" + "b" * 64, byte_length=1)
        row = dict(finding_id="fixture", state="open", severity="medium", location="fixture",
                   trigger="fault", impact="refusal required", intended_fault_case_id="fault",
                   benign_case_id="benign", finding_evidence=[],
                   repair=dict(pre_fix_subject_sha256=None, post_fix_subject_sha256=None,
                               diff_refs=[], affected_identity_refs=[]),
                   retest=dict(fault_result_refs=[], benign_result_refs=[], integrated_regression_refs=[], disposition="not-run"),
                   owner="worker", scope="schema fixture only", residual_obligations=[])
        validator.validate(row)
        row["state"] = "reproduced"
        self.assertFalse(validator.is_valid(row))
        row["finding_evidence"] = [reference]
        validator.validate(row)
        row["state"] = "fixed-pending-retest"
        self.assertFalse(validator.is_valid(row))
        row["repair"] = dict(pre_fix_subject_sha256="sha256:" + "a" * 64,
                             post_fix_subject_sha256="sha256:" + "c" * 64,
                             diff_refs=[reference], affected_identity_refs=[reference])
        validator.validate(row)
        row["state"] = "closed-with-exact-retest"
        row["retest"].update(fault_result_refs=[reference], disposition="passed-exact-subject")
        self.assertFalse(validator.is_valid(row))
        row["retest"]["benign_result_refs"] = [reference]
        validator.validate(row)
        row["retest"]["disposition"] = "failed"
        self.assertFalse(validator.is_valid(row))

    def test_declared_profiles_have_identical_fixed_candidate_bounds(self):
        profile = tomllib.loads((ROOT / "config/worker-profiles.toml").read_text())
        self.assertFalse(profile["execution_enabled"])
        self.assertEqual(set(profile["profiles"]), {"th_dev", "adv_bwrap"})
        expected = dict(task_seconds=1200, cleanup_reserve_seconds=300, term_grace_seconds=5,
                        empty_descendants_seconds=10, memory_bytes=8 * 1024**3,
                        memory_swap_bytes=0, tasks=128, cpu_percent=200, compiler_jobs=2,
                        julia_threads=1, blas_threads=1, scratch_bytes=4 * 1024**3,
                        exported_artifacts_bytes=64 * 1024**2,
                        stdout_bytes_per_command=8 * 1024**2, stderr_bytes_per_command=8 * 1024**2)
        for value in profile["profiles"].values():
            self.assertEqual(value["limits"], expected)
        self.assertFalse(profile["profiles"]["th_dev"]["adversarial_isolation"])
        self.assertEqual(profile["profiles"]["adv_bwrap"]["availability"], "unqualified-design-target")

    def test_threat_specs_are_unexecuted_and_uniquely_identified(self):
        data = json.loads((ROOT / "tests/security/threat-cases-v1.json").read_text())
        self.assertFalse(data["case_count_is_qualification_credit"])
        self.assertEqual(len(data["cases"]), 12)
        self.assertEqual(len({row["id"] for row in data["cases"]}), 12)

    def test_model_observation_matches_exact_session_lines(self):
        data = json.loads((ROOT / "evidence/implementation/T26/model-runtime-observation.json").read_text())
        lines = Path(data["source"]).read_bytes().splitlines(keepends=True)
        self.assertEqual(data["session_metadata"]["agent_path"], "/root/t01_security")
        self.assertFalse(data["selection_observed"])
        self.assertIsNone(data["effective_revision"])
        self.assertGreater(len(data["turn_observations"]), 0)
        for row in data["turn_observations"]:
            raw = lines[row["line"] - 1]
            self.assertEqual(hashlib.sha256(raw).hexdigest(), row["raw_line_sha256"])
            payload = json.loads(raw)["payload"]
            self.assertEqual(payload["model"], "gpt-daybreak-blue-latest")
            self.assertEqual(payload["effort"], row["effort"])
            self.assertEqual(payload["turn_id"], row["turn_id"])

    def test_actor_schema_refuses_false_effective_identity(self):
        schema = json.loads((ROOT / "tests/security/review-record-v1.schema.json").read_text())
        Draft202012Validator.check_schema(schema)
        validator = Draft202012Validator({"$defs": schema["$defs"], "$ref": "#/$defs/actor"})
        actual = json.loads((ROOT / "evidence/implementation/T26/model-runtime-observation.json").read_text())
        actor = dict(reviewer="/root/t01_security", requested_model=actual["requested_model"],
                     selection_surface="delegated-launch-override", selection_observed=False,
                     effective_model=actual["effective_model"], effective_revision=None,
                     effort=actual["effort"], review_model_override=None,
                     session_or_run_id=actual["session_metadata"]["id"],
                     observation_source=actual["source"], identity_verdict="effective-observed")
        validator.validate(actor)
        for key in ("effective_model", "effort", "session_or_run_id", "observation_source"):
            bad = copy.deepcopy(actor)
            bad[key] = None
            self.assertFalse(validator.is_valid(bad), key)
        actor["identity_verdict"] = "requested-only"
        self.assertFalse(validator.is_valid(actor))


if __name__ == "__main__":
    unittest.main(verbosity=2)
