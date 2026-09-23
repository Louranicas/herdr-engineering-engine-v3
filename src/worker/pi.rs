// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/worker/pi.rs
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

// Pi's wire types are distinct from HEE3-Control. These are worker-selected caps.
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value, json};
use std::fmt;

pub const PROFILE: &str = "pi-rpc-earendil-0.85.1";
pub const FRAME_LIMIT: usize = 1_048_576;
pub const STREAM_LIMIT: usize = 8 * 1_048_576;
pub const RECORD_LIMIT: usize = 65_536;
pub const DEPTH_LIMIT: usize = 32;
pub const SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Refusal preserves uncertainty; no variant conveys verification or acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    FrameLimit,
    StreamLimit,
    RecordLimit,
    Truncated,
    Json,
    Shape,
    Number,
    Unsupported,
    Correlation,
    Session,
    Model,
    Ordering,
    Usage,
    Deadline,
    ClockRewind,
    Poisoned,
}

/// An exact, uniquely keyed JSON object. Construction validates raw bytes.
#[derive(Clone, Debug)]
pub struct Frame {
    raw: Vec<u8>,
    value: Value,
}

impl Frame {
    /// Decode one payload, excluding LF. No other wire protocol is implied.
    ///
    /// # Errors
    /// Refuses oversize, noncompact outer framing, invalid UTF-8/JSON, duplicate
    /// keys, excessive nesting, and any non-object root.
    pub fn parse(raw: &[u8]) -> Result<Self, Refusal> {
        if raw.len() > FRAME_LIMIT {
            return Err(Refusal::FrameLimit);
        }
        if raw.first() != Some(&b'{')
            || raw.last() != Some(&b'}')
            || raw.contains(&b'\r')
            || raw.contains(&b'\n')
        {
            return Err(Refusal::Json);
        }
        let mut parser = serde_json::Deserializer::from_slice(raw);
        let value = Seed(0)
            .deserialize(&mut parser)
            .map_err(|_| Refusal::Json)?;
        parser.end().map_err(|_| Refusal::Json)?;
        if !value.is_object() {
            return Err(Refusal::Shape);
        }
        Ok(Self {
            raw: raw.to_vec(),
            value,
        })
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.raw
    }
}

struct Seed(usize);
impl<'de> DeserializeSeed<'de> for Seed {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(self, decoder: D) -> Result<Value, D::Error> {
        decoder.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded JSON with unique object keys")
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("nonfinite number"))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }
    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        if self.0 >= DEPTH_LIMIT {
            return Err(de::Error::custom("JSON depth limit"));
        }
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(Seed(self.0 + 1))? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        if self.0 >= DEPTH_LIMIT {
            return Err(de::Error::custom("JSON depth limit"));
        }
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON key"));
            }
            values.insert(key, map.next_value_seed(Seed(self.0 + 1))?);
        }
        Ok(Value::Object(values))
    }
}

/// Incremental byte-first LF decoder. A refusal permanently poisons the stream.
/// The process owner must separately enforce wall time, stderr and child cleanup.
///
/// ```
/// use habitat_engine::worker::pi::{Framer, validate_record};
/// let mut wire = Framer::default();
/// assert!(wire.push(br#"{"type":"agent_"#)?.is_empty());
/// let records = wire.push(b"start\"}\n")?;
/// assert_eq!(records.len(), 1);
/// validate_record(&records[0])?;
/// assert_eq!(records[0].bytes(), br#"{"type":"agent_start"}"#);
/// wire.finish()?;
/// # Ok::<(), habitat_engine::worker::pi::Refusal>(())
/// ```
#[derive(Default)]
pub struct Framer {
    pending: Vec<u8>,
    total: usize,
    records: usize,
    poisoned: bool,
    closed: bool,
}
impl Framer {
    /// # Errors
    /// Refuses malformed/oversize input and all calls after a previous refusal.
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Frame>, Refusal> {
        if self.poisoned || self.closed {
            return Err(Refusal::Poisoned);
        }
        let result = self.push_inner(bytes);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn push_inner(&mut self, bytes: &[u8]) -> Result<Vec<Frame>, Refusal> {
        self.total = self
            .total
            .checked_add(bytes.len())
            .ok_or(Refusal::StreamLimit)?;
        if self.total > STREAM_LIMIT {
            return Err(Refusal::StreamLimit);
        }
        let mut frames = Vec::new();
        for &byte in bytes {
            if byte == b'\n' {
                if self.records == RECORD_LIMIT {
                    return Err(Refusal::RecordLimit);
                }
                frames.push(Frame::parse(&self.pending)?);
                self.pending.clear();
                self.records += 1;
            } else {
                if self.pending.len() == FRAME_LIMIT {
                    return Err(Refusal::FrameLimit);
                }
                self.pending.push(byte);
            }
        }
        Ok(frames)
    }
    /// # Errors
    /// A nonempty EOF fragment refuses; it is never silently dispatched.
    pub fn finish(&mut self) -> Result<(), Refusal> {
        if self.poisoned {
            return Err(Refusal::Poisoned);
        }
        if !self.pending.is_empty() {
            self.poisoned = true;
            return Err(Refusal::Truncated);
        }
        self.closed = true;
        Ok(())
    }
}

fn fields<'a>(
    v: &'a Value,
    required: &[&str],
    optional: &[&str],
) -> Result<&'a Map<String, Value>, Refusal> {
    let map = v.as_object().ok_or(Refusal::Shape)?;
    if required.iter().any(|k| !map.contains_key(*k))
        || map
            .keys()
            .any(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()))
    {
        return Err(Refusal::Shape);
    }
    Ok(map)
}
fn text(v: &Value) -> Result<&str, Refusal> {
    v.as_str().ok_or(Refusal::Shape)
}
fn count(v: &Value) -> Result<u64, Refusal> {
    v.as_u64()
        .filter(|n| *n <= SAFE_INTEGER)
        .ok_or(Refusal::Number)
}
fn boolean(v: &Value) -> Result<bool, Refusal> {
    v.as_bool().ok_or(Refusal::Shape)
}
fn amount(v: &Value) -> Result<f64, Refusal> {
    v.as_f64()
        .filter(|n| n.is_finite() && *n >= 0.0)
        .ok_or(Refusal::Number)
}
fn array(v: &Value) -> Result<&[Value], Refusal> {
    v.as_array().map(Vec::as_slice).ok_or(Refusal::Shape)
}
fn strings(v: &Value) -> Result<&[Value], Refusal> {
    let list = array(v)?;
    for value in list {
        text(value)?;
    }
    Ok(list)
}
fn sum(values: &[u64]) -> Result<u64, Refusal> {
    values.iter().try_fold(0_u64, |total, value| {
        total
            .checked_add(*value)
            .filter(|n| *n <= SAFE_INTEGER)
            .ok_or(Refusal::Number)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Thinking {
    Off,
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
    Max,
}
impl Thinking {
    fn parse(v: &Value) -> Result<Self, Refusal> {
        match text(v)? {
            "off" => Ok(Self::Off),
            "minimal" => Ok(Self::Minimal),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "xhigh" => Ok(Self::XHigh),
            "max" => Ok(Self::Max),
            _ => Err(Refusal::Unsupported),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
            Self::Max => "max",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelIdentity {
    pub provider: String,
    pub id: String,
}
fn model(v: &Value) -> Result<ModelIdentity, Refusal> {
    let m = fields(
        v,
        &[
            "id",
            "name",
            "api",
            "provider",
            "baseUrl",
            "reasoning",
            "input",
            "cost",
            "contextWindow",
            "maxTokens",
        ],
        &[],
    )?;
    for key in ["id", "name", "api", "provider", "baseUrl"] {
        if text(&m[key])?.is_empty() {
            return Err(Refusal::Model);
        }
    }
    if m["id"] == "unknown" || m["provider"] == "unknown" {
        return Err(Refusal::Model);
    }
    boolean(&m["reasoning"])?;
    let inputs = strings(&m["input"])?;
    if inputs.is_empty() || inputs.iter().any(|v| v != "text" && v != "image") {
        return Err(Refusal::Unsupported);
    }
    let cost = fields(
        &m["cost"],
        &["input", "output", "cacheRead", "cacheWrite"],
        &[],
    )?;
    for value in cost.values() {
        amount(value)?;
    }
    if count(&m["contextWindow"])? == 0 || count(&m["maxTokens"])? == 0 {
        return Err(Refusal::Model);
    }
    Ok(ModelIdentity {
        provider: text(&m["provider"])?.to_owned(),
        id: text(&m["id"])?.to_owned(),
    })
}

/// Provider-reported data only. Subset fields never increase the total again.
#[derive(Clone, Debug, PartialEq)]
pub struct Usage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub reasoning: Option<u64>,
    pub cache_write_1h: Option<u64>,
    pub total: u64,
    pub cost: f64,
}
fn usage(v: &Value) -> Result<Usage, Refusal> {
    let m = fields(
        v,
        &[
            "input",
            "output",
            "cacheRead",
            "cacheWrite",
            "totalTokens",
            "cost",
        ],
        &["reasoning", "cacheWrite1h"],
    )?;
    let input = count(&m["input"])?;
    let output = count(&m["output"])?;
    let cache_read = count(&m["cacheRead"])?;
    let cache_write = count(&m["cacheWrite"])?;
    let total = count(&m["totalTokens"])?;
    if sum(&[input, output, cache_read, cache_write])? != total {
        return Err(Refusal::Usage);
    }
    let reasoning = m.get("reasoning").map(count).transpose()?;
    let cache_write_1h = m.get("cacheWrite1h").map(count).transpose()?;
    if reasoning.is_some_and(|n| n > output) || cache_write_1h.is_some_and(|n| n > cache_write) {
        return Err(Refusal::Usage);
    }
    let costs = fields(
        &m["cost"],
        &["input", "output", "cacheRead", "cacheWrite", "total"],
        &[],
    )?;
    for value in costs.values() {
        amount(value)?;
    }
    Ok(Usage {
        input,
        output,
        cache_read,
        cache_write,
        reasoning,
        cache_write_1h,
        total,
        cost: amount(&costs["total"])?,
    })
}

fn content(v: &Value, assistant: bool) -> Result<(), Refusal> {
    for block in array(v)? {
        match text(&block["type"])? {
            "text" => {
                let m = fields(block, &["type", "text"], &["textSignature"])?;
                text(&m["text"])?;
                if let Some(v) = m.get("textSignature") {
                    text(v)?;
                }
            }
            "thinking" if assistant => {
                let m = fields(
                    block,
                    &["type", "thinking"],
                    &["thinkingSignature", "redacted"],
                )?;
                text(&m["thinking"])?;
                if let Some(v) = m.get("thinkingSignature") {
                    text(v)?;
                }
                if let Some(v) = m.get("redacted") {
                    boolean(v)?;
                }
            }
            _ => return Err(Refusal::Unsupported),
        }
    }
    Ok(())
}
fn message(v: &Value) -> Result<Option<Usage>, Refusal> {
    match text(&v["role"])? {
        "user" => {
            let m = fields(v, &["role", "content", "timestamp"], &[])?;
            count(&m["timestamp"])?;
            if !m["content"].is_string() {
                content(&m["content"], false)?;
            }
            Ok(None)
        }
        "assistant" => {
            let m = fields(
                v,
                &[
                    "role",
                    "content",
                    "api",
                    "provider",
                    "model",
                    "usage",
                    "stopReason",
                    "timestamp",
                ],
                &[
                    "responseModel",
                    "responseId",
                    "providerThinkingLevel",
                    "errorMessage",
                    "rawStopReason",
                    "endTurn",
                ],
            )?;
            for key in ["api", "provider", "model"] {
                text(&m[key])?;
            }
            for key in [
                "responseModel",
                "responseId",
                "providerThinkingLevel",
                "errorMessage",
                "rawStopReason",
            ] {
                if let Some(v) = m.get(key) {
                    text(v)?;
                }
            }
            if let Some(v) = m.get("endTurn") {
                boolean(v)?;
            }
            count(&m["timestamp"])?;
            content(&m["content"], true)?;
            if !["pending", "stop", "length", "error", "aborted"].contains(&text(&m["stopReason"])?)
            {
                return Err(Refusal::Unsupported);
            }
            Ok(Some(usage(&m["usage"])?))
        }
        _ => Err(Refusal::Unsupported),
    }
}
fn delta(v: &Value) -> Result<(), Refusal> {
    let kind = text(&v["type"])?;
    match kind {
        "start" => {
            fields(v, &["type"], &[])?;
        }
        "text_start" | "thinking_start" => {
            fields(v, &["type", "contentIndex"], &[])?;
            count(&v["contentIndex"])?;
        }
        "text_delta" | "thinking_delta" => {
            fields(v, &["type", "contentIndex", "delta"], &[])?;
            count(&v["contentIndex"])?;
            text(&v["delta"])?;
        }
        "text_end" | "thinking_end" => {
            fields(v, &["type", "contentIndex", "content"], &[])?;
            count(&v["contentIndex"])?;
            text(&v["content"])?;
        }
        "done" => {
            fields(v, &["type", "reason", "message"], &[])?;
            if !["stop", "length"].contains(&text(&v["reason"])?) {
                return Err(Refusal::Unsupported);
            }
            if v["message"]["role"] != "assistant" || v["message"]["stopReason"] != v["reason"] {
                return Err(Refusal::Shape);
            }
            message(&v["message"])?;
        }
        "error" => {
            fields(v, &["type", "reason", "error"], &[])?;
            if !["aborted", "error"].contains(&text(&v["reason"])?) {
                return Err(Refusal::Unsupported);
            }
            if v["error"]["role"] != "assistant" || v["error"]["stopReason"] != v["reason"] {
                return Err(Refusal::Shape);
            }
            message(&v["error"])?;
        }
        _ => return Err(Refusal::Unsupported),
    }
    Ok(())
}

/// Validate only the reviewed no-tools wire subset; raw bytes remain available.
///
/// # Errors
/// Unknown fields/records, unreviewed optional vendor maps and invalid scalars
/// refuse compatibility. This function grants no command or process authority.
pub fn validate_record(frame: &Frame) -> Result<(), Refusal> {
    let v = &frame.value;
    match text(&v["type"])? {
        "response" => validate_response(v)?,
        "agent_start" | "agent_settled" | "turn_start" => {
            fields(v, &["type"], &[])?;
        }
        "agent_end" => {
            fields(v, &["type", "messages", "willRetry"], &[])?;
            boolean(&v["willRetry"])?;
            for m in array(&v["messages"])? {
                message(m)?;
            }
        }
        "message_start" | "message_end" => {
            fields(v, &["type", "message"], &[])?;
            message(&v["message"])?;
        }
        "message_update" => {
            fields(v, &["type", "usage", "assistantMessageEvent"], &[])?;
            usage(&v["usage"])?;
            delta(&v["assistantMessageEvent"])?;
        }
        "turn_end" => {
            fields(v, &["type", "message", "toolResults"], &[])?;
            message(&v["message"])?;
            if !array(&v["toolResults"])?.is_empty() {
                return Err(Refusal::Unsupported);
            }
        }
        "queue_update" => {
            fields(v, &["type", "steering", "followUp"], &[])?;
            strings(&v["steering"])?;
            strings(&v["followUp"])?;
        }
        "thinking_level_changed" => {
            fields(v, &["type", "level"], &[])?;
            Thinking::parse(&v["level"])?;
        }
        "auto_retry_start" => {
            fields(
                v,
                &["type", "attempt", "maxAttempts", "delayMs", "errorMessage"],
                &[],
            )?;
            let attempt = count(&v["attempt"])?;
            let maximum = count(&v["maxAttempts"])?;
            if attempt == 0 || attempt > maximum || maximum > 3 {
                return Err(Refusal::Unsupported);
            }
            count(&v["delayMs"])?;
            text(&v["errorMessage"])?;
        }
        "auto_retry_end" => {
            fields(v, &["type", "success", "attempt"], &["finalError"])?;
            boolean(&v["success"])?;
            if !(1..=3).contains(&count(&v["attempt"])?) {
                return Err(Refusal::Number);
            }
            if let Some(v) = v.get("finalError") {
                text(v)?;
            }
        }
        _ => return Err(Refusal::Unsupported),
    }
    Ok(())
}
fn validate_response(v: &Value) -> Result<(), Refusal> {
    if boolean(&v["success"])? {
        let cmd = text(&v["command"])?;
        let data = [
            "get_available_models",
            "set_model",
            "get_state",
            "clear_queue",
            "get_session_stats",
        ]
        .contains(&cmd);
        fields(
            v,
            if data {
                &["id", "type", "command", "success", "data"]
            } else {
                &["id", "type", "command", "success"]
            },
            &[],
        )?;
        text(&v["id"])?;
        match cmd {
            "prompt" | "abort" | "set_thinking_level" => (),
            "set_model" => {
                model(&v["data"])?;
            }
            "get_available_models" => {
                fields(&v["data"], &["models"], &[])?;
                for m in array(&v["data"]["models"])? {
                    model(m)?;
                }
            }
            "clear_queue" => {
                fields(&v["data"], &["steering", "followUp"], &[])?;
                strings(&v["data"]["steering"])?;
                strings(&v["data"]["followUp"])?;
            }
            "get_state" => {
                state(&v["data"])?;
            }
            "get_session_stats" => {
                stats(&v["data"])?;
            }
            _ => return Err(Refusal::Unsupported),
        }
    } else {
        fields(v, &["id", "type", "command", "success", "error"], &[])?;
        text(&v["id"])?;
        text(&v["error"])?;
        if !COMMAND_NAMES.contains(&text(&v["command"])?) {
            return Err(Refusal::Unsupported);
        }
    }
    Ok(())
}

const COMMAND_NAMES: [&str; 8] = [
    "get_available_models",
    "set_model",
    "set_thinking_level",
    "get_state",
    "prompt",
    "clear_queue",
    "abort",
    "get_session_stats",
];

#[derive(Clone, Debug, PartialEq)]
pub struct StateReadback {
    pub session_id: String,
    pub model: Option<ModelIdentity>,
    pub thinking: Thinking,
    pub idle: bool,
    pub message_count: u64,
    pub auto_compaction: bool,
}
fn state(v: &Value) -> Result<StateReadback, Refusal> {
    let m = fields(
        v,
        &[
            "thinkingLevel",
            "isStreaming",
            "isCompacting",
            "steeringMode",
            "followUpMode",
            "sessionId",
            "autoCompactionEnabled",
            "messageCount",
            "pendingMessageCount",
        ],
        &["model", "sessionFile", "sessionName"],
    )?;
    for key in ["sessionFile", "sessionName"] {
        if let Some(v) = m.get(key) {
            text(v)?;
        }
    }
    for key in ["steeringMode", "followUpMode"] {
        if !["all", "one-at-a-time"].contains(&text(&m[key])?) {
            return Err(Refusal::Unsupported);
        }
    }
    let auto_compaction = boolean(&m["autoCompactionEnabled"])?;
    let streaming = boolean(&m["isStreaming"])?;
    let compacting = boolean(&m["isCompacting"])?;
    let pending = count(&m["pendingMessageCount"])?;
    let session_id = text(&m["sessionId"])?;
    if session_id.is_empty() {
        return Err(Refusal::Session);
    }
    Ok(StateReadback {
        session_id: session_id.to_owned(),
        model: m.get("model").map(state_model).transpose()?.flatten(),
        thinking: Thinking::parse(&m["thinkingLevel"])?,
        idle: !streaming && !compacting && pending == 0,
        message_count: count(&m["messageCount"])?,
        auto_compaction,
    })
}
#[derive(Clone, Debug, PartialEq)]
pub struct SessionUsage {
    pub session_id: String,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub total: u64,
    pub cost: f64,
    pub context_tokens: Option<u64>,
}
fn stats(v: &Value) -> Result<SessionUsage, Refusal> {
    let m = fields(
        v,
        &[
            "sessionId",
            "userMessages",
            "assistantMessages",
            "toolCalls",
            "toolResults",
            "totalMessages",
            "tokens",
            "cost",
        ],
        &["sessionFile", "contextUsage"],
    )?;
    text(&m["sessionId"])?;
    if let Some(v) = m.get("sessionFile") {
        text(v)?;
    }
    let counts = [
        count(&m["userMessages"])?,
        count(&m["assistantMessages"])?,
        count(&m["toolResults"])?,
    ];
    if sum(&counts)? != count(&m["totalMessages"])?
        || count(&m["toolCalls"])? != 0
        || counts[2] != 0
    {
        return Err(Refusal::Usage);
    }
    let tokens = fields(
        &m["tokens"],
        &["input", "output", "cacheRead", "cacheWrite", "total"],
        &[],
    )?;
    let input = count(&tokens["input"])?;
    let output = count(&tokens["output"])?;
    let cache_read = count(&tokens["cacheRead"])?;
    let cache_write = count(&tokens["cacheWrite"])?;
    let total = count(&tokens["total"])?;
    if sum(&[input, output, cache_read, cache_write])? != total {
        return Err(Refusal::Usage);
    }
    let context_tokens = if let Some(context) = m.get("contextUsage") {
        let c = fields(context, &["tokens", "contextWindow", "percent"], &[])?;
        if count(&c["contextWindow"])? == 0 || c["tokens"].is_null() != c["percent"].is_null() {
            return Err(Refusal::Usage);
        }
        if c["tokens"].is_null() {
            None
        } else {
            amount(&c["percent"])?;
            Some(count(&c["tokens"])?)
        }
    } else {
        None
    };
    Ok(SessionUsage {
        session_id: text(&m["sessionId"])?.to_owned(),
        input,
        output,
        cache_read,
        cache_write,
        total,
        cost: amount(&m["cost"])?,
        context_tokens,
    })
}

fn state_model(v: &Value) -> Result<Option<ModelIdentity>, Refusal> {
    // Exact SDK fallback observed in the credential-free pinned-package smoke.
    let unavailable = json!({"id":"unknown","name":"unknown","api":"unknown","provider":"unknown","baseUrl":"","reasoning":false,"input":[],"cost":{"input":0,"output":0,"cacheRead":0,"cacheWrite":0},"contextWindow":0,"maxTokens":0});
    if *v == unavailable {
        Ok(None)
    } else {
        model(v).map(Some)
    }
}

use crate::contracts::{Generation, UuidV4};

/// Fixed task identity and generation supplied by the trusted worker owner.
#[derive(Clone, Copy, Debug)]
pub struct Binding<'a> {
    pub task: UuidV4<'a>,
    pub attempt: UuidV4<'a>,
    pub generation: Generation,
}

/// Closed vendor command subset. It confers no provider or tool grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    AvailableModels,
    SetModel(ModelIdentity),
    SetThinking(Thinking),
    State,
    Prompt(String),
    ClearQueue,
    Abort,
    Stats,
}
impl Command {
    fn name(&self) -> &'static str {
        match self {
            Self::AvailableModels => "get_available_models",
            Self::SetModel(_) => "set_model",
            Self::SetThinking(_) => "set_thinking_level",
            Self::State => "get_state",
            Self::Prompt(_) => "prompt",
            Self::ClearQueue => "clear_queue",
            Self::Abort => "abort",
            Self::Stats => "get_session_stats",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RunState {
    #[default]
    Idle,
    AwaitingStart,
    Running,
    Ended,
    PiSettled,
    Refused,
}

/// A Pi observation never settles external effects, descendants or acceptance.
#[derive(Clone, Debug, PartialEq)]
pub enum Observation {
    Acknowledged,
    Rejected(String),
    State(StateReadback),
    AvailableModels(Vec<ModelIdentity>),
    FinalMessage(Value),
    /// Snapshot while active or before a final idle readback; not reconciled.
    ProvisionalUsage(SessionUsage),
    /// Provider-reported counters match all observed final assistant messages.
    ReconciledUsage(SessionUsage),
    PiSettled,
    CancelledPiIdle,
    Activity,
}
struct Pending {
    id: String,
    command: Command,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Cancellation {
    #[default]
    None,
    Clearing,
    Cleared,
    Aborting,
    Aborted,
    Idle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Retry {
    #[default]
    None,
    Expected {
        previous: u64,
        maximum: Option<u64>,
    },
    Waiting {
        attempt: u64,
        maximum: u64,
    },
    Active {
        attempt: u64,
        maximum: u64,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Turn {
    #[default]
    Awaiting,
    Open,
    Closed,
}

/// One exclusively owned Pi session for one HEE attempt. There is no retry owner
/// here: automatic Pi continuations consume the unchanged original task clock.
pub struct Session<'a> {
    binding: Binding<'a>,
    next_id: u64,
    pending: Option<Pending>,
    vendor_id: Option<String>,
    desired_model: Option<ModelIdentity>,
    desired_thinking: Option<Thinking>,
    idle_readback: bool,
    run: RunState,
    cancellation: Cancellation,
    had_start: bool,
    message_role: Option<String>,
    last_assistant_stop: Option<String>,
    retry: Retry,
    turn: Turn,
    run_messages: Vec<Value>,
    run_message_bytes: usize,
    reported: [u64; 4],
    last_elapsed_ms: u64,
}
impl<'a> Session<'a> {
    #[must_use]
    pub fn new(binding: Binding<'a>) -> Self {
        Self {
            binding,
            next_id: 1,
            pending: None,
            vendor_id: None,
            desired_model: None,
            desired_thinking: None,
            idle_readback: false,
            run: RunState::Idle,
            cancellation: Cancellation::None,
            had_start: false,
            message_role: None,
            last_assistant_stop: None,
            retry: Retry::None,
            turn: Turn::Awaiting,
            run_messages: Vec::new(),
            run_message_bytes: 0,
            reported: [0; 4],
            last_elapsed_ms: 0,
        }
    }
    #[must_use]
    pub fn binding(&self) -> Binding<'a> {
        self.binding
    }
    #[must_use]
    pub fn run_state(&self) -> RunState {
        self.run
    }
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        self.vendor_id.as_deref()
    }
    #[must_use]
    pub fn pending_id(&self) -> Option<&str> {
        self.pending.as_ref().map(|p| p.id.as_str())
    }

    /// The process owner must call this after EOF/framing, timeout, stderr or
    /// cleanup refusal. It does not erase the pending operation or its identity.
    pub fn transport_failed(&mut self) {
        self.run = RunState::Refused;
        self.idle_readback = false;
    }

    fn clock(&mut self, elapsed_ms: u64) -> Result<(), Refusal> {
        if self.run == RunState::Refused {
            return Err(Refusal::Poisoned);
        }
        if elapsed_ms < self.last_elapsed_ms {
            self.transport_failed();
            return Err(Refusal::ClockRewind);
        }
        self.last_elapsed_ms = elapsed_ms;
        if elapsed_ms >= 1_200_000 {
            self.transport_failed();
            return Err(Refusal::Deadline);
        }
        Ok(())
    }

    /// Bind a unique ID before returning the exact LF-terminated request bytes.
    /// Dependent commands serialize; prompt requires final model/thinking readback.
    /// The caller must obtain its task/data/spending authority before invoking it.
    ///
    /// # Errors
    /// Refuses unsafe command order, missing readback, oversized prompts, clocks
    /// outside the unchanged task budget, and poisoned sessions.
    pub fn issue(&mut self, command: Command, elapsed_ms: u64) -> Result<Vec<u8>, Refusal> {
        self.clock(elapsed_ms)?;
        if self.pending.is_some() {
            return Err(Refusal::Ordering);
        }
        self.check_command(&command, elapsed_ms)?;
        let id = format!(
            "{}:{}:{}:{}",
            self.binding.task.as_str(),
            self.binding.attempt.as_str(),
            self.binding.generation,
            self.next_id
        );
        let next_id = self.next_id.checked_add(1).ok_or(Refusal::Number)?;
        let mut value = json!({"id":id,"type":command.name()});
        match &command {
            Command::SetModel(identity) => {
                value["provider"] = json!(identity.provider);
                value["modelId"] = json!(identity.id);
            }
            Command::SetThinking(level) => {
                value["level"] = json!(level.name());
            }
            Command::Prompt(message) => {
                value["message"] = json!(message);
            }
            _ => (),
        }
        let mut bytes = serde_json::to_vec(&value).map_err(|_| Refusal::Json)?;
        if bytes.len() > FRAME_LIMIT {
            return Err(Refusal::FrameLimit);
        }
        bytes.push(b'\n');
        match &command {
            Command::SetModel(_) | Command::SetThinking(_) => self.idle_readback = false,
            Command::Prompt(_) => {
                self.run = RunState::AwaitingStart;
                self.idle_readback = false;
            }
            Command::ClearQueue => {
                self.cancellation = Cancellation::Clearing;
                self.idle_readback = false;
            }
            Command::Abort => self.cancellation = Cancellation::Aborting,
            _ => (),
        }
        self.next_id = next_id;
        self.pending = Some(Pending { id, command });
        Ok(bytes)
    }
    fn check_command(&self, command: &Command, elapsed_ms: u64) -> Result<(), Refusal> {
        match command {
            Command::Prompt(message) => {
                if elapsed_ms >= 900_000 {
                    return Err(Refusal::Deadline);
                }
                if message.is_empty() || message.len() > 256 * 1024 {
                    return Err(Refusal::FrameLimit);
                }
                if self.run != RunState::Idle
                    || self.cancellation != Cancellation::None
                    || !self.idle_readback
                    || self.desired_model.is_none()
                    || self.desired_thinking.is_none()
                {
                    return Err(Refusal::Ordering);
                }
            }
            Command::SetModel(identity) => {
                if self.run != RunState::Idle
                    || self.cancellation != Cancellation::None
                    || self.vendor_id.is_none()
                    || identity.provider.is_empty()
                    || identity.id.is_empty()
                    || identity.provider == "unknown"
                    || identity.id == "unknown"
                {
                    return Err(Refusal::Ordering);
                }
            }
            Command::SetThinking(_) => {
                if self.run != RunState::Idle
                    || self.cancellation != Cancellation::None
                    || self.vendor_id.is_none()
                {
                    return Err(Refusal::Ordering);
                }
            }
            Command::ClearQueue => {
                if self.cancellation != Cancellation::None || self.vendor_id.is_none() {
                    return Err(Refusal::Ordering);
                }
            }
            Command::Abort => {
                if self.cancellation != Cancellation::Cleared {
                    return Err(Refusal::Ordering);
                }
            }
            Command::Stats => {
                if self.vendor_id.is_none() {
                    return Err(Refusal::Ordering);
                }
            }
            Command::State | Command::AvailableModels => (),
        }
        if matches!(
            self.cancellation,
            Cancellation::Clearing | Cancellation::Aborting
        ) {
            return Err(Refusal::Ordering);
        }
        Ok(())
    }

    /// Consume one structurally checked record from the exclusively owned pipe.
    ///
    /// # Errors
    /// Any unexpected record, correlation, identity, scalar or lifecycle error
    /// permanently refuses this attempt; retained raw bytes belong to the caller.
    pub fn observe(&mut self, frame: &Frame, elapsed_ms: u64) -> Result<Observation, Refusal> {
        self.clock(elapsed_ms)?;
        let result = validate_record(frame).and_then(|()| {
            if frame.value["type"] == "response" {
                self.response(&frame.value)
            } else {
                self.event(&frame.value)
            }
        });
        if result.is_err() {
            self.transport_failed();
        }
        result
    }
    fn response(&mut self, v: &Value) -> Result<Observation, Refusal> {
        let pending = self.pending.as_ref().ok_or(Refusal::Correlation)?;
        if text(&v["id"])? != pending.id || text(&v["command"])? != pending.command.name() {
            return Err(Refusal::Correlation);
        }
        if !boolean(&v["success"])? {
            let error = text(&v["error"])?.to_owned();
            self.pending = None;
            self.transport_failed();
            return Ok(Observation::Rejected(error));
        }
        let command = pending.command.clone();
        let result = match command {
            Command::State => self.readback(state(&v["data"])?)?,
            Command::AvailableModels => {
                let models = array(&v["data"]["models"])?
                    .iter()
                    .map(model)
                    .collect::<Result<Vec<_>, _>>()?;
                Observation::AvailableModels(models)
            }
            Command::SetModel(expected) => {
                if model(&v["data"])? != expected {
                    return Err(Refusal::Model);
                }
                self.desired_model = Some(expected);
                Observation::Acknowledged
            }
            Command::SetThinking(expected) => {
                self.desired_thinking = Some(expected);
                Observation::Acknowledged
            }
            Command::ClearQueue => {
                self.cancellation = Cancellation::Cleared;
                Observation::Acknowledged
            }
            Command::Abort => {
                self.cancellation = Cancellation::Aborted;
                Observation::Acknowledged
            }
            Command::Prompt(_) => Observation::Acknowledged,
            Command::Stats => self.usage_readback(stats(&v["data"])?)?,
        };
        self.pending = None;
        Ok(result)
    }
    fn readback(&mut self, state: StateReadback) -> Result<Observation, Refusal> {
        if self
            .vendor_id
            .as_ref()
            .is_some_and(|id| *id != state.session_id)
        {
            return Err(Refusal::Session);
        }
        if self.vendor_id.is_none() {
            if !state.idle || state.message_count != 0 {
                return Err(Refusal::Session);
            }
            self.vendor_id = Some(state.session_id.clone());
        }
        if self
            .desired_model
            .as_ref()
            .is_some_and(|m| Some(m) != state.model.as_ref())
            || self.desired_thinking.is_some_and(|t| t != state.thinking)
        {
            return Err(Refusal::Model);
        }
        if state.auto_compaction {
            return Err(Refusal::Unsupported);
        }
        self.idle_readback = state.idle;
        if self.cancellation == Cancellation::Aborted
            && state.idle
            && (!self.had_start || self.run == RunState::PiSettled)
        {
            self.cancellation = Cancellation::Idle;
            return Ok(Observation::CancelledPiIdle);
        }
        Ok(Observation::State(state))
    }
    fn usage_readback(&self, reported: SessionUsage) -> Result<Observation, Refusal> {
        if self.vendor_id.as_deref() != Some(reported.session_id.as_str()) {
            return Err(Refusal::Session);
        }
        if self.run != RunState::PiSettled || !self.idle_readback {
            return Ok(Observation::ProvisionalUsage(reported));
        }
        if [
            reported.input,
            reported.output,
            reported.cache_read,
            reported.cache_write,
        ] != self.reported
        {
            return Err(Refusal::Usage);
        }
        Ok(Observation::ReconciledUsage(reported))
    }
    fn event(&mut self, v: &Value) -> Result<Observation, Refusal> {
        let kind = text(&v["type"])?;
        if kind == "queue_update" {
            if !array(&v["steering"])?.is_empty() || !array(&v["followUp"])?.is_empty() {
                return Err(Refusal::Ordering);
            }
            return Ok(Observation::Activity);
        }
        if kind == "thinking_level_changed" {
            if !matches!(
                self.pending.as_ref().map(|p| &p.command),
                Some(Command::SetThinking(_) | Command::SetModel(_))
            ) {
                return Err(Refusal::Ordering);
            }
            return Ok(Observation::Activity);
        }
        if self.vendor_id.is_none() {
            return Err(Refusal::Session);
        }
        if kind == "agent_start" {
            if self
                .pending
                .as_ref()
                .is_some_and(|p| matches!(p.command, Command::Prompt(_)))
                || !matches!(self.run, RunState::AwaitingStart | RunState::Ended)
            {
                return Err(Refusal::Ordering);
            }
            self.retry = match (self.run, self.retry) {
                (RunState::AwaitingStart, Retry::None) => Retry::None,
                (RunState::Ended, Retry::Waiting { attempt, maximum }) => {
                    Retry::Active { attempt, maximum }
                }
                _ => return Err(Refusal::Ordering),
            };
            self.last_assistant_stop = None;
            self.turn = Turn::Awaiting;
            self.run_messages.clear();
            self.run_message_bytes = 0;
            self.run = RunState::Running;
            self.had_start = true;
            self.idle_readback = false;
            return Ok(Observation::Activity);
        }
        if !self.had_start || !matches!(self.run, RunState::Running | RunState::Ended) {
            return Err(Refusal::Ordering);
        }
        match kind {
            "agent_end" => {
                if self.run != RunState::Running
                    || self.message_role.is_some()
                    || self.turn != Turn::Closed
                {
                    return Err(Refusal::Ordering);
                }
                if array(&v["messages"])? != self.run_messages {
                    return Err(Refusal::Session);
                }
                if boolean(&v["willRetry"])? {
                    if self.last_assistant_stop.as_deref() != Some("error") {
                        return Err(Refusal::Ordering);
                    }
                    self.retry = match self.retry {
                        Retry::None => Retry::Expected {
                            previous: 0,
                            maximum: None,
                        },
                        Retry::Active { attempt, maximum } => Retry::Expected {
                            previous: attempt,
                            maximum: Some(maximum),
                        },
                        _ => return Err(Refusal::Ordering),
                    };
                }
                self.run = RunState::Ended;
            }
            "agent_settled" => {
                if self.run != RunState::Ended || self.retry != Retry::None {
                    return Err(Refusal::Ordering);
                }
                self.run = RunState::PiSettled;
                return Ok(Observation::PiSettled);
            }
            "message_start" | "message_update" | "message_end" => return self.message_event(v),
            "auto_retry_start" | "auto_retry_end" => return self.retry_event(v),
            "turn_start" | "turn_end" => return self.turn_event(v),
            _ => return Err(Refusal::Unsupported),
        }
        Ok(Observation::Activity)
    }
    fn turn_event(&mut self, v: &Value) -> Result<Observation, Refusal> {
        if self.run != RunState::Running || self.message_role.is_some() {
            return Err(Refusal::Ordering);
        }
        if v["type"] == "turn_start" {
            if self.turn != Turn::Awaiting {
                return Err(Refusal::Ordering);
            }
            self.turn = Turn::Open;
        } else {
            if self.turn != Turn::Open || self.last_assistant_stop.is_none() {
                return Err(Refusal::Ordering);
            }
            if self.run_messages.last() != Some(&v["message"]) {
                return Err(Refusal::Session);
            }
            self.turn = Turn::Closed;
        }
        Ok(Observation::Activity)
    }
    fn retry_event(&mut self, v: &Value) -> Result<Observation, Refusal> {
        let attempt = count(&v["attempt"])?;
        if v["type"] == "auto_retry_start" {
            let Retry::Expected { previous, maximum } = self.retry else {
                return Err(Refusal::Ordering);
            };
            let actual_maximum = count(&v["maxAttempts"])?;
            if self.run != RunState::Ended
                || attempt != previous + 1
                || maximum.is_some_and(|m| m != actual_maximum)
            {
                return Err(Refusal::Ordering);
            }
            self.retry = Retry::Waiting {
                attempt,
                maximum: actual_maximum,
            };
        } else {
            let (Retry::Waiting {
                attempt: expected, ..
            }
            | Retry::Active {
                attempt: expected, ..
            }) = self.retry
            else {
                return Err(Refusal::Ordering);
            };
            let successful = boolean(&v["success"])?;
            if attempt != expected
                || (successful
                    && (self.run != RunState::Running
                        || self.message_role.is_some()
                        || self
                            .last_assistant_stop
                            .as_deref()
                            .is_none_or(|s| s == "error")))
                || (!successful && self.run != RunState::Ended)
            {
                return Err(Refusal::Ordering);
            }
            self.retry = Retry::None;
        }
        self.idle_readback = false;
        Ok(Observation::Activity)
    }
    fn message_event(&mut self, v: &Value) -> Result<Observation, Refusal> {
        if self.run != RunState::Running || self.turn != Turn::Open {
            return Err(Refusal::Ordering);
        }
        match text(&v["type"])? {
            "message_start" => {
                if self.message_role.is_some() || self.last_assistant_stop.is_some() {
                    return Err(Refusal::Ordering);
                }
                self.message_role = Some(text(&v["message"]["role"])?.to_owned());
            }
            "message_update" => {
                if self.message_role.as_deref() != Some("assistant") {
                    return Err(Refusal::Ordering);
                }
            }
            "message_end" => {
                if self.message_role.as_deref() != Some(text(&v["message"]["role"])?) {
                    return Err(Refusal::Ordering);
                }
                if let Some(usage) = message(&v["message"])? {
                    if v["message"]["stopReason"] == "pending" {
                        return Err(Refusal::Ordering);
                    }
                    self.last_assistant_stop = Some(text(&v["message"]["stopReason"])?.to_owned());
                    if self.desired_model.as_ref().is_none_or(|m| {
                        v["message"]["provider"] != m.provider || v["message"]["model"] != m.id
                    }) {
                        return Err(Refusal::Model);
                    }
                    for (sum, value) in self.reported.iter_mut().zip([
                        usage.input,
                        usage.output,
                        usage.cache_read,
                        usage.cache_write,
                    ]) {
                        *sum = sum
                            .checked_add(value)
                            .filter(|n| *n <= SAFE_INTEGER)
                            .ok_or(Refusal::Number)?;
                    }
                }
                let bytes = serde_json::to_vec(&v["message"])
                    .map_err(|_| Refusal::Json)?
                    .len();
                self.run_message_bytes = self
                    .run_message_bytes
                    .checked_add(bytes)
                    .filter(|n| *n <= FRAME_LIMIT)
                    .ok_or(Refusal::FrameLimit)?;
                self.run_messages.push(v["message"].clone());
                self.message_role = None;
                return Ok(Observation::FinalMessage(v["message"].clone()));
            }
            _ => return Err(Refusal::Unsupported),
        }
        Ok(Observation::Activity)
    }
}
