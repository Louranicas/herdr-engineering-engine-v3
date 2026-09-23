// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/notify.rs
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
// Readiness binding: HEE3-READINESS-001; SHA-256 e25e29c671f6e570e17b57b5056f84eb97b312cf23937cfd14df5b1ad47acfa6; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-05, R90-07, R90-08, R90-09, R90-10; resolved contracts RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-notify; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-notify (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-notify
// Owns: Delivery/replay/deduplication of actionable outcomes
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/notify.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-notify)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T04, T07, T11, T14, T17, T18, T19, T20, T25, T26, T27
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K3](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K3)
// [contributing codebase CODE-CB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB04)
// [contributing codebase CODE-CB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB05)
// [contributing codebase CODE-CB11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB11)
// [task TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
// [task TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
// [task TASK-T11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T11)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [separate reference example EX-notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-notify)
// [flow FLOW-F10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F10)
// [handbook HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [handbook HB-socket-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-socket-map)
// [API API-API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01)
// [API API-API02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API02)
// [action ACT-events.subscribe](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-events.subscribe)
// [IPC IPC-IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-notify)
// [plan SEC-api-sockets](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-api-sockets)
// [plan SEC-economy](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-economy)
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
// [schematic SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
// [schematic SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
// [schematic SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
// [schematic SC-SC12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC12)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC21)
// [source SRC-D15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-D15)
// [source SRC-H12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-H12)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-notify)
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
// [readiness improvement grouping R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
// [readiness improvement grouping R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
// [readiness improvement grouping R90-08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-08)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-notify)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
// [applied learning LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
// [diary evidence source DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
// Applicable learning IDs: LRN01, LRN03, LRN08, LRN13; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [Why I Stopped Trusting Green](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhy%20I%20Stopped%20Trusting%20Green)
// [Working in Sandboxes on Kinoite](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20in%20Sandboxes%20on%20Kinoite)
// HEE3-ANCHORS-END

//! Committed-event subscription and delivery.
//!
//! `docs/modules/notify.md` fixes the complexity boundary: *"Notifications do not become a
//! task-state owner or redispatch engine effects."* Nothing here decides task state; every
//! type reports delivery, and the only writes are to this module's own cursors and receipts.
//!
//! Two of the contract's required proofs are discharged by the type system rather than by a
//! test that must remember to run:
//!
//! * **Commit before delivery.** [`Outbox::enqueue`] takes a [`Committed`], and a
//!   [`Committed`] can only be minted by [`Commit::witness`] — a token the ledger hands out
//!   *after* it has committed. There is no constructor that turns an uncommitted event into
//!   a deliverable one, so "pre-commit delivery" is unrepresentable rather than refused.
//! * **Dedup by durable identity.** A delivery is keyed on `(event, recipient)`, so a
//!   retried delivery after a lost acknowledgement is idempotent even though the transport
//!   cannot tell the two attempts apart.
//!
//! Backlog is bounded at [`MAX_BACKLOG`] and a replay window at [`MAX_REPLAY`]. A subscriber
//! that falls behind receives a [`Gap`] naming what it missed, never a silently truncated
//! stream: a consumer that cannot tell "nothing happened" from "I missed it" will eventually
//! act on the wrong one.

use std::collections::BTreeMap;
use std::fmt;

use crate::contracts::{ScalarError, UuidV4};

/// The most events one outbox retains for replay.
pub const MAX_BACKLOG: usize = 8192;

/// The most events one `subscribe` call returns, so a caller cannot ask the outbox to
/// materialise its whole backlog in one allocation.
pub const MAX_REPLAY: usize = 512;

/// The most recipients one event may be delivered to.
pub const MAX_RECIPIENTS: usize = 256;

/// Schema version of the persisted outbox shape owned by `store`.
pub const SCHEMA_VERSION: i64 = 1;

/// Who may observe an event.
///
/// Filtering happens where the event is read, not where it is written, so a visibility
/// change cannot retroactively leak an event that was already enqueued.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Visibility {
    /// Only the principal that owns the task.
    Owner,
    /// The owner and any operator role.
    Operator,
    /// Anything that may read the engine at all.
    Public,
}

impl Visibility {
    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Operator => "operator",
            Self::Public => "public",
        }
    }

    /// Every visibility, widest last.
    pub const ALL: [Self; 3] = [Self::Owner, Self::Operator, Self::Public];

    /// Whether a caller at `self` may observe an event marked `event`.
    ///
    /// The order is a total order on breadth: an operator sees operator and public events,
    /// an owner sees everything in its own scope.
    #[must_use]
    pub const fn admits(self, event: Self) -> bool {
        (self as u8) <= (event as u8)
    }

    /// Parse a wire name.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownVisibility`] for a name outside [`Visibility::ALL`].
    pub fn parse(name: &str) -> Result<Self, Refusal> {
        Self::ALL
            .into_iter()
            .find(|value| value.name() == name)
            .ok_or(Refusal::UnknownVisibility)
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Why a delivery has not completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureCategory {
    /// The recipient was unreachable. Retry is expected to help.
    Unreachable,
    /// The recipient rejected the event. Retry is not expected to help.
    Rejected,
    /// The attempt neither succeeded nor failed observably.
    Indeterminate,
}

impl FailureCategory {
    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unreachable => "unreachable",
            Self::Rejected => "rejected",
            Self::Indeterminate => "indeterminate",
        }
    }

    /// Every category.
    pub const ALL: [Self; 3] = [Self::Unreachable, Self::Rejected, Self::Indeterminate];

    /// Whether another attempt is worth making.
    ///
    /// [`FailureCategory::Indeterminate`] is retryable **and** stays visible: the engine
    /// cannot prove the event did not arrive, so the obligation is not discharged.
    #[must_use]
    pub const fn retryable(self) -> bool {
        matches!(self, Self::Unreachable | Self::Indeterminate)
    }
}

/// A reason this module refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// An identity that is not a lowercase hyphenated `UUIDv4`.
    MalformedIdentity(ScalarError),
    /// A visibility name outside [`Visibility::ALL`].
    UnknownVisibility,
    /// The outbox already holds [`MAX_BACKLOG`] events.
    BacklogFull,
    /// One event was offered to more than [`MAX_RECIPIENTS`] recipients.
    RecipientLimit,
    /// A replay was asked for more than [`MAX_REPLAY`] events.
    ReplayTooWide,
    /// The cursor names a sequence the outbox has already discarded.
    CursorExpired,
    /// The cursor names a sequence beyond anything committed.
    CursorAhead,
    /// The caller's epoch is not the outbox's epoch; a resync is required.
    EpochMismatch,
    /// The event is not in this outbox.
    UnknownEvent,
    /// A second event claimed an identity already enqueued.
    DuplicateEvent,
    /// A sequence number left `u64`.
    SequenceOverflow,
}

impl Refusal {
    /// The stable diagnostic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MalformedIdentity(_) => "malformed notify identity",
            Self::UnknownVisibility => "unknown visibility",
            Self::BacklogFull => "outbox backlog bound reached",
            Self::RecipientLimit => "recipient count bound reached",
            Self::ReplayTooWide => "replay window bound exceeded",
            Self::CursorExpired => "cursor precedes the retained backlog",
            Self::CursorAhead => "cursor follows the committed sequence",
            Self::EpochMismatch => "epoch differs; resync required",
            Self::UnknownEvent => "unknown event",
            Self::DuplicateEvent => "duplicate event identity",
            Self::SequenceOverflow => "sequence exceeds the permitted integer range",
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

/// Proof that the ledger committed, handed out by `store` after its transaction closes.
///
/// This type has no public constructor from thin air: [`Commit::witness`] is the only way to
/// mint one, and the ledger is the only caller positioned to invoke it truthfully. It exists
/// so that `Outbox::enqueue` can demand evidence of commit in its **signature** rather than
/// documenting a rule someone must remember.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Commit {
    epoch: u64,
    sequence: u64,
}

impl Commit {
    /// Witness that the ledger committed `sequence` in `epoch`.
    ///
    /// Callers other than the committing store must not invoke this; the engine has no way
    /// to enforce that beyond keeping the outbox the only consumer of the result.
    #[must_use]
    pub const fn witness(epoch: u64, sequence: u64) -> Self {
        Self { epoch, sequence }
    }

    /// The epoch this commit belongs to.
    #[must_use]
    pub const fn epoch(self) -> u64 {
        self.epoch
    }

    /// The committed ledger sequence.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

/// An event that has been committed and may therefore be delivered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Committed<'a> {
    identity: UuidV4<'a>,
    visibility: Visibility,
    commit: Commit,
}

impl<'a> Committed<'a> {
    /// Bind an event identity to its commit.
    ///
    /// # Errors
    ///
    /// [`Refusal::MalformedIdentity`] when `identity` is not a lowercase hyphenated
    /// `UUIDv4`.
    pub fn new(identity: &'a str, visibility: Visibility, commit: Commit) -> Result<Self, Refusal> {
        Ok(Self {
            identity: UuidV4::parse(identity).map_err(Refusal::MalformedIdentity)?,
            visibility,
            commit,
        })
    }

    /// The event identity.
    #[must_use]
    pub const fn identity(self) -> UuidV4<'a> {
        self.identity
    }

    /// Who may observe it.
    #[must_use]
    pub const fn visibility(self) -> Visibility {
        self.visibility
    }

    /// The commit that made it deliverable.
    #[must_use]
    pub const fn commit(self) -> Commit {
        self.commit
    }
}

/// Where a subscriber has read to.
///
/// A cursor is a sequence, not an index: the outbox may discard old events, and a cursor
/// that falls behind the retained window is [`Refusal::CursorExpired`] rather than a silent
/// jump forward.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cursor {
    epoch: u64,
    after: u64,
}

impl Cursor {
    /// A cursor positioned after `sequence` in `epoch`.
    #[must_use]
    pub const fn after(epoch: u64, sequence: u64) -> Self {
        Self {
            epoch,
            after: sequence,
        }
    }

    /// A cursor at the start of `epoch`.
    #[must_use]
    pub const fn genesis(epoch: u64) -> Self {
        Self { epoch, after: 0 }
    }

    /// The epoch this cursor belongs to.
    #[must_use]
    pub const fn epoch(self) -> u64 {
        self.epoch
    }

    /// The last sequence the subscriber has seen.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.after
    }
}

/// What a subscriber missed, when the outbox could not serve its cursor from the backlog.
///
/// A gap is returned rather than papered over. A consumer that receives an empty stream and
/// a consumer that receives a gap must behave differently, and only one of them can be told
/// apart from "nothing happened" after the fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Gap {
    /// The first sequence the outbox still retains.
    pub retained_from: u64,
    /// The cursor the subscriber offered.
    pub requested_after: u64,
}

/// One delivery obligation for one `(event, recipient)` pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivery {
    /// Acknowledged by the recipient. Terminal.
    Delivered,
    /// Not yet acknowledged, with the category of the last failure.
    Pending(FailureCategory),
    /// No attempt has been recorded.
    ///
    /// Distinct from `Pending`: "we have not tried" and "we tried and cannot tell" are
    /// different facts, and collapsing them would let an untried obligation read as an
    /// attempted one.
    Unknown,
}

impl Delivery {
    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Pending(_) => "pending",
            Self::Unknown => "unknown",
        }
    }

    /// Whether the obligation is discharged.
    #[must_use]
    pub const fn is_settled(self) -> bool {
        matches!(self, Self::Delivered)
    }
}

/// One retained event, with its per-recipient obligations.
#[derive(Clone, Debug)]
struct Record {
    identity: String,
    visibility: Visibility,
    sequence: u64,
    deliveries: BTreeMap<String, Delivery>,
}

/// An event as a subscriber sees it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observed<'a> {
    /// The event identity.
    pub identity: UuidV4<'a>,
    /// Who may observe it.
    pub visibility: Visibility,
    /// Its committed sequence.
    pub sequence: u64,
}

/// The result of a subscribe or replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream<'a> {
    /// The permitted events, in committed order.
    pub events: Vec<Observed<'a>>,
    /// Present when the cursor preceded the retained backlog.
    pub gap: Option<Gap>,
    /// Where to resume.
    pub cursor: Cursor,
}

/// Why a recipient was not woken.
///
/// T11's clause is *"actionable wakes are deduplicated/batched; controlled idle and repeated
/// unchanged events cause zero model requests"*. A wake is the thing that would cost a model
/// request, so "zero requests" is discharged by there being no `Wake::Actionable` to answer —
/// not by a caller remembering to skip one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Quiet {
    /// Nothing is outstanding for this recipient.
    Idle,
    /// Everything outstanding was already carried by the recipient's last wake, and nothing
    /// has been acknowledged or committed since. A repeated unchanged event lands here.
    Unchanged,
    /// The recipient is within its declared quiet window.
    Held,
}

impl Quiet {
    /// The stable wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Unchanged => "unchanged",
            Self::Held => "held",
        }
    }
}

/// What one recipient should be woken for, if anything.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Wake {
    /// One batch of actionable events, in committed order. Never empty.
    Actionable {
        /// The events this wake carries.
        events: Vec<String>,
        /// The highest sequence in the batch, which becomes the recipient's wake mark.
        through: u64,
    },
    /// No wake, with the ground.
    Quiet(Quiet),
}

impl Wake {
    /// Whether this wake would cost a model request.
    #[must_use]
    pub const fn is_actionable(&self) -> bool {
        matches!(self, Self::Actionable { .. })
    }

    /// The events carried, empty when quiet.
    #[must_use]
    pub fn events(&self) -> &[String] {
        match self {
            Self::Actionable { events, .. } => events,
            Self::Quiet(_) => &[],
        }
    }
}

/// What one recipient has already been woken for.
///
/// Held by the caller and handed back, so the outbox stays a value: two callers cannot
/// disagree about a recipient's wake state because neither owns it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WakeMark {
    /// The highest sequence already carried by a wake.
    through: u64,
    /// The earliest time a further wake is admitted.
    quiet_until_ms: u64,
}

impl WakeMark {
    /// A mark that has woken for nothing and holds nothing back.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            through: 0,
            quiet_until_ms: 0,
        }
    }

    /// The highest sequence already woken for.
    #[must_use]
    pub const fn through(self) -> u64 {
        self.through
    }

    /// The earliest admitted wake time.
    #[must_use]
    pub const fn quiet_until_ms(self) -> u64 {
        self.quiet_until_ms
    }

    /// The mark after a wake carrying `through`, holding further wakes for `quiet_ms`.
    #[must_use]
    pub const fn advanced(self, through: u64, now_ms: u64, quiet_ms: u64) -> Self {
        Self {
            through,
            quiet_until_ms: now_ms.saturating_add(quiet_ms),
        }
    }
}

/// The committed-event outbox for one epoch.
///
/// It owns delivery bookkeeping and nothing else. There is no method here that changes task
/// state, and none that re-runs an effect: a retry re-sends a **notification**, which is why
/// dedup is keyed on the durable `(event, recipient)` pair rather than on an attempt counter.
#[derive(Clone, Debug)]
pub struct Outbox {
    epoch: u64,
    next_sequence: u64,
    retained_from: u64,
    records: Vec<Record>,
}

impl Outbox {
    /// An empty outbox for `epoch`.
    #[must_use]
    pub const fn new(epoch: u64) -> Self {
        Self {
            epoch,
            next_sequence: 1,
            retained_from: 1,
            records: Vec::new(),
        }
    }

    /// The epoch this outbox serves.
    #[must_use]
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }

    /// The number of retained events.
    #[must_use]
    pub fn retained(&self) -> usize {
        self.records.len()
    }

    /// The first sequence still retained.
    #[must_use]
    pub const fn retained_from(&self) -> u64 {
        self.retained_from
    }

    /// The next sequence this outbox will assign.
    #[must_use]
    pub const fn next_sequence(&self) -> u64 {
        self.next_sequence
    }

    /// Enqueue a committed event for a set of recipients.
    ///
    /// The event must already be [`Committed`], which is the whole of "commit before
    /// delivery": there is no path from an uncommitted event to this call.
    ///
    /// # Errors
    ///
    /// * [`Refusal::EpochMismatch`] when the commit belongs to another epoch — a restored
    ///   epoch must resync rather than interleave;
    /// * [`Refusal::RecipientLimit`] beyond [`MAX_RECIPIENTS`], refused before the record is
    ///   built;
    /// * [`Refusal::BacklogFull`] at [`MAX_BACKLOG`];
    /// * [`Refusal::DuplicateEvent`] when the identity is already retained;
    /// * [`Refusal::MalformedIdentity`] for a recipient that is not a `UUIDv4`;
    /// * [`Refusal::SequenceOverflow`] when the sequence counter would leave `u64`.
    pub fn enqueue(&mut self, event: Committed<'_>, recipients: &[&str]) -> Result<u64, Refusal> {
        if event.commit().epoch() != self.epoch {
            return Err(Refusal::EpochMismatch);
        }
        if recipients.len() > MAX_RECIPIENTS {
            return Err(Refusal::RecipientLimit);
        }
        if self.records.len() >= MAX_BACKLOG {
            return Err(Refusal::BacklogFull);
        }
        if self.find(event.identity().as_str()).is_some() {
            return Err(Refusal::DuplicateEvent);
        }
        let mut deliveries = BTreeMap::new();
        for recipient in recipients {
            let recipient = UuidV4::parse(recipient).map_err(Refusal::MalformedIdentity)?;
            deliveries.insert(recipient.as_str().to_owned(), Delivery::Unknown);
        }
        let sequence = self.next_sequence;
        self.next_sequence = sequence.checked_add(1).ok_or(Refusal::SequenceOverflow)?;
        self.records.push(Record {
            identity: event.identity().as_str().to_owned(),
            visibility: event.visibility(),
            sequence,
            deliveries,
        });
        Ok(sequence)
    }

    /// Read forward from `cursor`, filtered to what `caller` may observe.
    ///
    /// # Errors
    ///
    /// * [`Refusal::EpochMismatch`] when the cursor belongs to another epoch;
    /// * [`Refusal::ReplayTooWide`] when `limit` exceeds [`MAX_REPLAY`] — the bound is taken
    ///   at the point of acquisition, before any event is copied;
    /// * [`Refusal::CursorAhead`] when the cursor names a sequence beyond anything committed.
    ///
    /// A cursor that precedes the retained backlog is **not** an error: it returns a
    /// [`Gap`] alongside whatever is still retained, because the subscriber needs to know it
    /// missed something and needs the events that survived.
    pub fn subscribe(
        &self,
        caller: Visibility,
        cursor: Cursor,
        limit: usize,
    ) -> Result<Stream<'_>, Refusal> {
        if cursor.epoch() != self.epoch {
            return Err(Refusal::EpochMismatch);
        }
        if limit > MAX_REPLAY {
            return Err(Refusal::ReplayTooWide);
        }
        if cursor.sequence() >= self.next_sequence {
            return Err(Refusal::CursorAhead);
        }
        let gap = (cursor.sequence() + 1 < self.retained_from).then_some(Gap {
            retained_from: self.retained_from,
            requested_after: cursor.sequence(),
        });
        let mut events = Vec::new();
        let mut last = cursor.sequence();
        for record in &self.records {
            if record.sequence <= cursor.sequence() {
                continue;
            }
            if events.len() >= limit {
                break;
            }
            last = record.sequence;
            if !caller.admits(record.visibility) {
                continue;
            }
            events.push(Observed {
                identity: UuidV4::parse(record.identity.as_str())
                    .map_err(Refusal::MalformedIdentity)?,
                visibility: record.visibility,
                sequence: record.sequence,
            });
        }
        Ok(Stream {
            events,
            gap,
            cursor: Cursor::after(self.epoch, last),
        })
    }

    /// Record the outcome of one delivery attempt.
    ///
    /// Idempotent by `(event, recipient)`: acknowledging an already-delivered obligation
    /// returns `false` and changes nothing, which is what makes a retry after a lost
    /// acknowledgement safe. A `Delivered` obligation is terminal and a later failure does
    /// **not** reopen it — the recipient already has the event, and re-pending it would
    /// invite a redelivery the contract forbids.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownEvent`] when the event or the recipient is not an obligation of
    /// this outbox, and [`Refusal::MalformedIdentity`] for a recipient that is not a
    /// `UUIDv4`.
    pub fn record(
        &mut self,
        event: &str,
        recipient: &str,
        outcome: Delivery,
    ) -> Result<bool, Refusal> {
        let recipient = UuidV4::parse(recipient).map_err(Refusal::MalformedIdentity)?;
        let index = self.find(event).ok_or(Refusal::UnknownEvent)?;
        let slot = self.records[index]
            .deliveries
            .get_mut(recipient.as_str())
            .ok_or(Refusal::UnknownEvent)?;
        if *slot == Delivery::Delivered {
            return Ok(false);
        }
        if *slot == outcome {
            return Ok(false);
        }
        *slot = outcome;
        Ok(true)
    }

    /// The delivery obligation for one `(event, recipient)` pair.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownEvent`], [`Refusal::MalformedIdentity`].
    pub fn obligation(&self, event: &str, recipient: &str) -> Result<Delivery, Refusal> {
        let recipient = UuidV4::parse(recipient).map_err(Refusal::MalformedIdentity)?;
        let index = self.find(event).ok_or(Refusal::UnknownEvent)?;
        self.records[index]
            .deliveries
            .get(recipient.as_str())
            .copied()
            .ok_or(Refusal::UnknownEvent)
    }

    /// Every obligation that is not yet discharged, oldest event first.
    ///
    /// The result is bounded by [`MAX_REPLAY`] so a caller draining the outbox cannot ask it
    /// to build an unbounded list; the returned count says whether more remain.
    #[must_use]
    pub fn outstanding(&self, limit: usize) -> Vec<(&str, &str, Delivery)> {
        let limit = limit.min(MAX_REPLAY);
        let mut out = Vec::new();
        for record in &self.records {
            for (recipient, delivery) in &record.deliveries {
                if out.len() >= limit {
                    return out;
                }
                if !delivery.is_settled() {
                    out.push((record.identity.as_str(), recipient.as_str(), *delivery));
                }
            }
        }
        out
    }

    /// What `recipient` should be woken for now, batching every outstanding obligation into
    /// one wake.
    ///
    /// Three suppressions, and each is a state rather than a caller's discipline:
    ///
    /// * **Idle.** Nothing outstanding gives [`Quiet::Idle`]. There is no empty
    ///   [`Wake::Actionable`] to construct, so "woken for nothing" is unrepresentable.
    /// * **Unchanged.** Everything outstanding at or below the recipient's wake mark gives
    ///   [`Quiet::Unchanged`]. A producer that re-commits an event the recipient has already
    ///   been woken for therefore costs no request — the clause's *"repeated unchanged
    ///   events cause zero model requests"*.
    /// * **Held.** Before `mark.quiet_until_ms()` the answer is [`Quiet::Held`], whatever is
    ///   outstanding, so a burst is batched into the wake that follows the window rather
    ///   than becoming one request per event.
    ///
    /// The batch is the whole outstanding set, in committed order, so a wake is one request
    /// for many events rather than many requests. `MAX_REPLAY` bounds it; a recipient with
    /// more outstanding than that is woken for the oldest, and the next wake carries the
    /// rest, because a bound that dropped the remainder would discharge a delivery by
    /// forgetting it.
    #[must_use]
    pub fn wake(&self, recipient: &str, mark: WakeMark, now_ms: u64) -> Wake {
        if now_ms < mark.quiet_until_ms() {
            return Wake::Quiet(Quiet::Held);
        }
        let mut events = Vec::new();
        let mut through = mark.through();
        let mut suppressed = false;
        for record in &self.records {
            let Some(delivery) = record.deliveries.get(recipient) else {
                continue;
            };
            if delivery.is_settled() {
                continue;
            }
            if record.sequence <= mark.through() {
                // Outstanding, but the recipient has already been woken for it. Seen, not
                // carried: the distinction is what separates `Unchanged` from `Idle`.
                suppressed = true;
                continue;
            }
            if events.len() >= MAX_REPLAY {
                break;
            }
            events.push(record.identity.clone());
            through = through.max(record.sequence);
        }
        if events.is_empty() {
            return Wake::Quiet(if suppressed {
                Quiet::Unchanged
            } else {
                Quiet::Idle
            });
        }
        Wake::Actionable { events, through }
    }

    /// Discard retained events up to and including `sequence`, keeping their obligations
    /// only if every one is settled.
    ///
    /// An event with an outstanding obligation is **kept**, whatever the caller asked for:
    /// dropping it would discharge a delivery by forgetting it.
    ///
    /// Returns the number discarded.
    pub fn compact(&mut self, through: u64) -> usize {
        let before = self.records.len();
        self.records.retain(|record| {
            record.sequence > through || record.deliveries.values().any(|d| !d.is_settled())
        });
        self.retained_from = self
            .records
            .iter()
            .map(|record| record.sequence)
            .min()
            .unwrap_or(self.next_sequence);
        before - self.records.len()
    }

    /// Begin a new epoch, discarding every retained event.
    ///
    /// A subscriber holding a cursor from the old epoch receives [`Refusal::EpochMismatch`]
    /// and must resync — the contract's *"invalid/restored epoch requires resync"*. The
    /// sequence counter restarts, which is why a cursor carries its epoch: without it, an
    /// old sequence would silently name a new event.
    pub fn restore(&mut self, epoch: u64) {
        self.epoch = epoch;
        self.next_sequence = 1;
        self.retained_from = 1;
        self.records.clear();
    }

    fn find(&self, identity: &str) -> Option<usize> {
        self.records
            .iter()
            .position(|record| record.identity == identity)
    }
}
