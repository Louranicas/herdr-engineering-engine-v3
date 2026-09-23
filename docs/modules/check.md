# HEE3-MOD-check · check

Protected verifier invocation and criterion evidence.

Primary path: `src/check.rs`. Cluster: K4. This is the full planned responsibility; a reference exemplar exercises only its stated subset.

## Anchors and source ownership

[Source comment anchor](file:///var/home/herdr-engineering-engine-v3/src/check.rs) · [Atlas module card](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-check) · [Scoped reference example](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-check) · [Ultra map master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md)

## Dependencies and integration

Declared build dependencies: contracts. Declared consumers: app. Runtime request/return paths below do not imply cyclic build imports.

## Adopted readiness obligations

Own criterion/oracle mapping and the narrow trusted collector boundary. Specify at T25/T26, implement/bootstrap at dependency-ready T06; forged PASS cannot self-admit the collector or candidate.

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). Binding: `HEE3-READINESS-001` / `e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6`. Adoption is complete; the required engine proof is pending.

| Applicable facet / criterion IDs | Primary contract owners |
| --- | --- |
| [F2 · Architecture, modularity and maintainability](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2): F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [F3 · Public interfaces and integration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3): F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06 | contracts, task, worker, actions, app, numerical, julia, herdr, bash, pi_extension, skills, workflows |
| [F4 · Testing, verification and completion evidence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4): F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07 | check, contracts, app, task, julia |
| [F5 · Security and hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5): F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06 | worker, check, service, actions, app, store, context, bash, pi_extension, skills, workflows |
| [F6 · Build, deployment, migration and operations](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6): F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08 | app, store, recovery, service, notify, check |
| [F7 · Traceability, cohesion and controlled change](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7): F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |

Implementation groupings: [R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04), [R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05), [R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06), [R90-08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-08), [R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09), [R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10). Original task prerequisites and all 13 completion gates remain controlling; package membership grants no action authority. Shared quality/security/interface rules apply to this module’s actual scope, without creating new runtime responsibilities.

**Resolved design contracts:** [RC01 · First workload, numeric limits and evidence policy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01), [RC02 · Pinned release profile and deployment custody](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02), [RC03 · Versioned control and Rust–Julia contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03), [RC04 · Protected collection, receipts and independent oracles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04), [RC05 · Trusted and adversarial execution profiles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05), [RC06 · SQLite, migration freeze and recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06). Apply the exact selected profile and pending-proof obligations.

Before a change, predict the affected owners and consumer contracts. Afterwards compare actual changes, rerun invalidated checks and retain counter-evidence. Minimum 50 distinct primary-owned cases, zero baseline diagnostics, trustworthy collection and independent parent/release acceptance remain mandatory.

## Public consumer interface contract

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

Convention SHA-256: `0a35d116843a2ebd31280948f10b4c48d341fdb8d37df91488c543bb099ad5f4`. This binding proves which documentation convention is projected, not runtime correctness.

## Fully-complete contract

**HEE3-DONE-check** · unassessed; acceptance collector unavailable.

[Concrete module outcome, failure controls and admission checklist](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-check) · [Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)

Completion standard SHA-256: `a8e1536e0691fbaea198e4bd1fd7f135bc8851f6f6e9d55c22e7bf92c7641234`. All 13 gates apply; only an inapplicable subcheck may be explicitly justified.

**Required finished behavior:** Acceptance checks have qualified detectors and complete exact-subject evidence; forged, empty, stale or wrong-profile proof cannot become success.

**Integrated proof scenario:** A known faulty candidate fails for the intended reason, its benign mirror passes, and a repaired integrated candidate is reverified against the same criteria.

**Fault and benign controls:** Mutate fixture identity, case inventory, profile, producer status or raw log; reject each without accepting an aggregate count.

**Complexity boundary:** The full admission collector must be implemented and qualified before this module is accepted; no self-declared trust.

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Daybreak defensive security review

Requested model: `gpt-daybreak-blue-latest`; select through the Codex `/model` picker and verify effective identity before the scoped review. Model selection and a reviewer message cannot accept this module.

**HEE3-SEC-check:** Forged evidence, compromised fixtures, invalid denominator and independent verifier/collector custody.

[Model selection, evidence and full security matrix](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB05 security hardening and re-verification](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)

Runtime security qualification is unassessed. Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Applicable patterns, antipatterns and learning triggers

[Reviewed diary learnings, evidence limits and retirement rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Assimilation and verification workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FCorpus%20Assimilation)

| Learning | Use when | Good pattern |
| --- | --- | --- |
| [LRN01 · Earn the verdict from the complete subject](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01) | Before quoting a result, admitting a module, closing a parent or presenting done in a client. | Report a typed scoped verdict with exact candidate, admitted criteria, required checks, raw evidence, gaps and a counter-evidence pointer. |
| [LRN02 · Qualify the detector before trusting its scan](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN02) | Before a new or changed detector is allowed to establish a clean result. | Bind an intended-fault fixture and a benign mirror to the exact production check path, with a stable fault definition and expected diagnostic. |
| [LRN05 · Identify the exact source before measuring it](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05) | At re-entry, before evaluation or hardening, and before updating a completion claim. | Bind the actual source/artifact, dependencies, fixtures, toolchain and environment to the run; distinguish intended branch, current checkout and installed artifact. |
| [LRN06 · Declare the coldness and execution layer actually tested](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06) | Before reproducibility, isolation or deployment claims and when crossing host, Toolbx and disposable worker boundaries. | Use explicit environment/toolchain/cache/workspace identities and independent output roots; qualify cold build, service state and installation separately. |
| [LRN08 · Keep meaningful coverage and the zero-warning baseline](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08) | When designing module tests, fixing lint diagnostics or reporting a hardening result. | Retain at least 50 distinct meaningful module-owned cases and zero baseline warnings/errors on every admitted profile; repair code and close material gaps. |
| [LRN12 · Calibrate the instrument and retain counterexamples](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12) | Before trusting a metric, known-answer fixture, statistical comparison or a confident correction. | Define the measured construct, population, denominator and exclusions; independently check the oracle and record falsifiers, uncertainty and changed conclusions. |
| [LRN14 · Make learning earn its maintenance cost](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14) | When adding another rule, helper, generated note, verification stage or repeated context packet. | Prefer one existing owner and reusable entry point. Record the problem, placement, footprint, verification cost, prevented failure, false-positive/rework cost and retirement condition. |

These are scoped design and review obligations. Their engine detectors remain unqualified; source reflections and navigation counts are not implementation evidence.

## Mandatory module testing standard

[Active testing standard and counting rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing): **minimum 50 distinct qualifying cases; zero baseline warnings and errors**, including pedantic Clippy for all admitted Rust targets/profiles.

Bind assimilation, mutation sensitivity, lint coverage and raw evidence to the exact candidate. Property iterations, assertions, retries, lint findings and mutant runs do not multiply case credits. The current observation schema does not establish this qualification.

Standard: `HEE3-TEST-STD-001`; SHA-256: `237c2085f25460543645ec668b626e71d30f5558f56901ae755a3be7c72b86a4`. Full-standard qualification: **unassessed; collector/validator not implemented**.

Module priorities: independent oracle and exact subject; zero/missing/skipped case refusal; forged or stale evidence; producer/collector custody and negative controls.

## Bidirectional contracts

| Flow | Request | Return | Closure |
| --- | --- | --- | --- |
| [F07 task → check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F07) | Acceptance revision, immutable candidate and protected fixture references | Criterion observations, checker errors and raw evidence references | Wrong subject, changed fixture, skipped mandatory test or forged pass cannot close. |

## APIs, sockets and commands

Internal module: external access is mediated by its declared consumers.

## Full deployment stems

| Path | Owning module | Scope | Purpose |
| --- | --- | --- | --- |
| docs/contract.md | task | original planned path | First-task admission, bounded loop and completion contract; shared boundary and release conditions remain atlas-owned. |
| Cargo.toml | app | original planned path | Shared Rust package composition, dependency and feature policy for the existing Rust module set. |
| rust-toolchain.toml | app | original planned path | Pinned compiler and tooling tuple for the existing Rust modules; no additional package boundary. |
| docs/checks.md | app | original planned path | Shared quality contract spans the planned stack; each module retains its own behavioral checks and disclosed limits. |
| docs/security.md | app | original planned path | Integrated threat model records existing authority, data, execution and verification boundaries across the planned stack. |
| config/worker-profiles.toml | worker | original planned path | Declared execution profiles support worker and candidate-verification isolation without granting collector authority. |
| migrations/001.sql | store | original planned path | Initial ledger schema joins task/attempt/event state, conservative reservations, cancellation and acceptance evidence references. |
| src/check.rs | check | original planned path | Owns protected verifier invocation and exact-subject criterion evidence; candidate exit status is not acceptance. |
| tests/recovery.rs | recovery | original planned path | Recovery cases cross live-attempt identity, cancellation, evidence/acceptance boundaries and restored event epochs. |
| tests/accounting.rs | budget | original planned path | Accounting cases include concurrency, retries, checking, compaction, parent/child allocations and constrained fallback. |
| evaluation/tasks/ | julia | original planned path | Held-out workload corpus compares routing and cohort outcomes with fixed acceptance and complete cost accounting. |
| evaluation/report.md | julia | original planned path | Evaluation report compares declared baselines and challengers from immutable outcomes and qualified acceptance evidence. |
| deploy/ | app | original planned path | Shared package and deployment corpus covers the existing stack, managed lifecycle integration, backup, upgrade and rollback. |
| deploy/worker.container | app | original planned path | Packaging artifact for worker and candidate-verification isolation; runtime ownership and cleanup remain with existing modules. |
| tests/isolation/ | worker | original planned path | Isolation fixtures cover candidate code, protected control/evidence paths, Julia hooks, credentials and process cleanup. |
| tests/faults/ | app | original planned path | Integrated critical-failure matrix crosses the planned stack and preserves intended-refusal, invalid-fixture and unmeasured distinctions. |
| docs/qualification.md | app | original planned path | Shared qualification record binds each claim to the exact integrated subject, fault, evidence and disclosed scope. |
| evaluation/pilot/ | app | original planned path | Commissioned real-work pilot evaluates scoped end-to-end outcomes and release targets without automatically enabling optional lanes. |
| README.md | app | original planned path | Release entrypoint documents the supported existing stack, commands, versions, limitations and evidence. |
| docs/atlas.json | app | original planned path | Deployment atlas projection describes existing module ownership and contracts; the planning spine remains design authority. |
| docs/security-findings.md | app | original planned path | Integrated finding dispositions record reproductions, fixes, residual risks and exact packaged subjects across existing modules. |
| tests/security/ | app | original planned path | Security qualification corpus tests the integrated threat model with independent collectors and explicit excluded or unmeasured scope. |
| evidence/release/ | app | original planned path | Release evidence aggregates exact-subject qualification receipts; aggregation cannot create or replace module proof. |
| tests/t01_contracts.rs | contracts | authored candidate / T01 | Explicitly declared Rust integration target for the original foundation candidate. |
| tests/t01_task.rs | task | authored candidate / T01 | Explicitly declared Rust integration target for the original foundation candidate. |
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
| evaluation/tasks/manifest-v1.schema.json | check | authored candidate / T01 | Frozen development manifest and full-suite schema; actual held-out materialization remains due before T12 evaluation. |
| evaluation/tasks/suite-v1.schema.json | check | authored candidate / T01 | Frozen development manifest and full-suite schema; actual held-out materialization remains due before T12 evaluation. |
| tools/check-quality | check | authored candidate / T25 | Bounded trusted-host quality recipe/process controls; no hostile collector authority. |
| tools/development_process.py | check | authored candidate / T25 | Bounded trusted-host quality recipe/process controls; no hostile collector authority. |
| tests/t25_process.py | check | authored candidate / T25 | Bounded trusted-host quality recipe/process controls; no hostile collector authority. |
| tests/t25_quality.py | check | authored candidate / T25 | Bounded trusted-host quality recipe/process controls; no hostile collector authority. |
| julia/Manifest.toml | julia | authored candidate / T25 | Exact minimal Julia package/test recipe and dependency observations, no numerical-module case credit. |
| julia/test/runtests.jl | julia | authored candidate / T25 | Exact minimal Julia package/test recipe and dependency observations, no numerical-module case credit. |
| tools/julia-quality.jl | julia | authored candidate / T25 | Exact minimal Julia package/test recipe and dependency observations, no numerical-module case credit. |
| tools/julia-quality-fixtures/assertion.jl | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-quality-fixtures/bounds.jl | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-quality-fixtures/broken.jl | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-quality-fixtures/deprecation.jl | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-quality-fixtures/empty.jl | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-quality-fixtures/registry/Registry.toml | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-quality-fixtures/skip.jl | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-quality-fixtures/warning.jl | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-format/Manifest.toml | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-format/Project.toml | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| tools/julia-format/check.jl | julia | authored candidate / T25 | Pinned Julia formatting/test-only tooling or deliberately labelled control fixture. |
| schemas/receipts/generate_receipt_schema.py | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| schemas/receipts/receipt-v1.schema.json | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| tests/fixtures/receipts/bootstrap-specs.json | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| tests/fixtures/receipts/invalid-overrides.json | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| tests/fixtures/receipts/receipt-examples.json | check | authored candidate / T25 | RC04 typed receipt structure or bootstrap specification; no collector execution or oracle truth claim. |
| tests/receipt_schema.py | check | authored candidate / T25 | Structural receipt positive/negative controls and documented runtime proof limits. |
| tests/security/check_foundation.py | app | authored candidate / T26 | Threat/review contract, bounded unexecuted negative specifications or scoped runtime reviewer identity evidence. |
| tests/security/review-record-v1.schema.json | app | authored candidate / T26 | Threat/review contract, bounded unexecuted negative specifications or scoped runtime reviewer identity evidence. |
| tests/security/threat-cases-v1.json | app | authored candidate / T26 | Threat/review contract, bounded unexecuted negative specifications or scoped runtime reviewer identity evidence. |
| evidence/implementation/T26/model-runtime-observation.json | app | authored candidate / T26 | Threat/review contract, bounded unexecuted negative specifications or scoped runtime reviewer identity evidence. |
| tests/fixtures/receipts/inventory-examples.json | check | authored candidate / T25 | Explicit closed inventory record/page examples; fictional references supply structure checks only. |
| tests/t02_pi.rs | worker | authored candidate / T02 | Scoped Pi wire/session implementation verification or source-backed compatibility guidance; TH-DEV only, no module admission. |
| tests/t02_transport.rs | worker | authored candidate / T02 | Scoped Pi wire/session implementation verification or source-backed compatibility guidance; TH-DEV only, no module admission. |
| development/transport_process.py | worker | authored candidate / T02 | Scoped Pi wire/session implementation verification or source-backed compatibility guidance; TH-DEV only, no module admission. |
| docs/pi-adapter.md | worker | authored candidate / T02 | Scoped Pi wire/session implementation verification or source-backed compatibility guidance; TH-DEV only, no module admission. |
| evidence/implementation/T02/sdk-smoke/metadata-benign.stdout | worker | authored candidate / T02 | Scoped Pi wire/session implementation verification or source-backed compatibility guidance; TH-DEV only, no module admission. |
| tests/fixtures/pi/README.md | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/authoring-check.json | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/build_fixtures.py | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/manifest.json | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/abort-ack.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/ack-null-data.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/agent-end-continuation.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/agent-end-missing-retry.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/agent-end-no-retry.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/agent-end-retry.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/agent-settled-extra.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/agent-settled.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/agent-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/assistant-end.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/assistant-error-end.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/assistant-missing-stop.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/assistant-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/assistant-unknown-block.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/assistant-unknown-usage.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/clear-empty.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/clear-removed.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/delta-missing-usage.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/delta-negative-index.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/delta-old-message.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/delta-old-partial.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/delta-unknown-kind.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/model-bad-input.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/model-image-capability.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/model-missing-cost.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/model-open-headers.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/model-selected.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/model-unavailable.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/model-unknown-cost.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/models-empty.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/models-one.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/prompt-ack.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/prompt-refused.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/queue-empty.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/queue-nonempty.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/queue-nonstring.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/raw-array-root.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/raw-crlf.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/raw-duplicate-id.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/raw-duplicate-nested.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/raw-empty-line.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/raw-invalid-utf8.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/raw-nonfinite.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/raw-trailing-value.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/raw-unterminated.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-abort.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-bash.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-clear_queue.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-get_available_models.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-get_session_stats.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-get_state.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-prompt-images.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-prompt.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-set_model.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/request-set_thinking_level.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/response-error-and-data.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/response-missing-error.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/response-no-id.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/response-success-string.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/response-unknown-field.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/response-wrong-command.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/response-wrong-id.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/retry-cancelled.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/retry-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/retry-success.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/sentinel-with-cost.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/sentinel-with-name.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/sentinel-with-route.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/sentinel-with-window.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-after-run.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-bad-enum.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-bool-counter.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-compacting.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-empty.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-foreign-model.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-fractional-counter.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-missing-field.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-model.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-negative-counter.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-null-model.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-queued.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-safe-counter.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-unavailable-sentinel.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-unknown-field.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/state-unsafe-counter.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/stats-known-context.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/stats-known.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/stats-missing-total.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/stats-unknown-context.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/stats-unknown-token.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/stats-wrong-session.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/stderr-forged-pass.wire | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/text-delta.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/text-end.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/text-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/thinking-ack.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/thinking-changed.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/thinking-delta.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/thinking-end.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/thinking-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/turn-end.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/turn-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/unicode-delta.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/unknown-lifecycle.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/unsupported-compaction.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/unsupported-deferred.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/unsupported-entry.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/unsupported-tool-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/unsupported-toolcall-delta.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/unsupported-toolcall-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/update-done.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/update-error.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/update-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/user-end.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/records/user-start.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/cancel-unsafe-dispatch.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/clear-removed-then-abort.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/compaction-after-end.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/metadata-empty.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/model-readback-mismatch.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/model-thinking-readback.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/prompt-ack-only.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/prompt-final-overrides-deltas.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/retry-remains-active.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/stale-settled.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tests/fixtures/pi/traces/thinking-clamped.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tools/pi-metadata/entry.mjs | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tools/pi-metadata/network-probe.mjs | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tools/pi-metadata/smoke.jsonl | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tools/pi-metadata/smoke.py | worker | authored candidate / T02 | Explicit Pi fixture, source provenance or credential-free SDK metadata probe; raw fixture bytes remain unchanged by corpus publication. |
| tools/check-pi-mutations | check | authored candidate / T02 | Pinned copied-input development quality support for the T02 slice; no hostile collection authority. |
| tools/rust-offline.py | check | authored candidate / T02 | Pinned copied-input development quality support for the T02 slice; no hostile collection authority. |
| tools/rust-offline.md | check | authored candidate / T02 | Pinned copied-input development quality support for the T02 slice; no hostile collection authority. |
| tools/rust-offline-inputs/itoa-1.0.18.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/manifest.json | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/memchr-2.8.3.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/proc-macro2-1.0.107.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/quote-1.0.47.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/serde-1.0.229.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/serde_core-1.0.229.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/serde_derive-1.0.229.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/serde_json-1.0.151.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/syn-3.0.5.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/unicode-ident-1.0.24.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tools/rust-offline-inputs/zmij-1.0.23.crate | check | authored candidate / T02 | Closed Cargo.lock-bound offline archive input; exact package checksums and extraction controls are development observations. |
| tests/t03_contract.rs | worker | authored candidate / T03 | Independent fake-dispatch development controls for common worker lifecycle and output-only result semantics; TH-DEV only, no module admission. |
| docs/worker-contract.md | worker | authored candidate / T03 | Scoped common worker/output-only behavior, identity and usage provenance, cancellation limits and development verification guidance; T06 launcher and T08 backend remain separate. |
| src/store/artifact.rs | store | authored candidate / T04 | Transactional store, independent development tests and scoped artifact/snapshot contract; exact runtime and source receipts remain distinct from module or operational restore admission. |
| src/store/schema.rs | store | authored candidate / T04 | Transactional store, independent development tests and scoped artifact/snapshot contract; exact runtime and source receipts remain distinct from module or operational restore admission. |
| src/store/backup.rs | store | authored candidate / T04 | Transactional store, independent development tests and scoped artifact/snapshot contract; exact runtime and source receipts remain distinct from module or operational restore admission. |
| tests/t04_store.rs | store | authored candidate / T04 | Transactional store, independent development tests and scoped artifact/snapshot contract; exact runtime and source receipts remain distinct from module or operational restore admission. |
| docs/store-contract.md | store | authored candidate / T04 | Transactional store, independent development tests and scoped artifact/snapshot contract; exact runtime and source receipts remain distinct from module or operational restore admission. |
| tools/sqlite-static.py | check | authored candidate / T04 | Fixed offline SQLite build input or bounded development mutation support for original T04; no protected collector or release authority. |
| tools/check-store-mutations | check | authored candidate / T04 | Fixed offline SQLite build input or bounded development mutation support for original T04; no protected collector or release authority. |
| tools/sqlite-inputs/sqlite-amalgamation-3530400.zip | check | authored candidate / T04 | Fixed offline SQLite build input or bounded development mutation support for original T04; no protected collector or release authority. |
| tools/rust-offline-inputs/bitflags-2.13.2.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/block-buffer-0.12.1.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/cfg-if-1.0.4.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/const-oid-0.10.2.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/cpufeatures-0.3.1.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/crypto-common-0.2.2.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/digest-0.11.3.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/errno-0.3.14.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/fallible-iterator-0.3.0.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/fallible-streaming-iterator-0.1.9.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/hybrid-array-0.4.15.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/libc-0.2.189.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/libsqlite3-sys-0.38.2.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/linux-raw-sys-0.12.1.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/pkg-config-0.3.34.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/rusqlite-0.40.2.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/rustix-1.1.4.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/sha2-0.11.0.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/smallvec-1.16.1.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/typenum-1.20.1.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/vcpkg-0.2.15.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/windows-link-0.2.1.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/windows-sys-0.61.2.crate | check | authored candidate / T04 | Exact Cargo.lock-bound offline archive closure required by original T04 static SQLite/store implementation; development input only. |
| tools/rust-offline-inputs/equivalent-1.0.2.crate | check | authored candidate / T05 | Exact original T05 TOML dependency archive; Cargo.lock and the local offline manifest bind its bytes. Development input only. |
| tools/rust-offline-inputs/hashbrown-0.17.1.crate | check | authored candidate / T05 | Exact original T05 TOML dependency archive; Cargo.lock and the local offline manifest bind its bytes. Development input only. |
| tools/rust-offline-inputs/indexmap-2.14.2.crate | check | authored candidate / T05 | Exact original T05 TOML dependency archive; Cargo.lock and the local offline manifest bind its bytes. Development input only. |
| tools/rust-offline-inputs/serde_spanned-1.1.1.crate | check | authored candidate / T05 | Exact original T05 TOML dependency archive; Cargo.lock and the local offline manifest bind its bytes. Development input only. |
| tools/rust-offline-inputs/toml-1.1.5+spec-1.1.0.crate | check | authored candidate / T05 | Exact original T05 TOML dependency archive; Cargo.lock and the local offline manifest bind its bytes. Development input only. |
| tools/rust-offline-inputs/toml_datetime-1.1.1+spec-1.1.0.crate | check | authored candidate / T05 | Exact original T05 TOML dependency archive; Cargo.lock and the local offline manifest bind its bytes. Development input only. |
| tools/rust-offline-inputs/toml_parser-1.1.3+spec-1.1.0.crate | check | authored candidate / T05 | Exact original T05 TOML dependency archive; Cargo.lock and the local offline manifest bind its bytes. Development input only. |
| tools/rust-offline-inputs/toml_writer-1.1.2+spec-1.1.0.crate | check | authored candidate / T05 | Exact original T05 TOML dependency archive; Cargo.lock and the local offline manifest bind its bytes. Development input only. |
| tools/rust-offline-inputs/winnow-1.0.4.crate | check | authored candidate / T05 | Exact original T05 TOML dependency archive; Cargo.lock and the local offline manifest bind its bytes. Development input only. |
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
| evaluation/harnesses/bootstrap/checker.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| evaluation/harnesses/bootstrap/producer.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| evaluation/harnesses/u64-public-wrapper.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
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
| tests/fixtures/receipt-import/nonpass.json | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/fixtures/receipt-import/preparation.json | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_availability.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_bounded_preflight.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_collector.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_consistency.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_decision.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_durable_control.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_evidence.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_graph.rs | check | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_receipt_import.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_receipts.rs | contracts | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_staging.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_store.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_store_staging.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_subjects.rs | app | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| tests/t06_terminal.rs | store | authored candidate / T06 | T06 current single-engine development composition and source-bound controls; no module, task or deployment admission. |
| julia/test/analysis.jl | julia | authored candidate / T21 | Bounded immutable Julia analysis exchange or source-bound numerical controls; no policy, Store write or module admission. |
| tests/fixtures/t21/J01.json | numerical | authored candidate / T21 | Source-bound numerical oracle/process/fault controls and finite requests; nested descendant control retains its bounded process owner and original failure observations. |
| tests/fixtures/t21/descendant-control.py | numerical | authored candidate / T21 | Source-bound numerical oracle/process/fault controls and finite requests; nested descendant control retains its bounded process owner and original failure observations. |
| tests/t13_local_probe.rs | service | authored candidate / T13 | Source-bound useful observation, authority, stale-revision and actual local probe controls; task evidence, not module qualification. |
| tests/t13_probe.rs | service | authored candidate / T13 | Source-bound useful observation, authority, stale-revision and actual local probe controls; task evidence, not module qualification. |
| tests/t13_service.rs | service | authored candidate / T13 | Source-bound useful observation, authority, stale-revision and actual local probe controls; task evidence, not module qualification. |
| tests/t21_analysis.rs | numerical | authored candidate / T21 | Source-bound numerical oracle/process/fault controls and finite requests; nested descendant control retains its bounded process owner and original failure observations. |
| tests/t21_process.rs | numerical | authored candidate / T21 | Source-bound numerical oracle/process/fault controls and finite requests; nested descendant control retains its bounded process owner and original failure observations. |

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

Dependencies: T04, T08, T09. Atlas task state: `wip`.

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

### T15 · Qualify worker isolation and secret delivery

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)

Dependencies: T08, T10, T14. Atlas task state: `idle`.

**Acceptance:** Untrusted worker cannot read coordinator state/socket or unintended credentials; argv/env/cwd are explicit; resource and process-group cleanup are tested; trusted-host mode has an honest separate claim. Qualify isolation for candidate-controlled verification subprocesses, including build.rs, procedural macros, tests and Julia hooks. The trusted launcher/collector remains outside that execution domain; candidate code cannot modify protected fixtures, authoritative evidence, ledger/socket or unrelated secrets.

**Validator:** Proposed, not implemented: habitat-engine-check case T15 --fixture-root <disposable-fixtures>

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

## Closure and rollout discipline

Apply the existing plan → execute → observe → verify → repair/escalate → verify → close loop. Before deployment, qualify the assembled candidate and relevant T25/T26 checks, T17 failures, T27 security finding closure and T18 backup/upgrade/rollback obligations in their declared scope. T19/T20 govern pilot and release; these links do not mark them complete. Package and configuration identity must match observed installation and useful readback. Preserve cancellation, uncertain effects, cleanup and rollback obligations separately.

A successful compile or lint of comment-only stubs is not module verification. The Rust pedantic-Clippy/test plan, Julia checks, transport/fault cases and security reviews remain governed by their actual task criteria. [The evidence and update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) refreshes all corpus views after retained observations are admitted.

## Remaining reference-example limits

The policy accepts already trusted collector reports. It does not launch, isolate or authenticate hostile candidate code; production collector custody remains outside candidate execution.

## Full return-anchor register

- [module cluster · CLU-K4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K4)
- [contributing codebase · CODE-CB01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB01)
- [contributing codebase · CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
- [contributing codebase · CODE-CB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB05)
- [contributing codebase · CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
- [task · TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
- [implementation support task · TASK-T02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T02)
- [implementation support task · TASK-T03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T03)
- [task · TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
- [implementation support task · TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
- [task · TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
- [task · TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
- [task · TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
- [task · TASK-T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)
- [implementation support task · TASK-T13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T13)
- [task · TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
- [task · TASK-T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)
- [task · TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
- [task · TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
- [task · TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
- [task · TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
- [implementation support task · TASK-T21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21)
- [task · TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
- [task · TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
- [task · TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
- [separate reference example · EX-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-check)
- [flow · FLOW-F07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F07)
- [handbook · HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
- [handbook · HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
- [handbook · HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
- [public interface convention · Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
- [planned module · MOD-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-check)
- [plan · SEC-hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-hardening)
- [plan · SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
- [plan · SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
- [plan · SEC-security](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-security)
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
- [schematic · SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
- [schematic · SC-SC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC06)
- [schematic · SC-SC08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC08)
- [schematic · SC-SC09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC09)
- [schematic · SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
- [schematic · SC-SC12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC12)
- [schematic · SC-SC15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC15)
- [schematic · SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
- [schematic · SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
- [schematic · SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
- [schematic · SC-SC22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC22)
- [source · SRC-C14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C14)
- [source · SRC-S05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S05)
- [testing standard · Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
- [progressive context workflow · Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
- [module context scout · CTX-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-check)
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
- [readiness criterion cluster · F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
- [readiness criterion cluster · F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
- [readiness criterion cluster · F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
- [readiness criterion cluster · F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
- [readiness criterion cluster · F6](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6)
- [readiness criterion cluster · F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
- [readiness improvement grouping · R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04)
- [readiness improvement grouping · R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
- [readiness improvement grouping · R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06)
- [readiness improvement grouping · R90-08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-08)
- [readiness improvement grouping · R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
- [readiness improvement grouping · R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
- [Graphify corpus projection · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
- [defensive security convention · Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
- [defensive security convention · RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
- [defensive security convention · Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
- [completion and operational convention · Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
- [completion and operational convention · DONE-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-check)
- [completion and operational convention · Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
- [completion and operational convention · Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
- [completion and operational convention · RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
- [completion and operational convention · RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
- [applied learning · LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
- [applied learning · LRN02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN02)
- [applied learning · LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
- [applied learning · LRN06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06)
- [applied learning · LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
- [applied learning · LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12)
- [applied learning · LRN14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14)
- [diary evidence source · DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
- [diary evidence source · DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
- [diary evidence source · DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
- [diary evidence source · DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
- [diary evidence source · DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
- [diary evidence source · DR07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR07)
- [diary evidence source · DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
- [diary evidence source · DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
- [diary evidence source · DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
- [diary evidence source · DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
