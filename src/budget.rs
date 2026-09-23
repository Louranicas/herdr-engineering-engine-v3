// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/budget.rs
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
// Readiness binding: HEE3-READINESS-001; SHA-256 e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-02, R90-03, R90-04, R90-05, R90-09, R90-10; resolved contracts RC02, RC03, RC04, RC05; runtime proof pending; original task DAG controls.
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

//! Whole-task cost accounting: reservations, usage settlement and bounded fallback.
//!
//! One owner for every counter. `docs/modules/budget.md` sets the complexity boundary
//! explicitly — *"Do not create independent budget counters in each wrapper or omit
//! checker/context cost"* — so [`Ledger`] is the only thing in the engine that adds or
//! subtracts an allocation, and every wrapper reports **through** it.
//!
//! Three properties are structural rather than asserted, because a test cannot prove the
//! absence of a second writer:
//!
//! * **Conservation.** `available + reserved + spent + unknown == limit` after every
//!   operation. Nothing mutates a counter directly; each transition moves an exact
//!   [`Amount`] between two columns, so the sum cannot drift.
//! * **Unknown usage is not zero.** [`Provenance::Unmeasured`] moves cost into its own
//!   column that no read collapses into `spent`, so an unmeasured worker stays visible in
//!   [`Balance::unknown`] instead of silently freeing budget.
//! * **Release needs proof.** [`Ledger::release`] takes a [`Settlement`], not a flag, so a
//!   reservation cannot be freed by a caller that merely believes the work finished.
//!
//! Amounts are integers in a declared [`Unit`]. There is no float in this module's API, so
//! "reject negative and nonfinite" is discharged by the type rather than by a validator that
//! someone must remember to call; the fallible edge is
//! [`Amount::parse`], which reuses `contracts::parse_u64_decimal` and inherits its error
//! precedence.

use std::collections::BTreeMap;
use std::fmt;

use crate::contracts::{ScalarError, UuidV4, parse_u64_decimal};

/// The most reservations one ledger admits.
///
/// The bound is taken at the point of acquisition: [`Ledger::reserve`] refuses by name
/// **before** allocating an entry, because a cap checked after a push has already paid for
/// the growth it was meant to prevent.
pub const MAX_RESERVATIONS: usize = 4096;

/// The most usage reports one reservation admits.
///
/// Reports are the derived set: a ledger at [`MAX_RESERVATIONS`] with an unbounded report
/// list per reservation is still an unbounded allocation, so the derived set takes its own
/// bound and the refusal names both numbers.
pub const MAX_REPORTS_PER_RESERVATION: usize = 256;

/// The most tools one constraint or candidate may name.
pub const MAX_TOOLS: usize = 32;

/// The most fallback candidates one selection may consider.
///
/// The bound is on the set the selection ACQUIRES, not only on the slice it was handed: the
/// loop below reads every candidate's tool list, so an unbounded candidate list is an
/// unbounded read however short the constraint is.
pub const MAX_CANDIDATES: usize = 64;

/// Schema version of the persisted accounting shape owned by `store`.
pub const SCHEMA_VERSION: i64 = 1;

/// A declared accounting unit. Mixing two units is a refusal, never a conversion:
/// the engine has no admitted exchange rate and inventing one would hide cost.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Unit {
    /// Model tokens, input and output summed by the reporting adapter.
    Tokens,
    /// Wall-clock milliseconds attributable to the task.
    Milliseconds,
    /// Cost in millionths of a currency unit, so no fractional arithmetic is needed.
    Microcents,
}

impl Unit {
    /// The stable wire name. Used by `store` and by every diagnostic.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Tokens => "tokens",
            Self::Milliseconds => "milliseconds",
            Self::Microcents => "microcents",
        }
    }

    /// Every unit, so a caller enumerating them cannot silently miss one added later.
    pub const ALL: [Self; 3] = [Self::Tokens, Self::Milliseconds, Self::Microcents];

    /// Parse a wire name.
    ///
    /// # Errors
    ///
    /// Returns [`Refusal::UnknownUnit`] for any name outside [`Unit::ALL`].
    pub fn parse(name: &str) -> Result<Self, Refusal> {
        Self::ALL
            .into_iter()
            .find(|unit| unit.name() == name)
            .ok_or(Refusal::UnknownUnit)
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A non-negative quantity in a declared [`Unit`].
///
/// There is no constructor that can express a negative or fractional amount, which is why
/// this module documents no "reject negative" validator: the state is unrepresentable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Amount {
    unit: Unit,
    value: u64,
}

impl Amount {
    /// An amount of `value` in `unit`.
    #[must_use]
    pub const fn new(unit: Unit, value: u64) -> Self {
        Self { unit, value }
    }

    /// The additive identity for `unit`.
    #[must_use]
    pub const fn zero(unit: Unit) -> Self {
        Self::new(unit, 0)
    }

    /// The declared unit.
    #[must_use]
    pub const fn unit(self) -> Unit {
        self.unit
    }

    /// The magnitude, in `self.unit()`.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }

    /// Whether this amount is the additive identity.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.value == 0
    }

    /// Parse a canonical decimal in a declared unit.
    ///
    /// # Errors
    ///
    /// Returns [`Refusal::Scalar`] carrying `contracts`' own error precedence: empty,
    /// invalid character, noncanonical leading zero, then overflow. A fractional or signed
    /// literal fails as an invalid character, which is the honest diagnosis — this module
    /// has no fractional representation to round into.
    pub fn parse(unit: Unit, decimal: &str) -> Result<Self, Refusal> {
        parse_u64_decimal(decimal)
            .map(|value| Self::new(unit, value))
            .map_err(Refusal::Scalar)
    }

    /// Checked addition within one unit.
    ///
    /// # Errors
    ///
    /// [`Refusal::IncompatibleUnit`] when the units differ, [`Refusal::Overflow`] when the
    /// sum leaves `u64`.
    pub fn checked_add(self, other: Self) -> Result<Self, Refusal> {
        let value = self.same_unit(other)?;
        self.value
            .checked_add(value)
            .map(|value| Self::new(self.unit, value))
            .ok_or(Refusal::Overflow)
    }

    /// Checked subtraction within one unit.
    ///
    /// # Errors
    ///
    /// [`Refusal::IncompatibleUnit`] when the units differ, [`Refusal::Underflow`] when the
    /// result would be negative. Underflow is a distinct refusal from overflow because the
    /// two mean opposite things about the caller's accounting.
    pub fn checked_sub(self, other: Self) -> Result<Self, Refusal> {
        let value = self.same_unit(other)?;
        self.value
            .checked_sub(value)
            .map(|value| Self::new(self.unit, value))
            .ok_or(Refusal::Underflow)
    }

    fn same_unit(self, other: Self) -> Result<u64, Refusal> {
        if self.unit == other.unit {
            Ok(other.value)
        } else {
            Err(Refusal::IncompatibleUnit)
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.unit.name())
    }
}

/// How a usage figure was obtained. The engine treats these differently on purpose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provenance {
    /// The worker settled the figure itself; the most trustworthy source.
    WorkerSettled,
    /// A checker measured it while verifying the attempt. Counted, never omitted.
    CheckerMeasured,
    /// Store compaction measured it. Counted, never omitted.
    CompactionMeasured,
    /// The figure is an estimate the engine computed; counted, but flagged.
    Estimated,
    /// No figure was obtained. Cost is real but unmeasured.
    Unmeasured,
}

impl Provenance {
    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WorkerSettled => "worker-settled",
            Self::CheckerMeasured => "checker-measured",
            Self::CompactionMeasured => "compaction-measured",
            Self::Estimated => "estimated",
            Self::Unmeasured => "unmeasured",
        }
    }

    /// Every provenance, so an exhaustive caller cannot miss one.
    pub const ALL: [Self; 5] = [
        Self::WorkerSettled,
        Self::CheckerMeasured,
        Self::CompactionMeasured,
        Self::Estimated,
        Self::Unmeasured,
    ];

    /// Whether this figure counts as measured spend.
    ///
    /// [`Provenance::Unmeasured`] is the only `false`: its cost is real, so it moves into
    /// [`Balance::unknown`] rather than into `spent`, and never into `available`.
    #[must_use]
    pub const fn is_measured(self) -> bool {
        !matches!(self, Self::Unmeasured)
    }
}

impl fmt::Display for Provenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A reason this module refused. Diagnostics never echo untrusted input, so every variant
/// carries its meaning in the type rather than in an interpolated string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// Two amounts in different units met. There is no admitted exchange rate.
    IncompatibleUnit,
    /// A sum left `u64`.
    Overflow,
    /// A difference would have been negative.
    Underflow,
    /// A wire unit name outside [`Unit::ALL`].
    UnknownUnit,
    /// The requested amount exceeds what the scope still has available.
    InsufficientBudget,
    /// A hard-ceiling scope met a figure it cannot account for exactly.
    HardCeilingIncompatible,
    /// The reservation identity is not a lowercase hyphenated `UUIDv4`.
    MalformedIdentity(ScalarError),
    /// No reservation with that identity exists in this ledger.
    UnknownReservation,
    /// A second reservation claimed an identity already in use.
    DuplicateReservation,
    /// The reservation is no longer open, so it cannot take a new report.
    ReservationClosed,
    /// A parent identity was given that is not itself an open reservation.
    UnknownParent,
    /// A reservation cannot be its own parent, and a cycle cannot be expressed.
    SelfParent,
    /// The ledger already holds [`MAX_RESERVATIONS`] reservations.
    ReservationLimit,
    /// This reservation already holds [`MAX_REPORTS_PER_RESERVATION`] reports.
    ReportLimit,
    /// The caller's expected revision is not the ledger's current revision.
    StaleRevision,
    /// Release was attempted while a child reservation is still open.
    ChildOutstanding,
    /// A fallback candidate would send the task's data somewhere its privacy class does
    /// not admit. T10: *"fallback cannot cross privacy or tool constraints"*.
    PrivacyCrossing,
    /// A fallback candidate needs a tool the task's constraint does not permit.
    ToolNotPermitted,
    /// A constraint or candidate names more tools than [`MAX_TOOLS`].
    ToolLimit,
    /// A wire name outside [`Privacy::ALL`].
    UnknownPrivacyClass,
    /// A constraint or candidate carries an empty identity, an empty tool name or a
    /// repeated tool. Distinct from a scalar fault: nothing here is a `contracts` scalar,
    /// and borrowing one would put this module's diagnostic under another's precedence.
    MalformedConstraint,
    /// More candidates were offered than [`MAX_CANDIDATES`].
    CandidateLimit,
    /// Two candidates in one selection claimed the same identity.
    DuplicateCandidate,
    /// No offered candidate both satisfies the constraint and fits the remaining budget.
    NoAdmissibleFallback,
    /// A scalar rejected by `contracts`, with its own precedence preserved.
    Scalar(ScalarError),
}

impl Refusal {
    /// The stable diagnostic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::IncompatibleUnit => "incompatible accounting units",
            Self::Overflow => "accounting sum exceeds the permitted integer range",
            Self::Underflow => "accounting difference would be negative",
            Self::UnknownUnit => "unknown accounting unit",
            Self::InsufficientBudget => "insufficient remaining budget",
            Self::HardCeilingIncompatible => "hard ceiling rejects inexact accounting",
            Self::MalformedIdentity(_) => "malformed reservation identity",
            Self::UnknownReservation => "unknown reservation",
            Self::DuplicateReservation => "duplicate reservation identity",
            Self::ReservationClosed => "reservation is closed",
            Self::UnknownParent => "unknown parent reservation",
            Self::PrivacyCrossing => "fallback would cross the task's privacy constraint",
            Self::ToolNotPermitted => "fallback needs a tool the task does not permit",
            Self::ToolLimit => "more tools than the permitted maximum",
            Self::UnknownPrivacyClass => "unknown privacy class",
            Self::MalformedConstraint => "malformed constraint or candidate",
            Self::CandidateLimit => "more fallback candidates than the permitted maximum",
            Self::DuplicateCandidate => "two fallback candidates share one identity",
            Self::NoAdmissibleFallback => "no candidate satisfies the constraint within budget",
            Self::SelfParent => "a reservation cannot be its own parent",
            Self::ReservationLimit => "reservation count bound reached",
            Self::ReportLimit => "usage report count bound reached",
            Self::StaleRevision => "expected ledger revision differs",
            Self::ChildOutstanding => "a child reservation is still open",
            Self::Scalar(_) => "invalid accounting scalar",
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedIdentity(error) | Self::Scalar(error) => {
                write!(f, "{}: {error}", self.name())
            }
            other => f.write_str(other.name()),
        }
    }
}

impl std::error::Error for Refusal {}

/// Whether a scope may spend beyond exact accounting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ceiling {
    /// Unmeasured and estimated usage is admitted; it stays visible in
    /// [`Balance::unknown`] but does not refuse the report.
    Soft,
    /// Only exactly-accounted usage is admitted. A report whose provenance is
    /// [`Provenance::Unmeasured`] or [`Provenance::Estimated`] is refused with
    /// [`Refusal::HardCeilingIncompatible`] — T10's *"hard-ceiling tasks reject incompatible
    /// accounting"*.
    Hard,
}

impl Ceiling {
    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Soft => "soft",
            Self::Hard => "hard",
        }
    }

    /// Whether this ceiling admits a figure of the given provenance.
    #[must_use]
    pub const fn admits(self, provenance: Provenance) -> bool {
        match self {
            Self::Soft => true,
            Self::Hard => matches!(
                provenance,
                Provenance::WorkerSettled
                    | Provenance::CheckerMeasured
                    | Provenance::CompactionMeasured
            ),
        }
    }
}

/// Where a task's data may go.
///
/// Ordered from most to least restrictive. The order is the whole mechanism: a fallback is
/// admissible only when its class is **at least as restrictive** as the task's, so the
/// comparison is `candidate <= required` and there is no branch that relaxes one.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Privacy {
    /// Nothing leaves this process.
    InProcess,
    /// May reach another process on this host, but not the network.
    OnHost,
    /// May reach a named, already-admitted external service.
    AdmittedExternal,
}

impl Privacy {
    /// Every class, most restrictive first.
    pub const ALL: [Self; 3] = [Self::InProcess, Self::OnHost, Self::AdmittedExternal];

    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::InProcess => "in-process",
            Self::OnHost => "on-host",
            Self::AdmittedExternal => "admitted-external",
        }
    }

    /// The class for a wire name.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownPrivacyClass`] for a name outside [`Self::ALL`].
    pub fn parse(name: &str) -> Result<Self, Refusal> {
        Self::ALL
            .into_iter()
            .find(|class| class.name() == name)
            .ok_or(Refusal::UnknownPrivacyClass)
    }

    /// Whether data admitted at `self` may also be handled at `other`.
    #[must_use]
    pub const fn admits(self, other: Self) -> bool {
        (other as u8) <= (self as u8)
    }
}

/// What a task permits: a privacy class and a set of tools.
///
/// Built once from the task's own declaration and then only ever read. There is no method
/// that widens a constraint, so "the fallback quietly gained a capability" is not a state
/// this module can reach — the same shape as the skills loader's authority rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Constraint {
    privacy: Privacy,
    tools: Vec<String>,
}

impl Constraint {
    /// A constraint permitting `privacy` and exactly `tools`.
    ///
    /// # Errors
    ///
    /// [`Refusal::ToolLimit`] for more than [`MAX_TOOLS`] tools;
    /// [`Refusal::MalformedConstraint`] for an empty tool name or a repeated one.
    pub fn new(privacy: Privacy, tools: &[&str]) -> Result<Self, Refusal> {
        if tools.len() > MAX_TOOLS {
            return Err(Refusal::ToolLimit);
        }
        let mut owned: Vec<String> = Vec::with_capacity(tools.len());
        for tool in tools {
            if tool.is_empty() || owned.iter().any(|held| held == tool) {
                return Err(Refusal::MalformedConstraint);
            }
            owned.push((*tool).to_owned());
        }
        owned.sort();
        Ok(Self {
            privacy,
            tools: owned,
        })
    }

    /// The privacy class this constraint admits.
    #[must_use]
    pub const fn privacy(&self) -> Privacy {
        self.privacy
    }

    /// The permitted tools, sorted.
    #[must_use]
    pub fn tools(&self) -> &[String] {
        &self.tools
    }

    /// Whether `tool` is permitted.
    #[must_use]
    pub fn permits_tool(&self, tool: &str) -> bool {
        self.tools.iter().any(|held| held == tool)
    }
}

/// One alternative the caller is willing to fall back to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    identity: String,
    privacy: Privacy,
    tools: Vec<String>,
    cost: Amount,
}

impl Candidate {
    /// A candidate named `identity`, handling data at `privacy`, needing `tools`, costing
    /// `cost`.
    ///
    /// # Errors
    ///
    /// [`Refusal::ToolLimit`] for more than [`MAX_TOOLS`] tools;
    /// [`Refusal::MalformedConstraint`] for an empty identity, an empty tool name or a
    /// repeated one.
    pub fn new(
        identity: &str,
        privacy: Privacy,
        tools: &[&str],
        cost: Amount,
    ) -> Result<Self, Refusal> {
        if identity.is_empty() {
            return Err(Refusal::MalformedConstraint);
        }
        let constraint = Constraint::new(privacy, tools)?;
        Ok(Self {
            identity: identity.to_owned(),
            privacy,
            tools: constraint.tools,
            cost,
        })
    }

    /// The candidate's identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// The privacy class this candidate handles data at.
    #[must_use]
    pub const fn privacy(&self) -> Privacy {
        self.privacy
    }

    /// The tools this candidate needs.
    #[must_use]
    pub fn tools(&self) -> &[String] {
        &self.tools
    }

    /// What this candidate would cost.
    #[must_use]
    pub const fn cost(&self) -> Amount {
        self.cost
    }
}

/// Why one candidate was not selected.
///
/// A rejection is recorded per candidate rather than collapsed into a single refusal, so a
/// caller can see that a cheaper option existed and was excluded, and on which ground. A
/// selection that returned only "none admissible" would hide exactly that.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rejected {
    identity: String,
    reason: Refusal,
}

impl Rejected {
    /// The rejected candidate's identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Why it was rejected.
    #[must_use]
    pub const fn reason(&self) -> Refusal {
        self.reason
    }
}

/// The outcome of a fallback selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    chosen: Candidate,
    rejected: Vec<Rejected>,
}

impl Selection {
    /// The selected candidate.
    #[must_use]
    pub const fn chosen(&self) -> &Candidate {
        &self.chosen
    }

    /// Every candidate that was not selected, with its ground.
    #[must_use]
    pub fn rejected(&self) -> &[Rejected] {
        &self.rejected
    }
}

/// One usage figure, with where it came from.
///
/// The provenance travels with the amount rather than beside it, so a caller cannot report a
/// figure and forget to say how it was obtained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage {
    amount: Amount,
    provenance: Provenance,
}

impl Usage {
    /// A usage figure.
    #[must_use]
    pub const fn new(amount: Amount, provenance: Provenance) -> Self {
        Self { amount, provenance }
    }

    /// The reported amount.
    #[must_use]
    pub const fn amount(self) -> Amount {
        self.amount
    }

    /// How the figure was obtained.
    #[must_use]
    pub const fn provenance(self) -> Provenance {
        self.provenance
    }
}

/// Proof that work finished, required before a reservation is released.
///
/// A boolean would let a caller free budget by believing; this type makes the caller name
/// what it observed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Settlement<'a> {
    /// The worker settled and the attempt is accounted. The final usage is already reported.
    Settled { attempt: UuidV4<'a> },
    /// The attempt was abandoned with a liability that stays on the books: the reserved
    /// remainder moves to `unknown`, not back to `available`.
    Abandoned { attempt: UuidV4<'a> },
    /// Cancellation was requested but the effect is unknown. Identical accounting to
    /// [`Settlement::Abandoned`], named separately so the record says which happened.
    CancelledUncertain { attempt: UuidV4<'a> },
}

impl<'a> Settlement<'a> {
    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Settled { .. } => "settled",
            Self::Abandoned { .. } => "abandoned",
            Self::CancelledUncertain { .. } => "cancelled-uncertain",
        }
    }

    /// Whether the unspent remainder returns to `available`.
    ///
    /// Only a settled attempt frees budget. An abandoned or uncertainly-cancelled attempt
    /// keeps its remainder as a visible liability, because the engine cannot prove the cost
    /// was not incurred.
    #[must_use]
    pub const fn frees_remainder(self) -> bool {
        matches!(self, Self::Settled { .. })
    }

    /// The attempt this settlement names.
    #[must_use]
    pub const fn attempt(self) -> UuidV4<'a> {
        match self {
            Self::Settled { attempt }
            | Self::Abandoned { attempt }
            | Self::CancelledUncertain { attempt } => attempt,
        }
    }
}

/// The four columns of a scope, which always sum to its limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Balance {
    limit: Amount,
    reserved: Amount,
    spent: Amount,
    unknown: Amount,
}

impl Balance {
    /// The configured upper bound.
    #[must_use]
    pub const fn limit(self) -> Amount {
        self.limit
    }

    /// Held by open reservations, not yet spent.
    #[must_use]
    pub const fn reserved(self) -> Amount {
        self.reserved
    }

    /// Measured usage.
    #[must_use]
    pub const fn spent(self) -> Amount {
        self.spent
    }

    /// Cost that is real but unmeasured, plus remainders of abandoned work.
    ///
    /// This column exists so that "unknown usage is not zero" is a readable fact rather than
    /// an omission. Nothing ever moves out of it.
    #[must_use]
    pub const fn unknown(self) -> Amount {
        self.unknown
    }

    /// What a new reservation may still take.
    ///
    /// # Errors
    ///
    /// [`Refusal::Underflow`] can only arise if the conservation invariant were already
    /// broken, which no public transition can do; it is returned rather than panicked so
    /// that a corrupted persisted ledger refuses instead of aborting the process.
    pub fn available(self) -> Result<Amount, Refusal> {
        self.limit
            .checked_sub(self.reserved)?
            .checked_sub(self.spent)?
            .checked_sub(self.unknown)
    }
}

/// The lifecycle of one reservation. A closed reservation never reopens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Open,
    Closed,
}

/// One reservation's own books.
#[derive(Clone, Debug)]
struct Entry {
    identity: String,
    parent: Option<String>,
    held: Amount,
    spent: Amount,
    unknown: Amount,
    state: State,
    /// Report identities already applied, so a retried report is idempotent **by identity**
    /// rather than by comparing figures — two honest reports can carry the same amount.
    applied: BTreeMap<String, Usage>,
}

/// What a reservation looks like to a caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Reservation<'a> {
    /// The reservation identity.
    pub identity: UuidV4<'a>,
    /// The parent reservation, if this is child work.
    pub parent: Option<UuidV4<'a>>,
    /// Still held and unspent.
    pub held: Amount,
    /// Measured usage attributed to this reservation.
    pub spent: Amount,
    /// Unmeasured usage attributed to this reservation.
    pub unknown: Amount,
    /// Whether the reservation is still open.
    pub open: bool,
}

/// The outcome of applying one usage report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Applied {
    /// `false` when the report identity had already been applied; the ledger is unchanged
    /// and the caller may treat its retry as successful.
    pub fresh: bool,
    /// The reservation's remaining held amount after the report.
    pub held: Amount,
    /// Reported-minus-held overrun that the ledger absorbed from the scope, if any.
    ///
    /// An overrun is a discrepancy, not a refusal: the cost was already incurred, and
    /// refusing to record it would make the books cheaper than reality.
    pub discrepancy: Option<Amount>,
}

/// The single owner of every counter in one accounting scope.
///
/// A scope is a task or a thread; `store` persists one ledger per scope at
/// [`SCHEMA_VERSION`]. Child work takes a reservation whose `parent` is its caller's
/// reservation, so parent and child draw from the **same** limit and concurrent oversubscription
/// is impossible without a second writer, which this type's ownership forbids.
#[derive(Clone, Debug)]
pub struct Ledger {
    unit: Unit,
    limit: Amount,
    ceiling: Ceiling,
    revision: u64,
    entries: Vec<Entry>,
}

impl Ledger {
    /// A ledger for `limit`, under `ceiling`.
    ///
    /// The unit is taken from the limit and every later amount must match it.
    #[must_use]
    pub fn new(limit: Amount, ceiling: Ceiling) -> Self {
        Self {
            unit: limit.unit(),
            limit,
            ceiling,
            revision: 0,
            entries: Vec::new(),
        }
    }

    /// The unit every amount in this ledger must use.
    #[must_use]
    pub const fn unit(&self) -> Unit {
        self.unit
    }

    /// The ceiling policy.
    #[must_use]
    pub const fn ceiling(&self) -> Ceiling {
        self.ceiling
    }

    /// The current revision, incremented by every mutation that changes a counter.
    ///
    /// A caller that read a balance and then reserves may pass this back as
    /// `expected_revision` to make its decision atomic with respect to the read.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// The number of reservations, open and closed.
    #[must_use]
    pub fn reservations(&self) -> usize {
        self.entries.len()
    }

    /// The scope's four columns.
    ///
    /// # Errors
    ///
    /// [`Refusal::Overflow`] if the columns cannot be summed, which a corrupted persisted
    /// ledger could cause.
    pub fn balance(&self) -> Result<Balance, Refusal> {
        let mut reserved = Amount::zero(self.unit);
        let mut spent = Amount::zero(self.unit);
        let mut unknown = Amount::zero(self.unit);
        for entry in &self.entries {
            reserved = reserved.checked_add(entry.held)?;
            spent = spent.checked_add(entry.spent)?;
            unknown = unknown.checked_add(entry.unknown)?;
        }
        Ok(Balance {
            limit: self.limit,
            reserved,
            spent,
            unknown,
        })
    }

    /// What a new reservation may still take.
    ///
    /// # Errors
    ///
    /// Propagates [`Ledger::balance`].
    pub fn available(&self) -> Result<Amount, Refusal> {
        self.balance()?.available()
    }

    /// Admit a reservation of `amount`, optionally as child work under `parent`.
    ///
    /// `expected_revision` makes a read-then-reserve atomic: pass the revision the balance
    /// was read at, or `None` to accept whatever the current one is.
    ///
    /// # Errors
    ///
    /// * [`Refusal::StaleRevision`] when `expected_revision` is not current — the caller's
    ///   view of `available` is out of date and re-reading may change its decision;
    /// * [`Refusal::ReservationLimit`] at [`MAX_RESERVATIONS`], refused **before** the entry
    ///   is allocated;
    /// * [`Refusal::IncompatibleUnit`] when `amount` is not in the ledger's unit;
    /// * [`Refusal::DuplicateReservation`], [`Refusal::MalformedIdentity`],
    ///   [`Refusal::SelfParent`], [`Refusal::UnknownParent`];
    /// * [`Refusal::InsufficientBudget`] when `amount` exceeds what remains. Concurrent
    ///   parent and child work cannot exceed the bound, because both draw here.
    pub fn reserve(
        &mut self,
        identity: &str,
        amount: Amount,
        parent: Option<&str>,
        expected_revision: Option<u64>,
    ) -> Result<Amount, Refusal> {
        if let Some(expected) = expected_revision
            && expected != self.revision
        {
            return Err(Refusal::StaleRevision);
        }
        if self.entries.len() >= MAX_RESERVATIONS {
            return Err(Refusal::ReservationLimit);
        }
        if amount.unit() != self.unit {
            return Err(Refusal::IncompatibleUnit);
        }
        let identity = UuidV4::parse(identity).map_err(Refusal::MalformedIdentity)?;
        if self.find(identity.as_str()).is_some() {
            return Err(Refusal::DuplicateReservation);
        }
        let parent = match parent {
            None => None,
            Some(parent) => {
                let parent = UuidV4::parse(parent).map_err(Refusal::MalformedIdentity)?;
                if parent.as_str() == identity.as_str() {
                    return Err(Refusal::SelfParent);
                }
                let entry = self.find(parent.as_str()).ok_or(Refusal::UnknownParent)?;
                if self.entries[entry].state != State::Open {
                    return Err(Refusal::UnknownParent);
                }
                Some(parent.as_str().to_owned())
            }
        };
        let available = self.available()?;
        if amount.value() > available.value() {
            return Err(Refusal::InsufficientBudget);
        }
        self.entries.push(Entry {
            identity: identity.as_str().to_owned(),
            parent,
            held: amount,
            spent: Amount::zero(self.unit),
            unknown: Amount::zero(self.unit),
            state: State::Open,
            applied: BTreeMap::new(),
        });
        self.revision = self.revision.wrapping_add(1);
        available.checked_sub(amount)
    }

    /// Apply one usage report against an open reservation.
    ///
    /// Idempotent by `report`: a retry with an identity already applied returns
    /// `fresh: false` and changes nothing, so a worker that resends after a lost reply does
    /// not double-count.
    ///
    /// A report larger than the remaining held amount is **recorded**, not refused: the cost
    /// is already incurred. The excess is taken from the scope's available budget and
    /// returned as [`Applied::discrepancy`]. If the scope cannot cover it, the remainder
    /// moves to `unknown` rather than vanishing.
    ///
    /// # Errors
    ///
    /// * [`Refusal::UnknownReservation`], [`Refusal::ReservationClosed`];
    /// * [`Refusal::IncompatibleUnit`] when the figure is in another unit;
    /// * [`Refusal::HardCeilingIncompatible`] under [`Ceiling::Hard`] for an
    ///   [`Provenance::Unmeasured`] or [`Provenance::Estimated`] figure;
    /// * [`Refusal::ReportLimit`] at [`MAX_REPORTS_PER_RESERVATION`], refused before the
    ///   report is stored;
    /// * [`Refusal::Overflow`] on a sum that leaves `u64`.
    pub fn report(
        &mut self,
        reservation: &str,
        report: &str,
        usage: Usage,
    ) -> Result<Applied, Refusal> {
        if usage.amount().unit() != self.unit {
            return Err(Refusal::IncompatibleUnit);
        }
        if !self.ceiling.admits(usage.provenance()) {
            return Err(Refusal::HardCeilingIncompatible);
        }
        let index = self.find(reservation).ok_or(Refusal::UnknownReservation)?;
        if self.entries[index].state != State::Open {
            return Err(Refusal::ReservationClosed);
        }
        if let Some(previous) = self.entries[index].applied.get(report) {
            let held = self.entries[index].held;
            let _ = previous;
            return Ok(Applied {
                fresh: false,
                held,
                discrepancy: None,
            });
        }
        if self.entries[index].applied.len() >= MAX_REPORTS_PER_RESERVATION {
            return Err(Refusal::ReportLimit);
        }
        let available = self.available()?;
        let entry = &self.entries[index];
        let reported = usage.amount();
        let from_held = reported.value().min(entry.held.value());
        let overrun = reported.value() - from_held;
        let from_available = overrun.min(available.value());
        let uncovered = overrun - from_available;

        let entry = &mut self.entries[index];
        entry.held = entry.held.checked_sub(Amount::new(self.unit, from_held))?;
        let recorded = Amount::new(self.unit, from_held + from_available + uncovered);
        if usage.provenance().is_measured() {
            entry.spent = entry.spent.checked_add(recorded)?;
        } else {
            entry.unknown = entry.unknown.checked_add(recorded)?;
        }
        entry.applied.insert(report.to_owned(), usage);
        let held = entry.held;
        self.revision = self.revision.wrapping_add(1);
        Ok(Applied {
            fresh: true,
            held,
            discrepancy: (overrun > 0).then(|| Amount::new(self.unit, overrun)),
        })
    }

    /// Close a reservation against settlement proof.
    ///
    /// Only [`Settlement::Settled`] returns the unspent remainder to `available`. An
    /// abandoned or uncertainly-cancelled attempt moves its remainder into `unknown`, where
    /// it stays visible as a liability — the engine cannot prove the cost was not incurred,
    /// and quietly returning it would make the books cheaper than reality.
    ///
    /// # Errors
    ///
    /// * [`Refusal::UnknownReservation`], [`Refusal::ReservationClosed`];
    /// * [`Refusal::MalformedIdentity`] when the settlement's attempt id is not a `UUIDv4`;
    /// * [`Refusal::ChildOutstanding`] when a child reservation is still open — releasing a
    ///   parent while its child holds budget would leave the child's hold parented to a
    ///   closed entry, which no later transition could settle;
    /// * [`Refusal::Overflow`] on a sum that leaves `u64`.
    pub fn release(
        &mut self,
        reservation: &str,
        settlement: Settlement<'_>,
    ) -> Result<Balance, Refusal> {
        let _ = UuidV4::parse(settlement.attempt().as_str()).map_err(Refusal::MalformedIdentity)?;
        let index = self.find(reservation).ok_or(Refusal::UnknownReservation)?;
        if self.entries[index].state != State::Open {
            return Err(Refusal::ReservationClosed);
        }
        let identity = self.entries[index].identity.clone();
        if self.entries.iter().any(|entry| {
            entry.state == State::Open && entry.parent.as_deref() == Some(identity.as_str())
        }) {
            return Err(Refusal::ChildOutstanding);
        }
        let entry = &mut self.entries[index];
        let remainder = entry.held;
        entry.held = Amount::zero(self.unit);
        if !settlement.frees_remainder() {
            entry.unknown = entry.unknown.checked_add(remainder)?;
        }
        entry.state = State::Closed;
        self.revision = self.revision.wrapping_add(1);
        self.balance()
    }

    /// One reservation's own books, borrowed from the ledger.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownReservation`] when no reservation carries that identity.
    pub fn reservation<'a>(&'a self, identity: &str) -> Result<Reservation<'a>, Refusal> {
        let index = self.find(identity).ok_or(Refusal::UnknownReservation)?;
        let entry = &self.entries[index];
        Ok(Reservation {
            identity: UuidV4::parse(entry.identity.as_str()).map_err(Refusal::MalformedIdentity)?,
            parent: match entry.parent.as_deref() {
                None => None,
                Some(parent) => Some(UuidV4::parse(parent).map_err(Refusal::MalformedIdentity)?),
            },
            held: entry.held,
            spent: entry.spent,
            unknown: entry.unknown,
            open: entry.state == State::Open,
        })
    }

    /// Every reservation, oldest first.
    ///
    /// # Errors
    ///
    /// [`Refusal::MalformedIdentity`] if a persisted identity is not a `UUIDv4`.
    pub fn reservation_list(&self) -> Result<Vec<Reservation<'_>>, Refusal> {
        self.entries
            .iter()
            .map(|entry| self.reservation(entry.identity.as_str()))
            .collect()
    }

    /// Choose the cheapest fallback that the task's constraint admits and the budget affords.
    ///
    /// T10's clause is *"fallback cannot cross privacy or tool constraints"*, and this is the
    /// only place in the module that selects one. Two properties are structural rather than
    /// asserted:
    ///
    /// * **The selection can only narrow.** A candidate is admissible when its privacy class
    ///   is at least as restrictive as the constraint's and its tools are a SUBSET of the
    ///   permitted set. There is no branch that relaxes the class or adds a tool, so "the
    ///   fallback widened what the task could do" is not a state reachable here.
    /// * **An excluded candidate is named.** Each rejection carries the candidate's identity
    ///   and its ground, so a caller can see that a cheaper option existed and why it was not
    ///   taken. Returning only "none admissible" would hide exactly that.
    ///
    /// Ties are broken by identity so the choice is deterministic; a caller that re-offers the
    /// same candidates gets the same answer, which a cost-only comparison would not give.
    ///
    /// # Errors
    ///
    /// [`Refusal::CandidateLimit`] for more than [`MAX_CANDIDATES`] candidates;
    /// [`Refusal::IncompatibleUnit`] if a candidate's cost is in another unit;
    /// [`Refusal::NoAdmissibleFallback`] when every candidate was excluded.
    pub fn select_fallback(
        &self,
        constraint: &Constraint,
        candidates: &[Candidate],
    ) -> Result<Selection, Refusal> {
        if candidates.len() > MAX_CANDIDATES {
            return Err(Refusal::CandidateLimit);
        }
        // Identities must be distinct. Two candidates sharing one made the tie-break's
        // comparison decide between them by position, which is not a decision this module
        // should be making: with equal cost and equal identity the pair still differ in
        // privacy or tools, so "the first one" would silently pick a different capability
        // set. Mutation testing found it -- `<` and `<=` in the comparison below were
        // indistinguishable only because no case could tell the two apart.
        for (index, candidate) in candidates.iter().enumerate() {
            if candidates[..index]
                .iter()
                .any(|earlier| earlier.identity() == candidate.identity())
            {
                return Err(Refusal::DuplicateCandidate);
            }
        }
        let available = self.available()?;
        let mut rejected: Vec<Rejected> = Vec::with_capacity(candidates.len());
        let mut best: Option<&Candidate> = None;
        for candidate in candidates {
            let reason = if !constraint.privacy().admits(candidate.privacy()) {
                Some(Refusal::PrivacyCrossing)
            } else if candidate
                .tools()
                .iter()
                .any(|tool| !constraint.permits_tool(tool))
            {
                Some(Refusal::ToolNotPermitted)
            } else if candidate.cost().unit() != available.unit() {
                Some(Refusal::IncompatibleUnit)
            } else if candidate.cost().value() > available.value() {
                Some(Refusal::InsufficientBudget)
            } else {
                None
            };
            if let Some(reason) = reason {
                rejected.push(Rejected {
                    identity: candidate.identity().to_owned(),
                    reason,
                });
                continue;
            }
            let better = match best {
                None => true,
                Some(held) => {
                    (candidate.cost().value(), candidate.identity())
                        < (held.cost().value(), held.identity())
                }
            };
            if better {
                best = Some(candidate);
            }
        }
        match best {
            Some(chosen) => {
                for candidate in candidates {
                    if candidate.identity() != chosen.identity()
                        && !rejected
                            .iter()
                            .any(|row| row.identity() == candidate.identity())
                    {
                        rejected.push(Rejected {
                            identity: candidate.identity().to_owned(),
                            reason: Refusal::NoAdmissibleFallback,
                        });
                    }
                }
                Ok(Selection {
                    chosen: chosen.clone(),
                    rejected,
                })
            }
            None => Err(Refusal::NoAdmissibleFallback),
        }
    }

    /// Whether conservation holds: the four columns sum to the limit.
    ///
    /// The transitions above maintain this by construction; this predicate exists so that a
    /// ledger **read back from storage** can be checked before it is trusted, and so a
    /// property test has something to assert that is not a restatement of the code.
    ///
    /// # Errors
    ///
    /// Propagates [`Ledger::balance`].
    pub fn conserves(&self) -> Result<bool, Refusal> {
        let balance = self.balance()?;
        let available = balance.available()?;
        let total = available
            .checked_add(balance.reserved())?
            .checked_add(balance.spent())?
            .checked_add(balance.unknown())?;
        Ok(total == self.limit)
    }

    fn find(&self, identity: &str) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| entry.identity == identity)
    }
}
