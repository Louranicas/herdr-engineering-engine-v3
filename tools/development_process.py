#!/usr/bin/env python3
"""Bounded Linux process supervision for standalone TH-DEV quality checks.

This is same-UID development custody, not hostile isolation or T15 evidence.
Callers must serialize use in a process that does not create unrelated children.
"""

from __future__ import annotations

import ctypes
import os
from pathlib import Path
import selectors
import signal
import subprocess
import threading
import time


_PR_SET_CHILD_SUBREAPER = 36
_LOCK = threading.Lock()
_SUBREAPER_SET = False
_TERM_GRACE_SECONDS = 5.0
_KILL_GRACE_SECONDS = 5.0
_DRAIN_GRACE_SECONDS = 0.25


class ProcessSupervisionError(RuntimeError):
    """An operational supervisor error after unconditional cleanup."""

    def __init__(self, original: Exception, result: dict, cleanup_error: BaseException | None):
        super().__init__(f"{type(original).__name__}: {original}")
        self.original = original
        self.result = result
        self.cleanup_error = cleanup_error


def _enable_subreaper() -> None:
    global _SUBREAPER_SET
    if _SUBREAPER_SET:
        return
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(_PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) != 0:
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error))
    _SUBREAPER_SET = True


def _children() -> set[int]:
    path = Path(f"/proc/self/task/{os.getpid()}/children")
    value = path.read_text().strip()
    return {int(item) for item in value.split()} if value else set()


def _pidfd(pid: int) -> int | None:
    try:
        return os.pidfd_open(pid, 0)
    except ProcessLookupError:
        return None


def _pidfd_signal(fd: int, sig: signal.Signals) -> None:
    try:
        signal.pidfd_send_signal(fd, sig)
    except ProcessLookupError:
        pass


def _reap_adopted(leader: int) -> None:
    for pid in _children() - {leader}:
        try:
            os.waitpid(pid, os.WNOHANG)
        except ChildProcessError:
            pass


def _owned_children(leader: int, baseline: set[int]) -> set[int]:
    return _children() - baseline - {leader}


def _leader_terminal(leader: int) -> bool:
    try:
        return os.waitid(
            os.P_PID, leader, os.WEXITED | os.WNOHANG | os.WNOWAIT
        ) is not None
    except ChildProcessError:
        return True


def _settle(leader: int, process_group: int, baseline: set[int]) -> tuple[bool, bool]:
    """Terminate the retained group and newly adopted direct children."""
    descendants_detected = False
    phases = ((signal.SIGTERM, _TERM_GRACE_SECONDS), (signal.SIGKILL, _KILL_GRACE_SECONDS))
    for sig, grace in phases:
        owned = _owned_children(leader, baseline)
        descendants_detected |= bool(owned)
        descriptors = []
        for pid in owned:
            fd = _pidfd(pid)
            if fd is not None:
                descriptors.append(fd)
                _pidfd_signal(fd, sig)
        for fd in descriptors:
            os.close(fd)
        try:
            os.killpg(process_group, sig)
        except ProcessLookupError:
            pass
        end = time.monotonic() + grace
        while True:
            owned = _owned_children(leader, baseline)
            descendants_detected |= bool(owned)
            descriptors = []
            for pid in owned:
                fd = _pidfd(pid)
                if fd is not None:
                    descriptors.append(fd)
                    _pidfd_signal(fd, sig)
            for fd in descriptors:
                os.close(fd)
            _reap_adopted(leader)
            owned = _owned_children(leader, baseline)
            if (not owned and _leader_terminal(leader)) or time.monotonic() >= end:
                break
            time.sleep(0.01)
        if not _owned_children(leader, baseline) and _leader_terminal(leader):
            break
    owned = _owned_children(leader, baseline)
    descendants_detected |= bool(owned)
    _reap_adopted(leader)
    return not _owned_children(leader, baseline) and _leader_terminal(leader), descendants_detected


def _status_code(status: int) -> int:
    if os.WIFEXITED(status):
        return os.WEXITSTATUS(status)
    if os.WIFSIGNALED(status):
        return -os.WTERMSIG(status)
    raise RuntimeError("leader did not reach a terminal wait status")


def _write_retained(destination, chunk: bytes, allowed: int) -> int:
    retained = min(len(chunk), allowed)
    if retained:
        destination.write(chunk[:retained])
    return retained


def _read_pipe(fd: int) -> bytes:
    return os.read(fd, 65536)


def _reap_terminal_leader(process: subprocess.Popen) -> int | None:
    if not _leader_terminal(process.pid):
        return None
    try:
        pid, status = os.waitpid(process.pid, os.WNOHANG)
    except ChildProcessError:
        return process.returncode
    if pid == 0:
        return None
    code = _status_code(status)
    process.returncode = code
    return code


def _drain_settled(selector, streams, result, stream_limit, deadline_monotonic) -> None:
    """Drain a terminal process's immediately available output within fixed bounds."""
    drain_deadline = min(deadline_monotonic, time.monotonic() + _DRAIN_GRACE_SECONDS)
    while selector.get_map() and time.monotonic() < drain_deadline:
        wait = min(0.05, max(0.0, drain_deadline - time.monotonic()))
        events = selector.select(wait)
        if not events:
            break
        refuse_more = False
        for key, _ in events:
            pipe = key.fileobj
            name, destination = streams[pipe]
            chunk = _read_pipe(pipe.fileno())
            if not chunk:
                selector.unregister(pipe)
                continue
            result["stream_bytes"][name + "_observed"] += len(chunk)
            retained = result["stream_bytes"][name + "_retained"]
            allowed = max(0, stream_limit - retained)
            if allowed:
                result["stream_bytes"][name + "_retained"] += _write_retained(
                    destination, chunk, allowed
                )
            if len(chunk) > allowed:
                result["output_limit_exceeded"] = True
                refuse_more = True
                break
        if refuse_more:
            break


def run_bounded(
    argv,
    cwd,
    env,
    stdout_path,
    stderr_path,
    deadline_monotonic,
    command_timeout=180,
    stream_limit=8 * 1024 * 1024,
):
    """Run one command and return a fail-closed structured observation.

    The deadline uses the caller's single monotonic origin. The caller owns the
    RC01 cleanup-reserve admission decision. Each stream retains exact bytes up
    to ``stream_limit``; crossing it terminates the process and is always a
    failure regardless of the eventual exit code.
    """
    if not argv or command_timeout <= 0 or stream_limit < 0:
        raise ValueError("invalid bounded-process arguments")
    if threading.current_thread() is not threading.main_thread():
        raise RuntimeError("TH-DEV supervisor must run on the main thread")
    started = time.monotonic()
    result = {
        "exit_code": None,
        "timed_out": False,
        "output_limit_exceeded": False,
        "cleanup_complete": False,
        "elapsed_seconds": 0.0,
        "stream_bytes": {
            "stdout_observed": 0,
            "stdout_retained": 0,
            "stderr_observed": 0,
            "stderr_retained": 0,
        },
        "started": False,
        "leader_pid": None,
        "descendants_detected": False,
    }
    stdout_path = Path(stdout_path)
    stderr_path = Path(stderr_path)
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    if started >= deadline_monotonic:
        with stdout_path.open("xb"), stderr_path.open("xb"):
            pass
        result["timed_out"] = True
        result["cleanup_complete"] = True
        result["elapsed_seconds"] = time.monotonic() - started
        return result

    with _LOCK:
        _enable_subreaper()
        baseline = _children()
        if baseline:
            raise RuntimeError("TH-DEV supervisor requires a standalone process with no existing children")
        with stdout_path.open("xb") as stdout_file, stderr_path.open("xb") as stderr_file:
            selector = None
            streams = {}
            pending_error = None
            cleanup_error = None
            settled = False
            leader = None
            process_group = None
            process = subprocess.Popen(
                list(argv), cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                start_new_session=True, close_fds=True,
            )
            try:
                result["started"] = True
                leader = process.pid
                result["leader_pid"] = leader
                process_group = leader
                streams = {
                    process.stdout: ("stdout", stdout_file),
                    process.stderr: ("stderr", stderr_file),
                }
                command_deadline = min(deadline_monotonic, started + command_timeout)
                leader_terminal = False
                termination_required = False
                selector = selectors.DefaultSelector()
                for pipe in streams:
                    os.set_blocking(pipe.fileno(), False)
                    selector.register(pipe, selectors.EVENT_READ)
                while not leader_terminal:
                    now = time.monotonic()
                    if now >= command_deadline:
                        result["timed_out"] = True
                        termination_required = True
                        break
                    wait = min(0.05, command_deadline - now)
                    events = selector.select(wait) if selector.get_map() else ()
                    if not events:
                        time.sleep(min(0.01, wait))
                    for key, _ in events:
                        pipe = key.fileobj
                        name, destination = streams[pipe]
                        chunk = _read_pipe(pipe.fileno())
                        if not chunk:
                            selector.unregister(pipe)
                            continue
                        result["stream_bytes"][name + "_observed"] += len(chunk)
                        retained = result["stream_bytes"][name + "_retained"]
                        allowed = max(0, stream_limit - retained)
                        if allowed:
                            result["stream_bytes"][name + "_retained"] += _write_retained(destination, chunk, allowed)
                        if len(chunk) > allowed:
                            result["output_limit_exceeded"] = True
                            termination_required = True
                            break
                    if termination_required:
                        break
                    observed = os.waitid(os.P_PID, leader, os.WEXITED | os.WNOHANG | os.WNOWAIT)
                    leader_terminal = observed is not None

                cleanup_complete, descendants = _settle(leader, process_group, baseline)
                settled = True
                result["cleanup_complete"] = cleanup_complete
                result["descendants_detected"] = descendants

                if cleanup_complete:
                    _drain_settled(
                        selector, streams, result, stream_limit, deadline_monotonic
                    )
                result["exit_code"] = _reap_terminal_leader(process)
            except BaseException as error:
                pending_error = error
            finally:
                if not settled and leader is not None:
                    try:
                        cleanup_complete, descendants = _settle(leader, process_group, baseline)
                        result["cleanup_complete"] = cleanup_complete
                        result["descendants_detected"] |= descendants
                    except BaseException as caught_cleanup_error:
                        cleanup_error = caught_cleanup_error
                        result["cleanup_complete"] = False
                        if pending_error is None:
                            pending_error = caught_cleanup_error
                if result["exit_code"] is None:
                    try:
                        result["exit_code"] = _reap_terminal_leader(process)
                    except BaseException as reap_error:
                        result["cleanup_complete"] = False
                        if cleanup_error is None:
                            cleanup_error = reap_error
                        if pending_error is None:
                            pending_error = reap_error
                closes = [pipe.close for pipe in streams]
                if selector is not None:
                    closes.insert(0, selector.close)
                for close in closes:
                    try:
                        close()
                    except BaseException as close_error:
                        if cleanup_error is None:
                            cleanup_error = close_error
                        if pending_error is None:
                            pending_error = close_error
                result["elapsed_seconds"] = time.monotonic() - started
                if pending_error is not None:
                    result["supervisor_error"] = f"{type(pending_error).__name__}: {pending_error}"
                    try:
                        pending_error.bounded_process_result = result
                    except (AttributeError, TypeError):
                        pass
            if pending_error is not None:
                if isinstance(pending_error, Exception):
                    raise ProcessSupervisionError(pending_error, result, cleanup_error) from pending_error
                raise pending_error
    result["elapsed_seconds"] = time.monotonic() - started
    return result
