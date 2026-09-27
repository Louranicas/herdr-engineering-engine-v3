#![forbid(unsafe_code)]
// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/main.rs
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
// Readiness binding: HEE3-READINESS-001; SHA-256 f574043487f39db6424c4988bce58e88a6e766f02f974fbcf3fa8dd6e0003548; clauses F1-C01, F1-C02, F1-C03, F1-C04, F1-C05, F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-01, R90-02, R90-03, R90-04, R90-06, R90-08, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-app; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-app (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-app
// Owns: CLI/serve composition and dependency injection; no duplicate task state
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/app.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app)
// Build dependencies: contracts, task, store, roster, route, budget, worker, check, recovery, cohort, context, notify, service, herdr, numerical, actions
// Consumers: none declared
// Related task contracts: T01, T06, T14, T15, T16, T17, T18, T19, T20, T25, T26, T27, T28
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K6](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K6)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [contributing codebase CODE-CB10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB10)
// [contributing codebase CODE-CB11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB11)
// [task TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
// [task TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)
// [task TASK-T16](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T16)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [task TASK-T28](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T28)
// [separate reference example EX-app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-app)
// [handbook HB-compatibility-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-compatibility-map)
// [handbook HB-module-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-module-map)
// [handbook HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
// [API API-API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01)
// [API API-API02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API02)
// [action ACT-health](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-health)
// [IPC IPC-IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-app)
// [plan SEC-architecture](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-architecture)
// [plan SEC-deployment](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-deployment)
// [plan SEC-hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-hardening)
// [plan SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
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
// [schematic SC-SC13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC13)
// [schematic SC-SC15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC15)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
// [schematic SC-SC24](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC24)
// [source SRC-A02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A02)
// [source SRC-A15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A15)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-app)
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
// [readiness criterion cluster F6](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-01)
// [readiness improvement grouping R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02)
// [readiness improvement grouping R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
// [readiness improvement grouping R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04)
// [readiness improvement grouping R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06)
// [readiness improvement grouping R90-08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-08)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-app)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [documentation procedure RB01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB01)
// [documentation procedure RB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB02)
// [applied learning LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
// [applied learning LRN02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN02)
// [applied learning LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03)
// [applied learning LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
// [applied learning LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
// [applied learning LRN06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06)
// [applied learning LRN07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN10)
// [applied learning LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11)
// [applied learning LRN12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN12)
// [applied learning LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13)
// [applied learning LRN14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14)
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
// [diary evidence source DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
// [diary evidence source DR12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR12)
// [diary evidence source DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
// Applicable learning IDs: LRN01, LRN02, LRN03, LRN04, LRN05, LRN06, LRN07, LRN08, LRN10, LRN11, LRN12, LRN13, LRN14; guidance only, engine detectors unqualified.
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
// [Why I Stopped Trusting Green](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhy%20I%20Stopped%20Trusting%20Green)
// [Working Style in This Habitat](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20Style%20in%20This%20Habitat)
// [Working in Sandboxes on Kinoite](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20in%20Sandboxes%20on%20Kinoite)
// HEE3-ANCHORS-END

use habitat_engine::actions::Catalogue;
use habitat_engine::actions::control::{Grants, NoGrants};
use habitat_engine::app::control_socket::{
    self, Drain, IDLE_TIMEOUT, RUNTIME_DIRECTORY, SOCKET_NAME, WRITE_TIMEOUT,
};
use habitat_engine::app::coordinator;
use habitat_engine::app::dispatcher::NoNative;
use habitat_engine::app::grants::{self, FileGrants};
use habitat_engine::app::native_provider::{self, Installed, NativeFileError, NativeProvider};
use habitat_engine::app::tasks::StoreTasks;
use habitat_engine::app::{class_profile, dispatcher, routing};
use habitat_engine::contracts::control::{FrameReader, MAX_FRAME_BYTES, ReadError};
use habitat_engine::worker::aggregate;
use habitat_engine::worker::namespace_shim::{self, NamespaceExec};
use habitat_engine::worker::native::{self, Systemd};
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

// The bash wrapper's producer contract (integrations/bash/hee3): the same numbers mean the same
// failures on both sides of the pipe.
const EXIT_USAGE: u8 = 2;
const EXIT_NO_ENGINE: u8 = 3;
const EXIT_BOUNDS: u8 = 4;
const EXIT_TIMEOUT: u8 = 5;
const EXIT_CONTRACT: u8 = 6;
/// The engine answered with a typed error record (BASH-G1). The record is printed unchanged; the
/// code alone tells a refusal from a result, so no caller has to parse stdout to know which.
const EXIT_REFUSED: u8 = 7;

/// Where the reviewed grant records live, under the operator's configuration root (RC02).
const GRANTS_DIRECTORY: &str = "grants";

/// `serve`'s one flag: drain when standard input, a pipe whose write end the parent holds, closes.
const UNTIL_STDIN_CLOSES: &str = "--until-stdin-closes";

/// What ends `serve`.
#[derive(Clone, Copy)]
enum Lifetime {
    /// The drain signals (SIGTERM, SIGHUP, SIGINT) alone: the default; a systemd unit's stdin is
    /// `/dev/null` and its cgroup owns it.
    Signalled,
    /// The drain signals, or the end of standard input, which the kernel delivers on any death of
    /// the parent holding the pipe's write end -- an unwound exit, an abort, SIGKILL. Like a signal,
    /// it drains between attempts: an in-flight exchange or check is waited for under the attempt's
    /// own deadline, never cut short (`runtime::Dispatch::drain`, R21 D8).
    UntilStdinCloses,
}

fn main() -> ExitCode {
    let Ok(args) = arguments() else {
        eprintln!("habitat-engine: invalid or overbound arguments");
        return ExitCode::from(EXIT_USAGE);
    };
    if args.len() >= 4 && args[0] == "__namespace-exec" && args[2] == "--" {
        let arguments: Vec<&str> = args[4..].iter().map(String::as_str).collect();
        return namespace_shim::run(&NamespaceExec {
            executable: &args[3],
            arguments: &arguments,
            working_directory: &args[1],
        });
    }
    match args.as_slice() {
        [command] if command == "serve" => serve(Lifetime::Signalled),
        [command, flag] if command == "serve" && flag == UNTIL_STDIN_CLOSES => {
            serve(Lifetime::UntilStdinCloses)
        }
        [command, seconds] if command == "commission" => commission_verb(seconds),
        [action] if Catalogue::find(action).is_ok() => request(action),
        _ => usage(),
    }
}

/// The usage line, said once for every argv no verb takes.
fn usage() -> ExitCode {
    eprintln!(
        "habitat-engine: usage: habitat-engine serve [{UNTIL_STDIN_CLOSES}] | \
         habitat-engine <action-id> < request | habitat-engine commission <deadline-seconds>"
    );
    ExitCode::from(EXIT_USAGE)
}

/// `habitat-engine commission <deadline-seconds>` (OPS-1; RC02; HO-03; R22-4): the operator's act
/// that creates the state root under `HOME`, its active-generation manifest and the first ledger,
/// through the one library door (`coordinator::commission`). IPC01 custody is taken first and held
/// until the verb exits, so no engine serves while a root is commissioned. The deadline is the
/// operator's, with no default: a missing, zero or unparsable one is a usage error. On success the
/// one line on standard output carries only values read back from the placed root.
fn commission_verb(seconds: &str) -> ExitCode {
    let refused = |code: u8, why: &str| {
        eprintln!("habitat-engine: commission refused: {why}");
        ExitCode::from(code)
    };
    let Some(deadline) = habitat_engine::contracts::parse_u64_decimal(seconds)
        .ok()
        .filter(|seconds| *seconds > 0)
        .and_then(|seconds| {
            std::time::Instant::now().checked_add(std::time::Duration::from_secs(seconds))
        })
    else {
        return usage();
    };
    let prepared =
        match control_socket::runtime_root(std::env::var_os("XDG_RUNTIME_DIR").as_deref())
            .and_then(|root| control_socket::prepare(&root))
        {
            Ok(prepared) => prepared,
            Err(control_socket::Error::Live) => {
                return refused(EXIT_CONTRACT, "an engine holds custody (Live)");
            }
            Err(error) => return refused(EXIT_CONTRACT, &format!("custody ({error:?})")),
        };
    let Some(home) = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
    else {
        return refused(EXIT_USAGE, "HOME is unset or relative");
    };
    let state_root = coordinator::state_root(&home);
    let ids = habitat_engine::app::evidence::fresh_id(deadline).and_then(|generation| {
        habitat_engine::app::evidence::fresh_id(deadline).map(|epoch| (generation, epoch))
    });
    let Ok((generation, epoch)) = ids else {
        return if std::time::Instant::now() >= deadline {
            refused(EXIT_TIMEOUT, "deadline")
        } else {
            refused(EXIT_CONTRACT, "entropy")
        };
    };
    let (Ok(generation), Ok(epoch)) = (
        habitat_engine::contracts::UuidV4::parse(generation.as_str()),
        habitat_engine::contracts::UuidV4::parse(epoch.as_str()),
    ) else {
        return refused(EXIT_CONTRACT, "a drawn id is not a UuidV4");
    };
    let active = coordinator::Active { generation, epoch };
    let commissioned = coordinator::commission(&state_root, active, deadline);
    drop(prepared);
    match commissioned {
        Ok(commissioned) => {
            if let Err(error) = writeln!(io::stdout().lock(), "{}", commissioned.line()) {
                eprintln!(
                    "habitat-engine: commissioned, but the line could not be written: {error}"
                );
                return ExitCode::from(EXIT_CONTRACT);
            }
            ExitCode::SUCCESS
        }
        Err(coordinator::CommissionError::Deadline) => refused(EXIT_TIMEOUT, "deadline"),
        Err(coordinator::CommissionError::Exists(path)) => refused(
            EXIT_CONTRACT,
            &format!("state root exists at {}", path.display()),
        ),
        Err(coordinator::CommissionError::NotCanonical(path)) => refused(
            EXIT_CONTRACT,
            &format!(
                "state root is not its own canonical path ({})",
                path.display()
            ),
        ),
        Err(coordinator::CommissionError::Store(why)) => {
            refused(EXIT_CONTRACT, &format!("store refused ({why})"))
        }
        Err(coordinator::CommissionError::Io(kind)) => {
            refused(EXIT_CONTRACT, &format!("io ({kind:?})"))
        }
        Err(coordinator::CommissionError::ReadBack(which)) => {
            refused(EXIT_CONTRACT, &format!("read-back differs ({which})"))
        }
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

fn socket_path() -> Result<PathBuf, control_socket::Error> {
    let root = control_socket::runtime_root(std::env::var_os("XDG_RUNTIME_DIR").as_deref())?;
    Ok(root.join(RUNTIME_DIRECTORY).join(SOCKET_NAME))
}

/// Read the class profile (B14-P2b), say it in one line — declaration only, no capture before the
/// socket binds — and return it: admission screens a task's workspace against it (B14-P2c).
fn say_class_profile(home: &Path) -> Result<class_profile::Profile, class_profile::Unready> {
    let class = coordinator::config_path(home, class_profile::CLASS_DIRECTORY);
    let read = class_profile::read(&class);
    match &read {
        Ok(profile) => say(format_args!(
            "class profile read from {} ({} workspaces declared)",
            class.display(),
            profile.declared.workspaces.len()
        )),
        Err(why) => say(format_args!(
            "dispatch unavailable: {} ({})",
            why.constraint(),
            class.display()
        )),
    }
    read
}

/// The grant store under `home`: the file grants when the directory exists, `NoGrants` (every
/// request refused forbidden, said once) when it does not, and the exit code when it is refused.
fn open_grants(home: &Path) -> Result<Box<dyn Grants + Sync>, ExitCode> {
    let directory = coordinator::config_path(home, GRANTS_DIRECTORY);
    match FileGrants::open(&directory) {
        Ok(store) => Ok(Box::new(store)),
        Err(grants::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
            say(format_args!(
                "no grant directory at {}; every request is refused forbidden",
                directory.display()
            ));
            Ok(Box::new(NoGrants))
        }
        Err(error) => {
            say(format_args!("grant directory refused: {error:?}"));
            Err(ExitCode::from(EXIT_CONTRACT))
        }
    }
}

/// The attempts root under `state_root`, created 0700 and marked when absent and read back as this
/// user's private directory through its one door (`coordinator::prepare_attempts_root`, B14b-2
/// closure C18): the shared plan and every attempt are materialised under it, and the plan refuses
/// a root whose parent does not exist (`plan_root_not_canonical`) — which would stop the owner's
/// task for the machine's missing directory. The door refuses the rest of that rule here, by name
/// (`NotCanonical`: a root reached through a link, B14b-2 closure D5), for the same reason. A root
/// that exists without its marker is refused, never re-created. When it cannot be had, dispatch is
/// said unavailable once and no dispatcher runs. `deadline` is the startup's own, passed through:
/// it bounds only drawing a new root's id. The id the door returns is kept with the root and
/// carried into every dispatch, where a begin compares the marker it reads against it (B14b-2
/// review round 2, D9) — never discarded and re-acquired as if it were new.
fn attempts_root(state_root: &Path, deadline: std::time::Instant) -> Option<AttemptsRoot> {
    let attempts = coordinator::attempts_root(state_root);
    match coordinator::prepare_attempts_root(&attempts, deadline) {
        Ok(id) => Some(AttemptsRoot { path: attempts, id }),
        Err(why) => {
            say(format_args!(
                "dispatch unavailable: attempts root {} refused ({why:?})",
                attempts.display()
            ));
            None
        }
    }
}

/// The attempts root `serve` prepared at its start, and the id its one door returned for it
/// (B14b-2 review round 2, D9): both are handed to the dispatcher, which carries the id into every
/// dispatch.
struct AttemptsRoot {
    path: PathBuf,
    id: String,
}

/// The native provider (B14b-2; R21 N3, N12): the operator's file read under custody, the class's
/// adapter row, the agent record installed as the operator inside the startup window, and the
/// provider over the user manager's pinned busctl door and the engine's runtime root. Every absence
/// or refusal is said once; the dispatcher then runs over `NoProvider`, holding the refusal's kind
/// so its own named state says it again (R21 round-1 LOW F8). Composed only once the attempts root
/// is prepared (`prepare_dispatch`, FT-5): the install writes the agent record to the ledger.
fn compose_native(
    home: &Path,
    tasks: &StoreTasks,
    runtime_root: &Path,
    deadline: std::time::Instant,
) -> Composed {
    let directory = coordinator::config_path(home, native_provider::NATIVE_DIRECTORY);
    let unavailable = |why: &str| {
        say(format_args!(
            "native provider unavailable: {why} ({})",
            directory.display()
        ));
    };
    let (file, bytes) = match native_provider::read(&directory) {
        Ok(read) => read,
        Err(NativeFileError::NotInstalled) => {
            unavailable("not installed");
            return Err(NoNative::NotInstalled);
        }
        Err(error) => {
            unavailable(&format!("refused: {error:?}"));
            return Err(native_provider::no_native(&error));
        }
    };
    // No class profile: the dispatcher says so itself (`unavailable: no class profile`).
    let profile = tasks.class_profile().map_err(|_| NoNative::ClassProfile)?;
    // A class with no native row installs nothing: `open` names it for every task it would serve.
    let adapter = match &profile.declared.native {
        None => None,
        Some(row) => {
            let Some(adapter) = native::adapter(&row.adapter) else {
                unavailable(&format!(
                    "the class's adapter row {} is unknown",
                    row.adapter
                ));
                return Err(NoNative::Adapter);
            };
            Some(adapter)
        }
    };
    let installed = match adapter {
        None => {
            say(format_args!(
                "native provider read from {}; the class declares no native model, \
                 so nothing is installed",
                directory.display()
            ));
            Installed {
                record_id: String::new(),
                selections: Vec::new(),
            }
        }
        Some(adapter) => {
            let euid = rustix::process::geteuid().as_raw();
            let principal = match control_socket::admit_peer(euid, euid) {
                Ok(principal) => principal,
                Err(why) => {
                    unavailable(&why);
                    return Err(NoNative::Principal);
                }
            };
            match native_provider::install(tasks, &principal, &file, &bytes, adapter, deadline) {
                Ok(installed) => {
                    say(format_args!(
                        "native provider installed from {} (record {}, revision {})",
                        directory.display(),
                        installed.record_id,
                        installed
                            .selections
                            .first()
                            .map_or("none", |selection| selection.expected_revision.as_str())
                    ));
                    installed
                }
                Err(error) => {
                    unavailable(&format!("install refused: {error:?}"));
                    return Err(NoNative::Install);
                }
            }
        }
    };
    let systemd = Systemd(aggregate::Config {
        busctl: aggregate::BUSCTL.into(),
        busctl_sha256: profile.declared.busctl_sha256.clone(),
        runtime_dir: runtime_root.to_path_buf(),
    });
    Ok((
        NativeProvider::new(file, systemd, runtime_root.to_path_buf()),
        installed,
    ))
}

/// What `compose_native` came to: the provider and its install, or the refusal's kind.
type Composed = Result<(NativeProvider<Systemd>, Installed), NoNative>;

/// What the dispatcher runs over (B14b-2 review round 2, FT-5): the task owner, the attempts root
/// `serve` prepared and its id, and the native provider composed after both.
struct Dispatching<'a> {
    tasks: &'a StoreTasks,
    root: AttemptsRoot,
    native: Composed,
}

/// Why no dispatcher runs, by name (B14b-2 review round 2, FT-5): an engine with a task owner whose
/// attempts root was refused is never said to have no task owner.
enum Undispatched {
    /// No task owner was composed; said by the serve scope.
    NoTaskOwner,
    /// The attempts root was refused; said once by `attempts_root`, at startup.
    AttemptsRoot,
}

/// The dispatcher's inputs, prepared at start in the order each needs the last (B14b-2 review round
/// 2, FT-5): the task owner, then the attempts root through its one door, and only then the native
/// provider — whose install writes the agent record to the ledger, so a refused root installs
/// nothing.
fn prepare_dispatch<'a>(
    home: &Path,
    state_root: &Path,
    tasks: Option<&'a StoreTasks>,
    runtime_root: &Path,
    deadline: std::time::Instant,
) -> Result<Dispatching<'a>, Undispatched> {
    let tasks = tasks.ok_or(Undispatched::NoTaskOwner)?;
    let root = attempts_root(state_root, deadline).ok_or(Undispatched::AttemptsRoot)?;
    let native = compose_native(home, tasks, runtime_root, deadline);
    Ok(Dispatching {
        tasks,
        root,
        native,
    })
}

/// `habitat-engine serve`: take single-instance custody of IPC01, reconcile the active
/// generation, compose the task owner over the ledger startup left open, and only then bind and
/// serve until SIGTERM (IPC01: acquire custody before recovery; bind after ready). SIGTERM drains
/// (APP-01): nothing more is admitted, each open connection finishes the frame it is serving, the
/// ledger's writer lock is released, the socket is removed and the engine exits 0. SIGHUP (a closed
/// terminal tab or pane) and SIGINT (Ctrl-C) drain the same way, through the same door
/// (`drain_signals`). Only the first drain signal is acted on: once the drain has begun, a further
/// Ctrl-C or SIGHUP is held and ignored, so an operator cannot hard-stop a draining engine that way
/// (APP-01, recorded by B14b-2 review round 2, D12). A stale socket left by a killed engine is
/// cleared at the next start; a live or starting one refuses the start. With
/// `--until-stdin-closes` the end of standard input raises that same SIGTERM (`watch_stdin`).
fn serve(lifetime: Lifetime) -> ExitCode {
    let mut signals = match drain_signals(lifetime) {
        Ok(signals) => signals,
        Err(code) => return code,
    };
    // The runtime root is kept: the native provider's aggregate and scopes are pinned to it.
    let (runtime_root, prepared) =
        match control_socket::runtime_root(std::env::var_os("XDG_RUNTIME_DIR").as_deref())
            .and_then(|root| control_socket::prepare(&root).map(|prepared| (root, prepared)))
        {
            Ok(both) => both,
            Err(error) => {
                say(format_args!("control socket refused: {error:?}"));
                return ExitCode::from(EXIT_CONTRACT);
            }
        };
    let Some(home) = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
    else {
        say(format_args!("HOME is unset or relative"));
        return ExitCode::from(EXIT_USAGE);
    };
    let store = match open_grants(&home) {
        Ok(store) => store,
        Err(code) => return code,
    };
    // Reconcile the active generation once, before the socket exists: health is what startup
    // left, observed at a named instant (D-C3 step 2). The manifest is read once; the generation
    // served is the one reconciled, over the ledger startup opened.
    let state_root = coordinator::state_root(&home);
    let manifest = coordinator::read_manifest(&state_root);
    // The one startup window: reconciliation and the native install both run inside it.
    let startup_deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let started =
        coordinator::observe_at_start(&state_root, &manifest, now_unix_ms(), startup_deadline);
    say(format_args!("{}", started.line));
    let health = started.health;
    // The route configuration task.preview screens against, read once under custody (B07). Its
    // outcome is said after the task owner's, so a refused ledger's reason stays in the first lines.
    let routes = coordinator::config_path(&home, routing::ROUTING_DIRECTORY);
    let routing = routing::read(&routes);
    let tasks = match started.reconciled.map(coordinator::compose_tasks) {
        Some(Ok(tasks)) => {
            match &routing {
                Ok(_) => say(format_args!(
                    "route configuration read from {}",
                    routes.display()
                )),
                Err(why) => say(format_args!(
                    "task.preview unavailable: {} ({})",
                    why.constraint(),
                    routes.display()
                )),
            }
            let profile = say_class_profile(&home);
            Some(tasks.with_routing(routing).with_class_profile(profile))
        }
        Some(Err(why)) => {
            say(format_args!("task actions unavailable: {why}"));
            None
        }
        None => {
            say(format_args!(
                "task actions unavailable: no generation was reconciled"
            ));
            None
        }
    };
    // What the dispatcher runs over, in order: the task owner, the attempts root, then the native
    // provider — or the one reason none runs (B14b-2 review round 2, FT-5).
    let dispatching = prepare_dispatch(
        &home,
        &state_root,
        tasks.as_ref(),
        &runtime_root,
        startup_deadline,
    );
    let listener = match control_socket::bind(&prepared) {
        Ok(listener) => listener,
        Err(error) => {
            say(format_args!("control socket refused: {error:?}"));
            return ExitCode::from(EXIT_CONTRACT);
        }
    };
    say(format_args!("serving {}", prepared.socket().display()));
    let drain = Drain::default();
    let shared = control_socket::Shared {
        grants: store.as_ref(),
        health: Some(&health),
        tasks: tasks
            .as_ref()
            .map(|tasks| tasks as &(dyn habitat_engine::actions::control::Tasks + Sync)),
        drain: &drain,
    };
    // The dispatcher (B14b-1): one thread beside the accept loop, over the same task owner and the
    // native provider when one composed (B14b-2) — else it runs the free checks, stops by name, and
    // reports its named unavailable state when a task passes them.
    if let Err(error) = serve_until_signalled(
        &mut signals,
        &listener,
        shared,
        &drain,
        &prepared,
        dispatching,
    ) {
        say(format_args!("accept failed: {error}"));
        return ExitCode::from(EXIT_CONTRACT);
    }
    // Drained: the task owner goes first, releasing the ledger's writer lock; then the socket,
    // removed while custody is still held; then custody itself.
    drop(tasks);
    drop(listener);
    finish_drained(prepared)
}

/// The one drain door, taken before anything else (APP-01): SIGTERM, SIGHUP and SIGINT are held
/// from here, in one set, so one that arrives during startup drains the engine once it serves
/// rather than killing it mid-reconciliation. SIGHUP is what closing a Ghostty tab or a herdr pane
/// sends and SIGINT is Ctrl-C (Kinoite vault, measured 2026-09-27); without them a pane-started
/// engine died undrained. Under `--until-stdin-closes` the parent-death watcher starts right
/// behind it, raising through it.
fn drain_signals(lifetime: Lifetime) -> Result<Signals, ExitCode> {
    let signals = Signals::new([SIGTERM, SIGHUP, SIGINT]).map_err(|error| {
        say(format_args!(
            "SIGTERM, SIGHUP and SIGINT could not be taken ({error})"
        ));
        ExitCode::from(EXIT_CONTRACT)
    })?;
    if let Lifetime::UntilStdinCloses = lifetime {
        watch_stdin()?;
    }
    Ok(signals)
}

/// `serve --until-stdin-closes`: standard input must be a pipe, whose write end the parent holds. A
/// detached thread reads it (discarding what it reads) to end of file -- which the kernel delivers
/// on any death of the parent, unwound or not -- and then raises SIGTERM, so the parent's death
/// drains the engine through the one door APP-01 already holds, not a second path: during startup
/// the signal is held, once serving it drains. Detached, so the serve scope never joins a thread
/// blocked in `read`. Any other standard input is refused by name with the contract's code: it
/// could never report the parent's death (`/dev/null` reads end of file at once). Tier-3 rows (each
/// reachable only by arranging the world, never by an argument): a standard input `fstat` refuses,
/// a read that fails other than by interruption, a watcher thread that cannot be spawned, and a
/// SIGTERM that cannot be raised (B14b-2 review round 2, D12).
fn watch_stdin() -> Result<(), ExitCode> {
    let refused = |why: String| {
        say(format_args!("{UNTIL_STDIN_CLOSES} refused: {why}"));
        ExitCode::from(EXIT_CONTRACT)
    };
    match rustix::fs::fstat(io::stdin()) {
        Ok(stat) => match rustix::fs::FileType::from_raw_mode(stat.st_mode) {
            rustix::fs::FileType::Fifo => {}
            other => return Err(refused(format!("standard input is not a pipe ({other:?})"))),
        },
        Err(error) => return Err(refused(format!("standard input unreadable ({error})"))),
    }
    let watcher = std::thread::Builder::new()
        .name("stdin-watch".into())
        .spawn(|| {
            let mut sink = [0_u8; 512];
            let ended = loop {
                match io::stdin().read(&mut sink) {
                    Ok(0) => break "closed".to_owned(),
                    Ok(_) => {}
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    // Unreadable is as blind as closed: the parent can no longer be observed.
                    Err(error) => break format!("unreadable ({error})"),
                }
            };
            // Through `say`, never `eprintln!`: a parent that dies may take standard error's
            // reader with it, and a panic here would end this thread before the drain is raised.
            say(format_args!("standard input {ended}; raising SIGTERM"));
            if let Err(error) = signal_hook::low_level::raise(SIGTERM) {
                say(format_args!("SIGTERM could not be raised ({error})"));
            }
        });
    match watcher {
        Ok(_detached) => Ok(()),
        Err(error) => Err(refused(format!("its watcher could not start ({error})"))),
    }
}

/// Serve until the first drain signal has drained every connection (APP-01). A watcher thread
/// waits on `signals` and begins the drain, naming the signal it received; the [`Release`] built
/// before anything else in the scope ends that wait unsignalled and releases the dispatcher once
/// `run` ends -- returned or unwound -- so the scope never joins a thread still waiting.
fn serve_until_signalled(
    signals: &mut Signals,
    listener: &std::os::unix::net::UnixListener,
    shared: control_socket::Shared<'_>,
    drain: &Drain,
    prepared: &control_socket::Prepared,
    dispatching: Result<Dispatching<'_>, Undispatched>,
) -> io::Result<()> {
    let tasks = dispatching
        .as_ref()
        .ok()
        .map(|dispatching| dispatching.tasks);
    let report = |line: &str| say(format_args!("{line}"));
    let handle = signals.handle();
    std::thread::scope(|scope| {
        // Whatever ends the accept loop -- its return or a panic unwinding through it -- the
        // dispatcher is released and the watcher's wait ended before the scope joins them
        // (closure H4; B14b-2 review round 2, D1).
        let release = Release {
            drain,
            tasks,
            handle,
        };
        scope.spawn(|| {
            if let Some(signal) = signals.forever().next() {
                // Through `say`, never `eprintln!`: a closed pane or a dead parent may have taken
                // standard error's reader, and a panic here would leave the drain unbegun.
                let name = signal_hook::low_level::signal_name(signal).unwrap_or("signal");
                say(format_args!("{name}: draining"));
                if let Err(error) = drain.begin(prepared.socket()) {
                    say(format_args!("drain wake-up failed ({error})"));
                }
                // The drain reaches the dispatcher's wait through the owner of both (B14b-1, D2),
                // under the store's guard so no wait window can swallow it (closure H5).
                if let Some(tasks) = tasks {
                    tasks.wake();
                }
            }
        });
        // The dispatcher runs over the task owner (the one class profile is read inside it);
        // without one, its absence is said once, like the other unavailable doors.
        match dispatching {
            Ok(Dispatching {
                tasks,
                root,
                native,
            }) => {
                scope.spawn(move || {
                    let exit = match native {
                        Ok((mut provider, installed)) => dispatcher::Dispatcher {
                            tasks,
                            attempts: &root.path,
                            root_id: &root.id,
                            provider: &mut provider,
                            agent_record_id: &installed.record_id,
                            selections: &installed.selections,
                            drain: drain.flag(),
                        }
                        .run(&report),
                        Err(why) => dispatcher::Dispatcher {
                            tasks,
                            attempts: &root.path,
                            root_id: &root.id,
                            provider: &mut dispatcher::NoProvider(why),
                            agent_record_id: "",
                            selections: &[],
                            drain: drain.flag(),
                        }
                        .run(&report),
                    };
                    say(format_args!("dispatcher stopped: {exit:?}"));
                });
            }
            Err(Undispatched::NoTaskOwner) => {
                say(format_args!("dispatch unavailable: no task owner"));
            }
            // Said once at startup, where the door refused it, and no native provider was composed.
            Err(Undispatched::AttemptsRoot) => {}
        }
        let served = control_socket::run(listener, shared, &now_unix_ms, &report);
        drop(release);
        served
    })
}

/// What `serve_until_signalled` releases when its accept loop ends, on its drop, so an unwinding
/// loop releases it too: the drain marked, the dispatcher's wait woken under the store's guard, and
/// the signal watcher's wait closed. Without it a panic in the loop left the scope joining a
/// dispatcher that was never released and a watcher still waiting for a signal.
struct Release<'a> {
    drain: &'a Drain,
    tasks: Option<&'a StoreTasks>,
    handle: signal_hook::iterator::Handle,
}

impl Drop for Release<'_> {
    fn drop(&mut self) {
        self.drain.mark();
        if let Some(tasks) = self.tasks {
            tasks.wake();
        }
        self.handle.close();
    }
}

/// Remove the socket and give up custody, then say so: the last step of a drain.
fn finish_drained(prepared: control_socket::Prepared) -> ExitCode {
    let socket = prepared.socket().to_path_buf();
    if let Err(error) = prepared.finish() {
        say(format_args!(
            "drained, but the socket was not removed: {error:?}"
        ));
        return ExitCode::from(EXIT_CONTRACT);
    }
    say(format_args!(
        "drained; removed {}; exiting",
        socket.display()
    ));
    ExitCode::SUCCESS
}

/// `serve`'s one standard-error door: `line` after the engine's name, the write's error discarded.
/// A closed pane or a dead parent can take standard error's reader with it, and `eprintln!` panics
/// on that error -- a drain that exited 101 and left its socket, or a thread that never released
/// the drain (B14b-2 review round 2, D1). What cannot be said is not a reason to stop serving.
fn say(line: std::fmt::Arguments<'_>) {
    let _ = writeln!(io::stderr().lock(), "habitat-engine: {line}");
}

/// `habitat-engine <action-id> < request`: the wrapper's producer. Sends the request's exact
/// bytes as one frame and prints the one record the engine answers with.
fn request(action: &str) -> ExitCode {
    let mut payload = Vec::new();
    let limit = u64::try_from(MAX_FRAME_BYTES).unwrap_or(u64::MAX) + 2;
    if io::stdin().take(limit).read_to_end(&mut payload).is_err() {
        eprintln!("habitat-engine: stdin unreadable");
        return ExitCode::from(EXIT_BOUNDS);
    }
    if payload.last() == Some(&b'\n') {
        payload.pop();
    }
    if payload.len() > MAX_FRAME_BYTES || payload.contains(&b'\n') {
        eprintln!("habitat-engine: the request is not one frame of at most 1048576 bytes");
        return ExitCode::from(EXIT_BOUNDS);
    }
    let named = serde_json::from_slice::<serde_json::Value>(&payload)
        .ok()
        .and_then(|value| {
            value
                .get("action")
                .and_then(|name| name.as_str().map(str::to_owned))
        });
    if named.as_deref() != Some(action) {
        eprintln!("habitat-engine: the request does not name action {action}");
        return ExitCode::from(EXIT_USAGE);
    }
    let stream = match socket_path()
        .map_err(|error| format!("{error:?}"))
        .and_then(|path| {
            UnixStream::connect(&path).map_err(|error| format!("{}: {error}", path.display()))
        }) {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!("habitat-engine: no engine is serving: {error}");
            return ExitCode::from(EXIT_NO_ENGINE);
        }
    };
    payload.push(b'\n');
    let sent = stream
        .set_write_timeout(Some(WRITE_TIMEOUT))
        .and_then(|()| stream.set_read_timeout(Some(IDLE_TIMEOUT)))
        .and_then(|()| (&stream).write_all(&payload))
        .and_then(|()| stream.shutdown(std::net::Shutdown::Write));
    if let Err(error) = sent {
        eprintln!("habitat-engine: the request could not be sent: {error}");
        return ExitCode::from(timeout_or(&error, EXIT_CONTRACT));
    }
    match FrameReader::new(&stream).next_frame() {
        Ok(Some(mut record)) => {
            let refusal = serde_json::from_slice::<serde_json::Value>(&record)
                .ok()
                .filter(|reply| {
                    reply.get("kind").and_then(serde_json::Value::as_str) == Some("error")
                })
                .map(|reply| {
                    reply
                        .get("code")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("without a code")
                        .to_owned()
                });
            record.push(b'\n');
            if let Err(error) = io::stdout().write_all(&record) {
                eprintln!("habitat-engine: the reply could not be written: {error}");
                return ExitCode::from(EXIT_CONTRACT);
            }
            if let Some(code) = refusal {
                eprintln!("habitat-engine: the engine refused the request: {code}");
                return ExitCode::from(EXIT_REFUSED);
            }
            ExitCode::SUCCESS
        }
        Ok(None) => {
            eprintln!("habitat-engine: the engine closed without a reply (the frame was refused)");
            ExitCode::from(EXIT_CONTRACT)
        }
        Err(ReadError::Fault(fault)) => {
            eprintln!("habitat-engine: the reply broke framing: {}", fault.name());
            ExitCode::from(EXIT_CONTRACT)
        }
        Err(ReadError::Io(error)) => {
            eprintln!("habitat-engine: the reply could not be read: {error}");
            ExitCode::from(timeout_or(&error, EXIT_CONTRACT))
        }
    }
}

fn timeout_or(error: &io::Error, otherwise: u8) -> u8 {
    match error.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => EXIT_TIMEOUT,
        _ => otherwise,
    }
}

fn arguments() -> Result<Vec<String>, ()> {
    let mut values = Vec::new();
    let mut bytes = 0_usize;
    for argument in std::env::args_os().skip(1).take(257) {
        let argument = argument.into_string().map_err(|_| ())?;
        bytes = bytes.checked_add(argument.len()).ok_or(())?;
        if values.len() >= 256 || argument.len() > 4096 || bytes > 64 * 1024 {
            return Err(());
        }
        values.push(argument);
    }
    Ok(values)
}
