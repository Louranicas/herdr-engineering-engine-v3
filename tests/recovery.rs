// HEE3-ANCHORS-BEGIN
// Anchor path: /var/home/herdr-engineering-engine-v3/tests/recovery.rs
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
//
// TEST AND INTERFACE VERIFICATION CONTRACT
// Primary path owner: recovery. Participating modules: recovery, task, worker, store, check, notify.
// Planned boundary: Recovery cases cross live-attempt identity, cancellation, evidence/acceptance boundaries and restored event epochs.
// For each case bind one primary module, stable case identity, requirement/interface/flow, independent oracle, subject/profile, fixtures and raw result. Other participants receive integration evidence, not duplicate primary credits.
// Minimum 50 distinct qualifying cases per module; zero baseline warnings/errors including admitted Rust pedantic Clippy, rustdoc and applicable Julia/package checks. Meaningful coverage gaps remain blocking even above the count.
// Exercise caller input and callee result/error/usage/cancellation paths, compatibility, authority, resource limits, idempotency and readback. Pair assimilation fault detection with a benign mirror; retain nonempty mutation/fault campaign dispositions.
// No tests or evaluation cases are implemented by these comments. Expected failures belong in isolated fixtures with asserted outcomes; they do not waive the zero-diagnostic baseline.
// Retain exact subject/toolchain/fixture hashes and observations; completion admission and full testing qualification remain unavailable. See docs/public-interfaces.md, tests/README.md and corpus/UPDATE_PROTOCOL.md.
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
// Readiness binding: HEE3-READINESS-001; SHA-256 3bcde91b4617c1a38bcbb97c36bddbc9e999c7692a69b0ba21caec9f559fe250; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-04, R90-05, R90-06, R90-08, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-check; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-check (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-check
// Owns: Protected verifier invocation and criterion evidence
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/check.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T01, T04, T06, T07, T10, T12, T14, T15, T17, T18, T19, T20, T25, T26, T27
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K4](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K4)
// [contributing codebase CODE-CB01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB01)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB05)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [task TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
// [implementation support task TASK-T02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T02)
// [implementation support task TASK-T03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T03)
// [task TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
// [implementation support task TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
// [task TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
// [task TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
// [task TASK-T12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T12)
// [implementation support task TASK-T13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T13)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [implementation support task TASK-T21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T21)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [separate reference example EX-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-check)
// [flow FLOW-F07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F07)
// [handbook HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [handbook HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-check)
// [plan SEC-hardening](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-hardening)
// [plan SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-security](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-security)
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
// [schematic SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
// [schematic SC-SC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC06)
// [schematic SC-SC08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC08)
// [schematic SC-SC09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC09)
// [schematic SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
// [schematic SC-SC12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC12)
// [schematic SC-SC15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC15)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC22)
// [source SRC-C14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C14)
// [source SRC-S05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S05)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-check)
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
// [readiness criterion cluster F6](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-04)
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
// [completion and operational convention DONE-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-check)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
// [applied learning LRN02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN02)
// [applied learning LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
// [applied learning LRN06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
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
// [diary evidence source DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
// [diary evidence source DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
// Applicable learning IDs: LRN01, LRN02, LRN05, LRN06, LRN08, LRN12, LRN14; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Seven Traits, Tested](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Seven%20Traits%2C%20Tested)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [Thematic Analysis of the Vaults](obsidian://open?vault=my-diary.vault&file=Reflections%2FThematic%20Analysis%20of%20the%20Vaults)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// [Why I Stopped Trusting Green](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhy%20I%20Stopped%20Trusting%20Green)
// [Working in Sandboxes on Kinoite](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20in%20Sandboxes%20on%20Kinoite)
// Readiness binding: HEE3-READINESS-001; SHA-256 3bcde91b4617c1a38bcbb97c36bddbc9e999c7692a69b0ba21caec9f559fe250; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-05, R90-07, R90-08, R90-09, R90-10; resolved contracts RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
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
// Readiness binding: HEE3-READINESS-001; SHA-256 3bcde91b4617c1a38bcbb97c36bddbc9e999c7692a69b0ba21caec9f559fe250; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F6-C01, F6-C02, F6-C03, F6-C04, F6-C05, F6-C06, F6-C07, F6-C08, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-05, R90-08, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-recovery; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-recovery (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-recovery
// Owns: Attempt reconciliation, ambiguity and cleanup policy
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/recovery.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-recovery)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T04, T07, T14, T15, T17, T18, T19, T20, T25, T26, T27
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K1)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB03)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [task TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
// [task TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
// [task TASK-T14](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T14)
// [task TASK-T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15)
// [task TASK-T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17)
// [task TASK-T18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T18)
// [task TASK-T19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T19)
// [task TASK-T20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T20)
// [task TASK-T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25)
// [task TASK-T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26)
// [task TASK-T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27)
// [separate reference example EX-recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-recovery)
// [flow FLOW-F11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F11)
// [handbook HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [handbook HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-recovery)
// [plan SEC-deployment](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-deployment)
// [plan SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
// [plan SEC-runtime](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-runtime)
// [plan SEC-security](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-security)
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
// [schematic SC-SC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC06)
// [schematic SC-SC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC07)
// [schematic SC-SC13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC13)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [source SRC-A03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A03)
// [source SRC-H05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-H05)
// [source SRC-H11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-H11)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-recovery)
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
// [readiness criterion cluster F6](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF6)
// [readiness criterion cluster F7](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FF7)
// [readiness improvement grouping R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
// [readiness improvement grouping R90-08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-08)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-recovery)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN05)
// [applied learning LRN06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN06)
// [applied learning LRN07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN07)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11)
// [applied learning LRN13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN13)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR06)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// [diary evidence source DR11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR11)
// [diary evidence source DR12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR12)
// [diary evidence source DR13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR13)
// Applicable learning IDs: LRN05, LRN06, LRN07, LRN08, LRN11, LRN13; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [The Spellbook and the Ember](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Spellbook%20and%20the%20Ember)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// [Why I Stopped Trusting Green](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhy%20I%20Stopped%20Trusting%20Green)
// [Working Style in This Habitat](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20Style%20in%20This%20Habitat)
// [Working in Sandboxes on Kinoite](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20in%20Sandboxes%20on%20Kinoite)
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
// Readiness binding: HEE3-READINESS-001; SHA-256 3bcde91b4617c1a38bcbb97c36bddbc9e999c7692a69b0ba21caec9f559fe250; clauses F1-C01, F1-C02, F1-C03, F1-C04, F1-C05, F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-01, R90-03, R90-05, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
// Completion identity: HEE3-DONE-task; all 13 applicable gates; current state unassessed. No documentation pass admits this module.
// Mandatory testing convention: at least 50 distinct qualifying module-owned cases; zero baseline warnings/errors, including pedantic Clippy on admitted Rust targets/profiles. Full qualification remains unassessed.
//
// Stable public interface: HEE3-IF-task (planned; concrete symbols and acceptance unavailable)
// Stable module anchor: HEE3-MOD-task
// Owns: Task/attempt and bounded verification-loop state
// [FULL MODULE DEPLOYMENT CONTRACT](file:///var/home/herdr-engineering-engine-v3/docs/modules/task.md)
// [MODULE STEM AND ALL RETURN ANCHORS](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task)
// Build dependencies: contracts
// Consumers: app
// Related task contracts: T01, T04, T06, T07, T10, T12, T14, T17, T18, T19, T20, T22, T25, T26, T27, T29
// Future validators are proposed/unavailable; no empty or skipped check establishes completion.
// [module cluster CLU-K1](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Clusters%2FCLU-K1)
// [contributing codebase CODE-CB02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB02)
// [contributing codebase CODE-CB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB03)
// [contributing codebase CODE-CB08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Codebases%2FCODE-CB08)
// [task TASK-T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01)
// [task TASK-T04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T04)
// [implementation support task TASK-T05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T05)
// [task TASK-T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06)
// [task TASK-T07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T07)
// [task TASK-T10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T10)
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
// [separate reference example EX-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Exemplars%2FEX-task)
// [flow FLOW-F01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F01)
// [flow FLOW-F02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F02)
// [flow FLOW-F03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F03)
// [flow FLOW-F04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F04)
// [flow FLOW-F05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F05)
// [flow FLOW-F06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F06)
// [flow FLOW-F07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F07)
// [flow FLOW-F08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F08)
// [flow FLOW-F10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F10)
// [flow FLOW-F11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F11)
// [flow FLOW-F13](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F13)
// [flow FLOW-F15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F15)
// [flow FLOW-F19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Flows%2FFLOW-F19)
// [handbook HB-failure-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-failure-map)
// [handbook HB-identity-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-identity-map)
// [handbook HB-state-map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Handbook%2FSections%2FHB-state-map)
// [API API-API01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API01)
// [API API-API10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FAPI%2FAPI-API10)
// [action ACT-task.cancel](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.cancel)
// [action ACT-task.get](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.get)
// [action ACT-task.list](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.list)
// [action ACT-task.preview](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.preview)
// [action ACT-task.resolve](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.resolve)
// [action ACT-task.submit](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FActions%2FACT-task.submit)
// [IPC IPC-IPC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC01)
// [IPC IPC-IPC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FIPC%2FIPC-IPC04)
// [public interface convention Module Public Contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Interfaces%2FModule%20Public%20Contracts)
// [planned module MOD-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Modules%2FMOD-task)
// [plan SEC-architecture](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-architecture)
// [plan SEC-loop](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-loop)
// [plan SEC-module-design](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Plan%2FSEC-module-design)
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
// [schematic SC-SC01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC01)
// [schematic SC-SC02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC02)
// [schematic SC-SC04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC04)
// [schematic SC-SC05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC05)
// [schematic SC-SC06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC06)
// [schematic SC-SC07](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC07)
// [schematic SC-SC08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC08)
// [schematic SC-SC10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC10)
// [schematic SC-SC11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC11)
// [schematic SC-SC12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC12)
// [schematic SC-SC17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC17)
// [schematic SC-SC18](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC18)
// [schematic SC-SC19](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC19)
// [schematic SC-SC20](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC20)
// [schematic SC-SC21](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC21)
// [schematic SC-SC22](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC22)
// [schematic SC-SC23](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC23)
// [schematic SC-SC24](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Schematics%2FSC-SC24)
// [source SRC-A03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-A03)
// [source SRC-C02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-C02)
// [source SRC-S15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Sources%2FSRC-S15)
// [testing standard Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing)
// [progressive context workflow Module Context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Workflows%2FModule%20Context)
// [module context scout CTX-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%2FModules%2FCTX-task)
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
// [readiness improvement grouping R90-03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-03)
// [readiness improvement grouping R90-05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-05)
// [readiness improvement grouping R90-09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-09)
// [readiness improvement grouping R90-10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FR90-10)
// [Graphify corpus projection Index](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex)
// [defensive security convention Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile)
// [defensive security convention RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05)
// [defensive security convention Configuration](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FConfiguration)
// [completion and operational convention Module Completion](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion)
// [completion and operational convention DONE-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-task)
// [completion and operational convention Context Handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff)
// [completion and operational convention Justfiles and Runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks)
// [completion and operational convention RB03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB03)
// [completion and operational convention RB04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB04)
// [applied learning LRN01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN01)
// [applied learning LRN04](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN04)
// [applied learning LRN08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN08)
// [applied learning LRN11](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FLRN11)
// [diary evidence source DR01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR01)
// [diary evidence source DR02](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR02)
// [diary evidence source DR03](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR03)
// [diary evidence source DR08](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR08)
// [diary evidence source DR09](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR09)
// [diary evidence source DR10](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR10)
// [diary evidence source DR12](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Learnings%2FSources%2FDR12)
// Applicable learning IDs: LRN01, LRN04, LRN08, LRN11; guidance only, engine detectors unqualified.
// [Assertions Measured Against the Code](obsidian://open?vault=my-diary.vault&file=Reflections%2FAssertions%20Measured%20Against%20the%20Code)
// [Mistakes I Made](obsidian://open?vault=my-diary.vault&file=Reflections%2FMistakes%20I%20Made)
// [The Antipattern Registers](obsidian://open?vault=my-diary.vault&file=Reflections%2FThe%20Antipattern%20Registers)
// [What My Ancestors Knew That I Did Not](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20My%20Ancestors%20Knew%20That%20I%20Did%20Not)
// [What Prototyping Is For](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20Prototyping%20Is%20For)
// [What the Workflow Is Worth](obsidian://open?vault=my-diary.vault&file=Reflections%2FWhat%20the%20Workflow%20Is%20Worth)
// [Working Style in This Habitat](obsidian://open?vault=my-diary.vault&file=Reflections%2FWorking%20Style%20in%20This%20Habitat)
// Readiness binding: HEE3-READINESS-001; SHA-256 3bcde91b4617c1a38bcbb97c36bddbc9e999c7692a69b0ba21caec9f559fe250; clauses F2-C01, F2-C02, F2-C03, F2-C04, F2-C05, F2-C06, F3-C01, F3-C02, F3-C03, F3-C04, F3-C05, F3-C06, F4-C01, F4-C02, F4-C03, F4-C04, F4-C05, F4-C06, F4-C07, F5-C01, F5-C02, F5-C03, F5-C04, F5-C05, F5-C06, F7-C01, F7-C02, F7-C03, F7-C04, F7-C05, F7-C06; groupings R90-02, R90-03, R90-05, R90-06, R90-09, R90-10; resolved contracts RC01, RC02, RC03, RC04, RC05, RC06; runtime proof pending; original task DAG controls.
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
//! T07 reconciliation policy cases (`T07-RC-nn`). Every case constructs its inputs
//! as values, asserts the whole `Decision` (rule and arm with its evidence) and
//! names the acceptance obligation it covers. No I/O, clock or process is involved.
use habitat_engine::recovery::{
    AcceptanceCandidate, Acknowledgement, AttemptFacts, AttemptState, Cleanup, CleanupReadback,
    CleanupTarget, Clock, CursorFacts, CursorRefusal, Decision, Dimension, Effect, Evidence,
    GenerationSubject, Lease, LedgerFacts, Mode, ObservationClaim, Observations, PiQueueCustody,
    ProcessCustody, Reconciliation, ReuseRefusal, Rule, TaskFacts, TaskHistory, TaskState, Unknown,
    Verdict, Verification, WorkspaceReadback, reconcile, reconcile_cursor,
};

const EPOCH: &str = "07000000-0000-4000-8000-0000000000e1";
const PRIOR: &str = "07000000-0000-4000-8000-0000000000e0";
const OTHER: &str = "07000000-0000-4000-8000-0000000000e2";
const CLOCK: &str = "07000000-0000-4000-8000-0000000000c1";
const CLOCK2: &str = "07000000-0000-4000-8000-0000000000c2";
const ACCEPT: &str = "07000000-0000-4000-8000-00000000000a";
const CHECK: &str = "07000000-0000-4000-8000-000000000009";
const ATTEMPT: &str = "07000000-0000-4000-8000-000000000006";
const HIGH_WATER: u64 = 7;
const TASK_GENERATION: u64 = 3;
const ATTEMPT_GENERATION: u64 = 2;
const EXPIRES: u64 = 5_000;

fn ledger() -> LedgerFacts<'static> {
    LedgerFacts {
        epoch: EPOCH,
        mode: Mode::Normal,
        event_high_water: HIGH_WATER,
        restored_from: None,
    }
}
fn task(state: TaskState, history: TaskHistory<'static>) -> TaskFacts<'static> {
    TaskFacts {
        generation: TASK_GENERATION,
        state,
        history,
        candidate: AcceptanceCandidate::None,
    }
}
fn open(state: TaskState) -> TaskFacts<'static> {
    task(state, TaskHistory::Open)
}
fn attempt(state: AttemptState, effect: Effect, cleanup: Cleanup) -> AttemptFacts<'static> {
    AttemptFacts {
        id: ATTEMPT,
        generation: ATTEMPT_GENERATION,
        state,
        effect,
        cleanup,
        acknowledgement: Acknowledgement::Unrecorded,
        lease: Lease::NotLeased,
        verification: Verification::None,
        evidence: Evidence::Unassessed,
    }
}
fn running() -> AttemptFacts<'static> {
    attempt(AttemptState::Running, Effect::Pending, Cleanup::Pending)
}
fn settled() -> AttemptFacts<'static> {
    attempt(AttemptState::Settled, Effect::None, Cleanup::Settled)
}
fn leased(acknowledgement: Acknowledgement) -> AttemptFacts<'static> {
    AttemptFacts {
        acknowledgement,
        lease: Lease::Leased {
            clock_epoch: CLOCK,
            expires_monotonic_ms: EXPIRES,
        },
        ..running()
    }
}
fn observed(process: ProcessCustody) -> Observations<'static> {
    Observations {
        claim: None,
        clock: None,
        process,
        pi_queue: PiQueueCustody::NotApplicable,
        cleanup: CleanupReadback::NotRead,
        workspace: WorkspaceReadback::NotRead,
    }
}
fn live() -> Observations<'static> {
    observed(ProcessCustody::LiveSameIdentity)
}
fn absent() -> Observations<'static> {
    observed(ProcessCustody::Absent)
}
fn reused() -> ProcessCustody {
    ProcessCustody::PidReused {
        differs: vec![Dimension::StartTicks],
    }
}
fn cleaned(process: ProcessCustody) -> Observations<'static> {
    Observations {
        cleanup: CleanupReadback::Complete,
        ..observed(process)
    }
}
fn claim(
    epoch: &'static str,
    task_generation: u64,
    attempt_generation: u64,
) -> Observations<'static> {
    Observations {
        claim: Some(ObservationClaim {
            epoch,
            task_generation,
            attempt_generation,
        }),
        ..live()
    }
}
fn writable(clock: Option<Clock<'static>>) -> Observations<'static> {
    Observations {
        clock,
        workspace: WorkspaceReadback::Writable { bytes: 41 },
        ..absent()
    }
}
fn at(epoch: &'static str, monotonic_ms: u64) -> Clock<'static> {
    Clock {
        epoch,
        monotonic_ms,
    }
}
fn unknown(
    rule: Rule,
    reason: Unknown,
    process: ProcessCustody,
    cancellation_pending: bool,
) -> Decision {
    Decision {
        rule,
        reconciliation: Reconciliation::RetainUnknown {
            reason,
            process,
            cancellation_pending,
        },
    }
}
fn refused(rule: Rule, reason: ReuseRefusal) -> Decision {
    Decision {
        rule,
        reconciliation: Reconciliation::WorkspaceReuseRefused {
            reason,
            process: ProcessCustody::Absent,
        },
    }
}
fn cursor(ledger: &LedgerFacts<'_>, epoch: &'static str, sequence: u64) -> Decision {
    reconcile_cursor(ledger, &CursorFacts { epoch, sequence })
}
fn stale_cursor(reason: CursorRefusal, epoch: &str, sequence: u64) -> Decision {
    Decision {
        rule: Rule::R13CursorEpoch,
        reconciliation: Reconciliation::RefuseStaleCursor {
            reason,
            cursor_epoch: epoch.to_owned(),
            cursor_sequence: sequence,
            ledger_epoch: EPOCH.to_owned(),
            event_high_water: HIGH_WATER,
        },
    }
}

// ---- stale observations and generations -------------------------------------------------

/// T07-RC-01 · restored epochs reject stale observations: a claim from another epoch is refused
/// before any physical evidence is read, even for a live child.
#[test]
fn observation_from_another_epoch_is_refused_before_anything_else() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &running(),
        &claim(OTHER, TASK_GENERATION, ATTEMPT_GENERATION),
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R01StaleObservationEpoch,
            reconciliation: Reconciliation::StaleObservationRefused {
                observed_epoch: OTHER.to_owned(),
                ledger_epoch: EPOCH.to_owned(),
            },
        }
    );
}
/// T07-RC-02 · benign neighbour of RC-01/03/04: a claim naming the current epoch and both current
/// generations is not stale; the live child is reattached for observation.
#[test]
fn current_claim_passes_through_to_the_physical_rules() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &running(),
        &claim(EPOCH, TASK_GENERATION, ATTEMPT_GENERATION),
    );
    assert_eq!(decision.rule, Rule::R06LiveOwnedChild);
    assert_eq!(decision.reconciliation.name(), "reattach_observation_only");
}
/// T07-RC-03 · stale generations: an observation claiming an older task generation is refused
/// with both numbers.
#[test]
fn older_task_generation_claim_is_refused() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &running(),
        &claim(EPOCH, TASK_GENERATION - 1, ATTEMPT_GENERATION),
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R02StaleGeneration,
            reconciliation: Reconciliation::StaleGenerationRefused {
                subject: GenerationSubject::Task,
                claimed: TASK_GENERATION - 1,
                current: TASK_GENERATION,
            },
        }
    );
}
/// T07-RC-04 · stale generations: the attempt generation is checked separately from the task's.
#[test]
fn older_attempt_generation_claim_is_refused() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &running(),
        &claim(EPOCH, TASK_GENERATION, ATTEMPT_GENERATION + 5),
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R02StaleGeneration,
            reconciliation: Reconciliation::StaleGenerationRefused {
                subject: GenerationSubject::Attempt,
                claimed: ATTEMPT_GENERATION + 5,
                current: ATTEMPT_GENERATION,
            },
        }
    );
}
/// T07-RC-05 · stale generations: a claim *ahead* of the ledger is as stale as one behind it
/// (nothing may act on a generation the ledger has not reached).
#[test]
fn future_task_generation_claim_is_refused_too() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &running(),
        &claim(EPOCH, TASK_GENERATION + 1, ATTEMPT_GENERATION),
    );
    assert_eq!(decision.rule, Rule::R02StaleGeneration);
    assert_eq!(
        decision.reconciliation,
        Reconciliation::StaleGenerationRefused {
            subject: GenerationSubject::Task,
            claimed: TASK_GENERATION + 1,
            current: TASK_GENERATION,
        }
    );
}

// ---- acceptance and cancellation ordering ------------------------------------------------

fn both(cancellation: u64, acceptance: u64) -> TaskFacts<'static> {
    task(
        TaskState::Accepted,
        TaskHistory::Both {
            cancellation,
            acceptance_event: ACCEPT,
            acceptance,
        },
    )
}
/// T07-RC-06 · acceptance explicitly rejects an earlier committed cancellation: two commits with
/// the cancellation first cannot be read as an acceptance; the ordinals are the evidence.
#[test]
fn cancellation_committed_before_acceptance_is_not_read_as_acceptance() {
    let decision = reconcile(&ledger(), &both(5, 9), &settled(), &absent());
    assert_eq!(
        decision,
        unknown(
            Rule::R03CommitOrdering,
            Unknown::CancellationPrecedesAcceptance {
                cancellation: 5,
                acceptance: 9,
            },
            ProcessCustody::Absent,
            false,
        )
    );
}
/// T07-RC-07 · if acceptance committed first, later cancellation cannot rewrite history.
#[test]
fn acceptance_committed_before_cancellation_stands_with_the_later_ordinal_recorded() {
    let decision = reconcile(&ledger(), &both(9, 5), &settled(), &absent());
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R03CommitOrdering,
            reconciliation: Reconciliation::AcceptanceStands {
                event: ACCEPT.to_owned(),
                ordinal: Some(5),
                later_cancellation: Some(9),
                cleanup: Cleanup::Settled,
            },
        }
    );
}
/// T07-RC-08 · two commits sharing one ordinal cannot be ordered; the history is contradictory.
#[test]
fn equal_commit_ordinals_are_contradictory() {
    let decision = reconcile(&ledger(), &both(6, 6), &settled(), &absent());
    assert_eq!(
        decision,
        unknown(
            Rule::R03CommitOrdering,
            Unknown::HistoryContradictory,
            ProcessCustody::Absent,
            false
        )
    );
}
/// T07-RC-09 · both terminal flags without ordinals (impossible under the store) are reported,
/// never guessed.
#[test]
fn contradictory_history_without_ordinals_is_retained_unknown() {
    let decision = reconcile(
        &ledger(),
        &task(TaskState::Accepted, TaskHistory::Contradictory),
        &running(),
        &live(),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R03CommitOrdering,
            Unknown::HistoryContradictory,
            ProcessCustody::LiveSameIdentity,
            false
        )
    );
}
fn accepted() -> TaskFacts<'static> {
    task(
        TaskState::Accepted,
        TaskHistory::Accepted {
            event: ACCEPT,
            ordinal: Some(HIGH_WATER),
        },
    )
}
/// T07-RC-10 · restart at the acceptance boundary: a committed acceptance stands once, carrying
/// its event and ordinal, with no later cancellation.
#[test]
fn committed_acceptance_stands_after_restart() {
    let decision = reconcile(
        &ledger(),
        &accepted(),
        &settled(),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R04AcceptanceStands,
            reconciliation: Reconciliation::AcceptanceStands {
                event: ACCEPT.to_owned(),
                ordinal: Some(HIGH_WATER),
                later_cancellation: None,
                cleanup: Cleanup::Settled,
            },
        }
    );
}
/// T07-RC-11 · history precedes physical evidence: an accepted task whose attempt row still
/// carries pending cleanup reports the acceptance with that cleanup fact, not a cleanup arm.
#[test]
fn acceptance_carries_residual_cleanup_rather_than_yielding_to_it() {
    let residual = attempt(AttemptState::Settled, Effect::Committed, Cleanup::Pending);
    let decision = reconcile(&ledger(), &accepted(), &residual, &live());
    assert_eq!(decision.rule, Rule::R04AcceptanceStands);
    assert_eq!(
        decision.reconciliation,
        Reconciliation::AcceptanceStands {
            event: ACCEPT.to_owned(),
            ordinal: Some(HIGH_WATER),
            later_cancellation: None,
            cleanup: Cleanup::Pending,
        }
    );
}
fn cancelled() -> TaskFacts<'static> {
    task(
        TaskState::CancellationRequested,
        TaskHistory::Cancelled { ordinal: Some(4) },
    )
}
/// T07-RC-12 · acceptance explicitly rejects an earlier committed cancellation intent, even before
/// terminal cleanup: a prepared acceptance on a cancelled task is rejected while the attempt is
/// still running with pending cleanup.
#[test]
fn prepared_acceptance_is_rejected_by_earlier_cancellation_before_cleanup() {
    let task = TaskFacts {
        candidate: AcceptanceCandidate::Prepared {
            verification_event: CHECK,
        },
        ..cancelled()
    };
    let decision = reconcile(&ledger(), &task, &running(), &live());
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R05CancellationStands,
            reconciliation: Reconciliation::CancellationStands {
                ordinal: Some(4),
                rejected_acceptance: Some(CHECK.to_owned()),
                attempt_state: AttemptState::Running,
                cleanup: Cleanup::Pending,
            },
        }
    );
}
/// T07-RC-13 · a committed cancellation stands for settled work with nothing to reject.
#[test]
fn cancellation_stands_for_settled_work() {
    let decision = reconcile(&ledger(), &cancelled(), &settled(), &absent());
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R05CancellationStands,
            reconciliation: Reconciliation::CancellationStands {
                ordinal: Some(4),
                rejected_acceptance: None,
                attempt_state: AttemptState::Settled,
                cleanup: Cleanup::Settled,
            },
        }
    );
}
/// T07-RC-14 · a cancelled task's live worker is still reconciled physically: observation
/// reattaches (to deliver the cancellation) with `cancellation_pending` carried, no redispatch.
#[test]
fn cancelled_task_with_live_worker_reattaches_observation_with_cancellation_pending() {
    let decision = reconcile(&ledger(), &cancelled(), &running(), &live());
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R06LiveOwnedChild,
            reconciliation: Reconciliation::ReattachObservationOnly {
                generation: ATTEMPT_GENERATION,
                process: ProcessCustody::LiveSameIdentity,
                pi_queue: PiQueueCustody::NotApplicable,
                cancellation_pending: true,
                redispatch: false,
            },
        }
    );
}
/// T07-RC-15 · a cancelled task's unknown effect stays unknown; cancellation does not resolve it.
#[test]
fn cancellation_does_not_resolve_an_unknown_effect() {
    let decision = reconcile(
        &ledger(),
        &cancelled(),
        &attempt(AttemptState::Unknown, Effect::Unknown, Cleanup::Unknown),
        &absent(),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R10EffectAmbiguity,
            Unknown::EffectUnknown,
            ProcessCustody::Absent,
            true
        )
    );
}

// ---- the live owned child and the Pi queue -----------------------------------------------

/// T07-RC-16 · startup may reattach observation to a positively reconciled live attempt without
/// redispatch: same identity, no Pi queue, current generation.
#[test]
fn live_same_identity_reattaches_observation_only() {
    let decision = reconcile(&ledger(), &open(TaskState::Running), &running(), &live());
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R06LiveOwnedChild,
            reconciliation: Reconciliation::ReattachObservationOnly {
                generation: ATTEMPT_GENERATION,
                process: ProcessCustody::LiveSameIdentity,
                pi_queue: PiQueueCustody::NotApplicable,
                cancellation_pending: false,
                redispatch: false,
            },
        }
    );
    assert!(!decision.reconciliation.permits_execution());
}
/// T07-RC-17 · a Pi session read back idle is positively reconciled and reattaches.
#[test]
fn idle_pi_queue_permits_observation_reattach() {
    let obs = Observations {
        pi_queue: PiQueueCustody::Idle,
        ..live()
    };
    let decision = reconcile(&ledger(), &open(TaskState::Running), &running(), &obs);
    assert_eq!(decision.rule, Rule::R06LiveOwnedChild);
    assert_eq!(
        decision.reconciliation,
        Reconciliation::ReattachObservationOnly {
            generation: ATTEMPT_GENERATION,
            process: ProcessCustody::LiveSameIdentity,
            pi_queue: PiQueueCustody::Idle,
            cancellation_pending: false,
            redispatch: false,
        }
    );
}
/// T07-RC-18 · queued Pi messages: an unreconciled queue keeps a live child unknown, with the
/// live custody carried as evidence.
#[test]
fn unreconciled_pi_queue_keeps_a_live_child_unknown() {
    let obs = Observations {
        pi_queue: PiQueueCustody::Unreconciled,
        ..live()
    };
    let decision = reconcile(&ledger(), &open(TaskState::Running), &running(), &obs);
    assert_eq!(
        decision,
        unknown(
            Rule::R06LiveOwnedChild,
            Unknown::PiQueueUnreconciled,
            ProcessCustody::LiveSameIdentity,
            false
        )
    );
}
/// T07-RC-19 · queued Pi messages are never replayed: a nonempty queue is reported by its counts.
#[test]
fn occupied_pi_queue_is_reported_by_its_counts() {
    let obs = Observations {
        pi_queue: PiQueueCustody::Queued {
            steering: 1,
            follow_up: 2,
        },
        ..live()
    };
    let decision = reconcile(&ledger(), &open(TaskState::Running), &running(), &obs);
    assert_eq!(
        decision,
        unknown(
            Rule::R06LiveOwnedChild,
            Unknown::PiQueueOccupied {
                steering: 1,
                follow_up: 2,
            },
            ProcessCustody::LiveSameIdentity,
            false,
        )
    );
}
/// T07-RC-20 · a clear whose reply was lost keeps its pending command identity and stays unknown.
#[test]
fn pending_pi_clear_keeps_its_command_identity_and_stays_unknown() {
    let obs = Observations {
        pi_queue: PiQueueCustody::ClearPending {
            command: "task:attempt:2:4".to_owned(),
        },
        ..live()
    };
    let decision = reconcile(&ledger(), &open(TaskState::Running), &running(), &obs);
    assert_eq!(
        decision,
        unknown(
            Rule::R06LiveOwnedChild,
            Unknown::PiClearPending {
                command: "task:attempt:2:4".to_owned(),
            },
            ProcessCustody::LiveSameIdentity,
            false,
        )
    );
}

// ---- PID reuse, unreadable, unobserved ---------------------------------------------------

/// T07-RC-21 · PID reuse does not attach to the wrong work: a reused PID is never reattached and
/// the differing dimensions are the evidence.
#[test]
fn reused_pid_is_never_reattached() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &running(),
        &observed(reused()),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R07ProcessNotOurs,
            Unknown::ProcessIdentityReused {
                differs: vec![Dimension::StartTicks],
            },
            reused(),
            false,
        )
    );
}
/// T07-RC-22 · precedence: a reused PID on both dimensions with an idle Pi queue and a writable
/// workspace is still the process rule's decision, not a reattach or a workspace verdict.
#[test]
fn reused_pid_precedes_pi_and_workspace_rules() {
    let both = ProcessCustody::PidReused {
        differs: vec![Dimension::StartTicks, Dimension::Namespace],
    };
    let obs = Observations {
        pi_queue: PiQueueCustody::Idle,
        workspace: WorkspaceReadback::Writable { bytes: 41 },
        clock: Some(at(CLOCK, EXPIRES + 1)),
        ..observed(both.clone())
    };
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &leased(Acknowledgement::NotSeen),
        &obs,
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R07ProcessNotOurs,
            Unknown::ProcessIdentityReused {
                differs: vec![Dimension::StartTicks, Dimension::Namespace],
            },
            both,
            false,
        )
    );
}
/// T07-RC-23 · an unreadable `/proc` entry keeps its text and is never treated as absent.
#[test]
fn unreadable_process_keeps_its_error_text() {
    let custody = ProcessCustody::Unreadable {
        error: "stat: Permission denied (os error 13)".to_owned(),
    };
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &running(),
        &observed(custody.clone()),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R07ProcessNotOurs,
            Unknown::ProcessUnreadable {
                error: "stat: Permission denied (os error 13)".to_owned(),
            },
            custody,
            false,
        )
    );
}
/// T07-RC-24 · an attempt whose roster observation is not a process identity is unobserved, not
/// absent: no cleanup and no reattach follow.
#[test]
fn unobserved_process_is_neither_absent_nor_live() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &running(),
        &observed(ProcessCustody::Unobserved),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R07ProcessNotOurs,
            Unknown::ProcessUnobserved,
            ProcessCustody::Unobserved,
            false
        )
    );
}

// ---- crashes before and after acknowledgement --------------------------------------------

/// T07-RC-25 · crash before dispatch acknowledgement: the absent worker's effect is unknown and
/// the reason names the missing acknowledgement; nothing is redispatched.
#[test]
fn crash_before_acknowledgement_retains_unknown_by_that_reason() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &leased(Acknowledgement::NotSeen),
        &absent(),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R08WorkerAbsent,
            Unknown::DispatchUnacknowledged,
            ProcessCustody::Absent,
            false
        )
    );
    assert!(!decision.reconciliation.permits_execution());
}
/// T07-RC-26 · crash after a correlated acknowledgement: the acknowledged generation travels with
/// the unknown so a duplicate write is impossible to license from this report.
#[test]
fn crash_after_acknowledgement_carries_the_acknowledged_generation() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &leased(Acknowledgement::Correlated { generation: 1 }),
        &absent(),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R08WorkerAbsent,
            Unknown::AcknowledgedWorkerLost { generation: 1 },
            ProcessCustody::Absent,
            false,
        )
    );
}
/// T07-RC-27 · a caller with no acknowledgement record at all (the inspector) gets a reason that
/// says so rather than either crash class.
#[test]
fn absent_worker_with_unrecorded_acknowledgement_is_named_as_such() {
    let decision = reconcile(&ledger(), &open(TaskState::Running), &running(), &absent());
    assert_eq!(
        decision,
        unknown(
            Rule::R08WorkerAbsent,
            Unknown::AcknowledgementUnrecorded,
            ProcessCustody::Absent,
            false
        )
    );
}
/// T07-RC-28 · a released workspace changes nothing about the absent worker's unknown effect.
#[test]
fn released_workspace_does_not_alter_the_absent_worker_verdict() {
    let obs = Observations {
        workspace: WorkspaceReadback::Released,
        ..absent()
    };
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &leased(Acknowledgement::Correlated { generation: 1 }),
        &obs,
    );
    assert_eq!(decision.rule, Rule::R08WorkerAbsent);
    assert_eq!(
        decision.reconciliation,
        Reconciliation::RetainUnknown {
            reason: Unknown::AcknowledgedWorkerLost { generation: 1 },
            process: ProcessCustody::Absent,
            cancellation_pending: false,
        }
    );
}

// ---- lease expiry and the still-writable workspace ---------------------------------------

/// T07-RC-29 · lease expiry alone cannot authorize reuse of a still-writable old workspace: an
/// expired lease with a writable workspace is refused, naming how long ago it expired.
#[test]
fn expired_lease_with_writable_workspace_is_refused() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &leased(Acknowledgement::Correlated { generation: 1 }),
        &writable(Some(at(CLOCK, EXPIRES + 250))),
    );
    assert_eq!(
        decision,
        refused(
            Rule::R09WorkspaceReuse,
            ReuseRefusal::LeaseExpiredWritable {
                expired_by_ms: 250,
                bytes: 41,
            },
        )
    );
}
/// T07-RC-30 · an unexpired lease is refused with the remaining time.
#[test]
fn held_lease_with_writable_workspace_is_refused_with_remaining_time() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &leased(Acknowledgement::NotSeen),
        &writable(Some(at(CLOCK, EXPIRES - 300))),
    );
    assert_eq!(
        decision,
        refused(
            Rule::R09WorkspaceReuse,
            ReuseRefusal::LeaseHeld { remaining_ms: 300 }
        )
    );
}
/// T07-RC-31 · boundary: at the exact expiry instant the lease is still held (remaining 0), not expired.
#[test]
fn lease_at_exact_expiry_instant_is_still_held() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &leased(Acknowledgement::NotSeen),
        &writable(Some(at(CLOCK, EXPIRES))),
    );
    assert_eq!(
        decision,
        refused(
            Rule::R09WorkspaceReuse,
            ReuseRefusal::LeaseHeld { remaining_ms: 0 }
        )
    );
}
/// T07-RC-32 · RC06 recovery clocks: a lease from another receiver clock epoch is not comparable
/// with the current clock; a reopened receiver never reinterprets an old lease as expired.
#[test]
fn lease_from_another_receiver_epoch_is_not_comparable() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &leased(Acknowledgement::NotSeen),
        &writable(Some(at(CLOCK2, u64::MAX))),
    );
    assert_eq!(
        decision,
        refused(
            Rule::R09WorkspaceReuse,
            ReuseRefusal::LeaseClockNotComparable {
                lease_epoch: CLOCK.to_owned(),
                clock_epoch: CLOCK2.to_owned(),
            },
        )
    );
}
/// T07-RC-33 · without a clock value nothing about the lease can be decided; reuse is refused.
#[test]
fn missing_clock_refuses_reuse_without_deciding_expiry() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &leased(Acknowledgement::NotSeen),
        &writable(None),
    );
    assert_eq!(
        decision,
        refused(Rule::R09WorkspaceReuse, ReuseRefusal::ClockUnavailable)
    );
}
/// T07-RC-34 · a writable workspace with no lease at all is refused too: absence of a lease is
/// not a release.
#[test]
fn writable_workspace_without_a_lease_is_refused() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &running(),
        &writable(Some(at(CLOCK, 1))),
    );
    assert_eq!(
        decision,
        refused(
            Rule::R09WorkspaceReuse,
            ReuseRefusal::NotLeasedWritable { bytes: 41 }
        )
    );
}
/// T07-RC-35 · a live worker of the same identity holds its settled attempt's workspace: reuse is
/// refused as a live holder regardless of the ledger's cleanup column.
#[test]
fn live_holder_of_a_settled_attempt_refuses_reuse() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Verifying),
        &settled(),
        &cleaned(ProcessCustody::LiveSameIdentity),
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R09WorkspaceReuse,
            reconciliation: Reconciliation::WorkspaceReuseRefused {
                reason: ReuseRefusal::LiveHolder,
                process: ProcessCustody::LiveSameIdentity,
            },
        }
    );
}

// ---- ambiguous effects -------------------------------------------------------------------

/// T07-RC-36 · ambiguous external outcomes remain explicit: an unknown effect is retained as unknown.
#[test]
fn unknown_effect_remains_explicit() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::EffectUnknown),
        &attempt(AttemptState::Unknown, Effect::Unknown, Cleanup::Unknown),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R10EffectAmbiguity,
            Unknown::EffectUnknown,
            ProcessCustody::Absent,
            false
        )
    );
}
/// T07-RC-37 · a pending effect is distinct from an unknown one and equally never resolved here.
#[test]
fn pending_effect_remains_distinct_from_unknown() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::EffectUnknown),
        &attempt(AttemptState::Unknown, Effect::Pending, Cleanup::Unknown),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R10EffectAmbiguity,
            Unknown::EffectPending,
            ProcessCustody::Absent,
            false
        )
    );
}
/// T07-RC-38 · an attempt settled with a committed effect but unknown cleanup is a cleanup
/// candidate once the readback is complete: the ledger column is what remains to settle.
#[test]
fn committed_effect_with_unknown_cleanup_is_a_ledger_settlement_candidate() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::EffectUnknown),
        &attempt(AttemptState::Unknown, Effect::Committed, Cleanup::Unknown),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R11CleanupReadback,
            reconciliation: Reconciliation::CleanupCandidate {
                what: vec![CleanupTarget::LedgerSettlement {
                    cleanup: Cleanup::Unknown,
                }],
                process: ProcessCustody::Absent,
            },
        }
    );
}

// ---- partial cleanup ---------------------------------------------------------------------

/// T07-RC-39 · a successful call is not a changed state: without a cleanup readback the settled
/// attempt's cleanup is unverified, not done.
#[test]
fn cleanup_without_readback_is_unverified() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Verifying),
        &settled(),
        &absent(),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R11CleanupReadback,
            Unknown::CleanupUnverified,
            ProcessCustody::Absent,
            false
        )
    );
}
/// T07-RC-40 · partial cleanup produces a cleanup candidate naming exactly what remains, in order.
#[test]
fn partial_cleanup_names_what_remains_in_order() {
    let obs = Observations {
        cleanup: CleanupReadback::Partial {
            remaining: vec!["workspace/output".to_owned(), "process-group".to_owned()],
        },
        ..absent()
    };
    let decision = reconcile(&ledger(), &open(TaskState::Verifying), &settled(), &obs);
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R11CleanupReadback,
            reconciliation: Reconciliation::CleanupCandidate {
                what: vec![
                    CleanupTarget::Remaining {
                        name: "workspace/output".to_owned(),
                    },
                    CleanupTarget::Remaining {
                        name: "process-group".to_owned(),
                    },
                ],
                process: ProcessCustody::Absent,
            },
        }
    );
}
/// T07-RC-41 · a complete readback against a ledger still marked pending leaves the ledger
/// settlement itself as the candidate; reopen does not settle it.
#[test]
fn complete_readback_with_pending_ledger_cleanup_is_a_settlement_candidate() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Verifying),
        &attempt(AttemptState::Settled, Effect::None, Cleanup::Pending),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R11CleanupReadback,
            reconciliation: Reconciliation::CleanupCandidate {
                what: vec![CleanupTarget::LedgerSettlement {
                    cleanup: Cleanup::Pending,
                }],
                process: ProcessCustody::Absent,
            },
        }
    );
}
/// T07-RC-42 · a terminal task with settled cleanup, complete readback and an absent worker is
/// the one positive release: evidence, not expiry, licenses it.
#[test]
fn failed_task_with_verified_cleanup_is_releasable() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Failed),
        &settled(),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R11CleanupReadback,
            reconciliation: Reconciliation::WorkspaceReleasable {
                cleanup_readback: CleanupReadback::Complete,
                process: ProcessCustody::Absent,
                task_state: TaskState::Failed,
            },
        }
    );
}
/// T07-RC-43 · PID reuse does not attach to the wrong work: for a settled attempt a reused PID is
/// evidence our worker is gone, so cleanup can proceed, and the reused identity is carried.
#[test]
fn reused_pid_on_a_settled_attempt_is_evidence_of_absence_for_cleanup() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Abandoned),
        &settled(),
        &cleaned(reused()),
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R11CleanupReadback,
            reconciliation: Reconciliation::WorkspaceReleasable {
                cleanup_readback: CleanupReadback::Complete,
                process: reused(),
                task_state: TaskState::Abandoned,
            },
        }
    );
}

// ---- restart at the verification, evidence and acceptance boundaries ---------------------

fn boundary(verification: Verification, evidence: Evidence) -> AttemptFacts<'static> {
    AttemptFacts {
        verification,
        evidence,
        ..settled()
    }
}
fn outstanding(
    task_state: TaskState,
    verification: Verification,
    evidence: Evidence,
    acceptance_prepared: bool,
) -> Decision {
    Decision {
        rule: Rule::R12VerificationBoundary,
        reconciliation: Reconciliation::VerificationOutstanding {
            task_state,
            verification,
            evidence,
            acceptance_prepared,
        },
    }
}
/// T07-RC-44 · restart after the worker returned: verification is outstanding, nothing is inferred.
#[test]
fn restart_after_worker_return_leaves_verification_outstanding() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Verifying),
        &boundary(Verification::None, Evidence::Unassessed),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        outstanding(
            TaskState::Verifying,
            Verification::None,
            Evidence::Unassessed,
            false
        )
    );
}
/// T07-RC-45 · restart after evidence publication: a published object is not a verification.
#[test]
fn published_evidence_is_not_verification() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Verifying),
        &boundary(Verification::None, Evidence::Published),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        outstanding(
            TaskState::Verifying,
            Verification::None,
            Evidence::Published,
            false
        )
    );
}
/// T07-RC-46 · restart after the verifier returned: a recorded pass is not an acceptance.
#[test]
fn recorded_pass_is_not_acceptance() {
    let passed = Verification::Recorded {
        verdict: Verdict::Passed,
        cleanup_settled: true,
    };
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Verifying),
        &boundary(passed, Evidence::Published),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        outstanding(TaskState::Verifying, passed, Evidence::Published, false)
    );
}
/// T07-RC-47 · a prepared (published, uncommitted) acceptance is reported as prepared, not committed.
#[test]
fn prepared_acceptance_is_not_committed_acceptance() {
    let passed = Verification::Recorded {
        verdict: Verdict::Passed,
        cleanup_settled: true,
    };
    let task = TaskFacts {
        candidate: AcceptanceCandidate::Prepared {
            verification_event: CHECK,
        },
        ..open(TaskState::Verifying)
    };
    let decision = reconcile(
        &ledger(),
        &task,
        &boundary(passed, Evidence::Published),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        outstanding(TaskState::Verifying, passed, Evidence::Published, true)
    );
}
/// T07-RC-48 · a recorded failure keeps the task at repair pending with the verdict carried.
#[test]
fn recorded_failure_remains_repair_pending() {
    let failed = Verification::Recorded {
        verdict: Verdict::Failed,
        cleanup_settled: false,
    };
    let decision = reconcile(
        &ledger(),
        &open(TaskState::RepairPending),
        &boundary(failed, Evidence::Absent),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        outstanding(TaskState::RepairPending, failed, Evidence::Absent, false)
    );
}
/// T07-RC-49 · a settled attempt under a task state the store cannot produce for it is not interpreted.
#[test]
fn settled_attempt_under_unexpected_task_state_is_not_interpreted() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Running),
        &settled(),
        &cleaned(ProcessCustody::Absent),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R14UnexpectedState,
            Unknown::TaskStateUnexpected {
                state: TaskState::Running,
            },
            ProcessCustody::Absent,
            false,
        )
    );
}
/// T07-RC-50 · an attempt state the store never inserts (`queued`) is refused rather than dispatched.
#[test]
fn queued_attempt_state_is_not_interpreted() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Queued),
        &attempt(AttemptState::Queued, Effect::None, Cleanup::None),
        &live(),
    );
    assert_eq!(
        decision,
        unknown(
            Rule::R14UnexpectedState,
            Unknown::UnexpectedAttemptState {
                state: AttemptState::Queued,
            },
            ProcessCustody::LiveSameIdentity,
            false,
        )
    );
}

// ---- restored epochs and cursors ---------------------------------------------------------

fn restored() -> LedgerFacts<'static> {
    LedgerFacts {
        restored_from: Some(PRIOR),
        mode: Mode::Reconciliation,
        ..ledger()
    }
}
/// T07-RC-51 · restored ledger epochs cannot silently reuse prior event cursors: a cursor from the
/// epoch the ledger was restored from is refused by that name.
#[test]
fn cursor_from_the_restored_prior_epoch_is_refused_by_name() {
    assert_eq!(
        cursor(&restored(), PRIOR, 3),
        stale_cursor(CursorRefusal::PriorEpochOfRestore, PRIOR, 3)
    );
}
/// T07-RC-52 · a cursor from any other epoch is refused as an epoch change.
#[test]
fn cursor_from_another_epoch_is_refused_as_changed() {
    assert_eq!(
        cursor(&ledger(), OTHER, u64::MAX),
        stale_cursor(CursorRefusal::EpochChanged, OTHER, u64::MAX)
    );
}
/// T07-RC-53 · a same-epoch cursor beyond the high-water mark is refused as a future sequence.
#[test]
fn same_epoch_future_sequence_is_refused() {
    assert_eq!(
        cursor(&ledger(), EPOCH, HIGH_WATER + 1),
        stale_cursor(CursorRefusal::FutureSequence, EPOCH, HIGH_WATER + 1)
    );
}
/// T07-RC-54 · a same-epoch cursor at the high-water mark is snapshot-only, never a replay.
#[test]
fn same_epoch_cursor_at_high_water_is_snapshot_only() {
    let decision = cursor(&ledger(), EPOCH, HIGH_WATER);
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R13CursorEpoch,
            reconciliation: Reconciliation::CursorSnapshotOnly {
                epoch: EPOCH.to_owned(),
                sequence: HIGH_WATER,
                event_high_water: HIGH_WATER,
                mode: Mode::Normal,
                replay: false,
            },
        }
    );
    assert!(!decision.reconciliation.permits_execution());
}
/// T07-RC-55 · on a restored ledger its own new epoch is accepted for a snapshot, with the
/// reconciliation mode carried and replay still false.
#[test]
fn restored_ledgers_own_epoch_is_snapshot_only_in_reconciliation_mode() {
    assert_eq!(
        cursor(&restored(), EPOCH, 2),
        Decision {
            rule: Rule::R13CursorEpoch,
            reconciliation: Reconciliation::CursorSnapshotOnly {
                epoch: EPOCH.to_owned(),
                sequence: 2,
                event_high_water: HIGH_WATER,
                mode: Mode::Reconciliation,
                replay: false,
            },
        }
    );
}

// ---- contracts the report relies on ------------------------------------------------------

/// T07-RC-56 · no arm permits execution: every decision the roster of inputs produces reads
/// `permits_execution() == false`, and the two arms that carry a flag carry `false`.
#[test]
fn no_decision_permits_execution() {
    let decisions = [
        reconcile(&ledger(), &open(TaskState::Running), &running(), &live()),
        reconcile(&ledger(), &open(TaskState::Running), &running(), &absent()),
        reconcile(&ledger(), &accepted(), &settled(), &absent()),
        reconcile(&ledger(), &cancelled(), &settled(), &absent()),
        reconcile(
            &ledger(),
            &open(TaskState::Failed),
            &settled(),
            &cleaned(ProcessCustody::Absent),
        ),
        reconcile(
            &ledger(),
            &open(TaskState::Verifying),
            &settled(),
            &cleaned(ProcessCustody::Absent),
        ),
        cursor(&ledger(), EPOCH, 1),
        cursor(&restored(), PRIOR, 1),
    ];
    let names: Vec<&str> = decisions.iter().map(|d| d.reconciliation.name()).collect();
    assert_eq!(
        names,
        [
            "reattach_observation_only",
            "retain_unknown",
            "acceptance_stands",
            "cancellation_stands",
            "workspace_releasable",
            "verification_outstanding",
            "cursor_snapshot_only",
            "refuse_stale_cursor",
        ]
    );
    for decision in &decisions {
        assert!(!decision.reconciliation.permits_execution(), "{decision:?}");
    }
}
/// T07-RC-57 · the ledger vocabularies round-trip through `parse`/`name` and refuse any other
/// spelling (case, whitespace), so a report field can never be defaulted from bad text.
#[test]
fn ledger_vocabularies_round_trip_and_refuse_other_spellings() {
    for (text, state) in [
        ("queued", AttemptState::Queued),
        ("running", AttemptState::Running),
        ("settled", AttemptState::Settled),
        ("unknown", AttemptState::Unknown),
    ] {
        assert_eq!(AttemptState::parse(text), Some(state));
        assert_eq!(state.name(), text);
    }
    assert_eq!(Effect::parse("committed"), Some(Effect::Committed));
    assert_eq!(
        Cleanup::parse("pending").map(Cleanup::name),
        Some("pending")
    );
    assert_eq!(
        TaskState::parse("cancellation_requested"),
        Some(TaskState::CancellationRequested)
    );
    assert_eq!(TaskState::EffectUnknown.name(), "effect_unknown");
    assert_eq!(Mode::parse("reconciliation"), Some(Mode::Reconciliation));
    assert_eq!(Verdict::parse("timeout"), Some(Verdict::Timeout));
    for bad in ["Running", " running", "running ", "", "none "] {
        assert_eq!(AttemptState::parse(bad), None, "{bad:?}");
        assert_eq!(Effect::parse(bad), None, "{bad:?}");
        assert_eq!(TaskState::parse(bad), None, "{bad:?}");
    }
    assert_eq!(ProcessCustody::Unobserved.name(), "unobserved");
    assert_eq!(reused().differs(), &[Dimension::StartTicks]);
    assert_eq!(ProcessCustody::Absent.differs(), &[]);
    assert_eq!(ProcessCustody::Absent.error(), None);
    assert_eq!(
        ProcessCustody::Unreadable {
            error: "ns: Permission denied".to_owned(),
        }
        .error(),
        Some("ns: Permission denied")
    );
    assert_eq!(Rule::R13CursorEpoch.id(), "R13");
}
/// T07-RC-58 · `permits_execution` reads the flags the two flag-carrying arms hold rather than
/// answering a constant: a hand-built reattach with `redispatch: true` or a snapshot with
/// `replay: true` reports `true`, which is what lets a consumer refuse such a report.
#[test]
fn permits_execution_reads_the_carried_flags() {
    let reattach = Reconciliation::ReattachObservationOnly {
        generation: ATTEMPT_GENERATION,
        process: ProcessCustody::LiveSameIdentity,
        pi_queue: PiQueueCustody::Idle,
        cancellation_pending: false,
        redispatch: true,
    };
    let snapshot = Reconciliation::CursorSnapshotOnly {
        epoch: EPOCH.to_owned(),
        sequence: 1,
        event_high_water: HIGH_WATER,
        mode: Mode::Normal,
        replay: true,
    };
    assert!(reattach.permits_execution());
    assert!(snapshot.permits_execution());
    assert!(
        !Reconciliation::RetainUnknown {
            reason: Unknown::ProcessUnobserved,
            process: ProcessCustody::Unobserved,
            cancellation_pending: true,
        }
        .permits_execution()
    );
}
/// T07-RC-59 · a still-writable workspace is never releasable, even after a complete cleanup
/// readback on a terminal task. The settled path used to reach `WorkspaceReleasable` without
/// reading the workspace at all, while the unsettled path refused the same workspace through
/// `lease_refusal`; both now go through that one rule (T07 obligation 9).
#[test]
fn a_writable_workspace_is_not_releasable_after_cleanup() {
    let decision = reconcile(
        &ledger(),
        &open(TaskState::Failed),
        &settled(),
        &Observations {
            workspace: WorkspaceReadback::Writable { bytes: 4096 },
            ..cleaned(ProcessCustody::Absent)
        },
    );
    assert_eq!(
        decision,
        Decision {
            rule: Rule::R09WorkspaceReuse,
            reconciliation: Reconciliation::WorkspaceReuseRefused {
                reason: ReuseRefusal::NotLeasedWritable { bytes: 4096 },
                process: ProcessCustody::Absent,
            },
        }
    );
}
/// T07-RC-60 · a settled attempt whose process could not be read is not released: "we could
/// not look" is not evidence the holder is gone. Unobserved custody is the other case and is
/// pinned beside it: a settled attempt whose observation was never a local process (RC-24) has no
/// local holder to protect, so its verified cleanup is still releasable.
#[test]
fn unreadable_custody_does_not_release_a_settled_workspace_but_unobserved_does() {
    let with = |custody: ProcessCustody| {
        reconcile(
            &ledger(),
            &open(TaskState::Failed),
            &settled(),
            &Observations {
                workspace: WorkspaceReadback::Released,
                ..cleaned(custody)
            },
        )
    };
    let unreadable = ProcessCustody::Unreadable {
        error: "EACCES".to_owned(),
    };
    assert_eq!(
        with(unreadable.clone()),
        unknown(
            Rule::R07ProcessNotOurs,
            Unknown::ProcessUnreadable {
                error: "EACCES".to_owned()
            },
            unreadable,
            false
        )
    );
    assert_eq!(
        with(ProcessCustody::Unobserved).rule,
        Rule::R11CleanupReadback
    );
}
