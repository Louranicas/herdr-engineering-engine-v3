# Testing, interface assimilation and migration verification

This directory contains planned test stubs. No runnable engine test cases or qualified counts are established by this document.

[Codebase master](../README.md) · [Testing standard](../docs/testing-standard.md) · [Public interfaces](../docs/public-interfaces.md) · [Update protocol](../corpus/UPDATE_PROTOCOL.md) · [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex)

## Case ownership and acceptance

Minimum **50 distinct qualifying module-owned cases** for each of 22 modules: at least 1,100 primary case credits across the assembled scope. Tests must have independent behavioral oracles; assertions, repetitions, seeds, lint checks and mutant executions do not multiply credits. Shared cases have one primary owner and explicit supporting-module evidence. Zero baseline warnings and errors applies to all admitted targets/profiles and applicable lint, pedantic Clippy, rustdoc, Julia and package checks. Qualification also requires assimilation, a meaningful nonempty mutation/fault campaign, integration/security dispositions and closure of material gaps.

## Required case record

Stable case ID; primary module; requirement and interface/flow obligation; risk and independent oracle; subject/test/fixture/dependency/toolchain hashes; applicable profile; invocation; expected and actual outcome; raw stdout/stderr/status references; skips/invalidity reasons; diagnostic counts; assimilation fault/benign pair; mutation campaign linkage; reviewer disposition. This is a record specification, not an implemented collector. Existing v1 aggregate observation counts cannot qualify it.

| Module | Interface / complete contract | Coverage priorities |
| --- | --- | --- |
| contracts | [HEE3-IF-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-contracts) | schema/version compatibility; identity/correlation integrity; bounded decode and malformed inputs; typed errors and serialization invariants |
| task | [HEE3-IF-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task) | state transitions and terminal invariants; duplicate/replayed commands; cancellation and deadlines; parent/child acceptance separation |
| store | [HEE3-IF-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store) | atomic writes and crash recovery; idempotent replay; schema migration/rollback; stale writers and durable identity |
| roster | [HEE3-IF-roster](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-roster) | agent/model/service schema admission; capability freshness and identity; unsupported versions and expiry; no authority from registry labels |
| route | [HEE3-IF-route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-route) | capability eligibility and refusal; task-specific quality/cost choices; budget/latency constraints; stale evidence and deterministic fallback |
| budget | [HEE3-IF-budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-budget) | reservation/spend accounting; overflow and boundary arithmetic; concurrent reservation ownership; refund/cancel/exhaustion semantics |
| worker | [HEE3-IF-worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-worker) | provider/adapter request-return contracts; partial streams and malformed output; cancel/deadline/cleanup; no worker self-acceptance |
| check | [HEE3-IF-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check) | independent oracle and exact subject; zero/missing/skipped case refusal; forged or stale evidence; producer/collector custody and negative controls |
| recovery | [HEE3-IF-recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-recovery) | uncertain effects and useful readback; retry without duplicate effects; crash points and preserved obligations; shutdown/rollback ordering |
| cohort | [HEE3-IF-cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-cohort) | thread/agent roster cohesion; ownership and claim transfer; join/fanout/cancellation; parent verification and stale messages |
| context | [HEE3-IF-context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-context) | provenance and authority separation; bounded context and truncation disclosure; prompt-injection boundaries; versioned retrieval/return identity |
| notify | [HEE3-IF-notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-notify) | delivery ordering and correlation; deduplication/replay; bounded subscribers and backpressure; disconnect/recovery without false acceptance |
| service | [HEE3-IF-service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-service) | admission and lifecycle ownership; binary/daemon/library/operator boundaries; health/readback and failure isolation; restart/stop and resource cleanup |
| herdr | [HEE3-IF-herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-herdr) | session/socket/generation binding; pane/agent identity drift; presentation versus engine authority; disconnect/reconnect and roundtrip contracts |
| numerical | [HEE3-IF-numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-numerical) | units/shapes/tolerances; independent reference and metamorphic properties; domain/shift/resource bounds; optional operator admission and evidence provenance |
| julia | [HEE3-IF-julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-julia) | Rust-Julia request/return framing; locked package/bounds-checked execution; numerical invalid/edge inputs; timeout/cancel and deterministic fixtures |
| app | [HEE3-IF-app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app) | CLI/serve composition and dependency injection; complete command/error propagation; startup/shutdown/configuration drift; packaged full-stack integration and release readback |
| actions | [HEE3-IF-actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-actions) | command/tool schema equivalence; method authority and effect classification; idempotency/readback/correlation; hostile arguments and consistent errors |
| bash | [HEE3-IF-bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-bash) | literal argv and quoting; producer/pipeline exit status; cancellation and cleanup; schema/version/status preservation |
| pi_extension | [HEE3-IF-pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-pi_extension) | host/extension version contract; tool-call/stream adapter behavior; cancellation and refusal fidelity; context and capability admission |
| skills | [HEE3-IF-skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-skills) | skill provenance/version/context boundary; procedure versus execution authority; required input/output schema; unsafe or incompatible tool-call refusal |
| workflows | [HEE3-IF-workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-workflows) | DAG join and dependency completion; partial failure and skipped stage refusal; fanout cancellation and ownership; complete evidence return and no duplicate effects |

## Test and evaluation paths

| Path | Primary owner | Boundary and supporting modules |
| --- | --- | --- |
| [tests/fixtures/pi/](file:///var/home/herdr-engineering-engine-v3/tests/fixtures/pi) | worker | Protocol fixtures for Pi framing, identity, usage, ordering and cancellation under T02. Modules: worker, contracts |
| [tests/recovery.rs](file:///var/home/herdr-engineering-engine-v3/tests/recovery.rs) | recovery | Recovery cases cross live-attempt identity, cancellation, evidence/acceptance boundaries and restored event epochs. Modules: recovery, task, worker, store, check, notify |
| [tests/fixtures/native/](file:///var/home/herdr-engineering-engine-v3/tests/fixtures/native) | worker | Native-adapter fixtures qualify actual model/effort, support limits and result semantics under T08. Modules: worker, contracts |
| [tests/accounting.rs](file:///var/home/herdr-engineering-engine-v3/tests/accounting.rs) | budget | Accounting cases include concurrency, retries, checking, compaction, parent/child allocations and constrained fallback. Modules: budget, task, worker, check, context, cohort, route |
| [evaluation/tasks/](file:///var/home/herdr-engineering-engine-v3/evaluation/tasks) | julia | Held-out workload corpus compares routing and cohort outcomes with fixed acceptance and complete cost accounting. Modules: julia, route, task, check, budget, numerical, cohort |
| [evaluation/report.md](file:///var/home/herdr-engineering-engine-v3/evaluation/report.md) | julia | Evaluation report compares declared baselines and challengers from immutable outcomes and qualified acceptance evidence. Modules: julia, route, numerical, budget, cohort, check |
| [evaluation/cohorts/](file:///var/home/herdr-engineering-engine-v3/evaluation/cohorts) | cohort | Cohort scenarios evaluate shared objectives, resource conflicts, joins, rework and conserved allocations. Modules: cohort, julia, context, budget, task, worker |
| [tests/isolation/](file:///var/home/herdr-engineering-engine-v3/tests/isolation) | worker | Isolation fixtures cover candidate code, protected control/evidence paths, Julia hooks, credentials and process cleanup. Modules: worker, check, recovery, store, actions, numerical, julia |
| [tests/faults/](file:///var/home/herdr-engineering-engine-v3/tests/faults) | app | Integrated critical-failure matrix crosses the planned stack and preserves intended-refusal, invalid-fixture and unmeasured distinctions. Modules: contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [evaluation/pilot/](file:///var/home/herdr-engineering-engine-v3/evaluation/pilot) | app | Commissioned real-work pilot evaluates scoped end-to-end outcomes and release targets without automatically enabling optional lanes. Modules: contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [tests/security/](file:///var/home/herdr-engineering-engine-v3/tests/security) | app | Security qualification corpus tests the integrated threat model with independent collectors and explicit excluded or unmeasured scope. Modules: contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [evaluation/runtime-compatibility.md](file:///var/home/herdr-engineering-engine-v3/evaluation/runtime-compatibility.md) | numerical | Optional Julia/tch/LibTorch or external-service comparison records the concrete model, device, format, cost and cancellation contract. Modules: numerical, julia, service, contracts |
| [evaluation/operator-card.md](file:///var/home/herdr-engineering-engine-v3/evaluation/operator-card.md) | julia | Optional operator evidence declares domain, held-out tests, fallback and actual decision value; rejection preserves baseline routing. Modules: julia, numerical, route |

## Initial migration contract

[Migration 001](../migrations/001.sql) · [Store deployment contract](../docs/modules/store.md)

New-context coding route: file:///var/home/herdr-engineering-engine-v3/corpus/CONTEXT_HANDOFF.md ↔ file:///var/home/herdr-engineering-engine-v3/QUICK_START.md . Recover the store task and exact source/config/acceptance subject before authorized migration work; this pointer refresh executes no SQL. Preserve the future whole-file freeze and sidecar requirement.

Resolved RC06/F6: schema1 uses whole-file SHA256 migration freeze before acceptance; later navigation goes in docs/migrations/001.md sidecar. Preserve current/failed ledger and reconcile every post-backup effect, usage and resource obligation before restore; an epoch alone cannot prevent external replay. Route: file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md

Migration status: planning comments only; no installed schema or executed migration is claimed.

Coding authorization: human operator Luke must type start coding as an actual instruction; this quoted phrase and source comments do not authorize implementation.

Defensive security: requested gpt-daybreak-blue-latest through Codex /model; effective identity must be verified before review (interactive selection is not evidenced by this comment); apply G06/G08 and RB05 to schema, transactions, replay, compatibility and restore. No migration execution or acceptance follows from model review. Route: file:///var/home/herdr-engineering-engine-v3/runbooks/05-security-hardening.md

Full completion: HEE3-DONE-store and completion gates G01-G13 apply. Migration-specific release gates include exact subject, zero diagnostics, atomicity, crash/upgrade/restore proof, compatibility and qualified immutable-history protection.

Procedure and return routes: file:///var/home/herdr-engineering-engine-v3/runbooks/04-release-recovery.md ; file:///var/home/herdr-engineering-engine-v3/docs/completion-standard.md ; file:///var/home/herdr-engineering-engine-v3/corpus/CONTEXT_HANDOFF.md

Schema owner: store. Consumers: task, budget, check, notify and recovery through typed store transactions.

Initial schema intent: durable tasks/attempt generations, events/outbox, reservations/usage, cancellation/cleanup obligations and acceptance-evidence references. Exact tables, columns and constraints are pending authorized implementation.

Public compatibility boundary: ordered schema version, checksum, supported reader/writer range and transactional migration result; actual database readback owns deployed schema state.

Use exclusive writer custody, bounded lock handling and all-or-nothing migration application. Refuse unsupported versions and changed historical checksums before application.

Enforce referential integrity, stable identities, expected-generation transitions, idempotency uniqueness and outbox/ledger atomicity; specify indexes from measured access paths.

Test fresh install, populated upgrade, duplicate application, concurrent startup, interrupted commit, disk full, corrupt input, missing parent and inconsistent restored epoch.

Verify backup/restore and the declared rollback strategy independently. If down-migration cannot preserve obligations, refuse it and restore a verified backup rather than claiming reversibility.

Read back actual schema/version, integrity and retained task/reservation/evidence obligations before declaring success; migration process exit alone is insufficient.

After release acceptance migration bytes are immutable. Append a new migration for subsequent changes; corpus synchronization may update navigation comments only under an explicitly compatible checksum policy, and must never rewrite an accepted migration silently.

Checksum boundary for migration 001 (chosen 2026-09-24, store-lane A01; record `~/hee3-evidence/T04/A01-schema-freeze-20260924/DECISION.md`): a migration's recorded identity is the SHA-256 of the bytes after its single anchor-block end-marker line, pinned as a literal in `tests/t04_store.rs` (`MIGRATION_1_BODY`). This is the explicitly compatible checksum policy above: the publisher rewrote the anchor block three times with no DDL change, and each rewrite changed the whole-file digest. Because the whole file executes, `store::schema::identity` refuses a block containing any line that is not a `--` comment or blank, and a missing or repeated end marker. Any change below the marker changes identity and must fail the literal pin; after release acceptance the body is immutable and a later change is a new migration. The earlier recommendation (whole-file history with anchors frozen or moved to a sidecar) remains the alternative if navigation comments ever need to live inside the body.

Additive migration chain (2026-09-25, store-lane A25; record `~/hee3-evidence/T04/A25-migration-chain-20260925/RECORD.md`): `store::schema::MIGRATIONS` is the one ordered list; each file's identity is its own pinned body digest (the rule above, per file). A ledger records one history row per version, linked through `predecessor_version`/`predecessor_checksum`, with `user_version` = its last version; open-time readback refuses each clause by name (`store::Chain`). `Store::upgrade` is forward only: a verified backup through `Store::backup` precedes the first step, and each step is one transaction that also proves every table it rebuilds kept exactly its rows. Every `migrations/*.sql` carries its own anchor block and is a T04 gate subject.

Apply minimum 50 meaningful store-owned qualifying cases plus independent cross-module obligations, assimilation/mutation proof and zero baseline warnings/errors. No stub content counts as a test.

[Daybreak security profile](../docs/security-profile.md) · [RB05 re-verification](../runbooks/05-security-hardening.md) · [Fully-complete standard](../docs/completion-standard.md) · [Module completion runbook](../runbooks/03-module-completion.md) · [Justfile](../justfile). Engine coding remains unauthorized until Luke types start coding as an actual instruction.

## Verification and return loop

Observe exact candidate → run admitted cases and zero-warning checks → collect independent raw evidence → repair/escalate → rerun affected qualification → obtain module and integrated acceptance → update code-derived interface facts and corpus views → verify matching publication records and native links. Test/migration source at the accepted revision is factual primary evidence; desired obligations remain normative in the atlas and standard.
[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)

Apply F4 and the complete testing standard, including protected collection, distinct primary case credits, separate Rust doctests and Julia stderr/log/Broken handling. RC04 resolves the collector/receipt/oracle design; its executable bootstrap and protected custody still require independent qualification.
