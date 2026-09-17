# HEE3-MOD-store · store

SQLite transactions, migrations and outbox persistence.

Primary path: `src/store.rs`. Cluster: K1. This is the full planned responsibility; a reference exemplar exercises only its stated subset.

## Anchors and source ownership

[Source comment anchor](file:///var/home/herdr-engineering-engine-v3/src/store.rs) · [Atlas module card](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-store) · [Scoped reference example](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-store) · [Ultra map master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md)

## Dependencies and integration

Declared build dependencies: contracts. Declared consumers: app. Runtime request/return paths below do not imply cyclic build imports.

## Adopted readiness obligations

Own the single SQLite ledger, transaction/outbox ordering and immutable migration history. Preserve failed/current state and reconcile post-backup effects before restore; never infer exactly-once external effects from a DB key.

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). Binding: `HEE3-READINESS-001` / `ea991737ce282467556e4997f28e97c4b7f0799d823aae98905fcf34f71766f7`. Adoption is complete; the required engine proof is pending.

| Applicable facet / criterion IDs | Primary contract owners |
| --- | --- |
| [F2 · Architecture, modularity and maintainability](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2): F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |
| [F3 · Public interfaces and integration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3): F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06 | contracts, task, worker, actions, app, numerical, julia, herdr, bash, pi_extension, skills, workflows |
| [F4 · Testing, verification and completion evidence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4): F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07 | check, contracts, app, task, julia |
| [F5 · Security and hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5): F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06 | worker, check, service, actions, app, store, context, bash, pi_extension, skills, workflows |
| [F6 · Build, deployment, migration and operations](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6): F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08 | app, store, recovery, service, notify, check |
| [F7 · Traceability, cohesion and controlled change](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7): F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06 | contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, julia, app, actions, bash, pi_extension, skills, workflows |

Implementation groupings: [R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02), [R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05), [R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06), [R90-08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-08), [R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09), [R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10). Original task prerequisites and all 13 completion gates remain controlling; package membership grants no action authority. Shared quality/security/interface rules apply to this module’s actual scope, without creating new runtime responsibilities.

**Resolved design contracts:** [RC02 · Pinned release profile and deployment custody](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02), [RC03 · Versioned control and Rust–Julia contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03), [RC04 · Protected collection, receipts and independent oracles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04), [RC05 · Trusted and adversarial execution profiles](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05), [RC06 · SQLite, migration freeze and recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06). Apply the exact selected profile and pending-proof obligations.

Before a change, predict the affected owners and consumer contracts. Afterwards compare actual changes, rerun invalidated checks and retain counter-evidence. Minimum 50 distinct primary-owned cases, zero baseline diagnostics, trustworthy collection and independent parent/release acceptance remain mandatory.

## Public consumer interface contract

**HEE3-IF-store** · proposed semantic contract; concrete callable signatures remain to be implemented and verified.

[Shared authority, visibility, compatibility and admission requirements](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

**Supported surface:** Rust ledger transaction facade; SQL files form a versioned schema boundary owned by store, with no direct consumer SQL writes.

**Declared build consumers:** app.

| Operation obligation | Typed inputs to specify | Results to specify |
| --- | --- | --- |
| Commit ledger transition | Expected state/generation, task/attempt facts, reservation deltas, evidence references and outbox events | Atomic commit identity and sequence or conflict with no partial effect |
| Read consistent state | Bounded identifiers, cursor, snapshot requirements | Typed rows/events with ledger epoch and pagination |
| Apply or inspect schema history | Exclusive writer custody, ordered migration identity/checksum and supported version range | Transactional schema transition or refusal; independently read-back version and integrity |

**Errors and bounds:** Bound transactions and busy retry windows. Surface disk full, corruption, constraint failure and unsupported schema explicitly; never turn a failed write into success.

**Effects, replay and concurrency:** Single logical ledger writer; transactions maintain cross-table invariants and durable outbox. Released migration bytes are immutable. Replay uses durable identities.

**Required proof:** Atomicity and crash recovery; foreign-key/uniqueness constraints; concurrent starters; fresh install/upgrade/restore; migration checksum mismatch and schema readback.

**Existing API references:** no direct external API declared.

**Owned public actions:** none; mediated by declared consumers.

**Existing IPC associations:** in-process/package boundary; no new listener. Association is not ownership or permission to open an endpoint.

Every operation inherits the catalogue requirements for caller capability, identity, schema/version, errors, bounds, cancellation/settlement, idempotency/readback, visibility, deprecation and exact evidence subject. Resolve concrete symbols, types and examples in the owning task before interface acceptance.

### Accepted current interface and implementation

Observed source kind: `engine_source`. Accepted current interface: **unavailable**. No completion admission collector is implemented. A source body would be an implementation candidate, not automatically an accepted module. After verified admission exists, exact accepted source/schema and tests govern current interface facts; intended changes remain in the atlas.

Convention SHA-256: `089e6c05bbe6f53db1c473153d26638028b2e3c8f99d99967d8f0e7baa97bb8d`. This binding proves which documentation convention is projected, not runtime correctness.

## Fully-complete contract

**HEE3-DONE-store** · unassessed; acceptance collector unavailable.

[Concrete module outcome, failure controls and admission checklist](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-store) · [Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile and runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Context handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) · [Daybreak security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [Graphify full corpus](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md)

Completion standard SHA-256: `860dc3c4021b780b94ddcb8b9af22c4fd09fe3590e7785de0bcf5a3b5025c3ea`. All 13 gates apply; only an inapplicable subcheck may be explicitly justified.

**Required finished behavior:** Transactional ledger preserves task, lease, reservation, event and evidence invariants across concurrent access, migration and restore.

**Integrated proof scenario:** Fresh database, supported upgrade and independent backup restore preserve all outstanding obligations and committed events.

**Fault and benign controls:** Crash between coupled writes yields all-or-nothing state; checksum mismatch and unsupported schema refuse before mutation; valid history migrates.

**Complexity boundary:** One ledger authority; no second persistence engine or mutable released migration history.

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Daybreak defensive security review

Requested model: `gpt-daybreak-blue-latest`; select through the Codex `/model` picker and verify effective identity before the scoped review. Model selection and a reviewer message cannot accept this module.

**HEE3-SEC-store:** Transaction integrity, migration immutability, concurrent writers, corruption and restored outstanding obligations.

[Model selection, evidence and full security matrix](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB05 security hardening and re-verification](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)

Runtime security qualification is unassessed. Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

## Applicable patterns, antipatterns and learning triggers

[Reviewed diary learnings, evidence limits and retirement rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Assimilation and verification workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FCorpus%20Assimilation)

| Learning | Use when | Good pattern |
| --- | --- | --- |
| [LRN04 · One semantic owner across every consumer surface](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04) | When adding a CLI, tool, Pi, Bash, workflow or cross-language entry point. | Use one typed owner and thin projections; pass source, environment, task and caller identities explicitly and preserve the return contract. |
| [LRN05 · Identify the exact source before measuring it](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05) | At re-entry, before evaluation or hardening, and before updating a completion claim. | Bind the actual source/artifact, dependencies, fixtures, toolchain and environment to the run; distinguish intended branch, current checkout and installed artifact. |
| [LRN08 · Keep meaningful coverage and the zero-warning baseline](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08) | When designing module tests, fixing lint diagnostics or reporting a hardening result. | Retain at least 50 distinct meaningful module-owned cases and zero baseline warnings/errors on every admitted profile; repair code and close material gaps. |
| [LRN13 · Test useful effects and declared persistence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13) | Before calling an integration connected, a deployment useful or a restore complete. | Send a real bounded request through the supported seam and verify useful return/readback; declare durable state and independently test restoration. |

These are scoped design and review obligations. Their engine detectors remain unqualified; source reflections and navigation counts are not implementation evidence.

## Mandatory module testing standard

[Active testing standard and counting rules](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing): **minimum 50 distinct qualifying cases; zero baseline warnings and errors**, including pedantic Clippy for all admitted Rust targets/profiles.

Bind assimilation, mutation sensitivity, lint coverage and raw evidence to the exact candidate. Property iterations, assertions, retries, lint findings and mutant runs do not multiply case credits. The current observation schema does not establish this qualification.

Standard: `HEE3-TEST-STD-001`; SHA-256: `1004749abf352815dc13b8317c4277ea124d06612137e9010666977bfd059986`. Full-standard qualification: **unassessed; collector/validator not implemented**.

Module priorities: atomic writes and crash recovery; idempotent replay; schema migration/rollback; stale writers and durable identity.

## Bidirectional contracts

| Flow | Request | Return | Closure |
| --- | --- | --- | --- |
| [F02 task → store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F02) | Expected state/generation plus atomic attempt/event changes | Committed identity or typed conflict/storage failure | Crash after commit preserves replayable obligation; same key/different body conflicts. |

## APIs, sockets and commands

Internal module: external access is mediated by its declared consumers.

## Full deployment stems

| Path | Owning module | Scope | Purpose |
| --- | --- | --- | --- |
| Cargo.toml | app | original planned path | Shared Rust package composition, dependency and feature policy for the existing Rust module set. |
| rust-toolchain.toml | app | original planned path | Pinned compiler and tooling tuple for the existing Rust modules; no additional package boundary. |
| docs/checks.md | app | original planned path | Shared quality contract spans the planned stack; each module retains its own behavioral checks and disclosed limits. |
| docs/security.md | app | original planned path | Integrated threat model records existing authority, data, execution and verification boundaries across the planned stack. |
| src/store.rs | store | original planned path | Owns the single SQLite write boundary, migrations, transactional state and outbox persistence. |
| migrations/001.sql | store | original planned path | Initial ledger schema joins task/attempt/event state, conservative reservations, cancellation and acceptance evidence references. |
| tests/recovery.rs | recovery | original planned path | Recovery cases cross live-attempt identity, cancellation, evidence/acceptance boundaries and restored event epochs. |
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
| src/contracts/roster.rs | contracts | authored candidate / T05 | Shared closed roster DTOs, exact field validation and dated proof comparison for original T05. |
| src/store/roster.rs | store | authored candidate / T05 | Original T05 roster persistence, request source identity and receiver clock through the sole Store owner. |
| src/store/roster/attempts.rs | store | authored candidate / T05 | Original T05 atomic roster selection, immutable attempt pins and attributable cancellation causes. |
| tests/t05_store.rs | roster | authored candidate / T05 | Independent T05 durable roster, atomic import, pin/disable and history development cases; separate Store regression credit. |
| docs/roster-contract.md | roster | authored candidate / T05 | Concrete T05 trusted-facade behavior, bounds, evidence and remaining integration obligations. |

## Delivery, verification and hardening contracts

### T04 · Implement the transactional task ledger

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)

Dependencies: T01, T02. Atlas task state: `done`.

**Acceptance:** Submission replay is idempotent; same key with different content conflicts; task/attempt/event updates commit together; migrations and consistent backup have failure tests. The first ledger includes atomic conservative work/verification reservations, effect uncertainty and cancellation intent. task.get can recover a lost admission reply by principal-scoped request key. Allocate event identity before immutable manifest creation; acceptance commits its reference with the outbox. Publish immutable artifacts crash-durably under a declared filesystem contract before committing their SQLite references; a completed write alone is insufficient. Use file/metadata synchronisation and atomic publication as required by that contract. Historical acceptance and current evidence availability are separate facts.

**Validator:** Proposed, not implemented: habitat-engine-check case T04 --fixture-root <disposable-fixtures>

### T06 · Close one execute–verify–repair loop with completion evidence

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)

Dependencies: T03, T04, T05. Atlas task state: `idle`.

**Acceptance:** A contained task uses the selected recipe and owned workspace, produces an artifact, runs an actual acceptance check and records result/cost. Worker end or exit zero alone cannot accept it. A failing candidate is repaired and reverified; exhausted and verifier-error cases stop truthfully. Acceptance and completion outbox event commit together with a criterion-to-evidence manifest bound to the actual candidate. Even this first loop requires a safe minimal budget gate and reserved verification allocation; unknown accounting cannot satisfy a hard ceiling. Rich cross-adapter reconciliation follows in T10. Terminal cancellation requires observed cleanup/effect reconciliation.

**Validator:** Proposed, not implemented: habitat-engine-check case T06 --fixture-root <disposable-fixtures>

### T07 · Recover and cancel owned attempts

[Owning task and current atlas state](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)

Dependencies: T06. Atlas task state: `idle`.

**Acceptance:** Crashes before/after dispatch acknowledgement, stale generations, PID reuse, queued Pi messages and partial cleanup produce correct recoverable states without duplicate writes. Ambiguous external outcomes remain explicit. Cover restart at verification/evidence/acceptance boundaries; lease expiry alone cannot authorize reuse of a still-writable old workspace. Acceptance explicitly rejects any earlier committed cancellation intent, even before terminal cleanup. If acceptance committed first, later cancellation cannot rewrite history. Startup may reattach observation to a positively reconciled live attempt without redispatch. Restored ledger epochs cannot silently reuse prior event cursors.

**Validator:** Proposed, not implemented: habitat-engine-check case T07 --fixture-root <disposable-fixtures>

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

Reopen and injected SQL abort are exercised. Power-cut behavior, disk-full uncertainty, adversarial path custody, retention, restore and multi-resource admission remain unqualified.

## Full return-anchor register

- [module cluster · CLU-K1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K1)
- [contributing codebase · CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
- [contributing codebase · CODE-CB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB03)
- [contributing codebase · CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
- [task · TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
- [implementation support task · TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
- [task · TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
- [task · TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
- [task · TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
- [task · TASK-T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)
- [task · TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
- [task · TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
- [task · TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
- [task · TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
- [task · TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
- [task · TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
- [task · TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
- [separate reference example · EX-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-store)
- [flow · FLOW-F02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F02)
- [handbook · HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
- [handbook · HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
- [handbook · HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
- [public interface convention · Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
- [planned module · MOD-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-store)
- [plan · SEC-architecture](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-architecture)
- [plan · SEC-deployment](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-deployment)
- [plan · SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
- [plan · SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
- [plan · SEC-runtime](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-runtime)
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
- [schematic · SC-SC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC04)
- [schematic · SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
- [schematic · SC-SC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC06)
- [schematic · SC-SC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC07)
- [schematic · SC-SC10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC10)
- [schematic · SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
- [schematic · SC-SC12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC12)
- [schematic · SC-SC13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC13)
- [schematic · SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
- [schematic · SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
- [schematic · SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
- [source · SRC-A03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A03)
- [source · SRC-A04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A04)
- [testing standard · Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
- [progressive context workflow · Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
- [module context scout · CTX-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-store)
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
- [completion and operational convention · DONE-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-store)
- [completion and operational convention · Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
- [completion and operational convention · Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
- [completion and operational convention · RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
- [completion and operational convention · RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
- [applied learning · LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
- [applied learning · LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
- [applied learning · LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
- [applied learning · LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13)
- [diary evidence source · DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
- [diary evidence source · DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
- [diary evidence source · DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
- [diary evidence source · DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
- [diary evidence source · DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
- [diary evidence source · DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
- [diary evidence source · DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
