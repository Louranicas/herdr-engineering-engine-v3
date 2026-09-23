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

import ctypes
import json
import os
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import textwrap
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WRAPPER = ROOT / "integrations/bash/hee3"
CATALOGUE = ROOT / "schemas/actions/control-v1.schema.json"
EXIT_USAGE, EXIT_NO_PRODUCER, EXIT_BOUNDS, EXIT_TIMEOUT, EXIT_CONTRACT = 2, 3, 4, 5, 6
RUN_BUDGET_S = 30
PR_SET_CHILD_SUBREAPER = 36
REAP_BUDGET_S = 10.0
LIVE_GRACE_S = 1.0


def become_subreaper():
    """Adopt this suite's orphans instead of handing them to whoever runs it.

    The chain cases kill process trees on purpose, and a parent that dies before its child
    orphans it. Without this, every orphan -- dead or alive -- re-parents to the nearest
    subreaper above the suite: tools/check-quality, which rightly reports it as a descendant
    the step left behind. The suite owns what it starts; it reaps it itself.
    """
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), "prctl(PR_SET_CHILD_SUBREAPER)")


def adopted():
    """This process's children that it did not wait for itself: adopted orphans."""
    return [int(pid) for pid in
            Path(f"/proc/self/task/{os.getpid()}/children").read_text().split()]


def reap_adopted():
    """Reap adopted zombies; fail on any adopted descendant still ALIVE after the test.

    A dead orphan is the expected residue of killing a tree. A live one is a custody leak,
    and it is attributed to the test that left it rather than surfacing as an anonymous
    `descendants_detected` on the whole suite. Bounded: a descendant that will not die fails
    the case with both numbers, it does not hang it.
    """
    deadline = time.monotonic() + REAP_BUDGET_S
    first_seen, live = {}, set()
    while True:
        remaining = adopted()
        if not remaining:
            break
        now = time.monotonic()
        for pid in remaining:
            try:
                state = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[0]
            except FileNotFoundError:
                continue
            # A process mid-exit is briefly not yet a zombie; only one still running a full
            # second after it was first seen is a leak.
            if state != "Z" and now - first_seen.setdefault(pid, now) > LIVE_GRACE_S:
                live.add(pid)
                os.kill(pid, signal.SIGKILL)
            try:
                os.waitpid(pid, os.WNOHANG)
            except ChildProcessError:
                pass
        if time.monotonic() > deadline:
            raise AssertionError(f"{len(remaining)} adopted descendants survived "
                                 f"{REAP_BUDGET_S}s: {remaining}")
        time.sleep(0.01)
    if live:
        raise AssertionError(f"the case left {len(live)} live descendant(s): {sorted(live)}")


class WrapperCase(unittest.TestCase):
    def setUp(self):
        # Registered first, so it runs LAST: every other cleanup -- a Popen killed and
        # waited -- has collected its own child before this one reaps what is left.
        self.addCleanup(reap_adopted)
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
        # A budget on every invocation (F102): without one, a step that leaks a descendant
        # holding the pipe makes the case hang for as long as the descendant lives, which
        # reads as a slow suite rather than the custody failure it is. The descendants the
        # chain cases plant sleep 60 s, so this bound is what turns a leak into a failure.
        return subprocess.run(["bash", str(WRAPPER), *argv], capture_output=True, text=True,
                              env=environment, cwd=self.work, check=False,
                              timeout=RUN_BUDGET_S)


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


class Check(WrapperCase):
    """`--check` is the door `invoke` and `chain` share; it must refuse what they refuse."""

    def test_check_prints_the_request_and_needs_no_producer(self):
        result = self.run_wrapper("--check", "task.get", "task_id=$(x) y")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout),
                         {"action": "task.get", "arguments": {"task_id": "$(x) y"},
                          "protocol": "hee3.control", "version": 1})

    def test_check_does_not_invoke_the_producer(self):
        marker = self.work / "invoked"
        result = self.run_wrapper("--check", "health",
                                  producer=self.producer(f"touch '{marker}'\n"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(marker.exists())

    def test_check_refuses_what_invoke_refuses(self):
        cases = ((("task.forge",), EXIT_USAGE),
                 (("task.get", "bare"), EXIT_USAGE),
                 (("task.get", "note=" + "x" * 8193), EXIT_BOUNDS),
                 (("task.get", *[f"a{i}=1" for i in range(65)]), EXIT_BOUNDS))
        for argv, code in cases:
            with self.subTest(argv=argv[:2]):
                self.assertEqual(self.run_wrapper("--check", *argv).returncode, code)

    def test_check_without_an_action_is_usage(self):
        self.assertEqual(self.run_wrapper("--check").returncode, EXIT_USAGE)


class Cleanup(WrapperCase):
    """*"cancellation"* for a single invocation: a cancelled wrapper leaves no scratch."""

    def test_a_terminated_invocation_removes_its_scratch_directory(self):
        tmp = self.work / "tmp"
        tmp.mkdir()
        started = self.work / "started"
        producer = self.producer(f"touch '{started}'\nexec sleep 30\n")
        environment = dict(os.environ, HEE3_CATALOGUE=str(CATALOGUE), LC_ALL="C",
                           HEE3_PRODUCER=str(producer), TMPDIR=str(tmp))
        process = subprocess.Popen(["bash", str(WRAPPER), "health"], env=environment,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   start_new_session=True)
        self.addCleanup(process.stderr.close)
        self.addCleanup(process.stdout.close)
        wait_for(started)
        self.assertEqual(len(list(tmp.iterdir())), 1, "the invocation made no scratch")
        os.killpg(process.pid, signal.SIGTERM)
        self.assertEqual(process.wait(timeout=10), 143)
        self.assertEqual(list(tmp.iterdir()), [])


def wait_for(path, budget=10.0):
    """Every loop in a test needs a budget, and the failure names both numbers."""
    deadline = time.monotonic() + budget
    while not path.exists():
        if time.monotonic() > deadline:
            raise AssertionError(f"{path.name} did not appear within {budget}s")
        time.sleep(0.02)


def pid_alive(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    # A zombie still answers kill(0); it is dead for every purpose that matters here.
    try:
        return Path(f"/proc/{pid}/stat").read_text().split(")")[-1].split()[0] != "Z"
    except FileNotFoundError:
        return False


class ChainCase(WrapperCase):
    """*"Chain bounded commands: declared input/output contracts, cancellation and timeout
    propagation -> ordered results and preserved failing producer status."*

    The producer is a MODEL, not a script: it appends every call it receives -- action,
    step, budget and the arguments that arrived -- to a log, so a case asserts on what each
    step was handed, not merely on the exit code the chain chose to report.
    """

    MODEL = r'''
        request=$(cat)
        python3 - "$1" "$request" "$HEE3_CHAIN_STEP" "${HEE3_TIMEOUT_MS:-}" <<'PY' >>"$LOG"
import json, sys
action, request, step, budget = sys.argv[1:5]
print(json.dumps({"action": action, "step": step, "budget": budget,
                  "arguments": json.loads(request)["arguments"]}, sort_keys=True))
PY
        behaviour="$BEHAVIOUR_DIR/$HEE3_CHAIN_STEP"
        if [[ -f $behaviour ]]; then source "$behaviour"; fi
    '''

    def setUp(self):
        super().setUp()
        self.log = self.work / "calls.jsonl"
        self.behaviours = self.work / "behaviour"
        self.behaviours.mkdir()
        self.model = self.producer(self.MODEL, name="model")

    def behave(self, step, body):
        """What the model does for one step, after it has logged the call."""
        (self.behaviours / step).write_text(textwrap.dedent(body))

    def calls(self):
        if not self.log.exists():
            return []
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def spec(self, steps, timeout_ms=20000, **extra):
        document = {"protocol": "hee3.chain", "version": 1, "timeout_ms": timeout_ms,
                    "steps": steps, **extra}
        path = self.work / "chain.json"
        path.write_text(json.dumps(document))
        return path

    def environment(self):
        return {"LOG": str(self.log), "BEHAVIOUR_DIR": str(self.behaviours)}

    def chain(self, steps, timeout_ms=20000, producer="model"):
        path = self.spec(steps, timeout_ms)
        return self.run_wrapper("chain", str(path),
                                producer=self.model if producer == "model" else producer,
                                env=self.environment())

    def records(self, result):
        return [json.loads(line) for line in result.stdout.splitlines()]


class ChainOrder(ChainCase):
    def test_steps_run_in_declared_order_and_report_in_that_order(self):
        steps = [{"id": name, "action": action}
                 for name, action in (("third", "health"), ("first", "task.list"),
                                      ("second", "tools.list"))]
        result = self.chain(steps)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call["step"] for call in self.calls()], ["third", "first", "second"])
        records = self.records(result)
        self.assertEqual([(r["kind"], r.get("step")) for r in records],
                         [("step", "third"), ("step", "first"), ("step", "second"),
                          ("summary", None)])

    def test_records_after_the_first_are_whole_and_carry_their_own_fields(self):
        # Off the identity element: rows 2 and 3, asserted whole, differing in every field.
        self.behave("b", 'printf "bee\\n"\n')
        self.behave("c", 'printf "sea\\n\\n"\n')
        result = self.chain([{"id": "a", "action": "health"},
                             {"id": "b", "action": "task.list"},
                             {"id": "c", "action": "tools.inspect"}])
        self.assertEqual(result.returncode, 0, result.stderr)
        records = self.records(result)
        self.assertEqual(records[1], {"kind": "step", "index": 1, "step": "b",
                                      "action": "task.list", "status": 0, "outcome": "ok",
                                      "stdout": "bee\n"})
        self.assertEqual(records[2], {"kind": "step", "index": 2, "step": "c",
                                      "action": "tools.inspect", "status": 0,
                                      "outcome": "ok", "stdout": "sea\n\n"})
        self.assertEqual(records[3], {"kind": "summary", "outcome": "ok", "status": 0,
                                      "ran": ["a", "b", "c"], "not_run": [],
                                      "failed_step": None})

    def test_spec_can_be_read_from_stdin(self):
        path = self.spec([{"id": "only", "action": "health"}])
        environment = dict(os.environ, HEE3_CATALOGUE=str(CATALOGUE), LC_ALL="C",
                           HEE3_PRODUCER=str(self.model), **self.environment())
        result = subprocess.run(["bash", str(WRAPPER), "chain", "-"], input=path.read_text(),
                                capture_output=True, text=True, env=environment,
                                cwd=self.work, check=False, timeout=RUN_BUDGET_S)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call["step"] for call in self.calls()], ["only"])

    def test_step_stderr_reaches_stderr_and_not_the_records(self):
        self.behave("a", 'printf "a warning\\n" >&2\nprintf "{}\\n"\n')
        result = self.chain([{"id": "a", "action": "health", "output": "json"}])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("a warning", result.stderr)
        self.assertNotIn("a warning", result.stdout)
        self.assertEqual(self.records(result)[0]["result"], {})


class ChainFailure(ChainCase):
    """*"preserved failing producer status"*."""

    def test_the_failing_producers_status_is_the_chains_status(self):
        for code in (7, 42):
            with self.subTest(code=code):
                self.log.unlink(missing_ok=True)
                self.behave("mid", f'printf "partial\\n"\nexit {code}\n')
                result = self.chain([{"id": "pre", "action": "health"},
                                     {"id": "mid", "action": "task.list"},
                                     {"id": "post", "action": "task.get"}])
                self.assertEqual(result.returncode, code)
                records = self.records(result)
                self.assertEqual(records[1]["status"], code)
                self.assertEqual(records[1]["outcome"], "failed")
                self.assertEqual(records[1]["stdout"], "partial\n")
                self.assertEqual(records[-1]["failed_step"], "mid")
                self.assertEqual(records[-1]["not_run"], ["post"])
                self.assertEqual(records[-1]["status"], code)

    def test_a_step_after_a_failure_is_never_invoked(self):
        self.behave("mid", "exit 9\n")
        self.chain([{"id": "pre", "action": "health"}, {"id": "mid", "action": "health"},
                    {"id": "post", "action": "health"}])
        self.assertEqual([call["step"] for call in self.calls()], ["pre", "mid"])

    def test_a_producer_killed_by_a_signal_fails_the_chain_and_is_not_a_cancellation(self):
        self.behave("a", "kill -KILL $$\n")
        result = self.chain([{"id": "a", "action": "health"}, {"id": "b", "action": "health"}])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.records(result)[-1]["outcome"], "failed")

    def test_a_missing_producer_runs_nothing(self):
        result = self.run_wrapper("chain", str(self.spec([{"id": "a", "action": "health"}])),
                                  env=self.environment())
        self.assertEqual(result.returncode, EXIT_NO_PRODUCER)
        self.assertEqual(result.stdout, "")
        self.assertEqual(self.calls(), [])


class ChainContracts(ChainCase):
    """*"declared input/output contracts"* -- checked twice: statically, before the first
    step runs, and against what each step actually produced."""

    def test_a_declared_field_arrives_as_one_literal_argument(self):
        hostile = "two  spaces; $(touch pwned) `touch pwned2` \"q\" \\ *.txt\nline"
        self.behave("find", f'printf "%s\\n" {shlex_quote(json.dumps({"task_id": hostile}))}\n')
        result = self.chain([
            {"id": "find", "action": "task.list", "output": "json", "provides": ["task_id"]},
            {"id": "get", "action": "task.get", "arguments": {"view": "full"},
             "inputs": {"task_id": {"step": "find", "field": "task_id"}}}])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls()[1]["arguments"], {"task_id": hostile, "view": "full"})
        self.assertFalse((self.work / "pwned").exists())
        self.assertFalse((self.work / "pwned2").exists())
        self.assertEqual(self.records(result)[0]["result"], {"task_id": hostile})

    def test_literal_arguments_arrive_whole(self):
        value = "a=b; $(x)"
        result = self.chain([{"id": "a", "action": "task.get", "arguments": {"q": value}}])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls()[0]["arguments"], {"q": value})

    def static_refusal(self, steps, needle):
        result = self.chain(steps)
        self.assertEqual(result.returncode, EXIT_USAGE, result.stderr)
        self.assertIn(needle, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertEqual(self.calls(), [], "a step ran before the spec was refused")

    def test_an_input_from_a_later_step_is_refused_before_anything_runs(self):
        self.static_refusal([
            {"id": "get", "action": "task.get",
             "inputs": {"task_id": {"step": "find", "field": "task_id"}}},
            {"id": "find", "action": "task.list", "output": "json", "provides": ["task_id"]}],
            "not an earlier step")

    def test_an_input_reading_an_undeclared_field_is_refused(self):
        self.static_refusal([
            {"id": "find", "action": "task.list", "output": "json", "provides": ["id"]},
            {"id": "get", "action": "task.get",
             "inputs": {"task_id": {"step": "find", "field": "task_id"}}}],
            "does not declare in provides")

    def test_a_text_step_cannot_provide_fields(self):
        self.static_refusal([
            {"id": "find", "action": "task.list", "provides": ["task_id"]}],
            "only a json output can provide")

    def test_a_name_that_is_both_literal_and_input_is_refused(self):
        self.static_refusal([
            {"id": "find", "action": "task.list", "output": "json", "provides": ["t"]},
            {"id": "get", "action": "task.get", "arguments": {"t": "x"},
             "inputs": {"t": {"step": "find", "field": "t"}}}],
            "both a literal argument and an input")

    def test_an_equals_sign_in_an_argument_name_is_refused(self):
        # It would be re-split into a different name and value by the request builder.
        self.static_refusal([{"id": "a", "action": "task.get", "arguments": {"a=b": "c"}}],
                            "without '='")

    def test_a_non_string_argument_value_is_refused(self):
        self.static_refusal([{"id": "a", "action": "task.get", "arguments": {"n": 5}}],
                            "must be a string")

    def test_an_unknown_action_in_a_late_step_stops_the_chain_before_the_first(self):
        result = self.chain([{"id": "a", "action": "health"}, {"id": "b", "action": "health"},
                             {"id": "c", "action": "task.forge"}])
        self.assertEqual(result.returncode, EXIT_USAGE)
        self.assertIn("unknown action: task.forge", result.stderr)
        self.assertIn("'c' was refused before any step ran", result.stderr)
        self.assertEqual(self.calls(), [])

    def test_an_oversized_literal_in_a_late_step_is_a_bound_before_the_first(self):
        result = self.chain([{"id": "a", "action": "health"},
                             {"id": "b", "action": "task.get",
                              "arguments": {"note": "x" * 8193}}])
        self.assertEqual(result.returncode, EXIT_BOUNDS)
        self.assertEqual(self.calls(), [])

    def test_an_input_counts_toward_the_argument_bound_before_anything_runs(self):
        # 64 literals fit; the 65th argument is an input whose value is not known yet. The
        # upfront check must count it by name, or step 1 runs and step 2 is refused after.
        literals = {f"a{i}": "1" for i in range(64)}
        result = self.chain([
            {"id": "find", "action": "task.list", "output": "json", "provides": ["t"]},
            {"id": "get", "action": "task.get", "arguments": literals,
             "inputs": {"t": {"step": "find", "field": "t"}}}])
        self.assertEqual(result.returncode, EXIT_BOUNDS)
        self.assertEqual(self.calls(), [])

    def test_an_oversized_produced_value_is_refused_by_the_same_bound(self):
        self.behave("find", 'python3 -c \'import json; print(json.dumps({"t": "x" * 9000}))\'\n')
        result = self.chain([
            {"id": "find", "action": "task.list", "output": "json", "provides": ["t"]},
            {"id": "get", "action": "task.get", "inputs": {"t": {"step": "find", "field": "t"}}},
            {"id": "never", "action": "health"}])
        self.assertEqual(result.returncode, EXIT_BOUNDS)
        self.assertEqual([call["step"] for call in self.calls()], ["find"])
        self.assertEqual(self.records(result)[-1]["not_run"], ["never"])

    def contract_failure(self, body, detail, provides=("task_id",)):
        self.behave("find", body)
        result = self.chain([
            {"id": "find", "action": "task.list", "output": "json", "provides": list(provides)},
            {"id": "get", "action": "task.get",
             "inputs": {"task_id": {"step": "find", "field": "task_id"}}}])
        self.assertEqual(result.returncode, EXIT_CONTRACT, result.stderr)
        records = self.records(result)
        self.assertEqual(records[0]["outcome"], "contract")
        self.assertEqual(records[0]["status"], 0)
        self.assertEqual(records[-1]["failed_step"], "find")
        self.assertEqual(records[-1]["not_run"], ["get"])
        self.assertIn(detail, records[-1]["detail"])
        self.assertEqual([call["step"] for call in self.calls()], ["find"])

    def test_declared_json_that_is_not_json_is_a_contract_failure(self):
        self.contract_failure('printf "not json\\n"\n', "not JSON")

    def test_a_declared_field_that_is_absent_is_a_contract_failure(self):
        self.contract_failure('printf \'{"other": "x"}\\n\'\n', "task_id")

    def test_a_declared_field_that_is_not_a_string_is_a_contract_failure(self):
        # Coercing 5 to "5" would be the runner deciding what the producer meant.
        self.contract_failure('printf \'{"task_id": 5}\\n\'\n', "not a string")

    def test_output_with_a_duplicated_key_is_a_contract_failure(self):
        self.contract_failure('printf \'{"task_id": "a", "task_id": "b"}\\n\'\n', "twice")

    def test_stdout_that_is_not_utf8_is_a_contract_failure(self):
        self.behave("a", "printf '\\xff\\xfe\\n'\n")
        result = self.chain([{"id": "a", "action": "health"}, {"id": "b", "action": "health"}])
        self.assertEqual(result.returncode, EXIT_CONTRACT)
        self.assertEqual(self.records(result)[-1]["detail"], "stdout is not UTF-8")


class ChainSpec(ChainCase):
    """The spec is acquired with its bound and parsed strictly; every refusal runs nothing."""

    def refused(self, document, needle, raw=None):
        path = self.work / "chain.json"
        path.write_text(raw if raw is not None else json.dumps(document))
        result = self.run_wrapper("chain", str(path), producer=self.model,
                                  env=self.environment())
        self.assertEqual(result.returncode, EXIT_USAGE, result.stderr)
        self.assertIn(needle, result.stderr)
        self.assertEqual(self.calls(), [])

    def base(self, **changes):
        document = {"protocol": "hee3.chain", "version": 1, "timeout_ms": 1000,
                    "steps": [{"id": "a", "action": "health"}]}
        document.update(changes)
        return document

    def test_sixteen_steps_are_accepted_and_seventeen_refused(self):
        steps = [{"id": f"s{i}", "action": "health"} for i in range(16)]
        self.assertEqual(self.chain(steps).returncode, 0)
        self.log.unlink()
        self.refused(self.base(steps=steps + [{"id": "s16", "action": "health"}]),
                     "1 to 16 steps")

    def test_no_steps_is_refused(self):
        self.refused(self.base(steps=[]), "1 to 16 steps")

    def test_a_spec_over_the_byte_bound_is_refused(self):
        self.refused(None, "over 65536 bytes", raw=" " * 65537)

    def test_a_spec_at_the_byte_bound_is_read(self):
        text = json.dumps(self.base())
        path = self.work / "chain.json"
        path.write_text(text + " " * (65536 - len(text)))
        result = self.run_wrapper("chain", str(path), producer=self.model,
                                  env=self.environment())
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_an_unreadable_spec_is_refused(self):
        result = self.run_wrapper("chain", str(self.work / "absent.json"), producer=self.model)
        self.assertEqual(result.returncode, EXIT_USAGE)
        self.assertIn("cannot read the spec", result.stderr)

    def test_chain_takes_exactly_one_spec(self):
        self.assertEqual(self.run_wrapper("chain", producer=self.model).returncode, EXIT_USAGE)
        self.assertEqual(self.run_wrapper("chain", "a", "b", producer=self.model).returncode,
                         EXIT_USAGE)

    def test_a_duplicated_key_in_the_spec_is_refused(self):
        self.refused(None, "appears twice",
                     raw='{"protocol":"hee3.chain","version":1,"timeout_ms":5,"timeout_ms":9,'
                         '"steps":[{"id":"a","action":"health"}]}')

    def test_an_unknown_key_is_refused_at_each_level(self):
        self.refused(self.base(retries=3), "unknown key(s) retries")
        self.refused(self.base(steps=[{"id": "a", "action": "health", "parallel": True}]),
                     "unknown key(s) parallel")

    def test_protocol_and_version_are_checked(self):
        self.refused(self.base(protocol="hee3.control"), "protocol must be hee3.chain")
        self.refused(self.base(version=2), "version 2 is not supported")
        self.refused(self.base(version=True), "is not supported")

    def test_budgets_are_bounded_from_both_sides(self):
        self.refused(self.base(timeout_ms=0), "from 1 to 3600000")
        self.refused(self.base(timeout_ms=3600001), "from 1 to 3600000")
        self.refused(self.base(timeout_ms=True), "from 1 to 3600000")
        self.refused(self.base(timeout_ms=1.5), "from 1 to 3600000")
        # A step may not claim more than the chain has.
        self.refused(self.base(steps=[{"id": "a", "action": "health", "timeout_ms": 1001}]),
                     "from 1 to 1000")

    # One case per refusal SITE, each asserting the detail only that site produces: a case per
    # reason name let neighbouring sites answer for each other (F140). tools/check-bash-sites
    # neuters every site and requires the suite to notice.

    def test_a_spec_that_is_not_utf8_is_refused(self):
        path = self.work / "chain.json"
        path.write_bytes(b'{"protocol": "\xff"}')
        result = self.run_wrapper("chain", str(path), producer=self.model, env=self.environment())
        self.assertEqual(result.returncode, EXIT_USAGE)
        self.assertIn("the spec is not UTF-8", result.stderr)

    def test_a_spec_that_is_not_an_object_is_refused(self):
        self.refused(None, "the spec is not an object", raw="[]")

    def test_a_step_that_is_not_an_object_is_refused(self):
        self.refused(self.base(steps=["health"]), "step 0 is not an object")

    def test_a_missing_top_level_key_is_named(self):
        document = self.base()
        del document["steps"]
        self.refused(document, "the spec lacks steps")

    def test_a_step_without_an_action_is_named(self):
        self.refused(self.base(steps=[{"id": "a"}]), "step 0 lacks action")

    def test_a_non_string_action_is_refused(self):
        self.refused(self.base(steps=[{"id": "a", "action": ["health"]}]),
                     "step 'a': action must be a string")

    def test_arguments_that_are_not_an_object_are_refused(self):
        self.refused(self.base(steps=[{"id": "a", "action": "health", "arguments": ["x=1"]}]),
                     "step 'a': arguments must be an object")

    def test_an_unknown_output_kind_is_refused(self):
        self.refused(self.base(steps=[{"id": "a", "action": "health", "output": "xml"}]),
                     "step 'a': output must be text or json")

    def test_provides_must_be_a_list_of_distinct_names(self):
        for provides in ("task_id", ["a", "a"], [""], [5]):
            with self.subTest(provides=provides):
                self.refused(self.base(steps=[{"id": "a", "action": "health", "output": "json",
                                               "provides": provides}]),
                             "step 'a': provides must be a list of at most 64")

    def test_inputs_that_are_not_an_object_are_refused(self):
        self.refused(self.base(steps=[{"id": "a", "action": "health", "inputs": ["t"]}]),
                     "step 'a': inputs must be an object")

    def test_an_input_source_must_name_its_step_and_field_as_strings(self):
        self.refused(self.base(steps=[
            {"id": "a", "action": "health", "output": "json", "provides": ["t"]},
            {"id": "b", "action": "health", "inputs": {"t": {"step": ["a"], "field": "t"}}}]),
            "input 't' step and field must be strings")

    def test_step_ids_are_unique_and_plain(self):
        self.refused(self.base(steps=[{"id": "a", "action": "health"},
                                      {"id": "a", "action": "health"}]), "used twice")
        self.refused(self.base(steps=[{"id": "A b", "action": "health"}]), "[a-z0-9_-]")
        self.refused(self.base(steps=[{"id": "x" * 65, "action": "health"}]), "[a-z0-9_-]")


class ChainTimeout(ChainCase):
    """*"timeout propagation"*: the budget in force reaches the step, and a step that
    outruns it is stopped -- with everything it started."""

    SLOW = True

    def test_the_step_is_told_its_budget(self):
        result = self.chain([{"id": "a", "action": "health", "timeout_ms": 3000},
                             {"id": "b", "action": "health"}], timeout_ms=20000)
        self.assertEqual(result.returncode, 0, result.stderr)
        budgets = [int(call["budget"]) for call in self.calls()]
        self.assertEqual(budgets[0], 3000)
        self.assertTrue(3000 < budgets[1] <= 20000, budgets)

    def test_a_step_that_outruns_its_budget_is_stopped_with_its_descendants(self):
        grandchild = self.work / "grandchild.pid"
        self.behave("slow", f"sleep 60 &\necho $! > '{grandchild}'\nwait\n")
        started = time.monotonic()
        result = self.chain([{"id": "slow", "action": "health", "timeout_ms": 400},
                             {"id": "next", "action": "health"}])
        elapsed = time.monotonic() - started
        self.assertEqual(result.returncode, EXIT_TIMEOUT, result.stderr)
        self.assertLess(elapsed, 8, "the budget did not bound the step")
        summary = self.records(result)[-1]
        self.assertEqual((summary["outcome"], summary["limit"], summary["failed_step"],
                          summary["not_run"]), ("timeout", "step", "slow", ["next"]))
        self.assertFalse(pid_alive(int(grandchild.read_text())),
                         "the step's descendant outlived the timeout")

    def test_the_chain_budget_bounds_the_sum_of_the_steps(self):
        self.behave("one", "sleep 0.6\n")
        self.behave("two", "sleep 5\n")
        result = self.chain([{"id": "one", "action": "health"},
                             {"id": "two", "action": "health"}], timeout_ms=1500)
        self.assertEqual(result.returncode, EXIT_TIMEOUT, result.stderr)
        records = self.records(result)
        self.assertEqual(records[0]["outcome"], "ok")
        self.assertEqual((records[1]["outcome"], records[-1]["limit"]), ("timeout", "chain"))
        self.assertLess(int(self.calls()[1]["budget"]), 1500)

    def test_a_step_that_ignores_term_is_killed_within_the_grace(self):
        # Elapsed time alone cannot tell a KILL from giving up after the grace; only the
        # process being gone can.
        stubborn = self.work / "stubborn.pid"
        self.behave("stubborn", f"trap '' TERM\necho $$ > '{stubborn}'\n"
                                "while :; do sleep 0.05; done\n")
        started = time.monotonic()
        result = self.chain([{"id": "stubborn", "action": "health", "timeout_ms": 300}])
        self.assertEqual(result.returncode, EXIT_TIMEOUT, result.stderr)
        self.assertLess(time.monotonic() - started, 10)
        self.assertNotIn("settled", self.records(result)[0])
        self.assertFalse(pid_alive(int(stubborn.read_text())), "TERM was not escalated to KILL")


class ChainCancellation(ChainCase):
    """*"cancellation"*: a signal to the chain stops the running step and everything it
    started, runs nothing further, and is reported as a cancellation, not a failure."""

    SLOW = True

    def cancel(self, sig, expected):
        started = self.work / "started"
        grandchild = self.work / "grandchild.pid"
        self.behave("long", f"sleep 60 &\necho $! > '{grandchild}'\ntouch '{started}'\nwait\n")
        path = self.spec([{"id": "quick", "action": "health"},
                          {"id": "long", "action": "health"},
                          {"id": "after", "action": "health"}])
        environment = dict(os.environ, HEE3_CATALOGUE=str(CATALOGUE), LC_ALL="C",
                           HEE3_PRODUCER=str(self.model), **self.environment())
        process = subprocess.Popen(["bash", str(WRAPPER), "chain", str(path)], env=environment,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        wait_for(started)
        process.send_signal(sig)
        stdout, stderr = process.communicate(timeout=15)
        self.assertEqual(process.returncode, expected, stderr)
        records = [json.loads(line) for line in stdout.splitlines()]
        self.assertEqual([r.get("outcome") for r in records], ["ok", "cancelled", "cancelled"])
        self.assertEqual(records[-1]["failed_step"], "long")
        self.assertEqual(records[-1]["not_run"], ["after"])
        self.assertEqual([call["step"] for call in self.calls()], ["quick", "long"])
        self.assertFalse(pid_alive(int(grandchild.read_text())),
                         "the step's descendant outlived the cancellation")

    def test_sigterm_cancels_with_143(self):
        self.cancel(signal.SIGTERM, 143)

    def test_sigint_cancels_with_130(self):
        self.cancel(signal.SIGINT, 130)


def blocked_writing(pid):
    """True while `pid` is parked in a write to a full pipe (Linux wchan)."""
    try:
        return "pipe_write" in Path(f"/proc/{pid}/wchan").read_text()
    except FileNotFoundError:
        return False


class ChainBetweenSteps(ChainCase):
    """The two stops that can only happen BETWEEN steps, made deterministic.

    A step's record is written to stdout after the step has ended. When nobody reads that
    stdout and the record is larger than the pipe, the runner parks in the write -- after one
    step, before the next. A signal or a spent budget that lands then must stop the chain
    before the next step starts, and must be reported as exactly that.
    """

    SLOW = True

    def start_parked(self, timeout_ms):
        self.behave("big", "head -c 300000 /dev/zero | tr '\\0' x\n")
        path = self.spec([{"id": "big", "action": "health"},
                          {"id": "next", "action": "health"}], timeout_ms)
        environment = dict(os.environ, HEE3_CATALOGUE=str(CATALOGUE), LC_ALL="C",
                           HEE3_PRODUCER=str(self.model), **self.environment())
        process = subprocess.Popen(["bash", str(WRAPPER), "chain", str(path)], env=environment,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(lambda: process.poll() is None and process.kill())
        deadline = time.monotonic() + 10
        while not blocked_writing(process.pid):
            if time.monotonic() > deadline:
                raise AssertionError("the runner never parked writing the first record in 10s")
            time.sleep(0.02)
        return process

    def summary_of(self, process):
        stdout, stderr = process.communicate(timeout=RUN_BUDGET_S)
        return process.returncode, json.loads(stdout.splitlines()[-1]), stderr

    def test_a_signal_between_steps_stops_before_the_next_starts(self):
        process = self.start_parked(20000)
        process.send_signal(signal.SIGTERM)
        code, summary, stderr = self.summary_of(process)
        self.assertEqual(code, 143, stderr)
        self.assertEqual((summary["outcome"], summary["detail"], summary["failed_step"],
                          summary["not_run"]),
                         ("cancelled", "cancelled between steps", None, ["next"]))
        self.assertEqual([call["step"] for call in self.calls()], ["big"])

    def test_a_budget_spent_between_steps_stops_before_the_next_starts(self):
        # 3 s leaves the first step room on a loaded host; the wait then spends the rest.
        process = self.start_parked(3000)
        time.sleep(3.5)
        code, summary, stderr = self.summary_of(process)
        self.assertEqual(code, EXIT_TIMEOUT, stderr)
        self.assertEqual((summary["outcome"], summary["failed_step"], summary["limit"],
                          summary["detail"]),
                         ("timeout", "next", "chain", "the chain budget was spent before this step"))
        self.assertEqual([call["step"] for call in self.calls()], ["big"])


class ChainCheckPhase(ChainCase):
    """The refusals of the check phase, before any step runs. The check reads the action
    catalogue; pointing it at a FIFO with no writer parks the check where the test can see it:
    a non-blocking open for writing succeeds only once a reader is waiting."""

    SLOW = True

    def parked_check(self):
        fifo = self.work / "catalogue.fifo"
        os.mkfifo(fifo)
        path = self.spec([{"id": "a", "action": "health"}])
        environment = dict(os.environ, HEE3_CATALOGUE=str(fifo), LC_ALL="C",
                           HEE3_PRODUCER=str(self.model), **self.environment())
        process = subprocess.Popen(["bash", str(WRAPPER), "chain", str(path)], env=environment,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(lambda: process.poll() is None and process.kill())
        deadline = time.monotonic() + 10
        while True:
            try:
                writer = os.open(fifo, os.O_WRONLY | os.O_NONBLOCK)
                break
            except OSError:
                if time.monotonic() > deadline:
                    raise AssertionError("the check never opened the catalogue in 10s") from None
                time.sleep(0.02)
        # Held open, so the reader sees neither data nor end-of-file until the case is done.
        self.addCleanup(os.close, writer)
        return process

    def test_a_signal_during_the_checks_runs_nothing(self):
        process = self.parked_check()
        process.send_signal(signal.SIGTERM)
        _, stderr = process.communicate(timeout=RUN_BUDGET_S)
        self.assertEqual(process.returncode, 143, stderr)
        self.assertIn("cancelled before any step ran", stderr)
        self.assertEqual(self.calls(), [])

    def test_a_check_that_does_not_finish_is_refused_within_its_budget(self):
        process = self.parked_check()
        _, stderr = process.communicate(timeout=RUN_BUDGET_S)
        self.assertEqual(process.returncode, EXIT_USAGE, stderr)
        self.assertIn("checking step 'a' did not finish in 3000 ms", stderr)
        self.assertEqual(self.calls(), [])


def shlex_quote(text):
    return "'" + text.replace("'", "'\"'\"'") + "'"


def load_tests(loader, tests, pattern):
    """Cheap cases first, the cases that wait on real budgets last.

    tools/check-bash-sites runs each site mutant with --failfast, so a kill costs only the
    cases before the first failure; with the waiting classes first, every kill paid for them.
    Order changes nothing about what runs -- every case still runs on a green suite.
    """
    cases = [case for group in tests for case in group]
    fast = [case for case in cases if not getattr(case, "SLOW", False)]
    slow = [case for case in cases if getattr(case, "SLOW", False)]
    return unittest.TestSuite(fast + slow)


if __name__ == "__main__":
    become_subreaper()
    result = unittest.main(exit=False, verbosity=1).result
    print(f"bash wrapper tests: run={result.testsRun} failures={len(result.failures)} "
          f"errors={len(result.errors)}")
    sys.exit(0 if result.wasSuccessful() else 1)
