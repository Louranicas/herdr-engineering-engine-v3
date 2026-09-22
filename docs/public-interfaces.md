# Module public interfaces and codebase authority

Completed, accepted code modules and the assembled accepted codebase are the primary source of truth for implemented behavior and supported interfaces at their exact accepted revision. The atlas and active standards retain intended requirements and acceptance obligations. Current comment stubs establish locations and proposals only; scoped observations are not completion admission.

[Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) · [Ultra map master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Atlas master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Atlas%2FMASTER_INDEX_habitat_engine)

Convention: `HEE3-INTERFACES-001`, revision 1. All entries below are planned interfaces; none is an accepted callable implementation.

## Meaning of a publicly accessible interface

Every module must present a documented, discoverable, versioned interface to its named consumers. Public means supported consumer access: an internal Rust facade, Julia API, CLI, typed tool action, local Unix socket, stdio exchange or package schema. It does not require exposing private helpers or adding TCP/HTTP listeners. Use the minimum language visibility that makes the supported boundary callable. External Rust crate consumers require an accessible library target and re-export path; the current binary/comment stubs do not provide one. Julia exported/public names and docstrings describe supported use; exports do not isolate code. Exact symbols and host registration signatures must be chosen and tested during authorized implementation, then reflected from accepted source.

## Authority follows accepted implementation

Lifecycle: planned_stub → implementation_candidate → accepted_module. Packaging and deployment are separate observations; accepted does not imply installed. A candidate may become accepted only through an admitted completion record binding module ID, source/test/fixture/dependency/toolchain digests, public-interface and migration identities, applicable task and standard revisions, full ≥50-case/zero-warning qualification, assimilation/mutation evidence, integration/security dispositions and owner/reviewer admission. Source changes invalidate applicability until re-admitted. The assembled codebase requires its own integrated release acceptance; a sum of module results or child successes cannot substitute. No admission collector is implemented in this documentation tooling; accepted_interfaces is empty and must remain empty until a separately verified admission path exists.

## Resolve divergence without rewriting facts

For accepted code, publish current interface facts from its exact source/schema/migration subject and keep desired changes alongside them. A defect remains a defect even if code exhibits it. Requirements, security constraints and testing floors are not weakened to match a failing implementation. Record current-versus-desired drift, affected callers/callees, change owner, repair or explicitly authorized requirement revision, and re-verification. Never generate runtime code from a stale atlas, infer acceptance from filesystem presence, or silently edit a released migration. Historical source captures and receipts retain their original bytes and scope.

## Required contract fields for every supported entry point

Before acceptance record: stable interface and operation ID; owner and intended audience; source symbol/schema path; signature and typed input/output schema with examples; errors and unknown-effect semantics; capabilities and credential custody; maximum sizes, queue/backpressure and deadline; idempotency key/replay/readback; cancellation and settlement; concurrency and workspace/resource ownership; compatibility/deprecation range; evidence and diagnostic references; exact admitted revision. Reuse the existing numbered APIs, IPC and actions. The per-module operations below are semantic design obligations, not newly invented callable symbols. Their mapped API/action notes supply existing detailed protocol contracts. Any missing concrete field blocks interface acceptance.

## Discoverability, documentation and compatibility gates

Rust modules need rustdoc for the supported facade, examples and caller compile checks across admitted targets/features; where rustdoc or other applicable checks emit baseline warnings/errors, qualification fails. Julia needs supported entry-point docstrings, deliberate exports or version-appropriate public declarations, contract examples and qualified package loading. CLI/actions need help/inspect output, input/result/error schemas and literal-value examples that agree across CLI, UDS and tool projections. Bash, Pi, skills and workflows need versioned manifest/entry-point and compatibility documentation. Private implementation details stay private. Check both directions of each integration: caller assumptions and callee guarantees, returned evidence/error/usage and cancellation. A breaking change requires explicit version/migration and affected-consumer testing.

## Local transport and capability boundaries

Keep each existing IPC endpoint under its declared lifecycle owner and custody. Public documentation does not authorize a world-readable/writable Unix socket, worker access to the operator socket, or Internet service. Any future transport addition requires a separate accepted contract for bind identity, authentication/authorization, credentials, resource limits, failure/readback and deployment evidence. Current interface mappings remain proposals; no listener, RPC or service is installed by this corpus update.

## Update loop and proof of completion

Inspect exact candidate and accepted revision → compare requirements and interface contracts → implement only authorized changes → execute relevant checks → retain independently collected observations → repair/escalate → reverify → obtain completion admission → project current code facts into atlas/vault/ultra map and master indexes → publish with matching hashes and live link readback. Preserve current and desired views separately. Until the admission collector exists, corpus-sync can verify navigation, hashes and declared observation consistency only. Documentation completion must never be reported as module or release completion.

## Module interface directory

| Stable interface | Module stem | Code and full contract | Owned action IDs |
| --- | --- | --- | --- |
| HEE3-IF-contracts | [contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-contracts) | [Source](file:///var/home/herdr-engineering-engine-v3/src/contracts.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/contracts.md) | internal/package consumer surface |
| HEE3-IF-task | [task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task) | [Source](file:///var/home/herdr-engineering-engine-v3/src/task.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/task.md) | task.preview, task.submit, task.get, task.list, task.cancel, task.resolve |
| HEE3-IF-store | [store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store) | [Source](file:///var/home/herdr-engineering-engine-v3/src/store.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/store.md) | internal/package consumer surface |
| HEE3-IF-roster | [roster](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-roster) | [Source](file:///var/home/herdr-engineering-engine-v3/src/roster.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/roster.md) | roster.list, roster.inspect, roster.update, roster.disable |
| HEE3-IF-route | [route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-route) | [Source](file:///var/home/herdr-engineering-engine-v3/src/route.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/route.md) | internal/package consumer surface |
| HEE3-IF-budget | [budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-budget) | [Source](file:///var/home/herdr-engineering-engine-v3/src/budget.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/budget.md) | internal/package consumer surface |
| HEE3-IF-worker | [worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-worker) | [Source](file:///var/home/herdr-engineering-engine-v3/src/worker/mod.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/worker.md) | internal/package consumer surface |
| HEE3-IF-check | [check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check) | [Source](file:///var/home/herdr-engineering-engine-v3/src/check.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/check.md) | internal/package consumer surface |
| HEE3-IF-recovery | [recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-recovery) | [Source](file:///var/home/herdr-engineering-engine-v3/src/recovery.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/recovery.md) | internal/package consumer surface |
| HEE3-IF-cohort | [cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-cohort) | [Source](file:///var/home/herdr-engineering-engine-v3/src/cohort.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/cohort.md) | thread.get, thread.list |
| HEE3-IF-context | [context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-context) | [Source](file:///var/home/herdr-engineering-engine-v3/src/context.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/context.md) | internal/package consumer surface |
| HEE3-IF-notify | [notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-notify) | [Source](file:///var/home/herdr-engineering-engine-v3/src/notify.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/notify.md) | events.subscribe |
| HEE3-IF-service | [service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-service) | [Source](file:///var/home/herdr-engineering-engine-v3/src/service.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/service.md) | service.inspect, service.probe, service.action |
| HEE3-IF-herdr | [herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-herdr) | [Source](file:///var/home/herdr-engineering-engine-v3/src/herdr.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/herdr.md) | internal/package consumer surface |
| HEE3-IF-numerical | [numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-numerical) | [Source](file:///var/home/herdr-engineering-engine-v3/src/numerical.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/numerical.md) | analysis.request, analysis.get |
| HEE3-IF-julia | [julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-julia) | [Source](file:///var/home/herdr-engineering-engine-v3/julia/src/HabitatAnalysis.jl) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/julia.md) | internal/package consumer surface |
| HEE3-IF-app | [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app) | [Source](file:///var/home/herdr-engineering-engine-v3/src/main.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/app.md) | health |
| HEE3-IF-actions | [actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-actions) | [Source](file:///var/home/herdr-engineering-engine-v3/src/actions.rs) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/actions.md) | tools.list, tools.inspect |
| HEE3-IF-bash | [bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-bash) | [Source](file:///var/home/herdr-engineering-engine-v3/integrations/bash/README.stub.md) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/bash.md) | internal/package consumer surface |
| HEE3-IF-pi_extension | [pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-pi_extension) | [Source](file:///var/home/herdr-engineering-engine-v3/integrations/pi/README.stub.md) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/pi_extension.md) | internal/package consumer surface |
| HEE3-IF-skills | [skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-skills) | [Source](file:///var/home/herdr-engineering-engine-v3/skills/README.stub.md) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/skills.md) | internal/package consumer surface |
| HEE3-IF-workflows | [workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-workflows) | [Source](file:///var/home/herdr-engineering-engine-v3/workflows/README.stub.md) · [Contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/workflows.md) | internal/package consumer surface |

## HEE3-IF-contracts

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-contracts)

**HEE3-IF-contracts** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust types and validation facade shared by declared consumers; serializable boundary schemas through existing APIs.

**Declared build consumers:** task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, app, actions.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Validate an envelope | Schema version; task, attempt, generation, caller and correlation identities; bounded payload | Validated typed envelope or field-specific rejection |
| Represent outcomes | Known lifecycle observation, error category and evidence reference | Stable discriminated result preserving denied, failed, cancelled and unknown distinctions |
| Negotiate compatibility | Producer/consumer version and declared capabilities | Supported subset or explicit incompatibility |

**Errors and bounds:** Bound lengths, integer ranges and enum variants; reject missing identity and ambiguous encodings before effects. Unknown versions are not guessed.

**Effects, replay and concurrency:** Pure constructors/validation. No database, grants, process control or state acceptance. Changes propagate to all wire consumers through explicit compatibility review.

**Required proof:** Round-trip and malformed schema cases; old/new consumer fixtures; non-lossy error and identity projection; fuzz/property campaign on parsing bounds.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-task

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task)

**HEE3-IF-task** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust task coordinator facade; operator and tool access exclusively through admitted actions and app transports.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Preview and admit | Caller capability, task specification, acceptance revision, limits and idempotency key | Preview exclusions or durable task/attempt identity and initial state |
| Read and request cancellation | Task identity; bounded listing cursor or cancellation request | Committed snapshot/page or acknowledged cancellation obligation |
| Advance or resolve | Expected generation, observed worker/checker facts, explicit operator resolution authority | Committed transition with evidence links or conflict/denial |

**Errors and bounds:** Bound task/attempt counts and deadlines. Refuse stale generations, incompatible retry bodies and missing grants. Cancellation acknowledgement is not process settlement.

**Effects, replay and concurrency:** Owns the plan/execute/observe/verify/repair/close loop. Mutations commit through store with conservative budget and retained uncertainty; checker evidence alone does not grant acceptance.

**Required proof:** Boundary acceptance, duplicate submission, lost reply/readback, stale result, verify-repair-verify loop, cancellation and exhausted-budget cases.

**Existing API references:** [API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10).

**Owned public actions:** [task.preview](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.preview), [task.submit](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.submit), [task.get](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.get), [task.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.list), [task.cancel](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.cancel), [task.resolve](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.resolve).

**Existing IPC associations:** [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01), [IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-store

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store)

**HEE3-IF-store** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust ledger transaction facade; SQL files form a versioned schema boundary owned by store, with no direct consumer SQL writes.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Commit ledger transition | Expected state/generation, task/attempt facts, reservation deltas, evidence references and outbox events | Atomic commit identity and sequence or conflict with no partial effect |
| Read consistent state | Bounded identifiers, cursor, snapshot requirements | Typed rows/events with ledger epoch and pagination |
| Apply or inspect schema history | Exclusive writer custody, ordered migration identity/checksum and supported version range | Transactional schema transition or refusal; independently read-back version and integrity |

**Errors and bounds:** Bound transactions and busy retry windows. Surface disk full, corruption, constraint failure and unsupported schema explicitly; never turn a failed write into success.

**Effects, replay and concurrency:** Single logical ledger writer; transactions maintain cross-table invariants and durable outbox. Released migration bytes are immutable. Replay uses durable identities.

**Required proof:** Atomicity and crash recovery; foreign-key/uniqueness constraints; concurrent starters; fresh install/upgrade/restore; migration checksum mismatch and schema readback.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-roster

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-roster)

**HEE3-IF-roster** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust roster queries and controlled updates; roster actions are the public projection for model, agent and service records.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Query capability and identity | Kind, constraints, locality, version, bounded page and observation cutoff | Declared/observed records with freshness, provenance and explicit unknowns |
| Update or disable record | Caller grant, stable record ID, expected revision and validated change | New roster revision or conflict/denial |
| Reconcile service health | Registered owner identity and approved probe observation | Dated useful-health observation without acquiring lifecycle authority |

**Errors and bounds:** Bound records and cache age; duplicate IDs, unsupported capability claims and stale observations remain visible.

**Effects, replay and concurrency:** Updates preserve revisions. Discovery cannot grant control, manufacture health or imply that a model is installed or eligible.

**Required proof:** Fresh/stale/unknown separation; capability filtering; concurrent update conflicts; disabled record exclusion; spoofed service/model identity refusal.

**Existing API references:** [API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10).

**Owned public actions:** [roster.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-roster.list), [roster.inspect](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-roster.inspect), [roster.update](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-roster.update), [roster.disable](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-roster.disable).

**Existing IPC associations:** [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01), [IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-route

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-route)

**HEE3-IF-route** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Pure Rust eligibility/ranking facade used by task; no independent routing daemon or new public endpoint.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Filter eligible recipes | Task class, constraints, roster snapshot, budget and admitted policy revision | Eligible recipes and explicit exclusions |
| Rank and explain | Eligible set, dated measured observations and deterministic tie policy | Chosen model/effort/adapter recipe, reasons, uncertainty and fallback order |
| Evaluate fallback | Failure category, remaining resources and previous attempt identity | Permitted next recipe or truthful refusal |

**Errors and bounds:** No eligible candidate is a normal refusal. Missing measurements stay unknown. Bound search and require deterministic replay inputs.

**Effects, replay and concurrency:** No dispatch or budget mutation. Julia recommendations are candidates until admitted; baseline survives invalid or stale challengers.

**Required proof:** Eligibility invariants; dominated/unsupported model refusal; stable tie-breaks; stale evaluation, constrained fallback and cost/failure-inclusive comparison.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-budget

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-budget)

**HEE3-IF-budget** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust reservation and accounting facade integrated with task/store; typed usage reports from workers/checkers/context/cohorts.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Reserve resources | Task/attempt/parent allocation, unit-labelled upper bounds and expected revision | Admitted reservation identity or refusal |
| Settle or reconcile usage | Reservation ID, observed usage provenance, uncertainty and worker settlement | Conservative balance, discrepancy and unresolved liability |
| Read remaining allocation | Task/thread scope and observation revision | Bounded available, reserved, spent and unknown totals |

**Errors and bounds:** Reject negative/nonfinite/incompatible units and overflow. Unknown usage is not zero; insufficient budget is explicit.

**Effects, replay and concurrency:** Atomic conservation across concurrent parent/child requests; release only with required settlement proof. Retried reports are idempotent by identity.

**Required proof:** Conservation properties; concurrency; retries and delayed usage; compaction/checker costs; abandoned work and uncertain cancellation liability.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-worker

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-worker)

**HEE3-IF-worker** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust adapter facade over bounded Pi/native/inference protocols; no access to the operator control socket by default.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Start an admitted attempt | Attempt/generation, exact recipe, scoped workspace, limits, credentials and cancellation channel | Start acknowledgement, actual runtime/model identity and capability limits |
| Observe activity and result | Framed ordered adapter messages tied to the admitted attempt | Activity, usage, candidate artifact references and classified exit |
| Cancel and settle | Attempt/process identity, deadline and cancellation reason | Cancellation acknowledgement, process/workspace settlement proof or explicit unknown |

**Errors and bounds:** Frame/stream/size/time bounds and schema negotiation; malformed, duplicated, stale or spoofed messages cannot close a task.

**Effects, replay and concurrency:** One adapter owns its process lifecycle under coordinator custody. Candidate output is untrusted and never checker evidence. Lost replies require identity readback.

**Required proof:** Pi/native protocol fixtures; launch failures; partial frames; wrong actual model; cancellation races; residual writable process and hostile output isolation.

**Existing API references:** [API03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API03), [API04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API04), [API08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API08).

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** [IPC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC03), [IPC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC07). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-check

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check)

**HEE3-IF-check** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust verifier facade operating on protected fixtures and the exact candidate subject; evidence consumers are task/release admission.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Prepare protected verification | Acceptance revision, source/artifact subject, admitted fixture and toolchain identities | Bounded verification plan with independent oracle and collector custody |
| Collect criterion observations | Exact execution outputs, profiles, case identities and diagnostics | Per-criterion pass/fail/invalid/unmeasured observations with raw references |
| Report qualification gaps | Observation set, module standard and integration obligations | Coverage/subject/diagnostic/assimilation/mutation gaps and candidate disposition |

**Errors and bounds:** Wrong-subject, zero-execution, skipped, stale and forged observations cannot pass. Baseline warnings/errors must be zero.

**Effects, replay and concurrency:** Checker cannot accept its own task or trust worker-authored receipts. Protected output capture and tests are isolated from candidate write authority.

**Required proof:** Forged result and fixture tamper detectors; wrong subject/profile; meaningful 50-case accounting; intended-fault plus benign mirror; full-standard collector qualification remains future work.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-recovery

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-recovery)

**HEE3-IF-recovery** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust reconciliation facade called by task/app on restart, timeout and uncertain effects.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Inventory outstanding obligations | Ledger epoch, active attempts, process/session identities and workspace claims | Reconciliation candidates with custody and uncertainty |
| Reconcile and clean up | Trusted process/readback observations and bounded cancellation policy | Settled obligations, pending cleanup or explicit ambiguity |
| Rebuild delivery/state view | Committed outbox/event identities and restore epoch | Replay plan without repeating accepted engine effects |

**Errors and bounds:** PID alone is insufficient identity. Missing process visibility, lease expiry and timeout cannot establish safe workspace reuse.

**Effects, replay and concurrency:** Acts through registered lifecycle owners; retains cleanup obligations until verified. Restoration changes epochs and rejects stale observations.

**Required proof:** Crash/restart, PID reuse, partial cleanup, restored epoch, late result, lost acknowledgement and retained reservation cases.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-cohort

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-cohort)

**HEE3-IF-cohort** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust orchestration facade for specialist threads, assignments, resource claims and parent joins; thread read actions project state.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Plan and assign bounded work | Parent acceptance, finite child DAG, roles, revisions, budgets and resource claims | Thread/assignment identities and explicit dependencies |
| Read thread and collect candidates | Thread/cursor and child observations tied to current assignment | Bounded state, evidence, dissent and unresolved gaps |
| Join or revise work | Required child criteria, context revisions, conflict and final-verifier disposition | Parent integration candidate, repair plan or blocked join |

**Errors and bounds:** Bound concurrency, fan-out, token/time budgets and retries. Detect cycles, stale briefs, duplicate ownership and write conflicts.

**Effects, replay and concurrency:** One coordinator owns assignment/join; child success never accepts parent. Shared allocation is conserved and workspaces follow explicit custody.

**Required proof:** Disjoint work synergy; stale brief; overlapping writes; missing child; contradictory evidence; dissent preservation; failed join repair and final integrated verification.

**Existing API references:** [API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10).

**Owned public actions:** [thread.get](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-thread.get), [thread.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-thread.list).

**Existing IPC associations:** [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01), [IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-context

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-context)

**HEE3-IF-context** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust bounded context packet facade; consumers are cohort, task and declared skill/worker integration paths.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Assemble context | Brief revision, task/role, permitted source IDs, dependency relationships and token/byte budget | Selected references, bounded content, omissions, source and relationship gaps |
| Refresh or compare packet | Old/new source revisions, permitted scope and stable context identity | Change/provenance report with affected consumers |
| Report context cost | Measured selection/compaction work and usage provenance | Usage fact for budget accounting |

**Errors and bounds:** Untrusted source instructions do not grant actions. Bound expansion, path access, recursion and packet size; failed fetches remain gaps.

**Effects, replay and concurrency:** Read scoped sources; preserve provenance and bidirectional dependency/return context. No silent expansion of access or evidence authority.

**Required proof:** Budget cutoffs; stale source; omitted dependency; prompt-injection boundary; denied path; reproducible packet ordering and cost inclusion.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-notify

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-notify)

**HEE3-IF-notify** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust committed-event subscription/delivery facade; events.subscribe is the declared consumer action.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Subscribe or replay | Caller visibility, bounded cursor, epoch and backlog limit | Ordered permitted event stream plus gap/resync indication |
| Deliver and acknowledge | Committed event ID, recipient identity and durable delivery key | Delivery receipt or pending retry with failure category |
| Read delivery obligation | Event/recipient identity and observation cutoff | Delivered/pending/unknown state without task-state invention |

**Errors and bounds:** Bound queues and replay windows; slow consumers receive backpressure or resumable gaps. Invalid/restored epoch requires resync.

**Effects, replay and concurrency:** Outbox begins after ledger commit. Delivery retry never reruns task effects; dedup uses durable event/recipient identity.

**Required proof:** Commit-before-delivery; lost ack, replay dedup, cursor expiry, restored epoch, slow subscriber and visibility filtering.

**Existing API references:** [API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API02).

**Owned public actions:** [events.subscribe](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-events.subscribe).

**Existing IPC associations:** [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-service

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-service)

**HEE3-IF-service** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust useful-health and delegated lifecycle facade, projected by service.inspect/probe/action.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Inspect or probe | Registered service ID, owner identity, requested useful check and explicit probe admission | Dated useful health, version and owner state |
| Delegate lifecycle action | Caller grant, service/unit/action, resource bounds and durable operation key | Owner job identity and accepted/refused/unknown effect |
| Read back lifecycle effect | Operation and owner/job identity | Independent observed result, pending state or explicit ambiguity |

**Errors and bounds:** Probe bounds include time/resource/network scope. Discovery does not authorize start/stop. Lost reply is not permission to repeat a destructive effect.

**Effects, replay and concurrency:** Lifecycle ownership remains with the registered binary/daemon/systemd/external owner; service is an adapter, not another supervisor.

**Required proof:** Unauthorized probe/action refusal; wrong unit/owner; lost response readback; apparently live but useless health; replay and conflicting action identity.

**Existing API references:** [API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10), [API07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API07).

**Owned public actions:** [service.inspect](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-service.inspect), [service.probe](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-service.probe), [service.action](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-service.action).

**Existing IPC associations:** [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01), [IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04), [IPC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC06). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-herdr

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-herdr)

**HEE3-IF-herdr** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Supported multiplexer/client integration surface calling existing actions/app interfaces; presentation remains a client.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Submit operator intent | Typed task context, explicit user constraints and caller authority | Admission/denial and durable task identity |
| Present task/thread state | Versioned snapshots, events and evidence references | Status, route explanation, gaps and navigable proof |
| Reconnect or request cancellation | Last cursor/epoch and task identity | Rebuilt view or acknowledged cancellation obligation |

**Errors and bounds:** Bound UI event buffers and reconnects. Pane title, process exit or rendered done text cannot establish task acceptance.

**Effects, replay and concurrency:** No second scheduler or authoritative task state in panes. Client loss leaves engine obligations durable.

**Required proof:** Pane loss and reconnect; duplicate UI submission; stale event; explicit cancellation; truthful failure/unknown presentation and proof navigation.

**Existing API references:** [API06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API06).

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** [IPC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC02). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-numerical

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-numerical)

**HEE3-IF-numerical** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust bounded Julia/numerical request facade and analysis.request/get projection; optional runtimes remain separately admitted.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Prepare numerical request | Immutable dataset identity/cutoff, units/schema, recipe/runtime, resource bounds and policy comparison | Validated job identity and input manifest or refusal |
| Exchange and collect | Bounded process/stdio/file protocol with pinned Julia or optional runtime | Report/artifact identity, measured cost, uncertainty and classified failure |
| Validate candidate policy | Report subject, held-out evidence, baseline and compatibility disposition | Accepted-for-review candidate or rejection preserving current baseline |

**Errors and bounds:** Bound memory, device use, files, framing, deadlines and cancellation. Nonfinite values, unit/schema mismatch and stale datasets are explicit failures.

**Effects, replay and concurrency:** No accepted task-state, grants or lease writes from Julia. PyTorch/tch/LibTorch or neural-operator lanes require their existing admission tasks; no automatic installation.

**Required proof:** Cross-language schema/units; crash/cancel/oversize output; immutable subject; incompatible runtime/device; invalid candidate and baseline retention.

**Existing API references:** [API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10), [API05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API05), [API09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API09).

**Owned public actions:** [analysis.request](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-analysis.request), [analysis.get](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-analysis.get).

**Existing IPC associations:** [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01), [IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04), [IPC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC05), [IPC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC07). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-julia

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-julia)

**HEE3-IF-julia** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Julia HabitatAnalysis module with documented supported entry points and intentional exports/public API; optional operator APIs gated by evidence.

**Declared build consumers:** none; use the explicit integration surface.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Evaluate routing outcomes | Immutable dataset with failures/cancellations/unknown usage, baselines and held-out split | Reproducible comparison with uncertainty, cost and dataset identity |
| Analyse cohesion | Thread/assignment outcomes, context revisions and conserved resource accounting | Cohesion/rework/join metrics with exclusions and limitations |
| Evaluate optional operator | Declared domain, units, model/runtime identity and out-of-domain policy | Validated operator report or explicit rejection/fallback |

**Errors and bounds:** Validate shapes, units, finite domains and resource limits. Record seeds, versions, device determinism and excluded/missing observations.

**Effects, replay and concurrency:** Produces analysis artifacts/candidate policy only. No grants, leases, task acceptance or control-socket access. Export is a usability contract, not a security boundary.

**Required proof:** Held-out comparison; complete denominator including failure; numerical tolerances justified by domain; wrong shape/units; out-of-domain fallback and runtime reproducibility.

**Existing API references:** [API05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API05), [API09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API09).

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** [IPC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC05), [IPC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC07). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-app

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app)

**HEE3-IF-app** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Executable CLI and local serve composition; exposes the existing action catalogue through declared transports.

**Declared build consumers:** none; use the explicit integration surface.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Parse and dispatch | CLI argv or bounded local request, transport/caller identity and action version | Typed action result, stable error category and decisive exit/status |
| Compose and supervise coordinator | Validated configuration, ledger location and registered dependencies | Single initialized coordinator or explicit startup refusal |
| Report health and shut down | Bounded health request or shutdown signal | Useful service/readiness status and settled/pending shutdown obligations |

**Errors and bounds:** No implicit network exposure. Refuse unsupported config/schema and insecure endpoint custody; bound concurrency, payloads and shutdown.

**Effects, replay and concurrency:** Dependency injection and lifecycle composition only; no duplicated task/roster/budget state machine. CLI, UDS and tools share action semantics.

**Required proof:** CLI/transport semantic parity; startup races; bad config; unavailable dependencies; broken pipe/status; endpoint custody; shutdown and integrated failure/security matrix.

**Existing API references:** [API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API02).

**Owned public actions:** [health](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-health).

**Existing IPC associations:** [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-actions

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-actions)

**HEE3-IF-actions** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Versioned authoritative action catalogue, inspectable schemas and typed dispatch metadata for CLI/Unix/tool projections.

**Declared build consumers:** app, bash, pi_extension, skills, workflows.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| List visible capabilities | Caller identity, optional bounded filter/page and catalogue revision | Visible action IDs/versions/purpose and pagination |
| Inspect a supported action | Visible stable action ID and requested version | Input/result/error schema, authority, effect, bounds and retry/readback contract |
| Validate dispatch metadata | Caller-bound action request and transport context | Typed dispatch to the existing owner or explicit denial/incompatibility |

**Errors and bounds:** Reject unknown versions, ambiguous fields and ungranted effects before dispatch. Catalogue visibility is recalculated for caller.

**Effects, replay and concurrency:** One action definition projects to all transports; wrapper/tool names cannot elevate authority. Does not create another task engine.

**Required proof:** Complete 21-action owner projection; schema drift; unknown/hidden action; denied effect; CLI/tool/UDS parity and literal input preservation.

**Existing API references:** [API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10).

**Owned public actions:** [tools.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-tools.list), [tools.inspect](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-tools.inspect).

**Existing IPC associations:** [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01), [IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-bash

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-bash)

**HEE3-IF-bash** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Documented thin Bash/Just command entry points with argv/JSON contracts over existing actions.

**Declared build consumers:** none; use the explicit integration surface.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Invoke action safely | Literal positional/array arguments, bounded JSON and explicit task context | Structured result plus unambiguous producer exit verdict |
| Chain bounded commands | Declared input/output contracts, cancellation and timeout propagation | Ordered results and preserved failing producer status |
| Inspect usage and prerequisites | Wrapper/version and existing action metadata | Supported options and dependency/version diagnostics |

**Errors and bounds:** No eval or reparsing untrusted strings. Bound retries, output and environment inheritance; missing producer is failure.

**Effects, replay and concurrency:** No scheduler/policy duplication; wrappers do not grant actions. Pipe/tee/display success must not swallow producer failure.

**Required proof:** Whitespace/metacharacters; empty input; stderr/stdout separation; failing producer through pipeline; cancellation and dependency/version mismatch.

**Existing API references:** [API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01).

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-pi_extension

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-pi_extension)

**HEE3-IF-pi_extension** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Versioned Pi-host extension package contract for registration, invocation, rendering and cancellation.

**Declared build consumers:** none; use the explicit integration surface.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Register supported tools | Pinned host/bridge version and action catalogue capability projection | Compatible tool definitions or visible refusal |
| Invoke and render | Host call identity, typed arguments, task context and bounded action result | Truthful result/error/usage/evidence presentation |
| Cancel and reconcile lifecycle | Host call/task identity, extension generation and cancellation signal | Acknowledgement and final/pending disposition tied to the same call |

**Errors and bounds:** Host registration signatures remain unselected until host compatibility is qualified. Reject stale handlers, unknown versions and mismatched call identities.

**Effects, replay and concurrency:** Pi host glue is a replaceable integration boundary; no second coordinator or implicit privilege. Parallel calls retain separate identities and limits.

**Required proof:** Host fixtures; reload/stale callback; parallel calls; cancel races; rendering failure without verdict loss; schema/version and permission parity.

**Existing API references:** [API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10).

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** [IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04). Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-skills

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-skills)

**HEE3-IF-skills** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Versioned skill-package manifest, entry instructions and bounded reference contract consumed by context and supported hosts.

**Declared build consumers:** none; use the explicit integration surface.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Discover compatible skill | Stable skill/version, supported action versions and declared references | Manifest/purpose/compatibility or rejection |
| Load scoped instructions | Role/task scope, permitted references and context budget | Bounded instruction packet with provenance, omissions and cost |
| Update a skill revision | Reviewed content/dependency change and compatibility disposition | New immutable version and affected-consumer/drift record |

**Errors and bounds:** Bound reference traversal and reject unsafe paths, missing required dependencies and incompatible actions. Text is not execution approval.

**Effects, replay and concurrency:** Skills provide instructions, not grants, acceptance or hidden daemons. A loaded skill cannot rewrite task criteria or broaden authority.

**Required proof:** Conflicting/untrusted instruction; missing/stale reference; version pinning; context cutoff; denied scope and revoked/retired skill behavior.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.


## HEE3-IF-workflows

[Complete module contract, dependencies and return anchors](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-workflows)

**HEE3-IF-workflows** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Versioned finite workflow/procedure schema using existing actions, task and cohort scheduling.

**Declared build consumers:** none; use the explicit integration surface.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Validate procedure | Typed finite steps, dependencies, action versions, acceptance and bounded retry policy | Validated DAG/obligations or cycle/incompatibility refusal |
| Start or resume composition | Admitted parent, stable step/child identities, resources and current evidence | Existing scheduler assignments and resumable step state |
| Join and close | Required child outcomes, unresolved effects and final integrated verification | Parent completion candidate or repair/block disposition |

**Errors and bounds:** Bound fan-out, retries, time and total resources; dynamic text cannot insert unapproved actions. Refuse stale versions or missing steps.

**Effects, replay and concurrency:** No alternative scheduler. Resume reads committed step identities; never blindly repeats uncertain service/task effects. Final parent acceptance remains independent.

**Required proof:** Cycle and incompatible step; partial resume; uncertain side effect; missing child; resource exhaustion; failed final verifier and repaired end-to-end join.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `comment_stub`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `c8a96767e52cf8b309ef5fc5ab968a1e875b0d97ab7d62aaec6f96f1d8436557`. This binding proves which documentation convention is projected, not runtime correctness.

## Interface evidence base

- [Rust Reference: visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html): Visibility must include an accessible ancestor or re-export path; choose consumer-scoped visibility.
- [The rustdoc book](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html): Document supported Rust APIs from source and inspect the resulting crate documentation.
- [Julia manual: modules](https://docs.julialang.org/en/v1/manual/modules/): Document supported module names, exports and public interface separately from access-control boundaries.

Reviewed on 2026-09-15. These sources support interface documentation conventions; they do not establish implementation, toolchain compatibility or a measured top-7% performance claim.

## Adopted supplementary requirements

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). These active clauses strengthen the existing owner; none relaxes its baseline or claims implementation.

### F3 · Public interfaces and integration

**Recorded score:** 87/100. **Conditional target:** 94/100; no promotion from documentation adoption.

**Primary owners:** [contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-contracts), [task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task), [worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-worker), [actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-actions), [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app), [numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-numerical), [julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-julia), [herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-herdr), [bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-bash), [pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-pi_extension), [skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-skills), [workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-workflows).

**Owning task contracts:** [T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01), [T02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T02), [T03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T03), [T21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21), [T28](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T28), [T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29).

**Governing standard/procedure routes:** [Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts). These are generated views of the same adopted clauses; edit the convention once.

- **F3-C01:** Resolve the existing 66 semantic operations into concrete versioned request/result/error schemas and Rust/Julia signatures as their owning slices are implemented. Record which interfaces are supported externally and which are internal module calls.
- **F3-C02:** For every supported boundary specify caller capability, authenticated identity, operation/request key, correlation ID, attempt generation, canonical request digest, deadlines, cancellation, bounded sizes, backpressure and retry/readback behavior.
- **F3-C03:** Complete each Unix-socket/pipe custody record: owner, runtime path, creation/bind readiness, access policy, peer identity limits, framing and maximum length, disconnect/reconnect, stale-socket detection and shutdown cleanup. Refuse unlinking a socket without proven custody.
- **F3-C04:** Use the one action definition for CLI, Unix API and LLM-tool projections. Commands, Bash, Pi extensions, skills and workflows preserve literal values and effect authority. A model-supplied approval or risk field never grants permission.
- **F3-C05:** Create a shared versioned compatibility fixture corpus with independent expected outcomes: valid round trips, malformed/oversized/truncated frames, missing/unknown fields, incompatible versions, nonfinite numerical data, cancellation, duplicate requests and lost acknowledgements.
- **F3-C06:** For Rust–Julia exchange specify numeric types, units, shape/layout, tolerances, missingness, dataset cutoff and report provenance. A Julia timeout or stale report leaves the approved operational policy unchanged.

**Required future proof:**

- Two real consumer projections produce the same typed decision, identity and error for each shared fixture; denial remains denial across CLI/API/tool surfaces.
- The Rust–Julia exchange passes independently defined expected results and failure controls; protocol changes invalidate affected compatibility evidence.
- All supported initial-release operations have concrete documentation/examples and tested compatibility. Planned optional operations stay visibly unavailable.

**Reassessment:** 90–91 follows a real cross-transport slice; 94 needs the supported release surface and Rust–Julia boundary qualified against the shared fixture corpus.

**Complexity guard:** Do not add endpoints to improve an API count. Reuse the ten API crossings, seven IPC mappings and 21 command/tool entries unless a reviewed requirement changes.

