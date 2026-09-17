# Corpus assimilation and module delivery workflows

[Learning contracts and counterexamples](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md)

[Progressive module context workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) · [Codebase context index](file:///var/home/herdr-engineering-engine-v3/docs/module-context.md) · [$hee-module-scout skill](file:///var/home/Louranicas/.codex/skills/hee-module-scout/SKILL.md)

Start each module task with its progressive context card and record exact source/relationship coverage before the next authorized step.

| Step | Owner | Required result | Learning routes |
| --- | --- | --- | --- |
| 1 · Intake and identify | Orchestrator / context | Admit the exact source, preserve source hash/date/scope and inspect current code and accepted revision. Source procedures are inert. | [LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05), [LRN09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN09) |
| 2 · Decide adoption | Module owner / reviewer | Name the local failure class, trigger, useful mirror and limit. Check existing standards; record reject, defer or adopt guidance explicitly. | [LRN10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10), [LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12) |
| 3 · Bind the seam | Module owner / actions / contracts | Map only relevant modules, tasks, APIs, IPC and call/return flows; one semantic owner and versioned consumer contract. | [LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03), [LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04) |
| 4 · Assign bounded work | Orchestrator / cohort | Use one objective, exact subject/brief, resource and write ownership, dependencies, limits, return evidence and cleanup per thread. Parent joins retain dissent and gaps. | [LRN07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07), [LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11) |
| 5 · Qualify the detector | Checker / security reviewer | First prove an intended-fault and benign fixture exercise the real check for the right reason. Missing/invalid measurement is not a clean scan. | [LRN02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN02), [LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12) |
| 6 · Execute authorized work | Module thread / existing workflow owner | Implement only the authorized scope; run checks before effects. Preserve producer verdicts and resource custody. This documentation task does not start engine implementation. | [LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03), [LRN07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07) |
| 7 · Verify and repair | Checker / module owner | Bind meaningful cases and exact profiles, ≥50 per module, zero warnings/errors, assimilation, mutation and integration/security dispositions. Repair/escalate, then reverify changed subjects. | [LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01), [LRN06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06), [LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08) |
| 8 · Admit module and assembled code | Task owner / independent reviewer | Obtain explicit evidence-bound completion through the future admitted collector; child success is insufficient. Accepted code governs implemented facts. This admission path is unavailable now. | [LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01), [LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05), [LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11) |
| 9 · Publish corpus and read back | Orchestrator / corpus-sync | Refresh README, interfaces, testing, workflows, lesson routes, atlas/vault/ultra map and masters. Freeze expected before/after hashes including diary backlinks; require matching complete records. | [LRN09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN09), [LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13) |
| 10 · Review usefulness and retire drift | Owner / reviewer | Record changed footprint, executed checks, actual prevented failures, false positives and rework. Supersede stale claims; merge duplication without deleting evidence or silently weakening requirements. | [LRN10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10), [LRN14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14) |

## The workflow available today

The installed documentation launcher supports consistency checks, explicit helper verification, synchronization and retained v1 observations. It does not execute engine validators, provider calls, migrations, service actions or future admission. From the host codebase directory, run each command separately:

```bash
./tools/corpus-sync --verify-tooling
```

This discovers the admitted maintenance tests, executes normal and optimized suites with Python warnings treated as errors, and binds case IDs plus raw logs to the helper receipt. It writes maintenance evidence in the atlas; it does not qualify a production module.

```bash
./tools/corpus-sync
```

This stages and verifies the corpora plus bounded diary navigation changes, publishes through the existing hash journal, checks live Obsidian navigation and issues complete receipts.

```bash
./tools/corpus-sync --check
```

This rechecks published bytes, graph structure, source/capture bindings and receipt applicability. The live-UI observation remains the one recorded during publication; it is not continuous monitoring.

## Boundaries for tests, migrations and integrations

[Test and interface matrix](file:///var/home/herdr-engineering-engine-v3/tests/README.md) · [Initial schema stub](file:///var/home/herdr-engineering-engine-v3/migrations/001.sql) · [Public consumer contracts](file:///var/home/herdr-engineering-engine-v3/docs/public-interfaces.md)

Use existing task/cohort scheduling for finite workflows; do not add a second scheduler. Bash retains literal argv and decisive status; Pi retains host/call/version identities; skills supply bounded references without grants; numerical work uses immutable datasets and declared uncertainty. Released migration history must be protected by a qualified checksum/freeze policy before acceptance.

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

[Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)

For defensive security review use gpt-daybreak-blue-latest: type `/model` in the Codex composer, select the exact requested model and effort, verify effective identity and any review_model override before assigning scoped work. Follow [RB05 repair and re-verification](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05). Publication cannot switch the active model or accept code.

## Closure record

Record scope and exact subject; changed modules and interfaces; criteria and case evidence; missing/invalid/skipped outcomes; raw diagnostics; reviewer dispositions; unresolved effects; current-versus-desired drift; publication generation and readback; counter-evidence locator; remaining next step. A documentation-complete record cannot promote a stub to accepted or deployed.

[Exact publication, conflict and recovery procedure](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol)

## Adopted supplementary requirements

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). These active clauses strengthen the existing owner; none relaxes its baseline or claims implementation.

### F7 · Traceability, cohesion and controlled change

**Recorded score:** 94/100. **Conditional target:** 96/100; no promotion from documentation adoption.

**Primary owners:** [contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-contracts), [task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task), [store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store), [roster](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-roster), [route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-route), [budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-budget), [worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-worker), [check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check), [recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-recovery), [cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-cohort), [context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-context), [notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-notify), [service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-service), [herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-herdr), [numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-numerical), [julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-julia), [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app), [actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-actions), [bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-bash), [pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-pi_extension), [skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-skills), [workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-workflows).

**Owning task contracts:** [T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01), [T11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T11), [T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20), [T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22), [T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29).

**Governing standard/procedure routes:** [Corpus Assimilation](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FCorpus%20Assimilation) · [Update Protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol). These are generated views of the same adopted clauses; edit the convention once.

- **F7-C01:** Use bounded task context packets containing the active objective/criteria, owned module and direct neighbors, relevant interfaces, specific lessons, source identities, write/resource custody and exact return obligations. Preserve critical relationships rather than just listing files.
- **F7-C02:** Derive only affected reading routes and projections from the owning catalogues. Keep accepted code facts, desired requirements, observations and publication status separate; invalidate applicability when any bound source/profile/oracle changes.
- **F7-C03:** Test context adequacy with a fresh executor: it must locate the governing requirement, owner, callable boundary, failure case and proof without inventing a missing decision. If necessary narrow the assignment or request the specific missing source.
- **F7-C04:** Plan one bounded relocation/restore drill using an explicit root mapping. Confirm the source/corpus/query routes work in a clean workspace without silently rewriting historical evidence or replacing absolute source identity with ambiguous basenames.
- **F7-C05:** Classify existing unresolved references as actionable current gaps, historical unavailable captures, external boundaries or parser limits. Preserve immutable archives; zeroing a link counter by deleting history does not improve quality.
- **F7-C06:** Measure corpus maintenance cost, stale-reference incidents, packet omissions, repeated reading and unnecessary regeneration. Consolidate duplicate procedures and retire demonstrably unused projections with preserved history.

**Required future proof:**

- A fresh context resumes a scoped task correctly using the handoff and packet; a missing-critical-contract negative control is detected.
- A relocation/recovery exercise preserves every required owner/anchor and exact evidence identity; current native routes remain complete and graph copies bind their core generation.
- A controlled change updates all affected facets and makes stale evidence visibly stale, with no competing acceptance state or new background service.

**Reassessment:** 94 already meets the requested band. Preserve it; 96 is conditional on clean-context and relocation/change drills that reduce demonstrated friction.

**Complexity guard:** Avoid a second knowledge graph, acceptance database or perpetual watcher. More links and notes do not themselves improve the score.

### Orchestrator and specialist custody

The orchestrator owns the objective, dependency-ready slice, criteria, resource allocations and final integration. Assign implementation owners a narrow write scope; a quality/evidence reviewer owns oracle and receipt review; a security reviewer owns the actual T26/T15/T27 boundary questions; operations owns disposable package/recovery proof. These are roles, not a requirement to keep four agents or services running.

Every thread brief binds objective/criterion revision, exact source subject, owned paths/resources, allowed actions, dependencies, limits, return schema and unresolved liabilities. Shared interfaces, lockfiles and migration bytes have one writer. A waiting parent releases resources needed by children. A child success message is a candidate result; the parent must independently verify the assembled subject.

Use the existing Daybreak security profile when that specialist review is indicated. Verify actual model identity and review overrides without silently substituting models. Independent review requires a different review action and disclosed shared assumptions; merely obtaining a second signature does not authenticate a result.

### Promotion, invalidation and stop rules

- Preserve the original assessment and record each later score as a new scoped review with exact source/profile/criteria/evidence identities. Reassess only the facets supported by the new results.
- Every applicable mandatory gate must pass for the claimed module/release. Missing, skipped, stale, excluded or invalid observations remain visible. A preparation score cannot override failed acceptance.
- Re-run affected and integrated checks after source, dependency, compiler, fixture, schema, permission or deployment-profile changes. Treat automatic baseline acceptance as a defect.
- Stop a slice for unresolved ownership, unavailable oracles, unknown external effects, failed security boundaries, exceeded resources or unauthorized scope; repair or explicitly narrow the claim.
- Keep optional PyTorch/neural operators outside mandatory startup. Use T23/T24 only after simple baselines and domain-valid evidence justify their total cost.

