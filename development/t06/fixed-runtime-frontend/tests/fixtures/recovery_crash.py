"""Test-only abrupt-loss fixture, joined by the existing process owner."""
from pathlib import Path
import hashlib
import importlib.util
import json
import os
import sys
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
PHASES = {"pre-ack": "pre-ack-held-stdin", "post-ack": "post-correlated-ack-live-writer",
          "worker-return": "worker-return", "evidence-published": "evidence-published",
          "verification-return": "verification-return", "acceptance-return": "acceptance-return"}


def validate(result, mode, path):
    if mode not in {*PHASES, "benign"}:
        raise ValueError("unknown finite fixture mode")
    if (result.get("started") is not True or type(result.get("exit_code")) is not int
        or result["exit_code"] != (0 if mode == "benign" else -9)
        or result.get("cleanup_complete") is not True
        or result.get("descendants_detected") is not (mode in ("pre-ack", "post-ack"))
        or result.get("timed_out") is not False or result.get("output_limit_exceeded") is not False
        or result.get("supervisor_error") is not None):
        raise ValueError("Actual loss/custody result refused")
    if mode == "benign":
        if json.loads((path / "benign.json").read_text()) != {"accepted": True, "fixture_only": True}:
            raise ValueError("benign fixture did not reach returned acceptance")
    else:
        cut = json.loads((path / "cut.json").read_text())
        if cut != {"phase": PHASES[mode], "signal": 9, "kind": "test-only returned-boundary cut"}:
            raise ValueError("wrong actual crash boundary")
    if mode != "pre-ack":
        ack = json.loads((path / "ack.json").read_text())
        if (ack.get("contract_accepted") is not True or ack.get("generation") != "1"
            or ack.get("raw") != "ACK " + ack["task"] + " " + ack["attempt"] + " 1 " + ack["invocation"] + "\n"):
            raise ValueError("correlated acknowledgement absent")
        live = json.loads((path / "live-writer.json").read_text())
        if (live.get("leader_terminal") is not False or live["workspace_bytes"] < 2
            or live["now_ms"] <= live["expires_ms"] or not live["begin_attempt_error"]
            or live["reserved_work_ms"] != 900000 or live["attempts"] != 1):
            raise ValueError("actual live writer/refused lease reuse absent")
    elif (path / "ack.json").exists() or (path / "workspace/output").exists():
        raise ValueError("pre-ack startup gate already released")


def pin(path):
    data = path.read_bytes()
    return {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


def main():
    binary, expected, mode, raw_path, raw_output = sys.argv[1:]
    binary, path, output = Path(binary), Path(raw_path), Path(raw_output)
    output.mkdir()
    argv = [str(binary), "driver", str(path), mode]
    inputs = [binary, Path(__file__), ROOT / "tools/development_process.py"]
    record = {"profile": "test-only local adapter crash; no native/Pi admission", "argv": argv,
              "inputs": {str(p): pin(p) for p in inputs}, "accepted_control": False}
    try:
        if not binary.is_absolute() or binary.is_symlink() or expected != "sha256:" + pin(binary)["sha256"]:
            raise ValueError("fixture binary pin")
        spec = importlib.util.spec_from_file_location("bounded", ROOT / "tools/development_process.py")
        owner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(owner)
        result = owner.run_bounded(argv, path, {"LC_ALL": "C", "TMPDIR": os.environ["TMPDIR"]},
                                   output / "stdout", output / "stderr", time.monotonic() + 25,
                                   command_timeout=15, stream_limit=1048576)
        record.update(result)
        validate(result, mode, path)
        if (output / "stderr").read_bytes():
            raise ValueError("unexpected fixture diagnostics")
        record["accepted_control"] = True
    except BaseException as error:
        observation = getattr(error, "result", getattr(error, "bounded_process_result", None))
        if observation is not None:
            record.update(observation)
        record["error"] = f"{type(error).__name__}: {error}"
    finally:
        record["input_drift"] = []
        record["postrun_errors"] = []
        for path in inputs:
            try:
                if pin(path) != record["inputs"][str(path)]: record["input_drift"].append(str(path))
            except OSError as error: record["postrun_errors"].append(str(error))
        if record["input_drift"] or record["postrun_errors"]: record["accepted_control"] = False
        try: (output / "result.json").write_text(json.dumps(record, indent=2) + "\n")
        except OSError as error:
            record["accepted_control"] = False
            record["publication_error"] = str(error)
            print(json.dumps(record), file=sys.stderr)
    print(json.dumps({"accepted_control": record["accepted_control"]}))
    return 0 if record["accepted_control"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
