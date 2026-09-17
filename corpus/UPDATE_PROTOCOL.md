# Full-corpus update and proof protocol

The ultra map connects the existing atlas, vault, codebase and reviewed diary reflections through stable **anchors**. Each anchor names a module, its source comments and its complete deployment contract. It supplies navigational redundancy and a verifiable update path. It does not grant execution authority or prove implementation.

## One owner for each kind of fact

| Fact | Owning record | Derived views |
| --- | --- | --- |
| Intended requirements, planned dependencies and task predicates | Preserved `PLAN_habitat_engine.json` plus current additive conventions/standards and their authoring sources | Planned atlas reports, module contracts and vault task notes |
| Current implemented behavior and supported public interfaces | Completed accepted module source/schema/tests/migration subject at the exact admitted revision; assembled accepted codebase for integrated behavior | Current interface facts in atlas, vault, ultra map and README; no accepted current interface exists yet |
| Completion and release admission | Trusted admission binding the complete subject, standards, integration and reviewer/owner disposition; collector is not implemented | Explicit accepted/current-versus-desired views; never inferred from a v1 observation |
| Module source paths, shared-path ownership and stable anchors | `corpus_catalogue.json` | Comment blocks, deployment stems, branch trees and return links |
| Fully-complete definitions, per-module outcomes and operational entry points | `COMPLETION_STANDARD_habitat_engine.json` and `completion_standard.py`; original atlas/testing/interface requirements still control their subjects | 22 completion cards, README, quick start, runbooks, generated Justfile, handoff and executive summary |
| Scoped patterns, antipatterns, source evidence and adoption limits | `REFLECTIONS_habitat_engine.json` with immutable captures under `evidence/reflections-20260915/`; the diary owns its historical authored text | Learning notes, module-specific lessons, source wrappers, workflows, quick start and managed diary return links |
| Curated themes and reference-example associations | Existing navigation and exemplar catalogues | Thematic notes and exemplar pages |
| Observed testing, hardening and installation | Retained scoped receipts referenced by `corpus/evidence-index.json` | Current-applicability summaries in every ultra-map copy |
| Which documentation generation reached each corpus | `CORPUS_PUBLICATION.json`, `corpus/publication.json`, `Maintenance/corpus-publication.json` | Completion/readback status; all three records must agree and bind the admitted diary writes too |

Two-way links do not imply two-way authority edits. Change an owning record, then regenerate its projections. Completed accepted code is the primary factual source for what exists and how supported consumers call it. The atlas remains normative for what is required, including safety and quality constraints. A bug in accepted code does not amend a requirement. Record divergence and require repair or an explicitly authorized requirement change. The JSON design spine and completion admission are not inferred from filesystem presence, model messages, comments, successful examples or deployment receipts.

## Root, branches, stems and leaves

The atlas master, vault master, codebase README and ultra-map master are root anchors. Six branches preserve the existing code clusters. Twenty-two stems correspond exactly to the proposed Rust, Julia and integration-package modules. Supporting leaves cover all 65 distinct task paths, including adapters, shared manifests, migrations, fixtures, tests, schemas, numerical evaluations and deployment material.

Every primary stub contains a maintained comments block between `HEE3-ANCHORS-BEGIN` and `HEE3-ANCHORS-END`. It names the owning module, full deployment contract, corpus masters and relevant atlas/vault facets. Module notes, task/interface notes and ultra-map stems return to that source path. Source snapshots remain unchanged; their provenance wrappers provide the return route.

Write implementation **outside** the maintained block. The synchronizer preserves that content. Changing a source location requires changing the declared mapping and reviewing all affected anchors; the tool does not silently delete retired files. Shared paths have one declared owner. Julia's Evaluate/Cohesion/operator stems remain within the existing Julia module; optional numerical workers do not become a new mandatory service.

## Separate lifecycle dimensions

| Dimension | What the view means | What cannot establish it |
| --- | --- | --- |
| Source kind | Comment stub or substantive source detected at the declared path | A file existing does not prove an implemented contract |
| Testing observation | A retained record has current subject/acceptance/log bindings and non-vacuous mandatory checks | Zero tests, skipped mandatory checks, an example receipt or a worker's “done” message |
| Hardening observation | Scoped checks plus a bound threat model and recorded finding dispositions are available | A lint pass, model review alone or an unexamined absence of findings |
| Deployment observation | The record binds installed artifact/configuration, environment and useful readback | Copying files, starting a process or a healthy listener alone |
| Accepted module | An admitted completion record binds exact source/interface/tests/migrations and every applicable acceptance/standard obligation | Source presence, scoped v1 observations, compile success or a documentation publication |
| Accepted assembled codebase | Separate integrated release admission binds the assembled subject, interfaces and operational/security gates | A sum of accepted modules or successful child tasks |
| Task acceptance | The existing task owner admits all required evidence and obligations | Any of the documentation observations above |

The observation checker validates record consistency and current applicability. It **does not authenticate the producer, execute a validator, certify an installation or grant task acceptance**. The trusted collector and operator/reviewer admission remain the existing workflow's responsibility. Preserve limitations and disclose self-check versus independent verification.

Receipt claims remain historical when their current applicability becomes `stale`, `unavailable` or `unqualified`. A changed source/configuration/fixture or acceptance hash invalidates the corresponding binding. A missing receipt or raw log cannot become a pass. Newer failed or incomplete observations must not be hidden behind older successful records. Cosmetic atlas navigation changes do not by themselves alter a receipt's declared source subject; an actual edit to a bound source file changes its byte identity.

Observations dated after the explicit assessment cutoff are unqualified. No clock-age expiry or continuing deployment health is inferred from an old receipt. Managed source comments are replaced while preserving the exact UTF-8 body bytes and line endings outside the block; structured JSON comment metadata preserves other fields semantically.

## Exact operator commands

From `/var/home/herdr-engineering-engine-v3`:

```bash
# Inspect consistency without executing engine code or changing corpus content.
tools/corpus-sync --check

# Verify maintenance helpers after their source or normative catalogue changes.
tools/corpus-sync --verify-tooling

# Reconcile authorized source/catalogue/evidence changes across the full corpus.
tools/corpus-sync

# Register an explicitly retained observation, then reconcile the corpus.
tools/corpus-sync --record corpus/receipts/observation.json
```

The record command accepts a receipt already retained under the codebase's `corpus/receipts/` directory. Its raw evidence and declared subject files must already exist in the permitted codebase scope. It does not execute command strings from receipts. The strict schema and current validation details are supplied by `corpus_evidence.py`; the generated codebase copy is `corpus/evidence-schema.json`.

The Bash launcher delegates to the existing atlas maintenance environment. It is not a new engine runtime dependency. It needs Python's standard library and the installed Obsidian CLI for the final live vault check. No daemon, watcher, cron job or network service is installed. The workflow invokes synchronization explicitly after a retained observation changes; a future qualified collector can invoke that same command at the existing completion/export boundary.

The canonical codebase is on the Fedora host at `/var/home/herdr-engineering-engine-v3`. This Toolbx session shares the user's home but exposes that sibling directory through `/run/host/var/home/herdr-engineering-engine-v3`. The maintenance entry point detects that situation and invokes the same command through `flatpak-spawn --host`; it does not create a second codebase or request elevated privileges. Stable anchors keep the canonical host path. In Dolphin, use Ctrl+L and paste the canonical path.

## Systematic update loop

1. **Contract or source change:** the assigned owner changes the authorized module, fixtures/configuration or owning atlas record. Source changes cannot advance an accepted compatibility baseline automatically.
2. **Execute and observe:** run the actual applicable test, hardening or deployment workflow in its admitted environment. Record the exact candidate and mandatory criteria before interpreting results.
3. **Verify and repair:** preserve failures, wrong-subject results, skipped cases, verifier errors, uncertain effects and cleanup obligations. Repair within the existing budget/scope; rerun affected checks and required integration regressions.
4. **Retain and admit observations:** the responsible collector/reviewer retains raw evidence and its scoped receipt. Register that receipt. This step does not close an atlas task or promote a deployment by itself.
5. **Reconcile:** rebuild comment anchors, module contracts, the portable ultra map, atlas entry points and native vault views from their owners. Derive current applicability from the declared subject; do not manufacture a fresh verification result.
6. **Check the generation:** verify complete module/path coverage, primary and support anchors, forward/return links, exact source copies, criterion text, managed-comment preservation and evidence limits. Preserve authored material and historical captures.
7. **Publish and read back:** freeze the planned documentation writes with expected before/after hashes. Reject concurrent edits. Publish each file atomically and verify all destinations, then check Obsidian's actual link graph. Multiple filesystems do not form one atomic transaction.
8. **Close the documentation update:** issue matching complete publication records only after atlas, vault, codebase and the specifically admitted diary navigation files read back correctly. Missing roots or an interrupted generation remain pending. Resume documentation publication; never rerun accepted engine effects to repair missing documentation.

Before step 5, distinguish `planned_stub`, `implementation_candidate` and `accepted_module`. The current tooling cannot admit a module: the completion collector and full testing-standard validator are unavailable. Once those are implemented and qualified, the admitted source/interface subject becomes primary for current facts, and source changes make its applicability stale until re-admitted. Whole-codebase acceptance and deployed installation remain separate.

The orchestrator owns this reconciliation and final integration. Module threads return candidates and evidence references. Quality and security threads check the actual integrated subject through their admitted evidence paths. Release/operations supplies installed-artifact and operational observations. Parent acceptance remains independent of child success.

## Gate mapping to the existing atlas

| Work boundary | Existing task or contract | Required observation |
| --- | --- | --- |
| Narrow task/adapter implementation | T01–T03 and the module's explicit task set | Contract-preserving source and boundary fixtures |
| Closed execute/verify/repair loop | T06/T07 | Failed candidate repaired and reverified, cancellation/recovery and truthful exhaustion |
| Quality baseline | T25 | Applicable Rust formatting, pedantic Clippy, tests/rustdoc, Julia and cross-language checks on a real candidate |
| Threat model and trust boundaries | T26/T15 | Admitted execution/collector custody, permissions and hostile fixture results |
| Integrated failure behavior | T17 | Intended failures actually reach the boundary; invalid or unmeasured cells remain visible |
| Security finding closure | T27 | Reproduction, repair, regression and explicit residual disposition on the integrated release |
| Upgrade, backup and rollback | T18 | Package/configuration identity, preserved obligations and useful independent restore readback |
| Pilot and scoped release | T19/T20 | Declared real-work outcomes and supported operational limits |
| Bash, Pi, skills and workflows | T28/T29 | Literal values, producer verdict, cancellation, version/authority boundaries and coherent return paths |
| Julia/PyTorch/neural operators | T21/T23/T24 | Bounded exchange and measured domain/runtime qualification; optional work can remain rejected or deferred |

This table navigates existing obligations; the owning task acceptance text controls their exact scope. A module can have a current testing observation while integration, hardening, deployment or task acceptance remains unresolved.

## Recovery, drift and redundant copies

Publication journals retain frozen output blobs and prior bytes for changed files. Replaying a partially completed journal accepts an already-published matching file, rejects a changed file and continues only the remaining documentation writes. Missing or corrupted journal blobs fail verification. No automatic deletion, rollback of user edits or engine redispatch occurs.

The synchronizer serializes its own invocations with an advisory lock and checks expected hashes before the batch and immediately before each replacement. Other writers must remain quiescent on the affected files during publication. Ordinary filesystem rename does not provide a content-hash compare-and-swap against a non-cooperating editor in the final check-to-rename interval. The tool detects observed conflicts and retains backups; it cannot promise exclusion of an arbitrary writer in that interval.

The ultra map and native tree provide alternate routes if one index is overlooked. The codebase carries its own portable map and module contracts; the atlas and vault retain their corresponding maps. A complete publication receipt binds the generation and output hashes without recursively hashing itself. This is navigation and publication redundancy; it does not replace storage redundancy, retention policy or off-site disaster recovery.

## Proof and limits

The corpus checks include every primary module and all planned support paths; unique comment anchors; source-to-note and note-to-source routes; task text and source-copy parity; module/test/hardening/deployment separation; stale/missing evidence; zero or skipped checks; concurrent edits; partial publication and resumption; and actual Obsidian navigation.

The original design and reference-example checks retain their own subjects. Comment-only Cargo, Julia, SQL and deployment placeholders are intentionally incomplete configuration; parsing or executing them is not a release check. HTML/browser behavior, real engine operation, producer authentication and production acceptance remain outside this documentation proof.

## Recorded maintenance-tool verification

Run `tools/corpus-sync --verify-tooling` after changing maintenance helpers or their bound conventions. Test discovery enumerates the actual top-level `test_*.py` files, requires every discovered file to be admitted in `build_vault.py`'s helper manifest, and rejects empty files, import errors, zero total cases and duplicate case IDs. `testing_standard.py` is a helper, not a test module. Both normal and optimized Python profiles run the complete discovered suite with warnings treated as errors; missing, skipped, failed or errored cases cannot pass. Named case counts are maintenance evidence, not primary credits toward any engine module's ≥50-case requirement.

The [tooling receipt](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Atlas%2Fevidence%2Fcorpus-tooling-verification.json) binds exact helper/convention hashes, discovered case IDs and the per-file case inventory, both executed profiles, observation time, Python version and executable path, fixed scope/limitations and SHA-256 hashes of raw output. Counts are derived from discovered and executed cases rather than a hand-maintained denominator. The validator rejects a changed schema, enlarged scope, omitted fields/profile/cases, altered per-file mapping or counts, changed interpreter identity, modified log or stale subject. A reported interpreter version/path is not binary or whole-environment attestation. This operator-owned self-check receipt does not authenticate an independent producer or prove sufficient oracles for every helper.

[Normal output](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Atlas%2Fevidence%2Fcorpus-helper-tests.txt) and [optimized output](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Atlas%2Fevidence%2Fcorpus-optimized-guard-tests.txt) retain actual results. The optimized log keeps its historical filename; it now covers the whole discovered suite. Current counts come from the receipt. Publication verifies the receipt against current discovery before staging and again at readback. `--check` inventories current cases and verifies receipt bindings; it does not rerun the suite or engine checks. A test-file admission does not itself make its tests meaningful: behavioral oracle review remains necessary.

The separate bounded security recheck closed the reproduced optimization-bypass, staging/collision and symlink gaps. Duplicate topology, stale catalogue hashes, authored new-path collisions and directory symlinks are now rejected before publication; CRLF source-body preservation and future-dated receipt rejection are covered by fixture tests. This records the scope of the review, not a security certification. The non-cooperating-writer rename interval remains the explicit protocol limitation above.

Requirement-to-module anchors follow the requirement’s exact declared task IDs into existing module task sets. The generated anchor graph retains `via_tasks` witnesses and requirement notes display them. A global planning-authority requirement without assigned tasks (R16), decision records without structured module assignments, and aggregate/index/source views retain their explicit owner and navigation routes; the mapper does not infer new module ownership from prose.

The [15 September corpus alignment review](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Atlas%2Fevidence%2Fcorpus-alignment-20260915.md) retains the independent facet/module matrix, exact task witnesses, file hashes, URI checks and live Obsidian observations. It distinguishes directly assigned facets from global/index/archive routes and records the requirement-anchor correction.

## Active module testing qualification

[The active testing standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing) adds a minimum of 50 distinct qualifying cases per module and zero baseline warnings/errors, including pedantic Clippy, to the original atlas predicates. Every module contract and source-comment anchor points to this standard. Its exact JSON/Markdown copies and per-module policy hashes are publication-checked.

The existing v1 observation schema does not qualify case identity, complete subject/profile coverage, assimilation, mutation sensitivity or zero diagnostics. An aggregate executed count cannot satisfy the new floor. Full-standard qualification remains unassessed until the dedicated collector described by the standard is implemented and verified. Changes to the standard revision invalidate affected qualification; they do not rewrite historical receipts or manufacture case counts.

## Public consumer interfaces and current code authority

[The module interface convention](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts) defines 22 supported consumer boundaries and their semantic operations. Its JSON/Markdown copies in the atlas, vault and codebase are byte-checked, and every module contract carries the convention hash. Exact source symbols, callable signatures, examples, schemas and compatibility tests must be supplied by authorized implementation before acceptance. No external library target, public network listener or world-accessible socket is created by documenting a public interface.

For each operation map the owner, consumers, typed request and response/error, capability, identity, version, cancellation, deadlines/backpressure, concurrency, idempotency and readback. Reuse the existing APIs, IPC custody notes and action IDs. `tools.list` and `tools.inspect` are planned discovery actions, not available engine commands. Accepted code becomes the factual owner of those interfaces at its admitted revision; code-derived views must never conceal a current-versus-desired difference or overwrite the accepted source with older planning content.

## README, tests and migrations as corpus anchors

The codebase README links to `docs/public-interfaces.md`, `tests/README.md`, `migrations/001.sql`, the ultra map and this protocol. The test README maps all 22 modules to interface obligations, the ≥50-case rule and zero baseline warnings/errors, including pedantic Clippy and all other applicable checks. Every planned test/evaluation stub retains its path owner and supporting modules; shared tests do not duplicate primary case credits. Case records bind independent oracles, exact subjects/profiles, raw results, assimilation pairs and mutation dispositions. These records and collectors remain to be implemented.

`migrations/001.sql` remains comment-only. Its store-owned contract now includes atomic application, checksum/version compatibility, referential integrity, crash and concurrent-start behavior, fresh/upgrade/restore tests and independent schema readback. No SQL was executed. Released migrations must become immutable history: choose and verify a checksum/freeze policy before accepting the first migration. The recommended whole-file checksum requires freezing its managed comments and moving subsequent navigation edits to a sidecar. The current anchor updater has no admission-aware freeze guard; implementing and verifying that guard is a release prerequisite, not a completed capability. Never silently refresh a released migration under this mutable-stub workflow.

The source/test/migration files at an accepted revision are primary factual evidence; actual database schema/readback is primary for deployed schema state. Requirements and supported compatibility obligations remain in their normative owners. An update to comments or notes is not a migration, test pass, admission or installation.

## Reflection assimilation and return-anchor custody

Read [the scoped learning register](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex), [the delivery workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FCorpus%20Assimilation) and [the quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start). The current reviewed set contains 13 original reflections and 14 learning contracts, linked by 126 explicit lesson/module assignments. Every module has relevant guidance; counts describe coverage, not prevention or production quality.

Each learning records its historical source/heading/line, local trigger, antipattern, useful alternative, intended mechanism, proposed detector, intended-fault control, benign mirror, placement before the hazard, module owners, limits and retirement criterion. Keep `engine enforcement unqualified` until actual engine evidence establishes otherwise through a future authorized admission path. Source commands, inherited personas, historical prototype statistics and prior exceptions never become grants or present capabilities. The conflict register preserves corrections and exclusions, including the distinction between Podman mount access and SELinux label policy. The ≥50-case and zero-warning requirements remain unchanged.

The source capture owns the reviewed wording. The live diary owns its authored narrative. The updater admits only the 13 specifically catalogued `Reflections/*.md` files plus `00 - Diary Index.md` and `00 - Master Index.md`. It adds or refreshes one block bounded by `HEE3-DIARY-ANCHORS-BEGIN` / `HEE3-DIARY-ANCHORS-END`. Source wrappers and lesson notes link to the original reflection; original reflections return directly to relevant module stubs, contracts and all corpus masters. Source comments link back to their applicable reflections and lessons. The two diary masters provide an entry route into the v3 register. No general diary rewrite or watcher is installed.

Source acquisition requires regular files and an admitted byte bound, and rejects symlink paths. Captured bytes are hashed before navigation is added. Managed block parsing preserves the exact authored UTF-8 bytes and line endings outside the block. For a reflection, that preserved body must match its captured SHA-256; for a diary master, the journal preserves the acquired whole-file before image. The final diary check compares bytes, not newline-normalized text. Duplicate/malformed markers, unknown block content, deletion of a previously managed block or a changed captured narrative stop publication. Preserve the change, review and reconcile its owner; never silently overwrite it.

A changed source requires a fresh reviewed capture of the authored body with the owned navigation block excluded. Retain the old capture and explicitly supersede its lesson locators/claims; update the catalogue's source identity, evidence locators, adoption decision and affected module assignments. Do not change a hash merely to get a green result. The routine updater does not perform source recapture or automated adoption.

Before writing any pending marker, synchronization acquires all three prior publication records and requires their identity/state to agree. A prior record and each marker's byte hash are retained in a snapshot named by the next generation. Ordinary authorized changes to owning atlas sources are expected; they need reconciliation rather than a demand that old output hashes still match. Existing guards separately protect generated vault notes, newly claimed output paths and source bodies. Prior marker disagreement must be reconciled before publishing; it cannot be erased by replacing all markers with a fresh pending value.

A complete prior generation admits only its current diary block hash. Once introduced, the diary inventory must cover every admitted reflection and both masters with canonical hashes; a partial ledger cannot masquerade as first installation. A pending generation retains only its recorded old/new block hashes and inherited pending allowlist for interruption recovery; completing that generation removes obsolete hashes from the next ordinary update's authority. The same frozen publication journal includes the 15 admitted diary files with expected before hashes, output blobs, permissions and retained prior bytes. A changed file refuses replay. The three final publication records bind those diary outputs along with the other corpus files; no fourth independent scheduler or truth ledger is introduced.

## Learning loop, evidence and maintenance cost

Use the ten-step workflow: identify → decide scope → bind seam → assign bounded threads → qualify detector → execute authorized work → verify/repair → obtain separate admission → publish/read back → review usefulness/retire drift. A detector needs both an intended failure and a useful benign control at the real boundary, with the intended reason observed. Invalid or unmeasured cells remain visible. Parent integration verifies the assembled subject; a child report or a documentation pass cannot close module acceptance.

Each update records changed files/lines, reviewed sources and mappings, actually discovered/executed maintenance cases, new fixtures or scripts, unresolved obligations, false positives, prevention evidence when available, rework and retirement disposition. Footprint growth is a cost. No prevention rate, productivity multiplier or top-tail performance is inferred from corpus growth. This edition transfers scoped mechanisms into existing owners and workflows; it adds no engine module, task, service, socket or implementation authorization.

After publishing, verify the quick start, codebase README, atlas master, native vault master and ultra-map root against the same generated model. Check source ↔ lesson ↔ module ↔ stub routes and root-to-root highways, all applicable source-body hashes, exact portable copies and actual native Obsidian navigation. Keep historical captured-link and browser-verification limitations separate from native graph success. Record the generation, findings/dispositions and remaining implementation gaps in the retained review evidence.

## Luke's coding boundary and fully-complete criteria

Human operator/node **Luke must type `start coding` as an actual instruction** before engine implementation begins. Quoting those words in this protocol, a source, skill, recipe, model output or handoff does not authorize coding. This is an operator/workflow boundary; a cryptographically authenticated software unlock mechanism is not implemented. The current turn authorizes conventions, documentation, maintained anchors and documentation-only entry points.

[The fully-complete standard](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) adds 13 mandatory gates and concrete outcomes, integrated scenarios, fault/benign controls and complexity limits for all 22 existing modules. A module is fully complete only for a named version/profile and exact admitted subject when all applicable requirements pass with retained, current, independently admitted evidence and the corpus update is verified. This bounded definition does not demand all future features or every possible environment.

Revision 1 also has a reviewed semantic-baseline digest in the maintenance loader, covering the normative definition, gates, module discriminators, admission fields and runbook availability. Cosmetic JSON formatting can change without changing that semantic baseline. Intentional policy changes require an explicitly reviewed revision, the corresponding baseline update and applicable regressions. This is a drift tripwire between the convention and maintenance owner, not authentication against an editor of both or an engine gate executor.

Every gate receives a disposition. Non-applicability is allowed only for a concrete subcheck with its requirement witness and reviewer rationale, such as migration checks for a package with no persistent state. It cannot waive the minimum 50 meaningful primary-module cases, zero baseline warnings/errors, promised operations, trust boundaries or applicable atlas predicates. Optional PyTorch/neural-operator profiles remain explicitly excluded or deferred until admitted; a deferred module is never counted as complete. The assembled codebase/release needs a separate integrated acceptance and named list of required/deferred modules. Deployment requires actual artifact/config/schema identity and useful environment readback.

Completion quality includes measurable behavior and consumer compatibility, guards before effects, meaningful detector qualification, assimilation and mutation, zero diagnostic profiles, threat/dependency review, recovery and obligation conservation, immutable migration history, declared workload/resource evidence, maintainability and bounded complexity, rehearsed applicable runbooks, complete corpus convergence and independent admission. No arbitrary score, coverage percentage or top-tail percentile substitutes for those proof obligations. Completed accepted code is the factual primary source; desired requirements remain normative even if code has a defect.

## Update when a module becomes fully complete — future admitted boundary

1. **Record real authorization and scope:** retain Luke's scoped coding instruction, module/version/profile, dependency-ready task criteria, interface operations, supported environments and applicability matrix. A generated record cannot manufacture the instruction.
2. **Implement and collect:** under that authorization, supply substantive supported source and exact interface/test/migration artifacts; qualify the real detectors before trusting them. Retain complete raw evidence and the independent producer/reviewer path.
3. **Verify, repair and reverify:** meet every applicable gate and owning criterion, preserve failures/unknowns and settle or explicitly retain effect/resource obligations. The full-standard collector and independent admission must themselves be qualified. They do not exist yet.
4. **Admit the module:** a trusted admission binds exact source, standards, interface, dependency/toolchain/profile, fixture/case and raw-evidence identities plus reviewer dispositions. Existing v1 observation registration cannot perform this step. There is no implemented `complete` command or accepted admission schema today.
5. **Derive current facts:** publish supported interface symbols, errors, examples and compatibility from the admitted source. Record any desired-versus-current gap. Do not overwrite code with older plan text or mark unrelated/deferred modules accepted.
6. **Reconcile every applicable facet:** update README, test matrix, migration comments or immutable-history sidecar, quick start, completion profiles, runbook definitions, generated Justfile, learning placements, atlas, vault, ultra map, code stubs/implementation anchors, master indices and context handoff from their owning records. Keep the normative plan history and exact accepted subject distinct.
7. **Publish and read back:** use RB02 and the existing synchronizer to bind one documentation generation across the corpora. Check actual recipe/source inventory, command availability, code-to-note and note-to-code paths, all return links and live native graph. A future collector may invoke this existing maintenance boundary; no watcher or second scheduler is added.
8. **Retain separate outcomes:** module admission, assembled release, deployment observation and corpus publication retain independent identities. If documentation fails after code admission or an external effect, preserve that fact and resume the documentation journal. Do not rerun the external effect or rewrite acceptance to hide the failure.

A changed bound source/interface/standard/dependency/toolchain/fixture/profile makes affected applicability stale until reverified and re-admitted. Cosmetic documentation changes follow their declared hash boundary. Before the first migration release, implement and verify the checksum/freeze rule: recommended whole-file immutable history with frozen managed comments and a navigation sidecar. The current mutable-stub updater is not safe to use for accepted migration history and cannot claim the future freeze gate is implemented.

## Justfile and runbook ownership, paths and evidence

[The operational catalogue](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) maps the current generated `justfile`, five procedure runbooks, their owners, availability and module return routes. `completion_standard.py` owns the twelve static documentation/maintenance recipes. They delegate to existing reads and corpus commands; engine build/test/migrate/complete/deploy recipes do not exist. RB01/RB02 are current documentation procedures; RB03/RB04 are planned module/release procedures with explicit blocking prerequisites. Markdown procedure documentation is not automatic compatibility with the installed habitat-runbook TOML executor.

The project discovery root is `/var/home/herdr-engineering-engine-v3`; the Toolbx view is `/run/host/var/home/herdr-engineering-engine-v3`. The inspected host PATH includes normal user binary directories but has no `just` executable; Toolbx has `just` 1.57.0. Adding a Justfile directory to PATH would not fix that. The quick start preserves direct host `tools/corpus-sync` commands and the verified Toolbx `just --justfile …` route. No PATH edit, binary installation, scheduler modification or Arena baseline acceptance is implied by this update. Recheck environment identity in a new session.

Compare actual discovered recipes and their bodies against the owned generated file, with an inert list default, fixed literal commands, strict producer status and disabled dotenv loading. For future dynamic inputs use a separately qualified literal argv/positional boundary; never interpolate untrusted values into shell source. A new recipe must declare its procedure/action owner, purpose, prerequisites, input/result/error/authority/effect, working directory/environment, deadlines/cancellation, resource/cleanup and retry/readback contract. Test actual intended failure and benign behavior; source freshness, formatting, listing and parse success alone are insufficient. Do not assume a dry run or a runbook precondition is side-effect free.

The reviewed Toolshed and official Just sources are recorded in `evidence/completion-20260915/sources.json`. Their historical examples and scheduler observations retain their dates. The existing Arena monthly watcher remains scoped to its own sources. This project's explicit publication checks verify its generated Justfile/runbooks; any future scheduler enrolment or accepted baseline requires its own scoped change and verification.

## Context handoff and executive view

[The restart note](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) links to the actual `.md` in `corpus/CONTEXT_HANDOFF.md`, the codebase README, quick start, completion standard, relevant atlas tasks, Fedora Master Index and Prime. A fresh context must read it, use `$prime habitat`, follow the named v3 atlas route, establish source-backed understanding and revalidate current state before continuing. The handoff cannot restore hidden memory or grant coding permission. Preserve current worktree ownership, evidence scope and unread dependencies explicitly.

[The executive summary](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Executive%20Summary) explains intended capacities, current delivered artifacts, architecture and quality constraints without presenting planned routing or deployment as available. Update it from current accepted source and corpus state after an admitted module changes supported behavior; do not market an unimplemented or excluded capacity as complete.

## Daybreak security profile and closure

`SECURITY_PROFILE_habitat_engine.json` and `completion_standard.py` own the additive 22-module security matrix, RB05 and config guide. The completion convention retains its four baseline runbooks; the security profile adds RB05. The generated `security-profile` recipe reads the profile. It never runs `/model`, a scan, an engine action or an acceptance command.

Use `gpt-daybreak-blue-latest` through the Codex `/model` picker and verify effective identity, effort and any `review_model` override. Requested alias, observed model metadata and immutable revision when exposed must be distinct; unavailable identity/provisioning remains an explicit gap. No global Codex configuration is changed by this corpus update. Consult [the security profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) and [RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05).

Retain exact source/interface/config/toolchain subjects, bounded threat scope and budget, findings with locations/triggers, fault and benign controls, repair and independent re-verification, residual liabilities and reviewer identity. Model output is review evidence, not completion authority. After any authorized hardening, regenerate README, quick start, config guide, migration comment/immutable sidecar, module cards/anchors, runbooks, Justfile, atlas, ultra map, vault/master indices and handoff; require current tooling receipt, matching complete publication records and live native readback. The semantic baseline refuses silent fallback, runtime activation or fabricated security qualification. All engine modules remain unassessed until implemented and admitted through qualified collectors.

## Graphify sidecar closure after core publication

[Graphify guide](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) maps the source-bound graph, tree, overview and module-cluster canvas. `graphify_corpus.py` owns this additive analysis procedure. `corpus-sync` finishes the existing core journal and verifies the live vault first, then freezes the complete publication inputs and invokes local Graphify extraction/analysis. `corpus-sync --check` requires both the core and all three graph copies to pass. A core completion marker alone does not imply a fresh graph; graph failure is explicit and does not replay core or engine effects. Use `--graph` to recover/rebuild the analysis sidecar only after the core passes.

The graph source inventory includes published facet files, registered atlas counterparts and helpers. Generated graph directories are excluded from their own inputs. Exact paths remain distinct even when bytes match. Literal/native links and declared task/module/API/IPC/action/context/gate relationships carry source locators; AST relations retain producer origin separately from confidence. Full graph JSON is a directed multigraph; Graphify clustering sees a declared simple undirected projection. Computed clusters, degree and node counts cannot accept code or amend declared module boundaries. Binary semantics and unrestricted prose inference are outside this extraction and remain explicit coverage limits.

Graphify artifacts are identical in codebase `corpus/graphify-out`, atlas `graphify-out`, and vault `Atlas/graphify-out`. A retained source/output receipt binds the core generation without replacing its authority. Writes use the existing frozen per-file publication journal, with source recheck and last-written manifests; partial sidecars fail readback and can resume their frozen journal. Authored changes refuse overwrite. The graph guide, module/stub anchors, README, quick start, master indices, ultra map, runbooks and context handoff route to these artifacts. No automatic Graphify Git hook, watcher, provider backend or additional engine coordinator is installed.

## Adopted readiness requirements and change propagation

The [adopted readiness convention](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/READINESS_CONVENTION_habitat_engine.json) is the single owner of the adopted recommendation clauses. Its [full plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md), [native readiness tree](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex), module/task views, standards, procedures and graph are projections. Historical advisory captures and the original seven-facet assessment retain their original scope. R90 packages contain no task state; the original 29 task records remain controlling.

1. Identify affected criterion IDs, module/consumer contracts, original task prerequisites, public interfaces and protected shared files. Use a bounded context packet with exact source/criterion identity, owned paths/resources, grants, limits and return obligations. Assign one writer per shared interface, lockfile or migration.
2. Change the narrow owner. For readiness revisions, review the bound plan/testing/interface/completion/security/assessment hashes and deliberately update its revision/semantic guard with independent disposition. Never refresh a baseline merely to clear a failed check. Preserve existing acceptance, the ≥50 distinct case floor and zero baseline diagnostics. A source or criterion mismatch invalidates affected evidence.
3. Maintain separate desired requirements, implemented accepted facts, scoped observations, module/release admission and deployment readback. Existing stubs have no engine qualification. A child result, model review, graph refresh or successful documentation update cannot admit a parent or raise a score.
4. Once coding is explicitly authorized, qualify the launch/collector boundary and its fault/benign controls under the original T25/T26→T02/T03/T04/T05→T06 order. Repair and reverify the exact subject; retain complete diagnostics and intended-detector outcomes. Stop dependent work for unknown effects, broken custody, unavailable oracles, failed security boundaries, exhausted resources or unauthorized scope. Continue independent authorized work where useful.
5. Publish with the existing maintenance flow and require complete core and Graphify readback for the same generation. Check exact JSON/Markdown parity, all 22 module bindings, 44 clause coverage, original-task links and reciprocal facet/source anchors. Graph recommendation arrows express grouping prerequisites; reverse links are navigation only. The graph never replaces task authority.
6. After actual module or release admission, update current implemented interface facts and affected evidence applicability through their owners. Compare predicted versus actual touched modules, compatibility impact, escaped defects, rework, context omissions and maintenance cost. Keep failed/stale/excluded observations visible and retain historical scores; a later independent scoped assessment may revise only supported facets.

Release/restore changes must freeze released migration bytes or move navigation to a sidecar, preserve current/failed state, and reconcile post-backup effects/usage/obligations before restoring older state. An event epoch alone does not prevent external replay. Optional T23/T24 remain outside mandatory startup until domain-valid held-out benefit covers their runtime, integration and maintenance costs.

[Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) ↔ [Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md) ↔ [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) ↔ [Ultra map](file:///var/home/herdr-engineering-engine-v3/corpus/ULTRA_MAP.md) ↔ [Restart handoff](file:///var/home/herdr-engineering-engine-v3/corpus/CONTEXT_HANDOFF.md).

## Resolved contract revisions and proof custody

[Six selected design contracts](CONTRACT_DECISIONS_habitat_engine.md) ↔ [Current readiness owner](READINESS_CONVENTION_habitat_engine.json) ↔ [Codebase contracts](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). RC01–RC06 are resolved design decisions with pending original-task proof. Edit their single owning records, not generated code/vault copies. Review and bind local/source observations, selected limits, dependency versions, schema and scope. A changed bound/profile requires a versioned decision and invalidates dependent proof; do not relabel design selection as qualification. Future suite/package/SQL hashes come from actual produced bytes. Keep 29 original tasks, 22 module IDs, 10 APIs, 7 IPC records, 21 actions, 66 public operations, 13 completion gates and source-body custody intact.

Regenerate contracts, module/stub anchors, configuration, tests, runbooks, README/quick start, native masters and Graphify in the existing publication transaction. Verify exact three-corpus copies, all six design states versus pending proof, reciprocal RC/module/task links, source preservation, diagnostic-clean maintenance checks and graph freshness. Retain the current pre-change snapshot and final readback. Luke’s actual `start coding` instruction is still required for engine implementation.

## Procedure and recommendation route verification

The existing operational owner `completion_standard.py` projects twelve literal Just recipes and five runbook focus maps. The readiness convention remains the owner of all44 clauses, ten work groupings and six resolved decisions; procedure routes do not change those requirements. [Runbook matrix](RUNBOOKS_habitat_engine.md) ↔ [Graphify map](GRAPHIFY_GUIDE_habitat_engine.md) ↔ [Module ultramap](ULTRA_MAP_habitat_engine.md).

After changes, verify every recipe's exact body/effect and failure propagation, all recommendations' applicable-module and primary-owner routes, original task coverage, reciprocal source/runbook anchors, and graph nodes/edges for the same complete generation. Preserve original task state, stub bodies, retained concurrent diary edits and the coding boundary. Refresh README, quick start, runbooks, Justfile, masters, standards, module cards and all three graph copies through the normal publication protocol. No graph sidecar may mark an unqualified engine module accepted.

## Progressive module context maintenance

[Module context workflow](file:///var/home/herdr-engineering-engine-v3/docs/module-context.md) connects the quick start, 22 module scout cards, original task/contracts, caller/callee returns and deeper corpus branches. Generated code cards live in `docs/module-context/`; native cards in `Context/Modules/`. Maintain module_context.py reading questions and existing catalogue/plan owners rather than a second task ledger. Skill source lives in `agent-skills/hee-module-scout`; published copies live in code `corpus/agent-skills/hee-module-scout` and vault `Atlas/agent-skills/hee-module-scout`.

After a relevant source/interface/schema/config/fixture/toolchain change, refresh both dependency and consumer context, invalidate affected briefs, and preserve source-read versus relationship-coverage gaps. Update the owner, run maintenance verification, publish the corpus/Graphify and check exact context-card/skill-copy parity. Refresh the installed personal skill from the reviewed published copy, preserving unrelated edits; run the skill validator and check its byte identity. Personal skill installation is separately verified documentation state, not an engine skill deployment or admission event.

The workflow must retain all 22 module routes, exact existing facet destinations, scoped quality/tool availability, source provenance and deeper references. Context preparation never turns a proposed engine validator into an executable tool, historical evidence into current behavior, or a quoted coding trigger into authority. Use the existing handoff/evidence location for briefs; do not create another scheduler, corpus crawler or acceptance registry.

## Corpus architecture schematics and completion boundary

[Six corpus schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) ↔ [Quick start](file:///var/home/herdr-engineering-engine-v3/QUICK_START.md) ↔ [Clickable architectural drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html).

CS01 maps fact owners and projections; CS02 separates future module admission from integration and corpus refresh; CS03 traces the existing publisher and its failure/recovery paths; CS04 follows exact-subject evidence; CS05 traces context and cross-module invalidation; CS06 provides the six-cluster matrix and every module pointer. The explanatory data/layout owner is `corpus_schematics.py`; existing requirements, path catalogue, evidence rules and implementation owners remain controlling. Generated JSON binds source hashes; Markdown/HTML/SVG/native notes derive from that same model and are included in complete publication checks. Update the owner, verify maintenance tooling, regenerate, then check the full core and graph.

**Present capability versus future completion:** the current completion loader explicitly retains unassessed modules and rejects accepted-state promotion. Graph validation likewise assumes zero admitted engine modules. A future trusted completion collector, admission policy and reviewed schema/loader/projection evolution must exist and be qualified before accepted-module or assembled-release facts can populate these views. Do not edit accepted arrays or weaken a guard to make a drawing appear current. A scoped observation can be retained and published today without promoting a module, task or release.

A core complete receipt can be written before Graphify refresh fails. Full-corpus closure requires the agreeing complete core records, exact output/readback hashes AND the current graph manifest/copies. `files_published` in a helper journal is not the final complete generation. Core `--resume` applies an explicitly reviewed frozen journal and then performs fresh synchronization and graph work. Graph-only recovery uses the existing `--graph` path after verifying the current core; do not resume a graph bound to a different generation. A failed pre-freeze run may have no resumable journal and needs a fresh bounded repair. Preserve retained module admission and evidence if documentation fails; never replay engine effects to repair documentation.

## Start-coding pointer across context windows

[Canonical restart note](file:///var/home/herdr-engineering-engine-v3/corpus/CONTEXT_HANDOFF.md) ↔ [Quick start](file:///var/home/herdr-engineering-engine-v3/QUICK_START.md) ↔ [README](file:///var/home/herdr-engineering-engine-v3/README.md) · [Vault handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff).

Maintain this existing pointer through `completion_standard.py`; do not create a second task/admission ledger or edit its generated copies independently. Preserve a compact current objective, actual authorization, ownership and repository observations, the Prime/Fedora and named-atlas route, original dependency-ready task selection, module scouting, verification/repair/parent integration and core/graph closure. Keep historical review subjects dated; the current complete publication and graph records are the freshness entry, avoiding a self-referential generation promise inside the note.

README and quick start return directly to the canonical Markdown pointer. Migration/configuration stubs carry the route only in their managed comments, with named owners and no active SQL/configuration change. The handoff links back to those files, configuration guidance and the corpus protocol/ultra map. After authorized changes, regenerate through the existing publisher and verify these reciprocal routes in the actual native graph and source comments; Graphify refresh follows complete core publication. A launch example is inert until Luke sends an actual coding instruction. Once authorized, resume within that scope without asking for the same permission again.
