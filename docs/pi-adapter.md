# Pi adapter: initial supported subset

T02 implements the Rust wire decoder and one owned Pi session state machine in
`src/worker/pi.rs`. The supported source pair is Rust 1.98.0 and
`@earendil-works/pi-coding-agent` 0.85.1. Credential-free SDK metadata checks use
Node 24.21.0. These are development observations; worker admission, the protected
collector, hostile containment and deployed provider execution remain separate
original tasks.

## Boundary and ownership

`Framer` retains exact payload bytes and dispatches only complete LF records.
`Session` binds one HEE task, attempt, generation and vendor session to an
exclusively owned pipe. It serializes dependent commands and correlates both the
request ID and response command. IDs include task, attempt, generation and a
monotonic request sequence. Callers must own the process, obtain task/tool/
spending authority, preserve raw streams, enforce deadlines and cleanup, and call
`transport_failed` after a transport or custody failure. There is no production
process launcher or provider grant in this module.

The closed command subset is available models, set model, set thinking, state,
prompt, clear queue, abort and session stats. Prompt dispatch requires an empty
owned session and a final matching model/thinking readback. The exact upstream
unknown-model sentinel means unavailable metadata and cannot be selected.

## Event and accounting rules

- Prompt acknowledgement establishes preflight only. `agent_end` remains
  intermediate; `agent_settled` describes Pi's own continuation loop.
- The selected profile admits one turn per agent run, with no tools, queued work,
  extensions or compaction. Unreviewed records, optional vendor maps and content
  branches refuse compatibility.
- Completed `message_end` values override provisional deltas. `turn_end` and
  ordered `agent_end.messages` must repeat those exact completed messages.
- Announced retries require ordered attempt numbers, a stable maximum, and the
  corresponding continuation/end events. They consume the original clock.
- Cancellation orders clear queue, its response, abort, its response, and idle
  state readback. A started run also needs Pi settlement. Removed queue content
  in a clear response differs from prohibited nonempty current queue state.
- Usage remains provider-reported. `ProvisionalUsage` cannot be mistaken for
  `ReconciledUsage`, which requires Pi settlement, idle state, matching session
  identity and totals equal to all final assistant messages. Reasoning and
  one-hour cache writes are subsets, not extra tokens. Unknown context remains
  unknown in the projection; the raw response retains the complete distinction.

## Explicit bounds

Pi uses separate worker-selected limits: 1 MiB before LF per record, 8 MiB per
raw stream, 65,536 records and depth 32 counting the root container. Completed
messages retained for one run have a cumulative 1 MiB serialized-byte cap.
Prompts are nonempty and at most 256 KiB. Counters must be nonnegative decimal
JSON integer tokens no larger than 9,007,199,254,740,991; fractional, exponent
and negative-zero spellings are refused. Costs remain finite nonnegative
provider-reported numbers.

The monotonic clock cannot rewind. Work has one 1,200-second origin with its
last 300 seconds reserved for cleanup; a new prompt cannot start in that reserve.
The process owner applies these limits to actual I/O and descendant custody.

## Development checks

`tools/check-quality` runs the pinned offline Rust/Julia quality matrix on copied
inputs. `tests/t02_pi.rs` consumes independently authored source-backed wire
specimens and exercises lifecycle, correlation and accounting. Fixture inventory
counts are distinct from qualifying module case counts. Targeted fixture traces
are not described as complete captured executions.

`tools/pi-metadata/smoke.py` mounts the pinned Node/Pi trees read-only in a private
Bubblewrap namespace. It supplies empty credentials, model storage, resources,
settings and tools; denies network access; and issues exactly two metadata
commands. It checks warning visibility after SDK startup. This helper is a
development probe, not the engine worker entrypoint. It makes no provider call.

Historical source, source-only security review and actual run evidence live in
`evidence/implementation/T02/`. Original task completion requires the retained
checks and owner review; a document or isolated passing test cannot admit the
worker module.
