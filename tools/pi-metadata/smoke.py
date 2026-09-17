#!/usr/bin/env python3
"""Bounded credential-free T02 Pi SDK metadata smoke under development isolation."""

import datetime
import hashlib
import json
import os
import re
from pathlib import Path
import stat
import sys
import time


BASE = Path(__file__).resolve().parents[2]
DRAFT = BASE / "tools/pi-metadata"
EVIDENCE = BASE / "evidence/implementation/T02/sdk-runs" / datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
PROJECT = BASE
sys.path.insert(0, str(PROJECT / "tools"))
from development_process import run_bounded

NODE_ROOT = Path("/var/home/Louranicas/.cache/hee3-tools/node-v24.21.0-linux-x64")
NODE = NODE_ROOT / "bin/node"
PI = Path("/var/home/Louranicas/.local/lib/node_modules/@earendil-works/pi-coding-agent")
BWRAP = Path("/usr/bin/bwrap")
LIMIT = 8 * 1024 * 1024
RUNTIME_FILES = tuple(Path(path) for path in (
    "/usr/bin/bash", "/usr/bin/cat", "/usr/bin/sleep", "/usr/bin/true",
    "/usr/lib64/ld-linux-x86-64.so.2", "/usr/lib64/libc.so.6", "/usr/lib64/libdl.so.2",
    "/usr/lib64/libgcc_s.so.1", "/usr/lib64/libm.so.6", "/usr/lib64/libpthread.so.0",
    "/usr/lib64/libstdc++.so.6", "/usr/lib64/libtinfo.so.6",
))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_manifest(root):
    result = {}
    for path in sorted(root.rglob("*")):
        relative = str(path.relative_to(root))
        mode = path.lstat().st_mode
        if stat.S_ISLNK(mode):
            result[relative] = {"kind": "symlink", "target": os.readlink(path)}
        elif stat.S_ISREG(mode):
            result[relative] = {"kind": "file", "bytes": path.stat().st_size, "sha256": digest(path)}
    return result


origin = time.monotonic()
deadline = origin + 240
EVIDENCE.mkdir(parents=True, exist_ok=True)
report = {
    "schema": 1,
    "scope": "T02 credential-free Pi SDK metadata smoke under TH-DEV; no provider request, module admission, or RC05 hostile qualification",
    "commands": [],
    "provider_requests": 0,
    "external_provider_currency": 0,
    "inputs": {
        "harness": {"path": str(Path(__file__).resolve()), "sha256": digest(Path(__file__))},
        "node_tree": tree_manifest(NODE_ROOT),
        "pi_tree": tree_manifest(PI),
        "draft_tree": tree_manifest(DRAFT),
        "bwrap": {"path": str(BWRAP), "sha256": digest(BWRAP)},
        "passwd": {"path": "/etc/passwd", "sha256": digest(Path("/etc/passwd"))},
        "runtime_files": {str(path): digest(path) for path in RUNTIME_FILES},
        "development_process": {"path": str(PROJECT / "tools/development_process.py"), "sha256": digest(PROJECT / "tools/development_process.py")},
    },
}

runtime_mounts = []
for path in RUNTIME_FILES:
    runtime_mounts.extend(("--ro-bind", str(path), str(path)))
base_argv = [
    str(BWRAP), "--unshare-all", "--die-with-parent", "--new-session",
    "--dir", "/usr", "--dir", "/usr/bin", "--dir", "/usr/lib64",
    *runtime_mounts, "--symlink", "bash", "/usr/bin/sh",
    "--symlink", "usr/bin", "/bin", "--symlink", "usr/lib64", "/lib64",
    "--ro-bind", "/etc/passwd", "/etc/passwd",
    "--ro-bind", str(NODE_ROOT), "/node", "--ro-bind", str(PI), "/pi",
    "--ro-bind", str(DRAFT), "/draft", "--proc", "/proc", "--dev", "/dev",
    "--tmpfs", "/tmp", "--tmpfs", "/work", "--tmpfs", "/agent", "--chdir", "/work", "--",
]
environment = {"PATH": "/usr/bin:/bin", "LC_ALL": "C", "PI_OFFLINE": "1"}


def run(label, tail, expected_stdout=None, expected_warning=False):
    stdout = EVIDENCE / f"{label}.stdout"
    stderr = EVIDENCE / f"{label}.stderr"
    argv = [*base_argv, *tail]
    row = {"id": label, "argv": argv, "environment": environment}
    report["commands"].append(row)
    try:
        result = run_bounded(argv, BASE, environment, stdout, stderr, deadline, command_timeout=60, stream_limit=LIMIT)
        row.update(result)
    except BaseException as error:
        row.update(getattr(error, "result", getattr(error, "bounded_process_result", {})))
        row["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        for name, path in (("stdout", stdout), ("stderr", stderr)):
            if path.is_file():
                row[name] = {"path": path.name, "bytes": path.stat().st_size, "sha256": digest(path)}
    streams = {"stdout": stdout.read_text(), "stderr": stderr.read_text()}
    for name, path in (("stdout", stdout), ("stderr", stderr)):
        row[name] = {"path": path.name, "bytes": path.stat().st_size, "sha256": digest(path)}
    if result["exit_code"] != 0 or result["timed_out"] or result["output_limit_exceeded"] or not result["cleanup_complete"]:
        raise RuntimeError("bounded command failed: " + label)
    if expected_stdout and expected_stdout not in streams["stdout"]:
        raise RuntimeError("missing stdout control: " + label)
    warning_pattern = r"\(node:[0-9]+\) Warning: HEE3_T02_VISIBLE_WARNING_CONTROL\n\(Use `node --trace-warnings \.\.\.` to show where the warning was created\)\n"
    if expected_warning:
        if not re.fullmatch(warning_pattern, streams["stderr"]):
            raise RuntimeError("warning control differed or had extra diagnostics: " + label)
    elif streams["stderr"]:
        raise RuntimeError("unexpected baseline stderr: " + label)
    return streams


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key")
        result[key] = value
    return result


def main():
    run("sandbox-true", ["/usr/bin/true"])
    run("network-denied", ["/node/bin/node", "/draft/network-probe.mjs"], "HEE3_NETWORK_DENIED_CONTROL")
    shell = "(/usr/bin/cat /draft/smoke.jsonl; /usr/bin/sleep 1) | /node/bin/node /draft/entry.mjs"
    benign = run("metadata-benign", ["/usr/bin/sh", "-c", shell])
    warning = run("metadata-warning", ["/usr/bin/sh", "-c", shell + " --warning-fault"], expected_warning=True)
    
    for label, streams in (("metadata-benign", benign), ("metadata-warning", warning)):
        records = [json.loads(line, object_pairs_hook=unique_object) for line in streams["stdout"].splitlines()]
        responses = {(item.get("id"), item.get("command")): item for item in records if item.get("type") == "response"}
        if len(records) != 2 or len(responses) != 2:
            raise RuntimeError("extra, missing or duplicate metadata record: " + label)
        state = responses[("hee3-t02-state", "get_state")]
        models = responses[("hee3-t02-models", "get_available_models")]
        if not state["success"] or not models["success"] or models["data"]["models"] != []:
            raise RuntimeError("metadata response differed: " + label)
        data = state["data"]
        model = data.get("model", {})
        sentinel = {"id":"unknown","name":"unknown","api":"unknown","provider":"unknown","baseUrl":"","reasoning":False,"input":[],"cost":{"input":0,"output":0,"cacheRead":0,"cacheWrite":0},"contextWindow":0,"maxTokens":0}
        if model != sentinel or data["thinkingLevel"] != "off" or data["isStreaming"] or data["isCompacting"] or data["pendingMessageCount"] != 0 or data["messageCount"] != 0 or data["autoCompactionEnabled"]:
            raise RuntimeError("isolated state differed: " + label)
    
    report["status"] = "pass credential-free metadata smoke"
    report["network_namespace_control"] = "bwrap --unshare-all plus denied IPv4 fetch; development evidence only"

try:
    main()
except BaseException as error:
    report["status"] = "failed metadata smoke"
    report["error"] = f"{type(error).__name__}: {error}"
    raise
finally:
    report["elapsed_seconds"] = time.monotonic() - origin
    (EVIDENCE / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    print(EVIDENCE / "results.json", flush=True)
