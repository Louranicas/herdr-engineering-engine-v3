# Cohort evaluation

What a cohort of specialist threads produced, described and compared against a declared
one-worker baseline. Nothing here grants, leases, accepts or admits: the comparison states
differences and stops, because deciding whether a cohort was worth its cost is a policy
question this edition does not answer.

## The comparison

`julia/src/Cohesion.jl::cohesion` takes one request frame and returns bounded UTF-8 bytes —
the same contract as its sibling `analyze`, capped at the 65,536 that `src/numerical.rs`
admits, so a report that would exceed it refuses rather than arriving truncated at a boundary
where the reader cannot tell truncation from a short answer.

It reports, per cohort:

| Section | What it states |
|---|---|
| `counts` | threads, required, and one count per outcome, plus rework and unknown-cost |
| `rates` | met / dissent / rework / error, each carrying its own denominator (`of`) |
| `allocation` | the declared limit and what the parts account for, refusing if they exceed it |
| `join` | the caller's verdict and its reasons, refused unless the two agree |
| `comparison` | cohort cost, errors, disagreement and rework against the declared baseline |
| `excluded` | what was left out of the cost totals and why — never silently dropped |
| `overlapping_claims` | thread pairs whose write claims are not disjoint |

Costs are summed with Kahan compensation, and a thread reporting no usage is excluded by name
rather than counted as zero — the two are different facts and the report keeps them apart.

## One rule, two implementations, one table

`src/cohort.rs::Claim::conflicts_with` and `julia/src/Cohesion.jl::claims_conflict` decide the
same question in two languages: whether two write claims overlap, segment-wise, so that
`src/store` overlaps `src/store/index` but not `src/storefront`.

Two implementations keeping one rule is a promise, and a promise holds only while somebody
remembers. `claim-overlap-v1.json` makes it a mechanism:

- `make-claim-overlap.py` generates the table from a **third** statement of the rule, so the
  answers are not a transcription of either implementation. `--check` re-derives it.
- `tests/t22_cohort.rs::claim_overlap_agrees_with_the_shared_table` reads it from Rust.
- `julia/test/analysis.jl` → `claim overlap agrees with the shared table` reads it from Julia.

Both read every case in both directions, take the case count from the file rather than from a
literal, and require the table to contain both answers — a table that answers one way
discriminates nothing.

**Control (2026-09-22).** Replacing the segment-wise test with a bare character prefix —
`long.starts_with(short)` / `startswith(long, short)` — was planted on each side in turn. Rust
went red naming the case (`shared prefix, different segment: conflicts_with("src/store",
"src/storefront")`); Julia went red on the same row. Neither plant was left in the tree.

The empty path has no row, deliberately: `Claim::new` refuses it with `Refusal::EmptyClaim` and
`thread_row` refuses it with `:schema`, so neither function can be handed one. A row for it
would pin behaviour neither implementation can reach while reading as coverage.

## Scope

These cases compare two implementations of one rule against a generated table. They are not a
measurement of the live tree, they observe no running cohort, and they admit no module.
