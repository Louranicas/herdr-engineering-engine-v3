# HEE3-MOD-task · task

Task/attempt and bounded verification-loop state.

Primary path: `src/task.rs`. Cluster: K1. This is the full planned responsibility; a reference exemplar exercises only its stated subset.

## Anchors and source ownership

[Source comment anchor](file:///var/home/herdr-engineering-engine-v3/src/task.rs) · [Atlas module card](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-task) · [Scoped reference example](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-task) · [Ultra map master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md)

## Dependencies and integration

Declared build dependencies: contracts. Declared consumers: app. Runtime request/return paths below do not imply cyclic build imports.

## Adopted readiness obligations

Own task intent, generation and acceptance transitions. Specify the T01 release charter and rejection of stale or cancelled candidates; keep task, effect, cleanup and evidence dimensions separate.

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). Binding: `HEE3-READINESS-001` / `e774f8984c7e39e85b156289fac809bded6df80ba3647cfb2024c69856447b94`. Adoption is complete; the required engine proof is pending.

| Applicable facet / criterion IDs | Primary contract owners |
| --- | --- |
| [F1 · Vision, requirements and scope](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF1): F1-C01, F1-C02, F1-C03, F1-C04, F1-C05 | task, route, budget, numerical, julia, app |
| [F2 · Architecture, modularity and maintainability](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2): F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [F3 · Public interfaces and integration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3): F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06 | contracts, task, worker, actions, app, numerical, julia, herdr, bash, pi_extension, skills, workflows |
| [F4 · Testing, verification and completion evidence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4): F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07 | check, contracts, app, task, julia |
| [F5 · Security and hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5): F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06 | worker, check, service, actions, app, store, context, bash, pi_extension, skills, workflows |
| [F7 · Traceability, cohesion and controlled change](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7): F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |

Implementation groupings: [R90-01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-01), [R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03), [R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05), [R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09), [R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10). Original task prerequisites and all 13 completion gates remain controlling; package membership grants no action authority. Shared quality/security/interface rules apply to this module’s actual scope, without creating new runtime responsibilities.

**Resolved design contracts:** [RC01 · First workload, numeric limits and evidence policy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01), [RC02 · Pinned release profile and deployment custody](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02), [RC03 · Versioned control and Rust–Julia contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03), [RC04 · Protected collection, receipts and independent oracles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04), [RC05 · Trusted and adversarial execution profiles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05), [RC06 · SQLite, migration freeze and recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06). Apply the exact selected profile and pending-proof obligations.

Before a change, predict the affected owners and consumer contracts. Afterwards compare actual changes, rerun invalidated checks and retain counter-evidence. Minimum 50 distinct primary-owned cases, zero baseline diagnostics, trustworthy collection and independent parent/release acceptance remain mandatory.

## Public consumer interface contract

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

## Fully-complete contract

**HEE3-DONE-task** · unassessed; acceptance collector unavailable.

[Concrete module outcome, failure controls and admission checklist](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-task) · [Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)

Completion standard SHA-256: `a36374a796816c1ea4c85c3292d376942fe9da226d714012b87da2588f598594`. All 13 gates apply; only an inapplicable subcheck may be explicitly justified.

**Required finished behavior:** One durable task lifecycle performs admit → execute → verify → repair/escalate → reverify → accept, with correct terminal and pending states.

**Integrated proof scenario:** Submit once, lose the reply, read back the same task, repair a failed candidate and accept only the verified subject.

**Fault and benign controls:** Late result, cancellation/acceptance race and exhausted budget never create duplicate effects or false acceptance; valid current result can advance.

**Complexity boundary:** Task state is not duplicated in adapters, panes or workflow packages.

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Daybreak defensive security review

Requested model: `gpt-daybreak-blue-latest`; select through the Codex `/model` picker and verify effective identity before the scoped review. Model selection and a reviewer message cannot accept this module.

**HEE3-SEC-task:** Unauthorized state transitions, forged/stale acceptance, replay, cancellation/acceptance races and effect custody.

[Model selection, evidence and full security matrix](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB05 security hardening and re-verification](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)

Runtime security qualification is unassessed. Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Applicable patterns, antipatterns and learning triggers

[Reviewed diary learnings, evidence limits and retirement rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Assimilation and verification workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FCorpus%20Assimilation)

| Learning | Use when | Good pattern |
| --- | --- | --- |
| [LRN01 · Earn the verdict from the complete subject](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01) | Before quoting a result, admitting a module, closing a parent or presenting done in a client. | Report a typed scoped verdict with exact candidate, admitted criteria, required checks, raw evidence, gaps and a counter-evidence pointer. |
| [LRN04 · One semantic owner across every consumer surface](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04) | When adding a CLI, tool, Pi, Bash, workflow or cross-language entry point. | Use one typed owner and thin projections; pass source, environment, task and caller identities explicitly and preserve the return contract. |
| [LRN08 · Keep meaningful coverage and the zero-warning baseline](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08) | When designing module tests, fixing lint diagnostics or reporting a hardening result. | Retain at least 50 distinct meaningful module-owned cases and zero baseline warnings/errors on every admitted profile; repair code and close material gaps. |
| [LRN11 · Make specialist threads cohere through explicit ownership](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11) | At delegation, handoff, shared-resource access and every meaningful task/phase boundary. | Give each thread one bounded objective, subject/brief revision, resource/write ownership, dependencies, limits, return schema and cleanup obligations. |

These are scoped design and review obligations. Their engine detectors remain unqualified; source reflections and navigation counts are not implementation evidence.

## Mandatory module testing standard

[Active testing standard and counting rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing): **minimum 50 distinct qualifying cases; zero baseline warnings and errors**, including pedantic Clippy for all admitted Rust targets/profiles.

Bind assimilation, mutation sensitivity, lint coverage and raw evidence to the exact candidate. Property iterations, assertions, retries, lint findings and mutant runs do not multiply case credits. The current observation schema does not establish this qualification.

Standard: `HEE3-TEST-STD-001`; SHA-256: `14a29dad1d089bc529ff9a132e5f80d5195e81aba2c2c0a07229e1c7c8960b09`. Full-standard qualification: **unassessed; collector/validator not implemented**.

Module priorities: state transitions and terminal invariants; duplicate/replayed commands; cancellation and deadlines; parent/child acceptance separation.

## Bidirectional contracts

| Flow | Request | Return | Closure |
| --- | --- | --- | --- |
| [F01 herdr → task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F01) | Versioned intent and operator constraints enter actions/app; authenticated task-scoped request reaches task | Status, route reason, blocking criterion and completion references | Pane loss does not lose the task; a displayed done message cannot accept it. |
| [F02 task → store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F02) | Expected state/generation plus atomic attempt/event changes | Committed identity or typed conflict/storage failure | Crash after commit preserves replayable obligation; same key/different body conflicts. |
| [F03 task → route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F03) | Task class, immutable constraints and candidate observations | Eligible recipe, exclusions, evidence freshness and fallback | Ineligible model never dispatches; reverse explanation names every decisive exclusion. |
| [F04 task → roster](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F04) | Capability/locality/version/health query whose returned facts are passed to pure route policy | Declared and observed facts with identity and age | Missing or stale facts retain unknown status; source refresh does not rewrite history. |
| [F05 task → budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F05) | Attempt/verification reservation and later measured usage | Admit/refuse, remaining budget and reconciliation discrepancy | Concurrent reservations and unknown usage preserve the declared cost mode. |
| [F06 task → worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F06) | Attempt/generation, recipe, owned workspace, limits and cancellation | Ack/activity, usage, candidate references, exit/error and settled state | Every forward dispatch has a reconciled terminal or explicit unknown return; no stale generation acceptance. |
| [F07 task → check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F07) | Acceptance revision, immutable candidate and protected fixture references | Criterion observations, checker errors and raw evidence references | Wrong subject, changed fixture, skipped mandatory test or forged pass cannot close. |
| [F08 task → cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F08) | Parent criteria, child DAG, thread briefs and resource/budget bounds | Child candidates, dissent, integration gaps and join readiness | Successful children cannot bypass the integrated parent check. |
| [F10 task → notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F10) | Committed outcome event and relevant recipients | Delivery receipt or pending/retryable notification state | Lost notification retries delivery rather than re-executing accepted work. |
| [F11 task → recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F11) | Persisted active attempts and observed process/session identities | Reconciled state, cleanup result or quarantined ambiguity | Expired lease alone cannot release a live writable workspace. |
| [F13 task → numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F13) | Immutable outcome dataset at declared cutoff, including failed/cancelled/abandoned attempts and unknown usage, plus requested comparison | Validated quantitative report and candidate-policy disposition | A failed or stale analysis leaves the approved baseline unchanged. |
| [F15 actions → task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F15) | Schema-validated, caller-bound task request and idempotency context | Admission/state/evidence or typed denial/conflict | CLI, Unix API and LLM-tool projections preserve the same authority and effect semantics. |
| [F19 workflows → task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F19) | Versioned finite action composition and expanded child identity, mediated by actions/app with current authority | Step results, outstanding obligations, repair/stop and parent evidence | Every required child and final verifier contributes to the one parent closure path. |

## APIs, sockets and commands

[API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10)

| IPC | Endpoint | Custody and recovery |
| --- | --- | --- |
| [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01) | $XDG_RUNTIME_DIR/habitat-engine/control.sock | Acquire single-instance custody before migration/recovery; bind after ready. Never unlink an unproven live socket. Absent/incompatible -> typed failure; reconnect with engine cursor or snapshot+cursor on expiry |
| [IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04) | Separate inherited task-bound tool pipe pair OR selected maintained native tool channel | One admitted attempt/version tuple; old reload handler/channel generation revoked before effects Lost reply -> scoped action readback; denied action stays denied; unsupported bridge blocks tool exposure |

| Action | CLI projection | Effect | Return/readback |
| --- | --- | --- | --- |
| [task.preview](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.preview) | habitat-engine task preview --file <spec.json> --json | read-only planning | Eligible recipes, exclusions and known cost mode; no reservation Refresh stale facts; preview never commits permission |
| [task.submit](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.submit) | habitat-engine task submit --file <spec.json> --json | durable admission | Task ID, admission state and engine cursor Same key/same canonical body returns same task; different body conflicts; task.get by key |
| [task.get](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.get) | habitat-engine task inspect <id> --json OR task inspect --request-key <key> --json | read | Outcome + attempt/effect/cleanup/delivery states, criteria and manifest references Key lookup includes authenticated principal and action; no cross-principal oracle |
| [task.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.list) | habitat-engine task list --json | read | Stable snapshot page or explicit expired continuation No unbounded transcript/evidence output |
| [task.cancel](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.cancel) | habitat-engine task cancel <id> --json | cancel intent | Requested/observed cancel, cleanup/effect state; not immediate terminal success task.get; terminal compare-and-set resolves cancel/accept race |
| [task.resolve](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.resolve) | habitat-engine task resolve <id> --file <disposition.json> --json | record disposition | Disposition record and next permitted state; never forged evidence task.get; stale obligation conflicts |


## Full deployment stems

| Path | Owning module | Scope | Purpose |
| --- | --- | --- | --- |
| docs/contract.md | task | original planned path | First-task admission, bounded loop and completion contract; shared boundary and release conditions remain atlas-owned. |
| src/task.rs | task | original planned path | Owns task/attempt state and the execute–verify–repair policy without another scheduler. |
| Cargo.toml | app | original planned path | Shared Rust package composition, dependency and feature policy for the existing Rust module set. |
| rust-toolchain.toml | app | original planned path | Pinned compiler and tooling tuple for the existing Rust modules; no additional package boundary. |
| docs/checks.md | app | original planned path | Shared quality contract spans the planned stack; each module retains its own behavioral checks and disclosed limits. |
| docs/security.md | app | original planned path | Integrated threat model records existing authority, data, execution and verification boundaries across the planned stack. |
| migrations/001.sql | store | original planned path | Initial ledger schema joins task/attempt/event state, conservative reservations, cancellation and acceptance evidence references. |
| tests/recovery.rs | recovery | original planned path | Recovery cases cross live-attempt identity, cancellation, evidence/acceptance boundaries and restored event epochs. |
| tests/accounting.rs | budget | original planned path | Accounting cases include concurrency, retries, checking, compaction, parent/child allocations and constrained fallback. |
| evaluation/tasks/ | julia | original planned path | Held-out workload corpus compares routing and cohort outcomes with fixed acceptance and complete cost accounting. |
| evaluation/cohorts/ | cohort | original planned path | Cohort scenarios evaluate shared objectives, resource conflicts, joins, rework and conserved allocations. |
| deploy/ | app | original planned path | Shared package and deployment corpus covers the existing stack, managed lifecycle integration, backup, upgrade and rollback. |
| workflows/ | workflows | original planned path | Finite typed procedures use the existing task/cohort scheduler and conserved allocations; no independent execution loop. |
| tests/faults/ | app | original planned path | Integrated critical-failure matrix crosses the planned stack and preserves intended-refusal, invalid-fixture and unmeasured distinctions. |
| docs/qualification.md | app | original planned path | Shared qualification record binds each claim to the exact integrated subject, fault, evidence and disclosed scope. |
| docs/operations.md | app | original planned path | Operator runbook joins packaging, lifecycle, retention, restore reconciliation and compatible integration updates. |
| evaluation/pilot/ | app | original planned path | Commissioned real-work pilot evaluates scoped end-to-end outcomes and release targets without automatically enabling optional lanes. |
| README.md | app | original planned path | Release entrypoint documents the supported existing stack, commands, versions, limitations and evidence. |
| docs/atlas.json | app | original planned path | Deployment atlas projection describes existing module ownership and contracts; the planning spine remains design authority. |
| docs/security-findings.md | app | original planned path | Integrated finding dispositions record reproductions, fixes, residual risks and exact packaged subjects across existing modules. |
| tests/security/ | app | original planned path | Security qualification corpus tests the integrated threat model with independent collectors and explicit excluded or unmeasured scope. |
| evidence/release/ | app | original planned path | Release evidence aggregates exact-subject qualification receipts; aggregation cannot create or replace module proof. |
| src/lib.rs | app | authored candidate / T25 | Assembles the implemented Rust package and meaningful doctest; no module admission. |
| tests/t01_task.rs | task | authored candidate / T01 | Explicitly declared Rust integration target for the original foundation candidate. |
| tools/check-t01 | check | authored candidate / T01 | Scoped foundation development runner and retained observation producer; not protected collection. |
| tools/check-workload-u64 | check | authored candidate / T01 | Scoped foundation development runner and retained observation producer; not protected collection. |
| evaluation/tasks/WL-U64-PARSE-001/v1/TASK.md | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| evaluation/tasks/WL-U64-PARSE-001/v1/base/Cargo.toml | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| evaluation/tasks/WL-U64-PARSE-001/v1/base/src/lib.rs | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| evaluation/tasks/WL-U64-PARSE-001/v1/manifest.json | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| evaluation/tasks/WL-U64-PARSE-001/v1/oracle/build_cases.py | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| evaluation/tasks/WL-U64-PARSE-001/v1/oracle/cases.json | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| evaluation/tasks/WL-U64-PARSE-001/v1/reference/src/lib.rs | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| evaluation/tasks/WL-U64-PARSE-001/v1/reference.patch | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| tests/fixtures/native/control-v1/C01-control-valid-cancel.jsonl | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| tests/fixtures/native/control-v1/C01-metadata.json | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| tests/fixtures/native/control-v1/actions.json | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| tests/fixtures/native/control-v1/observations.json | check | authored candidate / T01 | Immutable development fixture/oracle or explicit structural example; not benchmark or admission credit. |
| evaluation/tasks/manifest-v1.schema.json | check | authored candidate / T01 | Frozen development manifest and full-suite schema; actual held-out materialization remains due before T12 evaluation. |
| evaluation/tasks/suite-v1.schema.json | check | authored candidate / T01 | Frozen development manifest and full-suite schema; actual held-out materialization remains due before T12 evaluation. |
| src/store/roster/attempts.rs | store | authored candidate / T05 | Original T05 atomic roster selection, immutable attempt pins and attributable cancellation causes. |
| tests/t05_store.rs | roster | authored candidate / T05 | Independent T05 durable roster, atomic import, pin/disable and history development cases; separate Store regression credit. |
| development/t06/README.md | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/Cargo.lock | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/Cargo.toml | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/compare-result.py | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/prepare-inputs.py | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/src/frontend.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/src/inputs.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/src/lib.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/src/main.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/src/manifest.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/src/probes.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/tests/frontend_controls.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/tests/test_freezer.py | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/fixed-runtime-frontend/write-build-record.py | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/Cargo.lock | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/Cargo.toml | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/clock.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/lib.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/native_executor_observation.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/prepare.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/runtime-preparation.json | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/runtime.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/runtime_controls.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/runtime_fixture.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/runtime_readback.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/support.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/u64_receipt/binding.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/u64_receipt/observed.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/u64_receipt/preparation.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/src/u64_receipt.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/tests/fixtures/existing-objects.json | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/tests/fixtures/review-bundle.json | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/tests/fixtures/review-request.json | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/tests/native_store.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/tests/prepare_controls.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/tests/receipt_inventory_controls.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| development/t06/task-runtime/tests/support_controls.rs | app | authored candidate / T06 | Existing fixed T06 development runtime/frontend composition owned by app; single root engine dependencies, no new architectural module or T28 public action; scoped tooling retains explicit private external inputs. |
| src/app/capture.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/app/durable_control.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/app/evidence.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/app/mod.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/app/receipt_import.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/app/repair.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/app/subjects.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/app/workload.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/main.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/task/driver.rs | task | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_bounded_preflight.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_driver.rs | task | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_durable_control.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_evidence.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_receipt_import.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_staging.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_store.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_store_staging.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_subjects.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_terminal.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |

## Delivery, verification and hardening contracts

### T01 · Define the first task and release contract

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)

Dependencies: none. Atlas task state: `done`.

**Acceptance:** A real contained development task names inputs, acceptance, privacy, budget mode, pilot workload/window and measurable release targets. It can be expressed without an LLM or an inherited HEE packet set. Define loop limits, no-progress policy, criterion-to-verifier mapping and the minimal completion predicate before dispatch. Fix the initial module responsibilities, bidirectional boundary contracts and API/Unix-socket schema, with one state owner and no circular build dependencies. Freeze the minimal action IDs/envelope, error taxonomy, idempotency selector and state dimensions before the early CLI/UDS slice. Declare actual numeric resource/transport bounds during contract implementation.

**Validator:** Proposed, not implemented: habitat-engine-check case T01 --fixture-root <disposable-fixtures>

### T04 · Implement the transactional task ledger

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)

Dependencies: T01, T02. Atlas task state: `done`.

**Acceptance:** Submission replay is idempotent; same key with different content conflicts; task/attempt/event updates commit together; migrations and consistent backup have failure tests. The first ledger includes atomic conservative work/verification reservations, effect uncertainty and cancellation intent. task.get can recover a lost admission reply by principal-scoped request key. Allocate event identity before immutable manifest creation; acceptance commits its reference with the outbox. Publish immutable artifacts crash-durably under a declared filesystem contract before committing their SQLite references; a completed write alone is insufficient. Use file/metadata synchronisation and atomic publication as required by that contract. Historical acceptance and current evidence availability are separate facts.

**Validator:** Proposed, not implemented: habitat-engine-check case T04 --fixture-root <disposable-fixtures>

### T06 · Close one execute–verify–repair loop with completion evidence

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)

Dependencies: T03, T04, T05. Atlas task state: `done`.

**Acceptance:** A contained task uses the selected recipe and owned workspace, produces an artifact, runs an actual acceptance check and records result/cost. Worker end or exit zero alone cannot accept it. A failing candidate is repaired and reverified; exhausted and verifier-error cases stop truthfully. Acceptance and completion outbox event commit together with a criterion-to-evidence manifest bound to the actual candidate. Even this first loop requires a safe minimal budget gate and reserved verification allocation; unknown accounting cannot satisfy a hard ceiling. Rich cross-adapter reconciliation follows in T10. Terminal cancellation requires observed cleanup/effect reconciliation.

**Validator:** Proposed, not implemented: habitat-engine-check case T06 --fixture-root <disposable-fixtures>

### T07 · Recover and cancel owned attempts

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)

Dependencies: T06. Atlas task state: `wip`.

**Acceptance:** Crashes before/after dispatch acknowledgement, stale generations, PID reuse, queued Pi messages and partial cleanup produce correct recoverable states without duplicate writes. Ambiguous external outcomes remain explicit. Cover restart at verification/evidence/acceptance boundaries; lease expiry alone cannot authorize reuse of a still-writable old workspace. Acceptance explicitly rejects any earlier committed cancellation intent, even before terminal cleanup. If acceptance committed first, later cancellation cannot rewrite history. Startup may reattach observation to a positively reconciled live attempt without redispatch. Restored ledger epochs cannot silently reuse prior event cursors.

**Validator:** Proposed, not implemented: habitat-engine-check case T07 --fixture-root <disposable-fixtures>

### T10 · Account for whole-task cost and bounded fallback

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)

Dependencies: T04, T08, T09. Atlas task state: `idle`.

**Acceptance:** Concurrent reservations cannot exceed configured bounds; retries/checking/compaction are included; unknown usage is visible; hard-ceiling tasks reject incompatible accounting; fallback cannot cross privacy or tool constraints.

**Validator:** Proposed, not implemented: habitat-engine-check case T10 --fixture-root <disposable-fixtures>

### T12 · Compare routing against a fixed-model baseline

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)

Dependencies: T09, T10, T11, T21, T22. Atlas task state: `idle`.

**Acceptance:** Held-out representative tasks use the same acceptance method; failed/abandoned attempts enter cost totals; report uncertainty by class and review work. Retain a simple policy unless a challenger improves the declared tradeoff. Julia reports routing and cohort comparisons from the same immutable outcomes, with uncertainty and missingness. Include failed, cancelled and abandoned attempts and unknown usage at an explicit dataset cutoff; disclose exclusion and censoring policy.

**Validator:** Proposed, not implemented: habitat-engine-check case T12 --fixture-root <disposable-fixtures>

### T14 · Add scoped lifecycle actions through systemd

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)

Dependencies: T07, T13. Atlas task state: `idle`.

**Acceptance:** Only explicitly managed units can change state; dependencies and result readback are checked; stalled service, restart loop and coordinator restart cases do not create competing process owners.

**Validator:** Proposed, not implemented: habitat-engine-check case T14 --fixture-root <disposable-fixtures>

### T17 · Exercise the critical failure matrix

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)

Dependencies: T07, T10, T14, T15, T16, T21, T22, T25, T26, T28, T29. Atlas task state: `idle`.

**Acceptance:** Lost completion, stalled runtime, stale lease, duplicate submission, exhausted budget, provider failure and disk error each hit the intended boundary. Record exact build/config and limitations; do not substitute test count for outcome. Fabricated done, wrong-subject evidence, mid-check artifact mutation, broken verifier, repeated non-progress, failed parent integration and lost final notification cannot produce false accepted completion. Exercise partial/coalesced frames, slow API readers, stale event cursors, wrong socket/session and lost response after effect. Inject failures between artifact write, durability publication and acceptance commit under the declared filesystem contract. Missing, expired or corrupt evidence at later readback must report current unavailability while preserving the historical event; no fresh verification may be inferred. Fault experiments distinguish invalid fixtures, excluded cells and unmeasured gates. A formatting rejection that prevents the intended behavioral fault does not prove that behavioral detector works.

**Validator:** Proposed, not implemented: habitat-engine-check case T17 --fixture-root <disposable-fixtures>

### T18 · Package and prove backup, upgrade and rollback

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)

Dependencies: T04, T14, T15, T29. Atlas task state: `idle`.

**Acceptance:** Versioned artifact, dependencies and config are identifiable; login/reboot behaviour is observed; consistent backup restores tasks/artifacts; upgrade and schema-compatible rollback run on disposable state before installation. Before restoring a pre-upgrade database, preserve the failed/current ledger and reconcile post-backup operations, effects and usage. Use a disposable bounded qualification domain; a changed event epoch alone does not prevent replaying external effects erased by restore. Backup/restore and retention must preserve declared durability and unresolved evidence obligations. Retention policy states which accepted proof must remain retrievable; evidence loss is reported rather than silently rewriting acceptance or claiming current proof. Integration freshness reports include source additions/removals/changes; accepting an updated baseline is a separate versioned reconciliation, never an automatic consequence of discovery. Restore qualification compares the expected content/service inventory, includes an omitted-service control and independently reads back answering interfaces and residual processes in the disposable scope. File copying alone cannot prove restored operation; local recovery does not establish off-site disaster recovery.

**Validator:** Proposed, not implemented: habitat-engine-check case T18 --fixture-root <disposable-fixtures>

### T19 · Run the constrained real-work pilot

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)

Dependencies: T12, T16, T17, T18, T27. Atlas task state: `idle`.

**Acceptance:** Under the concrete commissioning choice D07, run the Phase-1 workload/window; measure accepted work, total cost, rework, idle calls and incidents. Meet declared release targets or fix the observed gap without widening scope. Audit sampled criterion-to-evidence records and real repair loops; report false acceptance, verification error and completion-delivery failures.

**Validator:** Proposed, not implemented: habitat-engine-check case T19 --fixture-root <disposable-fixtures>

### T20 · Release the scoped product and prune unnecessary machinery

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)

Dependencies: T19. Atlas task state: `idle`.

**Acceptance:** A fresh operator can submit, inspect, cancel, recover and manage the declared services. Publish supported versions/limits and actual evidence. Remove duplicate status/procedure sources; advanced roadmap items remain optional. Pruning/update review keeps observed drift separate from the accepted baseline. Exact source anchors support presence/location claims only, not runtime behavior or compatibility.

**Validator:** Proposed, not implemented: habitat-engine-check case T20 --fixture-root <disposable-fixtures>

### T22 · Add bounded cohorts and measurable cohesion

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)

Dependencies: T07, T10, T11, T21. Atlas task state: `idle`.

**Acceptance:** Roles share a versioned objective, distinct artifact ownership and bounded messages. Cross-agent resource conflicts and stale-generation callbacks are rejected. Compare one-worker and specialist-cohort outcomes including cost, errors, disagreement and rework; idle cohorts stop. Persist logical thread identity, versioned mailbox obligations and parent join requirements; protect shared interface/lockfile ownership and recover orchestrator loss without duplicate dispatch. Enforce one conserved parent/child allocation tree. Waiting parents release resources children require. Semantic cross-module calls remain mediated by the composition root and current action authority. Thread briefs preserve the task-critical relationships needed by the receiving specialist; a list of included files alone cannot prove adequate context. Narrow the question explicitly when required coverage exceeds its bound.

**Validator:** Proposed, not implemented: habitat-engine-check case T22 --fixture-root <disposable-fixtures>

### T25 · Establish the Rust and Julia quality contract

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)

Dependencies: T01. Atlas task state: `done`.

**Acceptance:** Pinned format/lint/test recipes include pedantic Clippy, supported feature profiles, Rust doctests, locked Julia tests with explicit bounds checks, and honest case/skip counts. A deliberately bad fixture fails for the intended reason. Checks grow with each vertical slice; no blanket lint suppression. The verifier recipe separates trusted launch/collection from candidate-controlled compiler/build/test processes. Captured process exit status is an observation; candidate structured reports remain claims requiring recipe checks and independent expectations. Early trusted-host fixture tests must disclose that stronger hostile-code containment is not yet qualified. Qualify consequential gate discrimination on a passing baseline and an independently justified fault with the intended failure reason; include benign near-neighbors where false positives matter. Non-blocking authoring hooks do not satisfy required release gates.

**Validator:** Proposed, not implemented: habitat-engine-check case T25 --fixture-root <disposable-fixtures>

### T26 · Define the threat model and security review contract

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)

Dependencies: T01. Atlas task state: `done`.

**Acceptance:** Map task authority, context provenance, credentials, IPC, same-UID limits, isolation and trusted verifier ownership. Assign bounded negative cases and finding disposition. Confirm the selected security-model capability without granting extra execution authority. Include candidate-controlled build scripts, procedural macros, test binaries and Julia package hooks in the threat model; invoking them via a trusted verifier confers no collector authority.

**Validator:** Proposed, not implemented: habitat-engine-check case T26 --fixture-root <disposable-fixtures>

### T27 · Close security findings against the integrated release

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)

Dependencies: T15, T17, T18, T25, T26, T28, T29. Atlas task state: `idle`.

**Acceptance:** Reproduce material findings, verify fixes and residual dispositions on the actual packaged candidate. Prompt injection, credential exposure, socket/method misuse, stale writers, fabricated evidence, dependency drift and resource exhaustion cannot produce false acceptance. Model review alone is not closure. In disposable qualified fixtures, candidate build/test code attempts to alter protected fixture, evidence and control paths. The boundary must refuse the effect and the independent collector must expose the attempt; an exit-zero candidate report cannot certify its own integrity. Security fault cases require a valid unmutated baseline and the intended observable refusal. Preserve unmeasured/excluded scope; one corpus with no unique catch does not justify removing a gate.

**Validator:** Proposed, not implemented: habitat-engine-check case T27 --fixture-root <disposable-fixtures>

### T29 · Qualify modular Bash, Pi, skill and workflow integration packages

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)

Dependencies: T21, T22, T25, T26, T28. Atlas task state: `idle`.

**Acceptance:** Thin updatable packages share action/procedure identity and preserve authority, arguments, producer exit status, cancellation and evidence. Pi errors/parallel calls/reload and effective tool-set readback are qualified; skills disclose bounded context. Two typed compositions close their real return paths. Version switching preserves active attempts and rollback; no separate scheduler or silent self-update. Keep the admitted package/protocol tuple pinned through every active attempt. Revoked/reloaded tool handlers cannot retain effect authority. No custom Pi-host glue is selected until its concrete packaging need is established. Each integration renders its owned template once, then passes literal typed data rather than copying/re-templating procedure bodies. Focused fixtures include braces, quotes, newlines, dollar signs and leading dashes, with exact final argv/JSON and producer-status checks. Preserve source and relationship coverage with whole-workflow context accounting. Source-change reports cannot advance accepted compatibility/source baselines.

**Validator:** Proposed, not implemented: habitat-engine-check case T29 --fixture-root <disposable-fixtures>

## Closure and rollout discipline

Apply the existing plan → execute → observe → verify → repair/escalate → verify → close loop. Before deployment, qualify the assembled candidate and relevant T25/T26 checks, T17 failures, T27 security finding closure and T18 backup/upgrade/rollback obligations in their declared scope. T19/T20 govern pilot and release; these links do not mark them complete. Package and configuration identity must match observed installation and useful readback. Preserve cancellation, uncertain effects, cleanup and rollback obligations separately.

A successful compile or lint of comment-only stubs is not module verification. The Rust pedantic-Clippy/test plan, Julia checks, transport/fault cases and security reviews remain governed by their actual task criteria. [The evidence and update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) refreshes all corpus views after retained observations are admitted.

## Remaining reference-example limits

The in-memory policy must be joined with durable task, budget, cohort and artifact conditions in one production acceptance transaction.

## Full return-anchor register

- [module cluster · CLU-K1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K1)
- [contributing codebase · CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
- [contributing codebase · CODE-CB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB03)
- [contributing codebase · CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
- [task · TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
- [task · TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
- [implementation support task · TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
- [task · TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
- [task · TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
- [task · TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
- [task · TASK-T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)
- [task · TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
- [task · TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
- [task · TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
- [task · TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
- [task · TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
- [task · TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
- [task · TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
- [task · TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
- [task · TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
- [task · TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
- [separate reference example · EX-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-task)
- [flow · FLOW-F01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F01)
- [flow · FLOW-F02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F02)
- [flow · FLOW-F03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F03)
- [flow · FLOW-F04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F04)
- [flow · FLOW-F05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F05)
- [flow · FLOW-F06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F06)
- [flow · FLOW-F07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F07)
- [flow · FLOW-F08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F08)
- [flow · FLOW-F10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F10)
- [flow · FLOW-F11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F11)
- [flow · FLOW-F13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F13)
- [flow · FLOW-F15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F15)
- [flow · FLOW-F19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F19)
- [handbook · HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
- [handbook · HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
- [handbook · HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
- [API · API-API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01)
- [API · API-API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10)
- [action · ACT-task.cancel](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.cancel)
- [action · ACT-task.get](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.get)
- [action · ACT-task.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.list)
- [action · ACT-task.preview](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.preview)
- [action · ACT-task.resolve](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.resolve)
- [action · ACT-task.submit](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.submit)
- [IPC · IPC-IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01)
- [IPC · IPC-IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04)
- [public interface convention · Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
- [planned module · MOD-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-task)
- [plan · SEC-architecture](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-architecture)
- [plan · SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
- [plan · SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
- [plan · SEC-threads](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-threads)
- [requirement · REQ-R02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R02)
- [requirement · REQ-R03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R03)
- [requirement · REQ-R04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R04)
- [requirement · REQ-R05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R05)
- [requirement · REQ-R06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R06)
- [requirement · REQ-R08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R08)
- [requirement · REQ-R09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R09)
- [requirement · REQ-R10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R10)
- [requirement · REQ-R11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R11)
- [requirement · REQ-R12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R12)
- [requirement · REQ-R13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R13)
- [requirement · REQ-R14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R14)
- [requirement · REQ-R15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R15)
- [requirement · REQ-R17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R17)
- [requirement · REQ-R18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R18)
- [requirement · REQ-R19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R19)
- [requirement · REQ-R20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R20)
- [requirement · REQ-R21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R21)
- [schematic · SC-SC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC01)
- [schematic · SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
- [schematic · SC-SC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC04)
- [schematic · SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
- [schematic · SC-SC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC06)
- [schematic · SC-SC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC07)
- [schematic · SC-SC08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC08)
- [schematic · SC-SC10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC10)
- [schematic · SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
- [schematic · SC-SC12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC12)
- [schematic · SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
- [schematic · SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
- [schematic · SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
- [schematic · SC-SC20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC20)
- [schematic · SC-SC21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC21)
- [schematic · SC-SC22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC22)
- [schematic · SC-SC23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC23)
- [schematic · SC-SC24](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC24)
- [source · SRC-A03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A03)
- [source · SRC-C02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C02)
- [source · SRC-S15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S15)
- [testing standard · Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
- [progressive context workflow · Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
- [module context scout · CTX-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-task)
- [corpus architecture and verification schematic · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
- [corpus architecture and verification schematic · CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
- [corpus architecture and verification schematic · CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
- [corpus architecture and verification schematic · CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
- [corpus architecture and verification schematic · CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
- [corpus architecture and verification schematic · CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
- [corpus architecture and verification schematic · CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
- [resolved design contracts · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
- [resolved module contract · RC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01)
- [resolved module contract · RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02)
- [resolved module contract · RC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03)
- [resolved module contract · RC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04)
- [resolved module contract · RC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05)
- [resolved module contract · RC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06)
- [adopted readiness convention · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
- [readiness criterion cluster · F1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF1)
- [readiness criterion cluster · F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
- [readiness criterion cluster · F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
- [readiness criterion cluster · F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
- [readiness criterion cluster · F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
- [readiness criterion cluster · F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
- [readiness improvement grouping · R90-01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-01)
- [readiness improvement grouping · R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
- [readiness improvement grouping · R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
- [readiness improvement grouping · R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
- [readiness improvement grouping · R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
- [Graphify corpus projection · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
- [defensive security convention · Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
- [defensive security convention · RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
- [defensive security convention · Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
- [completion and operational convention · Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
- [completion and operational convention · DONE-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-task)
- [completion and operational convention · Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
- [completion and operational convention · Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
- [completion and operational convention · RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
- [completion and operational convention · RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
- [applied learning · LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
- [applied learning · LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
- [applied learning · LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
- [applied learning · LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11)
- [diary evidence source · DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
- [diary evidence source · DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
- [diary evidence source · DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
- [diary evidence source · DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
- [diary evidence source · DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
- [diary evidence source · DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
- [diary evidence source · DR12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR12)
