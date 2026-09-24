#!/usr/bin/env python3
"""T29 composition (b), executed: a skill package bounds a procedure run through the engine.

    compose.py <skill-dir> <procedure.json> <held-actions,comma,separated> <spec.json-text>

The consumer the packages were built for, and nothing more. It loads the skill package (the
packet's `actions_in_effect` is the caller's held actions narrowed to what the skill needs),
validates the procedure, and then, while `resume()` offers steps, turns each into wrapper
arguments with `dispatch()`, runs the `hee3` wrapper -- which reaches the engine through its
producer -- and folds the reply into the record with `observe()`. It closes with `join()`.
The packages hold no loop, scheduler or timer; this one is bounded by the procedure's step
count, and a composition still offered steps after it is a failure, not a pause.

The wrapper takes its environment from the caller: XDG_RUNTIME_DIR, HEE3_PRODUCER,
HEE3_GRANT_ID, HEE3_SCOPE_SHA256. Prints one JSON document and exits 0, or prints
`{"refused": code, "detail": ...}` and exits 1 when a package refuses, before or between
requests.
"""

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def module(name, relative):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


SKILLS = module("hee3_load_skill", "skills/load_skill.py")
PROCEDURES = module("hee3_validate_procedure", "workflows/validate_procedure.py")


def run(skill_dir, procedure, held, spec):
    manifest = json.loads((Path(skill_dir) / "skill.json").read_text())
    contents = SKILLS.read_package(skill_dir, manifest)
    packet = SKILLS.load(manifest, held, {}, ["operator"], contents,
                         SKILLS.bounds()["max_packet_bytes"])
    order = PROCEDURES.validate(procedure)
    committed = {"procedure_id": procedure["procedure_id"],
                 "procedure_version": procedure["procedure_version"], "steps": {}}
    in_effect = set(packet["actions_in_effect"])
    replies = {}
    for _ in range(len(procedure["steps"])):
        ready = PROCEDURES.resume(procedure, committed)
        if not ready:
            break
        for step_id in ready:
            argv = PROCEDURES.dispatch(procedure, committed, step_id, in_effect, spec)
            ran = subprocess.run(["bash", str(ROOT / "integrations/bash/hee3"), *argv],
                                 capture_output=True, text=True, check=False, timeout=120)
            if ran.returncode != 0:
                raise SystemExit(f"{step_id}: wrapper exit {ran.returncode}: {ran.stderr.strip()}")
            replies[step_id] = json.loads(ran.stdout)
            committed = PROCEDURES.observe(procedure, committed, step_id, replies[step_id])
    if PROCEDURES.resume(procedure, committed):
        raise SystemExit(f"steps still offered after {len(procedure['steps'])} rounds")
    # The integrated verifier: every read step found the task its submit step admitted.
    by_id = {s["step_id"]: s for s in procedure["steps"]}
    reads = [s for s in by_id.values() if s["action"] == "task.get"]
    verified = bool(reads) and all(
        replies.get(s["step_id"], {}).get("body", {}).get("task", {}).get("task_id")
        == replies.get(s["depends_on"][0], {}).get("body", {}).get("task", {}).get("task_id")
        for s in reads)
    disposition, reasons = PROCEDURES.join(procedure, committed, verified, False)
    return {
        "skill": {key: packet[key] for key in
                  ("skill_id", "skill_version", "actions_in_effect", "complete", "omissions")},
        "order": order,
        "record": committed,
        "verified": verified,
        "disposition": disposition,
        "reasons": reasons,
        "keys": {s: PROCEDURES.step_key(procedure, s, None) for s in order},
        "task_ids": {s: replies[s].get("body", {}).get("task", {}).get("task_id")
                     for s in order if s in replies},
    }


def main(arguments):
    if len(arguments) != 4:
        print(__doc__.splitlines()[2].strip(), file=sys.stderr)
        return 2
    skill_dir, procedure_path, held, spec = arguments
    try:
        document = run(skill_dir, json.loads(Path(procedure_path).read_text()),
                       set(held.split(",")), json.loads(spec))
    except (SKILLS.SkillRefusal, PROCEDURES.ProcedureRefusal) as refusal:
        print(json.dumps({"refused": refusal.code, "detail": refusal.detail}, sort_keys=True))
        return 1
    print(json.dumps(document, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
