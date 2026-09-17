# Herdr Engineering Engine v3

**A bounded, auditable execution and evidence foundation for agent work — and, more unusually, a
corpus that refuses to claim it is finished.**

> **Status: paused, deliberately.** 5 of 22 modules are implemented, **0 are accepted**, and the
> admission validator is not built. `admission_validator_available: false` is *correct output*, not
> a gap someone forgot to close. Engine coding resumes only on an explicit human instruction.

---

## What this is

HEE-v3 turns an operator's task into bounded, auditable work: choose a suitable model and worker,
coordinate specialist threads, verify their outputs, repair failures, and retain proof of the
accepted result.

It is built around one idea that shapes everything else:

> **A verdict is a statement about the check, never about the world.**

Consequently the interesting part of this repository is not the Rust. It is the **machinery that
makes it very hard to claim something is done when it is not** — seven separate lifecycle
dimensions, each with an explicit list of what *cannot* establish it, and a structural rule that no
layer may promote the layer beneath it.

## Current state, measured

| | |
|---|---:|
| Modules declared | 22 across 6 clusters |
| Implemented | **5** (+1 seed) |
| Stubs | 16 |
| **Accepted modules** | **0** |
| `src/` | 29 files · 5,504 lines |
| `tests/` | 17 files · 9,723 lines · **289 `#[test]`** |
| Test : source | **1.77 : 1** |
| Mutation testing | **28 mutants, 28 killed**, each run closing on a restored benign baseline |
| Completion gates defined | 13 |
| Public operation obligations | 66 |

Implemented: `worker` (2,014) · `store` (761) · `roster` (301) · `task` (159) · `contracts` (156) ·
`julia` (seed). Everything else is a declared, registered stub — 65 of them, disjoint from the 255
implementation subjects.

> Counting rule: strip the managed `HEE3-ANCHORS-BEGIN..END` comment block, then count non-blank
> lines that do not begin with `//`. Do **not** also exclude `#` — in Rust that is an attribute,
> which is code.

## Code quality, as evidence rather than assertion

Measured across `src/`:

```
unwrap()  0     expect(  0     panic!  0     #[allow]  0     todo!  0     unimplemented!  0
#![forbid(unsafe_code)]
```

The single suppression in the tree is `#[cfg_attr(not(test), allow(clippy::unnecessary_wraps))]` —
conditional and block-scoped, the lowest rung that works.

### Three patterns worth stealing

**1 · A crash seam that compiles away.** `store` declares
`CutPoint::{MigrationWrite, RosterWrite, TaskWrite, BeforeCommit, AfterCommit}`. `check_point()` is
`#[cfg(test)]`-gated and `fault()` returns `None` in release, so every write boundary is
*independently interruptible* in tests at zero shipped cost. The fault is **passed in**, never
re-acquired — a seam, not a flag.

**2 · Bounds and idempotency at the point of acquisition.** `Store::submit` refuses
`request_bytes.len() > 1_048_576` *before* allocating, then keys on `request_key`: a prior row with
the same digest returns the stored result, a different digest returns `Error::Conflict`. Same key /
different body conflicts, by construction. `TransactionBehavior::Immediate` takes the write lock at
`BEGIN` rather than risking SQLite's upgrade deadlock, and `sync_all()` is called on the file **and
its containing directory**.

**3 · States made unrepresentable.** `worker` carries a dozen enums — `Phase`, `Terminal`,
`CancelReason`, `UsageScope/Form/Stage`, `Finish`, `IdentityOrigin` — so wrong combinations cannot
be constructed. `Submission<'a>` borrows rather than copies; numeric conversion goes through a
checked helper rather than `as`.

## The seven lifecycle dimensions

This is the heart of the design. Each dimension has an explicit *"what cannot establish it"*:

| Dimension | What cannot establish it |
|---|---|
| Source kind | a file existing does not prove an implemented contract |
| Testing observation | zero tests, skipped checks, or a worker's "done" message |
| Hardening observation | a lint pass or an unexamined absence of findings |
| Deployment observation | copying files, starting a process, or a healthy listener |
| Accepted module | source presence, compile success, or a documentation publication |
| Accepted codebase | a sum of accepted modules |
| Task acceptance | any of the above |

Because promotion between them is structurally forbidden, the corpus can hold 22 modules, 65 paths
and a 42,100-node graph without ever drifting into a false completion claim.

## Architecture

Six clusters, read off the dependency DAG. Every module belongs to exactly one; there are no
orphans.

| Cluster | Owns | Modules |
|---|---|---|
| **K1** | Task ownership and recovery | contracts · task · store · budget · recovery |
| **K2** | Worker routing and capability | roster · route · worker |
| **K3** | Thread context and cohesion | cohort · context · notify · skills · workflows |
| **K4** | Verification and numerical evidence | check · numerical · julia |
| **K5** | Habitat operation and presentation | service · herdr |
| **K6** | Application assembly | app · actions · bash · pi_extension |

`contracts` is the root: **0 dependencies, 16 consumers.** Rust owns coordination and trust
boundaries; Julia is a bounded analysis boundary, not a second engine.

Module-to-module behaviour is declared as **20 semantic flows**, each with a forward *and* a return
arrow — 40 directional arrows across 18 matrix cells. Every return correctly reverses its forward on
both the cluster pair and the module pair. These are declared contracts, **not** build imports or
measured runtime calls.

## The gap that everything routes around

`check` (K4) is 315 lines of navigation comment and **zero implementation**. It owns the sharpest
unanswered question in the corpus:

> *Who owns the oracle and the raw receipt? Can candidate code fabricate a pass, or mutate evidence,
> while the collector runs?*

That is the T06 qualification collector. Until it exists, nothing can be admitted — which is exactly
why `accepted_modules` is zero and why every evidence record is stamped `module_admission: false`.

## Repository layout

```
src/            22 module anchors; 5 implemented, 16 stubs
tests/          289 #[test] across 17 files
julia/          bounded numerical analysis boundary
migrations/     migration 001, unreleased draft; immutable once released
schemas/        JSON Schema for the action surface
tools/          check-quality, mutation runners, corpus-sync launcher
docs/           contracts, standards, module-context cards (the agent router)
corpus/         anchors.json, publication.json, UPDATE_PROTOCOL.md, receipts
runbooks/       RB01 inspect · RB02 publish · RB03 module completion · RB04 release · RB05 security
deploy/ config/ evaluation/ integrations/ skills/ workflows/
```

**Excluded from version control** (regenerable, ~950 MB): `target/`, `corpus/graphify-out/` (a
492 MB derived graph with byte-identical copies in two other roots), `evidence/` (285 MB of captured
runs), and vendored offline build inputs.

## Getting oriented

```bash
./tools/corpus-sync --check          # is the published corpus internally consistent?
just --list                          # documentation recipes
just handoff                         # current authority and the next dependency-ready task
cat docs/module-context/<module>.md  # the progressive-disclosure card for one module
```

Start at `QUICK_START.md`, then `corpus/UPDATE_PROTOCOL.md` — the governing document, which states
one owner per kind of fact. Per-module reading routes live in `docs/module-context/`, with a
five-stage disclosure ladder (scope → orientation → boundary closure → quality → targeted research),
each stage naming what to carry forward.

**The justfile is deliberately documentation-only.** There are no engine build, test, migrate or
deploy recipes yet; RB03 and RB04 are declared *planned*, gated on the collector. The real quality
matrix is `tools/check-quality`.

## Conventions that will surprise you

- **`README.md`, `QUICK_START.md` and `AGENTS.md` are generated outputs.** Editing them directly
  turns `corpus-sync --check` red. Change the owning record and regenerate. (This file lives in
  `.github/` precisely so it is not one of them.)
- **Captured snapshots are byte-preserved.** Anything under a capture path records what was true
  *at capture time*; "repairing" a stale line number inside one destroys the only evidence of it.
- **Every source file carries a managed anchor comment block.** Write implementation *outside*
  it; the synchroniser preserves the bytes. It is large, and it is why naive line counts mislead.

## Provenance and honesty

Numbers in this README were measured against the tree, not recalled. Where the corpus cannot support
a claim it says so: `accepted_interface_modules: 0`, `completion_admission_validator_available:
false`, `full_testing_qualification: unassessed`, `security_qualification: unassessed`.

A recorded corpus self-assessment scores preparation at 87/100 across seven facets. An independent
review scored the same facets at 69/100, with the largest gaps in build/deployment operations and in
corpus cohesion. **Both numbers are preparation estimates, not release readiness** — a production
judgement needs admitted implementations and operational results that do not yet exist.

## Licence and status

No licence file is present; all rights reserved unless one is added. This is an in-progress
engineering corpus published for inspection, not a supported product. Nothing here is running, and
no deployment is authorised.
