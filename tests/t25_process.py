#!/usr/bin/env python3
"""Owned TH-DEV process-supervisor qualification fixtures."""

from __future__ import annotations

import argparse
import io
import json
from pathlib import Path
import sys
import tempfile
import threading
import time
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "tools"))
import development_process as supervisor  # noqa: E402

run_bounded = supervisor.run_bounded


class ProcessControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="hee3-process-test-")
        self.root = Path(self.temporary.name)
        self.index = 0

    def tearDown(self):
        self.assertEqual(supervisor._children(), set())
        self.temporary.cleanup()

    def run_code(self, code, *, timeout=2, limit=1024, expired=False):
        self.index += 1
        deadline = time.monotonic() - 1 if expired else time.monotonic() + 20
        return run_bounded(
            [sys.executable, "-c", code], self.root, {},
            self.root / f"{self.index}.stdout", self.root / f"{self.index}.stderr",
            deadline, command_timeout=timeout, stream_limit=limit,
        )

    def test_normal_child(self):
        result = self.run_code("print('ok')")
        self.assertEqual(result["exit_code"], 0)
        self.assertFalse(result["timed_out"])
        self.assertFalse(result["output_limit_exceeded"])
        self.assertTrue(result["cleanup_complete"])

    def test_closed_streams_do_not_end_supervision(self):
        marker = self.root / "closed-stream-finished"
        code = "import os,time; os.close(1); os.close(2); time.sleep(.05); open('closed-stream-finished','w').write('done')"
        result = self.run_code(code)
        self.assertEqual(result["exit_code"], 0)
        self.assertEqual(marker.read_text(), "done")
        self.assertTrue(result["cleanup_complete"])

    def test_output_just_below_limit(self):
        result = self.run_code("import os; os.write(1, b'x' * 1023)")
        self.assertEqual(result["stream_bytes"]["stdout_retained"], 1023)
        self.assertFalse(result["output_limit_exceeded"])

    def test_output_above_limit(self):
        result = self.run_code("import os; os.write(1, b'x' * 1025)")
        self.assertEqual(result["stream_bytes"]["stdout_retained"], 1024)
        self.assertTrue(result["output_limit_exceeded"])
        self.assertTrue(result["cleanup_complete"])

    def test_background_same_group_child(self):
        code = "import subprocess,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); sys.exit(0)"
        result = self.run_code(code)
        self.assertTrue(result["descendants_detected"])
        self.assertTrue(result["cleanup_complete"])

    def test_detached_double_fork_child(self):
        code = """import os,time
pid=os.fork()
if pid==0:
 os.setsid()
 pid=os.fork()
 if pid==0:
  time.sleep(30)
 os._exit(0)
os._exit(0)
"""
        result = self.run_code(code)
        self.assertTrue(result["descendants_detected"])
        self.assertTrue(result["cleanup_complete"])

    def test_term_ignoring_timeout(self):
        code = "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(30)"
        result = self.run_code(code, timeout=0.05)
        self.assertTrue(result["timed_out"])
        self.assertTrue(result["cleanup_complete"])
        self.assertEqual(result["exit_code"], -signal_number("SIGKILL"))

    def test_expired_aggregate_deadline_starts_nothing(self):
        result = self.run_code("raise SystemExit(99)", expired=True)
        self.assertFalse(result["started"])
        self.assertTrue(result["timed_out"])
        self.assertTrue(result["cleanup_complete"])
        self.assertIsNone(result["exit_code"])

    def test_read_exception_still_settles_owned_child(self):
        code = "import os,subprocess,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); os.write(1,b'x')"
        with mock.patch.object(supervisor, "_read_pipe", side_effect=OSError("injected read")):
            with self.assertRaisesRegex(supervisor.ProcessSupervisionError, "injected read") as caught:
                self.run_code(code)
        self.assertIsInstance(caught.exception.original, OSError)
        observation = caught.exception.result
        self.assertTrue(observation["cleanup_complete"])
        self.assertTrue(observation["descendants_detected"])

    def test_log_write_exception_still_settles_owned_child(self):
        code = "import os,subprocess,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); os.write(1,b'x')"
        with mock.patch.object(supervisor, "_write_retained", side_effect=OSError("injected log write")):
            with self.assertRaisesRegex(supervisor.ProcessSupervisionError, "injected log write") as caught:
                self.run_code(code)
        self.assertIsInstance(caught.exception.original, OSError)
        observation = caught.exception.result
        self.assertTrue(observation["cleanup_complete"])
        self.assertTrue(observation["descendants_detected"])

    def test_keyboard_interrupt_still_settles_owned_child(self):
        code = "import os,subprocess,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); os.write(1,b'x')"
        with mock.patch.object(supervisor, "_read_pipe", side_effect=KeyboardInterrupt("injected interrupt")):
            with self.assertRaisesRegex(KeyboardInterrupt, "injected interrupt") as caught:
                self.run_code(code)
        observation = caught.exception.bounded_process_result
        self.assertTrue(observation["cleanup_complete"])
        self.assertTrue(observation["descendants_detected"])

    def test_selector_constructor_exception_still_settles_leader(self):
        with mock.patch.object(
            supervisor.selectors,
            "DefaultSelector",
            side_effect=OSError("injected selector construction"),
        ):
            with self.assertRaisesRegex(
                supervisor.ProcessSupervisionError, "injected selector construction"
            ) as caught:
                self.run_code("import time; time.sleep(30)")
        self.assertTrue(caught.exception.result["cleanup_complete"])
        self.assertIsNotNone(caught.exception.result["exit_code"])

    def test_selector_register_exception_still_settles_leader(self):
        selector = supervisor.selectors.DefaultSelector()
        selector.register = mock.Mock(side_effect=OSError("injected selector register"))
        with mock.patch.object(supervisor.selectors, "DefaultSelector", return_value=selector):
            with self.assertRaisesRegex(
                supervisor.ProcessSupervisionError, "injected selector register"
            ) as caught:
                self.run_code("import time; time.sleep(30)")
        self.assertTrue(caught.exception.result["cleanup_complete"])
        self.assertIsNotNone(caught.exception.result["exit_code"])

    def test_incomplete_cleanup_refuses_post_settlement_drain(self):
        real_settle = supervisor._settle

        def settle_but_report_incomplete(*args):
            _, descendants = real_settle(*args)
            return False, descendants

        with mock.patch.object(supervisor, "_settle", side_effect=settle_but_report_incomplete), mock.patch.object(
            supervisor, "_drain_settled", side_effect=AssertionError("drain must be refused")
        ) as drain:
            result = self.run_code("print('terminal output')")
        self.assertFalse(result["cleanup_complete"])
        drain.assert_not_called()

    def test_nonterminal_cleanup_returns_without_blocking_wait(self):
        with mock.patch.object(supervisor, "_TERM_GRACE_SECONDS", 0), mock.patch.object(
            supervisor, "_KILL_GRACE_SECONDS", 0
        ), mock.patch.object(supervisor, "_leader_terminal", return_value=False), mock.patch.object(
            supervisor, "_owned_children", return_value=set()
        ), mock.patch.object(supervisor.os, "killpg", side_effect=ProcessLookupError):
            started = time.monotonic()
            complete, descendants = supervisor._settle(999_991, 999_991, set())
        self.assertFalse(complete)
        self.assertFalse(descendants)
        self.assertLess(time.monotonic() - started, 0.5)

    def test_terminal_leader_reap_updates_popen_returncode(self):
        process = supervisor.subprocess.Popen([sys.executable, "-c", "raise SystemExit(7)"])
        supervisor.os.waitid(supervisor.os.P_PID, process.pid, supervisor.os.WEXITED | supervisor.os.WNOWAIT)
        self.assertEqual(supervisor._reap_terminal_leader(process), 7)
        self.assertEqual(process.returncode, 7)

    def test_non_main_thread_is_refused_before_spawn(self):
        errors = []

        def invoke():
            try:
                self.run_code("raise SystemExit(0)")
            except BaseException as error:
                errors.append(error)

        thread = threading.Thread(target=invoke)
        thread.start()
        thread.join()
        self.assertEqual(len(errors), 1)
        self.assertRegex(str(errors[0]), "main thread")


def signal_number(name):
    import signal
    return int(getattr(signal, name))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-dir", type=Path)
    args, unittest_args = parser.parse_known_args()
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(ProcessControls)
    started = time.monotonic()
    raw_log = io.StringIO()
    result = unittest.TextTestRunner(stream=raw_log, verbosity=2).run(suite)
    raw_text = raw_log.getvalue()
    print(raw_text, end="")
    report = {
        "schema": 1,
        "scope": "TH-DEV standalone process controls; not hostile isolation or admission",
        "tests_run": result.testsRun,
        "failures": len(result.failures),
        "errors": len(result.errors),
        "skipped": len(result.skipped),
        "successful": result.wasSuccessful(),
        "elapsed_seconds": time.monotonic() - started,
    }
    if args.evidence_dir:
        args.evidence_dir.mkdir(parents=True, exist_ok=True)
        (args.evidence_dir / "run.log").write_text(raw_text)
        (args.evidence_dir / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    raise SystemExit(0 if result.wasSuccessful() else 1)
