# Module completion: concrete definitions and verification standard

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

A module is fully complete only for an explicit version/profile and exact admitted subject when every applicable mandatory gate and owning atlas criterion passes with current evidence, non-applicability is justified without waiving requirements, independent acceptance is retained and corpus convergence is verified. Future ideas are outside that bounded claim.

[Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start)

## Vocabulary and proof ceiling

| Term | Meaning | Does not mean |
| --- | --- | --- |
| Planned stub | Named boundary, intended contract and return anchors; no substantive supported implementation | Compiled, qualified, complete or deployed |
| Implementation candidate | Authorized substantive source for a declared scope/profile with evidence still being collected | Accepted compatibility baseline |
| Verified observation | A retained check result matches its declared current subject and scope | Complete module or trusted independent admission |
| Fully complete module | Every applicable mandatory gate and owning criterion is independently admitted for a pinned version/profile, with corpus closure | All future features, every possible environment or assembled release |
| Fully complete codebase/release | Separately admitted composition of required module versions plus all integrated system, security, packaging and operational obligations | A sum of child passes |
| Deployed and verified | Installed artifact/config/schema identity and useful independent readback in the named environment | Continuing health forever |
| Stale/unassessed | A changed or missing bound subject/criterion/evidence invalidates current applicability | Deletion or rewriting of the historical result |

All 22 modules remain unassessed; inspect source bodies and the evidence index to distinguish comment stubs from substantive implementation candidates. No accepted module or release exists. The full-standard collector and independent admission path are unimplemented. The coding lock is an operator/workflow rule, not a cryptographically authenticated software gate. Quoting its trigger here does not unlock it.

The maintenance loader checks this revision’s reviewed semantic baseline as a drift tripwire, including normative prose and runbook availability. An intentional policy change needs an explicit reviewed revision and corresponding baseline/regression update. This is not an independent security boundary against an editor changing both the convention and verifier, and it does not execute any engine acceptance gate.

## Mandatory gates

### G01 · Scope, authority and applicability

Luke has issued an actual, unquoted start coding instruction for the selected work. Freeze the module version, declared profile, requirements, interfaces, task dependencies and supported environments before interpreting results.

**Evidence:** Scoped operator instruction, requirement/task/profile matrix, immutable candidate identity and change rationale.

**Refusal:** A quoted trigger, self-authorized scope expansion, unknown profile or omitted mandatory requirement refuses admission.

### G02 · Complete supported behavior

Every required operation has substantive implementation with real typed signatures, documented inputs/results/errors and meaningful consumer examples. No TODO, placeholder, panic or no-op substitutes for promised behavior. Internal implementation remains private unless an actual consumer contract requires exposure.

**Evidence:** Code-derived interface inventory and consumer compatibility fixtures covering every normative operation and supported version.

**Refusal:** An omitted operation, undocumented incompatible change or empty success path fails; intentionally unsupported behavior must return its declared refusal.

### G03 · Boundary and ownership correctness

Use the existing module owner for each state machine and semantic action. Validate capabilities, identities, versions, payload bounds and budgets before the hazard. Carry cancellation, deadlines, errors and provenance through request and return paths.

**Evidence:** Boundary tests and cross-module traces for valid input, denied scope, malformed input, stale identity, concurrency and lost reply.

**Refusal:** Duplicate semantic owners, bypass adapters, cyclic build dependencies or unknown effects presented as success block completion.

### G04 · Meaningful testing and detector qualification

Meet the existing minimum of 50 distinct qualifying primary-module cases. Cover actual requirements and independent oracles, not count padding. Exercise assimilation faults and benign mirrors, mutation sensitivity, applicable property/fuzz/concurrency/recovery and consumer integration.

**Evidence:** Exact case IDs and primary owners, full subject/profile matrix, raw outcomes, intended-reason fault/benign results and nonempty mutation dispositions under the active testing standard.

**Refusal:** Zero/skipped/duplicate/filler cases, missing denominator, dead detector, changed oracle merely to pass or unmeasured mandatory profile cannot qualify.

### G05 · Zero diagnostic quality baseline

All applicable build, format, pedantic lint, test, documentation and compatibility checks complete with zero baseline warnings and errors. Rust includes pedantic Clippy and rustdoc on admitted profiles; Julia/package/shell checks match their actual supported surfaces.

**Evidence:** Pinned toolchain and dependency/feature/target matrix with decisive producer status and retained raw diagnostics; follow TESTING_STANDARD exactly.

**Refusal:** Suppressions, reduced profiles, discarded stderr, successful tee/display or broad allows cannot erase an unresolved warning or required check.

### G06 · Security and supply-chain closure

Review the actual trust boundaries, dependencies, secrets, grants, parsing, filesystem/UDS custody, external process lifetime and update path. Reproduce and repair material findings on the integrated subject; recheck the fix.

**Evidence:** Bound threat model, dependency/license/provenance inventory, negative-control security results, reviewer dispositions and residual risk owner/scope.

**Refusal:** An unreviewed material finding, unbounded resource path, privilege bypass, unauthenticated completion producer or green scanner with failed controls blocks the applicable claim.

### G07 · Recovery and durable obligations

For state/effectful modules, prove crash, replay, cancellation, timeout, restart, ambiguous effect and cleanup behavior. Libraries expose explicit errors without pretending to own daemon recovery. Retain reservations and unresolved obligations until actual reconciliation.

**Evidence:** Fault-injected restart/replay traces, stable operation identities, independent readback and resource/obligation conservation checks.

**Refusal:** Process exit or listener health alone cannot prove task acceptance, effect completion, cleanup or useful service recovery.

### G08 · Data, migration and compatibility

For every persistent format or migration, declare version/checksum/read-write compatibility, exclusive custody and atomicity. Test fresh, upgrade, restart and independent restore; freeze released history. Nonpersistent modules explain why the migration subcheck is inapplicable.

**Evidence:** Exact source/schema/migration subject, disposable database/dataset fixtures, checksum mismatch refusal, backup/restore integrity and compatibility readback.

**Refusal:** A mutable released migration, missing admission-aware anchor freeze, data loss or restore that drops pending obligations blocks release of the affected profile.

### G09 · Measured performance and resource fitness

Declare relevant latency/tail, memory, queue/fan-out, cost and numerical accuracy/uncertainty budgets before judging the candidate. Use representative workloads, complete denominators and an appropriate baseline. A small static package need not invent a GPU or throughput benchmark.

**Evidence:** Workload/environment/build identities, predeclared thresholds and observed distributions/uncertainty including failures; justified per-subcheck non-applicability.

**Refusal:** Missing measurements, post-hoc favorable thresholds, hidden failures or a top-7% claim without a defined comparison population cannot pass.

### G10 · Maintainability and bounded complexity

Retain focused modules, clear names/types/invariants and one owner per behavior. Each new dependency, crate, trait, service, socket or scheduler must solve a present contract or explicitly approved need. Prefer existing seams; remove dead paths and duplicated policy.

**Evidence:** Reviewer comparison of before/after dependency topology, public surface, runtime components, configuration and operational cost, with concrete requirement witnesses.

**Refusal:** Speculative frameworks, duplicate state machines, undocumented coupling or abstractions justified only by imagined scale block the proposed change until simplified or explicitly justified.

### G11 · Operational and consumer readiness

Supply reproducible build/package/install instructions for the admitted profile, useful observability, bounded logs/redaction, owned resources, shutdown, diagnosis, backup/restore and rollback dispositions. A library documents consumer integration; it does not require a daemon.

**Evidence:** Runbook inventory and rehearsed applicable procedures on the candidate, installed-artifact/configuration identity where deployment is claimed, useful independent readback and failure exits.

**Refusal:** Unrun critical runbook, hidden prerequisite, unsupported command, unbounded retry, missing cleanup owner or assumed successful installation prevents the corresponding readiness/deployment claim.

### G12 · Full-corpus and entry-point convergence

README, quick start, public interfaces, tests, migration comments/sidecars, runbooks, Justfile, module contracts, atlas/vault/ultra map and master indices agree with admitted code and current requirements. All relevant anchors return in both directions.

**Evidence:** Matching publication generation/output hashes, expected module/path/recipe inventories, exact projection checks and live native graph readback after the evidence is retained.

**Refusal:** Stale generated wrapper, broken anchor, missing return route or documentation claiming unavailable behavior blocks documentation closure; it never reverses an already-observed external effect.

### G13 · Independent acceptance and truthful state

A qualified admission collector and responsible reviewer bind all applicable gate evidence to the exact candidate, profile, standard revisions and dependency/fixture/environment subjects. Child reports are candidates; the parent verifies integration.

**Evidence:** Evidence manifest with independent producer/reviewer identity, raw logs/artifacts, explicit gate dispositions, unresolved effects and acceptance identity. Separate assembled-codebase release admission.

**Refusal:** V1 observations, a model statement, source presence, aggregate test count or corpus PASS cannot admit a module. The collector is unavailable in this edition.

## Concrete module completion-contract directory

| Module | Required finished behavior | Exact contract |
| --- | --- | --- |
| contracts | Typed request/result/error and identity/version contracts round-trip without losing semantics across declared consumers. | [HEE3-DONE-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-contracts) · [stub](file:///var/home/herdr-engineering-engine-v3/src/contracts.rs) |
| task | One durable task lifecycle performs admit → execute → verify → repair/escalate → reverify → accept, with correct terminal and pending states. | [HEE3-DONE-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-task) · [stub](file:///var/home/herdr-engineering-engine-v3/src/task.rs) |
| store | Transactional ledger preserves task, lease, reservation, event and evidence invariants across concurrent access, migration and restore. | [HEE3-DONE-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-store) · [stub](file:///var/home/herdr-engineering-engine-v3/src/store.rs) |
| roster | Versioned agent, model and service registrations expose capability, provenance, availability and freshness without equating unknown to healthy. | [HEE3-DONE-roster](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-roster) · [stub](file:///var/home/herdr-engineering-engine-v3/src/roster.rs) |
| route | Policy chooses an eligible model/worker using task constraints and measured quality, cost, latency and failure evidence with explainable fallback. | [HEE3-DONE-route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-route) · [stub](file:///var/home/herdr-engineering-engine-v3/src/route.rs) |
| budget | Reservations and actual usage conserve limits across concurrent work, retries, verification, context processing and uncertain effects. | [HEE3-DONE-budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-budget) · [stub](file:///var/home/herdr-engineering-engine-v3/src/budget.rs) |
| worker | Replaceable admitted adapters execute bounded model work with actual model/process identity, stream custody, cancellation and retained result evidence. | [HEE3-DONE-worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-worker) · [stub](file:///var/home/herdr-engineering-engine-v3/src/worker/mod.rs) |
| check | Acceptance checks have qualified detectors and complete exact-subject evidence; forged, empty, stale or wrong-profile proof cannot become success. | [HEE3-DONE-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-check) · [stub](file:///var/home/herdr-engineering-engine-v3/src/check.rs) |
| recovery | Restart reconciles durable task/effect/process identities, leases and reservations without blindly repeating uncertain work. | [HEE3-DONE-recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-recovery) · [stub](file:///var/home/herdr-engineering-engine-v3/src/recovery.rs) |
| cohort | Bounded specialist threads maintain shared brief revisions, separate write/resource ownership, dissent and evidence-aware parent joins. | [HEE3-DONE-cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-cohort) · [stub](file:///var/home/herdr-engineering-engine-v3/src/cohort.rs) |
| context | Context packets are scoped, bounded and reproducible, preserving source identities, gaps and bidirectional dependency/return context. | [HEE3-DONE-context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-context) · [stub](file:///var/home/herdr-engineering-engine-v3/src/context.rs) |
| notify | Committed events reach permitted subscribers with durable delivery identity, bounded backlog and truthful gap/replay semantics. | [HEE3-DONE-notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-notify) · [stub](file:///var/home/herdr-engineering-engine-v3/src/notify.rs) |
| service | Registered binaries, daemons, libraries and numerical services expose useful health and delegated owner-controlled lifecycle with effect readback. | [HEE3-DONE-service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-service) · [stub](file:///var/home/herdr-engineering-engine-v3/src/service.rs) |
| herdr | The multiplexer/client presents truthful task, thread, route and proof state and reconnects through the same versioned action semantics. | [HEE3-DONE-herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-herdr) · [stub](file:///var/home/herdr-engineering-engine-v3/src/herdr.rs) |
| numerical | Rust mediates bounded immutable Julia/numerical requests and validates report identity, units, uncertainty, runtime and policy compatibility. | [HEE3-DONE-numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-numerical) · [stub](file:///var/home/herdr-engineering-engine-v3/src/numerical.rs) |
| julia | Julia analysis produces reproducible routing/cohesion reports with complete denominators, justified tolerances and held-out baseline comparisons. | [HEE3-DONE-julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-julia) · [stub](file:///var/home/herdr-engineering-engine-v3/julia/src/HabitatAnalysis.jl) |
| app | A small composition root exposes one action catalogue through CLI/local serve, validates startup custody and shuts down with obligations retained. | [HEE3-DONE-app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-app) · [stub](file:///var/home/herdr-engineering-engine-v3/src/main.rs) |
| actions | One versioned catalogue gives caller-visible discovery, typed inspection and dispatch contracts for all 21 declared actions. | [HEE3-DONE-actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-actions) · [stub](file:///var/home/herdr-engineering-engine-v3/src/actions.rs) |
| bash | Thin Bash/Just entry points preserve literal arguments, bounded I/O, decisive producer exit and cancellation over existing action semantics. | [HEE3-DONE-bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-bash) · [stub](file:///var/home/herdr-engineering-engine-v3/integrations/bash/README.stub.md) |
| pi_extension | A version-pinned replaceable Pi integration registers compatible action tools and carries host call identity, result/error, usage and cancellation. | [HEE3-DONE-pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-pi_extension) · [stub](file:///var/home/herdr-engineering-engine-v3/integrations/pi/README.stub.md) |
| skills | Versioned skills expose purpose, compatible actions and bounded reference graphs without expanding access or rewriting task criteria. | [HEE3-DONE-skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-skills) · [stub](file:///var/home/herdr-engineering-engine-v3/skills/README.stub.md) |
| workflows | Finite versioned procedures use existing task/cohort scheduling and retain stable step identities, bounded retries and independent parent acceptance. | [HEE3-DONE-workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-workflows) · [stub](file:///var/home/herdr-engineering-engine-v3/workflows/README.stub.md) |

## Applicability without scope creep

Every module answers every gate. A subcheck may be `not applicable` only with a concrete reason, owning requirement witness and reviewer disposition: a static skill package has no database migration; an in-process library has no daemon shutdown procedure. This cannot waive the 50-case floor, zero diagnostics, promised public operations, security boundaries or applicable atlas criteria. Optional PyTorch/neural-operator profiles remain explicitly excluded or deferred until separately admitted; an excluded optional profile is not a completed module. Whole-stack completion must identify any planned modules still deferred.

Avoid arbitrary universal coverage percentages, complexity scores or latency targets. Define measurements appropriate to the module before evaluation; report actual distributions, failures and uncertainty. A test floor is a minimum qualification constraint, not evidence of excellence by itself. Do not add frameworks, services or test filler to satisfy a checklist.

## Planned admission record — specification, not an accepted schema

- module/version/profile and exact candidate revision or content tree
- operator authorization and scoped requirement/criterion matrix
- standard/interface/recipe/runbook/fixture/dependency/environment hashes
- complete case inventory with primary ownership and raw producer verdicts
- assimilation fault/benign and mutation dispositions
- security review, findings, residual obligations and independent reviewer identity
- applicable gate verdicts and justified non-applicable subchecks
- supported consumer compatibility and integrated parent evidence
- migration/checksum/freeze and operational readback when applicable
- admission identity/time plus current-versus-desired and stale applicability
- corpus generation/outputs and post-admission documentation readback

The existing v1 evidence index remains an observation mechanism. It cannot store an invented complete verdict as accepted evidence. Before first admission, implement and qualify the collector, exact-subject verifier and its independent authority path under Luke’s coding authorization. No `complete` command or accepted-receipt format is available today.

## Drift rules and closure loop

Freeze scope → implement only after authorization → qualify detectors → execute all applicable checks → verify/repair/reverify → independent module admission → separate parent/release admission → derive current interface facts from accepted code → synchronize corpus → read back all anchors, wrappers and docs. Changed source, interface, dependency, toolchain, fixture, standard or profile invalidates affected evidence; a navigation-only change follows its declared hash boundary. Preserve current-versus-desired differences and historical receipts.

Released migrations require immutable whole-file history and an admission-aware anchor freeze or sidecar before release. The current mutable-stub updater does not implement that guard. An interrupted documentation update resumes its journal and never reruns accepted engine effects.

[50-case, assimilation, mutation and zero-diagnostic standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing) · [Public interfaces](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts) · [Patterns and counter-evidence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Full publication protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol)

## Adopted supplementary requirements

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). These active clauses strengthen the existing owner; none relaxes its baseline or claims implementation.

### F2 · Architecture, modularity and maintainability

**Recorded score:** 89/100. **Conditional target:** 93/100; no promotion from documentation adoption.

**Primary owners:** [contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-contracts), [task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task), [store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store), [roster](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-roster), [route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-route), [budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-budget), [worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-worker), [check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check), [recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-recovery), [cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-cohort), [context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-context), [notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-notify), [service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-service), [herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-herdr), [numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-numerical), [julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-julia), [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app), [actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-actions), [bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-bash), [pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-pi_extension), [skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-skills), [workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-workflows).

**Owning task contracts:** [T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01), [T03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T03), [T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04), [T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06), [T21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21), [T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22), [T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20).

**Governing standard/procedure routes:** [Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion). These are generated views of the same adopted clauses; edit the convention once.

- **F2-C01:** Give each existing module one concise boundary contract: state owned, invariants, dependencies, consumers, public operations, errors, resource lifetime and prohibited responsibilities. Reuse the module cards and completion contracts; avoid a second module taxonomy.
- **F2-C02:** Specify concrete composition and dependency rules. Rust owns operational authority; Julia owns bounded analysis. Keep SQLite as the one proposed task ledger and app as the composition root. A module is not automatically a crate, daemon or socket.
- **F2-C03:** Keep policy logic deterministic and separate from provider/process/filesystem effects. Pass typed identities and observations into route/budget/check policy; return explicit decisions instead of reading ambient global state.
- **F2-C04:** Choose and record async cancellation, task ownership, error propagation, transaction boundaries and shutdown semantics before broad adapter work. Retain task, effect, cleanup, evidence and delivery state as separate dimensions.
- **F2-C05:** Plan two maintainability exercises: add an adapter without changing task lifecycle ownership; add an action or evolve one schema with a deliberate compatibility path. Predict the affected modules before implementation and compare actual changes afterwards.
- **F2-C06:** Record complexity cost per change: new runtime processes, storage authorities, dependencies, configuration knobs, touched owners, review effort and rework. Introduce an abstraction only after a concrete second use or a demonstrated safety boundary.

**Required future proof:**

- A first integrated slice respects the declared dependency/ownership rules and proves cancellation, failed verification and lost-reply behavior.
- The two controlled change exercises preserve unaffected consumer contracts; unexpected cross-module edits have a documented cause and repair. No parallel task ledger or scheduler appears.

**Reassessment:** 90–91 requires a tested first slice with clear ownership. 93 requires evidence from a materially different slice or change exercise; documentation size alone earns no increase.

**Complexity guard:** Keep all 22 boundaries available, but implement dependency-ready slices. Defer optional capabilities explicitly instead of building empty frameworks for them.

### Promotion, invalidation and stop rules

- Preserve the original assessment and record each later score as a new scoped review with exact source/profile/criteria/evidence identities. Reassess only the facets supported by the new results.
- Every applicable mandatory gate must pass for the claimed module/release. Missing, skipped, stale, excluded or invalid observations remain visible. A preparation score cannot override failed acceptance.
- Re-run affected and integrated checks after source, dependency, compiler, fixture, schema, permission or deployment-profile changes. Treat automatic baseline acceptance as a defect.
- Stop a slice for unresolved ownership, unavailable oracles, unknown external effects, failed security boundaries, exceeded resources or unauthorized scope; repair or explicitly narrow the claim.
- Keep optional PyTorch/neural operators outside mandatory startup. Use T23/T24 only after simple baselines and domain-valid evidence justify their total cost.

