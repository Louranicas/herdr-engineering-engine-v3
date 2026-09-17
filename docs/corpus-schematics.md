# How the deployment corpus fits together

[Schematic index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) ↔ [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) ↔ [Update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) ↔ [Module context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) ↔ [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) ↔ [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) ↔ [Graphify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md)

[Open the clickable architectural drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html) · [Derived diagram and pointer data](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/CORPUS_SCHEMATICS_habitat_engine.json)

Six schematic views explain ownership, module completion, publication/recovery, evidence, context impact and all 22 module routes. They supplement the original engine API/socket/action schematics. Solid arrows transfer a requirement, artifact or control step; dashed arrows carry feedback, failure or navigation as named in the edge table. Each node has a source pointer and reading locator. Two-way navigation never implies two-way edit authority.

**Current boundary:** the documentation publisher and graph checks exist; engine modules remain unassessed. The completion/admission collector and accepted-state projection do not yet exist. A future accepted module requires reviewed loader/schema evolution before its accepted facts can be projected. No diagram implements that capability.

| View | Question | Pointer |
| --- | --- | --- |
| CS01 · One corpus, explicit fact owners | Follow an owning record to its projections and return through stable anchors. Navigation redundancy does not grant competing edit authority. | [Vault drawing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01) · [HTML section](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html#CS01) |
| CS02 · Module completion to corpus refresh | Separate the future engine completion chain from the documentation update tools that exist today. | [Vault drawing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02) · [HTML section](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html#CS02) |
| CS03 · Publication, failure and recovery | Read the real documentation publication sequence, including the gap between core completion and graph completion. | [Vault drawing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03) · [HTML section](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html#CS03) |
| CS04 · Evidence earns a scoped verdict | Trace requirements through a qualified oracle and exact-subject evidence before any module or release claim. | [Vault drawing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04) · [HTML section](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html#CS04) |
| CS05 · Context, change impact and return paths | Keep the full corpus reachable while scouts and coding threads read only the evidence needed for their current module and boundary. | [Vault drawing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05) · [HTML section](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html#CS05) |
| CS06 · Module clusters and corpus pointers | Matrix cells count declared request and return arrows between six ownership clusters. The directory below maps every one of the 22 modules to code, context, completion and public contracts. | [Vault drawing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06) · [HTML section](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html#CS06) |

## CS01 · One corpus, explicit fact owners

![One corpus, explicit fact owners](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics/CS01.svg)

**Purpose:** Follow an owning record to its projections and return through stable anchors. Navigation redundancy does not grant competing edit authority.

**Invariant:** Accepted code owns implemented facts at its admitted revision; requirements remain normative. Source kind and candidate evidence do not establish accepted behavior. Links are navigation unless the edge table says derivation.

| Pointer | Component / availability | Owning source and exact reading locator |
| --- | --- | --- |
| requirements | Atlas requirements and standards · Normative design owner | [PLAN_habitat_engine.json](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/PLAN_habitat_engine.json) — requirements; task_graph; module_atlas |
| paths | Stable IDs and path ownership · Catalogue owner | [corpus_catalogue.json](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/corpus_catalogue.json) — modules; planned_paths.owner |
| observations | Raw evidence and scoped receipts · Observation; not admission | [evidence-index.json](file:///var/home/herdr-engineering-engine-v3/corpus/evidence-index.json) — Read current applicability and retained logs |
| derive | Derive views from their owners · Existing documentation tooling | [UPDATE_PROTOCOL.md](file:///var/home/herdr-engineering-engine-v3/corpus/UPDATE_PROTOCOL.md) — One owner for each kind of fact; Systematic update loop |
| code | Codebase and accepted source · Source kind separate from admission | [README.md](file:///var/home/herdr-engineering-engine-v3/README.md) — Current versus required behavior; source comment anchors |
| atlas | Deployment atlas and ultra map · Generated plans and routes | [MASTER_INDEX_habitat_engine.md](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md) — Master and ULTRA_MAP_habitat_engine.md |
| vault | Vault, themes and context cards · Generated native notes | [00 - Master Index.md](file:///var/mnt/STORAGE-10TB/fedora-obsidian-vaults/herdr-engineering-engine-v3.vault/00%20-%20Master%20Index.md) — Module stems, note clusters and captured-source wrappers |
| graph | Graphify and publication identity · Derived graph; scoped proof | [graphify-guide.md](file:///var/home/herdr-engineering-engine-v3/docs/graphify-guide.md) — Core generation, manifest, typed graph and read-only queries |

| Arrow | From → to | Meaning | Condition / evidence |
| --- | --- | --- | --- |
| E01 | requirements → derive | forward | Normative requirements and original task predicates constrain every projection. |
| E02 | paths → derive | forward | Map stable module IDs, primary/support paths and managed comment anchors. |
| E03 | observations → derive | forward | Validate declared subject and log consistency; preserve failed, stale and unqualified records. |
| E04 | code → derive | return | Current source identity feeds observations; accepted facts require future trusted admission support. |
| E05 | derive → code | forward | Refresh generated documentation and managed comments; preserve developer-owned source bodies. |
| E06 | derive → atlas | forward | Regenerate owned plan supplements, context routes and ultra map. |
| E07 | derive → vault | forward | Regenerate native views, source wrappers and controlled diary return links. |
| E08 | code → atlas | return | Bidirectional root navigation; never infer authority from a backlink. |
| E09 | atlas → code | return | Return to the codebase master and exact module source; requirements remain separately owned. |
| E10 | atlas → vault | return | Portable copies and native routes return to exact owners. |
| E11 | vault → atlas | return | Return to the owning atlas and ultra-map records; native notes do not replace those owners. |
| E12 | vault → graph | forward | Only a completed core generation may feed the graph refresh. |
| E13 | graph → code | return | Graph query pointers return to code and evidence; full check binds graph to the same core. |

**Related notes:** [Highways](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FHighways) · [Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Update Protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) · [Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)

[Schematic index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) ↔ [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) ↔ [Update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) ↔ [Module context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) ↔ [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) ↔ [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) ↔ [Graphify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md)

## CS02 · Module completion to corpus refresh

![Module completion to corpus refresh](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics/CS02.svg)

**Purpose:** Separate the future engine completion chain from the documentation update tools that exist today.

**Invariant:** No automatic admission collector exists. Current loaders explicitly retain unassessed modules; accepted-state projection needs reviewed tooling evolution before it can be used. Released migration bytes require a qualified freeze guard and navigation sidecars.

| Pointer | Component / availability | Owning source and exact reading locator |
| --- | --- | --- |
| scope | Scout module and original task · Current planning workflow | [module-context.md](file:///var/home/herdr-engineering-engine-v3/docs/module-context.md) — Scope, task prerequisites, ownership and source/relationship coverage |
| candidate | Implement authorized candidate · Future; actual Luke instruction | [03-module-completion.md](file:///var/home/herdr-engineering-engine-v3/runbooks/03-module-completion.md) — Original task DAG; exact source/interface/test/migration subject |
| verify | Execute checks; retain evidence · Future engine qualification | [testing-standard.md](file:///var/home/herdr-engineering-engine-v3/docs/testing-standard.md) — At least 50 meaningful owned cases; applicable profiles and raw logs |
| repair | Repair or retain unresolved state · Future repair loop | [completion-standard.md](file:///var/home/herdr-engineering-engine-v3/docs/completion-standard.md) — Failure, skipped/missing checks, wrong subject and security findings |
| module | Admit one exact module subject · Collector not implemented | [completion-standard.md](file:///var/home/herdr-engineering-engine-v3/docs/completion-standard.md) — All applicable completion gates; trusted producer and owner disposition |
| join | Integrate and admit assembled code · Separate future release proof | [04-release-recovery.md](file:///var/home/herdr-engineering-engine-v3/runbooks/04-release-recovery.md) — Parent integration, compatibility, installed readback and recovery |
| projection | Enable accepted-fact projection · Required future tooling evolution | [completion_standard.py](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/completion_standard.py) — load rejects accepted state today; revise schema/loaders and qualify migration |
| publish | Refresh corpus and verify graph · Existing documentation tools | [UPDATE_PROTOCOL.md](file:///var/home/herdr-engineering-engine-v3/corpus/UPDATE_PROTOCOL.md) — Reconcile; complete core publication; Graphify refresh; final corpus-check |

| Arrow | From → to | Meaning | Condition / evidence |
| --- | --- | --- | --- |
| E01 | scope → candidate | forward | Only after actual coding authority and original prerequisites; no context or diagram grants permission. |
| E02 | candidate → verify | forward | Bind source, supported interfaces, tests, migrations, fixtures and toolchain before execution. |
| E03 | verify → repair | forward | Failures, unknowns, missing/zero/skipped/stale checks and unclosed findings prevent admission. |
| E04 | repair → candidate | return | Repair within scope; retain counterevidence and rerun affected checks. |
| E05 | verify → module | forward | Qualified PASS_CANDIDATE evidence may be considered by a separate future trusted admission boundary; collection writes no task state. |
| E06 | module → join | forward | Module acceptance is input to parent integration, never automatic assembled-code acceptance. |
| E07 | module → projection | forward | An admitted module may update its current facts independently of integrated release status. |
| E08 | join → projection | forward | An admitted release may update assembled-code facts; preserve installed subject/readback separately. |
| E09 | projection → publish | forward | After reviewed loader/schema evolution, derive accepted facts without silently amending requirements. |
| E10 | verify → publish | return | Today scoped retained observations can refresh documentation without module or task promotion. |
| E11 | publish → scope | return | Return complete-generation/graph proof and changed context to the next task; retain admission if docs fail. |

**Related notes:** [Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing) · [RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03) · [RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04) · [Update Protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol)

[Schematic index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) ↔ [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) ↔ [Update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) ↔ [Module context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) ↔ [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) ↔ [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) ↔ [Graphify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md)

## CS03 · Publication, failure and recovery

![Publication, failure and recovery](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics/CS03.svg)

**Purpose:** Read the real documentation publication sequence, including the gap between core completion and graph completion.

**Invariant:** A core PASS can precede a graph failure. Full-corpus synchronization requires agreeing complete core records AND a current verified graph; cross-filesystem writes are not one atomic transaction.

| Pointer | Component / availability | Owning source and exact reading locator |
| --- | --- | --- |
| pending | Lock, baseline, mark pending · Existing sync entry | [corpus_sync.py](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/corpus_sync.py) — main lock; sync; previous hashes and prior-record snapshot |
| stage | Stage and validate projections · Existing prepublication checks | [corpus_sync.py](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/corpus_sync.py) — prepare; Vault.run; verify staged notes, routes and bodies |
| freeze | Freeze expected writes and hashes · Existing reviewed journal | [corpus_publication.py](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/corpus_publication.py) — prepare: allowlisted roots and expected before/after bytes |
| apply | Apply each file atomically · Not whole-corpus atomicity | [corpus_publication.py](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/corpus_publication.py) — apply: reject changed targets and retain journal progress |
| readback | Read destinations and live links · Existing independent readback | [corpus_sync.py](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/corpus_sync.py) — verify; check_live; alignment; final QA identities |
| core | Three agreeing complete markers · Core complete only | [publication.json](file:///var/home/herdr-engineering-engine-v3/corpus/publication.json) — Atlas, code and vault marker identities; output and final-QA hashes |
| graph | Refresh three graph copies · Derived sidecar after core | [graphify_corpus.py](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/graphify_corpus.py) — invoke_build; graph journal, frozen inputs and core generation |
| check | Final corpus-check closure · Read-only full consistency | [UPDATE_PROTOCOL.md](file:///var/home/herdr-engineering-engine-v3/corpus/UPDATE_PROTOCOL.md) — tools/corpus-sync --check; complete core plus graph check |
| recover | Pending or mismatched: repair · Preserve journal and evidence | [UPDATE_PROTOCOL.md](file:///var/home/herdr-engineering-engine-v3/corpus/UPDATE_PROTOCOL.md) — Publication recovery; --resume reviewed core journal; --graph only after current core |

| Arrow | From → to | Meaning | Condition / evidence |
| --- | --- | --- | --- |
| E01 | pending → stage | forward | Acquire cooperating-writer lock; snapshot existing records and hashes before staging. |
| E02 | stage → freeze | forward | Only validated stage content enters the frozen publication journal. |
| E03 | freeze → apply | forward | Expected before/after hashes bind writes; concurrent edits must not be overwritten. |
| E04 | apply → readback | forward | Read every destination and check the actual Obsidian link graph. |
| E05 | readback → core | forward | Only successful core readback permits complete markers in all three corpora. |
| E06 | core → graph | forward | Core completion is the input to graph refresh, not the final full-corpus verdict. |
| E07 | graph → check | forward | Check graph inputs, outputs, copies and exact core-generation binding. |
| E08 | stage → recover | return | Stage/validation errors retain pending status; no accepted engine state is inferred. |
| E09 | apply → recover | return | Partial writes retain journal evidence; inspect conflicts before reviewed resume. |
| E10 | readback → recover | return | Readback/link failure prevents complete core closure; repair the actual failed destination or owner. |
| E11 | graph → recover | return | Core may remain complete while graph is failed/stale; do not report full-corpus success. |
| E12 | check → recover | return | Any output/graph mismatch is unresolved; return the actual error and affected subject. |
| E13 | recover → pending | return | Core failure: reviewed --resume applies a frozen core journal, then fresh synchronization and graph checks. Before freeze there may be no resumable journal; repair and restart. |
| E14 | recover → graph | return | Graph-only failure: --graph requires complete current core and any graph journal bound to that same generation; it does not start a new core publication. |

**Related notes:** [Update Protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) · [Update Workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Maintenance%2FUpdate%20Workflow) · [RB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB02) · [Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)

[Schematic index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) ↔ [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) ↔ [Update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) ↔ [Module context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) ↔ [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) ↔ [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) ↔ [Graphify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md)

## CS04 · Evidence earns a scoped verdict

![Evidence earns a scoped verdict](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics/CS04.svg)

**Purpose:** Trace requirements through a qualified oracle and exact-subject evidence before any module or release claim.

**Invariant:** Current corpus checks prove document/receipt consistency. Future engine admission needs trusted producer evidence and all applicable completion gates; a model review or child PASS is insufficient.

| Pointer | Component / availability | Owning source and exact reading locator |
| --- | --- | --- |
| criteria | Select criteria and threat model · Existing normative requirements | [readiness-plan.md](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) — Original task; applicable clauses and RC01–RC06; completion profile |
| subject | Freeze exact candidate subject · Future engine evidence identity | [public-interfaces.md](file:///var/home/herdr-engineering-engine-v3/docs/public-interfaces.md) — Source, interface, tests, schema, migration, configuration and toolchain |
| runner | Qualified test and security runner · Future trusted boundary | [05-security-hardening.md](file:///var/home/herdr-engineering-engine-v3/runbooks/05-security-hardening.md) — T25/T26; effective security reviewer identity; candidate containment |
| raw | Retain results, failures and logs · Future RC04 qualification receipt | [contract-decisions.md](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md) — RC04 collector envelope and producer status; current v1 evidence-schema.json supports scoped consistency observations only |
| controls | Challenge the decisive oracle · Future engine qualification | [testing-standard.md](file:///var/home/herdr-engineering-engine-v3/docs/testing-standard.md) — Fault and benign controls; mutation/assimilation; ≥50 owned cases; zero warnings/errors |
| admission | Owner adjudicates scoped proof · Future module/release admission | [completion-standard.md](file:///var/home/herdr-engineering-engine-v3/docs/completion-standard.md) — Independent verification, all applicable gates, integration and finding dispositions |
| gap | Gap: repair, rebind, reverify · No promotion from weak evidence | [UPDATE_PROTOCOL.md](file:///var/home/herdr-engineering-engine-v3/corpus/UPDATE_PROTOCOL.md) — Stale, missing, unavailable, skipped, wrong-subject and unqualified evidence |

| Arrow | From → to | Meaning | Condition / evidence |
| --- | --- | --- | --- |
| E01 | criteria → subject | forward | Make the intended behavior and acceptance obligations explicit before choosing tests. |
| E02 | subject → runner | forward | Candidate code cannot silently change the trusted oracle, logs or admission record. |
| E03 | runner → raw | forward | Retain actual producer exit/output and exact evidence; a planned command is not a run. |
| E04 | raw → controls | forward | Check qualification and intended-fault/benign discrimination; record surviving mutants and omissions. |
| E05 | controls → admission | forward | Qualified PASS_CANDIDATE is input to separate admission; only complete current evidence supports the appropriate module or integrated-release disposition. |
| E06 | controls → gap | return | Empty/skipped/failing/stale cases, warnings/errors and unqualified detectors cannot earn PASS. |
| E07 | admission → gap | return | Child success, a reviewer message or current v1 observation never admits the assembled codebase. |
| E08 | gap → subject | return | Repair and bind a new subject; retain historical failure evidence and re-run required checks. |

**Related notes:** [Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing) · [Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts) · [Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03) · [RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)

[Schematic index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) ↔ [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) ↔ [Update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) ↔ [Module context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) ↔ [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) ↔ [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) ↔ [Graphify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md)

## CS05 · Context, change impact and return paths

![Context, change impact and return paths](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics/CS05.svg)

**Purpose:** Keep the full corpus reachable while scouts and coding threads read only the evidence needed for their current module and boundary.

**Invariant:** Text coverage and relationship coverage are different. A graph lead or snapshot hash does not prove a read or current validity. Dependency edges, semantic flows and navigation links remain separate.

| Pointer | Component / availability | Owning source and exact reading locator |
| --- | --- | --- |
| sources | Locate exact owners and sources · Known paths before broad search | [module-context.md](file:///var/home/herdr-engineering-engine-v3/docs/module-context.md) — Cards; Toolshed companions; diary/source captures; bounded upstream research |
| brief | Prepare bounded module brief · Existing scouting skill | [SKILL.md](file:///var/home/herdr-engineering-engine-v3/corpus/agent-skills/hee-module-scout/SKILL.md) — Progressive disclosure; read/unread/omitted/access status; exact task and authority |
| boundary | Inspect callers and dependencies · Separate typed relationships | [anchors.json](file:///var/home/herdr-engineering-engine-v3/corpus/anchors.json) — Build dependencies AND consumers; request/result/error/cancel/usage/evidence flows |
| change | Authorized module or owner edit · One owner and exact subject | [module-context.md](file:///var/home/herdr-engineering-engine-v3/docs/module-context.md) — Original task; primary/support ownership; bounded specialist handoffs |
| impact | Find affected neighbors and gates · Graph is a navigation aid | [graphify-guide.md](file:///var/home/herdr-engineering-engine-v3/docs/graphify-guide.md) — Affected queries; inspect actual source and typed edges; no cluster-only rewrite |
| stale | Mark prior bindings inapplicable · Preserve historical receipts | [UPDATE_PROTOCOL.md](file:///var/home/herdr-engineering-engine-v3/corpus/UPDATE_PROTOCOL.md) — Changed source/config/fixture/toolchain/acceptance hashes; unavailable or unqualified evidence |
| refresh | Re-scout, verify and synchronize · Existing docs; future engine checks | [module-context.md](file:///var/home/herdr-engineering-engine-v3/docs/module-context.md) — Update owning records, regenerate cards, reverify affected contracts and publication |
| handoff | Return proof, omissions and next step · Orchestrator retains parent duty | [CONTEXT_HANDOFF.md](file:///var/home/herdr-engineering-engine-v3/corpus/CONTEXT_HANDOFF.md) — Generation and exact candidate; thread scope, unresolved gaps and parent integration |

| Arrow | From → to | Meaning | Condition / evidence |
| --- | --- | --- | --- |
| E01 | sources → brief | forward | Select sources progressively; conditional companions resolve a real gap, not a mandatory skill ritual. |
| E02 | brief → boundary | forward | Record read status, source identities and required relation coverage separately. |
| E03 | boundary → change | forward | Explain invariants and independent oracle; preserve actual authorization and original prerequisites. |
| E04 | change → impact | forward | Trace touched interfaces/schema/config/migration/tests across callers, dependencies and clusters. |
| E05 | impact → stale | forward | Invalidate only the bindings affected by changed subjects; never manufacture a fresh verdict. |
| E06 | stale → refresh | forward | Retain failed/history records; acquire missing context and rerun applicable checks. |
| E07 | refresh → handoff | forward | Carry complete core/graph proof for docs, and separately scoped engine evidence when available. |
| E08 | handoff → brief | return | Next thread receives bounded question, exact source/relationship gaps and revised subject. |
| E09 | boundary → sources | return | Unknown owner or missing relation triggers a targeted deeper read; budget partiality stays explicit. |

**Related notes:** [Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) · [Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FIndex) · [Update Protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol)

[Schematic index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) ↔ [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) ↔ [Update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) ↔ [Module context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) ↔ [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) ↔ [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) ↔ [Graphify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md)

## CS06 · Module clusters and corpus pointers

![Module clusters and corpus pointers](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics/CS06.svg)

**Purpose:** Matrix cells count declared request and return arrows between six ownership clusters. The directory below maps every one of the 22 modules to code, context, completion and public contracts.

**Invariant:** Forty directional arrows derive from 20 declared semantic flows. They are not build imports, measured runtime calls or new services. Same-cluster flows occupy diagonal cells.

| Pointer | Component / availability | Owning source and exact reading locator |
| --- | --- | --- |
| K1 | Task ownership and recovery · K1 | [CLU-K1.md](file:///var/mnt/STORAGE-10TB/fedora-obsidian-vaults/herdr-engineering-engine-v3.vault/Clusters/CLU-K1.md) — contracts, task, store, budget, recovery |
| K2 | Worker routing and capability · K2 | [CLU-K2.md](file:///var/mnt/STORAGE-10TB/fedora-obsidian-vaults/herdr-engineering-engine-v3.vault/Clusters/CLU-K2.md) — roster, route, worker |
| K3 | Thread context and cohesion · K3 | [CLU-K3.md](file:///var/mnt/STORAGE-10TB/fedora-obsidian-vaults/herdr-engineering-engine-v3.vault/Clusters/CLU-K3.md) — cohort, context, notify, skills, workflows |
| K4 | Verification and numerical evidence · K4 | [CLU-K4.md](file:///var/mnt/STORAGE-10TB/fedora-obsidian-vaults/herdr-engineering-engine-v3.vault/Clusters/CLU-K4.md) — check, numerical, julia |
| K5 | Habitat operation and presentation · K5 | [CLU-K5.md](file:///var/mnt/STORAGE-10TB/fedora-obsidian-vaults/herdr-engineering-engine-v3.vault/Clusters/CLU-K5.md) — service, herdr |
| K6 | Application assembly · K6 | [CLU-K6.md](file:///var/mnt/STORAGE-10TB/fedora-obsidian-vaults/herdr-engineering-engine-v3.vault/Clusters/CLU-K6.md) — app, actions, bash, pi_extension |

| Arrow | From → to | Meaning | Condition / evidence |
| --- | --- | --- | --- |
| E01 | K5 → K1 | forward | F01 herdr → task: Versioned intent and operator constraints enter actions/app; authenticated task-scoped request reaches task |
| E02 | K1 → K5 | return | F01 task → herdr: Status, route reason, blocking criterion and completion references |
| E03 | K1 → K1 | forward | F02 task → store: Expected state/generation plus atomic attempt/event changes |
| E04 | K1 → K1 | return | F02 store → task: Committed identity or typed conflict/storage failure |
| E05 | K1 → K2 | forward | F03 task → route: Task class, immutable constraints and candidate observations |
| E06 | K2 → K1 | return | F03 route → task: Eligible recipe, exclusions, evidence freshness and fallback |
| E07 | K1 → K2 | forward | F04 task → roster: Capability/locality/version/health query whose returned facts are passed to pure route policy |
| E08 | K2 → K1 | return | F04 roster → task: Declared and observed facts with identity and age |
| E09 | K1 → K1 | forward | F05 task → budget: Attempt/verification reservation and later measured usage |
| E10 | K1 → K1 | return | F05 budget → task: Admit/refuse, remaining budget and reconciliation discrepancy |
| E11 | K1 → K2 | forward | F06 task → worker: Attempt/generation, recipe, owned workspace, limits and cancellation |
| E12 | K2 → K1 | return | F06 worker → task: Ack/activity, usage, candidate references, exit/error and settled state |
| E13 | K1 → K4 | forward | F07 task → check: Acceptance revision, immutable candidate and protected fixture references |
| E14 | K4 → K1 | return | F07 check → task: Criterion observations, checker errors and raw evidence references |
| E15 | K1 → K3 | forward | F08 task → cohort: Parent criteria, child DAG, thread briefs and resource/budget bounds |
| E16 | K3 → K1 | return | F08 cohort → task: Child candidates, dissent, integration gaps and join readiness |
| E17 | K3 → K3 | forward | F09 cohort → context: Current brief revision, role, permitted sources and context limit |
| E18 | K3 → K3 | return | F09 context → cohort: Selected context/artifact pointers, separate source and task-relationship coverage gaps, whole-workflow context cost and provenance |
| E19 | K1 → K3 | forward | F10 task → notify: Committed outcome event and relevant recipients |
| E20 | K3 → K1 | return | F10 notify → task: Delivery receipt or pending/retryable notification state |
| E21 | K1 → K1 | forward | F11 task → recovery: Persisted active attempts and observed process/session identities |
| E22 | K1 → K1 | return | F11 recovery → task: Reconciled state, cleanup result or quarantined ambiguity |
| E23 | K4 → K4 | forward | F12 numerical → julia: Immutable dataset, schema/units, analysis recipe and resource bound |
| E24 | K4 → K4 | return | F12 julia → numerical: Report/policy candidate, dataset identity, uncertainty and error |
| E25 | K1 → K4 | forward | F13 task → numerical: Immutable outcome dataset at declared cutoff, including failed/cancelled/abandoned attempts and unknown usage, plus requested comparison |
| E26 | K4 → K1 | return | F13 numerical → task: Validated quantitative report and candidate-policy disposition |
| E27 | K2 → K5 | forward | F14 roster → service: Explicit discovery, cached-state query or specifically approved bounded probe; lifecycle mutation uses F20 |
| E28 | K5 → K2 | return | F14 service → roster: Timestamped useful-health and actual owner/state observation |
| E29 | K6 → K1 | forward | F15 actions → task: Schema-validated, caller-bound task request and idempotency context |
| E30 | K1 → K6 | return | F15 task → actions: Admission/state/evidence or typed denial/conflict |
| E31 | K6 → K6 | forward | F16 bash → actions: Typed argv/JSON and caller task context |
| E32 | K6 → K6 | return | F16 actions → bash: Structured result, decisive exit status and evidence references |
| E33 | K6 → K6 | forward | F17 pi_extension → actions: Registered tool call, cancellation signal and admitted versions |
| E34 | K6 → K6 | return | F17 actions → pi_extension: Typed result/error, usage, effective identity and evidence |
| E35 | K3 → K3 | forward | F18 skills → context: Selected skill/version, action compatibility and bounded references |
| E36 | K3 → K3 | return | F18 context → skills: Task-scoped instructions, source and relationship provenance, explicit omissions and context preparation cost |
| E37 | K3 → K1 | forward | F19 workflows → task: Versioned finite action composition and expanded child identity, mediated by actions/app with current authority |
| E38 | K1 → K3 | return | F19 task → workflows: Step results, outstanding obligations, repair/stop and parent evidence |
| E39 | K6 → K5 | forward | F20 actions → service: Caller-bound explicit grant, registered service/unit/action and durable operation key |
| E40 | K5 → K6 | return | F20 service → actions: Lifecycle owner/job identity, observed result/useful health or unknown effect |

**Related notes:** [Cluster Map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Connections%2FCluster%20Map) · [Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) · [Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)

[Schematic index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) ↔ [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) ↔ [Update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) ↔ [Module context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) ↔ [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) ↔ [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) ↔ [Graphify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md)

## Every module has a route back

| Module / cluster | Code and return anchor | Context | Completion | Public contract |
| --- | --- | --- | --- | --- |
| contracts / K1 | [src/contracts.rs](file:///var/home/herdr-engineering-engine-v3/src/contracts.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-contracts) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-contracts) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/contracts.md) |
| task / K1 | [src/task.rs](file:///var/home/herdr-engineering-engine-v3/src/task.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-task) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-task) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/task.md) |
| store / K1 | [src/store.rs](file:///var/home/herdr-engineering-engine-v3/src/store.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-store) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-store) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/store.md) |
| roster / K2 | [src/roster.rs](file:///var/home/herdr-engineering-engine-v3/src/roster.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-roster) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-roster) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/roster.md) |
| route / K2 | [src/route.rs](file:///var/home/herdr-engineering-engine-v3/src/route.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-route) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-route) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/route.md) |
| budget / K1 | [src/budget.rs](file:///var/home/herdr-engineering-engine-v3/src/budget.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-budget) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-budget) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/budget.md) |
| worker / K2 | [src/worker/mod.rs](file:///var/home/herdr-engineering-engine-v3/src/worker/mod.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-worker) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-worker) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/worker.md) |
| check / K4 | [src/check.rs](file:///var/home/herdr-engineering-engine-v3/src/check.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-check) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-check) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/check.md) |
| recovery / K1 | [src/recovery.rs](file:///var/home/herdr-engineering-engine-v3/src/recovery.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-recovery) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-recovery) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/recovery.md) |
| cohort / K3 | [src/cohort.rs](file:///var/home/herdr-engineering-engine-v3/src/cohort.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-cohort) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-cohort) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/cohort.md) |
| context / K3 | [src/context.rs](file:///var/home/herdr-engineering-engine-v3/src/context.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-context) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-context) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/context.md) |
| notify / K3 | [src/notify.rs](file:///var/home/herdr-engineering-engine-v3/src/notify.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-notify) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-notify) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/notify.md) |
| service / K5 | [src/service.rs](file:///var/home/herdr-engineering-engine-v3/src/service.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-service) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-service) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/service.md) |
| herdr / K5 | [src/herdr.rs](file:///var/home/herdr-engineering-engine-v3/src/herdr.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-herdr) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-herdr) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/herdr.md) |
| numerical / K4 | [src/numerical.rs](file:///var/home/herdr-engineering-engine-v3/src/numerical.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-numerical) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-numerical) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/numerical.md) |
| julia / K4 | [julia/src/HabitatAnalysis.jl](file:///var/home/herdr-engineering-engine-v3/julia/src/HabitatAnalysis.jl) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-julia) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-julia) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/julia.md) |
| app / K6 | [src/main.rs](file:///var/home/herdr-engineering-engine-v3/src/main.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-app) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-app) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/app.md) |
| actions / K6 | [src/actions.rs](file:///var/home/herdr-engineering-engine-v3/src/actions.rs) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-actions) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-actions) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/actions.md) |
| bash / K6 | [integrations/bash/README.stub.md](file:///var/home/herdr-engineering-engine-v3/integrations/bash/README.stub.md) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-bash) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-bash) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/bash.md) |
| pi_extension / K6 | [integrations/pi/README.stub.md](file:///var/home/herdr-engineering-engine-v3/integrations/pi/README.stub.md) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-pi_extension) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-pi_extension) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/pi_extension.md) |
| skills / K3 | [skills/README.stub.md](file:///var/home/herdr-engineering-engine-v3/skills/README.stub.md) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-skills) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-skills) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/skills.md) |
| workflows / K3 | [workflows/README.stub.md](file:///var/home/herdr-engineering-engine-v3/workflows/README.stub.md) | [Scout](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-workflows) | [Outcome](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-workflows) | [Full deployment contract](file:///var/home/herdr-engineering-engine-v3/docs/modules/workflows.md) |

## Update and verification

Edit the existing fact owner; schematic explanations/layout live in corpus_schematics.py. Preserve original task and contract identities. Regenerate through tools/corpus-sync after authorized changes; maintenance edits also require --verify-tooling. The final --check must verify both core and graph. Merely observing a successful code test or a complete core marker is insufficient. Generated SVG/HTML/Markdown/JSON and native notes share this derived data; complete publication hashes establish their current identity. Historical reports keep their original generation.

Failure of documentation or graph publication must not replay engine side effects or erase valid retained admission. Inspect the exact failed journal or graph binding, repair within scope, and repeat the existing readback. No watcher or automatic module-admission hook is installed.

[Schematic index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) ↔ [Quick start](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Quick%20Start) ↔ [Update protocol](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) ↔ [Module context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) ↔ [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) ↔ [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) ↔ [Graphify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md) · [Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md)
