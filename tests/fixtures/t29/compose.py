#!/usr/bin/env python3
"""T29 composition (b), executed: a skill package bounds a procedure run through the engine.

    compose.py <skill-dir> <procedure.json> <held-actions,comma,separated> <spec.json-text>
               [--lose STEP] [--unsent STEP]

The consumer the packages were built for, and nothing more. It loads the skill package (the
packet's `actions_in_effect` is the caller's held actions narrowed to what the skill needs),
validates the procedure, and then, while `resume()` offers steps, turns each into wrapper
arguments with `dispatch()`, runs the `hee3` wrapper -- which reaches the engine through its
producer -- and folds the reply into the record with `observe()`. It closes with `join()`.
The packages hold no loop, scheduler or timer; this one is bounded by the procedure's step
count, and a composition still offered steps after it is a failure, not a pause.

Before every `resume()`, each `effect_unknown` step is reconciled first: `reconcile_argv()`,
the wrapper, `reconcile()`; the reply that settled it is reported under `licences`. Two
failure injections exercise that path through the real engine (WF-14), stated for what they
are -- the consumer's, not the wire's: `--lose STEP` sends STEP's request and, after the engine
has answered, DISCARDS the reply as if it were lost; `--unsent STEP` records STEP as sent
without sending it, so the readback must find nothing. Every wrapper call is reported under
`calls`, so a case can prove an effect was requested exactly once.

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
# The producer's exit code for an engine error record (src/main.rs EXIT_REFUSED).
EXIT_REFUSED = 7


def module(name, relative):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


SKILLS = module("hee3_load_skill", "skills/load_skill.py")
PROCEDURES = module("hee3_validate_procedure", "workflows/validate_procedure.py")


def wrapper(argv):
    ran = subprocess.run(["bash", str(ROOT / "integrations/bash/hee3"), *argv],
                         capture_output=True, text=True, check=False, timeout=120)
    # 0: a result record. 7: the engine answered with a typed error record (BASH-G1), which the
    # composition reads like any other answer. Anything else is the door failing, not an answer.
    if ran.returncode not in (0, EXIT_REFUSED):
        raise SystemExit(f"{argv[0]}: wrapper exit {ran.returncode}: {ran.stderr.strip()}")
    return json.loads(ran.stdout)


def run(skill_dir, procedure, held, spec, lose=(), unsent=()):
    manifest = json.loads((Path(skill_dir) / "skill.json").read_text())
    contents = SKILLS.read_package(skill_dir, manifest)
    packet = SKILLS.load(manifest, held, {}, ["operator"], contents,
                         SKILLS.bounds()["max_packet_bytes"])
    order = PROCEDURES.validate(procedure)
    committed = {"procedure_id": procedure["procedure_id"],
                 "procedure_version": procedure["procedure_version"], "steps": {}}
    in_effect = set(packet["actions_in_effect"])
    lose, unsent = set(lose), set(unsent)
    replies, calls, licences = {}, [], {}
    # A step is dispatched, reconciled, and at most dispatched again: three rounds each.
    rounds = 3 * len(procedure["steps"])
    for _ in range(rounds):
        unknown = sorted(s for s, state in committed["steps"].items() if state == "effect_unknown")
        for step_id in unknown:
            argv = PROCEDURES.reconcile_argv(procedure, committed, step_id)
            calls.append([step_id, argv[0]])
            licences[step_id] = wrapper(argv)
            committed = PROCEDURES.reconcile(procedure, committed, step_id, licences[step_id])
            if committed["steps"].get(step_id) == "done":
                replies[step_id] = licences[step_id]
        if unknown:
            continue
        ready = PROCEDURES.resume(procedure, committed)
        if not ready:
            break
        for step_id in ready:
            argv = PROCEDURES.dispatch(procedure, committed, step_id, in_effect, spec)
            if step_id in unsent:
                unsent.discard(step_id)
                committed = PROCEDURES.unanswered(procedure, committed, step_id)
                continue
            calls.append([step_id, argv[0]])
            reply = wrapper(argv)
            if step_id in lose:
                lose.discard(step_id)
                committed = PROCEDURES.unanswered(procedure, committed, step_id)
                continue
            replies[step_id] = reply
            committed = PROCEDURES.observe(procedure, committed, step_id, reply)
    if PROCEDURES.resume(procedure, committed) or any(
            state == "effect_unknown" for state in committed["steps"].values()):
        raise SystemExit(f"the composition did not settle in {rounds} rounds")
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
        "calls": calls,
        "licences": licences,
        "task_ids": {s: replies[s].get("body", {}).get("task", {}).get("task_id")
                     for s in order if s in replies},
    }


def main(arguments):
    positional, injected = arguments[:4], arguments[4:]
    lose, unsent = [], []
    while len(injected) >= 2 and injected[0] in ("--lose", "--unsent"):
        (lose if injected[0] == "--lose" else unsent).append(injected[1])
        injected = injected[2:]
    if len(positional) != 4 or injected:
        print(__doc__.splitlines()[2].strip(), file=sys.stderr)
        return 2
    skill_dir, procedure_path, held, spec = positional
    try:
        document = run(skill_dir, json.loads(Path(procedure_path).read_text()),
                       set(held.split(",")), json.loads(spec), lose, unsent)
    except (SKILLS.SkillRefusal, PROCEDURES.ProcedureRefusal) as refusal:
        print(json.dumps({"refused": refusal.code, "detail": refusal.detail}, sort_keys=True))
        return 1
    print(json.dumps(document, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
