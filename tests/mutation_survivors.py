#!/usr/bin/env python3
"""Control for `tools/check-mutation-survivors`.

Run: python3 -W error tests/mutation_survivors.py
No dependency beyond the standard library. The author of the audit also wrote these cases;
independent oracle authorship is NOT claimed.

An audit that cannot refuse is decoration, and one that refuses for the wrong reason looks
cleared and comes with evidence. So every case here asserts on the audit's own DIAGNOSTIC, not
merely on a non-zero exit, and the declarations file is restored byte-for-byte afterwards.

The cases are the ways an equivalence declaration can launder a real survivor:
an undeclared survivor; a declaration with no falsifier; an argument too short to be one; and
a run that never produced a `missed.txt`, which must refuse rather than read as zero survivors.
"""

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "tools/check-mutation-survivors"
DECLARATIONS = ROOT / "evidence/mutation-equivalences.json"
DECLARED = "src/budget.rs:1455:25: replace < with <= in Ledger::select_fallback"
UNDECLARED = "src/store.rs:99:9: replace Store::accept -> Result with Ok(())"


class Audit(unittest.TestCase):
    def setUp(self):
        self.work = Path(tempfile.mkdtemp(prefix="hee3-survivors-"))
        self.addCleanup(shutil.rmtree, self.work, ignore_errors=True)
        (self.work / "run").mkdir()
        self.original = DECLARATIONS.read_bytes()
        self.addCleanup(DECLARATIONS.write_bytes, self.original)

    def audit(self, survivors, full=False):
        (self.work / "run/missed.txt").write_text("".join(line + "\n" for line in survivors))
        argv = [sys.executable, str(AUDIT), str(self.work / "run")] + (["--full"] if full else [])
        return subprocess.run(argv, capture_output=True, text=True, check=False)

    def edit(self, mutate):
        body = json.loads(DECLARATIONS.read_text())
        mutate(body)
        DECLARATIONS.write_text(json.dumps(body, indent=2) + "\n")

    def test_a_declared_survivor_passes(self):
        result = self.audit([DECLARED])
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIn("undeclared=0", result.stdout)
        self.assertIn("verdict=PASS", result.stdout)

    def test_no_survivors_at_all_passes(self):
        result = self.audit([])
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIn("survivors=0", result.stdout)

    def test_an_undeclared_survivor_is_refused_by_name(self):
        result = self.audit([DECLARED, UNDECLARED])
        self.assertEqual(result.returncode, 1)
        self.assertIn("UNDECLARED " + UNDECLARED, result.stdout)
        self.assertIn("verdict=FAIL", result.stdout)

    def test_a_declaration_without_a_falsifier_is_refused(self):
        self.edit(lambda body: body["equivalences"][0].__setitem__("falsifier", ""))
        result = self.audit([DECLARED])
        self.assertEqual(result.returncode, 1)
        self.assertIn("has no falsifier", result.stdout)

    def test_a_declaration_without_pinning_cases_is_refused(self):
        self.edit(lambda body: body["equivalences"][0].__setitem__("pinned_by", []))
        result = self.audit([DECLARED])
        self.assertEqual(result.returncode, 1)
        self.assertIn("has no pinned_by", result.stdout)

    def test_a_declaration_citing_a_nonexistent_case_is_refused(self):
        # Written because I did this by hand: the first draft of the claims_are_disjoint
        # declaration cited `overlapping_claims_are_refused_at_assignment`, which was never a
        # test. A citation nobody resolves reads as evidence and is worth less than none.
        self.edit(lambda body: body["equivalences"][0].__setitem__(
            "pinned_by", ["a_case_that_was_never_written"]))
        result = self.audit([DECLARED])
        self.assertEqual(result.returncode, 1)
        self.assertIn("a_case_that_was_never_written", result.stdout)
        self.assertIn("not a function in this tree", result.stdout)

    def test_a_declaration_naming_an_absent_file_is_refused(self):
        self.edit(lambda body: body["equivalences"][0].__setitem__("file", "src/gone.rs"))
        result = self.audit([DECLARED])
        self.assertEqual(result.returncode, 1)
        self.assertIn("not in the tree", result.stdout)

    def test_a_token_argument_is_refused(self):
        self.edit(lambda body: body["equivalences"][0].__setitem__("argument", "It is fine."))
        result = self.audit([DECLARED])
        self.assertEqual(result.returncode, 1)
        self.assertIn("too short to be one", result.stdout)

    def test_an_absent_missed_file_refuses_rather_than_reading_as_zero(self):
        # An interrupted run must not print PASS. This is the shape that made a completeness
        # check read `found == tested` by construction and pass on a run that never finished.
        argv = [sys.executable, str(AUDIT), str(self.work / "run")]
        result = subprocess.run(argv, capture_output=True, text=True, check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("the run did not complete", result.stdout + result.stderr)

    def test_an_unmatched_declaration_is_reported_and_only_fails_a_full_run(self):
        # A scoped run need not produce every mutant, so an unmatched declaration is a note.
        # On a full run it means the code moved under the declaration, and that is a refusal.
        scoped = self.audit([DECLARED])
        self.assertEqual(scoped.returncode, 0)
        # Derived from the declarations file, not typed: a literal here moves every time a
        # declaration is added, and the case would then be pinning its own staleness.
        total = sum(len(e["mutants"]) for e in json.loads(self.original)["equivalences"])
        self.assertIn(f"unmatched_declarations={total - 1}", scoped.stdout)
        whole = self.audit([DECLARED], full=True)
        self.assertEqual(whole.returncode, 1)
        self.assertIn("unmatched declaration:", whole.stdout)

    def test_every_live_declaration_satisfies_the_audit_it_is_checked_by(self):
        # The declarations as they actually stand, not a fixture: a file that only passes in a
        # temporary copy would be a control measuring the wrong world.
        result = self.audit([])
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertNotIn("FAULT", result.stdout)


if __name__ == "__main__":
    result = unittest.main(exit=False, verbosity=1).result
    print(f"mutation survivor audit: run={result.testsRun} failures={len(result.failures)} "
          f"errors={len(result.errors)}")
    sys.exit(0 if result.wasSuccessful() else 1)
