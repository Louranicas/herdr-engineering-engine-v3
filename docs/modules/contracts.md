# HEE3-MOD-contracts · contracts

Boundary data types, identities, errors and schema versions.

Primary path: `src/contracts.rs`. Cluster: K1. This is the full planned responsibility; a reference exemplar exercises only its stated subset.

## Anchors and source ownership

[Source comment anchor](file:///var/home/herdr-engineering-engine-v3/src/contracts.rs) · [Atlas module card](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-contracts) · [Scoped reference example](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-contracts) · [Ultra map master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md)

## Dependencies and integration

Declared build dependencies: none. Declared consumers: task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, app, actions. Runtime request/return paths below do not imply cyclic build imports.

## Adopted readiness obligations

Own typed identities, capability boundaries and request/result/error compatibility. Bind every supported operation and shared fixture to a version; do not become a second task ledger.

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). Binding: `HEE3-READINESS-001` / `3bcde91b4617c1a38bcbb97c36bddbc9e999c7692a69b0ba21caec9f559fe250`. Adoption is complete; the required engine proof is pending.

| Applicable facet / criterion IDs | Primary contract owners |
| --- | --- |
| [F2 · Architecture, modularity and maintainability](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2): F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [F3 · Public interfaces and integration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3): F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06 | contracts, task, worker, actions, app, numerical, julia, herdr, bash, pi_extension, skills, workflows |
| [F4 · Testing, verification and completion evidence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4): F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07 | check, contracts, app, task, julia |
| [F5 · Security and hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5): F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06 | worker, check, service, actions, app, store, context, bash, pi_extension, skills, workflows |
| [F7 · Traceability, cohesion and controlled change](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7): F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |

Implementation groupings: [R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02), [R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03), [R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04), [R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05), [R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09), [R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10). Original task prerequisites and all 13 completion gates remain controlling; package membership grants no action authority. Shared quality/security/interface rules apply to this module’s actual scope, without creating new runtime responsibilities.

**Resolved design contracts:** [RC02 · Pinned release profile and deployment custody](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02), [RC03 · Versioned control and Rust–Julia contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03), [RC04 · Protected collection, receipts and independent oracles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04), [RC05 · Trusted and adversarial execution profiles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05). Apply the exact selected profile and pending-proof obligations.

Before a change, predict the affected owners and consumer contracts. Afterwards compare actual changes, rerun invalidated checks and retain counter-evidence. Minimum 50 distinct primary-owned cases, zero baseline diagnostics, trustworthy collection and independent parent/release acceptance remain mandatory.

## Public consumer interface contract

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

Convention SHA-256: `fe9171dc482e2121ffb85acc0fec7cc794ac64ed853a7f76cda794fead8124ec`. This binding proves which documentation convention is projected, not runtime correctness.

## Fully-complete contract

**HEE3-DONE-contracts** · unassessed; acceptance collector unavailable.

[Concrete module outcome, failure controls and admission checklist](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-contracts) · [Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)

Completion standard SHA-256: `d593a1eef4c78de87afe0202ff1dcfbdd3c782692fae6480a425e40774b6a85b`. All 13 gates apply; only an inapplicable subcheck may be explicitly justified.

**Required finished behavior:** Typed request/result/error and identity/version contracts round-trip without losing semantics across declared consumers.

**Integrated proof scenario:** Send a valid versioned request through two declared consumer projections and compare typed identity, errors and cancellation fields.

**Fault and benign controls:** Reject oversized/malformed or incompatible payload before allocation/dispatch; accept the bounded compatible counterpart.

**Complexity boundary:** No registry, transport listener or task scheduler in the contract library.

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Daybreak defensive security review

Requested model: `gpt-daybreak-blue-latest`; select through the Codex `/model` picker and verify effective identity before the scoped review. Model selection and a reviewer message cannot accept this module.

**HEE3-SEC-contracts:** Untrusted schema/frame bounds, compatibility, identity preservation and parser fuzz controls.

[Model selection, evidence and full security matrix](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB05 security hardening and re-verification](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)

Runtime security qualification is unassessed. Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Applicable patterns, antipatterns and learning triggers

[Reviewed diary learnings, evidence limits and retirement rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Assimilation and verification workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FCorpus%20Assimilation)

| Learning | Use when | Good pattern |
| --- | --- | --- |
| [LRN01 · Earn the verdict from the complete subject](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01) | Before quoting a result, admitting a module, closing a parent or presenting done in a client. | Report a typed scoped verdict with exact candidate, admitted criteria, required checks, raw evidence, gaps and a counter-evidence pointer. |
| [LRN03 · Put the bound before allocation and the guard before effect](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03) | Before parsing, allocation, fan-out, queue insertion, process launch or an irreversible effect. | Bound input acquisition and concurrency at the boundary; validate identities, capabilities and constraints before dispatch. |
| [LRN04 · One semantic owner across every consumer surface](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04) | When adding a CLI, tool, Pi, Bash, workflow or cross-language entry point. | Use one typed owner and thin projections; pass source, environment, task and caller identities explicitly and preserve the return contract. |
| [LRN08 · Keep meaningful coverage and the zero-warning baseline](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08) | When designing module tests, fixing lint diagnostics or reporting a hardening result. | Retain at least 50 distinct meaningful module-owned cases and zero baseline warnings/errors on every admitted profile; repair code and close material gaps. |

These are scoped design and review obligations. Their engine detectors remain unqualified; source reflections and navigation counts are not implementation evidence.

## Mandatory module testing standard

[Active testing standard and counting rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing): **minimum 50 distinct qualifying cases; zero baseline warnings and errors**, including pedantic Clippy for all admitted Rust targets/profiles.

Bind assimilation, mutation sensitivity, lint coverage and raw evidence to the exact candidate. Property iterations, assertions, retries, lint findings and mutant runs do not multiply case credits. The current observation schema does not establish this qualification.

Standard: `HEE3-TEST-STD-001`; SHA-256: `f68d836a62a0522f46459f630f3beca159d1e87b6294a337101bd72392b5c578`. Full-standard qualification: **unassessed; collector/validator not implemented**.

Module priorities: schema/version compatibility; identity/correlation integrity; bounded decode and malformed inputs; typed errors and serialization invariants.

## Bidirectional contracts

No numbered direct flow is declared. Use the explicit consumers and contract/schema interfaces; do not invent a runtime channel.

## APIs, sockets and commands

Internal module: external access is mediated by its declared consumers.

## Full deployment stems

| Path | Owning module | Scope | Purpose |
| --- | --- | --- | --- |
| docs/contract.md | task | original planned path | First-task admission, bounded loop and completion contract; shared boundary and release conditions remain atlas-owned. |
| src/task.rs | task | original planned path | Owns task/attempt state and the execute–verify–repair policy without another scheduler. |
| src/worker/pi.rs | worker | original planned path | Selected Pi control adapter under the common worker contract; distinct from the custom tool bridge. |
| tests/fixtures/pi/ | worker | original planned path | Protocol fixtures for Pi framing, identity, usage, ordering and cancellation under T02. |
| src/worker/mod.rs | worker | original planned path | Existing worker module entrypoint and common launch, observation, cancellation and result contract. |
| src/worker/inference.rs | worker | original planned path | Output-only inference adapter preserves actual identity, usage and unsupported-capability refusals. |
| Cargo.toml | app | original planned path | Shared Rust package composition, dependency and feature policy for the existing Rust module set. |
| rust-toolchain.toml | app | original planned path | Pinned compiler and tooling tuple for the existing Rust modules; no additional package boundary. |
| docs/checks.md | app | original planned path | Shared quality contract spans the planned stack; each module retains its own behavioral checks and disclosed limits. |
| docs/security.md | app | original planned path | Integrated threat model records existing authority, data, execution and verification boundaries across the planned stack. |
| config/worker-profiles.toml | worker | original planned path | Declared execution profiles support worker and candidate-verification isolation without granting collector authority. |
| src/store.rs | store | original planned path | Owns the single SQLite write boundary, migrations, transactional state and outbox persistence. |
| src/roster.rs | roster | original planned path | Owns admitted roster revisions, declared/effective capabilities and pinned active snapshots. |
| src/worker/ | worker | original planned path | Existing adapter directory participates in the first execute–verify–repair slice; no second dispatch owner. |
| src/check.rs | check | original planned path | Owns protected verifier invocation and exact-subject criterion evidence; candidate exit status is not acceptance. |
| src/recovery.rs | recovery | original planned path | Owns attempt reconciliation, ambiguity and cleanup policy; generation fences do not stop physical effects. |
| src/numerical.rs | numerical | original planned path | Owns bounded Rust–Julia requests and validated returns while leaving task and policy authority in Rust. |
| src/worker/native.rs | worker | original planned path | Second selected native harness adapter remains inside the existing worker module and common contract. |
| tests/fixtures/native/ | worker | original planned path | Native-adapter fixtures qualify actual model/effort, support limits and result semantics under T08. |
| src/route.rs | route | original planned path | Owns deterministic hard eligibility, ranking and explanations with zero routing model calls. |
| src/budget.rs | budget | original planned path | Owns conserved reservations and complete usage reconciliation, including unknown usage and verification allocation. |
| src/context.rs | context | original planned path | Owns bounded context selection, provenance and separate source/critical-relationship coverage. |
| src/notify.rs | notify | original planned path | Owns actionable delivery, replay and deduplication without rewriting historical acceptance. |
| src/cohort.rs | cohort | original planned path | Owns logical threads, assignments, resource claims and parent join obligations inside the existing scheduler. |
| src/actions.rs | actions | original planned path | Owns the single typed action catalogue and scoped dispatch metadata shared by every transport projection. |
| src/worker/tools.rs | worker | original planned path | Worker-side tool adaptation consumes the action catalogue and a qualified task-bound channel, not the operator socket. |
| schemas/actions/ | actions | original planned path | Generated action schemas support typed CLI, Unix and integration projections without becoming another authority. |
| src/service.rs | service | original planned path | Owns useful probes and explicitly delegated lifecycle actions for existing registered service owners. |
| src/service/systemd.rs | service | original planned path | Internal systemd adapter delegates only scoped lifecycle actions; it is part of the existing service module. |
| deploy/ | app | original planned path | Shared package and deployment corpus covers the existing stack, managed lifecycle integration, backup, upgrade and rollback. |
| src/herdr.rs | herdr | original planned path | Owns Herdr client adaptation, pane/task presentation and reconnect reconciliation without durable Herdr event cursors. |
| integrations/bash/ | bash | original planned path | Thin updatable Bash/Just projection preserves literal arguments, action authority, cancellation and producer verdicts. |
| integrations/pi/ | pi_extension | original planned path | Versioned Pi registration/rendering/tool bridge remains distinct from base control RPC and cannot retain revoked authority. |
| tests/faults/ | app | original planned path | Integrated critical-failure matrix crosses the planned stack and preserves intended-refusal, invalid-fixture and unmeasured distinctions. |
| docs/qualification.md | app | original planned path | Shared qualification record binds each claim to the exact integrated subject, fault, evidence and disclosed scope. |
| evaluation/pilot/ | app | original planned path | Commissioned real-work pilot evaluates scoped end-to-end outcomes and release targets without automatically enabling optional lanes. |
| README.md | app | original planned path | Release entrypoint documents the supported existing stack, commands, versions, limitations and evidence. |
| docs/atlas.json | app | original planned path | Deployment atlas projection describes existing module ownership and contracts; the planning spine remains design authority. |
| docs/security-findings.md | app | original planned path | Integrated finding dispositions record reproductions, fixes, residual risks and exact packaged subjects across existing modules. |
| tests/security/ | app | original planned path | Security qualification corpus tests the integrated threat model with independent collectors and explicit excluded or unmeasured scope. |
| evidence/release/ | app | original planned path | Release evidence aggregates exact-subject qualification receipts; aggregation cannot create or replace module proof. |
| numerical-workers/ | numerical | original planned path | Conditional numerical worker packaging is owned by the existing numerical boundary; no new mandatory runtime module is introduced. |
| evaluation/runtime-compatibility.md | numerical | original planned path | Optional Julia/tch/LibTorch or external-service comparison records the concrete model, device, format, cost and cancellation contract. |
| src/lib.rs | app | authored candidate / T25 | Assembles the implemented Rust package and meaningful doctest; no module admission. |
| tests/t01_contracts.rs | contracts | authored candidate / T01 | Explicitly declared Rust integration target for the original foundation candidate. |
| tools/check-t01 | check | authored candidate / T01 | Scoped foundation development runner and retained observation producer; not protected collection. |
| tools/check-workload-u64 | check | authored candidate / T01 | Scoped foundation development runner and retained observation producer; not protected collection. |
| schemas/actions/generate_control_schema.py | actions | authored candidate / T01 | Closed control schema, generator or structural fixture checks; byte and authority gaps remain runtime-owned. |
| schemas/actions/control-v1.schema.json | actions | authored candidate / T01 | Closed control schema, generator or structural fixture checks; byte and authority gaps remain runtime-owned. |
| tests/control_schema.py | actions | authored candidate / T01 | Closed control schema, generator or structural fixture checks; byte and authority gaps remain runtime-owned. |
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
| schemas/receipts/generate_receipt_schema.py | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| schemas/receipts/receipt-v1.schema.json | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| tests/fixtures/receipts/bootstrap-specs.json | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| tests/fixtures/receipts/invalid-overrides.json | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| tests/fixtures/receipts/receipt-examples.json | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| tests/receipt_schema.py | check | authored candidate / T25 | Structural receipt positive/negative controls and documented runtime proof limits. |
| tests/fixtures/receipts/inventory-examples.json | check | authored candidate / T25 | Explicit closed inventory record/page examples; fictional references supply structure checks only. |
| src/contracts/roster.rs | contracts | authored candidate / T05 | Shared closed roster DTOs, exact field validation and dated proof comparison for original T05. |
| tests/t05_roster.rs | roster | authored candidate / T05 | Independent T05 definition, proof freshness and capability selection development cases. |
| tests/t05_codec.rs | roster | authored candidate / T05 | Independent T05 closed TOML import, snapshot export and captured query cases. |
| docs/roster-contract.md | roster | authored candidate / T05 | Concrete T05 trusted-facade behavior, bounds, evidence and remaining integration obligations. |
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
| src/check/collector.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/check/consistency.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/check/decision.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/check/graph.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/check/u64_oracle.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/contracts/receipt/codec.rs | contracts | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/contracts/receipt/invariants.rs | contracts | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/contracts/receipt/primitives.rs | contracts | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/contracts/receipt/records.rs | contracts | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/contracts/receipt.rs | contracts | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/contracts.rs | contracts | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/main.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/store/staging.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/store/terminal.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/store/verification.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/task/driver.rs | task | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/worker/aggregate.rs | worker | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/worker/aggregate_io.rs | worker | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/worker/namespace.rs | worker | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/worker/namespace_shim.rs | worker | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/worker/process.rs | worker | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/worker/resources.rs | worker | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| src/worker/workspace.rs | worker | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_availability.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_collector.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_consistency.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_decision.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_graph.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_receipts.rs | contracts | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t09_route.rs | route | authored candidate / T09 | Declared Rust integration target for deterministic eligibility and routing; pure policy, no model call. |
| tests/t11_notify.rs | notify | authored candidate / T11 | Declared Rust integration target for committed-event subscription and delivery obligations. |
| tests/t11_context.rs | context | authored candidate / T11 | Declared Rust integration target for bounded reproducible context packets and their accounted cost. |
| tests/t22_cohort.rs | cohort | authored candidate / T22 | Declared Rust integration target for bounded specialist threads, disjoint write ownership and parent joins. |
| tests/t28_actions.rs | actions | authored candidate / T28 | Declared Rust integration target for the versioned action catalogue, compared against the plan spine. |
| tests/t16_herdr.rs | herdr | authored candidate / T16 | Declared Rust integration target for the multiplexer client surface; presentation only, no authority. |

## Delivery, verification and hardening contracts

### T01 · Define the first task and release contract

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)

Dependencies: none. Atlas task state: `done`.

**Acceptance:** A real contained development task names inputs, acceptance, privacy, budget mode, pilot workload/window and measurable release targets. It can be expressed without an LLM or an inherited HEE packet set. Define loop limits, no-progress policy, criterion-to-verifier mapping and the minimal completion predicate before dispatch. Fix the initial module responsibilities, bidirectional boundary contracts and API/Unix-socket schema, with one state owner and no circular build dependencies. Freeze the minimal action IDs/envelope, error taxonomy, idempotency selector and state dimensions before the early CLI/UDS slice. Declare actual numeric resource/transport bounds during contract implementation.

**Validator:** Proposed, not implemented: habitat-engine-check case T01 --fixture-root <disposable-fixtures>

### T02 · Qualify the narrow Rust-to-Pi adapter

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T02)

Dependencies: T01, T25, T26. Atlas task state: `done`.

**Acceptance:** Fixture and later permitted smoke cases cover LF framing, model readback, usage, queued work, retry/end/settled ordering, cancellation and malformed/truncated records. Record the supported Rust/Pi protocol pair; no broad framework scaffold.

**Validator:** Proposed, not implemented: habitat-engine-check case T02 --fixture-root <disposable-fixtures>

### T03 · Define common worker and plain inference contracts

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T03)

Dependencies: T02. Atlas task state: `done`.

**Acceptance:** Launch, observe, cancel and result semantics work for tool-capable and output-only models. Missing features return unsupported; normalisation retains provider-specific usage and actual identity.

**Validator:** Proposed, not implemented: habitat-engine-check case T03 --fixture-root <disposable-fixtures>

### T04 · Implement the transactional task ledger

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)

Dependencies: T01, T02. Atlas task state: `done`.

**Acceptance:** Submission replay is idempotent; same key with different content conflicts; task/attempt/event updates commit together; migrations and consistent backup have failure tests. The first ledger includes atomic conservative work/verification reservations, effect uncertainty and cancellation intent. task.get can recover a lost admission reply by principal-scoped request key. Allocate event identity before immutable manifest creation; acceptance commits its reference with the outbox. Publish immutable artifacts crash-durably under a declared filesystem contract before committing their SQLite references; a completed write alone is insufficient. Use file/metadata synchronisation and atomic publication as required by that contract. Historical acceptance and current evidence availability are separate facts.

**Validator:** Proposed, not implemented: habitat-engine-check case T04 --fixture-root <disposable-fixtures>

### T05 · Build the model and agent roster

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)

Dependencies: T03, T04. Atlas task state: `done`.

**Acceptance:** Profiles and instances are distinct; declared/effective capabilities and freshness are visible; disable prevents new work; importing a roster starts nothing; secrets remain references. SQLite owns admitted roster revisions. Reviewed TOML manifests are explicit bootstrap/import/export inputs through the same validated update authority: expected-absent creates, expected-revision updates. Invalid or stale import changes nothing. Test update/disable, restart and stale re-import; no admitted mutation is lost and disabled records remain disabled. Active attempts retain their pinned record snapshots.

**Validator:** Proposed, not implemented: habitat-engine-check case T05 --fixture-root <disposable-fixtures>

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

### T08 · Add a second worker adapter

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T08)

Dependencies: T03, T06. Atlas task state: `done`.

**Acceptance:** A selected native harness or plain inference backend passes the common contract; actual supported model/effort is recorded. No success inferred from terminal prose. Version-specific limitations are listed.

**Validator:** Proposed, not implemented: habitat-engine-check case T08 --fixture-root <disposable-fixtures>

### T09 · Implement deterministic eligibility and routing

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T09)

Dependencies: T05, T08. Atlas task state: `done`.

**Acceptance:** Capability/context/privacy/availability constraints exclude invalid recipes; rules choose a valid recipe with stable explanation; ties and insufficient evidence use an explicit baseline; routing itself makes zero model calls.

**Validator:** Proposed, not implemented: habitat-engine-check case T09 --fixture-root <disposable-fixtures>

### T10 · Account for whole-task cost and bounded fallback

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)

Dependencies: T04, T08, T09. Atlas task state: `wip`.

**Acceptance:** Concurrent reservations cannot exceed configured bounds; retries/checking/compaction are included; unknown usage is visible; hard-ceiling tasks reject incompatible accounting; fallback cannot cross privacy or tool constraints.

**Validator:** Proposed, not implemented: habitat-engine-check case T10 --fixture-root <disposable-fixtures>

### T11 · Minimise repeated context and noisy supervision

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T11)

Dependencies: T06, T10. Atlas task state: `wip`.

**Acceptance:** Context is bounded to task needs; actionable wakes are deduplicated/batched; controlled idle and repeated unchanged events cause zero model requests; summaries and compaction usage are accounted for. Context records distinguish required source-file coverage from task-critical call/dependency/ownership relationship coverage. Expose each gap and account for discovery, supplemental reads and repeated context as well as final packet size.

**Validator:** Proposed, not implemented: habitat-engine-check case T11 --fixture-root <disposable-fixtures>

### T13 · Build inspect-only service inventory and useful probes

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T13)

Dependencies: T04, T05. Atlas task state: `done`.

**Acceptance:** Daemon, one-shot, library, remote endpoint and neural-operator records use appropriate observations. Unknown/stale readiness cannot become healthy; discovery claims no lifecycle ownership. Apply T05 admitted-revision and explicit import/export rules to service records. Test service update/disable, restart and stale/invalid import; observed inventory cannot overwrite effective authority or grant lifecycle control.

**Validator:** Proposed, not implemented: habitat-engine-check case T13 --fixture-root <disposable-fixtures>

### T14 · Add scoped lifecycle actions through systemd

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)

Dependencies: T07, T13. Atlas task state: `idle`.

**Acceptance:** Only explicitly managed units can change state; dependencies and result readback are checked; stalled service, restart loop and coordinator restart cases do not create competing process owners.

**Validator:** Proposed, not implemented: habitat-engine-check case T14 --fixture-root <disposable-fixtures>

### T16 · Expose the task and roster views in Herdr

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T16)

Dependencies: T07, T11, T13. Atlas task state: `wip`.

**Acceptance:** Task cards show owner, model, state, spend and outcome; pane closure does not erase task state; reconnect resynchronises; event gap/poll fallback and user cancel are exercised on the actual public Herdr API. Display the current loop iteration, failing criterion, remaining budget, completion evidence and notification state. The captured Herdr protocol has live events without durable replay cursors. Qualify resubscription and coherent snapshot reconciliation; do not claim gap-free Herdr history.

**Validator:** Proposed, not implemented: habitat-engine-check case T16 --fixture-root <disposable-fixtures>

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

### T21 · Implement the bounded Rust–Julia analysis exchange

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21)

Dependencies: T04. Atlas task state: `done`.

**Acceptance:** An immutable attempt dataset produces a reproducible Julia report with dataset/schema identity. Missing data, nonfinite values, stale results, timeout and cancellation are explicit. Julia failure preserves the approved route and cannot mutate task ownership. Exercise the bidirectional contract corpus with independent expectations, numeric tolerances and explicit Julia bounds checks.

**Validator:** Proposed, not implemented: habitat-engine-check case T21 --fixture-root <disposable-fixtures>

### T22 · Add bounded cohorts and measurable cohesion

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)

Dependencies: T07, T10, T11, T21. Atlas task state: `wip`.

**Acceptance:** Roles share a versioned objective, distinct artifact ownership and bounded messages. Cross-agent resource conflicts and stale-generation callbacks are rejected. Compare one-worker and specialist-cohort outcomes including cost, errors, disagreement and rework; idle cohorts stop. Persist logical thread identity, versioned mailbox obligations and parent join requirements; protect shared interface/lockfile ownership and recover orchestrator loss without duplicate dispatch. Enforce one conserved parent/child allocation tree. Waiting parents release resources children require. Semantic cross-module calls remain mediated by the composition root and current action authority. Thread briefs preserve the task-critical relationships needed by the receiving specialist; a list of included files alone cannot prove adequate context. Narrow the question explicitly when required coverage exceeds its bound.

**Validator:** Proposed, not implemented: habitat-engine-check case T22 --fixture-root <disposable-fixtures>

### T23 · Qualify a PyTorch-compatible numerical worker only when justified

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T23)

Dependencies: T21. Atlas task state: `idle`.

**Acceptance:** Compare Julia-native execution with a selected tch/LibTorch or external model-service path on the same model/data. Record language boundary, versions, device, operator support, memory/copy/startup cost and cancellation. A reject/defer decision is a valid outcome; no mandatory Python product code.

**Validator:** Proposed, not implemented: habitat-engine-check case T23 --fixture-root <disposable-fixtures>

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

### T28 · Expose one authoritative action catalogue to CLI, API and LLM tools

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T28)

Dependencies: T03, T05, T06, T09, T25, T26. Atlas task state: `wip`.

**Acceptance:** One action definition drives CLI/Unix schemas and one selected LLM-tool adapter. Bounded discovery/inspection/preview, scoped invocation, identity/idempotency and evidence readback work end to end. Denial remains denial across transports; untrusted workers receive no operator socket; generated projections cannot drift. task.get and analysis.get support their own principal-scoped admission-key readback; service.inspect can query the lifecycle operation key without confusing current health with historic effect. events.subscribe without a cursor provides a bounded coherent bootstrap snapshot plus high-water cursor. Distinguish request correlation from durable idempotency. Base Pi RPC does not supply the engine tool callback path; qualify the actual inherited channel or maintained bridge. Template-produced requests preserve literal typed values at the action boundary; model-emitted risk or approval fields never confer effect authority.

**Validator:** Proposed, not implemented: habitat-engine-check case T28 --fixture-root <disposable-fixtures>

### T29 · Qualify modular Bash, Pi, skill and workflow integration packages

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)

Dependencies: T21, T22, T25, T26, T28. Atlas task state: `wip`.

**Acceptance:** Thin updatable packages share action/procedure identity and preserve authority, arguments, producer exit status, cancellation and evidence. Pi errors/parallel calls/reload and effective tool-set readback are qualified; skills disclose bounded context. Two typed compositions close their real return paths. Version switching preserves active attempts and rollback; no separate scheduler or silent self-update. Keep the admitted package/protocol tuple pinned through every active attempt. Revoked/reloaded tool handlers cannot retain effect authority. No custom Pi-host glue is selected until its concrete packaging need is established. Each integration renders its owned template once, then passes literal typed data rather than copying/re-templating procedure bodies. Focused fixtures include braces, quotes, newlines, dollar signs and leading dashes, with exact final argv/JSON and producer-status checks. Preserve source and relationship coverage with whole-workflow context accounting. Source-change reports cannot advance accepted compatibility/source baselines.

**Validator:** Proposed, not implemented: habitat-engine-check case T29 --fixture-root <disposable-fixtures>

## Closure and rollout discipline

Apply the existing plan → execute → observe → verify → repair/escalate → verify → close loop. Before deployment, qualify the assembled candidate and relevant T25/T26 checks, T17 failures, T27 security finding closure and T18 backup/upgrade/rollback obligations in their declared scope. T19/T20 govern pilot and release; these links do not mark them complete. Package and configuration identity must match observed installation and useful readback. Preserve cancellation, uncertain effects, cleanup and rollback obligations separately.

A successful compile or lint of comment-only stubs is not module verification. The Rust pedantic-Clippy/test plan, Julia checks, transport/fault cases and security reviews remain governed by their actual task criteria. [The evidence and update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) refreshes all corpus views after retained observations are admitted.

## Remaining reference-example limits

Authentication, complete public schemas, canonical cross-language encodings and hostile collector custody remain production qualifications.

## Full return-anchor register

- [module cluster · CLU-K1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K1)
- [contributing codebase · CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
- [contributing codebase · CODE-CB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB03)
- [contributing codebase · CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
- [task · TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
- [task · TASK-T02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T02)
- [task · TASK-T03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T03)
- [task · TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
- [task · TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
- [task · TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
- [task · TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
- [task · TASK-T08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T08)
- [task · TASK-T09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T09)
- [task · TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
- [task · TASK-T11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T11)
- [task · TASK-T13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T13)
- [task · TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
- [task · TASK-T16](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T16)
- [task · TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
- [task · TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
- [task · TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
- [task · TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
- [task · TASK-T21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21)
- [task · TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
- [task · TASK-T23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T23)
- [task · TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
- [task · TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
- [task · TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
- [task · TASK-T28](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T28)
- [task · TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
- [separate reference example · EX-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-contracts)
- [handbook · HB-compatibility-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-compatibility-map)
- [handbook · HB-error-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-error-map)
- [handbook · HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
- [public interface convention · Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
- [planned module · MOD-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-contracts)
- [plan · SEC-api-sockets](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-api-sockets)
- [plan · SEC-architecture](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-architecture)
- [plan · SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
- [plan · SEC-security](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-security)
- [requirement · REQ-R01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R01)
- [requirement · REQ-R02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R02)
- [requirement · REQ-R03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R03)
- [requirement · REQ-R04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R04)
- [requirement · REQ-R05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R05)
- [requirement · REQ-R06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R06)
- [requirement · REQ-R07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R07)
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
- [schematic · SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
- [schematic · SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
- [schematic · SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
- [source · SRC-A08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A08)
- [source · SRC-C01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C01)
- [source · SRC-C02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C02)
- [testing standard · Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
- [progressive context workflow · Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
- [module context scout · CTX-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-contracts)
- [corpus architecture and verification schematic · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
- [corpus architecture and verification schematic · CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
- [corpus architecture and verification schematic · CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
- [corpus architecture and verification schematic · CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
- [corpus architecture and verification schematic · CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
- [corpus architecture and verification schematic · CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
- [corpus architecture and verification schematic · CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
- [resolved design contracts · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
- [resolved module contract · RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02)
- [resolved module contract · RC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03)
- [resolved module contract · RC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04)
- [resolved module contract · RC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05)
- [adopted readiness convention · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
- [readiness criterion cluster · F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
- [readiness criterion cluster · F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
- [readiness criterion cluster · F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
- [readiness criterion cluster · F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
- [readiness criterion cluster · F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
- [readiness improvement grouping · R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02)
- [readiness improvement grouping · R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
- [readiness improvement grouping · R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04)
- [readiness improvement grouping · R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
- [readiness improvement grouping · R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
- [readiness improvement grouping · R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
- [Graphify corpus projection · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
- [defensive security convention · Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
- [defensive security convention · RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
- [defensive security convention · Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
- [completion and operational convention · Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
- [completion and operational convention · DONE-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-contracts)
- [completion and operational convention · Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
- [completion and operational convention · Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
- [completion and operational convention · RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
- [completion and operational convention · RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
- [applied learning · LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
- [applied learning · LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03)
- [applied learning · LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
- [applied learning · LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
- [diary evidence source · DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
- [diary evidence source · DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
- [diary evidence source · DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
- [diary evidence source · DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
- [diary evidence source · DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
- [diary evidence source · DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
