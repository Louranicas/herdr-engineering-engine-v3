"""Finite test-only custody scope using the existing development process owner."""
from pathlib import Path
import hashlib
import importlib.util
import json
import os
import sys
import time

sys.dont_write_bytecode = True
root = Path(__file__).resolve().parents[3]
binary, expected_binary, output = sys.argv[1:]
binary, output = Path(binary), Path(output)
output.mkdir(parents=True, exist_ok=False)

def pin(path):
    data = path.read_bytes()
    return {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}

case = "descendant_in_owned_group_prevents_false_clean_success"
argv = [str(binary), "--exact", case, "--nocapture"]
environment = {
    "LC_ALL": "C", "RUST_TEST_THREADS": "1", "T21_DESCENDANT_INNER": "1",
    "TMPDIR": os.environ["TMPDIR"],
    "T21_PROCESS_EVIDENCE": os.environ["T21_PROCESS_EVIDENCE"],
    "T21_JULIA_REPORT": os.environ["T21_JULIA_REPORT"],
}
paths = [binary, Path(__file__), root / "tools/development_process.py", root / "tests/t21_process.rs"]
record = {"scope": "T21 residual-descendant control joined by existing owner", "argv": argv,
          "environment": environment, "inputs": {str(p): pin(p) for p in paths}, "accepted_control": False}
try:
    if not binary.is_absolute() or binary.is_symlink() or expected_binary != "sha256:" + pin(binary)["sha256"]:
        raise ValueError("Exact nested control binary pin refused")
    spec = importlib.util.spec_from_file_location("bounded", root / "tools/development_process.py")
    bounded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(bounded)
    result = bounded.run_bounded(argv, root, environment, output / "stdout", output / "stderr",
                                 time.monotonic() + 85, command_timeout=65, stream_limit=2 * 1024 * 1024)
    record.update(result)
    raw = (output / "stdout").read_text()
    if (result.get("started") is not True or type(result.get("exit_code")) is not int or result["exit_code"] != 0
        or result.get("cleanup_complete") is not True or result.get("descendants_detected") is not True
        or result.get("timed_out") is not False or result.get("output_limit_exceeded") is not False
        or result.get("supervision_error") is not None or (output / "stderr").read_bytes()
        or raw.count("test " + case + " ... ok") != 1
        or "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out;" not in raw):
        raise ValueError("Actual nested fault control or joined cleanup refused")
    record["accepted_control"] = True
except BaseException as error:
    observation = getattr(error, "result", getattr(error, "bounded_process_result", None))
    if observation is not None:
        record.update(observation)
    record["accepted_control"] = False
    record["error"] = f"{type(error).__name__}: {error}"
finally:
    record["input_drift"] = []
    record["postrun_errors"] = []
    for path in paths:
        try:
            if pin(path) != record["inputs"][str(path)]:
                record["input_drift"].append(str(path))
        except OSError as error:
            record["postrun_errors"].append(f"{path}: {type(error).__name__}: {error}")
    if record["input_drift"] or record["postrun_errors"]:
        record["accepted_control"] = False
    try:
        (output / "result.json").write_text(json.dumps(record, indent=2) + "\n")
    except OSError as error:
        record["accepted_control"] = False
        record["publication_error"] = f"{type(error).__name__}: {error}"
        # Preserve the original nested outcome if result publication also fails.
        print(json.dumps(record), file=sys.stderr)
print(json.dumps({"scope": record["scope"], "accepted_control": record["accepted_control"]}))
raise SystemExit(0 if record["accepted_control"] else 1)
