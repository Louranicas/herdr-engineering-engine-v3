# Configuration scope and Daybreak security mapping

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

`models.toml` and `agents.toml` contain explicit empty T05 roster change manifests. Import is an authorized, validated Store operation; reading either file starts nothing and never overlays admitted SQLite state. Other configuration facets retain their stated draft status. This note installs no credentials, runtime model selection or active service. See the roster contract for behavior and limits. The requested security alias is `gpt-daybreak-blue-latest`; exact effective identity must be observed when the reviewer is selected.

[Security profile and /model procedure](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [README](file:///var/home/herdr-engineering-engine-v3/README.md) · [Quick start](file:///var/home/herdr-engineering-engine-v3/QUICK_START.md) · [Restart handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)

## Resume configuration work in a new context

[Canonical start-coding restart pointer](file:///var/home/herdr-engineering-engine-v3/corpus/CONTEXT_HANDOFF.md) ↔ [Quick start](file:///var/home/herdr-engineering-engine-v3/QUICK_START.md)

Recover the current task, owner and generation before editing a declared config stub. Read RC02/RC03/RC05/RC06 and the module context card; inspect actual config bytes outside managed comments. A model/profile/service entry is a proposal until its implementation and useful readback are qualified. Keep secrets out of notes and snapshots. Return source/config changes and evidence to the existing corpus update protocol; do not treat this restart route as runtime activation.

| Existing configuration facet | Planned role / invariant |
| --- | --- |
| [models.toml](file:///var/home/herdr-engineering-engine-v3/config/models.toml) | Declare exact requested security alias, effective identity/effort and availability evidence; no silent fallback. |
| [agents.toml](file:///var/home/herdr-engineering-engine-v3/config/agents.toml) | Bind defensive reviewer role, model recipe, scoped grants, resource limits and return evidence. |
| [routes.toml](file:///var/home/herdr-engineering-engine-v3/config/routes.toml) | Keep designated security-review selection separate from ordinary task routing and independent acceptance. |
| [services.toml](file:///var/home/herdr-engineering-engine-v3/config/services.toml) | Model selection never grants lifecycle permission or useful-health truth. |
| [worker-profiles.toml](file:///var/home/herdr-engineering-engine-v3/config/worker-profiles.toml) | T26 threat/worker design profile: explicit grants, intended containment and trusted collector custody; execution is disabled and hostile isolation remains unqualified. |
| Other declared config/schema paths | Use the existing owning module and compatibility contract; do not invent provider secrets or a second registry. |

Codex user configuration is a separate product surface. The observed model/review-model defaults were read without changing them. Interactive `/model` affects the selected Codex session according to its UI; this project’s configuration files are not Codex configuration. Recheck overrides and provider availability on each material review.

## Selected release profile and required qualification

Read RC02, RC03, RC05 and RC06 in [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). RC02 fixes host runtime and Toolbx build responsibilities, toolchain candidates and private state/package paths; RC03 fixes wire versions; RC05 fixes execution grants; RC06 fixes ledger and restore policy. Apply these resolved design decisions. Actual locks, executable/package hashes and enforcement measurements remain pending implementation proof; no active configuration is claimed.

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

