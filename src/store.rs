// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/store.rs
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
// Readiness binding: HEE3-READINESS-001; SHA-256 3bcde91b4617c1a38bcbb97c36bddbc9e999c7692a69b0ba21caec9f559fe250; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-02, R90-03, R90-04, R90-05, R90-09, R90-10; resolved contracts RC02, RC03, RC04, RC05; runtime proof pending; original task DAG controls.
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
// Readiness binding: HEE3-READINESS-001; SHA-256 3bcde91b4617c1a38bcbb97c36bddbc9e999c7692a69b0ba21caec9f559fe250; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-02, R90-05, R90-06, R90-08, R90-09, R90-10; resolved contracts RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-store; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-store (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-store
// Owns: SQLite transactions, migrations and outbox persistence
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/store.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T04, T06, T07, T14, T15, T17, T18, T19, T20, T25, T26, T27
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K1)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB03)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [task TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
// [implementation support task TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
// [task TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
// [task TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
// [implementation support task TASK-T13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T13)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [separate reference example EX-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-store)
// [flow FLOW-F02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F02)
// [handbook HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [handbook HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-store)
// [plan SEC-architecture](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-architecture)
// [plan SEC-deployment](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-deployment)
// [plan SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-runtime](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-runtime)
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
// [schematic SC-SC10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC10)
// [schematic SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
// [schematic SC-SC12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC12)
// [schematic SC-SC13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC13)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [source SRC-A03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A03)
// [source SRC-A04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A04)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-store)
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
// [resolved module contract RC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FRC06)
// [adopted readiness convention Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex)
// [readiness criterion cluster F2](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF2)
// [readiness criterion cluster F3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF3)
// [readiness criterion cluster F4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF4)
// [readiness criterion cluster F5](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF5)
// [readiness criterion cluster F6](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-02)
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
// [completion and operational convention DONE-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-store)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
// [applied learning LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
// [diary evidence source DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
// Applicable learning IDs: LRN04, LRN05, LRN08, LRN13; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What Prototyping Is For](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20Prototyping%20Is%20For)
// [Working in Sandboxes on Kinoite](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20in%20Sandboxes%20on%20Kinoite)
// HEE3-ANCHORS-END

//! Single SQLite writer and immutable artifact ownership.
//!
//! Callers are trusted coordinator modules. This facade supplies persistence,
//! not peer authentication, verifier authority, provider dispatch or admission.

mod artifact;
mod backup;
mod reconciliation;
mod recovery;
mod roster;
mod schema;
mod staging;
mod verification;
pub use staging::ArtifactStaging;
pub use verification::{Verification, VerificationVerdict};

pub use roster::{RequestSource, RosterAttempt, RosterSnapshot, RosterStart};

pub use artifact::Object;
pub use backup::{BackupReport, RestoreStatus};

use crate::contracts::{Generation, Sha256Digest, UuidV4};
use artifact::Directory;
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Invalid,
    Forbidden,
    Bound,
    Conflict,
    NotFound,
    Cancelled,
    Outstanding,
    Budget,
    Deadline,
    Locked,
    Custody,
    Corrupt,
    UnsupportedSchema,
    Runtime,
    UncertainCommit,
    RecoveryRequired,
    InspectionOnly,
    Commit(rusqlite::Error),
    Cleanup {
        original: Box<Error>,
        failure: Box<Error>,
    },
    Rollback {
        original: Box<Error>,
        failure: rusqlite::Error,
    },
    Io(std::io::Error),
    Os(rustix::io::Errno),
    Sqlite(rusqlite::Error),
    Encoding(serde_json::Error),
    #[cfg(test)]
    Injected(String),
}

/// How long the store's single-writer lock is waited for when it is found held. A fork-to-exec
/// window in a multi-threaded process holds a duplicated descriptor for microseconds to a few
/// milliseconds; 250 ms covers that under load while a genuine duplicate start still refuses
/// promptly rather than after the caller's whole deadline.
pub const LOCK_SETTLE: Duration = Duration::from_millis(250);

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<rustix::io::Errno> for Error {
    fn from(value: rustix::io::Errno) -> Self {
        Self::Os(value)
    }
}
impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Encoding(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CutPoint {
    MigrationWrite,
    RosterWrite,
    RosterPin,
    RosterObservation,
    TaskWrite,
    AttemptWrite,
    ObjectWrite,
    ObjectSync,
    ObjectRename,
    ObjectDirectorySync,
    ManifestPublished,
    AcceptanceWrite,
    BeforeCommit,
    AfterCommit,
    BackupCopied,
    BackupObject,
    BackupManifest,
}

// Test-only cut points keep the same fallible call sites in production.
#[cfg_attr(not(test), allow(clippy::unnecessary_wraps))]
fn check_point(fault: Option<CutPoint>, point: CutPoint) -> Result<()> {
    #[cfg(test)]
    if fault == Some(point) {
        return Err(Error::Injected(format!("{point:?}")));
    }
    let _ = (fault, point);
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    digest_text(&Sha256::digest(bytes))
}

/// Stable principal supplied by the authenticated owner, never decoded from a body.
#[derive(Clone, Debug)]
pub struct Principal {
    uid: u32,
    role: String,
}
impl Principal {
    /// # Errors
    /// Refuses an empty, oversized or non-identifier configured role.
    pub fn new(uid: u32, role: &str) -> Result<Self> {
        if role.is_empty()
            || role.len() > 64
            || !role
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(Error::Invalid);
        }
        Ok(Self {
            uid,
            role: role.to_owned(),
        })
    }
    /// Whether this is the principal with `uid` and the configured `role`: how a record naming
    /// its principal (a grant) is matched without exposing either field.
    #[must_use]
    pub fn is(&self, uid: u32, role: &str) -> bool {
        self.uid == uid && self.role == role
    }
    fn recipient(&self) -> String {
        format!("{}:{}", self.uid, self.role)
    }
}

/// Fixed offline allocation in milliseconds. Provider currency remains zero.
#[derive(Clone, Copy, Debug)]
pub struct Allocation {
    pub limit_ms: u64,
    pub work_ms: u64,
    pub verify_ms: u64,
}
impl Allocation {
    fn valid(self) -> bool {
        self.limit_ms > 0
            && self.limit_ms <= 1_200_000
            && self.verify_ms > 0
            && self
                .work_ms
                .checked_add(self.verify_ms)
                .is_some_and(|sum| sum <= self.limit_ms)
    }
}

/// One immutable admission. Canonical control encoding belongs to the caller.
#[derive(Clone, Copy)]
pub struct Submission<'a> {
    pub principal: &'a Principal,
    pub key: UuidV4<'a>,
    pub task: UuidV4<'a>,
    pub event: UuidV4<'a>,
    pub request_bytes: &'a [u8],
    pub criteria: Sha256Digest<'a>,
    pub allocation: Allocation,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Admission {
    pub task: String,
    pub generation: String,
    pub epoch: String,
    pub sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskHead {
    pub id: String,
    pub generation: String,
    pub state: String,
    pub cancellation: bool,
    pub accepted_event: Option<String>,
    pub criteria: String,
    pub spent_ms: u64,
    pub reserved_work_ms: u64,
    pub reserved_verify_ms: u64,
}

/// Attempt generation is independent of the task's compare-and-set revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptHead {
    pub id: String,
    pub task_generation: String,
    pub generation: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Effect {
    None,
    Committed,
    Pending,
    Unknown,
}
impl Effect {
    const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Committed => "committed",
            Self::Pending => "pending",
            Self::Unknown => "unknown",
        }
    }
}

/// Cumulative work accounting and separately observed effect/workspace custody.
#[derive(Clone, Copy, Debug)]
pub struct Settlement {
    pub effect: Effect,
    pub used_ms: Option<u64>,
    pub cleanup_settled: bool,
    pub ready_to_verify: bool,
}

/// A task revision and exact active attempt, supplied by its single owner.
pub struct Expected<'a> {
    pub task: UuidV4<'a>,
    pub task_generation: Generation,
    pub attempt: UuidV4<'a>,
    pub attempt_generation: Generation,
}

#[derive(Debug)]
pub struct Store {
    connection: Connection,
    root: Directory,
    generation: Directory,
    objects: Directory,
    lock: File,
    epoch: String,
    poisoned: bool,
    inspection_only: bool,
    clock: roster::ReceiverClock,
    #[cfg(test)]
    fault: Option<CutPoint>,
}

impl Store {
    /// Open the explicitly selected generation under an existing private root.
    /// Only an empty newly created database can receive migration 1.
    /// # Errors
    /// Refuses duplicate custody, changed runtime/history/schema, invalid paths or expired work.
    pub fn open(
        root: &Path,
        generation: UuidV4<'_>,
        epoch: UuidV4<'_>,
        create: bool,
        deadline: Instant,
    ) -> Result<Self> {
        Self::open_inner(root, generation, epoch, create, deadline, None)
    }

    fn open_inner(
        root: &Path,
        generation: UuidV4<'_>,
        epoch: UuidV4<'_>,
        create: bool,
        deadline: Instant,
        fault: Option<CutPoint>,
    ) -> Result<Self> {
        Self::open_access(root, generation, epoch, create, deadline, fault, false)
    }

    /// Open existing durable state for inspection only, including reconciliation mode.
    /// This preserves the same sole lock and schema checks; WAL/profile effects
    /// remain possible, but mutations, artifact publication and dispatch are denied.
    /// # Errors
    /// Refuses missing state, invalid identity/schema/path, lock contention or deadline.
    pub fn open_inspection(
        root: &Path,
        generation: UuidV4<'_>,
        epoch: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Self> {
        Self::open_access(root, generation, epoch, false, deadline, None, true)
    }

    fn open_access(
        root: &Path,
        generation: UuidV4<'_>,
        epoch: UuidV4<'_>,
        create: bool,
        deadline: Instant,
        fault: Option<CutPoint>,
        inspection_only: bool,
    ) -> Result<Self> {
        remaining(deadline)?;
        schema::runtime(deadline)?;
        let root = Directory::root(root)?;
        let lock = root.lock(deadline)?;
        let generations = root.child("generations", create)?;
        let generation_dir = generations.child(generation.as_str(), create)?;
        let created = if create {
            match generation_dir.create_file("ledger.sqlite3") {
                Ok(file) => {
                    file.sync_all()?;
                    generation_dir.file.sync_all()?;
                    true
                }
                Err(Error::Os(error)) if error == rustix::io::Errno::EXIST => false,
                Err(error) => return Err(error),
            }
        } else {
            false
        };
        generation_dir.regular("ledger.sqlite3")?;
        let mut connection = Connection::open_with_flags(
            generation_dir.path.join("ledger.sqlite3"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
                | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        connection.busy_timeout(remaining(deadline)?.min(Duration::from_secs(5)))?;
        schema::initialize(
            &mut connection,
            created,
            generation.as_str(),
            epoch.as_str(),
            fault,
            deadline,
        )?;
        schema::profile(&connection, deadline)?;
        schema::validate(&connection, generation.as_str(), deadline)?;
        let (actual_epoch, mode): (String, String) = connection.query_row(
            "SELECT epoch,mode FROM ledger_meta WHERE singleton=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if !matches!(mode.as_str(), "normal" | "reconciliation") {
            return Err(Error::Corrupt);
        }
        if !inspection_only && mode != "normal" {
            return Err(Error::RecoveryRequired);
        }
        if inspection_only {
            connection.execute_batch("PRAGMA query_only=ON;")?;
        }
        if actual_epoch != epoch.as_str() {
            return Err(Error::Conflict);
        }
        let objects = generation_dir
            .child("objects", create)?
            .child("sha256", create)?;
        Ok(Self {
            connection,
            root,
            generation: generation_dir,
            objects,
            lock,
            epoch: actual_epoch,
            poisoned: false,
            inspection_only,
            clock: roster::ReceiverClock::new(deadline)?,
            #[cfg(test)]
            fault,
        })
    }

    // The production build contains no injectable fault field.
    #[cfg_attr(not(test), allow(clippy::unused_self))]
    fn fault(&self) -> Option<CutPoint> {
        #[cfg(test)]
        {
            self.fault
        }
        #[cfg(not(test))]
        {
            None
        }
    }

    fn require_writable(&self, deadline: Instant) -> Result<()> {
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        schema::bound(&self.connection, deadline)?;
        require_normal(&self.connection, &self.epoch, self.inspection_only)
    }

    fn transaction<T>(
        &mut self,
        deadline: Instant,
        action: impl FnOnce(&Transaction<'_>) -> Result<T>,
    ) -> Result<T> {
        if self.poisoned {
            return Err(Error::UncertainCommit);
        }
        if self.inspection_only {
            return Err(Error::InspectionOnly);
        }
        self.connection
            .busy_timeout(remaining(deadline)?.min(Duration::from_secs(5)))?;
        schema::bound(&self.connection, deadline)?;
        let fault = self.fault();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = require_normal(&tx, &self.epoch, self.inspection_only)
            .and_then(|()| action(&tx))
            .and_then(|value| {
                remaining(deadline)?;
                check_point(fault, CutPoint::BeforeCommit)?;
                Ok(value)
            });
        let value = match result {
            Ok(value) => value,
            Err(error) => {
                if let Err(failure) = tx.rollback() {
                    self.poisoned = true;
                    return Err(Error::Rollback {
                        original: Box::new(error),
                        failure,
                    });
                }
                return Err(error);
            }
        };
        if let Err(error) = tx.commit() {
            self.poisoned = true;
            return Err(Error::Commit(error));
        }
        if check_point(fault, CutPoint::AfterCommit).is_err() {
            self.poisoned = true;
            return Err(Error::UncertainCommit);
        }
        Ok(value)
    }

    /// Persist admission, conservative allocations and its event atomically.
    /// # Errors
    /// Same principal/key with changed bytes conflicts. Exact replay returns the original result.
    pub fn submit(&mut self, input: Submission<'_>, deadline: Instant) -> Result<Admission> {
        if input.request_bytes.is_empty()
            || input.request_bytes.len() > 1_048_576
            || !input.allocation.valid()
        {
            return Err(Error::Bound);
        }
        let request_digest = digest(input.request_bytes);
        let fault = self.fault();
        let epoch = self.epoch.clone();
        self.transaction(deadline, |tx| {
            let prior: Option<(String, Vec<u8>)> = tx.query_row(
                "SELECT request_digest,result FROM operations WHERE principal_uid=? AND principal_role=? AND action='task.submit' AND version=1 AND request_key=?",
                params![input.principal.uid,input.principal.role,input.key.as_str()], |row| Ok((row.get(0)?,row.get(1)?))).optional()?;
            if let Some((prior_digest, result)) = prior {
                if prior_digest != request_digest { return Err(Error::Conflict); }
                return Ok(serde_json::from_slice(&result)?);
            }
            tx.execute("INSERT INTO tasks(id,principal_uid,principal_role,spec,criteria_digest,generation,state,limit_ms,reserved_work_ms,reserved_verify_ms) VALUES(?,?,?,?,?,'1','admitted',?,?,?)",
                params![input.task.as_str(),input.principal.uid,input.principal.role,input.request_bytes,input.criteria.as_str(),number(input.allocation.limit_ms)?,number(input.allocation.work_ms)?,number(input.allocation.verify_ms)?])?;
            check_point(fault, CutPoint::TaskWrite)?;
            let sequence = event(tx,input.event.as_str(),input.task.as_str(),"1","admitted")?;
            let result = Admission { task:input.task.as_str().to_owned(),generation:"1".to_owned(),epoch,sequence };
            tx.execute("INSERT INTO operations(principal_uid,principal_role,action,version,request_key,request_digest,resource_id,result) VALUES(?,?,'task.submit',1,?,?,?,?)",
                params![input.principal.uid,input.principal.role,input.key.as_str(),request_digest,input.task.as_str(),serde_json::to_vec(&result)?])?;
            Ok(result)
        })
    }

    /// Recover admission using values known before a possibly lost first reply.
    /// # Errors
    /// An unavailable principal-scoped key returns `NotFound` without exposing another principal.
    pub fn get_by_key(
        &self,
        principal: &Principal,
        key: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<TaskHead> {
        schema::bound(&self.connection, deadline)?;
        let id: String = self.connection.query_row("SELECT resource_id FROM operations WHERE principal_uid=? AND principal_role=? AND action='task.submit' AND version=1 AND request_key=?",
            params![principal.uid,principal.role,key.as_str()], |row| row.get(0)).optional()?.ok_or(Error::NotFound)?;
        self.get(
            principal,
            UuidV4::parse(&id).map_err(|_| Error::Corrupt)?,
            deadline,
        )
    }

    /// # Errors
    /// Refuses missing or invisible task identity; this method never grants visibility.
    pub fn get(
        &self,
        principal: &Principal,
        id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<TaskHead> {
        schema::bound(&self.connection, deadline)?;
        self.connection.query_row("SELECT id,generation,state,cancellation,accepted_event,criteria_digest,spent_ms,reserved_work_ms,reserved_verify_ms FROM tasks WHERE id=? AND principal_uid=? AND principal_role=?",
            params![id.as_str(),principal.uid,principal.role], task_row).optional()?.ok_or(Error::NotFound)
    }

    /// Reserve a unique attempt before dispatch. Previous uncertain attempts block reuse.
    /// # Errors
    /// Refuses stale task revision, cancellation, exhausted allocations or unsettled ownership.
    pub fn begin_attempt(
        &mut self,
        task: UuidV4<'_>,
        expected: Generation,
        attempt: UuidV4<'_>,
        event_id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<AttemptHead> {
        let fault = self.fault();
        self.transaction(deadline, |tx| {
            begin_attempt_in(tx, task, expected, attempt, event_id, fault)
        })
    }

    /// Record independent settlement and measured work cost, retaining unknown liabilities.
    /// `ready_to_verify` means worker custody settled; it is not verifier success.
    /// # Errors
    /// Refuses a stale identity or cost above the retained conservative allocation.
    pub fn settle_attempt(
        &mut self,
        expected: &Expected<'_>,
        observation: Settlement,
        event_id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<String> {
        let Settlement {
            effect,
            used_ms,
            cleanup_settled,
            ready_to_verify,
        } = observation;
        self.transaction(deadline,|tx| {
            let head=head(tx,expected.task.as_str())?;same_generation(&head,expected.task_generation)?;
            require_attempt(tx,expected,false)?;
            let prior:(String,String,Option<u64>)=tx.query_row("SELECT effect,cleanup,used_ms FROM attempts WHERE id=?",[expected.attempt.as_str()],|row|Ok((row.get(0)?,row.get(1)?,read_optional_number(row,2)?)))?;
            if prior.2.is_some_and(|known|used_ms.is_none_or(|next|next<known))
                || (prior.1=="settled"&&!cleanup_settled)
                || (prior.0=="committed"&&effect!=Effect::Committed) {
                return Err(Error::Conflict);
            }
            if used_ms.is_some_and(|used|used>head.reserved_work_ms) {return Err(Error::Budget);}
            let settled=matches!(effect,Effect::None|Effect::Committed)&&used_ms.is_some()&&cleanup_settled;
            let generation=next(expected.task_generation)?;
            let state=if !settled {"effect_unknown"} else if head.cancellation {"cancellation_requested"} else if ready_to_verify {"verifying"} else {"repair_pending"};
            tx.execute("UPDATE attempts SET state=?,effect=?,cleanup=?,used_ms=? WHERE id=?",params![if settled {"settled"}else{"unknown"},effect.name(),if cleanup_settled{"settled"}else{"unknown"},used_ms.map(number).transpose()?,expected.attempt.as_str()])?;
            let used=if settled {used_ms.unwrap_or(0)}else{0};
            tx.execute("UPDATE tasks SET generation=?,state=?,spent_ms=spent_ms+?,reserved_work_ms=reserved_work_ms-? WHERE id=?",params![generation,state,number(used)?,number(used)?,expected.task.as_str()])?;
            event(tx,event_id.as_str(),expected.task.as_str(),&generation,"attempt_observed")?;Ok(generation)
        })
    }

    /// Commit cancellation intent; outstanding effect/usage reservations survive.
    /// # Errors
    /// Refuses stale state. Acceptance already committed remains historical.
    pub fn cancel(
        &mut self,
        task: UuidV4<'_>,
        expected: Generation,
        event_id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<String> {
        self.transaction(deadline,|tx| {
            let head=head(tx,task.as_str())?;same_generation(&head,expected)?;
            let stopped:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM task_stops WHERE task_id=?)",[task.as_str()],|row|row.get(0))?;
            if head.cancellation || head.accepted_event.is_some() || stopped {return Ok(head.generation);}
            let generation=next(expected)?;
            tx.execute("UPDATE tasks SET cancellation=1,state='cancellation_requested',generation=? WHERE id=?",params![generation,task.as_str()])?;
            event(tx,event_id.as_str(),task.as_str(),&generation,"cancellation_requested")?;Ok(generation)
        })
    }

    /// Borrow artifact-only custody while this thread retains the sole ledger owner.
    /// The capability duplicates already held directory and lock descriptors; it
    /// opens no database and cannot outlive the callback. Scoped workers using it
    /// must join before the callback returns. Artifact methods retain their own
    /// deadlines and publication/readback checks.
    ///
    /// The callback's result is returned unchanged, even after expiry, so partial
    /// artifact custody or caller errors are not discarded. This is not deadline
    /// acceptance: the caller must settle and account for the actual outcome.
    ///
    /// # Errors
    /// Refuses expired capability creation, changed directory custody or failed
    /// descriptor duplication before invoking the callback.
    pub fn with_artifact_staging<T>(
        &mut self,
        deadline: Instant,
        work: impl FnOnce(&mut Self, &ArtifactStaging) -> T,
    ) -> Result<T> {
        self.require_writable(deadline)?;
        let staging = ArtifactStaging::from_store(self, deadline)?;
        Ok(work(self, &staging))
    }

    /// Publish bounded immutable bytes before any ledger reference.
    /// # Errors
    /// Refuses unsafe ownership, corrupt existing identity, unsupported atomic publication or I/O failure.
    pub fn publish(
        &self,
        bytes: &[u8],
        staging_id: UuidV4<'_>,
        deadline: Instant,
    ) -> Result<Object> {
        self.require_writable(deadline)?;
        let object = artifact::publish(&self.objects, bytes, staging_id, |point| {
            check_point(self.fault(), point)
        })?;
        remaining(deadline)?;
        Ok(object)
    }

    /// # Errors
    /// Missing/corrupt proof is a current availability error; no task history is altered.
    pub fn read_object(&self, object: &Object, deadline: Instant) -> Result<Vec<u8>> {
        remaining(deadline)?;
        let bytes = artifact::verify(&self.objects, object)?;
        remaining(deadline)?;
        Ok(bytes)
    }

    /// Prepare exact immutable manifest bytes with its event identity already allocated.
    /// The caller's proof is untrusted until the T06 verifier owner authenticates it.
    /// # Errors
    /// Refuses missing objects, stale subjects, duplicate objects or excessive manifest size.
    pub fn prepare_acceptance(
        &self,
        expected: &Expected<'_>,
        event_id: UuidV4<'_>,
        objects: &[Object],
        deadline: Instant,
    ) -> Result<PublishedAcceptance> {
        self.prepare_acceptance_inner(expected, event_id, objects, None, deadline)
    }

    fn prepare_acceptance_inner(
        &self,
        expected: &Expected<'_>,
        event_id: UuidV4<'_>,
        objects: &[Object],
        verification: Option<verification::VerifiedSubject>,
        deadline: Instant,
    ) -> Result<PublishedAcceptance> {
        remaining(deadline)?;
        schema::bound(&self.connection, deadline)?;
        let head = head(&self.connection, expected.task.as_str())?;
        same_generation(&head, expected.task_generation)?;
        // A verified RC04 closure has its existing 4096-object / 64-MiB bound.
        // General trusted-owner acceptance keeps the original 64-object limit.
        let object_limit = if verification.is_some() { 4096 } else { 64 };
        if objects.is_empty() || objects.len() > object_limit {
            return Err(Error::Bound);
        }
        if verification.is_some() {
            let bytes = objects.iter().try_fold(0_u64, |total, object| {
                total.checked_add(object.size).ok_or(Error::Bound)
            })?;
            if bytes > 64 * 1024 * 1024 {
                return Err(Error::Bound);
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for object in objects {
            remaining(deadline)?;
            if !seen.insert(&object.digest) {
                return Err(Error::Invalid);
            }
            self.read_object(object, deadline)?;
        }
        let data = Manifest {
            event: event_id.as_str().to_owned(),
            task: expected.task.as_str().to_owned(),
            task_generation: head.generation,
            attempt: expected.attempt.as_str().to_owned(),
            attempt_generation: expected.attempt_generation.to_string(),
            criteria: head.criteria,
            objects: objects.to_vec(),
            verification,
        };
        let bytes = serde_json::to_vec(&data)?;
        let expected_manifest = Object {
            digest: digest(&bytes),
            size: u64::try_from(bytes.len()).map_err(|_| Error::Bound)?,
        };
        if data.verification.is_some() {
            verified_inventory_capacity(
                &self.connection,
                &data.objects,
                &expected_manifest,
                deadline,
            )?;
        }
        let manifest = self.publish(&bytes, event_id, deadline)?;
        if manifest != expected_manifest {
            return Err(Error::Corrupt);
        }
        check_point(self.fault(), CutPoint::ManifestPublished)?;
        Ok(PublishedAcceptance { manifest, data })
    }

    /// Commit already durable proof reference, historical acceptance and delivery obligation together.
    /// This trusted-owner API does not decide whether a candidate satisfies its criteria.
    /// # Errors
    /// Refuses cancellation, stale subjects, missing proof, unreconciled effects or insufficient verification reservation.
    pub fn accept(
        &mut self,
        published: &PublishedAcceptance,
        verification_ms: u64,
        deadline: Instant,
    ) -> Result<u64> {
        let bytes = self.read_object(&published.manifest, deadline)?;
        if bytes != serde_json::to_vec(&published.data)? {
            return Err(Error::Corrupt);
        }
        for object in &published.data.objects {
            self.read_object(object, deadline)?;
        }
        let data = &published.data;
        let fault = self.fault();
        self.transaction(deadline,|tx| {
            let head=head(tx,&data.task)?;
            if head.generation!=data.task_generation || head.criteria!=data.criteria || head.accepted_event.is_some() {return Err(Error::Conflict);}
            if head.cancellation {return Err(Error::Cancelled);}
            if head.state!="verifying" {return Err(Error::Outstanding);}
            let generation:Generation=head.generation.parse().map_err(|_|Error::Corrupt)?;
            let expected=Expected {task:UuidV4::parse(&data.task).map_err(|_|Error::Corrupt)?,task_generation:generation,attempt:UuidV4::parse(&data.attempt).map_err(|_|Error::Corrupt)?,attempt_generation:data.attempt_generation.parse().map_err(|_|Error::Corrupt)?};
            require_attempt(tx,&expected,true)?;
            if let Some(verified) = &data.verification {
                verification::require_verified(tx, &data.attempt, verified)?;
                if verification_ms != 0 { return Err(Error::Conflict); }
                verified_inventory_capacity(tx, &data.objects, &published.manifest, deadline)?;
            }
            if verification_ms>head.reserved_verify_ms {return Err(Error::Budget);}
            let generation=next(generation)?;
            for object in data.objects.iter().chain(std::iter::once(&published.manifest)) {
                tx.execute("INSERT INTO artifacts VALUES(?,?) ON CONFLICT(digest) DO NOTHING",params![object.digest,number(object.size)?])?;
                let size:u64=tx.query_row("SELECT size FROM artifacts WHERE digest=?",[&object.digest],|row|read_number(row,0))?;
                if size!=object.size {return Err(Error::Corrupt);}
            }
            let sequence=event(tx,&data.event,&data.task,&generation,"accepted")?;
            tx.execute("INSERT INTO acceptances VALUES(?,?,?,?,?,?)",params![data.event,data.task,data.attempt,data.attempt_generation,data.criteria,published.manifest.digest])?;
            for object in &data.objects {tx.execute("INSERT INTO acceptance_objects VALUES(?,?)",params![data.event,object.digest])?;}
            tx.execute("UPDATE tasks SET generation=?,state='accepted',accepted_event=?,spent_ms=spent_ms+?,reserved_work_ms=0,reserved_verify_ms=0 WHERE id=?",params![generation,data.event,number(verification_ms)?,data.task])?;
            check_point(fault,CutPoint::AcceptanceWrite)?;
            let principal=tx.query_row("SELECT principal_uid,principal_role FROM tasks WHERE id=?",[&data.task],|row|Ok(Principal {uid:row.get(0)?,role:row.get(1)?}))?;
            tx.execute("INSERT INTO outbox(event_id,recipient) VALUES(?,?)",params![data.event,principal.recipient()])?;
            Ok(sequence)
        })
    }

    /// Read a bounded ordered committed outbox; delivery does not rerun work.
    /// # Errors
    /// Refuses an oversized page or unavailable database.
    pub fn pending_delivery(
        &self,
        limit: u32,
        deadline: Instant,
    ) -> Result<Vec<(String, String, u64)>> {
        self.require_writable(deadline)?;
        if limit == 0 || limit > 256 {
            return Err(Error::Bound);
        }
        let mut statement=self.connection.prepare("SELECT o.event_id,o.recipient,e.sequence FROM outbox o JOIN events e ON e.id=o.event_id WHERE delivered=0 ORDER BY e.sequence LIMIT ?")?;
        Ok(statement
            .query_map([limit], |row| {
                Ok((row.get(0)?, row.get(1)?, read_number(row, 2)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// # Errors
    /// Refuses an unknown event/recipient; acknowledgement cannot alter task acceptance.
    pub fn acknowledge_delivery(
        &mut self,
        event_id: UuidV4<'_>,
        recipient: &str,
        deadline: Instant,
    ) -> Result<()> {
        self.transaction(deadline, |tx| {
            if tx.execute(
                "UPDATE outbox SET delivered=1 WHERE event_id=? AND recipient=?",
                params![event_id.as_str(), recipient],
            )? != 1
            {
                return Err(Error::NotFound);
            }
            Ok(())
        })
    }
}

fn require_normal(connection: &Connection, epoch: &str, inspection_only: bool) -> Result<()> {
    if inspection_only {
        return Err(Error::InspectionOnly);
    }
    let (actual_epoch, mode): (String, String) = connection.query_row(
        "SELECT epoch,mode FROM ledger_meta WHERE singleton=1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if actual_epoch != epoch {
        return Err(Error::Conflict);
    }
    match mode.as_str() {
        "normal" => Ok(()),
        "reconciliation" => Err(Error::RecoveryRequired),
        _ => Err(Error::Corrupt),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Manifest {
    event: String,
    task: String,
    task_generation: String,
    attempt: String,
    attempt_generation: String,
    criteria: String,
    objects: Vec<Object>,
    verification: Option<verification::VerifiedSubject>,
}

/// A verified acceptance must not itself exceed the existing snapshot object cap.
/// Other registrations and the snapshot's byte bound keep their own semantics.
fn verified_inventory_capacity(
    connection: &Connection,
    objects: &[Object],
    manifest: &Object,
    deadline: Instant,
) -> Result<()> {
    let mut prospective = std::collections::BTreeMap::new();
    let mut statement =
        connection.prepare("SELECT digest,size FROM artifacts ORDER BY digest LIMIT 4097")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, read_number(row, 1)?))
    })?;
    for row in rows {
        remaining(deadline)?;
        let (digest, size) = row?;
        prospective.insert(digest, size);
        if prospective.len() > 4096 {
            return Err(Error::Bound);
        }
    }
    for object in objects.iter().chain(std::iter::once(manifest)) {
        remaining(deadline)?;
        if let Some(previous) = prospective.insert(object.digest.clone(), object.size)
            && previous != object.size
        {
            return Err(Error::Corrupt);
        }
        if prospective.len() > 4096 {
            return Err(Error::Bound);
        }
    }
    Ok(())
}
#[derive(Debug)]
pub struct PublishedAcceptance {
    manifest: Object,
    data: Manifest,
}
impl PublishedAcceptance {
    #[must_use]
    pub const fn object(&self) -> &Object {
        &self.manifest
    }
}

fn remaining(deadline: Instant) -> Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|left| !left.is_zero())
        .ok_or(Error::Deadline)
}
fn next(generation: Generation) -> Result<String> {
    generation
        .next()
        .map(|next| next.to_string())
        .map_err(|_| Error::Bound)
}
fn same_generation(head: &TaskHead, expected: Generation) -> Result<()> {
    if head.generation == expected.to_string() {
        Ok(())
    } else {
        Err(Error::Conflict)
    }
}
fn task_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskHead> {
    Ok(TaskHead {
        id: row.get(0)?,
        generation: row.get(1)?,
        state: row.get(2)?,
        cancellation: row.get(3)?,
        accepted_event: row.get(4)?,
        criteria: row.get(5)?,
        spent_ms: read_number(row, 6)?,
        reserved_work_ms: read_number(row, 7)?,
        reserved_verify_ms: read_number(row, 8)?,
    })
}
fn head(connection: &Connection, id: &str) -> Result<TaskHead> {
    connection.query_row("SELECT id,generation,state,cancellation,accepted_event,criteria_digest,spent_ms,reserved_work_ms,reserved_verify_ms FROM tasks WHERE id=?",[id],task_row).optional()?.ok_or(Error::NotFound)
}
fn event(tx: &Transaction<'_>, id: &str, task: &str, generation: &str, kind: &str) -> Result<u64> {
    tx.execute(
        "INSERT INTO events(id,task_id,generation,kind,body) VALUES(?,?,?,?,?)",
        params![id, task, generation, kind, b"{}".as_slice()],
    )?;
    u64::try_from(tx.last_insert_rowid()).map_err(|_| Error::Bound)
}
fn require_attempt(connection: &Connection, expected: &Expected<'_>, settled: bool) -> Result<()> {
    let row:Option<(String,String,String,Option<u64>)>=connection.query_row("SELECT state,effect,cleanup,used_ms FROM attempts WHERE id=? AND task_id=? AND generation=?",params![expected.attempt.as_str(),expected.task.as_str(),expected.attempt_generation.to_string()],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,read_optional_number(row,3)?))).optional()?;
    match row {
        Some((state, effect, cleanup, used))
            if settled
                && state == "settled"
                && matches!(effect.as_str(), "none" | "committed")
                && cleanup == "settled"
                && used.is_some() =>
        {
            Ok(())
        }
        Some((state, _, _, _)) if !settled && matches!(state.as_str(), "running" | "unknown") => {
            Ok(())
        }
        _ => Err(Error::Outstanding),
    }
}

fn number(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| Error::Bound)
}
fn read_number(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}
fn read_optional_number(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Option<u64>> {
    let value: Option<i64> = row.get(index)?;
    value
        .map(|v| u64::try_from(v).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, v)))
        .transpose()
}
fn digest_text(bytes: &[u8]) -> String {
    let alphabet = b"0123456789abcdef";
    let mut text = String::from("sha256:");
    for &byte in bytes {
        text.push(char::from(alphabet[usize::from(byte >> 4)]));
        text.push(char::from(alphabet[usize::from(byte & 15)]));
    }
    text
}

#[cfg(test)]
#[path = "../tests/t04_store.rs"]
mod tests;

fn begin_attempt_in(
    tx: &Transaction<'_>,
    task: UuidV4<'_>,
    expected: Generation,
    attempt: UuidV4<'_>,
    event_id: UuidV4<'_>,
    fault: Option<CutPoint>,
) -> Result<AttemptHead> {
    let head = head(tx, task.as_str())?;
    same_generation(&head, expected)?;
    if head.cancellation {
        return Err(Error::Cancelled);
    }
    if !matches!(head.state.as_str(), "admitted" | "repair_pending") {
        return Err(Error::Conflict);
    }
    if head.reserved_work_ms == 0 || head.reserved_verify_ms == 0 {
        return Err(Error::Budget);
    }
    let count: u64 = tx.query_row(
        "SELECT count(*) FROM attempts WHERE task_id=?",
        [task.as_str()],
        |row| read_number(row, 0),
    )?;
    if count >= 3 {
        return Err(Error::Bound);
    }
    let outstanding: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM attempts WHERE task_id=? AND state!='settled')",
        [task.as_str()],
        |row| row.get(0),
    )?;
    if outstanding {
        return Err(Error::Outstanding);
    }
    let generation = (count + 1).to_string();
    let task_generation = next(expected)?;
    tx.execute(
        "INSERT INTO attempts VALUES(?,?,?,'running','pending','pending',NULL)",
        params![attempt.as_str(), task.as_str(), generation],
    )?;
    check_point(fault, CutPoint::AttemptWrite)?;
    tx.execute(
        "UPDATE tasks SET generation=?,state='running' WHERE id=?",
        params![task_generation, task.as_str()],
    )?;
    event(
        tx,
        event_id.as_str(),
        task.as_str(),
        &task_generation,
        "attempt_started",
    )?;
    Ok(AttemptHead {
        id: attempt.as_str().to_owned(),
        task_generation,
        generation,
    })
}

#[cfg(test)]
#[path = "../tests/t05_store.rs"]
mod roster_tests;

mod terminal;
pub use terminal::{Stop, Stopped};

#[cfg(test)]
#[path = "../tests/t06_store_staging.rs"]
mod staging_tests;

pub use recovery::{
    DurableAcceptance, DurableAttempt, DurableStop, DurableTask, DurableVerification,
    PendingDelivery, RecoveryInventory, RecoveryLimits,
};

pub use reconciliation::{
    EvidenceAvailability, RECORD_BODY_LIMIT, RECORD_SCAN_LIMIT, ReconciliationRecord, RecordKind,
    RecordRow, Recorded, TerminalOrdinals, record_id,
};
