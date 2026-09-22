#!/usr/bin/env python3
"""HEE3-IF-bash wrapper tests.

Run: python3 -W error tests/bash_wrapper.py
No dependency beyond the standard library and bash. The wrapper author also implemented these
tests; independent oracle authorship is NOT claimed. No engine, transport or admission is
exercised: the producer is a fixture this file writes, and every case runs in a temporary
directory.

The contract's required proof is *"Whitespace/metacharacters; empty input; stderr/stdout
separation; failing producer through pipeline; cancellation and dependency/version
mismatch"*, and each class below names which part it covers.
"""

import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WRAPPER = ROOT / "integrations/bash/hee3"
CATALOGUE = ROOT / "schemas/actions/control-v1.schema.json"
EXIT_USAGE, EXIT_NO_PRODUCER, EXIT_BOUNDS = 2, 3, 4


class WrapperCase(unittest.TestCase):
    def setUp(self):
        self.work = Path(tempfile.mkdtemp(prefix="hee3-bash-"))
        self.addCleanup(shutil.rmtree, self.work, ignore_errors=True)

    def producer(self, body, name="producer"):
        """Write an executable producer fixture and return its path."""
        path = self.work / name
        path.write_text("#!/usr/bin/env bash\n" + textwrap.dedent(body))
        path.chmod(path.stat().st_mode | stat.S_IEXEC)
        return path

    def run_wrapper(self, *argv, producer=None, env=None):
        environment = dict(os.environ, HEE3_CATALOGUE=str(CATALOGUE), LC_ALL="C")
        environment.pop("HEE3_PRODUCER", None)
        if producer is not None:
            environment["HEE3_PRODUCER"] = str(producer)
        if env:
            environment.update(env)
        # Invoked through its interpreter, as `tools/check-quality` is invoked through
        # python3. The corpus publisher owns this file's mode -- `integrations/bash/hee3` is
        # listed in `corpus/publication-outputs.json` -- and normalises it to 0644 on every
        # generation. Depending on the executable bit made 54 cases fail with PermissionError
        # after a routine publish, which is a test depending on state it does not own.
        return subprocess.run(["bash", str(WRAPPER), *argv], capture_output=True, text=True,
                              env=environment, cwd=self.work, check=False)


class Syntax(WrapperCase):
    def test_the_script_parses(self):
        self.assertEqual(subprocess.run(["bash", "-n", str(WRAPPER)], check=False).returncode, 0)

    def test_the_wrapper_declares_its_interpreter(self):
        # The suite runs the wrapper through `bash` rather than relying on the executable
        # bit, so the shebang is what makes it runnable for anyone who does install it.
        self.assertTrue(WRAPPER.read_text().startswith("#!/usr/bin/env bash\n"))

    def test_the_script_contains_no_eval(self):
        # Not a general safety proof; the specific construct the contract forbids.
        source = WRAPPER.read_text()
        for forbidden in ("eval ", "eval\t", "eval\n"):
            self.assertNotIn(forbidden, source)


class Vocabulary(WrapperCase):
    def test_actions_are_the_catalogue(self):
        result = self.run_wrapper("--actions")
        self.assertEqual(result.returncode, 0, result.stderr)
        listed = result.stdout.split()
        catalogue = json.loads(CATALOGUE.read_text())["$defs"]["ActionId"]["enum"]
        self.assertEqual(listed, catalogue)

    def test_an_action_outside_the_catalogue_is_usage(self):
        result = self.run_wrapper("task.forge", producer=self.producer("exit 0\n"))
        self.assertEqual(result.returncode, EXIT_USAGE)
        self.assertIn("unknown action", result.stderr)

    def test_an_unreadable_catalogue_is_a_failure_not_an_empty_list(self):
        result = self.run_wrapper("--actions", env={"HEE3_CATALOGUE": str(self.work / "gone")})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")

    def test_an_empty_catalogue_is_a_failure(self):
        empty = self.work / "empty.json"
        empty.write_text(json.dumps({"$defs": {"ActionId": {"enum": []}}}))
        result = self.run_wrapper("--actions", env={"HEE3_CATALOGUE": str(empty)})
        self.assertEqual(result.returncode, EXIT_NO_PRODUCER)
        self.assertIn("empty", result.stderr)


class MissingProducer(WrapperCase):
    """*"missing producer is failure"* — never an empty success."""

    def test_unset_producer_fails(self):
        result = self.run_wrapper("health")
        self.assertEqual(result.returncode, EXIT_NO_PRODUCER)
        self.assertIn("HEE3_PRODUCER", result.stderr)

    def test_absent_producer_fails(self):
        result = self.run_wrapper("health", producer=self.work / "not-there")
        self.assertEqual(result.returncode, EXIT_NO_PRODUCER)

    def test_non_executable_producer_fails(self):
        path = self.work / "plain"
        path.write_text("#!/usr/bin/env bash\nexit 0\n")
        result = self.run_wrapper("health", producer=path)
        self.assertEqual(result.returncode, EXIT_NO_PRODUCER)
        self.assertIn("not executable", result.stderr)


class ProducerVerdict(WrapperCase):
    """*"Pipe/tee/display success must not swallow producer failure"*."""

    def test_success_is_passed_through(self):
        result = self.run_wrapper("health", producer=self.producer('printf "ok\\n"\n'))
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "ok\n")

    def test_a_failing_producer_fails_the_wrapper_through_the_pipe(self):
        # The producer PRINTS and THEN fails. A pipeline reports its last element, so a
        # wrapper reading `$?` rather than PIPESTATUS reports success here -- head always
        # succeeds. This is the case the whole PIPESTATUS discipline exists for.
        result = self.run_wrapper("health",
                                  producer=self.producer('printf "partial\\n"\nexit 7\n'))
        self.assertEqual(result.returncode, 7)
        self.assertEqual(result.stdout, "partial\n")
        self.assertIn("producer exited 7", result.stderr)

    def test_a_silent_failing_producer_still_fails(self):
        result = self.run_wrapper("health", producer=self.producer("exit 9\n"))
        self.assertEqual(result.returncode, 9)

    def test_every_producer_code_is_passed_through_unchanged(self):
        for code in (1, 7, 42, 126, 127):
            with self.subTest(code=code):
                result = self.run_wrapper(
                    "health", producer=self.producer(f"exit {code}\n"))
                self.assertEqual(result.returncode, code)

    def test_a_producer_killed_by_a_signal_is_not_a_success(self):
        result = self.run_wrapper("health", producer=self.producer("kill -TERM $$\n"))
        self.assertNotEqual(result.returncode, 0)

    def test_output_beyond_the_bound_does_not_become_a_success(self):
        # head closes the pipe; the producer dies of SIGPIPE. The wrapper must report that,
        # not the zero that head returns.
        body = 'exec 2>/dev/null\nwhile :; do printf "%s" "aaaaaaaaaaaaaaaa"; done\n'
        result = self.run_wrapper("health", producer=self.producer(body))
        self.assertNotEqual(result.returncode, 0)
        self.assertLessEqual(len(result.stdout), 1048576 + 1)


class Streams(WrapperCase):
    """*"stderr/stdout separation"*."""

    def test_producer_stderr_does_not_reach_stdout(self):
        result = self.run_wrapper(
            "health", producer=self.producer('printf "warn\\n" >&2\nprintf "{}\\n"\n'))
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "{}\n")
        self.assertIn("warn", result.stderr)

    def test_wrapper_diagnostics_do_not_reach_stdout(self):
        result = self.run_wrapper("health")
        self.assertEqual(result.stdout, "")
        self.assertNotEqual(result.stderr, "")

    def test_stdout_stays_parseable_when_the_producer_fails(self):
        result = self.run_wrapper(
            "health", producer=self.producer('printf \'{"error":"x"}\\n\'\nexit 5\n'))
        self.assertEqual(result.returncode, 5)
        self.assertEqual(json.loads(result.stdout), {"error": "x"})


class HostileValues(WrapperCase):
    """*"Whitespace/metacharacters; empty input"*.

    Each value is echoed back by a producer that reads the request from stdin, so the test
    asserts what ARRIVED, not merely that nothing exploded. A wrapper that dropped, split or
    executed a value would pass a smoke test and fail here.
    """

    ECHO = '''
        python3 -c '
import json, sys
print(json.dumps(json.load(sys.stdin)["arguments"], sort_keys=True))
'
    '''

    def arrived(self, *pairs):
        result = self.run_wrapper("task.submit", *pairs, producer=self.producer(self.ECHO))
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads(result.stdout)

    def test_a_value_with_spaces_arrives_whole(self):
        self.assertEqual(self.arrived("note=two  spaces  here"),
                         {"note": "two  spaces  here"})

    def test_command_substitution_is_data(self):
        self.assertEqual(self.arrived("note=$(touch pwned)"), {"note": "$(touch pwned)"})
        self.assertFalse((self.work / "pwned").exists())

    def test_backticks_are_data(self):
        self.assertEqual(self.arrived("note=`touch pwned2`"), {"note": "`touch pwned2`"})
        self.assertFalse((self.work / "pwned2").exists())

    def test_a_semicolon_is_data(self):
        self.assertEqual(self.arrived("note=a; touch pwned3"), {"note": "a; touch pwned3"})
        self.assertFalse((self.work / "pwned3").exists())

    def test_quotes_and_backslashes_survive_encoding(self):
        self.assertEqual(self.arrived('note="quoted" and \\ backslash'),
                         {"note": '"quoted" and \\ backslash'})

    def test_a_newline_inside_a_value_survives(self):
        self.assertEqual(self.arrived("note=first\nsecond"), {"note": "first\nsecond"})

    def test_a_glob_is_not_expanded(self):
        (self.work / "decoy.txt").write_text("x")
        self.assertEqual(self.arrived("note=*.txt"), {"note": "*.txt"})

    def test_a_leading_dash_in_a_value_is_data(self):
        self.assertEqual(self.arrived("note=--actions"), {"note": "--actions"})

    def test_an_empty_value_is_carried(self):
        self.assertEqual(self.arrived("note="), {"note": ""})

    def test_unicode_survives(self):
        self.assertEqual(self.arrived("note=café — 日本語"), {"note": "café — 日本語"})

    def test_several_arguments_keep_their_names(self):
        self.assertEqual(self.arrived("a=1", "b=2", "c="), {"a": "1", "b": "2", "c": ""})

    def test_a_value_containing_an_equals_sign_splits_only_once(self):
        self.assertEqual(self.arrived("filter=state=running"), {"filter": "state=running"})

    def test_an_argument_without_an_equals_sign_is_usage(self):
        result = self.run_wrapper("task.submit", "bare", producer=self.producer(self.ECHO))
        self.assertEqual(result.returncode, EXIT_USAGE)

    def test_an_argument_with_an_empty_name_is_usage(self):
        result = self.run_wrapper("task.submit", "=value", producer=self.producer(self.ECHO))
        self.assertEqual(result.returncode, EXIT_USAGE)

    def test_an_argument_named_twice_is_usage(self):
        result = self.run_wrapper("task.submit", "a=1", "a=2", producer=self.producer(self.ECHO))
        self.assertEqual(result.returncode, EXIT_USAGE)

    def test_no_arguments_is_a_valid_request(self):
        self.assertEqual(self.arrived(), {})


class Bounds(WrapperCase):
    def test_too_many_arguments_is_refused(self):
        pairs = [f"a{i}=1" for i in range(65)]
        result = self.run_wrapper("task.submit", *pairs, producer=self.producer("exit 0\n"))
        self.assertEqual(result.returncode, EXIT_BOUNDS)
        self.assertIn("64", result.stderr)

    def test_the_largest_allowed_argument_count_is_accepted(self):
        # The boundary from the other side: 64 must pass, or the bound is off by one and
        # only the refusing half was ever checked.
        pairs = [f"a{i}=1" for i in range(64)]
        result = self.run_wrapper("task.submit", *pairs, producer=self.producer("exit 0\n"))
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_an_oversized_value_is_refused(self):
        result = self.run_wrapper("task.submit", "note=" + "x" * 8193,
                                  producer=self.producer("exit 0\n"))
        self.assertEqual(result.returncode, EXIT_BOUNDS)

    def test_a_value_at_the_bound_is_accepted(self):
        pair = "note=" + "x" * (8192 - len("note="))
        result = self.run_wrapper("task.submit", pair, producer=self.producer("exit 0\n"))
        self.assertEqual(result.returncode, 0, result.stderr)


class Inspection(WrapperCase):
    """*"Inspect usage and prerequisites ... dependency/version diagnostics"*."""

    def test_version_reports_the_wrapper_and_the_producer(self):
        result = self.run_wrapper("--version", producer=self.producer("exit 0\n"))
        self.assertEqual(result.returncode, 0)
        self.assertIn("wrapper_version: 1", result.stdout)
        self.assertIn("catalogue_actions: 21", result.stdout)

    def test_version_names_an_absent_producer(self):
        self.assertIn("producer: ABSENT", self.run_wrapper("--version").stdout)

    def test_inspect_reports_the_bounds_it_enforces(self):
        result = self.run_wrapper("--inspect", "health", producer=self.producer("exit 0\n"))
        self.assertEqual(result.returncode, 0)
        for line in ("action: health", "max_args: 64", "max_value_bytes: 8192"):
            self.assertIn(line, result.stdout)

    def test_inspect_works_without_a_producer(self):
        result = self.run_wrapper("--inspect", "health")
        self.assertEqual(result.returncode, 0)
        self.assertIn("producer: ABSENT", result.stdout)

    def test_inspect_of_an_unknown_action_is_usage(self):
        self.assertEqual(self.run_wrapper("--inspect", "task.forge").returncode, EXIT_USAGE)

    def test_inspect_requires_exactly_one_action(self):
        self.assertEqual(self.run_wrapper("--inspect").returncode, EXIT_USAGE)
        self.assertEqual(self.run_wrapper("--inspect", "health", "task.get").returncode,
                         EXIT_USAGE)

    def test_no_arguments_prints_usage_to_stderr(self):
        result = self.run_wrapper()
        self.assertEqual(result.returncode, EXIT_USAGE)
        self.assertEqual(result.stdout, "")
        self.assertIn("hee3 <action>", result.stderr)

    def test_an_unknown_option_is_usage(self):
        self.assertEqual(self.run_wrapper("--wat").returncode, EXIT_USAGE)


class RequestShape(WrapperCase):
    def test_the_request_is_well_formed_json_with_the_action(self):
        producer = self.producer("cat\n")
        result = self.run_wrapper("task.get", "task_id=abc", producer=producer)
        self.assertEqual(result.returncode, 0)
        request = json.loads(result.stdout)
        self.assertEqual(request["protocol"], "hee3.control")
        self.assertEqual(request["action"], "task.get")
        self.assertEqual(request["arguments"], {"task_id": "abc"})

    def test_trailing_newlines_are_not_edited_away(self):
        # $(...) strips them. A wrapper that reports its producer's result must not rewrite
        # the bytes of it, and a count of two is off the identity element that one would be.
        producer = self.producer(r'printf "{}\n\n\n"' + "\n")
        result = self.run_wrapper("health", producer=producer)
        self.assertEqual(result.stdout, "{}\n\n\n")

    def test_output_without_a_final_newline_gets_exactly_one(self):
        producer = self.producer(r'printf "no-newline"' + "\n")
        result = self.run_wrapper("health", producer=producer)
        self.assertEqual(result.stdout, "no-newline\n")

    def test_empty_producer_output_stays_empty(self):
        result = self.run_wrapper("health", producer=self.producer("exit 0\n"))
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")

    def test_interior_blank_lines_survive(self):
        producer = self.producer(r'printf "a\n\nb\n"' + "\n")
        self.assertEqual(self.run_wrapper("health", producer=producer).stdout, "a\n\nb\n")

    def test_the_producer_receives_the_action_as_its_argument(self):
        producer = self.producer('printf "%s\\n" "$1"\n')
        result = self.run_wrapper("task.list", producer=producer)
        self.assertEqual(result.stdout, "task.list\n")


if __name__ == "__main__":
    result = unittest.main(exit=False, verbosity=1).result
    print(f"bash wrapper tests: run={result.testsRun} failures={len(result.failures)} "
          f"errors={len(result.errors)}")
    sys.exit(0 if result.wasSuccessful() else 1)
