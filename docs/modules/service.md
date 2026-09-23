# HEE3-MOD-service · service

Useful probes and actions delegated to the registered lifecycle owner.

Primary path: `src/service.rs`. Cluster: K5. This is the full planned responsibility; a reference exemplar exercises only its stated subset.

## Anchors and source ownership

[Source comment anchor](file:///var/home/herdr-engineering-engine-v3/src/service.rs) · [Atlas module card](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-service) · [Scoped reference example](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-service) · [Ultra map master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md)

## Dependencies and integration

Declared build dependencies: contracts. Declared consumers: app. Runtime request/return paths below do not imply cyclic build imports.

## Adopted readiness obligations

Own registry/lifecycle bridges to existing systemd/Podman authorities. Verify answering interfaces, expected inventory, residual processes and cleanup; a listener or process is insufficient health proof.

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). Binding: `HEE3-READINESS-001` / `e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6`. Adoption is complete; the required engine proof is pending.

| Applicable facet / criterion IDs | Primary contract owners |
| --- | --- |
| [F2 · Architecture, modularity and maintainability](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2): F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [F3 · Public interfaces and integration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3): F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06 | contracts, task, worker, actions, app, numerical, julia, herdr, bash, pi_extension, skills, workflows |
| [F4 · Testing, verification and completion evidence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4): F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07 | check, contracts, app, task, julia |
| [F5 · Security and hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5): F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06 | worker, check, service, actions, app, store, context, bash, pi_extension, skills, workflows |
| [F6 · Build, deployment, migration and operations](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6): F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08 | app, store, recovery, service, notify, check |
| [F7 · Traceability, cohesion and controlled change](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7): F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |

Implementation groupings: [R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02), [R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06), [R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07), [R90-08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-08), [R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09), [R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10). Original task prerequisites and all 13 completion gates remain controlling; package membership grants no action authority. Shared quality/security/interface rules apply to this module’s actual scope, without creating new runtime responsibilities.

**Resolved design contracts:** [RC02 · Pinned release profile and deployment custody](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02), [RC03 · Versioned control and Rust–Julia contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03), [RC04 · Protected collection, receipts and independent oracles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04), [RC05 · Trusted and adversarial execution profiles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05), [RC06 · SQLite, migration freeze and recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06). Apply the exact selected profile and pending-proof obligations.

Before a change, predict the affected owners and consumer contracts. Afterwards compare actual changes, rerun invalidated checks and retain counter-evidence. Minimum 50 distinct primary-owned cases, zero baseline diagnostics, trustworthy collection and independent parent/release acceptance remain mandatory.

## Public consumer interface contract

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

Convention SHA-256: `0a35d116843a2ebd31280948f10b4c48d341fdb8d37df91488c543bb099ad5f4`. This binding proves which documentation convention is projected, not runtime correctness.

## Fully-complete contract

**HEE3-DONE-service** · unassessed; acceptance collector unavailable.

[Concrete module outcome, failure controls and admission checklist](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-service) · [Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)

Completion standard SHA-256: `a8e1536e0691fbaea198e4bd1fd7f135bc8851f6f6e9d55c22e7bf92c7641234`. All 13 gates apply; only an inapplicable subcheck may be explicitly justified.

**Required finished behavior:** Registered binaries, daemons, libraries and numerical services expose useful health and delegated owner-controlled lifecycle with effect readback.

**Integrated proof scenario:** Inspect the registered owner, perform an admitted bounded probe/action and reconcile its durable operation/job identity.

**Fault and benign controls:** Unauthorized action, wrong unit, useless-but-listening service and lost reply never imply healthy completion; valid useful probe succeeds.

**Complexity boundary:** Delegate to established lifecycle owners such as systemd/Podman; do not introduce a competing supervisor.

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Daybreak defensive security review

Requested model: `gpt-daybreak-blue-latest`; select through the Codex `/model` picker and verify effective identity before the scoped review. Model selection and a reviewer message cannot accept this module.

**HEE3-SEC-service:** Unauthorized lifecycle effects, wrong owner/unit, useful-health spoofing and uncertain replay.

[Model selection, evidence and full security matrix](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB05 security hardening and re-verification](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)

Runtime security qualification is unassessed. Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Applicable patterns, antipatterns and learning triggers

[Reviewed diary learnings, evidence limits and retirement rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Assimilation and verification workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FCorpus%20Assimilation)

| Learning | Use when | Good pattern |
| --- | --- | --- |
| [LRN06 · Declare the coldness and execution layer actually tested](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06) | Before reproducibility, isolation or deployment claims and when crossing host, Toolbx and disposable worker boundaries. | Use explicit environment/toolchain/cache/workspace identities and independent output roots; qualify cold build, service state and installation separately. |
| [LRN07 · Preserve producer verdict and process custody](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07) | Before shell chaining, retrying a job, cancelling a process or presenting progress as success. | Capture the decisive producer status and raw output, keep explicit job/process/session identities and check existing obligations before retry or cleanup. |
| [LRN08 · Keep meaningful coverage and the zero-warning baseline](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08) | When designing module tests, fixing lint diagnostics or reporting a hardening result. | Retain at least 50 distinct meaningful module-owned cases and zero baseline warnings/errors on every admitted profile; repair code and close material gaps. |
| [LRN10 · Transfer the mechanism and keep prototypes disposable](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10) | When selecting a prototype component, optional neural operator, new helper or inherited working pattern. | Name the local problem, smallest useful mechanism, evidence gained, commitment justified and what can be discarded. |
| [LRN13 · Test useful effects and declared persistence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13) | Before calling an integration connected, a deployment useful or a restore complete. | Send a real bounded request through the supported seam and verify useful return/readback; declare durable state and independently test restoration. |

These are scoped design and review obligations. Their engine detectors remain unqualified; source reflections and navigation counts are not implementation evidence.

## Mandatory module testing standard

[Active testing standard and counting rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing): **minimum 50 distinct qualifying cases; zero baseline warnings and errors**, including pedantic Clippy for all admitted Rust targets/profiles.

Bind assimilation, mutation sensitivity, lint coverage and raw evidence to the exact candidate. Property iterations, assertions, retries, lint findings and mutant runs do not multiply case credits. The current observation schema does not establish this qualification.

Standard: `HEE3-TEST-STD-001`; SHA-256: `237c2085f25460543645ec668b626e71d30f5558f56901ae755a3be7c72b86a4`. Full-standard qualification: **unassessed; collector/validator not implemented**.

Module priorities: admission and lifecycle ownership; binary/daemon/library/operator boundaries; health/readback and failure isolation; restart/stop and resource cleanup.

## Bidirectional contracts

| Flow | Request | Return | Closure |
| --- | --- | --- | --- |
| [F14 roster → service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F14) | Explicit discovery, cached-state query or specifically approved bounded probe; lifecycle mutation uses F20 | Timestamped useful-health and actual owner/state observation | A live PID cannot override a failed request-path probe; discovery grants no control. |
| [F20 actions → service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F20) | Caller-bound explicit grant, registered service/unit/action and durable operation key | Lifecycle owner/job identity, observed result/useful health or unknown effect | Discovery grants no mutation. Lost manager response requires readback; no competing lifecycle owner. |

## APIs, sockets and commands

[API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01), [API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10), [API07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API07)

| IPC | Endpoint | Custody and recovery |
| --- | --- | --- |
| [IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01) | $XDG_RUNTIME_DIR/habitat-engine/control.sock | Acquire single-instance custody before migration/recovery; bind after ready. Never unlink an unproven live socket. Absent/incompatible -> typed failure; reconnect with engine cursor or snapshot+cursor on expiry |
| [IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04) | Separate inherited task-bound tool pipe pair OR selected maintained native tool channel | One admitted attempt/version tuple; old reload handler/channel generation revoked before effects Lost reply -> scoped action readback; denied action stays denied; unsupported bridge blocks tool exposure |
| [IPC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC06) | systemctl --user argv; manager-selected IPC underneath | Manager owns units/Quadlet; engine stores observations, not competing supervision Read unit/job identity and useful probe before replay; unknown result remains visible |

| Action | CLI projection | Effect | Return/readback |
| --- | --- | --- | --- |
| [service.inspect](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-service.inspect) | habitat-engine service inspect <id> --json | read | Lifecycle owner, operation record if requested, and cached liveness/readiness/useful-progress observations No probe unless explicitly requested; current unit state alone does not prove an ambiguous earlier action result |
| [service.probe](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-service.probe) | habitat-engine service probe <id> --json | bounded declared probe | Useful-health observation, effect/cost disclosure and age Repeat only under probe effect policy; no hidden provider spend |
| [service.action](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-service.action) | habitat-engine service action <id> <action> --json | managed lifecycle effect | Operation/job identity and observed state; useful health separate service.inspect/probe; read back before replaying uncertain external effect |


## Full deployment stems

| Path | Owning module | Scope | Purpose |
| --- | --- | --- | --- |
| Cargo.toml | app | original planned path | Shared Rust package composition, dependency and feature policy for the existing Rust module set. |
| rust-toolchain.toml | app | original planned path | Pinned compiler and tooling tuple for the existing Rust modules; no additional package boundary. |
| docs/checks.md | app | original planned path | Shared quality contract spans the planned stack; each module retains its own behavioral checks and disclosed limits. |
| docs/security.md | app | original planned path | Integrated threat model records existing authority, data, execution and verification boundaries across the planned stack. |
| src/service.rs | service | original planned path | Owns useful probes and explicitly delegated lifecycle actions for existing registered service owners. |
| config/services.toml | roster | original planned path | Reviewed service roster input follows the same admitted-revision import/export rules as models and agents. |
| src/service/systemd.rs | service | original planned path | Internal systemd adapter delegates only scoped lifecycle actions; it is part of the existing service module. |
| deploy/ | app | original planned path | Shared package and deployment corpus covers the existing stack, managed lifecycle integration, backup, upgrade and rollback. |
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
| src/service/local_probe.rs | service | authored candidate / T13 | Inspect-only useful service observations and admitted-revision-bound preparation/local execution through existing owners; no lifecycle grant. |
| src/service/observations.rs | service | authored candidate / T13 | Inspect-only useful service observations and admitted-revision-bound preparation/local execution through existing owners; no lifecycle grant. |
| src/service/probe.rs | service | authored candidate / T13 | Inspect-only useful service observations and admitted-revision-bound preparation/local execution through existing owners; no lifecycle grant. |
| src/service/profile.rs | service | authored candidate / T13 | Inspect-only useful service observations and admitted-revision-bound preparation/local execution through existing owners; no lifecycle grant. |
| tests/t13_local_probe.rs | service | authored candidate / T13 | Source-bound useful observation, authority, stale-revision and actual local probe controls; task evidence, not module qualification. |
| tests/t13_probe.rs | service | authored candidate / T13 | Source-bound useful observation, authority, stale-revision and actual local probe controls; task evidence, not module qualification. |
| tests/t13_service.rs | service | authored candidate / T13 | Source-bound useful observation, authority, stale-revision and actual local probe controls; task evidence, not module qualification. |

## Delivery, verification and hardening contracts

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

## Closure and rollout discipline

Apply the existing plan → execute → observe → verify → repair/escalate → verify → close loop. Before deployment, qualify the assembled candidate and relevant T25/T26 checks, T17 failures, T27 security finding closure and T18 backup/upgrade/rollback obligations in their declared scope. T19/T20 govern pilot and release; these links do not mark them complete. Package and configuration identity must match observed installation and useful readback. Preserve cancellation, uncertain effects, cleanup and rollback obligations separately.

A successful compile or lint of comment-only stubs is not module verification. The Rust pedantic-Clippy/test plan, Julia checks, transport/fault cases and security reviews remain governed by their actual task criteria. [The evidence and update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) refreshes all corpus views after retained observations are admitted.

## Remaining reference-example limits

The example constructs plans and evaluates observations; it does not invoke systemd, establish unit custody or qualify replay/readback of uncertain effects.

## Full return-anchor register

- [module cluster · CLU-K5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K5)
- [contributing codebase · CODE-CB01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB01)
- [contributing codebase · CODE-CB11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB11)
- [contributing codebase · CODE-CB12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB12)
- [task · TASK-T13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T13)
- [task · TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
- [task · TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
- [task · TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
- [task · TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
- [task · TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
- [task · TASK-T23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T23)
- [task · TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
- [task · TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
- [task · TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
- [separate reference example · EX-service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-service)
- [flow · FLOW-F14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F14)
- [flow · FLOW-F20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F20)
- [handbook · HB-action-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-action-map)
- [handbook · HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
- [handbook · HB-socket-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-socket-map)
- [API · API-API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01)
- [API · API-API07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API07)
- [API · API-API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10)
- [action · ACT-service.action](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-service.action)
- [action · ACT-service.inspect](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-service.inspect)
- [action · ACT-service.probe](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-service.probe)
- [IPC · IPC-IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01)
- [IPC · IPC-IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04)
- [IPC · IPC-IPC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC06)
- [public interface convention · Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
- [planned module · MOD-service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-service)
- [plan · SEC-api-sockets](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-api-sockets)
- [plan · SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
- [plan · SEC-rosters](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-rosters)
- [plan · SEC-runtime](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-runtime)
- [plan · SEC-security](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-security)
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
- [schematic · SC-SC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC07)
- [schematic · SC-SC16](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC16)
- [schematic · SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
- [schematic · SC-SC20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC20)
- [schematic · SC-SC23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC23)
- [schematic · SC-SC24](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC24)
- [source · SRC-D07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-D07)
- [source · SRC-D08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-D08)
- [source · SRC-H05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-H05)
- [testing standard · Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
- [progressive context workflow · Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
- [module context scout · CTX-service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-service)
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
- [resolved module contract · RC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06)
- [adopted readiness convention · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
- [readiness criterion cluster · F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
- [readiness criterion cluster · F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
- [readiness criterion cluster · F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
- [readiness criterion cluster · F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
- [readiness criterion cluster · F6](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6)
- [readiness criterion cluster · F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
- [readiness improvement grouping · R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02)
- [readiness improvement grouping · R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06)
- [readiness improvement grouping · R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
- [readiness improvement grouping · R90-08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-08)
- [readiness improvement grouping · R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
- [readiness improvement grouping · R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
- [Graphify corpus projection · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
- [defensive security convention · Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
- [defensive security convention · RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
- [defensive security convention · Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
- [completion and operational convention · Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
- [completion and operational convention · DONE-service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-service)
- [completion and operational convention · Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
- [completion and operational convention · Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
- [completion and operational convention · RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
- [completion and operational convention · RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
- [applied learning · LRN06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06)
- [applied learning · LRN07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07)
- [applied learning · LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
- [applied learning · LRN10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10)
- [applied learning · LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13)
- [diary evidence source · DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
- [diary evidence source · DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
- [diary evidence source · DR04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR04)
- [diary evidence source · DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
- [diary evidence source · DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
- [diary evidence source · DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
- [diary evidence source · DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
- [diary evidence source · DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
- [diary evidence source · DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
