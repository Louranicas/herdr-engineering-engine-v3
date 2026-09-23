// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/route.rs
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
// HEE3-ANCHORS-END
//! Pure eligibility and routing policy over supplied values. A task's
//! requirements, the candidate recipes (roster identity, declared
//! capabilities, an availability observation with its age, and cost, quality
//! and latency figures that are fixture data here) and one explicit baseline
//! recipe go in as values; a closed [`Route`] decision comes out, explained by
//! the named rules applied in order. This file makes no model call, performs
//! no I/O and reads no clock: the age of every observation is an input value,
//! and the only place a route configuration is read is [`Policy::load`].

use crate::contracts::roster::{Availability, Locality};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

/// The roster's record bound; a larger candidate set is refused by count.
pub const MAX_CANDIDATES: usize = 256;
/// The roster's observation TTL bound; a staleness bound outside `1..=MAX_STALENESS_MS` is refused.
pub const MAX_STALENESS_MS: u64 = 60_000;
/// Quality figures are basis points of a caller-owned scale; larger values are refused.
pub const MAX_QUALITY_BASIS_POINTS: u16 = 10_000;
/// The route configuration schema this loader reads.
pub const SCHEMA_VERSION: i64 = 1;
const MAX_TEXT: usize = 128;
const MAX_CAPABILITIES: usize = 128;

/// The named policy rules, in evaluation order. A decision names the rule that made it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Rule {
    /// The baseline is screened first; when a fallback is required it must have passed every
    /// filter with evidence, or the decision is a refusal naming the defect.
    R01BaselineGuard,
    /// A recipe must declare every capability the task requires.
    R02RequiredCapabilities,
    /// A recipe's context limit must hold the task's context.
    R03ContextLimit,
    /// A local-only task admits only local recipes.
    R04PrivacyClass,
    /// A recipe observed unavailable is excluded; an unknown, stale or absent observation is an evidence gap.
    R05Availability,
    /// Under a cost ceiling a recipe's cost figure must not exceed it.
    R06CostCeiling,
    /// Under a deadline a recipe's latency figure must not exceed it.
    R07Deadline,
    /// Under a quality floor a recipe's quality figure must reach it.
    R08QualityFloor,
    /// Any evidence gap among the screened recipes routes to the baseline.
    R09InsufficientEvidence,
    /// An empty eligible set routes to the baseline.
    R10NoEligibleCandidate,
    /// The eligible set is totally ordered by the declared keys; a strict best is chosen.
    R11Ranking,
    /// Equal top keys route to the baseline.
    R12Tie,
}

impl Rule {
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::R01BaselineGuard => "R01",
            Self::R02RequiredCapabilities => "R02",
            Self::R03ContextLimit => "R03",
            Self::R04PrivacyClass => "R04",
            Self::R05Availability => "R05",
            Self::R06CostCeiling => "R06",
            Self::R07Deadline => "R07",
            Self::R08QualityFloor => "R08",
            Self::R09InsufficientEvidence => "R09",
            Self::R10NoEligibleCandidate => "R10",
            Self::R11Ranking => "R11",
            Self::R12Tie => "R12",
        }
    }
}

/// A hard filter as the configuration names it. Every filter is applied to every
/// screened recipe in the declared order; the first non-pass decides that recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Filter {
    RequiredCapabilities,
    ContextLimit,
    PrivacyClass,
    Availability,
    CostCeiling,
    Deadline,
    QualityFloor,
}

impl Filter {
    /// The complete filter set: a declaration must name each exactly once.
    pub const ALL: [Self; 7] = [
        Self::RequiredCapabilities,
        Self::ContextLimit,
        Self::PrivacyClass,
        Self::Availability,
        Self::CostCeiling,
        Self::Deadline,
        Self::QualityFloor,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::RequiredCapabilities => "required_capabilities",
            Self::ContextLimit => "context_limit",
            Self::PrivacyClass => "privacy_class",
            Self::Availability => "availability",
            Self::CostCeiling => "cost_ceiling",
            Self::Deadline => "deadline",
            Self::QualityFloor => "quality_floor",
        }
    }

    #[must_use]
    pub fn rule(self) -> Rule {
        match self {
            Self::RequiredCapabilities => Rule::R02RequiredCapabilities,
            Self::ContextLimit => Rule::R03ContextLimit,
            Self::PrivacyClass => Rule::R04PrivacyClass,
            Self::Availability => Rule::R05Availability,
            Self::CostCeiling => Rule::R06CostCeiling,
            Self::Deadline => Rule::R07Deadline,
            Self::QualityFloor => Rule::R08QualityFloor,
        }
    }

    fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|filter| filter.name() == name)
    }
}

/// A ranking key as the configuration names it. Keys are compared in the declared
/// order; lower cost, higher quality and lower latency rank first.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    Cost,
    Quality,
    Latency,
}

impl Key {
    /// The complete key set: a declaration must name each exactly once.
    pub const ALL: [Self; 3] = [Self::Cost, Self::Quality, Self::Latency];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Cost => "cost",
            Self::Quality => "quality",
            Self::Latency => "latency",
        }
    }

    #[must_use]
    pub fn figure(self) -> Figure {
        match self {
            Self::Cost => Figure::Cost,
            Self::Quality => Figure::Quality,
            Self::Latency => Figure::Latency,
        }
    }

    fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| key.name() == name)
    }
}

/// What happens when the top of the ranking is shared. Only routing to the
/// explicit baseline is admitted: a tie is reported, never broken silently.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TieRule {
    Baseline,
}

impl TieRule {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
        }
    }
}

/// A figure a recipe carries as supplied fixture data.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Figure {
    Cost,
    Quality,
    Latency,
}

/// The task's privacy class. `LocalOnly` is the only class the RC01 profile admits.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyClass {
    LocalOnly,
    RemoteAllowed,
}

/// The availability observation a recipe carries, with its age as a value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Observation {
    Unobserved,
    Observed {
        availability: Availability,
        age_ms: u64,
    },
}

/// The task's requirements. Every optional bound applies only when supplied.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Task<'a> {
    pub required_capabilities: &'a [&'a str],
    pub context_tokens: u64,
    pub privacy: PrivacyClass,
    pub cost_ceiling_microunits: Option<u64>,
    pub deadline_ms: Option<u64>,
    pub quality_floor_basis_points: Option<u16>,
}

/// One candidate recipe: a roster record's identity and declared shape, its
/// availability observation and its supplied figures. Nothing here is measured.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Recipe<'a> {
    pub id: &'a str,
    pub revision: &'a str,
    pub capabilities: &'a [&'a str],
    pub context_limit_tokens: u64,
    pub locality: Locality,
    pub availability: Observation,
    pub cost_microunits: Option<u64>,
    pub quality_basis_points: Option<u16>,
    pub latency_ms: Option<u64>,
}

/// Why a hard filter excluded a recipe, with the compared values.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "why")]
pub enum Exclusion<'a> {
    MissingCapability {
        capability: &'a str,
    },
    ContextExceeded {
        limit_tokens: u64,
        required_tokens: u64,
    },
    PrivacyViolated {
        locality: Locality,
    },
    Unavailable {
        age_ms: u64,
    },
    CostAboveCeiling {
        cost_microunits: u64,
        ceiling_microunits: u64,
    },
    LatencyAboveDeadline {
        latency_ms: u64,
        deadline_ms: u64,
    },
    QualityBelowFloor {
        quality_basis_points: u16,
        floor_basis_points: u16,
    },
}

/// Evidence a rule needed and did not have. A gap never excludes; it routes to the baseline.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "evidence")]
pub enum Gap {
    Unobserved,
    AvailabilityUnknown { age_ms: u64 },
    StaleAvailability { age_ms: u64, bound_ms: u64 },
    MissingFigure { figure: Figure },
}

/// A recipe's position in the ranking with the figures that placed it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Ranked<'a> {
    pub recipe: &'a str,
    pub cost_microunits: u64,
    pub quality_basis_points: u16,
    pub latency_ms: u64,
}

/// One applied rule, in the order the policy applied it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "step")]
pub enum Step<'a> {
    /// The baseline passed the guard: every filter, in order, with evidence. A baseline
    /// that fails the guard appears as `Excluded` or `Gap` in this first position instead.
    Guarded { recipe: &'a str, passed: Vec<Rule> },
    /// A candidate passed every filter and carries every ranking figure.
    Eligible { recipe: &'a str, passed: Vec<Rule> },
    /// A recipe was excluded by the named rule after passing the listed ones.
    Excluded {
        recipe: &'a str,
        passed: Vec<Rule>,
        rule: Rule,
        why: Exclusion<'a>,
    },
    /// The named rule could not be evaluated for a recipe.
    Gap {
        recipe: &'a str,
        passed: Vec<Rule>,
        rule: Rule,
        evidence: Gap,
    },
    /// The eligible set ordered best first by the declared keys.
    Ranked { rule: Rule, order: Vec<Ranked<'a>> },
    /// The rule that made the decision.
    Decided { rule: Rule },
}

/// The stable, ordered explanation of one decision.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Explanation<'a> {
    pub steps: Vec<Step<'a>>,
}

impl<'a> Explanation<'a> {
    /// Every excluded recipe with the rule that excluded it, in explanation order.
    #[must_use]
    pub fn exclusions(&self) -> Vec<(&'a str, Rule)> {
        self.steps
            .iter()
            .filter_map(|step| match step {
                Step::Excluded { recipe, rule, .. } => Some((*recipe, *rule)),
                _ => None,
            })
            .collect()
    }

    /// The rule that made the decision: the last `Decided` step.
    #[must_use]
    pub fn decided_by(&self) -> Option<Rule> {
        self.steps.iter().rev().find_map(|step| match step {
            Step::Decided { rule } => Some(*rule),
            _ => None,
        })
    }
}

/// An evidence gap that routed the decision to the baseline.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct EvidenceGap<'a> {
    pub recipe: &'a str,
    pub rule: Rule,
    pub evidence: Gap,
}

/// Why the baseline was chosen instead of a candidate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum Fallback<'a> {
    Tie { between: Vec<&'a str> },
    InsufficientEvidence { gaps: Vec<EvidenceGap<'a>> },
    NoEligibleCandidate,
}

/// Why nothing can be routed: a fallback was required and the baseline itself failed
/// its guard for this task by the named rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum Refusal<'a> {
    BaselineExcluded { rule: Rule, why: Exclusion<'a> },
    BaselineEvidence { rule: Rule, evidence: Gap },
}

/// The closed decision. An ineligible recipe is never named by `Chosen` or `Baseline`;
/// `Refused` names the fallback that was required and the baseline defect that denied it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "route")]
pub enum Route<'a> {
    Chosen {
        recipe: &'a str,
        revision: &'a str,
        explanation: Explanation<'a>,
    },
    Baseline {
        recipe: &'a str,
        revision: &'a str,
        reason: Fallback<'a>,
        explanation: Explanation<'a>,
    },
    Refused {
        fallback: Fallback<'a>,
        reason: Refusal<'a>,
        explanation: Explanation<'a>,
    },
}

impl<'a> Route<'a> {
    #[must_use]
    pub const fn explanation(&self) -> &Explanation<'a> {
        match self {
            Self::Chosen { explanation, .. }
            | Self::Baseline { explanation, .. }
            | Self::Refused { explanation, .. } => explanation,
        }
    }

    /// The recipe this decision would dispatch, if any.
    #[must_use]
    pub const fn dispatches(&self) -> Option<&'a str> {
        match self {
            Self::Chosen { recipe, .. } | Self::Baseline { recipe, .. } => Some(*recipe),
            Self::Refused { .. } => None,
        }
    }
}

/// Structurally invalid input: refused before any rule is applied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Invalid {
    TooManyCandidates { count: usize, limit: usize },
    Identity { recipe: String },
    DuplicateIdentity { recipe: String },
    BaselineAmongCandidates { recipe: String },
    BaselineMismatch { declared: String, supplied: String },
    Capabilities { recipe: Option<String> },
    QualityRange { recipe: Option<String> },
}

impl fmt::Display for Invalid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyCandidates { count, limit } => {
                write!(formatter, "{count} candidates exceed the bound of {limit}")
            }
            Self::Identity { recipe } => write!(formatter, "invalid recipe identity {recipe:?}"),
            Self::DuplicateIdentity { recipe } => {
                write!(formatter, "duplicate recipe identity {recipe:?}")
            }
            Self::BaselineAmongCandidates { recipe } => {
                write!(formatter, "baseline {recipe:?} is also a candidate")
            }
            Self::BaselineMismatch { declared, supplied } => write!(
                formatter,
                "policy declares baseline {declared:?}, caller supplied {supplied:?}"
            ),
            Self::Capabilities {
                recipe: Some(recipe),
            } => {
                write!(formatter, "invalid capabilities on recipe {recipe:?}")
            }
            Self::Capabilities { recipe: None } => {
                write!(formatter, "invalid required capabilities on the task")
            }
            Self::QualityRange {
                recipe: Some(recipe),
            } => {
                write!(formatter, "quality figure out of range on {recipe:?}")
            }
            Self::QualityRange { recipe: None } => {
                write!(formatter, "quality floor out of range on the task")
            }
        }
    }
}

impl std::error::Error for Invalid {}

/// The declared values of a route configuration, as read or as supplied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    pub schema_version: i64,
    pub filters: Vec<String>,
    pub ranking: Vec<String>,
    pub tie: String,
    pub staleness_bound_ms: u64,
    pub baseline: String,
}

/// Why a declaration was refused, by name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    Syntax,
    UnknownKey { key: String },
    MissingKey { key: String },
    WrongType { key: String },
    SchemaVersion { found: i64 },
    UnknownRule { name: String },
    DuplicateRule { name: String },
    MissingRule { name: String },
    UnknownRankingKey { name: String },
    DuplicateRankingKey { name: String },
    MissingRankingKey { name: String },
    UnknownTieRule { name: String },
    StalenessBound { value: i64 },
    MissingBaseline,
    BaselineIdentity { recipe: String },
    BaselineMismatch { declared: String, supplied: String },
    BaselineNotLocal { recipe: String, locality: Locality },
    BaselineMissingFigure { recipe: String, figure: Figure },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax => write!(formatter, "route configuration is not valid TOML"),
            Self::UnknownKey { key } => write!(formatter, "unknown key {key:?}"),
            Self::MissingKey { key } => write!(formatter, "missing key {key:?}"),
            Self::WrongType { key } => write!(formatter, "wrong type at key {key:?}"),
            Self::SchemaVersion { found } => {
                write!(formatter, "unsupported schema_version {found}")
            }
            Self::UnknownRule { name } => write!(formatter, "unknown filter rule {name:?}"),
            Self::DuplicateRule { name } => write!(formatter, "duplicate filter rule {name:?}"),
            Self::MissingRule { name } => write!(formatter, "missing filter rule {name:?}"),
            Self::UnknownRankingKey { name } => write!(formatter, "unknown ranking key {name:?}"),
            Self::DuplicateRankingKey { name } => {
                write!(formatter, "duplicate ranking key {name:?}")
            }
            Self::MissingRankingKey { name } => write!(formatter, "missing ranking key {name:?}"),
            Self::UnknownTieRule { name } => write!(formatter, "unknown tie rule {name:?}"),
            Self::StalenessBound { value } => write!(
                formatter,
                "staleness_bound_ms {value} is outside 1..={MAX_STALENESS_MS}"
            ),
            Self::MissingBaseline => write!(formatter, "no baseline recipe is declared"),
            Self::BaselineIdentity { recipe } => {
                write!(formatter, "invalid baseline identity {recipe:?}")
            }
            Self::BaselineMismatch { declared, supplied } => write!(
                formatter,
                "declared baseline {declared:?} is not the supplied baseline {supplied:?}"
            ),
            Self::BaselineNotLocal { recipe, locality } => write!(
                formatter,
                "baseline {recipe:?} is {locality:?}, not local, and cannot serve a local-only task"
            ),
            Self::BaselineMissingFigure { recipe, figure } => write!(
                formatter,
                "baseline {recipe:?} carries no {figure:?} figure and cannot be screened"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

/// A validated route policy. Constructed only through [`Policy::load`] (the one
/// place a configuration is read) or [`Policy::declare`] (the same validation
/// over supplied values); its fields cannot be set directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    filters: Vec<Filter>,
    ranking: Vec<Key>,
    tie: TieRule,
    baseline: String,
    staleness_bound_ms: u64,
}

impl Policy {
    /// Read a `routes.toml` document and validate it against the supplied baseline recipe.
    ///
    /// # Errors
    /// Refuses, by name, malformed TOML, an unknown, missing or mistyped key, an
    /// unsupported schema version, an unknown, duplicate or missing filter rule
    /// or ranking key, an unknown tie rule, a staleness bound outside
    /// `1..=MAX_STALENESS_MS`, a missing baseline, a baseline other than the one
    /// supplied, and a baseline ineligible by construction (not local, or
    /// lacking a figure the filters and ranking read).
    pub fn load(source: &str, baseline: &Recipe<'_>) -> Result<Self, ConfigError> {
        let table: toml::Table = source.parse().map_err(|_| ConfigError::Syntax)?;
        Self::declare(declaration(&table)?, baseline)
    }

    /// Validate supplied declaration values against the supplied baseline recipe.
    ///
    /// # Errors
    /// The same refusals as [`Policy::load`], except the TOML-only ones.
    pub fn declare(declaration: Declaration, baseline: &Recipe<'_>) -> Result<Self, ConfigError> {
        if declaration.schema_version != SCHEMA_VERSION {
            return Err(ConfigError::SchemaVersion {
                found: declaration.schema_version,
            });
        }
        let filters = ordered_filters(&declaration.filters)?;
        let ranking = ordered_keys(&declaration.ranking)?;
        let tie = match declaration.tie.as_str() {
            "baseline" => TieRule::Baseline,
            other => {
                return Err(ConfigError::UnknownTieRule {
                    name: other.to_owned(),
                });
            }
        };
        if !(1..=MAX_STALENESS_MS).contains(&declaration.staleness_bound_ms) {
            return Err(ConfigError::StalenessBound {
                value: i64::try_from(declaration.staleness_bound_ms).unwrap_or(i64::MAX),
            });
        }
        if declaration.baseline.is_empty() {
            return Err(ConfigError::MissingBaseline);
        }
        if !text_valid(&declaration.baseline) {
            return Err(ConfigError::BaselineIdentity {
                recipe: declaration.baseline,
            });
        }
        if declaration.baseline != baseline.id {
            return Err(ConfigError::BaselineMismatch {
                declared: declaration.baseline,
                supplied: baseline.id.to_owned(),
            });
        }
        if baseline.locality != Locality::Local {
            return Err(ConfigError::BaselineNotLocal {
                recipe: declaration.baseline,
                locality: baseline.locality,
            });
        }
        if let Some(figure) = ranking
            .iter()
            .map(|key| key.figure())
            .find(|figure| baseline_figure(baseline, *figure).is_none())
        {
            return Err(ConfigError::BaselineMissingFigure {
                recipe: declaration.baseline,
                figure,
            });
        }
        Ok(Self {
            filters,
            ranking,
            tie,
            baseline: declaration.baseline,
            staleness_bound_ms: declaration.staleness_bound_ms,
        })
    }

    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    #[must_use]
    pub fn ranking(&self) -> &[Key] {
        &self.ranking
    }

    #[must_use]
    pub const fn tie(&self) -> TieRule {
        self.tie
    }

    #[must_use]
    pub fn baseline(&self) -> &str {
        &self.baseline
    }

    #[must_use]
    pub const fn staleness_bound_ms(&self) -> u64 {
        self.staleness_bound_ms
    }
}

fn declaration(table: &toml::Table) -> Result<Declaration, ConfigError> {
    const KEYS: [&str; 6] = [
        "schema_version",
        "filters",
        "ranking",
        "tie",
        "staleness_bound_ms",
        "baseline",
    ];
    if let Some(key) = table.keys().find(|key| !KEYS.contains(&key.as_str())) {
        return Err(ConfigError::UnknownKey { key: key.clone() });
    }
    let baseline = match table.get("baseline") {
        None => return Err(ConfigError::MissingBaseline),
        Some(toml::Value::Table(inner)) => {
            if let Some(key) = inner.keys().find(|key| key.as_str() != "recipe") {
                return Err(ConfigError::UnknownKey {
                    key: format!("baseline.{key}"),
                });
            }
            match inner.get("recipe") {
                None => return Err(ConfigError::MissingBaseline),
                Some(toml::Value::String(recipe)) => recipe.clone(),
                Some(_) => {
                    return Err(ConfigError::WrongType {
                        key: "baseline.recipe".to_owned(),
                    });
                }
            }
        }
        Some(_) => {
            return Err(ConfigError::WrongType {
                key: "baseline".to_owned(),
            });
        }
    };
    let staleness = integer(table, "staleness_bound_ms")?;
    Ok(Declaration {
        schema_version: integer(table, "schema_version")?,
        filters: strings(table, "filters")?,
        ranking: strings(table, "ranking")?,
        tie: string(table, "tie")?,
        staleness_bound_ms: u64::try_from(staleness)
            .map_err(|_| ConfigError::StalenessBound { value: staleness })?,
        baseline,
    })
}

fn required<'t>(table: &'t toml::Table, key: &str) -> Result<&'t toml::Value, ConfigError> {
    table.get(key).ok_or_else(|| ConfigError::MissingKey {
        key: key.to_owned(),
    })
}

fn integer(table: &toml::Table, key: &str) -> Result<i64, ConfigError> {
    match required(table, key)? {
        toml::Value::Integer(value) => Ok(*value),
        _ => Err(ConfigError::WrongType {
            key: key.to_owned(),
        }),
    }
}

fn string(table: &toml::Table, key: &str) -> Result<String, ConfigError> {
    match required(table, key)? {
        toml::Value::String(value) => Ok(value.clone()),
        _ => Err(ConfigError::WrongType {
            key: key.to_owned(),
        }),
    }
}

fn strings(table: &toml::Table, key: &str) -> Result<Vec<String>, ConfigError> {
    let toml::Value::Array(items) = required(table, key)? else {
        return Err(ConfigError::WrongType {
            key: key.to_owned(),
        });
    };
    items
        .iter()
        .map(|item| match item {
            toml::Value::String(value) => Ok(value.clone()),
            _ => Err(ConfigError::WrongType {
                key: key.to_owned(),
            }),
        })
        .collect()
}

fn ordered_filters(names: &[String]) -> Result<Vec<Filter>, ConfigError> {
    let mut seen = Vec::with_capacity(names.len());
    for name in names {
        let filter =
            Filter::named(name).ok_or_else(|| ConfigError::UnknownRule { name: name.clone() })?;
        if seen.contains(&filter) {
            return Err(ConfigError::DuplicateRule { name: name.clone() });
        }
        seen.push(filter);
    }
    if let Some(missing) = Filter::ALL
        .into_iter()
        .find(|filter| !seen.contains(filter))
    {
        return Err(ConfigError::MissingRule {
            name: missing.name().to_owned(),
        });
    }
    Ok(seen)
}

fn ordered_keys(names: &[String]) -> Result<Vec<Key>, ConfigError> {
    let mut seen = Vec::with_capacity(names.len());
    for name in names {
        let key = Key::named(name)
            .ok_or_else(|| ConfigError::UnknownRankingKey { name: name.clone() })?;
        if seen.contains(&key) {
            return Err(ConfigError::DuplicateRankingKey { name: name.clone() });
        }
        seen.push(key);
    }
    if let Some(missing) = Key::ALL.into_iter().find(|key| !seen.contains(key)) {
        return Err(ConfigError::MissingRankingKey {
            name: missing.name().to_owned(),
        });
    }
    Ok(seen)
}

fn text_valid(value: &str) -> bool {
    (1..=MAX_TEXT).contains(&value.len())
        && value.is_ascii()
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn capabilities_valid(values: &[&str]) -> bool {
    values.len() <= MAX_CAPABILITIES
        && values.iter().all(|value| text_valid(value))
        && values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn baseline_figure(recipe: &Recipe<'_>, figure: Figure) -> Option<u64> {
    match figure {
        Figure::Cost => recipe.cost_microunits,
        Figure::Quality => recipe.quality_basis_points.map(u64::from),
        Figure::Latency => recipe.latency_ms,
    }
}

fn validate_recipe(recipe: &Recipe<'_>) -> Result<(), Invalid> {
    if !text_valid(recipe.id) || !text_valid(recipe.revision) {
        return Err(Invalid::Identity {
            recipe: recipe.id.to_owned(),
        });
    }
    if !capabilities_valid(recipe.capabilities) {
        return Err(Invalid::Capabilities {
            recipe: Some(recipe.id.to_owned()),
        });
    }
    if recipe
        .quality_basis_points
        .is_some_and(|quality| quality > MAX_QUALITY_BASIS_POINTS)
    {
        return Err(Invalid::QualityRange {
            recipe: Some(recipe.id.to_owned()),
        });
    }
    Ok(())
}

fn validate(
    policy: &Policy,
    task: &Task<'_>,
    candidates: &[Recipe<'_>],
    baseline: &Recipe<'_>,
) -> Result<(), Invalid> {
    if candidates.len() > MAX_CANDIDATES {
        return Err(Invalid::TooManyCandidates {
            count: candidates.len(),
            limit: MAX_CANDIDATES,
        });
    }
    if !capabilities_valid(task.required_capabilities) {
        return Err(Invalid::Capabilities { recipe: None });
    }
    if task
        .quality_floor_basis_points
        .is_some_and(|floor| floor > MAX_QUALITY_BASIS_POINTS)
    {
        return Err(Invalid::QualityRange { recipe: None });
    }
    validate_recipe(baseline)?;
    if baseline.id != policy.baseline {
        return Err(Invalid::BaselineMismatch {
            declared: policy.baseline.clone(),
            supplied: baseline.id.to_owned(),
        });
    }
    let mut ids = BTreeSet::new();
    for candidate in candidates {
        validate_recipe(candidate)?;
        if candidate.id == baseline.id {
            return Err(Invalid::BaselineAmongCandidates {
                recipe: candidate.id.to_owned(),
            });
        }
        if !ids.insert(candidate.id) {
            return Err(Invalid::DuplicateIdentity {
                recipe: candidate.id.to_owned(),
            });
        }
    }
    Ok(())
}

/// Why a screening stopped: the only two non-pass outcomes a rule can have.
#[derive(Clone, Copy)]
enum Stop<'a> {
    Excluded(Exclusion<'a>),
    Gap(Gap),
}

fn apply_filter<'a>(
    filter: Filter,
    policy: &Policy,
    task: &Task<'a>,
    recipe: &Recipe<'a>,
) -> Option<Stop<'a>> {
    match filter {
        Filter::RequiredCapabilities => task
            .required_capabilities
            .iter()
            .find(|required| !recipe.capabilities.contains(required))
            .map(|capability| Stop::Excluded(Exclusion::MissingCapability { capability })),
        Filter::ContextLimit => (recipe.context_limit_tokens < task.context_tokens).then_some(
            Stop::Excluded(Exclusion::ContextExceeded {
                limit_tokens: recipe.context_limit_tokens,
                required_tokens: task.context_tokens,
            }),
        ),
        Filter::PrivacyClass => (task.privacy == PrivacyClass::LocalOnly
            && recipe.locality != Locality::Local)
            .then_some(Stop::Excluded(Exclusion::PrivacyViolated {
                locality: recipe.locality,
            })),
        Filter::Availability => match recipe.availability {
            Observation::Unobserved => Some(Stop::Gap(Gap::Unobserved)),
            Observation::Observed { age_ms, .. } if age_ms >= policy.staleness_bound_ms => {
                Some(Stop::Gap(Gap::StaleAvailability {
                    age_ms,
                    bound_ms: policy.staleness_bound_ms,
                }))
            }
            Observation::Observed {
                availability: Availability::Available,
                ..
            } => None,
            Observation::Observed {
                availability: Availability::Unavailable,
                age_ms,
            } => Some(Stop::Excluded(Exclusion::Unavailable { age_ms })),
            Observation::Observed {
                availability: Availability::Unknown,
                age_ms,
            } => Some(Stop::Gap(Gap::AvailabilityUnknown { age_ms })),
        },
        Filter::CostCeiling => match (task.cost_ceiling_microunits, recipe.cost_microunits) {
            (None, _) => None,
            (Some(_), None) => Some(Stop::Gap(Gap::MissingFigure {
                figure: Figure::Cost,
            })),
            (Some(ceiling), Some(cost)) => {
                (cost > ceiling).then_some(Stop::Excluded(Exclusion::CostAboveCeiling {
                    cost_microunits: cost,
                    ceiling_microunits: ceiling,
                }))
            }
        },
        Filter::Deadline => match (task.deadline_ms, recipe.latency_ms) {
            (None, _) => None,
            (Some(_), None) => Some(Stop::Gap(Gap::MissingFigure {
                figure: Figure::Latency,
            })),
            (Some(deadline), Some(latency)) => {
                (latency > deadline).then_some(Stop::Excluded(Exclusion::LatencyAboveDeadline {
                    latency_ms: latency,
                    deadline_ms: deadline,
                }))
            }
        },
        Filter::QualityFloor => {
            match (task.quality_floor_basis_points, recipe.quality_basis_points) {
                (None, _) => None,
                (Some(_), None) => Some(Stop::Gap(Gap::MissingFigure {
                    figure: Figure::Quality,
                })),
                (Some(floor), Some(quality)) => {
                    (quality < floor).then_some(Stop::Excluded(Exclusion::QualityBelowFloor {
                        quality_basis_points: quality,
                        floor_basis_points: floor,
                    }))
                }
            }
        }
    }
}

/// What screening one recipe produced: the rules it passed and how it ended.
struct Screening<'a> {
    passed: Vec<Rule>,
    verdict: Verdict<'a>,
}

enum Verdict<'a> {
    /// Every filter passed and every ranking figure is present.
    Eligible(Ranked<'a>),
    /// The named rule stopped the screening.
    Stopped { rule: Rule, stop: Stop<'a> },
}

/// Apply every filter in the declared order, then read every ranking figure in
/// the declared key order. The first non-pass stops the screening for that recipe.
fn screen<'a>(policy: &Policy, task: &Task<'a>, recipe: &Recipe<'a>) -> Screening<'a> {
    let mut passed = Vec::with_capacity(policy.filters.len());
    for filter in &policy.filters {
        if let Some(stop) = apply_filter(*filter, policy, task, recipe) {
            return Screening {
                passed,
                verdict: Verdict::Stopped {
                    rule: filter.rule(),
                    stop,
                },
            };
        }
        passed.push(filter.rule());
    }
    // The declaration names every key exactly once, so every field below is
    // written before the value is read; the placeholders never survive.
    let mut ranked = Ranked {
        recipe: recipe.id,
        cost_microunits: 0,
        quality_basis_points: 0,
        latency_ms: 0,
    };
    for key in &policy.ranking {
        let present = match key {
            Key::Cost => recipe
                .cost_microunits
                .map(|cost| ranked.cost_microunits = cost),
            Key::Quality => recipe
                .quality_basis_points
                .map(|quality| ranked.quality_basis_points = quality),
            Key::Latency => recipe.latency_ms.map(|latency| ranked.latency_ms = latency),
        };
        if present.is_none() {
            return Screening {
                passed,
                verdict: Verdict::Stopped {
                    rule: Rule::R11Ranking,
                    stop: Stop::Gap(Gap::MissingFigure {
                        figure: key.figure(),
                    }),
                },
            };
        }
    }
    Screening {
        passed,
        verdict: Verdict::Eligible(ranked),
    }
}

/// The declared total preorder: keys in declared order; lower cost, higher
/// quality and lower latency first. Equal on every key is a tie.
fn compare(ranking: &[Key], left: &Ranked<'_>, right: &Ranked<'_>) -> Ordering {
    ranking
        .iter()
        .map(|key| match key {
            Key::Cost => left.cost_microunits.cmp(&right.cost_microunits),
            Key::Quality => right.quality_basis_points.cmp(&left.quality_basis_points),
            Key::Latency => left.latency_ms.cmp(&right.latency_ms),
        })
        .find(|ordering| ordering.is_ne())
        .unwrap_or(Ordering::Equal)
}

/// Screen the baseline first: its step leads the explanation. A defect is held, not
/// decided; it refuses only when a fallback later requires the baseline.
fn guard_baseline<'a>(
    policy: &Policy,
    task: &Task<'a>,
    baseline: &Recipe<'a>,
    explanation: &mut Explanation<'a>,
) -> Option<Refusal<'a>> {
    let guard = screen(policy, task, baseline);
    match guard.verdict {
        Verdict::Eligible(_) => {
            explanation.steps.push(Step::Guarded {
                recipe: baseline.id,
                passed: guard.passed,
            });
            None
        }
        Verdict::Stopped {
            rule,
            stop: Stop::Excluded(why),
        } => {
            explanation.steps.push(Step::Excluded {
                recipe: baseline.id,
                passed: guard.passed,
                rule,
                why,
            });
            Some(Refusal::BaselineExcluded { rule, why })
        }
        Verdict::Stopped {
            rule,
            stop: Stop::Gap(evidence),
        } => {
            explanation.steps.push(Step::Gap {
                recipe: baseline.id,
                passed: guard.passed,
                rule,
                evidence,
            });
            Some(Refusal::BaselineEvidence { rule, evidence })
        }
    }
}

/// Screen every candidate in identity order, recording one step each; return the
/// eligible recipes with their ranking figures and every evidence gap, both in that order.
fn screen_candidates<'a, 'c>(
    policy: &Policy,
    task: &Task<'a>,
    candidates: &'c [Recipe<'a>],
    explanation: &mut Explanation<'a>,
) -> (Vec<(&'c Recipe<'a>, Ranked<'a>)>, Vec<EvidenceGap<'a>>) {
    let mut ordered: Vec<&'c Recipe<'a>> = candidates.iter().collect();
    ordered.sort_by(|left, right| left.id.cmp(right.id));
    let mut eligible = Vec::new();
    let mut gaps = Vec::new();
    for recipe in ordered {
        let screening = screen(policy, task, recipe);
        match screening.verdict {
            Verdict::Eligible(ranked) => {
                explanation.steps.push(Step::Eligible {
                    recipe: recipe.id,
                    passed: screening.passed,
                });
                eligible.push((recipe, ranked));
            }
            Verdict::Stopped {
                rule,
                stop: Stop::Excluded(why),
            } => explanation.steps.push(Step::Excluded {
                recipe: recipe.id,
                passed: screening.passed,
                rule,
                why,
            }),
            Verdict::Stopped {
                rule,
                stop: Stop::Gap(evidence),
            } => {
                explanation.steps.push(Step::Gap {
                    recipe: recipe.id,
                    passed: screening.passed,
                    rule,
                    evidence,
                });
                gaps.push(EvidenceGap {
                    recipe: recipe.id,
                    rule,
                    evidence,
                });
            }
        }
    }
    (eligible, gaps)
}

/// Decide a route for one task over the candidate recipes and the explicit baseline.
///
/// The baseline is screened first and candidates are screened in identity order,
/// so the decision and its explanation do not depend on the order the caller
/// supplied the candidates in.
///
/// # Errors
/// Refuses structurally invalid input before any rule is applied: more than
/// `MAX_CANDIDATES` recipes, an invalid or duplicate identity, invalid
/// capability lists, a quality value above `MAX_QUALITY_BASIS_POINTS`, a
/// baseline that is also a candidate, or a baseline other than the policy's.
pub fn route<'a>(
    policy: &Policy,
    task: &Task<'a>,
    candidates: &[Recipe<'a>],
    baseline: &Recipe<'a>,
) -> Result<Route<'a>, Invalid> {
    validate(policy, task, candidates, baseline)?;
    let mut explanation = Explanation::default();
    let defect = guard_baseline(policy, task, baseline, &mut explanation);
    let (mut eligible, gaps) = screen_candidates(policy, task, candidates, &mut explanation);
    eligible.sort_by(|(_, left), (_, right)| compare(&policy.ranking, left, right));
    let fallback = if !gaps.is_empty() {
        explanation.steps.push(Step::Decided {
            rule: Rule::R09InsufficientEvidence,
        });
        Fallback::InsufficientEvidence { gaps }
    } else if let Some((best, best_ranked)) = eligible.first().copied() {
        let between: Vec<&'a str> = eligible
            .iter()
            .take_while(|(_, other)| {
                compare(&policy.ranking, &best_ranked, other) == Ordering::Equal
            })
            .map(|(recipe, _)| recipe.id)
            .collect();
        explanation.steps.push(Step::Ranked {
            rule: Rule::R11Ranking,
            order: eligible.iter().map(|(_, ranked)| *ranked).collect(),
        });
        if between.len() == 1 {
            explanation.steps.push(Step::Decided {
                rule: Rule::R11Ranking,
            });
            return Ok(Route::Chosen {
                recipe: best.id,
                revision: best.revision,
                explanation,
            });
        }
        explanation.steps.push(Step::Decided { rule: Rule::R12Tie });
        let TieRule::Baseline = policy.tie;
        Fallback::Tie { between }
    } else {
        explanation.steps.push(Step::Decided {
            rule: Rule::R10NoEligibleCandidate,
        });
        Fallback::NoEligibleCandidate
    };
    match defect {
        None => Ok(Route::Baseline {
            recipe: baseline.id,
            revision: baseline.revision,
            reason: fallback,
            explanation,
        }),
        Some(reason) => {
            explanation.steps.push(Step::Decided {
                rule: Rule::R01BaselineGuard,
            });
            Ok(Route::Refused {
                fallback,
                reason,
                explanation,
            })
        }
    }
}
