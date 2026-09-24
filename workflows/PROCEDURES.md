# Workflows

A finite, versioned procedure over actions that are already admitted. This module describes
procedures and decides nothing: it starts no scheduler, grants no action, performs no effect,
and never accepts a parent.

## What is here

| File | What it is |
|---|---|
| `procedure-v1.schema.json` | The procedure shape. A build output — see below. |
| `generate_procedure_schema.py` | Authors that schema. `--check` compares exact bytes. |
| `validate_procedure.py` | The obligations a schema cannot state: acyclicity, completeness, budget, resume safety. Also the command line (below). |
| `examples/verify-and-close-v1.json` | A reference procedure: preview → submit → threads → verify → record. |
| `../tests/procedure_schema.py` | 75 cases over both. |

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
name. It waits for a reconciliation record instead. The same holds at the join for an **optional**
step: optional means the parent may close *without* that step's outcome, not *over* an effect
that may have landed unobserved, so `effect_unknown` blocks `join()` whether or not the step is
required.

**No branch returns an acceptance.** `join()` returns `completion_candidate`, `repair` or
`blocked`. Whether the parent is accepted is decided independently, and there is no code path
here that could make a child's success into the parent's.

## What a procedure is checked against

**Shape.** `validate()` walks the schema's own `type`, `required`, `properties` and `items`
before indexing anything, so a missing or mistyped member is `malformed_procedure` naming its
path, never a `KeyError`. The schema is the one statement of shape; the validator reads it
rather than keeping a second list. A `true` is not an integer here, though Python says it is.

**Action versions.** The generator reads each action's pinned `action_version` from the
catalogue's request definitions into `hee3.action_versions`. A step pinning any other version
is `unsupported_action_version` here, rather than a refusal at the receiver.

**Records are bound.** `resume()` and `join()` take a record `{procedure_id, procedure_version,
steps}` and read it through one function, so the two cannot disagree about what a foreign
record is. Another procedure's record, or a foreign step id or unknown state, is
`identity_mismatch`; the same procedure at another version, whose step ids may coincide while
meaning different things, is `stale_procedure_version`.

**Budget exhaustion blocks.** `join(procedure, outcomes, verified, exhausted)` takes the
caller's observation that the budget ran out, as it takes the verifier's verdict, because
nothing here runs or meters a procedure. Neither has a default. Only reasons `join()` can
append are declared: `dissent` and `stale_version` were declared with no input able to
produce them and have been removed, and `block_reason_sites()` holds the declaration to the
source as `refusal_sites()` does.

**Criteria are identities.** Two acceptance criteria with one `criterion_id` are
`duplicate_criterion`, as two steps with one `step_id` are `duplicate_step`.

## Command line

`python3 workflows/validate_procedure.py <procedure.json>` prints `order=<step,...>` and exits
0, prints `refused <code>: <detail>` and exits 1, or says why the file could not be read and
exits 2. It reads at most `hee3.bounds.max_procedure_bytes` (8 MiB; the largest schema-valid
procedure measures about 1.1 MB) before parsing anything. It runs nothing. A `just` recipe for
it is not here: the justfile is a generated publication output owned by `completion_standard.py`.

## Why the case count is not the evidence

However many cases pass, they say nothing about whether the rules are load-bearing.
`tools/check-workflow-sites` neuters each refusal site in `validate_procedure.py` in turn —
the sites enumerated from the file's own syntax tree, never from a list — and requires the
suite to go red.

That sweep earned its keep immediately. The first run read `sites=17 killed=16 survived=1`:
an explicit self-dependency check was unpinned, because neutering it let the case fall
through to Kahn's remainder, which raises the same `cycle` code. Two sites keeping one rule
meant neither was pinned. The self-dependency check was **removed** rather than given a
distinguishing message — a cycle of length one is a cycle, and one door needs no agreement.
It read `sites=16 killed=16 survived=0` until the shape, version-binding and criterion
refusals landed; it now reads `sites=22 killed=22 survived=0`.

The sweep counts any failing run as a kill. The rules added with those refusals were also
planted by hand, each counted killed only when its NAMED test failed (21 of 21). That harness
first reported two survivors whose failures were the previous plant's: the suite loads this
module through `workflows/__pycache__`, whose bytecode is validated by source mtime in whole
seconds and by size, so a plant and its restore inside one second can run stale code. Run any
in-place rewrite of this file with `PYTHONDONTWRITEBYTECODE=1`; the sweep's own runs were
repeated that way and still read 22 of 22.

The same shape is why `test_every_refusal_site_has_a_case` takes its denominator from the
validator's syntax tree and its numerator from what the cases actually asserted as they ran.
Counting refusal *names* would have read `13/13` while four of those names were raised from
two places each.

## Scope

Describes procedures. Starts no scheduler, grants no action, accepts no parent, admits no
module, and observes no running system.
