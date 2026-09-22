# HEE3-MOD-workflows · workflows

Finite typed procedure definitions using the existing task/cohort scheduler.

Primary path: `workflows/README.stub.md`. Cluster: K3. This is the full planned responsibility; a reference exemplar exercises only its stated subset.

## Anchors and source ownership

[Source comment anchor](file:///var/home/herdr-engineering-engine-v3/workflows/README.stub.md) · [Atlas module card](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-workflows) · [Scoped reference example](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-workflows) · [Ultra map master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md)

## Dependencies and integration

Declared build dependencies: actions. Declared consumers: none. Runtime request/return paths below do not imply cyclic build imports.

## Adopted readiness obligations

Own composition of admitted commands and completion/update procedures. Keep single-writer shared artifacts, exact return obligations and parent acceptance; do not invent a parallel scheduler.

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). Binding: `HEE3-READINESS-001` / `e774f8984c7e39e85b156289fac809bded6df80ba3647cfb2024c69856447b94`. Adoption is complete; the required engine proof is pending.

| Applicable facet / criterion IDs | Primary contract owners |
| --- | --- |
| [F2 · Architecture, modularity and maintainability](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2): F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [F3 · Public interfaces and integration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3): F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06 | contracts, task, worker, actions, app, numerical, julia, herdr, bash, pi_extension, skills, workflows |
| [F4 · Testing, verification and completion evidence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4): F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07 | check, contracts, app, task, julia |
| [F5 · Security and hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5): F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06 | worker, check, service, actions, app, store, context, bash, pi_extension, skills, workflows |
| [F7 · Traceability, cohesion and controlled change](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7): F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |

Implementation groupings: [R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03), [R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07), [R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09), [R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10). Original task prerequisites and all 13 completion gates remain controlling; package membership grants no action authority. Shared quality/security/interface rules apply to this module’s actual scope, without creating new runtime responsibilities.

**Resolved design contracts:** [RC01 · First workload, numeric limits and evidence policy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01), [RC02 · Pinned release profile and deployment custody](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02), [RC03 · Versioned control and Rust–Julia contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03), [RC04 · Protected collection, receipts and independent oracles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04), [RC05 · Trusted and adversarial execution profiles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05), [RC06 · SQLite, migration freeze and recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06). Apply the exact selected profile and pending-proof obligations.

Before a change, predict the affected owners and consumer contracts. Afterwards compare actual changes, rerun invalidated checks and retain counter-evidence. Minimum 50 distinct primary-owned cases, zero baseline diagnostics, trustworthy collection and independent parent/release acceptance remain mandatory.

## Public consumer interface contract

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

## Fully-complete contract

**HEE3-DONE-workflows** · unassessed; acceptance collector unavailable.

[Concrete module outcome, failure controls and admission checklist](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-workflows) · [Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)

Completion standard SHA-256: `a36374a796816c1ea4c85c3292d376942fe9da226d714012b87da2588f598594`. All 13 gates apply; only an inapplicable subcheck may be explicitly justified.

**Required finished behavior:** Finite versioned procedures use existing task/cohort scheduling and retain stable step identities, bounded retries and independent parent acceptance.

**Integrated proof scenario:** Validate and execute an admitted DAG, interrupt it after an effect, read back uncertain state and resume to a repaired, reverified parent join.

**Fault and benign controls:** Cycles, unknown steps, missing child, budget exhaustion and failed final verifier prevent success; valid settled steps are not repeated.

**Complexity boundary:** One procedure definition projects to Just/CLI/host surfaces; no parallel scheduler or automatic runner compatibility claim.

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Daybreak defensive security review

Requested model: `gpt-daybreak-blue-latest`; select through the Codex `/model` picker and verify effective identity before the scoped review. Model selection and a reviewer message cannot accept this module.

**HEE3-SEC-workflows:** Injected steps, cycles, retries after uncertain effects, source/wrapper drift and independent final verification.

[Model selection, evidence and full security matrix](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB05 security hardening and re-verification](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)

Runtime security qualification is unassessed. Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Applicable patterns, antipatterns and learning triggers

[Reviewed diary learnings, evidence limits and retirement rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Assimilation and verification workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FCorpus%20Assimilation)

| Learning | Use when | Good pattern |
| --- | --- | --- |
| [LRN01 · Earn the verdict from the complete subject](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01) | Before quoting a result, admitting a module, closing a parent or presenting done in a client. | Report a typed scoped verdict with exact candidate, admitted criteria, required checks, raw evidence, gaps and a counter-evidence pointer. |
| [LRN02 · Qualify the detector before trusting its scan](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN02) | Before a new or changed detector is allowed to establish a clean result. | Bind an intended-fault fixture and a benign mirror to the exact production check path, with a stable fault definition and expected diagnostic. |
| [LRN04 · One semantic owner across every consumer surface](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04) | When adding a CLI, tool, Pi, Bash, workflow or cross-language entry point. | Use one typed owner and thin projections; pass source, environment, task and caller identities explicitly and preserve the return contract. |
| [LRN07 · Preserve producer verdict and process custody](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07) | Before shell chaining, retrying a job, cancelling a process or presenting progress as success. | Capture the decisive producer status and raw output, keep explicit job/process/session identities and check existing obligations before retry or cleanup. |
| [LRN08 · Keep meaningful coverage and the zero-warning baseline](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08) | When designing module tests, fixing lint diagnostics or reporting a hardening result. | Retain at least 50 distinct meaningful module-owned cases and zero baseline warnings/errors on every admitted profile; repair code and close material gaps. |
| [LRN09 · Preserve provenance and supersede stale claims](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN09) | When recalling a lesson, updating a roster observation or projecting a changed accepted module. | Retain original source/hash/date/scope, adoption decision and supersession reason; accepted code owns current implemented facts and requirements retain their owner. |
| [LRN10 · Transfer the mechanism and keep prototypes disposable](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10) | When selecting a prototype component, optional neural operator, new helper or inherited working pattern. | Name the local problem, smallest useful mechanism, evidence gained, commitment justified and what can be discarded. |
| [LRN11 · Make specialist threads cohere through explicit ownership](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11) | At delegation, handoff, shared-resource access and every meaningful task/phase boundary. | Give each thread one bounded objective, subject/brief revision, resource/write ownership, dependencies, limits, return schema and cleanup obligations. |
| [LRN13 · Test useful effects and declared persistence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13) | Before calling an integration connected, a deployment useful or a restore complete. | Send a real bounded request through the supported seam and verify useful return/readback; declare durable state and independently test restoration. |
| [LRN14 · Make learning earn its maintenance cost](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14) | When adding another rule, helper, generated note, verification stage or repeated context packet. | Prefer one existing owner and reusable entry point. Record the problem, placement, footprint, verification cost, prevented failure, false-positive/rework cost and retirement condition. |

These are scoped design and review obligations. Their engine detectors remain unqualified; source reflections and navigation counts are not implementation evidence.

## Mandatory module testing standard

[Active testing standard and counting rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing): **minimum 50 distinct qualifying cases; zero baseline warnings and errors**, including pedantic Clippy for all admitted Rust targets/profiles.

Bind assimilation, mutation sensitivity, lint coverage and raw evidence to the exact candidate. Property iterations, assertions, retries, lint findings and mutant runs do not multiply case credits. The current observation schema does not establish this qualification.

Standard: `HEE3-TEST-STD-001`; SHA-256: `14a29dad1d089bc529ff9a132e5f80d5195e81aba2c2c0a07229e1c7c8960b09`. Full-standard qualification: **unassessed; collector/validator not implemented**.

Module priorities: DAG join and dependency completion; partial failure and skipped stage refusal; fanout cancellation and ownership; complete evidence return and no duplicate effects.

## Bidirectional contracts

| Flow | Request | Return | Closure |
| --- | --- | --- | --- |
| [F19 workflows → task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F19) | Versioned finite action composition and expanded child identity, mediated by actions/app with current authority | Step results, outstanding obligations, repair/stop and parent evidence | Every required child and final verifier contributes to the one parent closure path. |

## APIs, sockets and commands

Internal module: external access is mediated by its declared consumers.

## Full deployment stems

| Path | Owning module | Scope | Purpose |
| --- | --- | --- | --- |
| docs/checks.md | app | original planned path | Shared quality contract spans the planned stack; each module retains its own behavioral checks and disclosed limits. |
| docs/security.md | app | original planned path | Integrated threat model records existing authority, data, execution and verification boundaries across the planned stack. |
| schemas/actions/ | actions | original planned path | Generated action schemas support typed CLI, Unix and integration projections without becoming another authority. |
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

## Delivery, verification and hardening contracts

### T10 · Account for whole-task cost and bounded fallback

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)

Dependencies: T04, T08, T09. Atlas task state: `idle`.

**Acceptance:** Concurrent reservations cannot exceed configured bounds; retries/checking/compaction are included; unknown usage is visible; hard-ceiling tasks reject incompatible accounting; fallback cannot cross privacy or tool constraints.

**Validator:** Proposed, not implemented: habitat-engine-check case T10 --fixture-root <disposable-fixtures>

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

### T28 · Expose one authoritative action catalogue to CLI, API and LLM tools

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T28)

Dependencies: T03, T05, T06, T09, T25, T26. Atlas task state: `idle`.

**Acceptance:** One action definition drives CLI/Unix schemas and one selected LLM-tool adapter. Bounded discovery/inspection/preview, scoped invocation, identity/idempotency and evidence readback work end to end. Denial remains denial across transports; untrusted workers receive no operator socket; generated projections cannot drift. task.get and analysis.get support their own principal-scoped admission-key readback; service.inspect can query the lifecycle operation key without confusing current health with historic effect. events.subscribe without a cursor provides a bounded coherent bootstrap snapshot plus high-water cursor. Distinguish request correlation from durable idempotency. Base Pi RPC does not supply the engine tool callback path; qualify the actual inherited channel or maintained bridge. Template-produced requests preserve literal typed values at the action boundary; model-emitted risk or approval fields never confer effect authority.

**Validator:** Proposed, not implemented: habitat-engine-check case T28 --fixture-root <disposable-fixtures>

### T29 · Qualify modular Bash, Pi, skill and workflow integration packages

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)

Dependencies: T21, T22, T25, T26, T28. Atlas task state: `idle`.

**Acceptance:** Thin updatable packages share action/procedure identity and preserve authority, arguments, producer exit status, cancellation and evidence. Pi errors/parallel calls/reload and effective tool-set readback are qualified; skills disclose bounded context. Two typed compositions close their real return paths. Version switching preserves active attempts and rollback; no separate scheduler or silent self-update. Keep the admitted package/protocol tuple pinned through every active attempt. Revoked/reloaded tool handlers cannot retain effect authority. No custom Pi-host glue is selected until its concrete packaging need is established. Each integration renders its owned template once, then passes literal typed data rather than copying/re-templating procedure bodies. Focused fixtures include braces, quotes, newlines, dollar signs and leading dashes, with exact final argv/JSON and producer-status checks. Preserve source and relationship coverage with whole-workflow context accounting. Source-change reports cannot advance accepted compatibility/source baselines.

**Validator:** Proposed, not implemented: habitat-engine-check case T29 --fixture-root <disposable-fixtures>

## Closure and rollout discipline

Apply the existing plan → execute → observe → verify → repair/escalate → verify → close loop. Before deployment, qualify the assembled candidate and relevant T25/T26 checks, T17 failures, T27 security finding closure and T18 backup/upgrade/rollback obligations in their declared scope. T19/T20 govern pilot and release; these links do not mark them complete. Package and configuration identity must match observed installation and useful readback. Preserve cancellation, uncertain effects, cleanup and rollback obligations separately.

A successful compile or lint of comment-only stubs is not module verification. The Rust pedantic-Clippy/test plan, Julia checks, transport/fault cases and security reviews remain governed by their actual task criteria. [The evidence and update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) refreshes all corpus views after retained observations are admitted.

## Remaining reference-example limits

Only a read-action subset is demonstrated. Durable reservations, task/cohort submission, resumption and integration acceptance belong to the production scheduler.

## Full return-anchor register

- [module cluster · CLU-K3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K3)
- [contributing codebase · CODE-CB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB04)
- [contributing codebase · CODE-CB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB05)
- [contributing codebase · CODE-CB11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB11)
- [task · TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
- [task · TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
- [task · TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
- [task · TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
- [task · TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
- [task · TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
- [task · TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
- [task · TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
- [task · TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
- [task · TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
- [task · TASK-T28](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T28)
- [task · TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
- [separate reference example · EX-workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-workflows)
- [flow · FLOW-F19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F19)
- [handbook · HB-compatibility-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-compatibility-map)
- [handbook · HB-flow-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-flow-map)
- [public interface convention · Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
- [planned module · MOD-workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-workflows)
- [plan · SEC-economy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-economy)
- [plan · SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
- [plan · SEC-threads](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-threads)
- [plan · SEC-toolchain](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-toolchain)
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
- [schematic · SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
- [schematic · SC-SC13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC13)
- [schematic · SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
- [schematic · SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
- [schematic · SC-SC21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC21)
- [source · SRC-B03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-B03)
- [source · SRC-B08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-B08)
- [source · SRC-B14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-B14)
- [testing standard · Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
- [progressive context workflow · Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
- [module context scout · CTX-workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-workflows)
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
- [readiness criterion cluster · F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
- [readiness improvement grouping · R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
- [readiness improvement grouping · R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
- [readiness improvement grouping · R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
- [readiness improvement grouping · R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
- [Graphify corpus projection · Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
- [defensive security convention · Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
- [defensive security convention · RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
- [defensive security convention · Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
- [completion and operational convention · Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
- [completion and operational convention · DONE-workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-workflows)
- [completion and operational convention · Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
- [completion and operational convention · Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
- [completion and operational convention · RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
- [completion and operational convention · RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
- [documentation procedure · RB01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB01)
- [documentation procedure · RB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB02)
- [applied learning · LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
- [applied learning · LRN02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN02)
- [applied learning · LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
- [applied learning · LRN07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07)
- [applied learning · LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
- [applied learning · LRN09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN09)
- [applied learning · LRN10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10)
- [applied learning · LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11)
- [applied learning · LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13)
- [applied learning · LRN14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14)
- [diary evidence source · DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
- [diary evidence source · DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
- [diary evidence source · DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
- [diary evidence source · DR04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR04)
- [diary evidence source · DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
- [diary evidence source · DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
- [diary evidence source · DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
- [diary evidence source · DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
- [diary evidence source · DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
- [diary evidence source · DR12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR12)
- [diary evidence source · DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
