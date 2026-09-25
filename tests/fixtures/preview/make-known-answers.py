"""Independent known answers for task.preview (B07b; F94/F113).

A second implementation of the preview composition, written from the B07 design
(`~/hee3-evidence/T28/B07-task-preview-20260925/DESIGN.md`, R1-R3) rather than from
src/app/routing.rs: the adapter (a declared recipe joined with its roster record) is written
here, and every rule outcome comes from the T09 route oracle's own `screen`
(tests/fixtures/route/make-known-answers.py), which shares no code with the Rust router.

Adapter, per the design:
* a recipe whose record is absent, another principal's, or disabled -> exclusion `unavailable`;
* capabilities: the task requires exactly its class; a recipe carries it when it `serves` it
  (a roster record cannot claim class suitability, so its labels never do);
* availability: a record with no observation is unobserved; an observation of this fixture is
  current (the test seeds it moments before the preview), so it is observed at age 0; after the
  ledger is reopened (a new receiver epoch, `docs/roster-contract.md`: "Reopen creates a new
  receiver epoch ... invalidating their current freshness") every observation is unobserved;
* locality from the record, figures from the recipe (absent = unknown);
* the task: context 32768 (RC01's input bound), local_only, currency ceiling 0, deadline = the
  work share (wall_ms minus the 300000 ms verification reserve), no quality floor.

Codes, per design P7: R02/R03/R08 capability, R04 privacy, R05 excluded unavailable, R05 gap
stale, R06/R07 budget; eligible = passed every filter (an R11 ranking gap included).
`cost_mode` is `bounded` when anything is eligible, else `unknown` (R1.6).

Run with `--check` to compare the committed table byte for byte (prints `matches_generator=yes`).
"""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

HERE = Path(__file__).resolve().parent
ORACLE = HERE.parent / "route" / "make-known-answers.py"
FIXTURE = HERE / "fixture.json"
ANSWERS = HERE / "known-answers.json"
RESERVE_MS = 300000  # src/contracts/rc01.rs CLEANUP_RESERVE (5 min)
INPUT_TOKENS = 32768  # docs/contract-decisions.md:63
CODE = {"R02": "capability", "R03": "capability", "R08": "capability", "R04": "privacy",
        "R06": "budget", "R07": "budget"}


def oracle():
    spec = importlib.util.spec_from_file_location("route_oracle", ORACLE)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def answers(reopened):
    route = oracle()
    bound = route.declared_bound()
    fixture = json.loads(FIXTURE.read_text())
    records = {r["name"]: r for r in fixture["records"]}
    task = {"required_capabilities": [fixture["class"]], "context_tokens": INPUT_TOKENS,
            "privacy": "local_only", "cost_ceiling_microunits": 0,
            "deadline_ms": fixture["wall_ms"] - RESERVE_MS, "quality_floor_basis_points": None}
    eligible, exclusions = [], []
    for recipe in fixture["recipes"]:
        record = records.get(recipe["record"])
        if record is None or record["name"] == "stranger" or record.get("disabled"):
            exclusions.append({"recipe_id": recipe["id"], "code": "unavailable"})
            continue
        observation = None if reopened else record["observation"]
        screened = {
            "id": recipe["id"],
            "capabilities": [fixture["class"]],
            "context_limit_tokens": recipe.get("context_limit_tokens"),
            "locality": record["locality"],
            "availability": ({"kind": "unobserved"} if observation is None else
                             {"kind": "observed", "availability": observation["availability"], "age_ms": 0}),
            "cost_microunits": recipe.get("cost_microunits"),
            "quality_basis_points": recipe.get("quality_basis_points"),
            "latency_ms": recipe.get("latency_ms"),
        }
        verdict = route.screen(task, screened, bound)
        if verdict[0] == "eligible" or (verdict[0] == "gap" and verdict[1] == "R11"):
            eligible.append({"recipe_id": recipe["id"], "recipe_version": recipe["version"],
                             "adapter_id": recipe["adapter"],
                             "actual_model_required": recipe["actual_model_required"]})
        elif verdict[1] == "R05":
            exclusions.append({"recipe_id": recipe["id"],
                               "code": "unavailable" if verdict[0] == "excluded" else "stale"})
        else:
            exclusions.append({"recipe_id": recipe["id"], "code": CODE[verdict[1]]})
    eligible.sort(key=lambda row: row["recipe_id"].encode())
    exclusions.sort(key=lambda row: row["recipe_id"].encode())
    return {"eligible": eligible, "exclusions": exclusions,
            "cost_mode": "bounded" if eligible else "unknown"}


def main():
    current, reopened = answers(False), answers(True)
    text = json.dumps({"scope": "known answers for tests/fixtures/preview/fixture.json, computed by "
                       "this generator over the T09 route oracle's screen",
                       "answers": current, "after_reopen": reopened}, indent=1, ensure_ascii=True) + "\n"
    if sys.argv[1:] == ["--check"]:
        same = ANSWERS.read_text() == text
        print(f"eligible={len(current['eligible'])} exclusions={len(current['exclusions'])} "
              f"reopened_eligible={len(reopened['eligible'])} matches_generator={'yes' if same else 'no'}")
        sys.exit(0 if same else 1)
    ANSWERS.write_text(text)
    print(f"sha256={hashlib.sha256(text.encode()).hexdigest()}")


if __name__ == "__main__":
    main()
