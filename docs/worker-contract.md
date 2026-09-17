# Common worker contract — T03

The common contract in `src/worker/mod.rs` validates one coordinator-owned invocation for a selected adapter. `src/worker/inference.rs` preserves an already decoded output-only result. This is executable contract behavior; T06 supplies the first process launcher and T08 supplies the selected second backend.

## Lifecycle and ownership

The coordinator supplies task, attempt, generation and invocation identity, immutable recipe/workspace digests, selected adapter/model/effort, required features and a bounded prompt. `launch` returns one dispatch intent after checking every required feature. Unsupported features produce an explicit error before dispatch. Capability declarations grant no tool, credential or spending authority.

Every observation must carry the exact invocation and the next sequence number. Invalid observations or transport loss after dispatch permanently leave the contract uncertain, while preserving already observed output and history. The owner retains raw streams and supplies elapsed time from the original monotonic clock: 1,200 seconds total, with the final 300 seconds reserved for cleanup. The contract creates no retry loop or new deadline.

Final output remains an untrusted candidate. A terminal classification must agree with the candidate finish reason; completion and truncation require a candidate. Adapter completion supplies no verification, process custody, workspace settlement or engine acceptance. Pi ACK and `agent_end` cannot be mapped directly to common completion; the selected Pi terminal rule still applies.

Cancellation keeps intent, command issuance, acknowledgement and adapter-idle readback separate. Repeated cancellation sends no duplicate command. Unsupported cancellation retains intent. Idle without an issued cancellation is refused. A raced terminal result preserves cancellation history; external effects and descendant cleanup remain coordinator obligations.

## Identity and usage

Requested identity cannot satisfy a runtime identity requirement. Runtime readback records the selected adapter instance/model/effort; optional provider model or revision is an attributed self-report, not proof of an alias revision. Unknown provider identity stays unknown. Conflicting invocation or known identity observations are refused.

Usage retains exact provider bytes and explicit scope, form and provisional/final stage. Missing counters stay unknown. The contract neither sums tokens with provider-specific subset meanings nor converts cost to measured spending. Once the primary usage stream is final, later records must remain final with the same provider/scope/form and cannot erase a known numeric field. A present value can be corrected; T10 owns reconciliation. Required usage needs a final report with known scope and form, which alone does not establish a safe hard spending ceiling.

## Output-only and tool proposals

The output-only normalizer rejects tool proposals and preserves accepted text, finish reason, identity, usage and raw bytes unchanged. The common contract can carry opaque tool proposals only when the selected adapter declares that feature. It neither interprets nor executes a concrete tool vocabulary; later action authority remains separate. Current Pi selection remains the reviewed no-tools profile.

## Bounds and verification

Prompts are at most 256 KiB, candidate text and raw record at most 1 MiB each, identity/usage raw records at most 64 KiB each, accumulated deltas at most 8 MiB, and an invocation at most 65,536 observations. Names are nonempty, control-free and at most 256 bytes.

`tests/t03_contract.rs` contains 34 independently designed behavioral test families covering both adapter profiles, exact correlation, feature refusal before dispatch, event order, identity, final usage, cancellation races, output preservation and bounds. Repeated build profiles and mutation controls are validation, not additional module-owned case credits. `tools/check-quality` discovers this exact target and requires its complete summary. No module admission or hostile-code containment is established by these development checks.
