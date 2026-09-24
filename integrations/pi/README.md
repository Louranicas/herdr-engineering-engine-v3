# Pi host extension

The half of this integration that can be built honestly today, and a clear statement of the
half that cannot.

## What is blocked, and by what

`corpus/public-interfaces.json` → `pi_extension` says:

> Host registration signatures remain unselected until host compatibility is qualified.

So they are not modelled here. `host_binding` in the schema is a **sealed, empty object**: a
package that declares one is refused by name (`host_binding_unqualified`) rather than
validated against a shape nobody has confirmed. `register()` returns tool definitions a host
*could* be offered, with `registered_with_host: false` and the reason, because offering them
is the step that needs the signature.

The contract's *"Host fixtures"* proof obligation is **not met and not claimed**. No host is
started anywhere in this directory, and `tests/pi_extension.py` prints `host_fixtures_run=0`
in its own summary so the gap travels with the result rather than having to be remembered.

**What would unblock it:** a qualified host compatibility statement naming the registration
signature, its version and its lifetime. Until then, building against a guess would produce a
package that validates cleanly and cannot load.

## What is decided here

Everything the contract states *without* reference to a host signature.

**A package is admitted whole or refused by name.** `register()` checks the published
schema's structural rules itself -- object, required keys, no undeclared keys, each key's JSON
type, at least one tool -- and refuses a breach as `malformed_package` rather than raising
`KeyError`. A tool's `action_version` must be the version the control-v1 catalogue pins for
its `Request_<action>` (`hee3.action_versions`, read by the generator, never retyped);
anything else is `unknown_action_version`.

**A stale handler cannot return a result.** `Calls` is built from `register()`'s admitted
result, and every call is pinned at open to that admission: extension, generation, bridge
version and its tool's action version. `reload()` takes the next admitted result, advances
the generation and names the calls it revoked; a callback under the old generation is
refused. A reload's whole purpose is that the previous registration no longer speaks for the
extension.

**A reload revokes; it does not forget.** A revoked call keeps its admitted tuple and its
state (`running` or `effect_unknown`) in `Calls.revoked`. It is never settled by a later
handler -- a result for it is `stale_generation` -- and its id cannot be opened again. A
reload is not a reconciliation, so an unobserved effect is still an obligation after it.
Nothing here settles a revoked call: that needs the engine's own disposition.

**A parallel call cannot return another call's success.** A result is applied only when the
call id, the tool id *and* the generation all match what was recorded at open. A reconciler
keyed only on call id would accept `c1`'s result claiming `c2`'s tool; that case is
`test_a_result_for_the_wrong_tool_is_refused`, with two calls genuinely live.

**An action error is a failure, not a rendering.** `fail()` settles a call as `failed` with
a control-v1 error code, through the same identity check as `render()`; a code outside that
vocabulary is `unknown_error_code`.

**A cancellation request is acknowledged, not final.** `cancel_requested()` records the
acknowledgement on the same call and leaves it open as `cancellation_requested` (or
`effect_unknown`, which it never overwrites). The call settles on the engine's disposition:
a result, a failure, or `cancel()` -- the confirmed cancellation. An effectful action may
complete after the request, and an acknowledgement presented as final would hide that.

**A cancel race has one winner, from both sides.** A call reaches a terminal state once. A
result racing a cancellation cannot overwrite the cancellation, and a cancellation racing a
result cannot erase the result — asserted as two cases, not one, because a rule checked in
one direction is half-checked.

**Rendering failure loses no verdict.** A render over the byte bound is refused *without*
settling the call: it stays open, still its own, and can still be rendered. A bound that
settled the call as failed would turn a transport problem into a task outcome.

**An unobserved effect is not done.** `unknown_effect` marks a call `effect_unknown` and
leaves it open, so nothing reports it as complete and nothing retries it. It is reconciled,
never guessed at.

## Files

| File | What it is |
|---|---|
| `pi-extension-v1.schema.json` | The package contract. A build output. |
| `generate_extension_schema.py` | Authors it; `--check` compares exact bytes. |
| `reconcile.py` | Registration admission and call lifecycle. |
| `examples/hee3-tools.json` | A three-tool package at generation 3. |
| `../../tests/pi_extension.py` | 67 cases; `host_fixtures_run=0`. |
| `../../tools/check-pi-sites` | Neuters each refusal site and requires the test named for it to fail; `sites=19 killed=19 wrong_reason=0 survived=0`. |

A tool's `action` enum is read from `schemas/actions/control-v1.schema.json`, never retyped,
so a tool cannot expose an action the engine does not have.

## Scope

Describes an extension package and reconciles call lifecycle as values. Registers nothing
with a host, invokes nothing, grants no action, and admits no module.
