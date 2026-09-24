// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/cohort.rs
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
// Readiness binding: HEE3-READINESS-001; SHA-256 f574043487f39db6424c4988bce58e88a6e766f02f974fbcf3fa8dd6e0003548; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-07, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05; runtime proof pending; original task DAG controls.
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
// Readiness binding: HEE3-READINESS-001; SHA-256 f574043487f39db6424c4988bce58e88a6e766f02f974fbcf3fa8dd6e0003548; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-02, R90-03, R90-04, R90-05, R90-09, R90-10; resolved contracts RC02, RC03, RC04, RC05; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-contracts; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-contracts (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-contracts
// Owns: Boundary data types, identities, errors and schema versions
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/contracts.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-contracts)
// Build dependencies: none declared
// Consumers: task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, app, actions
// Related task contracts: T01, T02, T03, T04, T05, T06, T07, T08, T09, T10, T11, T13, T14, T16, T17, T18, T19, T20, T21, T22, T23, T25, T26, T27, T28, T29
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K1)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB03)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [task TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
// [task TASK-T02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T02)
// [task TASK-T03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T03)
// [task TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
// [task TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
// [task TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
// [task TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
// [task TASK-T08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T08)
// [task TASK-T09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T09)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
// [task TASK-T11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T11)
// [task TASK-T13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T13)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T16](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T16)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21)
// [task TASK-T22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T22)
// [task TASK-T23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T23)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [task TASK-T28](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T28)
// [task TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
// [separate reference example EX-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-contracts)
// [handbook HB-compatibility-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-compatibility-map)
// [handbook HB-error-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-error-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-contracts)
// [plan SEC-api-sockets](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-api-sockets)
// [plan SEC-architecture](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-architecture)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-security](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-security)
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
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [source SRC-A08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A08)
// [source SRC-C01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C01)
// [source SRC-C02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C02)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-contracts)
// [corpus architecture and verification schematic Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FIndex)
// [corpus architecture and verification schematic CS01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS01)
// [corpus architecture and verification schematic CS02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS02)
// [corpus architecture and verification schematic CS03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS03)
// [corpus architecture and verification schematic CS04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS04)
// [corpus architecture and verification schematic CS05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS05)
// [corpus architecture and verification schematic CS06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FCorpus%2FCS06)
// [resolved design contracts Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex)
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
// [readiness improvement grouping R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02)
// [readiness improvement grouping R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
// [readiness improvement grouping R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04)
// [readiness improvement grouping R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-contracts)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
// [applied learning LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03)
// [applied learning LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
// [diary evidence source DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
// Applicable learning IDs: LRN01, LRN03, LRN04, LRN08; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What Prototyping Is For](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20Prototyping%20Is%20For)
// [Why I Stopped Trusting Green](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhy%20I%20Stopped%20Trusting%20Green)
// HEE3-ANCHORS-END

//! Bounded specialist threads, disjoint write ownership and evidence-aware parent joins.
//!
//! `docs/modules/cohort.md` fixes the boundary: *"No second scheduler or majority vote
//! replacing proof; synergy must show useful integration, not agent count."* Nothing here
//! schedules, and nothing counts votes. A join succeeds because every required child met its
//! criterion on a current brief with disjoint writes — never because most of them agreed.
//!
//! Three rules are structural:
//!
//! * **One coordinator owns assignment and join.** [`Cohort`] is the only type that mutates
//!   a thread, so "two schedulers" is not a state this module can reach.
//! * **A child never accepts the parent.** [`Cohort::join`] is the only producer of a
//!   [`Join`], and it takes the parent's required criteria; a child can only
//!   [`Cohort::report`] its own outcome. There is no method by which a child's success
//!   becomes the parent's.
//! * **Overlapping writes cannot be assigned.** [`Cohort::assign`] refuses a claim that
//!   intersects a live one, so two threads owning one path is refused at assignment rather
//!   than detected at join, when the damage is already done.
//!
//! Dissent is preserved rather than resolved. A child that disagrees is recorded with its
//! reason and blocks the join; a module that averaged it away would be the majority vote the
//! contract forbids.

use std::fmt;

use crate::contracts::{ScalarError, UuidV4};

/// The most threads one cohort admits.
pub const MAX_THREADS: usize = 64;

/// The most resource claims one thread may hold.
pub const MAX_CLAIMS: usize = 64;

/// The most dependencies one thread may declare.
pub const MAX_DEPENDENCIES: usize = 64;

/// Schema version of the persisted cohort shape.
pub const SCHEMA_VERSION: i64 = 1;

/// A reason this module refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// An identity that is not a lowercase hyphenated `UUIDv4`.
    MalformedIdentity(ScalarError),
    /// The cohort already holds [`MAX_THREADS`] threads.
    ThreadLimit,
    /// One thread declared more than [`MAX_CLAIMS`] resource claims.
    ClaimLimit,
    /// One thread declared more than [`MAX_DEPENDENCIES`] dependencies.
    DependencyLimit,
    /// A second thread claimed an identity already assigned.
    DuplicateThread,
    /// The named thread is not in this cohort.
    UnknownThread,
    /// A declared dependency is not a thread in this cohort.
    UnknownDependency,
    /// A thread cannot depend on itself.
    SelfDependency,
    /// The declared dependencies would close a cycle. A finite child DAG is required.
    DependencyCycle,
    /// A resource claim intersects a claim another live thread already holds.
    OverlappingClaim,
    /// The thread's brief revision is older than the cohort's.
    StaleBrief,
    /// The thread has already reported, and an outcome is not revised in place.
    AlreadyReported,
    /// A join was attempted while a required thread has not reported.
    ChildOutstanding,
    /// An empty claim path was declared.
    EmptyClaim,
    /// A claim path that is not in canonical form: absolute, or with an empty, `.` or `..`
    /// segment. Such a path could name a claimed resource without matching it (review N3).
    NonCanonicalClaim,
    /// A brief revision lower than the current one: a regression would re-admit work done
    /// against an older brief (review D5).
    BriefRegressed,
}

impl Refusal {
    /// The stable diagnostic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MalformedIdentity(_) => "malformed cohort identity",
            Self::ThreadLimit => "thread count bound reached",
            Self::ClaimLimit => "resource claim bound reached",
            Self::DependencyLimit => "declared dependency bound reached",
            Self::DuplicateThread => "duplicate thread identity",
            Self::UnknownThread => "unknown thread",
            Self::UnknownDependency => "unknown dependency thread",
            Self::SelfDependency => "a thread cannot depend on itself",
            Self::DependencyCycle => "declared dependencies close a cycle",
            Self::OverlappingClaim => "resource claim overlaps a live claim",
            Self::StaleBrief => "thread brief revision precedes the cohort revision",
            Self::AlreadyReported => "thread has already reported",
            Self::ChildOutstanding => "a required thread has not reported",
            Self::EmptyClaim => "an empty resource claim",
            Self::NonCanonicalClaim => "a resource claim not in canonical form",
            Self::BriefRegressed => "a brief revision lower than the current one",
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedIdentity(error) => write!(f, "{}: {error}", self.name()),
            other => f.write_str(other.name()),
        }
    }
}

impl std::error::Error for Refusal {}

/// What a specialist thread concluded.
///
/// There is no `Approve`: a child reports what it did and what it observed, and the parent
/// decides. `Dissent` carries a reason precisely so it cannot be reduced to a tally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The thread met its criterion and its evidence supports it.
    Met,
    /// The thread did not meet its criterion.
    Unmet,
    /// The thread completed but disagrees with the brief or with a sibling's evidence.
    ///
    /// A join with a live dissent is blocked, not outvoted.
    Dissent,
    /// The thread could not reach a conclusion. Not the same as `Unmet`.
    Indeterminate,
}

impl Outcome {
    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::Unmet => "unmet",
            Self::Dissent => "dissent",
            Self::Indeterminate => "indeterminate",
        }
    }

    /// Every outcome.
    pub const ALL: [Self; 4] = [Self::Met, Self::Unmet, Self::Dissent, Self::Indeterminate];

    /// Whether this outcome permits the parent to integrate the thread's work.
    ///
    /// Only [`Outcome::Met`] does. Three of four do not, which is the asymmetry the
    /// complexity boundary demands: integration is earned, not voted for.
    #[must_use]
    pub const fn permits_integration(self) -> bool {
        matches!(self, Self::Met)
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// An exclusive claim on a write path.
///
/// Overlap is prefix-based on path segments, because a thread owning `src/store` and one
/// owning `src/store/reconciliation.rs` are the same conflict as two owning the same file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    path: String,
}

impl Claim {
    /// A claim on `path`, which must be in canonical form: relative, with no empty, `.` or
    /// `..` segment. The overlap rule compares segments, so a path that spells one resource two
    /// ways (`src/store/` for `src/store`) would otherwise slip past a sibling's claim — every
    /// statement of the rule read `src/store/` as `[src, store, ""]` (review N3).
    ///
    /// # Errors
    ///
    /// * [`Refusal::EmptyClaim`] for an empty path;
    /// * [`Refusal::NonCanonicalClaim`] for any other path not in canonical form.
    pub fn new(path: &str) -> Result<Self, Refusal> {
        if path.is_empty() {
            return Err(Refusal::EmptyClaim);
        }
        if path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        {
            return Err(Refusal::NonCanonicalClaim);
        }
        Ok(Self {
            path: path.to_owned(),
        })
    }

    /// The claimed path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Whether two claims conflict.
    ///
    /// True when either path is a segment-wise prefix of the other. Comparing whole segments
    /// matters: `src/store` must not be read as a prefix of `src/storefront`.
    #[must_use]
    pub fn conflicts_with(&self, other: &Self) -> bool {
        let (a, b) = (self.path.as_str(), other.path.as_str());
        let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
        long == short || long.starts_with(&format!("{short}/"))
    }
}

/// One specialist thread's record.
#[derive(Clone, Debug)]
struct Thread {
    identity: String,
    brief: u64,
    dependencies: Vec<String>,
    claims: Vec<Claim>,
    required: bool,
    outcome: Option<Outcome>,
    evidence: Option<String>,
}

/// A thread as a caller sees it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assignment<'a> {
    /// The thread identity.
    pub identity: UuidV4<'a>,
    /// The brief revision it was assigned against.
    pub brief: u64,
    /// The threads it depends on, in declaration order.
    pub dependencies: Vec<&'a str>,
    /// Its exclusive write claims.
    pub claims: &'a [Claim],
    /// Whether the parent join requires it.
    pub required: bool,
    /// What it reported, if anything.
    pub outcome: Option<Outcome>,
}

/// Why a join is blocked, with the threads responsible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blocked {
    /// One or more required threads did not meet their criterion.
    Unmet(Vec<String>),
    /// One or more threads dissented. Preserved with their reasons, never outvoted.
    Dissent(Vec<(String, String)>),
    /// One or more threads were assigned against a brief the cohort has since revised.
    Stale(Vec<String>),
    /// One or more required threads reported nothing.
    Missing(Vec<String>),
}

impl Blocked {
    /// The stable wire name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Unmet(_) => "unmet",
            Self::Dissent(_) => "dissent",
            Self::Stale(_) => "stale-brief",
            Self::Missing(_) => "missing-child",
        }
    }
}

/// The result of a parent join.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Join {
    /// Every required thread met its criterion on a current brief with disjoint writes.
    ///
    /// This is an **integration candidate**, not an acceptance: the parent still verifies.
    Integrable {
        /// The threads whose work is integrated, in assignment order.
        threads: Vec<String>,
    },
    /// The join cannot proceed, with every reason it cannot — not just the first.
    Blocked(Vec<Blocked>),
}

impl Join {
    /// Whether the parent may integrate.
    #[must_use]
    pub const fn is_integrable(&self) -> bool {
        matches!(self, Self::Integrable { .. })
    }

    /// Every reason the join is blocked, or an empty slice.
    #[must_use]
    pub fn reasons(&self) -> &[Blocked] {
        match self {
            Self::Integrable { .. } => &[],
            Self::Blocked(reasons) => reasons,
        }
    }
}

/// The single coordinator for one parent's specialist threads.
#[derive(Clone, Debug)]
pub struct Cohort {
    brief: u64,
    threads: Vec<Thread>,
}

impl Cohort {
    /// A cohort at brief revision `brief`.
    #[must_use]
    pub const fn new(brief: u64) -> Self {
        Self {
            brief,
            threads: Vec::new(),
        }
    }

    /// The current brief revision.
    #[must_use]
    pub const fn brief(&self) -> u64 {
        self.brief
    }

    /// The number of assigned threads.
    #[must_use]
    pub fn len(&self) -> usize {
        self.threads.len()
    }

    /// Whether nothing is assigned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.threads.is_empty()
    }

    /// Revise the brief.
    ///
    /// Threads assigned against the old revision become stale and block the join until they
    /// are reassigned. Nothing is silently migrated: a thread that did its work against an
    /// old brief did different work. Revising to the current value changes nothing.
    ///
    /// # Errors
    ///
    /// [`Refusal::BriefRegressed`] when `brief` is lower than the current revision; a
    /// regression would make work done against an older brief read as current (review D5).
    pub const fn revise(&mut self, brief: u64) -> Result<(), Refusal> {
        if brief < self.brief {
            return Err(Refusal::BriefRegressed);
        }
        self.brief = brief;
        Ok(())
    }

    /// Assign one bounded thread.
    ///
    /// # Errors
    ///
    /// * [`Refusal::ThreadLimit`], [`Refusal::ClaimLimit`], [`Refusal::DependencyLimit`],
    ///   each refused before the thread is built;
    /// * [`Refusal::MalformedIdentity`], [`Refusal::DuplicateThread`];
    /// * [`Refusal::SelfDependency`], [`Refusal::UnknownDependency`],
    ///   [`Refusal::DependencyCycle`] — a finite child DAG is required;
    /// * [`Refusal::OverlappingClaim`] when a claim intersects one a live thread holds.
    ///   Overlap is refused at assignment, not detected at join.
    pub fn assign(
        &mut self,
        identity: &str,
        dependencies: &[&str],
        claims: Vec<Claim>,
        required: bool,
    ) -> Result<(), Refusal> {
        if self.threads.len() >= MAX_THREADS {
            return Err(Refusal::ThreadLimit);
        }
        if claims.len() > MAX_CLAIMS {
            return Err(Refusal::ClaimLimit);
        }
        if dependencies.len() > MAX_DEPENDENCIES {
            return Err(Refusal::DependencyLimit);
        }
        let identity = UuidV4::parse(identity).map_err(Refusal::MalformedIdentity)?;
        if self.find(identity.as_str()).is_some() {
            return Err(Refusal::DuplicateThread);
        }
        let mut declared = Vec::with_capacity(dependencies.len());
        for dependency in dependencies {
            let dependency = UuidV4::parse(dependency).map_err(Refusal::MalformedIdentity)?;
            if dependency.as_str() == identity.as_str() {
                return Err(Refusal::SelfDependency);
            }
            if self.find(dependency.as_str()).is_none() {
                return Err(Refusal::UnknownDependency);
            }
            declared.push(dependency.as_str().to_owned());
        }
        for claim in &claims {
            for thread in &self.threads {
                if thread.claims.iter().any(|held| held.conflicts_with(claim)) {
                    return Err(Refusal::OverlappingClaim);
                }
            }
        }
        // Dependencies may only name threads already assigned, so the graph is acyclic by
        // construction. The check below is not redundant with that: it is what makes the
        // property hold if `assign` ever gains a way to name a later thread.
        if Self::closes_cycle(&self.threads, identity.as_str(), &declared) {
            return Err(Refusal::DependencyCycle);
        }
        self.threads.push(Thread {
            identity: identity.as_str().to_owned(),
            brief: self.brief,
            dependencies: declared,
            claims,
            required,
            outcome: None,
            evidence: None,
        });
        Ok(())
    }

    fn closes_cycle(threads: &[Thread], identity: &str, dependencies: &[String]) -> bool {
        let mut stack: Vec<&str> = dependencies.iter().map(String::as_str).collect();
        let mut seen: Vec<&str> = Vec::new();
        while let Some(current) = stack.pop() {
            if current == identity {
                return true;
            }
            if seen.contains(&current) {
                continue;
            }
            seen.push(current);
            if let Some(thread) = threads.iter().find(|t| t.identity == current) {
                stack.extend(thread.dependencies.iter().map(String::as_str));
            }
        }
        false
    }

    fn find(&self, identity: &str) -> Option<usize> {
        self.threads
            .iter()
            .position(|thread| thread.identity == identity)
    }
}

impl Cohort {
    /// Record one thread's own outcome.
    ///
    /// A child reports; it does not accept. There is no argument here by which a thread can
    /// speak for the parent or for a sibling.
    ///
    /// # Errors
    ///
    /// * [`Refusal::UnknownThread`], [`Refusal::MalformedIdentity`];
    /// * [`Refusal::AlreadyReported`] — an outcome is not revised in place, because a
    ///   silently rewritten conclusion is indistinguishable from the first one;
    /// * [`Refusal::StaleBrief`] when the cohort has been revised since the assignment.
    pub fn report(
        &mut self,
        thread: &str,
        outcome: Outcome,
        evidence: &str,
    ) -> Result<(), Refusal> {
        let thread = UuidV4::parse(thread).map_err(Refusal::MalformedIdentity)?;
        let index = self.find(thread.as_str()).ok_or(Refusal::UnknownThread)?;
        if self.threads[index].outcome.is_some() {
            return Err(Refusal::AlreadyReported);
        }
        if self.threads[index].brief < self.brief {
            return Err(Refusal::StaleBrief);
        }
        self.threads[index].outcome = Some(outcome);
        self.threads[index].evidence = Some(evidence.to_owned());
        Ok(())
    }

    /// Reassign a stale thread against the current brief, clearing its outcome.
    ///
    /// This is the repair path: a thread whose brief moved did different work, so its
    /// conclusion is discarded rather than carried forward.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownThread`], [`Refusal::MalformedIdentity`].
    pub fn rebrief(&mut self, thread: &str) -> Result<(), Refusal> {
        let thread = UuidV4::parse(thread).map_err(Refusal::MalformedIdentity)?;
        let index = self.find(thread.as_str()).ok_or(Refusal::UnknownThread)?;
        self.threads[index].brief = self.brief;
        self.threads[index].outcome = None;
        self.threads[index].evidence = None;
        Ok(())
    }

    /// One thread's assignment and state.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownThread`], [`Refusal::MalformedIdentity`].
    pub fn thread(&self, identity: &str) -> Result<Assignment<'_>, Refusal> {
        let index = self.find(identity).ok_or(Refusal::UnknownThread)?;
        let thread = &self.threads[index];
        Ok(Assignment {
            identity: UuidV4::parse(thread.identity.as_str())
                .map_err(Refusal::MalformedIdentity)?,
            brief: thread.brief,
            dependencies: thread.dependencies.iter().map(String::as_str).collect(),
            claims: &thread.claims,
            required: thread.required,
            outcome: thread.outcome,
        })
    }

    /// Every thread, in assignment order.
    ///
    /// # Errors
    ///
    /// [`Refusal::MalformedIdentity`] if a persisted identity is not a `UUIDv4`.
    pub fn threads(&self) -> Result<Vec<Assignment<'_>>, Refusal> {
        self.threads
            .iter()
            .map(|thread| self.thread(thread.identity.as_str()))
            .collect()
    }

    /// The recorded evidence for one thread, if it has reported.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownThread`], [`Refusal::MalformedIdentity`].
    pub fn evidence(&self, thread: &str) -> Result<Option<&str>, Refusal> {
        let thread = UuidV4::parse(thread).map_err(Refusal::MalformedIdentity)?;
        let index = self.find(thread.as_str()).ok_or(Refusal::UnknownThread)?;
        Ok(self.threads[index].evidence.as_deref())
    }

    /// Attempt the parent join.
    ///
    /// Returns [`Join::Integrable`] only when every **required** thread reported
    /// [`Outcome::Met`] on the current brief, and no thread at all dissented. An optional
    /// thread that is unmet does not block; a dissenting one does, required or not, because
    /// dissent is a claim about the work rather than about one thread's share of it.
    ///
    /// Every blocking reason is reported, not just the first: a caller that repairs one and
    /// re-joins should not discover the next one at a time.
    #[must_use]
    pub fn join(&self) -> Join {
        let mut missing = Vec::new();
        let mut unmet = Vec::new();
        let mut dissent = Vec::new();
        let mut stale = Vec::new();
        let mut integrated = Vec::new();

        for thread in &self.threads {
            if thread.brief < self.brief {
                stale.push(thread.identity.clone());
                continue;
            }
            match thread.outcome {
                None => {
                    if thread.required {
                        missing.push(thread.identity.clone());
                    }
                }
                Some(Outcome::Dissent) => dissent.push((
                    thread.identity.clone(),
                    thread.evidence.clone().unwrap_or_default(),
                )),
                Some(Outcome::Met) => integrated.push(thread.identity.clone()),
                Some(Outcome::Unmet | Outcome::Indeterminate) => {
                    if thread.required {
                        unmet.push(thread.identity.clone());
                    }
                }
            }
        }

        let mut reasons = Vec::new();
        if !missing.is_empty() {
            reasons.push(Blocked::Missing(missing));
        }
        if !unmet.is_empty() {
            reasons.push(Blocked::Unmet(unmet));
        }
        if !dissent.is_empty() {
            reasons.push(Blocked::Dissent(dissent));
        }
        if !stale.is_empty() {
            reasons.push(Blocked::Stale(stale));
        }
        if reasons.is_empty() {
            Join::Integrable {
                threads: integrated,
            }
        } else {
            Join::Blocked(reasons)
        }
    }

    /// Whether every live claim is disjoint from every other.
    ///
    /// **This cannot currently return `false`, and saying otherwise was a claim about a
    /// capability that does not exist.** [`Cohort::new`] is the only constructor, [`Thread`]
    /// is private, and the sole `threads.push` is inside [`Cohort::assign`], which refuses an
    /// overlapping claim. So every reachable cohort satisfies the predicate by construction.
    ///
    /// It is kept for the read-back path the module does not have yet: when a cohort can be
    /// restored from storage, this is what checks it before it is trusted, and *then* the
    /// `false` arm becomes reachable. Until that constructor exists the mutants of this
    /// function are equivalent, declared in `evidence/mutation-equivalences.json` — a trigger
    /// that can never fire is worse than a known gap, because it reads as future work.
    #[must_use]
    pub fn claims_are_disjoint(&self) -> bool {
        for (index, thread) in self.threads.iter().enumerate() {
            for other in &self.threads[index + 1..] {
                for claim in &thread.claims {
                    if other.claims.iter().any(|held| held.conflicts_with(claim)) {
                        return false;
                    }
                }
            }
        }
        true
    }
}
