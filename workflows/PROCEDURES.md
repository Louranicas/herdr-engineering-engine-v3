# Workflows

A finite, versioned procedure over actions that are already admitted. This module describes
procedures and decides nothing: it starts no scheduler, grants no action, performs no effect,
and never accepts a parent.

## What is here

| File | What it is |
|---|---|
| `procedure-v1.schema.json` | The procedure shape. A build output — see below. |
| `generate_procedure_schema.py` | Authors that schema. `--check` compares exact bytes. |
| `validate_procedure.py` | The obligations a schema cannot state: acyclicity, completeness, budget, resume safety. |
| `examples/verify-and-close-v1.json` | A reference procedure: preview → submit → threads → verify → record. |
| `../tests/procedure_schema.py` | 46 cases over both. |

## The action vocabulary is not copied here

A step's `action` is a closed enum, and that enum is **read from
`schemas/actions/control-v1.schema.json`** when the schema is generated — never retyped. This
is what makes *"dynamic text cannot insert unapproved actions"* a mechanism rather than a
rule somebody follows: an action name outside the catalogue cannot be spelled in a valid
procedure at all.

It also means `generate_procedure_schema.py --check` fails when the catalogue changes, which
is the intended behaviour, not a nuisance. A procedure schema that quietly admits a stale
action vocabulary is the stale-version problem this module exists to refuse. Verified by
adding `task.forge` to the catalogue: `--check` went red naming the drift, and the artifact
regenerates to match once the change is deliberate.

## Three things that are structural, not advisory

**A finite procedure is finite by construction.** There is no loop form and no expansion
form, so the step count is known before anything runs. `max_steps` is 64.

**`effect_unknown` is a state, not an error.** A step whose service effect could not be
observed is never retried — `retry_on` cannot even name it — and `resume()` refuses to offer
it, because repeating an effect that may already have landed is the harm the state exists to
name. It waits for a reconciliation record instead.

**No branch returns an acceptance.** `join()` returns `completion_candidate`, `repair` or
`blocked`. Whether the parent is accepted is decided independently, and there is no code path
here that could make a child's success into the parent's.

## Why the case count is not the evidence

Forty-six passing cases say nothing about whether the rules are load-bearing.
`tools/check-workflow-sites` neuters each refusal site in `validate_procedure.py` in turn —
the sites enumerated from the file's own syntax tree, never from a list — and requires the
suite to go red.

That sweep earned its keep immediately. The first run read `sites=17 killed=16 survived=1`:
an explicit self-dependency check was unpinned, because neutering it let the case fall
through to Kahn's remainder, which raises the same `cycle` code. Two sites keeping one rule
meant neither was pinned. The self-dependency check was **removed** rather than given a
distinguishing message — a cycle of length one is a cycle, and one door needs no agreement.
The sweep now reads `sites=16 killed=16 survived=0`.

The same shape is why `test_every_refusal_site_has_a_case` takes its denominator from the
validator's syntax tree and its numerator from what the cases actually asserted as they ran.
Counting refusal *names* would have read `13/13` while four of those names were raised from
two places each.

## Scope

Describes procedures. Starts no scheduler, grants no action, accepts no parent, admits no
module, and observes no running system.
