"""Build fixture C02 and compute its answers in Python.

The oracle is this file, which implements the rule from the T22 brief in a second language.
It never reads Cohesion.jl, so agreement between the two is evidence and not self-consistency.
C02 differs from C01 in every field the report computes, and puts the two values C01 left on
their identity element -- an empty overlap list and a zero error delta -- off the origin.
"""
import json, hashlib
from fractions import Fraction
from pathlib import Path

U = lambda n: f"{n:08x}-0000-4000-8000-000000000000"
T = lambda n, outcome, rev, req, cost, claims: {
    "thread_id": U(n), "outcome": outcome, "brief_revision": rev,
    "required": req, "cost_tokens": cost, "claims": claims}

threads = [
    T(0x20, "met",           "6", True,  "2400", ["src/budget", "src/notify"]),
    T(0x21, "dissent",       "6", True,   "900", ["src/budget/ledger"]),   # overlaps 0x20
    T(0x22, "met",           "5", True,   "750", ["src/context"]),
    T(0x23, "unmet",         "6", True,   "300", ["src/cohort"]),
    T(0x24, "dissent",       "4", False,  "150", ["docs/b.md"]),
    T(0x25, "met",           "6", True,   "500", ["src/contextual"]),      # NOT an overlap of src/context
]
q = {
    "protocol": "hee3.cohesion", "version": 1, "request_id": U(0x21),
    "subject": {"task_id": U(0x22), "cohort_id": U(0x23), "generation": "31",
                "artifact_sha256": "sha256:" + "c2" * 32},
    "cutoff_unix_ms": "1769999000000", "expires_unix_ms": "1770000000000",
    "recipe": {"id": "cohesion", "version": 1},
    "units": {"usage": "token"}, "brief_revision": "6",
    "join": {"verdict": "integrable", "reasons": []},
    "allocation": {"limit_tokens": "20000", "reserved_tokens": "2000",
                   "spent_tokens": "5000", "unknown_tokens": "0"},
    "baseline": {"cost_tokens": "3000", "errors": "3", "rework": "3"},
    "shape": {"rows": len(threads), "fields": 6},
    "threads": threads,
}
raw = (json.dumps(q, separators=(",", ":")) + "\n").encode()
Path("fixtures/C02.json").write_bytes(raw)

def conflicts(a, b):
    short, long = (a, b) if len(a) <= len(b) else (b, a)
    return long == short or long.startswith(short + "/")

total = len(threads)
required = sum(1 for t in threads if t["required"])
by = lambda name: sum(1 for t in threads if t["outcome"] == name)
met, unmet, dissent, indeterminate = by("met"), by("unmet"), by("dissent"), by("indeterminate")
brief = int(q["brief_revision"])
rework = sum(1 for t in threads if int(t["brief_revision"]) < brief)
unknown_cost = sum(1 for t in threads if t["cost_tokens"] is None)
errors = unmet + indeterminate + dissent - dissent          # errors = unmet + indeterminate
errors = unmet + indeterminate
cohort_cost = sum(int(t["cost_tokens"]) for t in threads if t["cost_tokens"] is not None)
base = q["baseline"]
overlaps = [f'{threads[i]["thread_id"]} {threads[j]["thread_id"]}'
            for i in range(total) for j in range(i + 1, total)
            if any(conflicts(a, b) for a in threads[i]["claims"] for b in threads[j]["claims"])]
alloc = q["allocation"]
answers = {
    "request_sha256": "sha256:" + hashlib.sha256(raw).hexdigest(),
    "counts": {"threads": str(total), "required": str(required), "met": str(met),
               "unmet": str(unmet), "dissent": str(dissent),
               "indeterminate": str(indeterminate), "rework": str(rework),
               "unknown_cost": str(unknown_cost)},
    "rates": {"met": float(Fraction(met, total)), "dissent": float(Fraction(dissent, total)),
              "rework": float(Fraction(rework, total)), "error": float(Fraction(errors, total)),
              "of": str(total)},
    "allocation_accounted_tokens": str(int(alloc["reserved_tokens"]) + int(alloc["spent_tokens"])
                                       + int(alloc["unknown_tokens"])),
    "comparison": {"cohort_cost_tokens": str(cohort_cost), "cohort_errors": str(errors),
                   "cohort_disagreement": str(dissent), "cohort_rework": str(rework),
                   "cost_delta_tokens": str(cohort_cost - int(base["cost_tokens"])),
                   "errors_delta": str(errors - int(base["errors"])),
                   "rework_delta": str(rework - int(base["rework"]))},
    "overlapping_claims": overlaps,
    "excluded": ([f"{unknown_cost} threads report no usage; cost totals exclude them"]
                 if unknown_cost > 0 else ["none"]),
}
Path("fixtures/C02-known-answers.json").write_text(json.dumps(answers, indent=1) + "\n")
print(json.dumps(answers, indent=1))
