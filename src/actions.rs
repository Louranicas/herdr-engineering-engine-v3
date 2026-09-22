// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/src/actions.rs
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
// Readiness binding: HEE3-READINESS-001; SHA-256 7fd2285b611328e2c8d7aef755d55451ef2d4e0f952524526891ca49533fc090; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-03, R90-06, R90-07, R90-09, R90-10; resolved contracts RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-actions; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-actions (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-actions
// Owns: Authoritative typed action catalogue, capability projection and transport dispatch metadata
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/actions.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-actions)
// Build dependencies: contracts
// Consumers: app, bash, pi_extension, skills, workflows
// Related task contracts: T01, T14, T15, T16, T17, T18, T19, T20, T25, T26, T27, T28, T29
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K6](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K6)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [contributing codebase CODE-CB10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB10)
// [contributing codebase CODE-CB11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB11)
// [task TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
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
// [task TASK-T29](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T29)
// [separate reference example EX-actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-actions)
// [flow FLOW-F15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F15)
// [flow FLOW-F16](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F16)
// [flow FLOW-F17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F17)
// [flow FLOW-F20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F20)
// [handbook HB-action-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-action-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [handbook HB-socket-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-socket-map)
// [API API-API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01)
// [API API-API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10)
// [action ACT-tools.inspect](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-tools.inspect)
// [action ACT-tools.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-tools.list)
// [IPC IPC-IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01)
// [IPC IPC-IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-actions)
// [plan SEC-api-sockets](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-api-sockets)
// [plan SEC-llm-tools](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-llm-tools)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-security](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-security)
// [plan SEC-toolchain](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-toolchain)
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
// [schematic SC-SC03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC03)
// [schematic SC-SC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC04)
// [schematic SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
// [schematic SC-SC16](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC16)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC23)
// [schematic SC-SC24](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC24)
// [source SRC-A08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A08)
// [source SRC-A11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A11)
// [source SRC-D19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-D19)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-actions)
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
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
// [readiness improvement grouping R90-06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-06)
// [readiness improvement grouping R90-07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-07)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-actions)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
// [applied learning LRN02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN02)
// [applied learning LRN03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN03)
// [applied learning LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN14)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// [diary evidence source DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
// Applicable learning IDs: LRN01, LRN02, LRN03, LRN04, LRN08, LRN14; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What Prototyping Is For](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20Prototyping%20Is%20For)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// [Why I Stopped Trusting Green](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhy%20I%20Stopped%20Trusting%20Green)
// Readiness binding: HEE3-READINESS-001; SHA-256 7fd2285b611328e2c8d7aef755d55451ef2d4e0f952524526891ca49533fc090; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-02, R90-03, R90-04, R90-05, R90-09, R90-10; resolved contracts RC02, RC03, RC04, RC05; runtime proof pending; original task DAG controls.
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

//! The one versioned action catalogue every transport projects from.
//!
//! `docs/modules/actions.md` fixes the boundary: *"An advertised action is not an authority
//! grant; wrappers must derive from the same owner."* Both halves are structural here.
//!
//! * **Advertising is not granting.** [`Catalogue::list`] and [`Catalogue::inspect`] return
//!   descriptions. The only type that authorises anything is [`Dispatch`], and the only way
//!   to obtain one is [`Catalogue::validate`], which checks the caller's grants against the
//!   action's declared [`Effect`] first. A caller that can see an action still cannot invoke
//!   it.
//! * **One owner, all transports.** [`Dispatch::owner`] comes from the catalogue entry, not
//!   from the request. A CLI wrapper, a tool name and a socket frame that name the same
//!   action produce the same [`Owner`], because none of them can supply one.
//!
//! The catalogue below is the projection of `corpus/PLAN_habitat_engine.json` →
//! `schematic_atlas.actions`. `tests/t28_actions.rs` reads that file and compares it entry
//! by entry: the table is checked against the artefact that declares it, not against itself.

use std::fmt;

/// The number of declared actions. A projection that gains or loses one is a visible change.
pub const DECLARED_ACTIONS: usize = 21;

/// The most entries one `list` page returns.
pub const MAX_PAGE: usize = 32;

/// The catalogue revision this build projects.
pub const CATALOGUE_REVISION: i64 = 1;

/// The module that owns an action's behaviour.
///
/// A wrapper cannot supply one: it is read from the catalogue, which is what makes
/// "wrappers derive from the same owner" true by construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Owner {
    /// The catalogue itself.
    Actions,
    /// The assembled application.
    App,
    /// Specialist thread orchestration.
    Cohort,
    /// Committed-event delivery.
    Notify,
    /// Bounded numerical work.
    Numerical,
    /// The model and agent roster.
    Roster,
    /// Habitat service lifecycle.
    Service,
    /// The task ledger.
    Task,
}

impl Owner {
    /// The stable wire name, which is the owning module's own name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Actions => "actions",
            Self::App => "app",
            Self::Cohort => "cohort",
            Self::Notify => "notify",
            Self::Numerical => "numerical",
            Self::Roster => "roster",
            Self::Service => "service",
            Self::Task => "task",
        }
    }

    /// Every owner.
    pub const ALL: [Self; 8] = [
        Self::Actions,
        Self::App,
        Self::Cohort,
        Self::Notify,
        Self::Numerical,
        Self::Roster,
        Self::Service,
        Self::Task,
    ];
}

impl fmt::Display for Owner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// What invoking an action does.
///
/// The grant a caller must hold is derived from this, so a new effect cannot be added
/// without deciding what authorises it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Effect {
    /// Reads state.
    Read,
    /// Reads a continuing stream of state.
    ReadStream,
    /// Plans without committing anything.
    ReadOnlyPlanning,
    /// Admits durable work.
    DurableAdmission,
    /// Records an intent to cancel.
    CancelIntent,
    /// Records a disposition.
    RecordDisposition,
    /// Mutates configuration.
    ConfigurationMutation,
    /// Runs a bounded, declared probe.
    BoundedProbe,
    /// Drives a managed service lifecycle.
    ManagedLifecycle,
    /// Performs bounded analysis work.
    BoundedAnalysis,
}

impl Effect {
    /// The stable wire name, as the plan spine spells it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::ReadStream => "read stream",
            Self::ReadOnlyPlanning => "read-only planning",
            Self::DurableAdmission => "durable admission",
            Self::CancelIntent => "cancel intent",
            Self::RecordDisposition => "record disposition",
            Self::ConfigurationMutation => "configuration mutation",
            Self::BoundedProbe => "bounded declared probe",
            Self::ManagedLifecycle => "managed lifecycle effect",
            Self::BoundedAnalysis => "bounded analysis work",
        }
    }

    /// Every effect.
    pub const ALL: [Self; 10] = [
        Self::Read,
        Self::ReadStream,
        Self::ReadOnlyPlanning,
        Self::DurableAdmission,
        Self::CancelIntent,
        Self::RecordDisposition,
        Self::ConfigurationMutation,
        Self::BoundedProbe,
        Self::ManagedLifecycle,
        Self::BoundedAnalysis,
    ];

    /// Whether this effect changes durable state.
    ///
    /// The three read effects do not; every other one does. A caller holding only read
    /// grants is refused before dispatch, not after.
    #[must_use]
    pub const fn mutates(self) -> bool {
        !matches!(self, Self::Read | Self::ReadStream | Self::ReadOnlyPlanning)
    }
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One catalogue entry: a description, never an authorisation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Action {
    /// The stable action identity.
    pub id: &'static str,
    /// The declared version.
    pub version: &'static str,
    /// The module that owns the behaviour.
    pub owner: Owner,
    /// What invoking it does.
    pub effect: Effect,
    /// Its command-line projection.
    pub cli: &'static str,
    /// Its tool projection, when it has one.
    ///
    /// Five of the declared actions have no tool name in the plan spine. `None` says so;
    /// an earlier draft of this table wrote the literal string `"None"` for all five, which
    /// collapsed them into one apparent projection. `tests/t28_actions.rs` caught it by
    /// comparing against the spine rather than against this table.
    pub tool: Option<&'static str>,
    /// The capability a caller must hold, as the plan spine states it.
    pub capability: &'static str,
}

/// Every declared action, projected from the plan spine.
pub const CATALOGUE: [Action; DECLARED_ACTIONS] = [
    Action {
        id: "tools.list",
        version: "1 (proposed)",
        owner: Owner::Actions,
        effect: Effect::Read,
        cli: "habitat-engine tools list --json",
        tool: Some("habitat_tools_list"),
        capability: "visible actions only",
    },
    Action {
        id: "tools.inspect",
        version: "1 (proposed)",
        owner: Owner::Actions,
        effect: Effect::Read,
        cli: "habitat-engine tools inspect <action-id> --json",
        tool: Some("habitat_tools_inspect"),
        capability: "visible action only",
    },
    Action {
        id: "task.preview",
        version: "1 (proposed)",
        owner: Owner::Task,
        effect: Effect::ReadOnlyPlanning,
        cli: "habitat-engine task preview --file <spec.json> --json",
        tool: Some("habitat_task_preview"),
        capability: "task preview within caller scope",
    },
    Action {
        id: "task.submit",
        version: "1 (proposed)",
        owner: Owner::Task,
        effect: Effect::DurableAdmission,
        cli: "habitat-engine task submit --file <spec.json> --json",
        tool: Some("habitat_task_submit"),
        capability: "root submission or bounded child allocation",
    },
    Action {
        id: "task.get",
        version: "1 (proposed)",
        owner: Owner::Task,
        effect: Effect::Read,
        cli: "habitat-engine task inspect <id> --json OR task inspect --request-key <key> --json",
        tool: Some("habitat_task_get"),
        capability: "task visibility for actual principal",
    },
    Action {
        id: "task.list",
        version: "1 (proposed)",
        owner: Owner::Task,
        effect: Effect::Read,
        cli: "habitat-engine task list --json",
        tool: Some("habitat_task_list"),
        capability: "scoped visible tasks",
    },
    Action {
        id: "task.cancel",
        version: "1 (proposed)",
        owner: Owner::Task,
        effect: Effect::CancelIntent,
        cli: "habitat-engine task cancel <id> --json",
        tool: Some("habitat_task_cancel"),
        capability: "cancellation grant for target",
    },
    Action {
        id: "task.resolve",
        version: "1 (proposed)",
        owner: Owner::Task,
        effect: Effect::RecordDisposition,
        cli: "habitat-engine task resolve <id> --file <disposition.json> --json",
        tool: None,
        capability: "operator only",
    },
    Action {
        id: "thread.get",
        version: "1 (proposed)",
        owner: Owner::Cohort,
        effect: Effect::Read,
        cli: "habitat-engine thread inspect <id> --json",
        tool: Some("habitat_thread_get"),
        capability: "thread/task visibility",
    },
    Action {
        id: "thread.list",
        version: "1 (proposed)",
        owner: Owner::Cohort,
        effect: Effect::Read,
        cli: "habitat-engine thread list --json",
        tool: Some("habitat_thread_list"),
        capability: "thread/task visibility",
    },
    Action {
        id: "roster.list",
        version: "1 (proposed)",
        owner: Owner::Roster,
        effect: Effect::Read,
        cli: "habitat-engine roster list --kind <kind> --json",
        tool: Some("habitat_roster_list"),
        capability: "visible profile/instance/service records",
    },
    Action {
        id: "roster.inspect",
        version: "1 (proposed)",
        owner: Owner::Roster,
        effect: Effect::Read,
        cli: "habitat-engine roster inspect <id> --json",
        tool: Some("habitat_roster_inspect"),
        capability: "visible record",
    },
    Action {
        id: "roster.update",
        version: "1 (proposed)",
        owner: Owner::Roster,
        effect: Effect::ConfigurationMutation,
        cli: "habitat-engine roster update <id> --file <record.json> --json",
        tool: None,
        capability: "operator only",
    },
    Action {
        id: "roster.disable",
        version: "1 (proposed)",
        owner: Owner::Roster,
        effect: Effect::ConfigurationMutation,
        cli: "habitat-engine roster disable <id> --json",
        tool: None,
        capability: "operator only",
    },
    Action {
        id: "service.inspect",
        version: "1 (proposed)",
        owner: Owner::Service,
        effect: Effect::Read,
        cli: "habitat-engine service inspect <id> --json",
        tool: Some("habitat_service_inspect"),
        capability: "visible registered service",
    },
    Action {
        id: "service.probe",
        version: "1 (proposed)",
        owner: Owner::Service,
        effect: Effect::BoundedProbe,
        cli: "habitat-engine service probe <id> --json",
        tool: Some("habitat_service_probe"),
        capability: "specific approved probe capability",
    },
    Action {
        id: "service.action",
        version: "1 (proposed)",
        owner: Owner::Service,
        effect: Effect::ManagedLifecycle,
        cli: "habitat-engine service action <id> <action> --json",
        tool: Some("habitat_service_action"),
        capability: "specific service/action grant",
    },
    Action {
        id: "analysis.request",
        version: "1 (proposed)",
        owner: Owner::Numerical,
        effect: Effect::BoundedAnalysis,
        cli: "habitat-engine analysis request --dataset <id> --recipe <id> --json",
        tool: Some("habitat_analysis_request"),
        capability: "analysis grant + resource allocation",
    },
    Action {
        id: "analysis.get",
        version: "1 (proposed)",
        owner: Owner::Numerical,
        effect: Effect::Read,
        cli: "habitat-engine analysis inspect <id> --json OR analysis inspect --request-key <key> --json",
        tool: Some("habitat_analysis_get"),
        capability: "analysis/task visibility",
    },
    Action {
        id: "events.subscribe",
        version: "1 (proposed)",
        owner: Owner::Notify,
        effect: Effect::ReadStream,
        cli: "habitat-engine events follow [--cursor <cursor>] --json",
        tool: None,
        capability: "filtered event visibility",
    },
    Action {
        id: "health",
        version: "1 (proposed)",
        owner: Owner::App,
        effect: Effect::Read,
        cli: "habitat-engine health --json",
        tool: None,
        capability: "trusted operator health",
    },
];

/// A reason this module refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// No action carries that identity.
    UnknownAction,
    /// The action exists but not at the requested version.
    UnknownVersion,
    /// The action exists but this caller cannot see it. Deliberately distinct from
    /// [`Refusal::UnknownAction`] only at the API boundary: [`Catalogue::inspect`] reports
    /// `UnknownAction` for a hidden action so visibility is not leaked by the error.
    NotVisible,
    /// The caller does not hold a grant for the action's effect.
    UngrantedEffect,
    /// A page larger than [`MAX_PAGE`] was requested.
    PageTooWide,
    /// A page token beyond the end of the visible catalogue.
    PageOutOfRange,
}

impl Refusal {
    /// The stable diagnostic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnknownAction => "unknown action",
            Self::UnknownVersion => "action version is not declared",
            Self::NotVisible => "action is not visible to this caller",
            Self::UngrantedEffect => "caller holds no grant for this effect",
            Self::PageTooWide => "page bound exceeded",
            Self::PageOutOfRange => "page token beyond the visible catalogue",
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl std::error::Error for Refusal {}

/// What a caller may see and do.
///
/// Visibility and grants are separate: a caller may see an action it cannot invoke, which is
/// the whole point of a discoverable catalogue that is not an authority grant.
#[derive(Clone, Debug, Default)]
pub struct Caller {
    visible: Vec<Owner>,
    granted: Vec<Effect>,
}

impl Caller {
    /// A caller that can see nothing and do nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Make every action owned by `owner` visible.
    #[must_use]
    pub fn seeing(mut self, owner: Owner) -> Self {
        if !self.visible.contains(&owner) {
            self.visible.push(owner);
        }
        self
    }

    /// Grant `effect`.
    #[must_use]
    pub fn granted(mut self, effect: Effect) -> Self {
        if !self.granted.contains(&effect) {
            self.granted.push(effect);
        }
        self
    }

    /// Whether this caller can see actions owned by `owner`.
    #[must_use]
    pub fn sees(&self, owner: Owner) -> bool {
        self.visible.contains(&owner)
    }

    /// Whether this caller holds a grant for `effect`.
    #[must_use]
    pub fn holds(&self, effect: Effect) -> bool {
        self.granted.contains(&effect)
    }
}

/// An authorised dispatch.
///
/// The only producer is [`Catalogue::validate`]. Its [`Dispatch::owner`] is read from the
/// catalogue, so no transport can name a different one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Dispatch {
    action: Action,
}

impl Dispatch {
    /// The owning module, taken from the catalogue rather than from the request.
    #[must_use]
    pub const fn owner(self) -> Owner {
        self.action.owner
    }

    /// The action being dispatched.
    #[must_use]
    pub const fn action(self) -> Action {
        self.action
    }
}

/// One page of visible actions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Page {
    /// The visible entries on this page, in catalogue order.
    pub entries: Vec<Action>,
    /// Where to resume, or `None` when the listing is complete.
    pub next: Option<usize>,
    /// The catalogue revision this page was computed from.
    pub revision: i64,
}

/// The versioned action catalogue.
#[derive(Clone, Copy, Debug, Default)]
pub struct Catalogue;

impl Catalogue {
    /// Every declared action, regardless of visibility.
    #[must_use]
    pub const fn all() -> &'static [Action] {
        &CATALOGUE
    }

    /// The actions `caller` may see, paged.
    ///
    /// Visibility is recalculated for the caller on every call rather than cached, so a
    /// grant that is withdrawn takes effect on the next read.
    ///
    /// # Errors
    ///
    /// [`Refusal::PageTooWide`] beyond [`MAX_PAGE`], refused before any entry is copied;
    /// [`Refusal::PageOutOfRange`] for a token past the end.
    pub fn list(caller: &Caller, from: usize, limit: usize) -> Result<Page, Refusal> {
        if limit > MAX_PAGE {
            return Err(Refusal::PageTooWide);
        }
        let visible: Vec<Action> = CATALOGUE
            .into_iter()
            .filter(|action| caller.sees(action.owner))
            .collect();
        if from > visible.len() {
            return Err(Refusal::PageOutOfRange);
        }
        let end = from.saturating_add(limit).min(visible.len());
        Ok(Page {
            entries: visible[from..end].to_vec(),
            next: (end < visible.len()).then_some(end),
            revision: CATALOGUE_REVISION,
        })
    }

    /// Inspect one visible action at a requested version.
    ///
    /// A hidden action reports [`Refusal::UnknownAction`], not [`Refusal::NotVisible`]: the
    /// error must not tell a caller that something it may not see exists.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownAction`], [`Refusal::UnknownVersion`].
    pub fn inspect(caller: &Caller, id: &str, version: &str) -> Result<Action, Refusal> {
        let action = CATALOGUE
            .into_iter()
            .find(|action| action.id == id)
            .filter(|action| caller.sees(action.owner))
            .ok_or(Refusal::UnknownAction)?;
        if action.version != version {
            return Err(Refusal::UnknownVersion);
        }
        Ok(action)
    }

    /// Validate a dispatch request and, if it is authorised, produce the [`Dispatch`].
    ///
    /// Every refusal happens **before** a dispatch exists, which is what "refuse before
    /// dispatch" means when the refusal is a type rather than a comment.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownAction`], [`Refusal::UnknownVersion`],
    /// [`Refusal::UngrantedEffect`].
    pub fn validate(caller: &Caller, id: &str, version: &str) -> Result<Dispatch, Refusal> {
        let action = Self::inspect(caller, id, version)?;
        if !caller.holds(action.effect) {
            return Err(Refusal::UngrantedEffect);
        }
        Ok(Dispatch { action })
    }

    /// Look up one action by identity, ignoring visibility.
    ///
    /// # Errors
    ///
    /// [`Refusal::UnknownAction`].
    pub fn find(id: &str) -> Result<Action, Refusal> {
        CATALOGUE
            .into_iter()
            .find(|action| action.id == id)
            .ok_or(Refusal::UnknownAction)
    }
}
