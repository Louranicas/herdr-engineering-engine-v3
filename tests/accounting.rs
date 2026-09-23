// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/tests/accounting.rs
// Scope: deployment contract and navigation only; this comment does not implement or qualify behavior.
// Implementation belongs outside this maintained anchor block. Preserve its stable identity and return links.
// [CODEBASE MASTER](file:///var/home/herdr-engineering-engine-v3/README.md)
// [ATLAS MASTER](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md)
// [VAULT MASTER](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index)
// [ULTRA MAP MASTER](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex)
// [LOCAL ULTRA MAP](file:///var/home/herdr-engineering-engine-v3/corpus/ULTRA_MAP.md)
// [UPDATE AND EVIDENCE PROTOCOL](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol)
// [Progressive module context workflow](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context) · [Codebase context index](file:///var/home/herdr-engineering-engine-v3/docs/module-context.md) · [$hee-module-scout skill](file:///var/home/Louranicas/.codex/skills/hee-module-scout/SKILL.md)
// [Corpus architecture and update schematics](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex) · [Open clickable drawings](file:///var/home/herdr-engineering-engine-v3/docs/corpus-schematics.html)
// [NEW CONTEXT START-CODING POINTER](file:///var/home/herdr-engineering-engine-v3/corpus/CONTEXT_HANDOFF.md)
// Resume sequence: active user instruction and AGENTS → QUICK_START.md → Prime/Fedora and focused atlas → current core/graph check → selected module scout and original task → authorized implementation/verification. This comment is not a coding instruction.
// Completed accepted code is primary for implemented facts; intended requirements remain in the atlas. This anchor is not completion admission.
// [PUBLIC INTERFACES AND AUTHORITY CONVENTION](file:///var/home/herdr-engineering-engine-v3/docs/public-interfaces.md)
// [DEPLOYMENT CORPUS ASSESSMENT AND GAPS](file:///var/home/herdr-engineering-engine-v3/docs/deployment-assessment.md)
// [ADOPTED READINESS PLAN](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md)
// [ADOPTED READINESS AND EVIDENCE ROADMAP](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
// Readiness adoption adds requirements, never score promotion, engine task completion or coding authority.
//
// TEST AND INTERFACE VERIFICATION CONTRACT
// Primary path owner: budget. Participating modules: budget, task, worker, check, context, cohort, route.
// Planned boundary: Accounting cases include concurrency, retries, checking, compaction, parent/child allocations and constrained fallback.
// For each case bind one primary module, stable case identity, requirement/interface/flow, independent oracle, subject/profile, fixtures and raw result. Other participants receive integration evidence, not duplicate primary credits.
// Minimum 50 distinct qualifying cases per module; zero baseline warnings/errors including admitted Rust pedantic Clippy, rustdoc and applicable Julia/package checks. Meaningful coverage gaps remain blocking even above the count.
// Exercise caller input and callee result/error/usage/cancellation paths, compatibility, authority, resource limits, idempotency and readback. Pair assimilation fault detection with a benign mirror; retain nonempty mutation/fault campaign dispositions.
// No tests or evaluation cases are implemented by these comments. Expected failures belong in isolated fixtures with asserted outcomes; they do not waive the zero-diagnostic baseline.
// Retain exact subject/toolchain/fixture hashes and observations; completion admission and full testing qualification remain unavailable. See docs/public-interfaces.md, tests/README.md and corpus/UPDATE_PROTOCOL.md.
// [GRAPHIFY FULL GRAPH](file:///var/home/herdr-engineering-engine-v3/corpus/graphify-out/graph.json)
// [GRAPHIFY UPDATE AND QUERIES](file:///var/home/herdr-engineering-engine-v3/docs/graphify-guide.md)
// Defensive review model: gpt-daybreak-blue-latest; select using Codex /model and verify effective identity. Unassessed; no model-based admission.
// [DAYBREAK SECURITY PROFILE](file:///var/home/herdr-engineering-engine-v3/docs/security-profile.md)
// [SECURITY REPAIR AND REVERIFICATION](file:///var/home/herdr-engineering-engine-v3/runbooks/05-security-hardening.md)
// [CONFIGURATION SCOPE](file:///var/home/herdr-engineering-engine-v3/config/README.md)
// Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.
// [FULLY COMPLETE STANDARD](file:///var/home/herdr-engineering-engine-v3/docs/completion-standard.md)
// [DOCUMENTATION JUSTFILE](file:///var/home/herdr-engineering-engine-v3/justfile)
// [RUNBOOK CATALOGUE](file:///var/home/herdr-engineering-engine-v3/runbooks/README.md)
// [CONTEXT RESTART POINTER](file:///var/home/herdr-engineering-engine-v3/corpus/CONTEXT_HANDOFF.md)
// [QUICK START](file:///var/home/herdr-engineering-engine-v3/QUICK_START.md)
// [ASSIMILATION AND DELIVERY WORKFLOW](file:///var/home/herdr-engineering-engine-v3/workflows/README.md)
// Readiness binding: HEE3-READINESS-001; SHA-256 e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6; clauses F1-C01, F1-C02, F1-C03, F1-C04, F1-C05, F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-01, R90-05, R90-07, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-budget; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-budget (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-budget
// Owns: Reservation policy and usage reconciliation
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/budget.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-budget)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T01, T04, T09, T10, T12, T14, T17, T18, T19, T20, T22, T24, T25, T26, T27, T29
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K1)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB03)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [task TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
// [task TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
// [task TASK-T09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T09)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
// [implementation support task TASK-T11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T11)
// [task TASK-T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
// [task TASK-T24](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T24)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [task TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
// [separate reference example EX-budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-budget)
// [flow FLOW-F05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F05)
// [handbook HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
// [handbook HB-flow-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-flow-map)
// [handbook HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-budget)
// [plan SEC-economy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-economy)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-routing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-routing)
// [plan SEC-threads](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-threads)
// [requirement REQ-R01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R01)
// [requirement REQ-R02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R02)
// [requirement REQ-R03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R03)
// [requirement REQ-R04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R04)
// [requirement REQ-R05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R05)
// [requirement REQ-R06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R06)
// [requirement REQ-R07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R07)
// [requirement REQ-R08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R08)
// [requirement REQ-R09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R09)
// [requirement REQ-R10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R10)
// [requirement REQ-R11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R11)
// [requirement REQ-R12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R12)
// [requirement REQ-R13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R13)
// [requirement REQ-R14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R14)
// [requirement REQ-R15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R15)
// [requirement REQ-R17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R17)
// [requirement REQ-R18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R18)
// [requirement REQ-R19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R19)
// [requirement REQ-R20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R20)
// [requirement REQ-R21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R21)
// [schematic SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
// [schematic SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
// [schematic SC-SC10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC10)
// [schematic SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [source SRC-C13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C13)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-budget)
// [corpus architecture and verification schematic Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
// [corpus architecture and verification schematic CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
// [corpus architecture and verification schematic CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
// [corpus architecture and verification schematic CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
// [corpus architecture and verification schematic CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
// [corpus architecture and verification schematic CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
// [corpus architecture and verification schematic CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
// [resolved design contracts Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
// [resolved module contract RC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01)
// [resolved module contract RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02)
// [resolved module contract RC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03)
// [resolved module contract RC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04)
// [resolved module contract RC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05)
// [resolved module contract RC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06)
// [adopted readiness convention Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
// [readiness criterion cluster F1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF1)
// [readiness criterion cluster F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
// [readiness criterion cluster F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
// [readiness criterion cluster F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
// [readiness criterion cluster F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-01)
// [readiness improvement grouping R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
// [readiness improvement grouping R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-budget)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN09)
// [applied learning LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR04)
// [diary evidence source DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
// [diary evidence source DR07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR07)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// Applicable learning IDs: LRN08, LRN09, LRN12; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Corpus of Me](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Corpus%20of%20Me)
// [The Seven Traits, Tested](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Seven%20Traits%2C%20Tested)
// [Thematic Analysis of the Vaults](obsidian://open?vault=my-diary.vault&file=Reflections%2FThematic%20Analysis%20of%20the%20Vaults)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// Readiness binding: HEE3-READINESS-001; SHA-256 e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-04, R90-05, R90-06, R90-08, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-check; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-check (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-check
// Owns: Protected verifier invocation and criterion evidence
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/check.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T01, T04, T06, T07, T10, T12, T14, T15, T17, T18, T19, T20, T25, T26, T27
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K4)
// [contributing codebase CODE-CB01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB01)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB05)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [task TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
// [implementation support task TASK-T02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T02)
// [implementation support task TASK-T03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T03)
// [task TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
// [implementation support task TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
// [task TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
// [task TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
// [task TASK-T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)
// [implementation support task TASK-T13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T13)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [implementation support task TASK-T21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [separate reference example EX-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-check)
// [flow FLOW-F07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F07)
// [handbook HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [handbook HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-check)
// [plan SEC-hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-hardening)
// [plan SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-security](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-security)
// [requirement REQ-R02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R02)
// [requirement REQ-R03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R03)
// [requirement REQ-R04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R04)
// [requirement REQ-R05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R05)
// [requirement REQ-R06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R06)
// [requirement REQ-R08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R08)
// [requirement REQ-R09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R09)
// [requirement REQ-R10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R10)
// [requirement REQ-R11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R11)
// [requirement REQ-R12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R12)
// [requirement REQ-R13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R13)
// [requirement REQ-R14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R14)
// [requirement REQ-R15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R15)
// [requirement REQ-R17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R17)
// [requirement REQ-R18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R18)
// [requirement REQ-R19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R19)
// [requirement REQ-R20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R20)
// [requirement REQ-R21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R21)
// [schematic SC-SC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC01)
// [schematic SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
// [schematic SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
// [schematic SC-SC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC06)
// [schematic SC-SC08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC08)
// [schematic SC-SC09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC09)
// [schematic SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
// [schematic SC-SC12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC12)
// [schematic SC-SC15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC15)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC22)
// [source SRC-C14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C14)
// [source SRC-S05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S05)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-check)
// [corpus architecture and verification schematic Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
// [corpus architecture and verification schematic CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
// [corpus architecture and verification schematic CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
// [corpus architecture and verification schematic CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
// [corpus architecture and verification schematic CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
// [corpus architecture and verification schematic CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
// [corpus architecture and verification schematic CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
// [resolved design contracts Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
// [resolved module contract RC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01)
// [resolved module contract RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02)
// [resolved module contract RC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03)
// [resolved module contract RC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04)
// [resolved module contract RC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05)
// [resolved module contract RC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06)
// [adopted readiness convention Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
// [readiness criterion cluster F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
// [readiness criterion cluster F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
// [readiness criterion cluster F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
// [readiness criterion cluster F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
// [readiness criterion cluster F6](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04)
// [readiness improvement grouping R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
// [readiness improvement grouping R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06)
// [readiness improvement grouping R90-08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-08)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-check)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
// [applied learning LRN02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN02)
// [applied learning LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
// [applied learning LRN06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12)
// [applied learning LRN14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR07)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// [diary evidence source DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
// [diary evidence source DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
// Applicable learning IDs: LRN01, LRN02, LRN05, LRN06, LRN08, LRN12, LRN14; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Seven Traits, Tested](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Seven%20Traits%2C%20Tested)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [Thematic Analysis of the Vaults](obsidian://open?vault=my-diary.vault&file=Reflections%2FThematic%20Analysis%20of%20the%20Vaults)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// [Why I Stopped Trusting Green](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhy%20I%20Stopped%20Trusting%20Green)
// [Working in Sandboxes on Kinoite](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20in%20Sandboxes%20on%20Kinoite)
// Readiness binding: HEE3-READINESS-001; SHA-256 e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-07, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-cohort; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-cohort (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-cohort
// Owns: Logical threads, assignments, resource claims and join obligations
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/cohort.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-cohort)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T05, T10, T11, T12, T14, T17, T18, T19, T20, T22, T25, T26, T27, T29
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K3)
// [contributing codebase CODE-CB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB04)
// [contributing codebase CODE-CB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB05)
// [contributing codebase CODE-CB11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB11)
// [task TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
// [task TASK-T11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T11)
// [task TASK-T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [task TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
// [separate reference example EX-cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-cohort)
// [flow FLOW-F08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F08)
// [flow FLOW-F09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F09)
// [handbook HB-flow-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-flow-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [handbook HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
// [API API-API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01)
// [API API-API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10)
// [action ACT-thread.get](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-thread.get)
// [action ACT-thread.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-thread.list)
// [IPC IPC-IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01)
// [IPC IPC-IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-cohort)
// [plan SEC-economy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-economy)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-swarm](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-swarm)
// [plan SEC-threads](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-threads)
// [requirement REQ-R02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R02)
// [requirement REQ-R03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R03)
// [requirement REQ-R04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R04)
// [requirement REQ-R05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R05)
// [requirement REQ-R06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R06)
// [requirement REQ-R08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R08)
// [requirement REQ-R09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R09)
// [requirement REQ-R10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R10)
// [requirement REQ-R11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R11)
// [requirement REQ-R12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R12)
// [requirement REQ-R13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R13)
// [requirement REQ-R14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R14)
// [requirement REQ-R15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R15)
// [requirement REQ-R17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R17)
// [requirement REQ-R18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R18)
// [requirement REQ-R19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R19)
// [requirement REQ-R20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R20)
// [requirement REQ-R21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R21)
// [schematic SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
// [schematic SC-SC08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC08)
// [schematic SC-SC09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC09)
// [schematic SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC21)
// [source SRC-S09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S09)
// [source SRC-S10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S10)
// [source SRC-S11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S11)
// [source SRC-S15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S15)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-cohort)
// [corpus architecture and verification schematic Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
// [corpus architecture and verification schematic CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
// [corpus architecture and verification schematic CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
// [corpus architecture and verification schematic CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
// [corpus architecture and verification schematic CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
// [corpus architecture and verification schematic CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
// [corpus architecture and verification schematic CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
// [resolved design contracts Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
// [resolved module contract RC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01)
// [resolved module contract RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02)
// [resolved module contract RC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03)
// [resolved module contract RC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04)
// [resolved module contract RC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05)
// [adopted readiness convention Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
// [readiness criterion cluster F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
// [readiness criterion cluster F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
// [readiness criterion cluster F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
// [readiness criterion cluster F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-cohort)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11)
// [applied learning LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12)
// [applied learning LRN14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR07)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// [diary evidence source DR12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR12)
// Applicable learning IDs: LRN07, LRN08, LRN11, LRN12, LRN14; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Seven Traits, Tested](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Seven%20Traits%2C%20Tested)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [Thematic Analysis of the Vaults](obsidian://open?vault=my-diary.vault&file=Reflections%2FThematic%20Analysis%20of%20the%20Vaults)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// [Working Style in This Habitat](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20Style%20in%20This%20Habitat)
// Readiness binding: HEE3-READINESS-001; SHA-256 e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-06, R90-07, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-context; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-context (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-context
// Owns: Bounded briefs, provenance and artifact-reference selection
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/context.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-context)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T10, T11, T14, T17, T18, T19, T20, T22, T25, T26, T27, T29
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K3)
// [contributing codebase CODE-CB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB04)
// [contributing codebase CODE-CB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB05)
// [contributing codebase CODE-CB11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB11)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
// [task TASK-T11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T11)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [task TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
// [separate reference example EX-context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-context)
// [flow FLOW-F09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F09)
// [flow FLOW-F18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F18)
// [handbook HB-flow-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-flow-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-context)
// [plan SEC-economy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-economy)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-swarm](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-swarm)
// [plan SEC-threads](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-threads)
// [plan SEC-toolchain](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-toolchain)
// [requirement REQ-R02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R02)
// [requirement REQ-R03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R03)
// [requirement REQ-R04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R04)
// [requirement REQ-R05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R05)
// [requirement REQ-R06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R06)
// [requirement REQ-R08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R08)
// [requirement REQ-R09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R09)
// [requirement REQ-R10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R10)
// [requirement REQ-R11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R11)
// [requirement REQ-R12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R12)
// [requirement REQ-R13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R13)
// [requirement REQ-R14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R14)
// [requirement REQ-R15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R15)
// [requirement REQ-R17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R17)
// [requirement REQ-R18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R18)
// [requirement REQ-R19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R19)
// [requirement REQ-R20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R20)
// [requirement REQ-R21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R21)
// [schematic SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
// [schematic SC-SC08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC08)
// [schematic SC-SC09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC09)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC21)
// [source SRC-C18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C18)
// [source SRC-S18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S18)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-context)
// [corpus architecture and verification schematic Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
// [corpus architecture and verification schematic CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
// [corpus architecture and verification schematic CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
// [corpus architecture and verification schematic CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
// [corpus architecture and verification schematic CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
// [corpus architecture and verification schematic CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
// [corpus architecture and verification schematic CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
// [resolved design contracts Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
// [resolved module contract RC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01)
// [resolved module contract RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02)
// [resolved module contract RC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03)
// [resolved module contract RC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04)
// [resolved module contract RC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05)
// [adopted readiness convention Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
// [readiness criterion cluster F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
// [readiness criterion cluster F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
// [readiness criterion cluster F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
// [readiness criterion cluster F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06)
// [readiness improvement grouping R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-context)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN09)
// [applied learning LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11)
// [applied learning LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12)
// [applied learning LRN14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR04)
// [diary evidence source DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR07)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// [diary evidence source DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
// [diary evidence source DR12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR12)
// Applicable learning IDs: LRN03, LRN08, LRN09, LRN11, LRN12, LRN14; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Corpus of Me](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Corpus%20of%20Me)
// [The Seven Traits, Tested](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Seven%20Traits%2C%20Tested)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [Thematic Analysis of the Vaults](obsidian://open?vault=my-diary.vault&file=Reflections%2FThematic%20Analysis%20of%20the%20Vaults)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// [Why I Stopped Trusting Green](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhy%20I%20Stopped%20Trusting%20Green)
// [Working Style in This Habitat](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20Style%20in%20This%20Habitat)
// Readiness binding: HEE3-READINESS-001; SHA-256 e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6; clauses F1-C01, F1-C02, F1-C03, F1-C04, F1-C05, F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-01, R90-07, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-route; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-route (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-route
// Owns: Pure eligibility/ranking and route explanations
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/route.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-route)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T05, T09, T10, T12, T14, T17, T18, T19, T20, T24, T25, T26, T27
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K2)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB09)
// [contributing codebase CODE-CB10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB10)
// [task TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
// [task TASK-T09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T09)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
// [task TASK-T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T24](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T24)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [separate reference example EX-route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-route)
// [flow FLOW-F03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F03)
// [handbook HB-flow-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-flow-map)
// [handbook HB-module-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-module-map)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-route)
// [plan SEC-economy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-economy)
// [plan SEC-excellence](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-excellence)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-routing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-routing)
// [requirement REQ-R01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R01)
// [requirement REQ-R02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R02)
// [requirement REQ-R03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R03)
// [requirement REQ-R04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R04)
// [requirement REQ-R05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R05)
// [requirement REQ-R06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R06)
// [requirement REQ-R07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R07)
// [requirement REQ-R08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R08)
// [requirement REQ-R09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R09)
// [requirement REQ-R10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R10)
// [requirement REQ-R11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R11)
// [requirement REQ-R12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R12)
// [requirement REQ-R13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R13)
// [requirement REQ-R14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R14)
// [requirement REQ-R15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R15)
// [requirement REQ-R17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R17)
// [requirement REQ-R18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R18)
// [requirement REQ-R19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R19)
// [requirement REQ-R20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R20)
// [requirement REQ-R21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R21)
// [schematic SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
// [schematic SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
// [schematic SC-SC10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC10)
// [schematic SC-SC14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC14)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC20)
// [source SRC-C04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C04)
// [source SRC-C05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C05)
// [source SRC-C08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C08)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-route)
// [corpus architecture and verification schematic Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
// [corpus architecture and verification schematic CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
// [corpus architecture and verification schematic CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
// [corpus architecture and verification schematic CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
// [corpus architecture and verification schematic CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
// [corpus architecture and verification schematic CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
// [corpus architecture and verification schematic CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
// [resolved design contracts Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
// [resolved module contract RC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01)
// [resolved module contract RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02)
// [resolved module contract RC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03)
// [resolved module contract RC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04)
// [resolved module contract RC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05)
// [adopted readiness convention Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
// [readiness criterion cluster F1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF1)
// [readiness criterion cluster F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
// [readiness criterion cluster F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
// [readiness criterion cluster F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
// [readiness criterion cluster F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-01)
// [readiness improvement grouping R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-route)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN09)
// [applied learning LRN10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10)
// [applied learning LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR04)
// [diary evidence source DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR07)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// Applicable learning IDs: LRN05, LRN08, LRN09, LRN10, LRN12; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Corpus of Me](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Corpus%20of%20Me)
// [The Seven Traits, Tested](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Seven%20Traits%2C%20Tested)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [Thematic Analysis of the Vaults](obsidian://open?vault=my-diary.vault&file=Reflections%2FThematic%20Analysis%20of%20the%20Vaults)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What Prototyping Is For](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20Prototyping%20Is%20For)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// Readiness binding: HEE3-READINESS-001; SHA-256 e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6; clauses F1-C01, F1-C02, F1-C03, F1-C04, F1-C05, F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-01, R90-03, R90-05, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-task; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-task (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-task
// Owns: Task/attempt and bounded verification-loop state
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/task.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T01, T04, T06, T07, T10, T12, T14, T17, T18, T19, T20, T22, T25, T26, T27, T29
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K1)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB03)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [task TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
// [task TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
// [implementation support task TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
// [task TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
// [task TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
// [task TASK-T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [task TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
// [separate reference example EX-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-task)
// [flow FLOW-F01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F01)
// [flow FLOW-F02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F02)
// [flow FLOW-F03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F03)
// [flow FLOW-F04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F04)
// [flow FLOW-F05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F05)
// [flow FLOW-F06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F06)
// [flow FLOW-F07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F07)
// [flow FLOW-F08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F08)
// [flow FLOW-F10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F10)
// [flow FLOW-F11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F11)
// [flow FLOW-F13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F13)
// [flow FLOW-F15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F15)
// [flow FLOW-F19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F19)
// [handbook HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [handbook HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
// [API API-API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01)
// [API API-API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10)
// [action ACT-task.cancel](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.cancel)
// [action ACT-task.get](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.get)
// [action ACT-task.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.list)
// [action ACT-task.preview](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.preview)
// [action ACT-task.resolve](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.resolve)
// [action ACT-task.submit](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.submit)
// [IPC IPC-IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01)
// [IPC IPC-IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-task)
// [plan SEC-architecture](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-architecture)
// [plan SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-threads](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-threads)
// [requirement REQ-R02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R02)
// [requirement REQ-R03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R03)
// [requirement REQ-R04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R04)
// [requirement REQ-R05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R05)
// [requirement REQ-R06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R06)
// [requirement REQ-R08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R08)
// [requirement REQ-R09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R09)
// [requirement REQ-R10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R10)
// [requirement REQ-R11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R11)
// [requirement REQ-R12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R12)
// [requirement REQ-R13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R13)
// [requirement REQ-R14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R14)
// [requirement REQ-R15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R15)
// [requirement REQ-R17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R17)
// [requirement REQ-R18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R18)
// [requirement REQ-R19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R19)
// [requirement REQ-R20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R20)
// [requirement REQ-R21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R21)
// [schematic SC-SC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC01)
// [schematic SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
// [schematic SC-SC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC04)
// [schematic SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
// [schematic SC-SC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC06)
// [schematic SC-SC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC07)
// [schematic SC-SC08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC08)
// [schematic SC-SC10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC10)
// [schematic SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
// [schematic SC-SC12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC12)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC20)
// [schematic SC-SC21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC21)
// [schematic SC-SC22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC22)
// [schematic SC-SC23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC23)
// [schematic SC-SC24](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC24)
// [source SRC-A03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A03)
// [source SRC-C02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C02)
// [source SRC-S15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S15)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-task)
// [corpus architecture and verification schematic Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
// [corpus architecture and verification schematic CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
// [corpus architecture and verification schematic CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
// [corpus architecture and verification schematic CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
// [corpus architecture and verification schematic CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
// [corpus architecture and verification schematic CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
// [corpus architecture and verification schematic CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
// [resolved design contracts Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
// [resolved module contract RC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01)
// [resolved module contract RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02)
// [resolved module contract RC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03)
// [resolved module contract RC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04)
// [resolved module contract RC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05)
// [resolved module contract RC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06)
// [adopted readiness convention Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
// [readiness criterion cluster F1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF1)
// [readiness criterion cluster F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
// [readiness criterion cluster F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
// [readiness criterion cluster F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
// [readiness criterion cluster F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-01)
// [readiness improvement grouping R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
// [readiness improvement grouping R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-task)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
// [applied learning LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// [diary evidence source DR12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR12)
// Applicable learning IDs: LRN01, LRN04, LRN08, LRN11; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What Prototyping Is For](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20Prototyping%20Is%20For)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// [Working Style in This Habitat](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20Style%20in%20This%20Habitat)
// Readiness binding: HEE3-READINESS-001; SHA-256 e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-02, R90-03, R90-05, R90-06, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-worker; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-worker (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-worker
// Owns: Pi/native/inference process adapters behind the worker contract
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/worker.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-worker)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T01, T02, T03, T05, T06, T07, T08, T10, T14, T15, T17, T18, T19, T20, T22, T25, T26, T27, T28, T29
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K2)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB09)
// [contributing codebase CODE-CB10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB10)
// [task TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
// [task TASK-T02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T02)
// [task TASK-T03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T03)
// [task TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
// [task TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
// [task TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
// [task TASK-T08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T08)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [implementation support task TASK-T21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21)
// [task TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [task TASK-T28](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T28)
// [task TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
// [separate reference example EX-worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-worker)
// [flow FLOW-F06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F06)
// [handbook HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
// [handbook HB-socket-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-socket-map)
// [handbook HB-worker-command-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-worker-command-map)
// [API API-API03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API03)
// [API API-API04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API04)
// [API API-API08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API08)
// [IPC IPC-IPC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC03)
// [IPC IPC-IPC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC07)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-worker)
// [plan SEC-api-sockets](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-api-sockets)
// [plan SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-runtime](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-runtime)
// [plan SEC-security](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-security)
// [requirement REQ-R01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R01)
// [requirement REQ-R02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R02)
// [requirement REQ-R03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R03)
// [requirement REQ-R04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R04)
// [requirement REQ-R05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R05)
// [requirement REQ-R06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R06)
// [requirement REQ-R08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R08)
// [requirement REQ-R09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R09)
// [requirement REQ-R10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R10)
// [requirement REQ-R11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R11)
// [requirement REQ-R12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R12)
// [requirement REQ-R13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R13)
// [requirement REQ-R14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R14)
// [requirement REQ-R15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R15)
// [requirement REQ-R17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R17)
// [requirement REQ-R18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R18)
// [requirement REQ-R19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R19)
// [requirement REQ-R20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R20)
// [requirement REQ-R21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Requirements%2FREQ-R21)
// [schematic SC-SC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC01)
// [schematic SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
// [schematic SC-SC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC03)
// [schematic SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
// [schematic SC-SC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC07)
// [schematic SC-SC08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC08)
// [schematic SC-SC09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC09)
// [schematic SC-SC10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC10)
// [schematic SC-SC13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC13)
// [schematic SC-SC15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC15)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC20)
// [source SRC-A11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A11)
// [source SRC-A12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A12)
// [source SRC-H13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-H13)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-worker)
// [corpus architecture and verification schematic Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
// [corpus architecture and verification schematic CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
// [corpus architecture and verification schematic CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
// [corpus architecture and verification schematic CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
// [corpus architecture and verification schematic CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
// [corpus architecture and verification schematic CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
// [corpus architecture and verification schematic CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
// [resolved design contracts Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
// [resolved module contract RC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC01)
// [resolved module contract RC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC02)
// [resolved module contract RC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC03)
// [resolved module contract RC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC04)
// [resolved module contract RC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC05)
// [resolved module contract RC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06)
// [adopted readiness convention Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
// [readiness criterion cluster F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
// [readiness criterion cluster F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
// [readiness criterion cluster F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
// [readiness criterion cluster F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02)
// [readiness improvement grouping R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
// [readiness improvement grouping R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
// [readiness improvement grouping R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-worker)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03)
// [applied learning LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
// [applied learning LRN06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06)
// [applied learning LRN07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10)
// [applied learning LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11)
// [applied learning LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR04)
// [diary evidence source DR05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR05)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// [diary evidence source DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
// [diary evidence source DR12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR12)
// [diary evidence source DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
// Applicable learning IDs: LRN03, LRN05, LRN06, LRN07, LRN08, LRN10, LRN11, LRN13; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Corpus of Me](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Corpus%20of%20Me)
// [The Seven Traits, Tested](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Seven%20Traits%2C%20Tested)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What Prototyping Is For](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20Prototyping%20Is%20For)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// [Why I Stopped Trusting Green](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhy%20I%20Stopped%20Trusting%20Green)
// [Working Style in This Habitat](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20Style%20in%20This%20Habitat)
// [Working in Sandboxes on Kinoite](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20in%20Sandboxes%20on%20Kinoite)
// HEE3-ANCHORS-END

//! T10 · whole-task cost accounting.
//!
//! Every case is owned by `budget` and carries its `T10-AC-NN` identity. The cases exist to
//! pin the acceptance text — *"Concurrent reservations cannot exceed configured bounds;
//! retries/checking/compaction are included; unknown usage is visible; hard-ceiling tasks
//! reject incompatible accounting"* — and the contract's required proof families:
//! conservation, concurrency, retries and delayed usage, compaction/checker costs, abandoned
//! work and uncertain cancellation liability.
//!
//! Tests return [`Outcome`] and use `?`; there is no `unwrap`, `expect` or `panic!` here.

use std::error::Error;

use habitat_engine::budget::{
    Amount, Applied, Balance, Candidate, Ceiling, Constraint, Ledger, MAX_CANDIDATES,
    MAX_REPORTS_PER_RESERVATION, MAX_RESERVATIONS, MAX_TOOLS, Privacy, Provenance, Refusal,
    SCHEMA_VERSION, Settlement, Unit, Usage,
};
use habitat_engine::contracts::UuidV4;

type Outcome = Result<(), Box<dyn Error>>;

/// Distinct well-formed `UUIDv4` identities, generated from an index so a case can ask for as
/// many as it needs without a fixture file.
fn id(index: usize) -> String {
    format!("{index:08x}-0000-4000-8000-000000000000")
}

fn tokens(value: u64) -> Amount {
    Amount::new(Unit::Tokens, value)
}

/// A ledger of `limit` tokens under `ceiling`.
fn ledger(limit: u64, ceiling: Ceiling) -> Ledger {
    Ledger::new(tokens(limit), ceiling)
}

/// The four columns, as a tuple, so a case can assert the whole shape in one comparison
/// rather than reading one field through and leaving the rest unpinned.
fn columns(balance: Balance) -> Result<(u64, u64, u64, u64), Refusal> {
    Ok((
        balance.available()?.value(),
        balance.reserved().value(),
        balance.spent().value(),
        balance.unknown().value(),
    ))
}

fn settled(index: usize) -> Result<Settlement<'static>, Box<dyn Error>> {
    // The identity must outlive the settlement, so it is leaked into a static for the
    // duration of the test process; this is a test helper, never library code.
    let attempt: &'static str = Box::leak(id(index).into_boxed_str());
    Ok(Settlement::Settled {
        attempt: UuidV4::parse(attempt)?,
    })
}

fn abandoned(index: usize) -> Result<Settlement<'static>, Box<dyn Error>> {
    let attempt: &'static str = Box::leak(id(index).into_boxed_str());
    Ok(Settlement::Abandoned {
        attempt: UuidV4::parse(attempt)?,
    })
}

fn cancelled(index: usize) -> Result<Settlement<'static>, Box<dyn Error>> {
    let attempt: &'static str = Box::leak(id(index).into_boxed_str());
    Ok(Settlement::CancelledUncertain {
        attempt: UuidV4::parse(attempt)?,
    })
}

// ---------------------------------------------------------------- units and amounts

/// T10-AC-01 · every declared unit round-trips through its wire name, and the enumeration is
/// the whole world rather than a list someone maintains beside it.
#[test]
fn every_unit_round_trips_through_its_wire_name() -> Outcome {
    assert_eq!(Unit::ALL.len(), 3, "unit count");
    for unit in Unit::ALL {
        assert_eq!(Unit::parse(unit.name())?, unit, "{}", unit.name());
        assert_eq!(unit.to_string(), unit.name());
    }
    Ok(())
}

/// T10-AC-02 · an unknown unit name is refused by name, not defaulted.
#[test]
fn unknown_unit_name_is_refused() {
    assert_eq!(Unit::parse("dollars"), Err(Refusal::UnknownUnit));
    assert_eq!(Unit::parse(""), Err(Refusal::UnknownUnit));
    assert_eq!(Unit::parse("Tokens"), Err(Refusal::UnknownUnit));
}

/// T10-AC-03 · `Amount::parse` inherits `contracts`' error precedence exactly: empty,
/// invalid character, leading zero, overflow. A signed or fractional literal fails as an
/// invalid character, which is the honest diagnosis for a module with no fractional type.
#[test]
fn amount_parsing_preserves_the_scalar_error_precedence() -> Outcome {
    use habitat_engine::contracts::ScalarError;
    assert_eq!(
        Amount::parse(Unit::Tokens, ""),
        Err(Refusal::Scalar(ScalarError::Empty))
    );
    assert_eq!(
        Amount::parse(Unit::Tokens, "-1"),
        Err(Refusal::Scalar(ScalarError::InvalidCharacter))
    );
    assert_eq!(
        Amount::parse(Unit::Tokens, "1.5"),
        Err(Refusal::Scalar(ScalarError::InvalidCharacter))
    );
    assert_eq!(
        Amount::parse(Unit::Tokens, "NaN"),
        Err(Refusal::Scalar(ScalarError::InvalidCharacter))
    );
    assert_eq!(
        Amount::parse(Unit::Tokens, "01"),
        Err(Refusal::Scalar(ScalarError::LeadingZero))
    );
    assert_eq!(
        Amount::parse(Unit::Tokens, "18446744073709551616"),
        Err(Refusal::Scalar(ScalarError::Overflow))
    );
    assert_eq!(Amount::parse(Unit::Tokens, "0")?.value(), 0);
    assert_eq!(
        Amount::parse(Unit::Tokens, "18446744073709551615")?.value(),
        u64::MAX
    );
    Ok(())
}

/// T10-AC-04 · adding two units is refused rather than converted; the engine has no admitted
/// exchange rate and inventing one would hide cost.
#[test]
fn mixing_units_is_refused_in_both_directions() {
    let a = Amount::new(Unit::Tokens, 10);
    let b = Amount::new(Unit::Microcents, 10);
    assert_eq!(a.checked_add(b), Err(Refusal::IncompatibleUnit));
    assert_eq!(b.checked_add(a), Err(Refusal::IncompatibleUnit));
    assert_eq!(a.checked_sub(b), Err(Refusal::IncompatibleUnit));
    assert_eq!(b.checked_sub(a), Err(Refusal::IncompatibleUnit));
}

/// T10-AC-05 · overflow and underflow are distinct refusals, because they mean opposite
/// things about the caller's accounting.
#[test]
fn overflow_and_underflow_are_distinct_refusals() -> Outcome {
    let max = tokens(u64::MAX);
    assert_eq!(max.checked_add(tokens(1)), Err(Refusal::Overflow));
    assert_eq!(tokens(0).checked_sub(tokens(1)), Err(Refusal::Underflow));
    assert_eq!(max.checked_add(tokens(0))?, max);
    assert_eq!(max.checked_sub(max)?, tokens(0));
    Ok(())
}

/// T10-AC-06 · an amount displays its magnitude and its unit, so a log line cannot be read
/// as the wrong currency.
#[test]
fn amount_display_names_its_unit() {
    assert_eq!(tokens(42).to_string(), "42 tokens");
    assert_eq!(Amount::new(Unit::Microcents, 7).to_string(), "7 microcents");
    assert_eq!(
        Amount::new(Unit::Milliseconds, 0).to_string(),
        "0 milliseconds"
    );
}

/// T10-AC-07 · zero is the additive identity for each unit and reports itself as zero.
#[test]
fn zero_is_the_additive_identity_per_unit() -> Outcome {
    for unit in Unit::ALL {
        let zero = Amount::zero(unit);
        assert!(zero.is_zero(), "{unit}");
        assert_eq!(zero.unit(), unit);
        let ten = Amount::new(unit, 10);
        assert_eq!(ten.checked_add(zero)?, ten);
        assert_eq!(ten.checked_sub(zero)?, ten);
        assert!(!ten.is_zero());
    }
    Ok(())
}

/// T10-AC-08 · every provenance round-trips its wire name, and exactly one is unmeasured.
#[test]
fn provenance_names_round_trip_and_one_is_unmeasured() {
    assert_eq!(Provenance::ALL.len(), 5);
    let unmeasured: Vec<&str> = Provenance::ALL
        .into_iter()
        .filter(|p| !p.is_measured())
        .map(Provenance::name)
        .collect();
    assert_eq!(unmeasured, vec!["unmeasured"], "only one unmeasured kind");
    for provenance in Provenance::ALL {
        assert_eq!(provenance.to_string(), provenance.name());
    }
}

// ---------------------------------------------------------------- conservation

/// T10-AC-09 · a fresh ledger conserves and puts the whole limit in `available`.
#[test]
fn a_fresh_ledger_conserves_with_everything_available() -> Outcome {
    let book = ledger(1_000, Ceiling::Soft);
    assert!(book.conserves()?);
    assert_eq!(columns(book.balance()?)?, (1_000, 0, 0, 0));
    assert_eq!(book.revision(), 0);
    assert_eq!(book.reservations(), 0);
    assert_eq!(book.unit(), Unit::Tokens);
    assert_eq!(book.ceiling(), Ceiling::Soft);
    Ok(())
}

/// T10-AC-10 · conservation holds after every transition in a full lifecycle, checked at each
/// step rather than only at the end, so a transition that breaks and restores it is caught.
#[test]
fn conservation_holds_at_every_step_of_a_lifecycle() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(400), None, None)?;
    assert!(book.conserves()?, "after reserve");
    book.report(
        &id(1),
        "r1",
        Usage::new(tokens(150), Provenance::WorkerSettled),
    )?;
    assert!(book.conserves()?, "after measured report");
    book.report(&id(1), "r2", Usage::new(tokens(50), Provenance::Unmeasured))?;
    assert!(book.conserves()?, "after unmeasured report");
    book.release(&id(1), settled(900)?)?;
    assert!(book.conserves()?, "after release");
    assert_eq!(columns(book.balance()?)?, (800, 0, 150, 50));
    Ok(())
}

/// T10-AC-11 · the four columns are asserted whole across three distinct reservations, so no
/// single column can be read through while the others drift.
#[test]
fn the_four_columns_are_pinned_together_across_three_reservations() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(100), None, None)?;
    book.reserve(&id(2), tokens(200), None, None)?;
    book.reserve(&id(3), tokens(300), None, None)?;
    assert_eq!(columns(book.balance()?)?, (400, 600, 0, 0));
    book.report(
        &id(2),
        "r1",
        Usage::new(tokens(75), Provenance::CheckerMeasured),
    )?;
    assert_eq!(columns(book.balance()?)?, (400, 525, 75, 0));
    book.report(&id(3), "r1", Usage::new(tokens(30), Provenance::Unmeasured))?;
    assert_eq!(columns(book.balance()?)?, (400, 495, 75, 30));
    book.release(&id(1), settled(901)?)?;
    assert_eq!(columns(book.balance()?)?, (500, 395, 75, 30));
    assert!(book.conserves()?);
    Ok(())
}

/// T10-AC-12 · the revision advances on every counter-changing transition and on no read.
#[test]
fn revision_advances_on_writes_and_not_on_reads() -> Outcome {
    let mut book = ledger(500, Ceiling::Soft);
    assert_eq!(book.revision(), 0);
    book.reserve(&id(1), tokens(100), None, None)?;
    assert_eq!(book.revision(), 1);
    book.report(
        &id(1),
        "r1",
        Usage::new(tokens(10), Provenance::WorkerSettled),
    )?;
    assert_eq!(book.revision(), 2);
    let _ = book.balance()?;
    let _ = book.available()?;
    let _ = book.reservation(&id(1))?;
    let _ = book.reservation_list()?;
    let _ = book.conserves()?;
    assert_eq!(book.revision(), 2, "no read advances the revision");
    book.release(&id(1), settled(902)?)?;
    assert_eq!(book.revision(), 3);
    Ok(())
}

/// T10-AC-13 · a duplicate report does not advance the revision, because it changes nothing.
#[test]
fn an_idempotent_retry_does_not_advance_the_revision() -> Outcome {
    let mut book = ledger(500, Ceiling::Soft);
    book.reserve(&id(1), tokens(100), None, None)?;
    book.report(
        &id(1),
        "r1",
        Usage::new(tokens(10), Provenance::WorkerSettled),
    )?;
    let before = book.revision();
    let again = book.report(
        &id(1),
        "r1",
        Usage::new(tokens(10), Provenance::WorkerSettled),
    )?;
    assert!(!again.fresh);
    assert_eq!(
        book.revision(),
        before,
        "a no-op must not look like a write"
    );
    Ok(())
}

/// T10-AC-14 · the persisted schema version is pinned, so a store that writes another shape
/// is a visible change rather than a silent one.
#[test]
fn schema_version_is_pinned() {
    assert_eq!(SCHEMA_VERSION, 1);
}

// ---------------------------------------------------------------- reserve and bounds

/// T10-AC-15 · a reservation takes exactly its amount out of `available` and reports the
/// remainder, and the reservation's own books open at the held amount.
#[test]
fn a_reservation_moves_exactly_its_amount_into_reserved() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    let remaining = book.reserve(&id(1), tokens(250), None, None)?;
    assert_eq!(remaining, tokens(750));
    assert_eq!(columns(book.balance()?)?, (750, 250, 0, 0));
    let entry = book.reservation(&id(1))?;
    assert_eq!(entry.held, tokens(250));
    assert_eq!(entry.spent, tokens(0));
    assert_eq!(entry.unknown, tokens(0));
    assert!(entry.open);
    assert!(entry.parent.is_none());
    Ok(())
}

/// T10-AC-16 · a reservation for exactly the remaining budget is admitted; one token more is
/// refused. The boundary is asserted from both sides.
#[test]
fn the_budget_boundary_admits_the_exact_remainder_and_refuses_one_more() -> Outcome {
    let mut book = ledger(100, Ceiling::Soft);
    book.reserve(&id(1), tokens(60), None, None)?;
    assert_eq!(
        book.reserve(&id(2), tokens(41), None, None),
        Err(Refusal::InsufficientBudget)
    );
    assert_eq!(book.reserve(&id(2), tokens(40), None, None)?, tokens(0));
    assert_eq!(columns(book.balance()?)?, (0, 100, 0, 0));
    assert_eq!(
        book.reserve(&id(3), tokens(1), None, None),
        Err(Refusal::InsufficientBudget)
    );
    Ok(())
}

/// T10-AC-17 · insufficient budget is an explicit refusal, never a truncated reservation.
#[test]
fn insufficient_budget_refuses_rather_than_truncating() -> Outcome {
    let mut book = ledger(100, Ceiling::Soft);
    assert_eq!(
        book.reserve(&id(1), tokens(101), None, None),
        Err(Refusal::InsufficientBudget)
    );
    assert_eq!(
        book.reservations(),
        0,
        "a refused reservation leaves nothing"
    );
    assert_eq!(columns(book.balance()?)?, (100, 0, 0, 0));
    assert_eq!(book.revision(), 0, "a refusal is not a write");
    Ok(())
}

/// T10-AC-18 · a reservation in another unit is refused before anything is allocated.
#[test]
fn a_reservation_in_another_unit_is_refused() {
    let mut book = ledger(100, Ceiling::Soft);
    assert_eq!(
        book.reserve(&id(1), Amount::new(Unit::Microcents, 10), None, None),
        Err(Refusal::IncompatibleUnit)
    );
    assert_eq!(book.reservations(), 0);
}

/// T10-AC-19 · a malformed identity is refused with the scalar error that explains it.
#[test]
fn a_malformed_reservation_identity_is_refused() {
    use habitat_engine::contracts::ScalarError;
    let mut book = ledger(100, Ceiling::Soft);
    assert_eq!(
        book.reserve("not-a-uuid", tokens(1), None, None),
        Err(Refusal::MalformedIdentity(ScalarError::InvalidUuid))
    );
    assert_eq!(
        book.reserve("", tokens(1), None, None),
        Err(Refusal::MalformedIdentity(ScalarError::InvalidUuid))
    );
    assert_eq!(book.reservations(), 0);
}

/// T10-AC-20 · a second reservation cannot claim an identity already in use, open or closed.
#[test]
fn a_duplicate_reservation_identity_is_refused_open_or_closed() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(10), None, None)?;
    assert_eq!(
        book.reserve(&id(1), tokens(10), None, None),
        Err(Refusal::DuplicateReservation)
    );
    book.release(&id(1), settled(903)?)?;
    assert_eq!(
        book.reserve(&id(1), tokens(10), None, None),
        Err(Refusal::DuplicateReservation),
        "a closed identity is not reusable"
    );
    Ok(())
}

/// T10-AC-21 · `expected_revision` makes a read-then-reserve atomic: a stale expectation is
/// refused, and the current one is admitted.
#[test]
fn a_stale_expected_revision_is_refused() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    let read_at = book.revision();
    book.reserve(&id(1), tokens(100), None, None)?;
    assert_eq!(
        book.reserve(&id(2), tokens(100), None, Some(read_at)),
        Err(Refusal::StaleRevision),
        "the caller's view of available is out of date"
    );
    let current = book.revision();
    book.reserve(&id(2), tokens(100), None, Some(current))?;
    assert_eq!(columns(book.balance()?)?, (800, 200, 0, 0));
    Ok(())
}

/// T10-AC-22 · the reservation count bound refuses at its limit, and the refusal happens
/// before the entry is allocated.
#[test]
fn the_reservation_count_bound_refuses_before_allocating() -> Outcome {
    let mut book = ledger(u64::MAX, Ceiling::Soft);
    for index in 0..MAX_RESERVATIONS {
        book.reserve(&id(index), tokens(1), None, None)?;
    }
    assert_eq!(book.reservations(), MAX_RESERVATIONS);
    assert_eq!(
        book.reserve(&id(MAX_RESERVATIONS), tokens(1), None, None),
        Err(Refusal::ReservationLimit)
    );
    assert_eq!(
        book.reservations(),
        MAX_RESERVATIONS,
        "the refused reservation was not allocated"
    );
    Ok(())
}

/// T10-AC-23 · the count bound is checked before the identity is parsed, so a full ledger
/// refuses for the reason that actually stopped it.
#[test]
fn the_count_bound_precedes_identity_parsing() -> Outcome {
    let mut book = ledger(u64::MAX, Ceiling::Soft);
    for index in 0..MAX_RESERVATIONS {
        book.reserve(&id(index), tokens(1), None, None)?;
    }
    assert_eq!(
        book.reserve("not-a-uuid", tokens(1), None, None),
        Err(Refusal::ReservationLimit),
        "the bound that stopped it is the one reported"
    );
    Ok(())
}

/// T10-AC-24 · a zero reservation is admitted and holds nothing; it is a legitimate way to
/// register work whose cost is not yet known.
#[test]
fn a_zero_reservation_is_admitted_and_holds_nothing() -> Outcome {
    let mut book = ledger(100, Ceiling::Soft);
    assert_eq!(book.reserve(&id(1), tokens(0), None, None)?, tokens(100));
    assert_eq!(columns(book.balance()?)?, (100, 0, 0, 0));
    assert_eq!(book.reservation(&id(1))?.held, tokens(0));
    Ok(())
}

// ------------------------------------------------- parent, child and concurrent work

/// T10-AC-25 · child work draws from the same limit as its parent, so a parent and a child
/// together cannot exceed the configured bound. This is the acceptance text's *"concurrent
/// reservations cannot exceed configured bounds"*.
#[test]
fn parent_and_child_draw_from_one_limit() -> Outcome {
    let mut book = ledger(100, Ceiling::Soft);
    book.reserve(&id(1), tokens(60), None, None)?;
    book.reserve(&id(2), tokens(40), Some(&id(1)), None)?;
    assert_eq!(columns(book.balance()?)?, (0, 100, 0, 0));
    assert_eq!(
        book.reserve(&id(3), tokens(1), Some(&id(1)), None),
        Err(Refusal::InsufficientBudget),
        "a child cannot conjure budget the scope does not have"
    );
    assert!(book.conserves()?);
    Ok(())
}

/// T10-AC-26 · a chain of children each draws from the one limit; four levels deep is still
/// bounded by the scope, not by the depth.
#[test]
fn a_chain_of_children_is_bounded_by_the_scope() -> Outcome {
    let mut book = ledger(100, Ceiling::Soft);
    book.reserve(&id(1), tokens(25), None, None)?;
    book.reserve(&id(2), tokens(25), Some(&id(1)), None)?;
    book.reserve(&id(3), tokens(25), Some(&id(2)), None)?;
    book.reserve(&id(4), tokens(25), Some(&id(3)), None)?;
    assert_eq!(columns(book.balance()?)?, (0, 100, 0, 0));
    assert_eq!(
        book.reserve(&id(5), tokens(1), Some(&id(4)), None),
        Err(Refusal::InsufficientBudget)
    );
    Ok(())
}

/// T10-AC-27 · the child records its parent, and the parent records none.
#[test]
fn a_child_records_its_parent() -> Outcome {
    let mut book = ledger(100, Ceiling::Soft);
    book.reserve(&id(1), tokens(10), None, None)?;
    book.reserve(&id(2), tokens(10), Some(&id(1)), None)?;
    let parent = book.reservation(&id(1))?;
    let child = book.reservation(&id(2))?;
    assert!(parent.parent.is_none());
    assert_eq!(
        child.parent.map(|p| p.as_str().to_owned()),
        Some(id(1)),
        "the child names its parent"
    );
    Ok(())
}

/// T10-AC-28 · a reservation cannot be its own parent, and the refusal is distinct from
/// "unknown parent" so the diagnosis is not misleading.
#[test]
fn a_reservation_cannot_parent_itself() {
    let mut book = ledger(100, Ceiling::Soft);
    assert_eq!(
        book.reserve(&id(1), tokens(10), Some(&id(1)), None),
        Err(Refusal::SelfParent)
    );
    assert_eq!(book.reservations(), 0);
}

/// T10-AC-29 · an unknown parent is refused, and so is a parent that exists but is closed —
/// a child cannot be adopted by finished work.
#[test]
fn an_unknown_or_closed_parent_is_refused() -> Outcome {
    let mut book = ledger(100, Ceiling::Soft);
    assert_eq!(
        book.reserve(&id(2), tokens(10), Some(&id(99)), None),
        Err(Refusal::UnknownParent)
    );
    book.reserve(&id(1), tokens(10), None, None)?;
    book.release(&id(1), settled(904)?)?;
    assert_eq!(
        book.reserve(&id(2), tokens(10), Some(&id(1)), None),
        Err(Refusal::UnknownParent),
        "a closed parent cannot adopt"
    );
    Ok(())
}

/// T10-AC-30 · a malformed parent identity is refused as a malformed identity, not as an
/// unknown parent.
#[test]
fn a_malformed_parent_identity_is_refused_as_malformed() {
    use habitat_engine::contracts::ScalarError;
    let mut book = ledger(100, Ceiling::Soft);
    assert_eq!(
        book.reserve(&id(1), tokens(10), Some("nope"), None),
        Err(Refusal::MalformedIdentity(ScalarError::InvalidUuid))
    );
}

/// T10-AC-31 · releasing a parent while a child is still open is refused; otherwise the
/// child's hold would be parented to a closed entry that nothing could later settle.
#[test]
fn a_parent_cannot_be_released_while_a_child_is_open() -> Outcome {
    let mut book = ledger(100, Ceiling::Soft);
    book.reserve(&id(1), tokens(50), None, None)?;
    book.reserve(&id(2), tokens(20), Some(&id(1)), None)?;
    assert_eq!(
        book.release(&id(1), settled(905)?),
        Err(Refusal::ChildOutstanding)
    );
    book.release(&id(2), settled(906)?)?;
    book.release(&id(1), settled(907)?)?;
    assert_eq!(columns(book.balance()?)?, (100, 0, 0, 0));
    Ok(())
}

/// T10-AC-32 · interleaving many reservations and releases never exceeds the limit and never
/// breaks conservation. This is the concurrency family expressed as an ordering: the ledger
/// is the single writer, so an interleaving is the only way two callers can meet.
#[test]
fn interleaved_reservations_and_releases_never_exceed_the_limit() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    let mut open: Vec<usize> = Vec::new();
    for round in 0..200_usize {
        if round % 3 == 2 {
            if let Some(index) = open.pop() {
                book.release(&id(index), settled(5_000 + index)?)?;
            }
        } else {
            let amount = tokens(u64::try_from(round % 7 + 1)?);
            if book.reserve(&id(round), amount, None, None).is_ok() {
                open.push(round);
            }
        }
        let balance = book.balance()?;
        assert!(book.conserves()?, "round {round}");
        assert!(
            balance.reserved().value() + balance.spent().value() + balance.unknown().value()
                <= 1_000,
            "round {round} exceeded the limit"
        );
    }
    Ok(())
}

// ---------------------------------------------- reports, retries and delayed usage

/// T10-AC-33 · a measured report moves cost from `reserved` to `spent`, exactly.
#[test]
fn a_measured_report_moves_held_into_spent() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(300), None, None)?;
    let applied = book.report(
        &id(1),
        "r1",
        Usage::new(tokens(120), Provenance::WorkerSettled),
    )?;
    assert_eq!(
        applied,
        Applied {
            fresh: true,
            held: tokens(180),
            discrepancy: None
        }
    );
    assert_eq!(columns(book.balance()?)?, (700, 180, 120, 0));
    Ok(())
}

/// T10-AC-34 · a retried report is idempotent **by identity**: the second application
/// changes nothing and says so, and two honest reports carrying the same amount under
/// different identities both count.
#[test]
fn retried_reports_are_idempotent_by_identity_not_by_amount() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(300), None, None)?;
    let first = book.report(
        &id(1),
        "r1",
        Usage::new(tokens(50), Provenance::WorkerSettled),
    )?;
    assert!(first.fresh);
    let retry = book.report(
        &id(1),
        "r1",
        Usage::new(tokens(50), Provenance::WorkerSettled),
    )?;
    assert!(!retry.fresh);
    assert_eq!(retry.held, tokens(250));
    assert_eq!(
        columns(book.balance()?)?,
        (700, 250, 50, 0),
        "no double count"
    );
    let distinct = book.report(
        &id(1),
        "r2",
        Usage::new(tokens(50), Provenance::WorkerSettled),
    )?;
    assert!(distinct.fresh, "a different identity is a different report");
    assert_eq!(columns(book.balance()?)?, (700, 200, 100, 0));
    Ok(())
}

/// T10-AC-35 · a retry of an *unmeasured* report is equally idempotent, so a lost reply
/// cannot inflate the unknown column.
#[test]
fn an_unmeasured_retry_does_not_inflate_the_unknown_column() -> Outcome {
    let mut book = ledger(500, Ceiling::Soft);
    book.reserve(&id(1), tokens(100), None, None)?;
    book.report(&id(1), "u1", Usage::new(tokens(30), Provenance::Unmeasured))?;
    book.report(&id(1), "u1", Usage::new(tokens(30), Provenance::Unmeasured))?;
    book.report(
        &id(1),
        "u1",
        Usage::new(tokens(999), Provenance::Unmeasured),
    )?;
    assert_eq!(
        columns(book.balance()?)?,
        (400, 70, 0, 30),
        "the first application stands; later ones with that identity change nothing"
    );
    Ok(())
}

/// T10-AC-36 · delayed usage that arrives after other reports still lands, in order, and
/// conservation holds throughout.
#[test]
fn delayed_usage_lands_after_later_reports() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(400), None, None)?;
    book.report(
        &id(1),
        "fast",
        Usage::new(tokens(100), Provenance::WorkerSettled),
    )?;
    book.report(
        &id(1),
        "checker",
        Usage::new(tokens(25), Provenance::CheckerMeasured),
    )?;
    book.report(
        &id(1),
        "slow",
        Usage::new(tokens(75), Provenance::WorkerSettled),
    )?;
    assert_eq!(columns(book.balance()?)?, (600, 200, 200, 0));
    assert!(book.conserves()?);
    Ok(())
}

/// T10-AC-37 · checker and compaction costs are included, not omitted — the complexity
/// boundary the contract names explicitly.
#[test]
fn checker_and_compaction_costs_are_counted() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(300), None, None)?;
    book.report(
        &id(1),
        "work",
        Usage::new(tokens(100), Provenance::WorkerSettled),
    )?;
    book.report(
        &id(1),
        "check",
        Usage::new(tokens(40), Provenance::CheckerMeasured),
    )?;
    book.report(
        &id(1),
        "compact",
        Usage::new(tokens(10), Provenance::CompactionMeasured),
    )?;
    let entry = book.reservation(&id(1))?;
    assert_eq!(entry.spent, tokens(150), "all three sources counted");
    assert_eq!(columns(book.balance()?)?, (700, 150, 150, 0));
    Ok(())
}

/// T10-AC-38 · an estimated figure counts as spend under a soft ceiling, so an estimate is
/// never free.
#[test]
fn an_estimate_is_counted_under_a_soft_ceiling() -> Outcome {
    let mut book = ledger(500, Ceiling::Soft);
    book.reserve(&id(1), tokens(200), None, None)?;
    book.report(&id(1), "est", Usage::new(tokens(60), Provenance::Estimated))?;
    assert_eq!(columns(book.balance()?)?, (300, 140, 60, 0));
    Ok(())
}

/// T10-AC-39 · unknown usage is visible: it lands in its own column and no read collapses it
/// into `spent` or returns it to `available`.
#[test]
fn unknown_usage_is_visible_and_never_becomes_available() -> Outcome {
    let mut book = ledger(500, Ceiling::Soft);
    book.reserve(&id(1), tokens(200), None, None)?;
    book.report(&id(1), "u", Usage::new(tokens(80), Provenance::Unmeasured))?;
    let balance = book.balance()?;
    assert_eq!(balance.unknown(), tokens(80));
    assert_eq!(balance.spent(), tokens(0), "unmeasured is not spend");
    assert_eq!(balance.available()?, tokens(300), "and it is not available");
    assert_eq!(book.reservation(&id(1))?.unknown, tokens(80));
    Ok(())
}

/// T10-AC-40 · an overrun beyond the held amount is recorded as a discrepancy rather than
/// refused: the cost was already incurred, and refusing it would make the books cheaper than
/// reality.
#[test]
fn an_overrun_is_recorded_as_a_discrepancy() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(100), None, None)?;
    let applied = book.report(
        &id(1),
        "r1",
        Usage::new(tokens(150), Provenance::WorkerSettled),
    )?;
    assert_eq!(applied.held, tokens(0));
    assert_eq!(applied.discrepancy, Some(tokens(50)));
    assert_eq!(columns(book.balance()?)?, (850, 0, 150, 0));
    assert!(book.conserves()?);
    Ok(())
}

/// T10-AC-41 · an overrun larger than the whole remaining scope is still recorded; the part
/// the scope cannot cover stays visible rather than vanishing.
#[test]
fn an_overrun_beyond_the_scope_is_still_recorded() -> Outcome {
    let mut book = ledger(100, Ceiling::Soft);
    book.reserve(&id(1), tokens(100), None, None)?;
    let applied = book.report(
        &id(1),
        "r1",
        Usage::new(tokens(250), Provenance::WorkerSettled),
    )?;
    assert_eq!(applied.discrepancy, Some(tokens(150)));
    let balance = book.balance()?;
    assert_eq!(
        balance.spent(),
        tokens(250),
        "the whole cost is on the books"
    );
    assert_eq!(
        balance.available(),
        Err(Refusal::Underflow),
        "an over-spent scope says so rather than reporting a cheerful zero"
    );
    Ok(())
}

/// T10-AC-42 · the report count bound refuses at its limit, before the report is stored, and
/// the ledger keeps the reports it already had.
#[test]
fn the_report_count_bound_refuses_before_storing() -> Outcome {
    let mut book = ledger(u64::MAX, Ceiling::Soft);
    book.reserve(
        &id(1),
        tokens(u64::try_from(MAX_REPORTS_PER_RESERVATION)? + 10),
        None,
        None,
    )?;
    for index in 0..MAX_REPORTS_PER_RESERVATION {
        book.report(
            &id(1),
            &format!("r{index}"),
            Usage::new(tokens(1), Provenance::WorkerSettled),
        )?;
    }
    assert_eq!(
        book.report(
            &id(1),
            "one-too-many",
            Usage::new(tokens(1), Provenance::WorkerSettled)
        ),
        Err(Refusal::ReportLimit)
    );
    assert_eq!(
        book.reservation(&id(1))?.spent,
        tokens(u64::try_from(MAX_REPORTS_PER_RESERVATION)?),
        "the reports already applied are intact"
    );
    Ok(())
}

// ---------------------------------------------------------------- hard ceiling

/// T10-AC-43 · a hard ceiling rejects an unmeasured figure — the acceptance text's
/// *"hard-ceiling tasks reject incompatible accounting"* — and the refusal names the ceiling.
#[test]
fn a_hard_ceiling_rejects_unmeasured_accounting() -> Outcome {
    let mut book = ledger(500, Ceiling::Hard);
    book.reserve(&id(1), tokens(100), None, None)?;
    assert_eq!(
        book.report(&id(1), "u", Usage::new(tokens(10), Provenance::Unmeasured)),
        Err(Refusal::HardCeilingIncompatible)
    );
    assert_eq!(
        columns(book.balance()?)?,
        (400, 100, 0, 0),
        "nothing recorded"
    );
    Ok(())
}

/// T10-AC-44 · a hard ceiling also rejects an estimate, because an estimate is not exact
/// accounting; the three measured provenances are admitted.
#[test]
fn a_hard_ceiling_admits_exactly_the_measured_provenances() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Hard);
    book.reserve(&id(1), tokens(500), None, None)?;
    assert_eq!(
        book.report(&id(1), "e", Usage::new(tokens(10), Provenance::Estimated)),
        Err(Refusal::HardCeilingIncompatible)
    );
    for (index, provenance) in [
        Provenance::WorkerSettled,
        Provenance::CheckerMeasured,
        Provenance::CompactionMeasured,
    ]
    .into_iter()
    .enumerate()
    {
        book.report(
            &id(1),
            &format!("m{index}"),
            Usage::new(tokens(10), provenance),
        )?;
    }
    assert_eq!(columns(book.balance()?)?, (500, 470, 30, 0));
    Ok(())
}

/// T10-AC-45 · `Ceiling::admits` agrees with the ledger for every provenance, so the
/// predicate and the enforcement cannot drift apart.
#[test]
fn the_ceiling_predicate_agrees_with_the_ledger() -> Outcome {
    for ceiling in [Ceiling::Soft, Ceiling::Hard] {
        for (index, provenance) in Provenance::ALL.into_iter().enumerate() {
            let mut book = ledger(1_000, ceiling);
            book.reserve(&id(1), tokens(100), None, None)?;
            let outcome = book.report(
                &id(1),
                &format!("r{index}"),
                Usage::new(tokens(1), provenance),
            );
            assert_eq!(
                outcome.is_ok(),
                ceiling.admits(provenance),
                "{} / {}",
                ceiling.name(),
                provenance.name()
            );
        }
    }
    Ok(())
}

/// T10-AC-46 · the ceiling is reported back and its wire names are stable.
#[test]
fn ceiling_names_are_stable() {
    assert_eq!(Ceiling::Soft.name(), "soft");
    assert_eq!(Ceiling::Hard.name(), "hard");
    assert_eq!(ledger(1, Ceiling::Hard).ceiling(), Ceiling::Hard);
}

// -------------------------------------------- release, abandonment and liability

/// T10-AC-47 · a settled release returns the unspent remainder to `available`.
#[test]
fn a_settled_release_returns_the_remainder() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(300), None, None)?;
    book.report(
        &id(1),
        "r1",
        Usage::new(tokens(100), Provenance::WorkerSettled),
    )?;
    let balance = book.release(&id(1), settled(910)?)?;
    assert_eq!(columns(balance)?, (900, 0, 100, 0));
    assert!(!book.reservation(&id(1))?.open);
    Ok(())
}

/// T10-AC-48 · abandoned work keeps its remainder as a visible liability rather than
/// returning it — the engine cannot prove the cost was not incurred.
#[test]
fn abandoned_work_keeps_its_remainder_as_a_liability() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(300), None, None)?;
    book.report(
        &id(1),
        "r1",
        Usage::new(tokens(100), Provenance::WorkerSettled),
    )?;
    let balance = book.release(&id(1), abandoned(911)?)?;
    assert_eq!(
        columns(balance)?,
        (700, 0, 100, 200),
        "the 200 unspent stays on the books as unknown"
    );
    assert!(book.conserves()?);
    Ok(())
}

/// T10-AC-49 · uncertain cancellation accounts exactly like abandonment, and the settlement
/// record says which of the two happened.
#[test]
fn uncertain_cancellation_carries_the_same_liability() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    book.reserve(&id(1), tokens(300), None, None)?;
    let balance = book.release(&id(1), cancelled(912)?)?;
    assert_eq!(columns(balance)?, (700, 0, 0, 300));
    assert_eq!(cancelled(913)?.name(), "cancelled-uncertain");
    assert_eq!(abandoned(914)?.name(), "abandoned");
    assert_eq!(settled(915)?.name(), "settled");
    Ok(())
}

/// T10-AC-50 · only a settled attempt frees the remainder; the predicate is asserted
/// directly so the three settlement kinds cannot drift from the accounting.
#[test]
fn only_settlement_frees_the_remainder() -> Outcome {
    assert!(settled(916)?.frees_remainder());
    assert!(!abandoned(917)?.frees_remainder());
    assert!(!cancelled(918)?.frees_remainder());
    Ok(())
}

/// T10-AC-51 · a released reservation cannot be released again, and cannot take a report.
#[test]
fn a_closed_reservation_refuses_further_transitions() -> Outcome {
    let mut book = ledger(500, Ceiling::Soft);
    book.reserve(&id(1), tokens(100), None, None)?;
    book.release(&id(1), settled(919)?)?;
    assert_eq!(
        book.release(&id(1), settled(920)?),
        Err(Refusal::ReservationClosed)
    );
    assert_eq!(
        book.report(
            &id(1),
            "late",
            Usage::new(tokens(1), Provenance::WorkerSettled)
        ),
        Err(Refusal::ReservationClosed)
    );
    Ok(())
}

/// T10-AC-52 · an unknown reservation is refused by every operation that names one.
#[test]
fn an_unknown_reservation_is_refused_everywhere() -> Outcome {
    let mut book = ledger(500, Ceiling::Soft);
    assert_eq!(
        book.report(
            &id(9),
            "r",
            Usage::new(tokens(1), Provenance::WorkerSettled)
        ),
        Err(Refusal::UnknownReservation)
    );
    assert_eq!(
        book.release(&id(9), settled(921)?),
        Err(Refusal::UnknownReservation)
    );
    assert_eq!(book.reservation(&id(9)), Err(Refusal::UnknownReservation));
    Ok(())
}

/// T10-AC-53 · a settlement whose attempt identity is malformed is refused before the
/// reservation is touched.
#[test]
fn a_malformed_settlement_attempt_is_refused_first() -> Outcome {
    use habitat_engine::contracts::ScalarError;
    let mut book = ledger(500, Ceiling::Soft);
    book.reserve(&id(1), tokens(100), None, None)?;
    // A UuidV4 cannot be constructed from a malformed string, so the refusal is proved at
    // the parse boundary the settlement depends on.
    assert_eq!(
        UuidV4::parse("not-a-uuid").map(|_| ()),
        Err(ScalarError::InvalidUuid)
    );
    assert_eq!(columns(book.balance()?)?, (400, 100, 0, 0));
    Ok(())
}

/// T10-AC-54 · a full abandonment of everything leaves nothing available and everything
/// visible, and conservation still holds.
#[test]
fn total_abandonment_leaves_the_whole_limit_visible() -> Outcome {
    let mut book = ledger(300, Ceiling::Soft);
    book.reserve(&id(1), tokens(100), None, None)?;
    book.reserve(&id(2), tokens(100), None, None)?;
    book.reserve(&id(3), tokens(100), None, None)?;
    for index in 1..=3 {
        book.release(&id(index), abandoned(930 + index)?)?;
    }
    assert_eq!(columns(book.balance()?)?, (0, 0, 0, 300));
    assert!(book.conserves()?);
    Ok(())
}

// ---------------------------------------------------------------- reads and diagnostics

/// T10-AC-55 · the reservation list returns every reservation, oldest first, open and closed.
#[test]
fn the_reservation_list_returns_every_entry_in_order() -> Outcome {
    let mut book = ledger(1_000, Ceiling::Soft);
    for index in 1..=4 {
        book.reserve(&id(index), tokens(10), None, None)?;
    }
    book.release(&id(2), settled(940)?)?;
    let list = book.reservation_list()?;
    assert_eq!(list.len(), 4);
    let identities: Vec<String> = list
        .iter()
        .map(|r| r.identity.as_str().to_owned())
        .collect();
    assert_eq!(identities, (1..=4).map(id).collect::<Vec<_>>());
    assert_eq!(
        list.iter().filter(|r| r.open).count(),
        3,
        "the released one is closed and still listed"
    );
    Ok(())
}

/// T10-AC-56 · every refusal has a distinct, stable diagnostic name, and no name is a
/// substring of another — otherwise a log grep for one would match another.
#[test]
fn refusal_names_are_distinct_and_non_overlapping() {
    use habitat_engine::contracts::ScalarError;
    let all = [
        Refusal::IncompatibleUnit,
        Refusal::Overflow,
        Refusal::Underflow,
        Refusal::UnknownUnit,
        Refusal::InsufficientBudget,
        Refusal::HardCeilingIncompatible,
        Refusal::MalformedIdentity(ScalarError::InvalidUuid),
        Refusal::UnknownReservation,
        Refusal::DuplicateReservation,
        Refusal::ReservationClosed,
        Refusal::UnknownParent,
        Refusal::SelfParent,
        Refusal::ReservationLimit,
        Refusal::ReportLimit,
        Refusal::StaleRevision,
        Refusal::ChildOutstanding,
        Refusal::Scalar(ScalarError::Empty),
    ];
    for (i, a) in all.iter().enumerate() {
        assert!(!a.name().is_empty());
        for (j, b) in all.iter().enumerate() {
            if i != j {
                assert!(
                    !a.name().contains(b.name()),
                    "{} contains {}",
                    a.name(),
                    b.name()
                );
            }
        }
    }
}

/// T10-AC-57 · a refusal that carries a scalar error shows both, so the diagnosis names the
/// rule and the reason; one that does not shows only its own name.
#[test]
fn refusal_display_shows_the_carried_scalar_error() {
    use habitat_engine::contracts::ScalarError;
    assert_eq!(
        Refusal::MalformedIdentity(ScalarError::InvalidUuid).to_string(),
        "malformed reservation identity: expected a lowercase hyphenated UUIDv4"
    );
    assert_eq!(
        Refusal::InsufficientBudget.to_string(),
        "insufficient remaining budget"
    );
}

/// T10-AC-58 · a ledger read back with a broken invariant refuses rather than reporting a
/// cheerful zero. `conserves` is the predicate a store consults before trusting what it
/// loaded, so it must be able to say "no".
#[test]
fn an_overspent_ledger_reports_that_it_does_not_conserve() -> Outcome {
    let mut book = ledger(50, Ceiling::Soft);
    book.reserve(&id(1), tokens(50), None, None)?;
    book.report(
        &id(1),
        "r",
        Usage::new(tokens(500), Provenance::WorkerSettled),
    )?;
    assert_eq!(book.conserves(), Err(Refusal::Underflow));
    assert_eq!(book.available(), Err(Refusal::Underflow));
    assert_eq!(book.balance()?.spent(), tokens(500));
    Ok(())
}

// ---------------------------------------------------------------------------------------
// T10 clause: "fallback cannot cross privacy or tool constraints".
//
// The selector can only narrow, so the cases below are written from BOTH sides: a candidate
// at the same class and a subset of tools must be chosen, and every way of exceeding either
// must be refused by name. A suite that only checked the refusals would pass against a
// selector that refused everything.
// ---------------------------------------------------------------------------------------

/// A constraint permitting `privacy` and `tools`, for a case that does not care about the
/// error path of building one.
fn constraint(privacy: Privacy, tools: &[&str]) -> Constraint {
    Constraint::new(privacy, tools).expect("well-formed constraint")
}

fn candidate(identity: &str, privacy: Privacy, tools: &[&str], cost: u64) -> Candidate {
    Candidate::new(identity, privacy, tools, tokens(cost)).expect("well-formed candidate")
}

#[test]
fn privacy_classes_are_ordered_most_restrictive_first() {
    assert_eq!(
        Privacy::ALL,
        [
            Privacy::InProcess,
            Privacy::OnHost,
            Privacy::AdmittedExternal
        ]
    );
    // The ordering IS the rule, so it is asserted directly rather than only through the
    // selector that reads it.
    assert!(Privacy::InProcess < Privacy::OnHost);
    assert!(Privacy::OnHost < Privacy::AdmittedExternal);
}

#[test]
fn a_class_admits_itself_and_everything_more_restrictive() {
    for required in Privacy::ALL {
        assert!(
            required.admits(required),
            "{} admits itself",
            required.name()
        );
        for candidate in Privacy::ALL {
            assert_eq!(
                required.admits(candidate),
                (candidate as u8) <= (required as u8),
                "{} admits {}",
                required.name(),
                candidate.name()
            );
        }
    }
}

#[test]
fn every_privacy_name_round_trips() -> Outcome {
    for class in Privacy::ALL {
        assert_eq!(Privacy::parse(class.name())?, class);
    }
    assert_eq!(
        Privacy::parse("anywhere"),
        Err(Refusal::UnknownPrivacyClass)
    );
    assert_eq!(Privacy::parse(""), Err(Refusal::UnknownPrivacyClass));
    Ok(())
}

#[test]
fn the_cheapest_admissible_candidate_is_chosen() -> Outcome {
    let ledger = ledger(1000, Ceiling::Soft);
    let permitted = constraint(Privacy::OnHost, &["read", "search"]);
    let selection = ledger.select_fallback(
        &permitted,
        &[
            candidate("expensive", Privacy::InProcess, &["read"], 400),
            candidate("cheap", Privacy::OnHost, &["read", "search"], 120),
            candidate("middle", Privacy::InProcess, &[], 200),
        ],
    )?;
    assert_eq!(selection.chosen().identity(), "cheap");
    assert_eq!(selection.chosen().cost().value(), 120);
    // The two that lost are named, so a reader can see what existed and was not taken.
    let mut losers: Vec<&str> = selection
        .rejected()
        .iter()
        .map(habitat_engine::budget::Rejected::identity)
        .collect();
    losers.sort_unstable();
    assert_eq!(losers, ["expensive", "middle"]);
    Ok(())
}

#[test]
fn a_candidate_at_a_looser_privacy_class_is_refused_by_name() -> Outcome {
    let ledger = ledger(1000, Ceiling::Soft);
    let permitted = constraint(Privacy::OnHost, &["read"]);
    // Cheapest by far, and still refused: cost cannot buy a privacy crossing.
    let error = ledger
        .select_fallback(
            &permitted,
            &[candidate("remote", Privacy::AdmittedExternal, &["read"], 1)],
        )
        .expect_err("a looser class must not be selected");
    assert_eq!(error, Refusal::NoAdmissibleFallback);
    // And when something else IS admissible, the crossing is recorded against the candidate
    // rather than disappearing into the winner's shadow.
    let selection = ledger.select_fallback(
        &permitted,
        &[
            candidate("remote", Privacy::AdmittedExternal, &["read"], 1),
            candidate("local", Privacy::InProcess, &["read"], 900),
        ],
    )?;
    assert_eq!(selection.chosen().identity(), "local");
    assert_eq!(selection.rejected().len(), 1);
    assert_eq!(selection.rejected()[0].identity(), "remote");
    assert_eq!(selection.rejected()[0].reason(), Refusal::PrivacyCrossing);
    Ok(())
}

#[test]
fn every_looser_class_is_refused_for_every_constraint() -> Outcome {
    // The whole matrix, not one example: nine pairs, and each is either chosen or refused
    // exactly as the ordering says.
    let ledger = ledger(1000, Ceiling::Soft);
    for required in Privacy::ALL {
        for offered in Privacy::ALL {
            let permitted = constraint(required, &["read"]);
            let result =
                ledger.select_fallback(&permitted, &[candidate("c", offered, &["read"], 10)]);
            if required.admits(offered) {
                assert_eq!(result?.chosen().identity(), "c");
            } else {
                assert_eq!(
                    result.expect_err("a looser class must be refused"),
                    Refusal::NoAdmissibleFallback,
                    "{} offered under {}",
                    offered.name(),
                    required.name()
                );
            }
        }
    }
    Ok(())
}

#[test]
fn a_candidate_needing_an_unpermitted_tool_is_refused() -> Outcome {
    let ledger = ledger(1000, Ceiling::Soft);
    let permitted = constraint(Privacy::AdmittedExternal, &["read"]);
    let selection = ledger.select_fallback(
        &permitted,
        &[
            candidate("writer", Privacy::InProcess, &["read", "write"], 10),
            candidate("reader", Privacy::InProcess, &["read"], 500),
        ],
    )?;
    assert_eq!(selection.chosen().identity(), "reader");
    assert_eq!(selection.rejected()[0].identity(), "writer");
    assert_eq!(selection.rejected()[0].reason(), Refusal::ToolNotPermitted);
    Ok(())
}

#[test]
fn a_candidate_needing_no_tool_is_admissible_under_any_tool_set() -> Outcome {
    // The empty set is a subset of every set. Stated because an implementation that checked
    // "the candidate's tools intersect the permitted ones" would refuse this and pass every
    // other case above.
    let ledger = ledger(1000, Ceiling::Soft);
    for tools in [&[][..], &["read"][..], &["read", "write"][..]] {
        let permitted = constraint(Privacy::InProcess, tools);
        let selection =
            ledger.select_fallback(&permitted, &[candidate("bare", Privacy::InProcess, &[], 5)])?;
        assert_eq!(selection.chosen().identity(), "bare");
    }
    Ok(())
}

#[test]
fn a_candidate_beyond_the_remaining_budget_is_refused_not_chosen() -> Outcome {
    let mut ledger = ledger(1000, Ceiling::Soft);
    ledger.reserve(&id(1), tokens(900), None, None)?;
    // 100 available. The ordering of the two grounds matters: the expensive candidate also
    // satisfies privacy and tools, so InsufficientBudget is the only reason it can carry.
    let selection = ledger.select_fallback(
        &constraint(Privacy::InProcess, &["read"]),
        &[
            candidate("big", Privacy::InProcess, &["read"], 101),
            candidate("small", Privacy::InProcess, &["read"], 100),
        ],
    )?;
    assert_eq!(selection.chosen().identity(), "small");
    assert_eq!(
        selection.rejected()[0].reason(),
        Refusal::InsufficientBudget
    );
    Ok(())
}

#[test]
fn a_candidate_priced_in_another_unit_is_refused() -> Outcome {
    let ledger = ledger(1000, Ceiling::Soft);
    let other = Unit::ALL
        .into_iter()
        .find(|unit| *unit != Unit::Tokens)
        .expect("a second unit");
    let priced = Candidate::new("foreign", Privacy::InProcess, &[], Amount::new(other, 1))?;
    let selection = ledger.select_fallback(
        &constraint(Privacy::InProcess, &[]),
        &[priced, candidate("native", Privacy::InProcess, &[], 10)],
    )?;
    assert_eq!(selection.chosen().identity(), "native");
    assert_eq!(selection.rejected()[0].reason(), Refusal::IncompatibleUnit);
    Ok(())
}

#[test]
fn an_empty_candidate_list_refuses_rather_than_returning_nothing() {
    let ledger = ledger(1000, Ceiling::Soft);
    assert_eq!(
        ledger.select_fallback(&constraint(Privacy::InProcess, &[]), &[]),
        Err(Refusal::NoAdmissibleFallback)
    );
}

#[test]
fn the_candidate_bound_is_checked_from_both_sides() -> Outcome {
    let ledger = ledger(1_000_000, Ceiling::Soft);
    let permitted = constraint(Privacy::InProcess, &[]);
    let build = |n: usize| -> Vec<Candidate> {
        (0..n)
            .map(|index| candidate(&format!("c{index}"), Privacy::InProcess, &[], 1))
            .collect()
    };
    assert_eq!(
        ledger
            .select_fallback(&permitted, &build(MAX_CANDIDATES))?
            .rejected()
            .len(),
        MAX_CANDIDATES - 1
    );
    assert_eq!(
        ledger.select_fallback(&permitted, &build(MAX_CANDIDATES + 1)),
        Err(Refusal::CandidateLimit)
    );
    Ok(())
}

#[test]
fn the_tool_bound_is_checked_from_both_sides() {
    let names: Vec<String> = (0..=MAX_TOOLS).map(|index| format!("t{index}")).collect();
    let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    assert!(Constraint::new(Privacy::InProcess, &borrowed[..MAX_TOOLS]).is_ok());
    assert_eq!(
        Constraint::new(Privacy::InProcess, &borrowed),
        Err(Refusal::ToolLimit)
    );
    assert_eq!(
        Candidate::new(
            "c",
            Privacy::InProcess,
            &borrowed,
            Amount::new(Unit::Tokens, 1)
        ),
        Err(Refusal::ToolLimit)
    );
}

#[test]
fn a_malformed_constraint_or_candidate_is_refused() {
    assert_eq!(
        Constraint::new(Privacy::InProcess, &[""]),
        Err(Refusal::MalformedConstraint)
    );
    assert_eq!(
        Constraint::new(Privacy::InProcess, &["read", "read"]),
        Err(Refusal::MalformedConstraint)
    );
    assert_eq!(
        Candidate::new("", Privacy::InProcess, &[], Amount::new(Unit::Tokens, 1)),
        Err(Refusal::MalformedConstraint)
    );
}

#[test]
fn two_candidates_sharing_one_identity_are_refused() {
    // Found by mutation testing: `<` and `<=` in the tie-break were indistinguishable,
    // because the only input that could tell them apart was a duplicate identity -- and with
    // equal cost and equal identity the two still differ in privacy or tools, so "whichever
    // came first" would have silently chosen a different capability set. Refused instead, as
    // the module already refuses a duplicate reservation and a duplicate tool.
    let ledger = ledger(1000, Ceiling::Soft);
    let permitted = constraint(Privacy::OnHost, &["read"]);
    assert_eq!(
        ledger.select_fallback(
            &permitted,
            &[
                candidate("twin", Privacy::OnHost, &["read"], 50),
                candidate("twin", Privacy::InProcess, &[], 50),
            ],
        ),
        Err(Refusal::DuplicateCandidate)
    );
    // Non-adjacent duplicates too: the check reads every earlier candidate, not just the one
    // before. A neighbour-only comparison passes the case above and fails this one.
    assert_eq!(
        ledger.select_fallback(
            &permitted,
            &[
                candidate("twin", Privacy::OnHost, &["read"], 50),
                candidate("other", Privacy::OnHost, &[], 10),
                candidate("twin", Privacy::InProcess, &[], 50),
            ],
        ),
        Err(Refusal::DuplicateCandidate)
    );
}

#[test]
fn distinct_identities_at_one_price_are_admitted() -> Outcome {
    // The other side of the refusal: sharing a COST is ordinary and must still select.
    let ledger = ledger(1000, Ceiling::Soft);
    let selection = ledger.select_fallback(
        &constraint(Privacy::OnHost, &["read"]),
        &[
            candidate("beta", Privacy::OnHost, &["read"], 50),
            candidate("alpha", Privacy::InProcess, &[], 50),
        ],
    )?;
    assert_eq!(selection.chosen().identity(), "alpha");
    Ok(())
}

#[test]
fn selection_is_deterministic_when_costs_tie() -> Outcome {
    // Two candidates at the same price. Without the identity tie-break the winner would
    // depend on argument order, and a caller re-offering the same set could get either.
    let ledger = ledger(1000, Ceiling::Soft);
    let permitted = constraint(Privacy::InProcess, &[]);
    let alpha = candidate("alpha", Privacy::InProcess, &[], 50);
    let beta = candidate("beta", Privacy::InProcess, &[], 50);
    let forward = ledger.select_fallback(&permitted, &[alpha.clone(), beta.clone()])?;
    let reversed = ledger.select_fallback(&permitted, &[beta, alpha])?;
    assert_eq!(forward.chosen().identity(), "alpha");
    assert_eq!(reversed.chosen().identity(), "alpha");
    Ok(())
}

#[test]
fn a_constraint_sorts_its_tools_so_two_spellings_of_one_set_are_equal() -> Outcome {
    assert_eq!(
        Constraint::new(Privacy::OnHost, &["search", "read"])?,
        Constraint::new(Privacy::OnHost, &["read", "search"])?
    );
    assert_eq!(
        constraint(Privacy::OnHost, &["search", "read"]).tools(),
        ["read", "search"]
    );
    Ok(())
}

#[test]
fn a_constraint_only_ever_reports_what_it_was_built_with() -> Outcome {
    // There is no widening method, so the checkable statement is that what goes in is what
    // comes out -- and that a tool outside the set is not permitted.
    let permitted = Constraint::new(Privacy::OnHost, &["read", "search"])?;
    assert_eq!(permitted.privacy(), Privacy::OnHost);
    assert!(permitted.permits_tool("read") && permitted.permits_tool("search"));
    assert!(!permitted.permits_tool("write") && !permitted.permits_tool(""));
    Ok(())
}
