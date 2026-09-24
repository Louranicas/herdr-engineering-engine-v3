// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/herdr.rs
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
// Readiness binding: HEE3-READINESS-001; SHA-256 f574043487f39db6424c4988bce58e88a6e766f02f974fbcf3fa8dd6e0003548; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-03, R90-07, R90-09, R90-10; resolved contracts RC02, RC03, RC04, RC05; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-herdr; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-herdr (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-herdr
// Owns: Herdr client adaptation and task/pane presentation links
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/herdr.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-herdr)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T14, T16, T17, T18, T19, T20, T25, T26, T27
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K5)
// [contributing codebase CODE-CB01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB01)
// [contributing codebase CODE-CB11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB11)
// [contributing codebase CODE-CB12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB12)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T16](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T16)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [separate reference example EX-herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-herdr)
// [flow FLOW-F01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F01)
// [handbook HB-herdr-method-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-herdr-method-map)
// [handbook HB-socket-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-socket-map)
// [API API-API06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API06)
// [IPC IPC-IPC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC02)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-herdr)
// [plan SEC-api-sockets](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-api-sockets)
// [plan SEC-architecture](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-architecture)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-runtime](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-runtime)
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
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC23)
// [source SRC-D19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-D19)
// [source SRC-H10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-H10)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-herdr)
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
// [readiness improvement grouping R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
// [readiness improvement grouping R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-herdr)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
// Applicable learning IDs: LRN01, LRN08, LRN13; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [Working in Sandboxes on Kinoite](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20in%20Sandboxes%20on%20Kinoite)
// HEE3-ANCHORS-END

//! The multiplexer/client integration surface: a **client**, never an authority.
//!
//! `docs/modules/herdr.md` fixes the boundary twice over — *"Panes are clients, not
//! authoritative rosters, schedulers or completion databases"* and *"Pane title, process exit
//! or rendered done text cannot establish task acceptance."*
//!
//! Both are structural:
//!
//! * **A view cannot mint acceptance.** [`Acceptance`] has one producer, [`Admitted::from_engine`],
//!   which takes an engine-issued [`EngineReceipt`]. Nothing a pane observes — a title, an exit
//!   status, rendered text — constructs one, because none of those is an `EngineReceipt`. A
//!   client that believes a task succeeded still cannot say so in this type system.
//! * **The client owns no task state.** [`View`] holds a snapshot and a cursor. It has no
//!   method that transitions a task, and reconnecting rebuilds from the engine's
//!   [`Snapshot`] rather than from anything the client retained.
//!
//! Submission is idempotent by the caller's own [`IntentKey`], so a duplicate click, a
//! replayed keystroke or a reconnect that resends does not admit two tasks.

use std::collections::BTreeMap;
use std::fmt;

use crate::contracts::{ScalarError, UuidV4};

/// The most events one client buffers before it must reconnect by cursor.
pub const MAX_BUFFERED_EVENTS: usize = 1024;

/// The most reconnect attempts recorded for one client.
pub const MAX_RECONNECTS: u32 = 1024;

/// Schema version of the persisted client view shape.
pub const SCHEMA_VERSION: i64 = 1;

/// A reason this module refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// An identity that is not a lowercase hyphenated `UUIDv4`.
    MalformedIdentity(ScalarError),
    /// The client's epoch is not the engine's; the view must be rebuilt.
    EpochMismatch,
    /// An event older than the client's cursor was offered.
    StaleEvent,
    /// The event buffer is full; the client must reconnect by cursor.
    BufferFull,
    /// The reconnect bound was reached.
    ReconnectLimit,
    /// No task with that identity is in this view.
    UnknownTask,
    /// A cursor beyond anything the engine has emitted.
    CursorAhead,
    /// A second, different task was reported under an intent key already submitted. The
    /// earlier outcome stands; the view does not relabel the new one as a duplicate.
    IntentConflict,
}

impl Refusal {
    /// The stable diagnostic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MalformedIdentity(_) => "malformed client identity",
            Self::EpochMismatch => "client epoch differs from the engine",
            Self::StaleEvent => "event precedes the client cursor",
            Self::BufferFull => "client event buffer bound reached",
            Self::ReconnectLimit => "reconnect bound reached",
            Self::UnknownTask => "unknown task in this view",
            Self::CursorAhead => "cursor follows the engine sequence",
            Self::IntentConflict => "a different task under an intent key already submitted",
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

/// A receipt the engine issued. The client cannot construct one from an observation.
///
/// This is the seam that makes *"pane title, process exit or rendered done text cannot
/// establish task acceptance"* a property of the types rather than a rule in a document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EngineReceipt<'a> {
    task: UuidV4<'a>,
    epoch: u64,
    sequence: u64,
}

impl<'a> EngineReceipt<'a> {
    /// The view's rendering of an engine admission: the task, epoch and sequence the engine
    /// reported after durable admission.
    ///
    /// This is **not** authority, and no type could make it so: a client parses receipts from
    /// engine responses, and a client can always fabricate a response. Acceptance is decided by
    /// the store's durable admission; the protection is that no engine module takes an
    /// `EngineReceipt` as evidence (pinned by `T16-HD-54`, which requires the type to be named
    /// in this file and nowhere else under `src/`). Review D3.
    ///
    /// # Errors
    ///
    /// [`Refusal::MalformedIdentity`] when `task` is not a `UUIDv4`.
    pub fn issue(task: &'a str, epoch: u64, sequence: u64) -> Result<Self, Refusal> {
        Ok(Self {
            task: UuidV4::parse(task).map_err(Refusal::MalformedIdentity)?,
            epoch,
            sequence,
        })
    }

    /// The durable task identity.
    #[must_use]
    pub const fn task(self) -> UuidV4<'a> {
        self.task
    }

    /// The epoch the admission belongs to.
    #[must_use]
    pub const fn epoch(self) -> u64 {
        self.epoch
    }

    /// The admitting sequence.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

/// Durable acceptance, obtainable only from an [`EngineReceipt`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Acceptance<'a> {
    receipt: EngineReceipt<'a>,
}

impl<'a> Acceptance<'a> {
    /// The task the engine admitted.
    #[must_use]
    pub const fn task(self) -> UuidV4<'a> {
        self.receipt.task()
    }

    /// The receipt that established it.
    #[must_use]
    pub const fn receipt(self) -> EngineReceipt<'a> {
        self.receipt
    }
}

/// What a submission produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Admitted<'a> {
    /// The engine admitted the intent. Carries the receipt that proves it.
    Accepted(Acceptance<'a>),
    /// The engine refused. The client shows the refusal; it does not retry on its own.
    Denied,
    /// This intent key was already submitted; the earlier outcome stands.
    Duplicate(Acceptance<'a>),
}

impl<'a> Admitted<'a> {
    /// Build an acceptance from an engine receipt. The only producer of [`Acceptance`].
    #[must_use]
    pub const fn from_engine(receipt: EngineReceipt<'a>) -> Self {
        Self::Accepted(Acceptance { receipt })
    }

    /// The acceptance, when there is one.
    #[must_use]
    pub const fn acceptance(self) -> Option<Acceptance<'a>> {
        match self {
            Self::Accepted(acceptance) | Self::Duplicate(acceptance) => Some(acceptance),
            Self::Denied => None,
        }
    }

    /// Whether a durable task exists because of this submission.
    ///
    /// A duplicate is `true`: the task exists, it simply was not created twice.
    #[must_use]
    pub const fn is_durable(self) -> bool {
        matches!(self, Self::Accepted(_) | Self::Duplicate(_))
    }
}

/// How the engine currently describes one task.
///
/// `Unknown` is a first-class state. A client that cannot tell "failed" from "I have not
/// heard" will eventually present one as the other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// Admitted and not yet finished.
    Running,
    /// Finished, by the engine's own account.
    Passed,
    /// Failed, by the engine's own account.
    Failed,
    /// Cancellation is recorded as an obligation, not yet settled.
    CancellationPending,
    /// The engine has not reported. Never rendered as success or failure.
    Unknown,
}

impl Status {
    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::CancellationPending => "cancellation-pending",
            Self::Unknown => "unknown",
        }
    }

    /// Every status.
    pub const ALL: [Self; 5] = [
        Self::Running,
        Self::Passed,
        Self::Failed,
        Self::CancellationPending,
        Self::Unknown,
    ];

    /// Whether this status is a settled engine verdict.
    ///
    /// `Unknown` and `CancellationPending` are not: presenting either as settled would be
    /// the client inventing an outcome.
    #[must_use]
    pub const fn is_settled(self) -> bool {
        matches!(self, Self::Passed | Self::Failed)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The engine's description of one task, as the client received it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    /// The durable task identity.
    pub task: String,
    /// The engine's current status.
    pub status: Status,
    /// The route explanation, when the engine supplied one.
    pub route_explanation: Option<String>,
    /// References to retained proof, for navigation. The client never interprets them.
    pub evidence: Vec<String>,
    /// What the engine could not tell the client, preserved rather than omitted.
    pub gaps: Vec<String>,
    /// The engine sequence this snapshot reflects.
    pub sequence: u64,
}

/// A caller-chosen key that makes submission idempotent.
///
/// A duplicate click, a replayed keystroke and a reconnect that resends all carry the same
/// key, so the engine admits one task. The key is the client's, because only the client
/// knows that two keystrokes were one intent.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IntentKey(String);

impl IntentKey {
    /// A key for one operator intent.
    #[must_use]
    pub fn new(key: &str) -> Self {
        Self(key.to_owned())
    }

    /// The underlying key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A client view: a presentation of engine state, owning none of it.
#[derive(Clone, Debug)]
pub struct View {
    epoch: u64,
    cursor: u64,
    reconnects: u32,
    buffered: Vec<(u64, String)>,
    snapshots: BTreeMap<String, Snapshot>,
    submitted: BTreeMap<String, String>,
}

impl View {
    /// An empty view attached to `epoch`.
    #[must_use]
    pub fn new(epoch: u64) -> Self {
        Self {
            epoch,
            cursor: 0,
            reconnects: 0,
            buffered: Vec::new(),
            snapshots: BTreeMap::new(),
            submitted: BTreeMap::new(),
        }
    }

    /// The epoch this view is attached to.
    #[must_use]
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Where the client has read to.
    #[must_use]
    pub const fn cursor(&self) -> u64 {
        self.cursor
    }

    /// How many times this client has reconnected.
    #[must_use]
    pub const fn reconnects(&self) -> u32 {
        self.reconnects
    }

    /// The number of buffered, unrendered events.
    #[must_use]
    pub fn buffered(&self) -> usize {
        self.buffered.len()
    }

    /// The number of tasks this view is presenting.
    #[must_use]
    pub fn tasks(&self) -> usize {
        self.snapshots.len()
    }

    /// Record an operator submission, idempotent by `key`.
    ///
    /// The engine decides; this records what it decided so a resubmission of the same intent
    /// returns [`Admitted::Duplicate`] rather than admitting a second task.
    ///
    /// # Errors
    ///
    /// * [`Refusal::EpochMismatch`] when the receipt belongs to another epoch;
    /// * [`Refusal::IntentConflict`] when `key` was already submitted for a different task.
    pub fn submit<'a>(
        &mut self,
        key: &IntentKey,
        outcome: Admitted<'a>,
    ) -> Result<Admitted<'a>, Refusal> {
        if let Some(acceptance) = outcome.acceptance()
            && acceptance.receipt().epoch() != self.epoch
        {
            return Err(Refusal::EpochMismatch);
        }
        if let Some(existing) = self.submitted.get(key.as_str()) {
            // The earlier outcome stands (review D2): the same task again is a duplicate; a
            // different task under this key is refused rather than relabelled as the first.
            return match outcome.acceptance() {
                Some(acceptance) if acceptance.task().as_str() != existing.as_str() => {
                    Err(Refusal::IntentConflict)
                }
                Some(acceptance) => Ok(Admitted::Duplicate(acceptance)),
                None => Ok(outcome),
            };
        }
        if let Some(acceptance) = outcome.acceptance() {
            self.submitted.insert(
                key.as_str().to_owned(),
                acceptance.task().as_str().to_owned(),
            );
        }
        Ok(outcome)
    }

    /// The durable task a previous submission of `key` produced, if any.
    #[must_use]
    pub fn submitted(&self, key: &IntentKey) -> Option<&str> {
        self.submitted.get(key.as_str()).map(String::as_str)
    }

    /// Accept one engine event, advancing the cursor.
    ///
    /// # Errors
    ///
    /// * [`Refusal::EpochMismatch`] when the event belongs to another epoch;
    /// * [`Refusal::StaleEvent`] for a sequence at or before the cursor — a replayed event
    ///   must not be rendered as new;
    /// * [`Refusal::BufferFull`] at [`MAX_BUFFERED_EVENTS`], refused before the event is
    ///   stored so a slow renderer cannot make the client grow without bound.
    pub fn observe(&mut self, epoch: u64, sequence: u64, text: &str) -> Result<(), Refusal> {
        if epoch != self.epoch {
            return Err(Refusal::EpochMismatch);
        }
        if sequence <= self.cursor {
            return Err(Refusal::StaleEvent);
        }
        if self.buffered.len() >= MAX_BUFFERED_EVENTS {
            return Err(Refusal::BufferFull);
        }
        self.buffered.push((sequence, text.to_owned()));
        self.cursor = sequence;
        Ok(())
    }

    /// Take the buffered events for rendering, leaving the buffer empty.
    #[must_use]
    pub fn drain(&mut self) -> Vec<(u64, String)> {
        std::mem::take(&mut self.buffered)
    }

    /// Apply an engine snapshot.
    ///
    /// A snapshot older than one already held is ignored, so an out-of-order arrival cannot
    /// move a task's presentation backwards.
    ///
    /// # Errors
    ///
    /// [`Refusal::MalformedIdentity`] when the snapshot's task is not a `UUIDv4`.
    pub fn present(&mut self, snapshot: Snapshot) -> Result<bool, Refusal> {
        let task = UuidV4::parse(snapshot.task.as_str()).map_err(Refusal::MalformedIdentity)?;
        let key = task.as_str().to_owned();
        if let Some(existing) = self.snapshots.get(&key)
            && existing.sequence >= snapshot.sequence
        {
            return Ok(false);
        }
        self.snapshots.insert(key, snapshot);
        Ok(true)
    }

    /// The presented state of one task.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownTask`] when this view is not presenting it.
    pub fn task(&self, task: &str) -> Result<&Snapshot, Refusal> {
        self.snapshots.get(task).ok_or(Refusal::UnknownTask)
    }

    /// Rebuild after losing the client, from the engine's own state.
    ///
    /// The buffer is discarded and the cursor is taken from the engine, because anything the
    /// client retained across the loss is exactly what it cannot vouch for. Engine
    /// obligations are unaffected: this method does not touch task state, and there is none
    /// here to touch.
    ///
    /// # Errors
    ///
    /// * [`Refusal::ReconnectLimit`] at [`MAX_RECONNECTS`];
    /// * [`Refusal::CursorAhead`] when the engine reports a cursor behind the client's,
    ///   which means the client saw something the engine did not emit.
    pub fn reconnect(&mut self, epoch: u64, engine_cursor: u64) -> Result<(), Refusal> {
        if self.reconnects >= MAX_RECONNECTS {
            return Err(Refusal::ReconnectLimit);
        }
        if epoch == self.epoch && engine_cursor < self.cursor {
            return Err(Refusal::CursorAhead);
        }
        if epoch != self.epoch {
            self.snapshots.clear();
            self.submitted.clear();
        }
        self.epoch = epoch;
        self.cursor = engine_cursor;
        self.buffered.clear();
        self.reconnects += 1;
        Ok(())
    }
}
