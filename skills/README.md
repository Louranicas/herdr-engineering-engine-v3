# Skills

Versioned packages of instructions and bounded references. A skill supplies **text, never
authority**: nothing here grants an action, accepts a task, changes a task's criteria, or
starts anything.

| File | What it is |
|---|---|
| `skill-v1.schema.json` | The manifest shape. A build output. |
| `generate_skill_schema.py` | Authors it. `--check` compares exact bytes. |
| `load_skill.py` | Discovery, loading and revision — the obligations a schema cannot state. |
| `examples/receipt-reading/` | A reference package, version 2, superseding 1. |
| `../tests/skill_schema.py` | 44 cases. |

## A skill cannot widen authority

`load()` takes the actions the caller **already holds**. A manifest naming one outside that
set is refused as `authority_widening`, and there is no branch that adds an action — so "a
loaded skill granted itself a capability" is not a state this module can reach. The packet
reports `actions_in_effect`, which is the *intersection*: no set wider than the caller's own
appears anywhere in it.

The test for this is deliberately not a search for the word "grant". The packet's own scope
line says "No grant", so a substring test passes whether or not the rule holds — a needle the
output carries either way. The checkable claim is the subset relation, asserted over four
scope sets.

## What is not carried is named

Scope denial, a budget cutoff, a stale hash, a reference over its size bound and a traversal
bound all produce an **omission** carrying the reference id and the reason. A packet is never
quietly short, because a short packet that looks complete is the failure that bounding exists
to make visible. `complete` is `false` whenever anything was left out.

`denied_scope` and `stale_reference` are omission reasons and deliberately **not** refusals:
refusing a whole load over one reference would throw away the instructions the caller can
legitimately have. The schema's `hee3.refusals` and `hee3.omission_reasons` are disjoint, and
a test asserts it.

## Pins are exact

A dependency names `skill_id` and `skill_version` exactly — never a range. A range would let
a dependency change under a consumer that pinned this skill precisely to stop that. A
published revision is immutable; `revise()` refuses a version that does not advance, and
returns the drift record naming what changed and which consumers pinned the old one.

A reference carries the `sha256` it was reviewed against. Content that no longer hashes to it
is omitted as `stale_reference`, never silently substituted.

## Why the case count is not the evidence

`tools/check-skill-sites` neuters each refusal site in `load_skill.py` — enumerated from the
file's own syntax tree — and requires the suite to go red. Its first run read `sites=15
killed=14 survived=1`: the dependency *count* bound was unpinned, because the case fed it
seventeen dependencies that were all absent, so the per-dependency check answered instead and
the same code came back. The case now supplies every dependency as present, leaving only the
count able to fire. `sites=15 killed=15 survived=0`.

Counting refusal *names* would have read `10/10` throughout, while five of those ten names
were raised from more than one place.

## Scope

Instructions and bounded references. Grants nothing, accepts nothing, rewrites no task
criterion, widens no authority, and starts no daemon.
