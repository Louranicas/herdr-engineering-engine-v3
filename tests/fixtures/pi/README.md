# Pi 0.85.1 wire development fixtures

These fixtures were authored from the installed emitter and type declarations,
separately from the initial Rust parser implementation. They are development
expectations, with **zero module-case credits**. Neither fixture generation nor
passing these fixtures qualifies a provider, process boundary or module.

All authored wire values are fictional. The manifest also binds a separately
identified, actual metadata stdout capture supplied by the parent. The fixture
author read that capture and did not launch Pi, Node or a provider.

## Layout and use

- `manifest.json` records exact SHA-256 hashes and byte lengths, source locators,
  expected acceptance at a named layer, and the projected meaning.
- `records/` contains exact UTF-8 JSONL bytes, except deliberate malformed `.wire`
  fixtures. Requests use `outbound_profile`; do not feed them into the stdout
  response parser.
- `traces/` contains inbound bytes only. Manifest steps identify the outbound
  requests and where dependent sends must wait. Some traces are explicitly
  targeted lifecycle subsequences rather than complete captured transcripts.
- `boundary_recipes` in the manifest describe exact repeated raw bytes and their
  resulting hashes. Materialize ordered `ascii` or referenced fixture parts with
  the given `repeat` count. This avoids storing many megabytes of repeated text.
  Their expectations apply only to the named bound, not lifecycle validity.
- `chunking_cases` specifies split delivery without changing fixture bytes.

Run `python3 tests/fixtures/pi/build_fixtures.py` from the isolated T02 draft to
reproduce the authoring output. This reads the exact installed package and the
parent's `evidence/implementation/T02/sdk-smoke/metadata-benign.stdout`; changing
either changes provenance. Regeneration in a different checkout creates a new
location-bound provenance snapshot. Published fixtures preserve the exact
reviewed bytes instead of silently regenerating that snapshot.
The script does not execute Pi or Rust, and does not inspect parser code.

## Key expectations

`expected_accept` always names a limited validation layer. A structurally valid
record can still fail correlation, session attribution, recipe compatibility,
profile enablement or lifecycle order. No fixture expects task acceptance.

The exact `unknown` model sentinel comes from `pi-agent-core/dist/agent.js:14–24`
and appears in the parent's credential-free capture. It is unavailable metadata,
never a routable model. Reserved sentinel near-neighbors are refused. The optional
omitted-model fixture covers the declared type, and does not claim that this SDK
startup emits an omitted model. Model metadata may advertise `image`; this does
not admit image prompt inputs.

Prompt success acknowledges preflight. `agent_end`, including `willRetry:false`,
does not settle the session. Retry/backoff cannot renew the worker deadline.
`agent_settled` requires an owned run; the worker still needs idle readback and
separate process/descendant/workspace custody evidence. Unexpected compaction,
entry, tool or deferred branches are refused without settlement.

Actual `message_update` has only `type`, `usage` and `assistantMessageEvent`.
The emitted nested event omits `partial`; `done.message` and `error.error` remain.
Final text in `message_end` is authoritative over provisional deltas. Text,
signatures and incomplete argument deltas never grant control authority.

`clear_queue` first emits empty current queues, then returns removed prior
contents. Nonempty returned arrays do not mean the queues remain populated.
The worker waits for the correlated clear response, then abort response, then
idle state. Pi's async input dispatch does not serialize those dependencies.

The fictional usage example totals `20 + 10 + 3 + 6 = 39`. Reasoning `4` is
already inside output `10`; one-hour cache writes `2` are already inside cache
writes `6`. Neither is added again. Null context usage remains unknown. Session
stats require exact owned-session attribution and are provider-reported values,
without authoritative request-count, currency or invoice claims.

## Chosen worker bounds

The Pi worker profile explicitly selects these limits; they are not inherited
from HEE3-Control:

| Limit | Value |
| --- | ---: |
| Payload before LF | 1,048,576 bytes |
| Each raw stdout/stderr stream including LF | 8,388,608 bytes |
| JSON container depth, root counted as one | 32 |
| Records per stdout stream | 65,536 |
| Exact nonnegative integer counter | 9,007,199,254,740,991 |

The worker uses strict UTF-8 and LF framing, requires final LF, refuses CRLF,
duplicate keys, extra JSON values and unknown fields. This is stricter than Pi's
input helper, which trims CR and dispatches a non-LF EOF fragment. Unicode U+2028
and U+2029 stay inside strings and are not frame delimiters. Counter fixtures use
decimal integer tokens; equivalent exponent/decimal token policy is not decided
by these fixtures. Stderr is always diagnostic evidence, including forged PASS
text or response-shaped JSON.

Startup warning visibility, empty extension/resource/auth inventories, network
denial and import closure remain separate runtime controls. Their earlier scout
scenarios are retained as deferred work rather than fictional wire successes.
