# HEE3-MOD-julia · julia

Evaluation/statistics with internal Evaluate, Cohesion and optional operator submodules.

Primary path: `julia/src/HabitatAnalysis.jl`. Cluster: K4. This is the full planned responsibility; a reference exemplar exercises only its stated subset.

## Anchors and source ownership

[Source comment anchor](file:///var/home/herdr-engineering-engine-v3/julia/src/HabitatAnalysis.jl) · [Atlas module card](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-julia) · [Scoped reference example](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-julia) · [Ultra map master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md)

## Dependencies and integration

Declared build dependencies: none. Declared consumers: none. Runtime request/return paths below do not imply cyclic build imports.

## Adopted readiness obligations

Own bounded Julia analysis, compatible locked test environment and independent numerical oracles. Required Broken/skips cannot qualify; collect stderr and structured logs, and validate language-specific mutation strategy.

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). Binding: `HEE3-READINESS-001` / `e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6`. Adoption is complete; the required engine proof is pending.

| Applicable facet / criterion IDs | Primary contract owners |
| --- | --- |
| [F1 · Vision, requirements and scope](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF1): F1-C01, F1-C02, F1-C03, F1-C04, F1-C05 | task, route, budget, numerical, julia, app |
| [F2 · Architecture, modularity and maintainability](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2): F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [F3 · Public interfaces and integration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3): F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06 | contracts, task, worker, actions, app, numerical, julia, herdr, bash, pi_extension, skills, workflows |
| [F4 · Testing, verification and completion evidence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4): F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07 | check, contracts, app, task, julia |
| [F5 · Security and hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5): F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06 | worker, check, service, actions, app, store, context, bash, pi_extension, skills, workflows |
| [F7 · Traceability, cohesion and controlled change](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7): F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |

Implementation groupings: [R90-01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-01), [R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02), [R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03), [R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04), [R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07), [R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09), [R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10). Original task prerequisites and all 13 completion gates remain controlling; package membership grants no action authority. Shared quality/security/interface rules apply to this module’s actual scope, without creating new runtime responsibilities.

**Resolved design contracts:** [RC01 · First workload, numeric limits and evidence policy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01), [RC02 · Pinned release profile and deployment custody](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02), [RC03 · Versioned control and Rust–Julia contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03), [RC04 · Protected collection, receipts and independent oracles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04), [RC05 · Trusted and adversarial execution profiles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05). Apply the exact selected profile and pending-proof obligations.

Before a change, predict the affected owners and consumer contracts. Afterwards compare actual changes, rerun invalidated checks and retain counter-evidence. Minimum 50 distinct primary-owned cases, zero baseline diagnostics, trustworthy collection and independent parent/release acceptance remain mandatory.

## Public consumer interface contract

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

Convention SHA-256: `0a35d116843a2ebd31280948f10b4c48d341fdb8d37df91488c543bb099ad5f4`. This binding proves which documentation convention is projected, not runtime correctness.

## Fully-complete contract

**HEE3-DONE-julia** · unassessed; acceptance collector unavailable.

[Concrete module outcome, failure controls and admission checklist](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-julia) · [Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)

Completion standard SHA-256: `a8e1536e0691fbaea198e4bd1fd7f135bc8851f6f6e9d55c22e7bf92c7641234`. All 13 gates apply; only an inapplicable subcheck may be explicitly justified.

**Required finished behavior:** Julia analysis produces reproducible routing/cohesion reports with complete denominators, justified tolerances and held-out baseline comparisons.

**Integrated proof scenario:** Evaluate immutable outcomes including failures/cancellations/unknown usage and reproduce report identity and declared uncertainty.

**Fault and benign controls:** Wrong units/shape, data leakage, invalid oracle and out-of-domain operator request fail or use an explicit baseline fallback; in-domain known cases pass.

**Complexity boundary:** Analysis proposes evidence/candidate policy; it never writes grants, leases or task acceptance.

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Daybreak defensive security review

Requested model: `gpt-daybreak-blue-latest`; select through the Codex `/model` picker and verify effective identity before the scoped review. Model selection and a reviewer message cannot accept this module.

**HEE3-SEC-julia:** Numerical oracle validity, data leakage, nonfinite/shape/unit input, out-of-domain behavior and reproducibility.

[Model selection, evidence and full security matrix](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB05 security hardening and re-verification](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)

Runtime security qualification is unassessed. Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Applicable patterns, antipatterns and learning triggers

[Reviewed diary learnings, evidence limits and retirement rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Assimilation and verification workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FCorpus%20Assimilation)

| Learning | Use when | Good pattern |
| --- | --- | --- |
| [LRN03 · Put the bound before allocation and the guard before effect](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03) | Before parsing, allocation, fan-out, queue insertion, process launch or an irreversible effect. | Bound input acquisition and concurrency at the boundary; validate identities, capabilities and constraints before dispatch. |
| [LRN05 · Identify the exact source before measuring it](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05) | At re-entry, before evaluation or hardening, and before updating a completion claim. | Bind the actual source/artifact, dependencies, fixtures, toolchain and environment to the run; distinguish intended branch, current checkout and installed artifact. |
| [LRN06 · Declare the coldness and execution layer actually tested](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06) | Before reproducibility, isolation or deployment claims and when crossing host, Toolbx and disposable worker boundaries. | Use explicit environment/toolchain/cache/workspace identities and independent output roots; qualify cold build, service state and installation separately. |
| [LRN08 · Keep meaningful coverage and the zero-warning baseline](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08) | When designing module tests, fixing lint diagnostics or reporting a hardening result. | Retain at least 50 distinct meaningful module-owned cases and zero baseline warnings/errors on every admitted profile; repair code and close material gaps. |
| [LRN09 · Preserve provenance and supersede stale claims](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN09) | When recalling a lesson, updating a roster observation or projecting a changed accepted module. | Retain original source/hash/date/scope, adoption decision and supersession reason; accepted code owns current implemented facts and requirements retain their owner. |
| [LRN10 · Transfer the mechanism and keep prototypes disposable](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10) | When selecting a prototype component, optional neural operator, new helper or inherited working pattern. | Name the local problem, smallest useful mechanism, evidence gained, commitment justified and what can be discarded. |
| [LRN12 · Calibrate the instrument and retain counterexamples](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12) | Before trusting a metric, known-answer fixture, statistical comparison or a confident correction. | Define the measured construct, population, denominator and exclusions; independently check the oracle and record falsifiers, uncertainty and changed conclusions. |

These are scoped design and review obligations. Their engine detectors remain unqualified; source reflections and navigation counts are not implementation evidence.

## Mandatory module testing standard

[Active testing standard and counting rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing): **minimum 50 distinct qualifying cases; zero baseline warnings and errors**, including pedantic Clippy for all admitted Rust targets/profiles.

Bind assimilation, mutation sensitivity, lint coverage and raw evidence to the exact candidate. Property iterations, assertions, retries, lint findings and mutant runs do not multiply case credits. The current observation schema does not establish this qualification.

Standard: `HEE3-TEST-STD-001`; SHA-256: `237c2085f25460543645ec668b626e71d30f5558f56901ae755a3be7c72b86a4`. Full-standard qualification: **unassessed; collector/validator not implemented**.

Module priorities: Rust-Julia request/return framing; locked package/bounds-checked execution; numerical invalid/edge inputs; timeout/cancel and deterministic fixtures.

## Bidirectional contracts

| Flow | Request | Return | Closure |
| --- | --- | --- | --- |
| [F12 numerical → julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F12) | Immutable dataset, schema/units, analysis recipe and resource bound | Report/policy candidate, dataset identity, uncertainty and error | Rust validates identity/ranges; Julia cannot write leases, grants or accepted status. |

## APIs, sockets and commands

[API05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API05), [API09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API09)

| IPC | Endpoint | Custody and recovery |
| --- | --- | --- |
| [IPC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC05) | Julia immutable request/result files plus bounded stdio | On demand, deadline/memory bound; optional warm process requires measured need Missing/truncated/stale/mismatched report rejected; keep approved baseline |
| [IPC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC07) | Pinned subprocess OR explicitly configured inference/gateway endpoint | No default public HTTP listener; optional gateway independently managed One retry owner, actual identity/usage readback and explicit unknown-effect handling |


## Full deployment stems

| Path | Owning module | Scope | Purpose |
| --- | --- | --- | --- |
| julia/Project.toml | julia | original planned path | Julia dependency and compatibility declaration for the bounded Rust–Julia analysis exchange and Julia checks. |
| docs/checks.md | app | original planned path | Shared quality contract spans the planned stack; each module retains its own behavioral checks and disclosed limits. |
| docs/security.md | app | original planned path | Integrated threat model records existing authority, data, execution and verification boundaries across the planned stack. |
| julia/src/HabitatAnalysis.jl | julia | original planned path | Existing Julia analysis entrypoint receives immutable outcomes and returns advisory reports through the numerical boundary. |
| evaluation/tasks/ | julia | original planned path | Held-out workload corpus compares routing and cohort outcomes with fixed acceptance and complete cost accounting. |
| julia/src/Evaluate.jl | julia | original planned path | Planned internal Julia evaluation source retains all outcome classes, uncertainty, missingness and censoring policy. |
| evaluation/report.md | julia | original planned path | Evaluation report compares declared baselines and challengers from immutable outcomes and qualified acceptance evidence. |
| julia/src/Cohesion.jl | julia | original planned path | Planned internal Julia cohesion analysis compares coordination outcomes and relationship coverage. |
| evaluation/cohorts/ | cohort | original planned path | Cohort scenarios evaluate shared objectives, resource conflicts, joins, rework and conserved allocations. |
| deploy/ | app | original planned path | Shared package and deployment corpus covers the existing stack, managed lifecycle integration, backup, upgrade and rollback. |
| tests/isolation/ | worker | original planned path | Isolation fixtures cover candidate code, protected control/evidence paths, Julia hooks, credentials and process cleanup. |
| tests/faults/ | app | original planned path | Integrated critical-failure matrix crosses the planned stack and preserves intended-refusal, invalid-fixture and unmeasured distinctions. |
| docs/qualification.md | app | original planned path | Shared qualification record binds each claim to the exact integrated subject, fault, evidence and disclosed scope. |
| docs/operations.md | app | original planned path | Operator runbook joins packaging, lifecycle, retention, restore reconciliation and compatible integration updates. |
| evaluation/pilot/ | app | original planned path | Commissioned real-work pilot evaluates scoped end-to-end outcomes and release targets without automatically enabling optional lanes. |
| README.md | app | original planned path | Release entrypoint documents the supported existing stack, commands, versions, limitations and evidence. |
| docs/atlas.json | app | original planned path | Deployment atlas projection describes existing module ownership and contracts; the planning spine remains design authority. |
| docs/security-findings.md | app | original planned path | Integrated finding dispositions record reproductions, fixes, residual risks and exact packaged subjects across existing modules. |
| tests/security/ | app | original planned path | Security qualification corpus tests the integrated threat model with independent collectors and explicit excluded or unmeasured scope. |
| evidence/release/ | app | original planned path | Release evidence aggregates exact-subject qualification receipts; aggregation cannot create or replace module proof. |
| numerical-workers/ | numerical | original planned path | Conditional numerical worker packaging is owned by the existing numerical boundary; no new mandatory runtime module is introduced. |
| evaluation/runtime-compatibility.md | numerical | original planned path | Optional Julia/tch/LibTorch or external-service comparison records the concrete model, device, format, cost and cancellation contract. |
| julia/experiments/operators/ | julia | original planned path | Conditional operator experiments compare a declared function/field domain with simple predictors and full scheduling/resource costs. |
| evaluation/operator-card.md | julia | original planned path | Optional operator evidence declares domain, held-out tests, fallback and actual decision value; rejection preserves baseline routing. |
| evaluation/tasks/manifest-v1.schema.json | check | authored candidate / T01 | Frozen development manifest and full-suite schema; actual held-out materialization remains due before T12 evaluation. |
| evaluation/tasks/suite-v1.schema.json | check | authored candidate / T01 | Frozen development manifest and full-suite schema; actual held-out materialization remains due before T12 evaluation. |
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
| julia/bin/analysis.jl | julia | authored candidate / T21 | Bounded immutable Julia analysis exchange or source-bound numerical controls; no policy, Store write or module admission. |
| julia/test/analysis.jl | julia | authored candidate / T21 | Bounded immutable Julia analysis exchange or source-bound numerical controls; no policy, Store write or module admission. |

## Delivery, verification and hardening contracts

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

### T24 · Evaluate neural operators against simple predictors

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T24)

Dependencies: T12, T22. Atlas task state: `idle`.

**Acceptance:** A defined function/field domain, dataset and decision use case exist. Held-out time/topology/domain tests compare prediction and actual scheduling value against simple baselines, including all resource cost. Out-of-domain detection and fallback are tested; rejection leaves baseline routing unchanged.

**Validator:** Proposed, not implemented: habitat-engine-check case T24 --fixture-root <disposable-fixtures>

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

The numerical gate is not a trained neural operator. PyTorch/LibTorch, artifact provenance, accelerator determinism, held-out generalization and promotion remain optional workload-gated work.

## Full return-anchor register

- [module cluster · CLU-K4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K4)
- [contributing codebase · CODE-CB01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB01)
- [contributing codebase · CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
- [contributing codebase · CODE-CB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB05)
- [contributing codebase · CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
- [implementation support task · TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
- [task · TASK-T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)
- [task · TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
- [task · TASK-T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)
- [task · TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
- [task · TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
- [task · TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
- [task · TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
- [task · TASK-T21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21)
- [task · TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
- [task · TASK-T23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T23)
- [task · TASK-T24](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T24)
- [task · TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
- [task · TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
- [task · TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
- [separate reference example · EX-julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-julia)
- [flow · FLOW-F12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F12)
- [handbook · HB-compatibility-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-compatibility-map)
- [handbook · HB-flow-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-flow-map)
- [handbook · HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
- [API · API-API05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API05)
- [API · API-API09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API09)
- [IPC · IPC-IPC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC05)
- [IPC · IPC-IPC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC07)
- [public interface convention · Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
- [planned module · MOD-julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-julia)
- [plan · SEC-excellence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-excellence)
- [plan · SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
- [plan · SEC-numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-numerical)
- [plan · SEC-routing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-routing)
- [plan · SEC-swarm](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-swarm)
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
- [schematic · SC-SC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC01)
- [schematic · SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
- [schematic · SC-SC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC03)
- [schematic · SC-SC14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC14)
- [schematic · SC-SC15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC15)
- [schematic · SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
- [schematic · SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
- [schematic · SC-SC22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC22)
- [source · SRC-S05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S05)
- [source · SRC-S13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S13)
- [source · SRC-S14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S14)
- [testing standard · Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
- [progressive context workflow · Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
- [module context scout · CTX-julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-julia)
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
- [adopted readiness convention · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
- [readiness criterion cluster · F1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF1)
- [readiness criterion cluster · F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
- [readiness criterion cluster · F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
- [readiness criterion cluster · F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
- [readiness criterion cluster · F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
- [readiness criterion cluster · F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
- [readiness improvement grouping · R90-01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-01)
- [readiness improvement grouping · R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02)
- [readiness improvement grouping · R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
- [readiness improvement grouping · R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04)
- [readiness improvement grouping · R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
- [readiness improvement grouping · R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
- [readiness improvement grouping · R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
- [Graphify corpus projection · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
- [defensive security convention · Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
- [defensive security convention · RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
- [defensive security convention · Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
- [completion and operational convention · Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
- [completion and operational convention · DONE-julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-julia)
- [completion and operational convention · Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
- [completion and operational convention · Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
- [completion and operational convention · RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
- [completion and operational convention · RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
- [applied learning · LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03)
- [applied learning · LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
- [applied learning · LRN06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06)
- [applied learning · LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
- [applied learning · LRN09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN09)
- [applied learning · LRN10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10)
- [applied learning · LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12)
- [diary evidence source · DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
- [diary evidence source · DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
- [diary evidence source · DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
- [diary evidence source · DR04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR04)
- [diary evidence source · DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
- [diary evidence source · DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
- [diary evidence source · DR07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR07)
- [diary evidence source · DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
- [diary evidence source · DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
- [diary evidence source · DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
- [diary evidence source · DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
- [diary evidence source · DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
