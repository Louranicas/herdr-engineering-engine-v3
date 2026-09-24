// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/worker/native.rs
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
// Readiness binding: HEE3-READINESS-001; SHA-256 f574043487f39db6424c4988bce58e88a6e766f02f974fbcf3fa8dd6e0003548; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-02, R90-03, R90-05, R90-06, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
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

//! The concrete Ollama adapter is appended below the retained module anchors.
//!
//! One local, output-only invocation. The coordinator owns commissioning, alias
//! custody, Store transitions and acceptance. Client cleanup is not provider
//! cancellation or loaded-model resource settlement.

use super::{
    CancelReason, Candidate, Capabilities, Contract, ContractError, Envelope, Event, Feature,
    Finish, Identity, IdentityOrigin, Request, Terminal, Usage, UsageForm, UsageScope, UsageStage,
    process::{self, ProcessReport, ProcessSpec},
};
use crate::contracts::Sha256Digest;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const PROFILE: &str = "ollama-fc44-12ff8654/1";
pub const PROVIDER: &str = "ollama-local";
const ENDPOINT: &str = "http://127.0.0.1:11434/api/";
const FRAME_LIMIT: usize = 65_536;
const MODEL_LIMIT: u64 = 4 * 1024 * 1024 * 1024;

/// Trusted coordinator configuration, never decoded from a candidate response.
#[derive(Clone, Debug)]
pub struct FilePin {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}
#[derive(Clone, Debug)]
pub struct Daemon {
    pub pid: u32,
    pub start_ticks: u64,
    pub boot_id: String,
    pub executable_sha256: String,
    pub executable_bytes: u64,
}
#[derive(Clone, Debug)]
pub struct Profile {
    pub model: String,
    pub manifest: FilePin,
    pub blobs: PathBuf,
    pub client: FilePin,
    pub daemon: Daemon,
    pub directory: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Profile,
    Subject,
    Deadline,
    Cancelled,
    Json,
    Identity,
    Usage,
    Response,
    Process,
    Contract(ContractError),
}
/// This is provider observation, distinct from the HTTP client's process custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderState {
    NotDispatched,
    ObservedComplete,
    Unknown,
}
#[derive(Debug)]
pub struct Exchange {
    pub operation: &'static str,
    pub result: Result<ProcessReport, process::Refusal>,
}
#[derive(Debug)]
pub struct Run<'a> {
    pub contract: Contract<'a>,
    pub exchanges: Vec<Exchange>,
    pub provider: ProviderState,
    pub error: Option<Error>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    #[serde(rename = "mediaType")]
    media_type: String,
    config: Blob,
    layers: Vec<Blob>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Blob {
    #[serde(rename = "mediaType")]
    media_type: String,
    digest: String,
    size: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelConfig {
    model_format: String,
    model_family: String,
    model_families: Vec<String>,
    model_type: String,
    file_type: String,
    architecture: String,
    os: String,
    rootfs: RootFs,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RootFs {
    #[serde(rename = "type")]
    kind: String,
    diff_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Version {
    version: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Details {
    parent_model: String,
    format: String,
    family: String,
    families: Vec<String>,
    parameter_size: String,
    quantization_level: String,
}
impl Details {
    fn local_llama(&self) -> bool {
        self.parent_model.is_empty()
            && self.format == "gguf"
            && self.family == "llama"
            && self.families == ["llama"]
            && self.parameter_size == "3.2B"
            && self.quantization_level == "Q4_K_M"
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalogue {
    models: Vec<CatalogueModel>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogueModel {
    name: String,
    model: String,
    modified_at: String,
    size: u64,
    digest: String,
    details: Details,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Loaded {
    models: Vec<LoadedModel>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoadedModel {
    name: String,
    model: String,
    size: u64,
    digest: String,
    details: Details,
    expires_at: String,
    size_vram: u64,
    context_length: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Generated {
    model: String,
    created_at: String,
    response: String,
    done: bool,
    done_reason: String,
    #[serde(rename = "total_duration")]
    _total_duration: Option<u64>,
    #[serde(rename = "load_duration")]
    _load_duration: Option<u64>,
    prompt_eval_count: u64,
    #[serde(rename = "prompt_eval_duration")]
    _prompt_eval_duration: Option<u64>,
    eval_count: u64,
    #[serde(rename = "eval_duration")]
    _eval_duration: Option<u64>,
}

fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    if bytes.is_empty() || bytes.len() > FRAME_LIMIT {
        return Err(Error::Json);
    }
    serde_json::from_slice(bytes).map_err(|_| Error::Json)
}
fn tick(deadline: Instant, cancelled: &AtomicBool) -> Result<(), Error> {
    if cancelled.load(Ordering::Acquire) {
        Err(Error::Cancelled)
    } else if Instant::now() >= deadline {
        Err(Error::Deadline)
    } else {
        Ok(())
    }
}
fn digest_text(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut value = String::from("sha256:");
    for byte in bytes {
        let _ = write!(value, "{byte:02x}");
    }
    value
}
fn hash_file(pin: &FilePin, deadline: Instant, cancelled: &AtomicBool) -> Result<(), Error> {
    if pin.bytes == 0 || pin.bytes > MODEL_LIMIT || Sha256Digest::parse(&pin.sha256).is_err() {
        return Err(Error::Subject);
    }
    let mut file = File::open(&pin.path).map_err(|_| Error::Subject)?;
    let before = file.metadata().map_err(|_| Error::Subject)?;
    if !before.is_file() || before.len() != pin.bytes {
        return Err(Error::Subject);
    }
    let mut hash = Sha256::new();
    let mut size = 0_u64;
    loop {
        tick(deadline, cancelled)?;
        let mut buffer = [0_u8; 8192];
        let count = file.read(&mut buffer).map_err(|_| Error::Subject)?;
        if count == 0 {
            break;
        }
        size += u64::try_from(count).map_err(|_| Error::Subject)?;
        if size > pin.bytes {
            return Err(Error::Subject);
        }
        hash.update(&buffer[..count]);
    }
    let after = file.metadata().map_err(|_| Error::Subject)?;
    if size != pin.bytes
        || digest_text(&hash.finalize()) != pin.sha256
        || (
            before.dev(),
            before.ino(),
            before.mtime(),
            before.mtime_nsec(),
            before.ctime(),
            before.ctime_nsec(),
        ) != (
            after.dev(),
            after.ino(),
            after.mtime(),
            after.mtime_nsec(),
            after.ctime(),
            after.ctime_nsec(),
        )
    {
        return Err(Error::Subject);
    }
    Ok(())
}
fn small(path: &Path) -> Result<Vec<u8>, Error> {
    let file = File::open(path).map_err(|_| Error::Subject)?;
    let mut bytes = Vec::new();
    file.take(u64::try_from(FRAME_LIMIT + 1).map_err(|_| Error::Subject)?)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Subject)?;
    if bytes.len() > FRAME_LIMIT {
        return Err(Error::Subject);
    }
    Ok(bytes)
}
fn canonical(path: &Path) -> Result<(), Error> {
    if !path.is_absolute() || fs::canonicalize(path).map_err(|_| Error::Subject)? != path {
        return Err(Error::Subject);
    }
    Ok(())
}
fn daemon(profile: &Profile, deadline: Instant, cancelled: &AtomicBool) -> Result<(), Error> {
    let expected = &profile.daemon;
    let path = PathBuf::from(format!("/proc/{}/stat", expected.pid));
    let before = small(&path)?;
    let parse_start = |bytes: &[u8]| -> Result<u64, Error> {
        let value = std::str::from_utf8(bytes).map_err(|_| Error::Identity)?;
        let (_, tail) = value.rsplit_once(')').ok_or(Error::Identity)?;
        tail.split_whitespace()
            .nth(19)
            .ok_or(Error::Identity)?
            .parse()
            .map_err(|_| Error::Identity)
    };
    if expected.pid == 0
        || expected.start_ticks == 0
        || parse_start(&before)? != expected.start_ticks
        || String::from_utf8(small(Path::new("/proc/sys/kernel/random/boot_id"))?)
            .map_err(|_| Error::Identity)?
            .trim()
            != expected.boot_id
    {
        return Err(Error::Identity);
    }
    hash_file(
        &FilePin {
            path: PathBuf::from(format!("/proc/{}/exe", expected.pid)),
            sha256: expected.executable_sha256.clone(),
            bytes: expected.executable_bytes,
        },
        deadline,
        cancelled,
    )?;
    if parse_start(&small(&path)?)? != expected.start_ticks {
        return Err(Error::Identity);
    }
    Ok(())
}
fn subject(profile: &Profile, deadline: Instant, cancelled: &AtomicBool) -> Result<(), Error> {
    for path in [
        &profile.manifest.path,
        &profile.blobs,
        &profile.client.path,
        &profile.directory,
    ] {
        canonical(path)?;
    }
    let directory = fs::metadata(&profile.directory).map_err(|_| Error::Subject)?;
    if !directory.is_dir()
        || directory.uid() != rustix::process::geteuid().as_raw()
        || directory.mode() & 0o777 != 0o700
    {
        return Err(Error::Profile);
    }
    hash_file(&profile.client, deadline, cancelled)?;
    hash_file(&profile.manifest, deadline, cancelled)?;
    let m: Manifest = parse(&small(&profile.manifest.path)?)?;
    if m.schema_version != 2
        || m.media_type != "application/vnd.docker.distribution.manifest.v2+json"
        || m.config.media_type != "application/vnd.docker.container.image.v1+json"
        || m.layers.is_empty()
        || m.layers.len() > 16
    {
        return Err(Error::Subject);
    }
    let mut total = 0_u64;
    for blob in std::iter::once(&m.config).chain(&m.layers) {
        Sha256Digest::parse(&blob.digest).map_err(|_| Error::Subject)?;
        total = total.checked_add(blob.size).ok_or(Error::Subject)?;
        if total > MODEL_LIMIT {
            return Err(Error::Subject);
        }
        let path = profile.blobs.join(blob.digest.replace(':', "-"));
        canonical(&path)?;
        hash_file(
            &FilePin {
                path,
                sha256: blob.digest.clone(),
                bytes: blob.size,
            },
            deadline,
            cancelled,
        )?;
    }
    let config: ModelConfig = parse(&small(
        &profile.blobs.join(m.config.digest.replace(':', "-")),
    )?)?;
    if config.model_format != "gguf"
        || config.model_family != "llama"
        || config.model_families != ["llama"]
        || config.model_type != "3.2B"
        || config.file_type != "Q4_K_M"
        || config.architecture != "amd64"
        || config.os != "linux"
        || config.rootfs.kind != "layers"
        || config.rootfs.diff_ids
            != m.layers
                .iter()
                .map(|b| b.digest.clone())
                .collect::<Vec<_>>()
    {
        return Err(Error::Subject);
    }
    daemon(profile, deadline, cancelled)
}

fn clean(report: &ProcessReport) -> bool {
    report.exit_code == Some(0)
        && report.signal.is_none()
        && report.interruption.is_none()
        && report.leader_reaped
        && report.process_group_settled
        && report.pending.is_none()
        && report.stdout.eof
        && report.stderr.eof
        && !report.stdout.failed
        && !report.stderr.failed
        && !report.stdout.truncated
        && !report.stderr.truncated
        && report.stderr.bytes.is_empty()
}
fn exchange(
    run: &mut Run<'_>,
    profile: &Profile,
    operation: &'static str,
    input: Vec<u8>,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, Error> {
    tick(deadline, cancelled)?;
    let generation = operation == "generate";
    let seconds = if generation { 180 } else { 5 };
    let until = deadline.min(Instant::now() + Duration::from_secs(seconds));
    let mut arguments = vec![
        "-q".into(),
        "--fail-with-body".into(),
        "--silent".into(),
        "--show-error".into(),
        "--max-time".into(),
        seconds.to_string().into(),
        "--connect-timeout".into(),
        "2".into(),
        "--max-redirs".into(),
        "0".into(),
        "--noproxy".into(),
        "*".into(),
        "--proto".into(),
        "=http".into(),
    ];
    if generation {
        arguments.extend([
            "--header".into(),
            "Content-Type: application/json".into(),
            "--data-binary".into(),
            "@-".into(),
        ]);
        run.provider = ProviderState::Unknown;
    }
    arguments.push(format!("{ENDPOINT}{operation}").into());
    let spec = ProcessSpec {
        executable: profile.client.path.clone(),
        arguments,
        directory: profile.directory.clone(),
        environment: vec![("LC_ALL".into(), "C".into())],
        input,
        stream_limit: FRAME_LIMIT,
    };
    let result = process::run(&spec, until, cancelled);
    let raw = result
        .as_ref()
        .ok()
        .filter(|r| clean(r))
        .map(|r| r.stdout.bytes.clone());
    run.exchanges.push(Exchange { operation, result });
    raw.ok_or(Error::Process)
}
fn identity(
    run: &mut Run<'_>,
    profile: &Profile,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<Identity, Error> {
    let version: Version = parse(&exchange(
        run,
        profile,
        "version",
        vec![],
        deadline,
        cancelled,
    )?)?;
    if version.version != "0.0.0" {
        return Err(Error::Identity);
    }
    let tags: Catalogue = parse(&exchange(
        run,
        profile,
        "tags",
        vec![],
        deadline,
        cancelled,
    )?)?;
    if tags.models.len() > 64 {
        return Err(Error::Identity);
    }
    let digest = profile
        .manifest
        .sha256
        .strip_prefix("sha256:")
        .ok_or(Error::Identity)?;
    let matching: Vec<_> = tags
        .models
        .iter()
        .filter(|m| m.name == profile.model)
        .collect();
    if matching.len() != 1 {
        return Err(Error::Identity);
    }
    let selected = matching[0];
    if selected.model != profile.model
        || selected.digest != digest
        || !selected.details.local_llama()
        || selected.size == 0
        || selected.size > MODEL_LIMIT
        || selected.modified_at.is_empty()
        || selected.modified_at.len() > 64
    {
        return Err(Error::Identity);
    }
    let raw = exchange(run, profile, "ps", vec![], deadline, cancelled)?;
    let loaded: Loaded = parse(&raw)?;
    if loaded.models.len() > 64 {
        return Err(Error::Identity);
    }
    let matching: Vec<_> = loaded
        .models
        .iter()
        .filter(|m| m.digest == digest)
        .collect();
    if matching.len() != 1 {
        return Err(Error::Identity);
    }
    let actual = matching[0];
    if actual.name != actual.model
        || actual.model.is_empty()
        || actual.model.len() > 256
        || !actual.details.local_llama()
        || actual.size == 0
        || actual.size > 8 * 1024 * 1024 * 1024
        || actual.size_vram > actual.size
        || actual.context_length != 512
        || actual.expires_at.is_empty()
        || actual.expires_at.len() > 64
    {
        return Err(Error::Identity);
    }
    Ok(Identity {
        selection: super::Selection {
            provider: PROVIDER.into(),
            model: profile.model.clone(),
            effort: None,
        },
        adapter_profile: PROFILE.into(),
        runtime_instance: format!(
            "{}:{}:{}",
            profile.daemon.boot_id, profile.daemon.pid, profile.daemon.start_ticks
        ),
        origin: IdentityOrigin::RuntimeReadback,
        provider_model: Some(actual.model.clone()),
        provider_revision: Some(profile.manifest.sha256.clone()),
        raw,
    })
}
fn elapsed(origin: Instant) -> Result<u64, Error> {
    u64::try_from(origin.elapsed().as_millis()).map_err(|_| Error::Deadline)
}
fn emit(
    run: &mut Run<'_>,
    invocation: super::Invocation<'_>,
    sequence: u64,
    event: Event,
    origin: Instant,
) -> Result<(), Error> {
    run.contract
        .observe(
            Envelope {
                invocation,
                sequence,
                event,
            },
            elapsed(origin)?,
        )
        .map_err(Error::Contract)
}

/// One prepared local-only request, with actual metadata/subject readback before
/// and after generation. The caller supplies its original task clock and work
/// deadline; neither is reconstructed from elapsed observations.
///
/// # Errors
/// Pre-effect configuration/capability failures return Err. Once any HTTP child
/// is attempted, Run retains every raw report and error, including pending child
/// custody. The caller must keep and settle any pending process owner.
pub fn execute<'a>(
    request: &Request<'a>,
    profile: &Profile,
    origin: Instant,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<Run<'a>, Error> {
    let now = Instant::now();
    if origin > now || deadline <= now || deadline > origin + Duration::from_mins(15) {
        return Err(Error::Deadline);
    }
    tick(deadline, cancelled)?;
    if request.selection.provider != PROVIDER
        || request.selection.model != profile.model
        || request.selection.effort.is_some()
        || request.adapter_profile != PROFILE
        || profile.model.is_empty()
        || profile.model.len() > 256
        || !profile
            .model
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:./".contains(&b))
    {
        return Err(Error::Profile);
    }
    let mut contract = Contract::new(request.clone()).map_err(Error::Contract)?;
    contract
        .launch(
            Capabilities::new(&[Feature::FinalOutput, Feature::Identity, Feature::Usage]),
            elapsed(origin)?,
        )
        .map_err(Error::Contract)?;
    subject(profile, deadline, cancelled)?;
    let mut run = Run {
        contract,
        exchanges: vec![],
        provider: ProviderState::NotDispatched,
        error: None,
    };
    let result = execute_inner(&mut run, request, profile, origin, deadline, cancelled);
    if let Err(error) = result {
        if cancelled.load(Ordering::Acquire) || Instant::now() >= deadline {
            let reason = if cancelled.load(Ordering::Acquire) {
                CancelReason::Operator
            } else {
                CancelReason::Deadline
            };
            if let Ok(at) = elapsed(origin) {
                let _ = run.contract.cancel(reason, at);
            }
        }
        run.contract.transport_lost();
        run.error = Some(error);
    }
    Ok(run)
}
fn execute_inner(
    run: &mut Run<'_>,
    request: &Request<'_>,
    profile: &Profile,
    origin: Instant,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<(), Error> {
    let before = identity(run, profile, deadline, cancelled)?;
    daemon(profile, deadline, cancelled)?;
    let input = serde_json::to_vec(
        &json!({"model":profile.model,"prompt":request.prompt,"stream":false,"raw":true,
        "truncate":false,"shift":false,"keep_alive":60,"options":{"num_ctx":512,"num_predict":64}}),
    )
    .map_err(|_| Error::Json)?;
    let raw = exchange(run, profile, "generate", input, deadline, cancelled)?;
    let value: Generated = parse(&raw)?;
    if value.model != profile.model
        || value.created_at.is_empty()
        || value.created_at.len() > 64
        || !value.done
    {
        return Err(Error::Response);
    }
    if value.prompt_eval_count > 512 || value.eval_count > 64 {
        return Err(Error::Usage);
    }
    let (finish, terminal) = match value.done_reason.as_str() {
        "stop" => (Finish::Stop, Terminal::Completed),
        "length" => (Finish::Length, Terminal::Truncated),
        _ => return Err(Error::Response),
    };
    // Duration fields retain their actual optional values in raw, never replace
    // the owner clock or become an invented total/currency conversion.
    let after = identity(run, profile, deadline, cancelled)?;
    if before.selection != after.selection
        || before.runtime_instance != after.runtime_instance
        || before.provider_model != after.provider_model
        || before.provider_revision != after.provider_revision
    {
        return Err(Error::Identity);
    }
    subject(profile, deadline, cancelled)?;
    let candidate = Candidate {
        text: value.response,
        has_tool_proposals: false,
        finish,
        identity: Some(after.clone()),
        usage: Usage::Reported {
            provider: PROVIDER.into(),
            scope: UsageScope::Invocation,
            form: UsageForm::Cumulative,
            stage: UsageStage::Final,
            input: Some(value.prompt_eval_count),
            output: Some(value.eval_count),
            total: None,
            raw: raw.clone(),
        },
        raw,
    };
    emit(run, request.invocation, 1, Event::Identity(after), origin)?;
    emit(run, request.invocation, 2, Event::Final(candidate), origin)?;
    emit(
        run,
        request.invocation,
        3,
        Event::Terminal(terminal),
        origin,
    )?;
    run.provider = ProviderState::ObservedComplete;
    Ok(())
}
