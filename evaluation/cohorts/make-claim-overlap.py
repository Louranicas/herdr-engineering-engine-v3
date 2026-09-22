"""Generate the claim-overlap table both implementations are checked against.

`src/cohort.rs::Claim::conflicts_with` and `julia/src/Cohesion.jl::claims_conflict` state the
same rule in two languages. Two implementations keeping one rule is a promise; the promise
holds only while someone remembers. This table makes it a mechanism: one file, generated from
a THIRD statement of the rule (below), read by a test on each side.

The rule, as the T22 brief states it:

    Two claims overlap when one names the other, or when one is an ancestor of the other on a
    SEGMENT boundary. `src/store` overlaps `src/store/index`; it does not overlap
    `src/storefront`, because `front` is not a new segment.

Run `python3 evaluation/cohorts/make-claim-overlap.py --check` to verify the committed table
still matches this generator; that is what the Rust and Julia tests assert against.
"""

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
TABLE = HERE / "claim-overlap-v1.json"


def overlaps(a: str, b: str) -> bool:
    """The rule, stated a third time. Deliberately not a transcription of either side."""
    left, right = a.split("/"), b.split("/")
    if len(left) > len(right):
        left, right = right, left
    return right[: len(left)] == left


# Cases are grouped by the distinction each one is here to draw, so a case cannot be deleted
# without deleting the reason it existed.
CASES = [
    ("identical", "src/store", "src/store"),
    ("identical single segment", "src", "src"),
    ("parent of child", "src/store", "src/store/index"),
    ("grandparent of descendant", "src", "src/store/index/mod.rs"),
    ("child of parent, argument order reversed", "src/store/index", "src/store"),
    ("root segment is an ancestor", "src", "src/store"),
    # The discriminating pair: a shared PREFIX that is not a shared SEGMENT. A rule written
    # with a bare `starts_with` passes every case above and fails exactly this one.
    ("shared prefix, different segment", "src/store", "src/storefront"),
    ("shared prefix, reversed", "src/storefront", "src/store"),
    ("shared prefix deeper in the path", "src/store/index", "src/store/indexer"),
    ("one character apart at the leaf", "a/bb", "a/b"),
    ("siblings", "src/store", "src/route"),
    ("siblings under a shared parent", "src/store/a", "src/store/b"),
    ("disjoint roots", "docs/a.md", "src/store"),
    ("same leaf under different parents", "a/x", "b/x"),
    ("file extension does not make a segment", "src/main", "src/main.rs"),
    ("trailing separator makes an empty final segment", "a/b", "a/b/"),
    ("case differs in one segment", "src/Store", "src/store"),
    ("prefix at the root segment only", "s", "src/store"),
    ("deep identical paths", "a/b/c/d/e", "a/b/c/d/e"),
    ("deep paths diverging at the last segment", "a/b/c/d/e", "a/b/c/d/f"),
    ("ancestor several segments up", "a/b", "a/b/c/d/e"),
]
# Deliberately absent: the empty path. `Claim::new` refuses it with `Refusal::EmptyClaim` and
# `thread_row` refuses it with `:schema`, so neither `conflicts_with` nor `claims_conflict`
# can be handed one. A row for it would pin unreachable behaviour and read as coverage.


def build():
    return {
        "schema": "hee3.evaluation.claim-overlap.v1",
        "rule": (
            "Two claims overlap when one is a prefix of the other on a segment boundary; "
            "a shared character prefix that does not end a segment is not an overlap."
        ),
        "generator": "evaluation/cohorts/make-claim-overlap.py",
        "read_by": [
            "src/cohort.rs::Claim::conflicts_with (tests/t22_cohort.rs)",
            "julia/src/Cohesion.jl::claims_conflict (julia/test/analysis.jl)",
        ],
        "scope": (
            "Agreement between two implementations of one rule. This table is not a "
            "measurement of the live tree and admits nothing."
        ),
        "cases": [
            {"name": name, "a": a, "b": b, "overlap": overlaps(a, b)} for name, a, b in CASES
        ],
    }


def main():
    table = build()
    names = [case["name"] for case in table["cases"]]
    if len(names) != len(set(names)):
        raise SystemExit("case names must be distinct; they are how a failure is reported")
    # A table whose every answer is the same value discriminates nothing.
    answers = {case["overlap"] for case in table["cases"]}
    if answers != {True, False}:
        raise SystemExit(f"the table must contain both answers; it contains {answers}")
    text = json.dumps(table, indent=2, sort_keys=False) + "\n"
    if "--check" in sys.argv:
        if not TABLE.exists():
            raise SystemExit(f"{TABLE} is absent")
        if TABLE.read_text() != text:
            raise SystemExit(f"{TABLE} differs from this generator; regenerate or fix the rule")
        print(f"claim-overlap table: cases={len(names)} matches_generator=yes")
        return
    TABLE.write_text(text)
    print(f"wrote {TABLE.name}: cases={len(names)} "
          f"true={sum(c['overlap'] for c in table['cases'])} "
          f"false={sum(not c['overlap'] for c in table['cases'])}")


main()
