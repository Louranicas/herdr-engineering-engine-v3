# RB04 · Accept migration, assemble and release

**Availability:** planned; qualified release and freeze controls required.

[Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md) · [Justfile](file:///var/home/herdr-engineering-engine-v3/justfile)

**Owner:** release/operations owner and independent security/integration reviewer. **Availability:** planned; no current release/migration execution recipe. **Preconditions:** admitted required module profiles, supported configuration, qualified collector, migration checksum/freeze guard and authorized release action.

1. Assemble exact versions and test the integrated task → route → worker → check → repair → accept path, thread joins, loss/restart and budget/roster/service boundaries.
2. Before accepting migration 001, test fresh/upgrade/restore, incompatible schema/checksum refusal, crash and concurrent startup in disposable databases. Freeze released bytes and move later navigation to a sidecar under the qualified policy.
3. Rehearse artifact/config/schema identity, installation, useful readback, bounded shutdown, diagnosis, backup/restore and rollback/forward-recovery. Record irreversible effects and obligation retention.
4. Complete T17/T18 and applicable T25–T27 evidence; T19/T20 control the pilot/release claims. Deferred optional modules remain named and unaccepted.
5. Obtain separate assembled-codebase release admission; deployment evidence identifies the actual environment and observation cutoff.
6. Publish code-derived facts and runbooks through RB02 and verify every relevant anchor. A listener, copy or unit start is insufficient.

**Failure/recovery:** refuse unsafe upgrade or changed historical migration; preserve evidence and execute only the admitted recovery strategy. Lost acknowledgement requires operation readback before retry. Do not roll back user edits or erase pending effects. **Closure:** admitted assembled profile plus separately verified deployment and current corpus; continuing operational health needs fresh observations.

[Apply the Daybreak security profile and actual-model evidence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB05 hardening/re-verification procedure](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)

## Module and corpus return routes

| Module | Completion proof | Source stub |
| --- | --- | --- |
| contracts | [HEE3-DONE-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-contracts) | [src/contracts.rs](file:///var/home/herdr-engineering-engine-v3/src/contracts.rs) |
| task | [HEE3-DONE-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-task) | [src/task.rs](file:///var/home/herdr-engineering-engine-v3/src/task.rs) |
| store | [HEE3-DONE-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-store) | [src/store.rs](file:///var/home/herdr-engineering-engine-v3/src/store.rs) |
| roster | [HEE3-DONE-roster](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-roster) | [src/roster.rs](file:///var/home/herdr-engineering-engine-v3/src/roster.rs) |
| route | [HEE3-DONE-route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-route) | [src/route.rs](file:///var/home/herdr-engineering-engine-v3/src/route.rs) |
| budget | [HEE3-DONE-budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-budget) | [src/budget.rs](file:///var/home/herdr-engineering-engine-v3/src/budget.rs) |
| worker | [HEE3-DONE-worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-worker) | [src/worker/mod.rs](file:///var/home/herdr-engineering-engine-v3/src/worker/mod.rs) |
| check | [HEE3-DONE-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-check) | [src/check.rs](file:///var/home/herdr-engineering-engine-v3/src/check.rs) |
| recovery | [HEE3-DONE-recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-recovery) | [src/recovery.rs](file:///var/home/herdr-engineering-engine-v3/src/recovery.rs) |
| cohort | [HEE3-DONE-cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-cohort) | [src/cohort.rs](file:///var/home/herdr-engineering-engine-v3/src/cohort.rs) |
| context | [HEE3-DONE-context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-context) | [src/context.rs](file:///var/home/herdr-engineering-engine-v3/src/context.rs) |
| notify | [HEE3-DONE-notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-notify) | [src/notify.rs](file:///var/home/herdr-engineering-engine-v3/src/notify.rs) |
| service | [HEE3-DONE-service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-service) | [src/service.rs](file:///var/home/herdr-engineering-engine-v3/src/service.rs) |
| herdr | [HEE3-DONE-herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-herdr) | [src/herdr.rs](file:///var/home/herdr-engineering-engine-v3/src/herdr.rs) |
| numerical | [HEE3-DONE-numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-numerical) | [src/numerical.rs](file:///var/home/herdr-engineering-engine-v3/src/numerical.rs) |
| julia | [HEE3-DONE-julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-julia) | [julia/src/HabitatAnalysis.jl](file:///var/home/herdr-engineering-engine-v3/julia/src/HabitatAnalysis.jl) |
| app | [HEE3-DONE-app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-app) | [src/main.rs](file:///var/home/herdr-engineering-engine-v3/src/main.rs) |
| actions | [HEE3-DONE-actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-actions) | [src/actions.rs](file:///var/home/herdr-engineering-engine-v3/src/actions.rs) |
| bash | [HEE3-DONE-bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-bash) | [integrations/bash/README.stub.md](file:///var/home/herdr-engineering-engine-v3/integrations/bash/README.stub.md) |
| pi_extension | [HEE3-DONE-pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-pi_extension) | [integrations/pi/README.stub.md](file:///var/home/herdr-engineering-engine-v3/integrations/pi/README.stub.md) |
| skills | [HEE3-DONE-skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-skills) | [skills/README.stub.md](file:///var/home/herdr-engineering-engine-v3/skills/README.stub.md) |
| workflows | [HEE3-DONE-workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-workflows) | [workflows/README.stub.md](file:///var/home/herdr-engineering-engine-v3/workflows/README.stub.md) |

[Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) · [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) · [Update/recovery protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol)

## Adopted readiness in this procedure

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)


## Adopted supplementary requirements

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). These active clauses strengthen the existing owner; none relaxes its baseline or claims implementation.

### F6 · Build, deployment, migration and operations

**Recorded score:** 78/100. **Conditional target:** 92/100; no promotion from documentation adoption.

**Primary owners:** [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app), [store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store), [recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-recovery), [service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-service), [notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-notify), [check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check).

**Owning task contracts:** [T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04), [T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14), [T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17), [T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18), [T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19), [T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20), [T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27).

**Governing standard/procedure routes:** [Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04) · [Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration). These are generated views of the same adopted clauses; edit the convention once.

- **F6-C01:** Write a supported release profile identifying host versus Toolbx responsibilities, Rust/Julia toolchain and package versions, dependency locks, configuration schema, state/artifact paths and lifecycle owner. Production start must not depend accidentally on an authoring shell.
- **F6-C02:** Specify package identity and repeatable build inputs. Distinguish a repeatable build procedure from byte-identical reproducibility; investigate and disclose nondeterministic inputs instead of asserting equivalence.
- **F6-C03:** Define migration 001 ownership and freeze policy before release: ordered version/checksum, exclusive writer, transactional application, supported readers/writers and immutable released bytes. Freeze its comments or move later navigation to a sidecar.
- **F6-C04:** Define artifact/ledger durability ordering, idempotency conflict behavior and outbox coupling. Cover interruption between writing an artifact, making it durable, committing its database reference and delivering completion.
- **F6-C05:** Extend the existing runbooks with exact implemented commands only when available: preconditions, authority/effects, expected outputs, decisive checks, timeout, rollback/forward recovery, cleanup and retained evidence. Keep Justfile wrappers thin and test producer failure propagation.
- **F6-C06:** Specify a disposable rehearsal matrix: fresh install, populated upgrade, duplicate/concurrent startup, incompatible schema/checksum, disk full, process crash, login/reboot behavior, shutdown, backup, restore and rollback/forward recovery.
- **F6-C07:** Before restoring older state, preserve the current/failed ledger and reconcile every post-backup operation, external effect, resource obligation and usage record. Changing an event epoch cannot prevent replay of an external effect erased by restoring a database.
- **F6-C08:** Define useful health and recovery objectives from the actual workload. Read back answering interfaces, installed artifact/config/schema, outstanding obligations and residual processes; file copying, process existence or a listener alone is insufficient.

**Required future proof:**

- One versioned package completes the whole disposable installation/migration/recovery cycle, including intended-fault and benign controls and immutable-history refusal.
- An independent operator or clean environment repeats the procedure and verifies expected task/artifact/evidence/service inventory; an omitted-service fixture is detected.
- Restore preserves or explicitly reconciles post-backup effects and proof retention; the exact assembled release receives separate acceptance and deployment observation.

**Reassessment:** 90 follows one fully rehearsed disposable package/migration/recovery cycle. 92 requires repeatability or independent environment readback plus integrated release evidence.

**Complexity guard:** Use the established systemd/Podman ownership where qualified. Do not build another deployer, supervisor or backup service merely to raise this score.


[Progressive module context workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) · [Codebase context index](file:///var/home/herdr-engineering-engine-v3/docs/module-context.md) · [$hee-module-scout skill](file:///var/home/Louranicas/.codex/skills/hee-module-scout/SKILL.md)

Use the selected context card before this procedure; preserve read/unread source status, caller/callee return coverage and exact subject freshness.

## Recommendations routed through this procedure

Use RC01 cutoff-based backup freshness and full incident-to-readback recovery clock; reserve active and recovery-generation capacity before effects. Bind RC02 exact package/runtime/config paths and RC06 static SQLite identity, migration checksum/sidecar, writer custody and artifact-before-ledger ordering. Preserve failed/current state, classify every post-backup obligation including unknown effects, rehearse independent restore, and keep ordinary dispatch blocked until reconciliation permits it. Follow T17/T18/T27 then T19/T20; do not widen limits to turn failure into success.

**Current entry points:** `just runbooks`, `just contract-decisions`. Reading or checking documentation does not execute this procedure’s future engine steps.

| Governing recommendation cluster | Exact criterion IDs | Primary module owners |
| --- | --- | --- |
| [F1 · Vision, requirements and scope](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF1) | F1-C01, F1-C02, F1-C03, F1-C04, F1-C05 | [task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task), [route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-route), [budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-budget), [numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-numerical), [julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-julia), [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app) |
| [F6 · Build, deployment, migration and operations](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6) | F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08 | [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app), [store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store), [recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-recovery), [service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-service), [notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-notify), [check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check) |

| Resolved contract | Owner / original tasks | Required selected input |
| --- | --- | --- |
| [RC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01) | [task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task) / [T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01), [T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12), [T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19) | One offline small Rust library change, zero external spend, one candidate, three attempts, 20-minute task bound, explicit oracle/split, cancellation and recovery objectives. |
| [RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02) | [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app) / [T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01), [T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25), [T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26), [T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18) | Fedora44 x86_64 host runtime; Rust1.98.0, Julia1.12.7, Node24.21.0, Pi0.85.1 and private static SQLite3.53.4; Toolbx for authoring/build only. |
| [RC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06) | [store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store) / [T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04), [T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18), [T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27) | One store-owned WAL/FULL ledger, frozen whole-file migration checksum, durable artifacts before atomic ledger/outbox commit, verified quiesced backups and preserved-state reconciliation. |

The focus above does not waive any applicable module clause or completion gate. Follow the module’s complete source contract and scoped role; participating modules do not inherit another owner’s implementation responsibilities.

[Complete recommendation → procedure → module routing matrix](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) ↔ [Reviewed Justfile](file:///var/home/herdr-engineering-engine-v3/justfile) ↔ [Detailed graph in this vault](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
