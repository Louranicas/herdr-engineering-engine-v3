"""Independent known-answer oracle for T09 routing (F94/F113).

A second implementation of the declared policy in config/routes.toml, written
from the declaration and the rule table rather than from src/route.rs: the
filters are plain predicates applied in the declared order, the ranking is
Python's tuple sort, and a tie is a top group of size two or more. It reads the
shared fixture, writes known-answers.json beside itself and prints the sha256
of what it wrote. It shares no code with the Rust router.

Moved into the repository 2026-09-24 (review N9): it had lived only in the T09 evidence store
(`~/hee3-evidence/T09/route-20260922/oracle/route_oracle.py`), so a clone could read the table
tests/t09_route.rs pins but not the independent source of its answers. Only the two paths and
`--check` changed; the table it writes is byte-identical to the retained one, including its
`scope` text, which names the file's original location.

Extended 2026-09-24 (route-G4): a null `context_limit_tokens` is unknown and screens as a gap at
R03 (`missing_context_limit`), per docs/modules/route.md "Missing measurements stay unknown";
fixture recipe r-13 carries one and task T13-vision reaches it alone.

Extended 2026-09-24 (route-G1, operation 3 "Evaluate fallback"): each row of the fixture's
`fallbacks` names a task, the recipe whose attempt failed and its failure category. The oracle
decides that task again with the named recipe excluded before any filter (R13), candidate or
baseline alike, and every other rule unchanged; a baseline so excluded refuses a fallback that
needs it, exactly as a baseline excluded by a filter does. The rows go to `fallback_answers`.

Run `python3 tests/fixtures/route/make-known-answers.py --check` to re-derive the committed table
and compare bytes (prints `matches_generator=yes`, exit 0; otherwise exit 1). Without `--check`
it rewrites the table.
"""
from pathlib import Path
import hashlib
import json
import sys

HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixture.json"
CONFIG = HERE.parents[2] / "config/routes.toml"
STALENESS_BOUND_MS = 30000  # config/routes.toml: staleness_bound_ms
FILTER_ORDER = ["required_capabilities", "context_limit", "privacy_class", "availability", "cost_ceiling", "deadline", "quality_floor"]
RULE_ID = {"required_capabilities": "R02", "context_limit": "R03", "privacy_class": "R04", "availability": "R05",
           "cost_ceiling": "R06", "deadline": "R07", "quality_floor": "R08", "ranking": "R11",
           "previous_attempt": "R13"}


def declared_bound():
    # Read the declared bound back from the configuration text so the oracle and the
    # router agree on the same declared value; a mismatch is refused, not patched.
    for line in CONFIG.read_text().splitlines():
        if line.startswith("staleness_bound_ms"):
            value = int(line.split("=")[1].strip())
            if value != STALENESS_BOUND_MS:
                raise SystemExit(f"declared bound {value} differs from the oracle's {STALENESS_BOUND_MS}")
            return value
    raise SystemExit("staleness_bound_ms not declared")


def screen(task, recipe, bound):
    """Return ('eligible', key) | ('excluded', rule) | ('gap', rule, name)."""
    for name in FILTER_ORDER:
        rule = RULE_ID[name]
        if name == "required_capabilities":
            if any(c not in recipe["capabilities"] for c in task["required_capabilities"]):
                return ("excluded", rule)
        elif name == "context_limit":
            # A null limit is unknown (the roster declares none): a gap, never a guessed number.
            if recipe["context_limit_tokens"] is None:
                return ("gap", rule, "missing_context_limit")
            if recipe["context_limit_tokens"] < task["context_tokens"]:
                return ("excluded", rule)
        elif name == "privacy_class":
            if task["privacy"] == "local_only" and recipe["locality"] != "local":
                return ("excluded", rule)
        elif name == "availability":
            obs = recipe["availability"]
            if obs["kind"] == "unobserved":
                return ("gap", rule, "unobserved")
            if obs["age_ms"] >= bound:
                return ("gap", rule, "stale_availability")
            if obs["availability"] == "unavailable":
                return ("excluded", rule)
            if obs["availability"] == "unknown":
                return ("gap", rule, "availability_unknown")
        elif name == "cost_ceiling":
            if task["cost_ceiling_microunits"] is not None:
                if recipe["cost_microunits"] is None:
                    return ("gap", rule, "missing_cost")
                if recipe["cost_microunits"] > task["cost_ceiling_microunits"]:
                    return ("excluded", rule)
        elif name == "deadline":
            if task["deadline_ms"] is not None:
                if recipe["latency_ms"] is None:
                    return ("gap", rule, "missing_latency")
                if recipe["latency_ms"] > task["deadline_ms"]:
                    return ("excluded", rule)
        elif name == "quality_floor":
            if task["quality_floor_basis_points"] is not None:
                if recipe["quality_basis_points"] is None:
                    return ("gap", rule, "missing_quality")
                if recipe["quality_basis_points"] < task["quality_floor_basis_points"]:
                    return ("excluded", rule)
    for figure, name in (("cost_microunits", "missing_cost"), ("quality_basis_points", "missing_quality"), ("latency_ms", "missing_latency")):
        if recipe[figure] is None:
            return ("gap", RULE_ID["ranking"], name)
    return ("eligible", (recipe["cost_microunits"], -recipe["quality_basis_points"], recipe["latency_ms"]))


def decide(task, recipes, baseline, bound, previous=None):
    # A recipe whose attempt failed is excluded before any filter; nothing else changes.
    def screen_or_previous(recipe):
        if previous is not None and recipe["id"] == previous:
            return ("excluded", RULE_ID["previous_attempt"])
        return screen(task, recipe, bound)

    # The baseline is screened first and its defect is held; it refuses the decision only
    # when a fallback (gap, empty eligible set or tie) actually requires the baseline.
    guard = screen_or_previous(baseline)
    baseline_steps = {"exclusions": [], "gaps": []}
    defect = None
    if guard[0] == "excluded":
        defect = ["baseline_excluded", guard[1]]
        baseline_steps["exclusions"].append([baseline["id"], guard[1]])
    elif guard[0] == "gap":
        defect = ["baseline_evidence", guard[1]]
        baseline_steps["gaps"].append([baseline["id"], guard[1], guard[2]])
    exclusions, gaps, eligible = list(baseline_steps["exclusions"]), list(baseline_steps["gaps"]), []
    for recipe in sorted(recipes, key=lambda r: r["id"]):
        verdict = screen_or_previous(recipe)
        if verdict[0] == "excluded":
            exclusions.append([recipe["id"], verdict[1]])
        elif verdict[0] == "gap":
            gaps.append([recipe["id"], verdict[1], verdict[2]])
        else:
            eligible.append((verdict[1], recipe["id"]))
    eligible.sort()  # tuple order: cost ascending, quality descending, latency ascending, then id
    common = {"exclusions": exclusions, "gaps": gaps}
    candidate_gaps = [g for g in gaps if g[0] != baseline["id"]]
    order = []  # a ranking is reported only when it was performed: no gap, a non-empty eligible set
    if candidate_gaps:
        fallback, decided = ["insufficient_evidence"], "R09"
    elif not eligible:
        fallback, decided = ["no_eligible_candidate"], "R10"
    else:
        order = [rid for _, rid in eligible]
        top = [rid for key, rid in eligible if key == eligible[0][0]]
        if len(top) == 1:
            return {"route": "chosen", "recipe": order[0], "reason": None, "decided_by": "R11", "order": order, **common}
        fallback, decided = ["tie", top], "R12"
    if defect is None:
        return {"route": "baseline", "recipe": baseline["id"], "reason": fallback, "decided_by": decided, "order": order, **common}
    return {"route": "refused", "recipe": None, "reason": defect + [fallback], "decided_by": "R01", "order": order, **common}


def main():
    bound = declared_bound()
    fixture = json.loads(FIXTURE.read_text())
    answers = [{"task": task["id"], "answer": decide(task, fixture["recipes"], fixture["baseline"], bound)} for task in fixture["tasks"]]
    tasks = {task["id"]: task for task in fixture["tasks"]}
    known = {recipe["id"] for recipe in fixture["recipes"]} | {fixture["baseline"]["id"]}
    fallback_answers = []
    for row in fixture["fallbacks"]:
        if row["previous"] not in known:
            raise SystemExit(f"fallback row names unknown recipe {row['previous']}")
        fallback_answers.append({**row, "answer": decide(tasks[row["task"]], fixture["recipes"], fixture["baseline"], bound,
                                                          previous=row["previous"])})
    document = {"scope": "known answers computed by oracle/route_oracle.py, an independent Python implementation of config/routes.toml over tests/fixtures/route/fixture.json",
                "fixture_sha256": hashlib.sha256(FIXTURE.read_bytes()).hexdigest(), "staleness_bound_ms": bound, "answers": answers,
                "fallback_answers": fallback_answers}
    text = json.dumps(document, indent=1) + "\n"
    table = HERE / "known-answers.json"
    if "--check" in sys.argv[1:]:
        committed = table.read_bytes()
        same = committed == text.encode()
        print(f"answers={len(answers)} fallback_answers={len(fallback_answers)} matches_generator={'yes' if same else 'no'} "
              f"committed_sha256={hashlib.sha256(committed).hexdigest()}")
        sys.exit(0 if same else 1)
    table.write_text(text)
    print(json.dumps({"answers": len(answers), "sha256": hashlib.sha256(text.encode()).hexdigest(),
                      "routes": [(a["task"], a["answer"]["route"], a["answer"]["recipe"], a["answer"]["decided_by"]) for a in answers]}, indent=1))


if __name__ == "__main__":
    main()
