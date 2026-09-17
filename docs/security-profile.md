# Daybreak Blue — full-stack defensive security profile

Requested model: **`gpt-daybreak-blue-latest`**. Defensive security review and hardening assistance for every declared module and integrated release; model output is evidence to verify, not acceptance authority. Planning analysis is read-only by default. Reproduction uses explicitly owned disposable targets under actual action authorization; engine repairs, tests, dependencies and release work require Luke’s actual coding instruction and applicable scope.

Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.

[Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) · [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Completion gates](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Completion) · [Justfile/runbooks](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks) · [Restart handoff](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Context%20Handoff) · [README](file:///var/home/herdr-engineering-engine-v3/README.md) · [Quick start](file:///var/home/herdr-engineering-engine-v3/QUICK_START.md)

## Evidence and availability

The local cache advertised the requested alias when inspected; Codex CLI reported 0.154.0. Selected metadata and config fields are retained in [the scoped observation](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Atlas%2Fevidence%2Fdaybreak-20260915%2Fmodel-selection-observation.json). Cache client version differs from installed CLI version, so cache contents are advertised metadata, not proof of a current provider response. The inspected default was gpt-6-astra, xhigh, with review_model unset; no global model config was changed. This corpus update does not claim an interactive `/model` switch or current root-session model change.

OpenAI describes Daybreak Blue as a defensive cybersecurity alias and notes separate provisioning. Do not infer future account access or an immutable underlying revision from today’s cache. [Official model page](https://developers.openai.com/api/docs/models/gpt-daybreak-blue-latest).

## Select with /model in Codex

1. Open the scoped security-review session and inspect its current working root, permissions and task brief.
2. In the **Codex composer**, type `/model` and press Enter. Select **Daybreak Blue**, confirming the exact ID `gpt-daybreak-blue-latest`; choose and record an available reasoning effort appropriate to the review budget.
3. Inspect `/status` and the session/model selection display, then retain effective model/effort and run metadata when exposed. An unavailable model, failed selection, mismatch or unknown identity is a visible gap; do not silently choose another model.
4. If using `/review`, check the effective `review_model` override as well; `/review` may use that override instead of the session model. Bind the actual reviewer identity. A read-only scoped review prompt is also possible after selection.
5. Begin the bounded defensive review with exact subject and evidence obligations. Return findings for independent reproduction and disposition.

`/model` and `/status` are interactive Codex commands, not Bash commands or Just recipes. The [official model guide](https://learn.chatgpt.com/docs/models) documents the picker and the alternative `--model`/`-m` launch option. The [developer command guide](https://learn.chatgpt.com/docs/developer-commands?surface=cli) documents review behavior. A quoted selection instruction in a runbook does not demonstrate that selection occurred.

Current interactive identity observation: `{"effective_model": null, "effective_revision": null, "effort": null, "identity_verdict": "requested only; interactive picker not evidenced", "observation_source": null, "review_model_override": null, "selection_observed": false}`. A delegated reviewer or CLI launch override is recorded separately and cannot prove the picker was exercised.

## Security loop across deployment stages

| Stage | Existing gates | Required return evidence |
| --- | --- | --- |
| S01 · Threat model before implementation | G03/G06 | Bound module/data/trust/effect surfaces, threats and intended-fault/benign controls. |
| S02 · Review the actual candidate | G04/G05/G06 | Inspect exact code/interface/dependency/config/test subject; reproduce candidate findings in admitted disposable scope. |
| S03 · Repair and reverify | G04/G06/G07 | Authorized implementation owner repairs; independent deterministic tests and reviewer recheck bind the changed integrated subject. |
| S04 · Integrated release and migration review | G06/G08/G11/G13 | Close applicable T17/T18/T25-T27 obligations, installation/recovery and migration immutability findings; separate admission remains required. |

The orchestrator owns scope and integration; the module thread owns repairs; the Daybreak thread supplies defensive analysis; qualified checkers and independent reviewers reproduce and close findings on the actual subject. A different model or thread alone is not independent assurance. Initial threat-model work can remain read-only in planning; engine fixes, mutation campaigns and release actions require their actual coding/action authority. No current production module is security-qualified.

## Module security matrix and return anchors

| Module | Security focus | Completion / source |
| --- | --- | --- |
| contracts | Untrusted schema/frame bounds, compatibility, identity preservation and parser fuzz controls. | [HEE3-DONE-contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-contracts) · [src/contracts.rs](file:///var/home/herdr-engineering-engine-v3/src/contracts.rs) |
| task | Unauthorized state transitions, forged/stale acceptance, replay, cancellation/acceptance races and effect custody. | [HEE3-DONE-task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-task) · [src/task.rs](file:///var/home/herdr-engineering-engine-v3/src/task.rs) |
| store | Transaction integrity, migration immutability, concurrent writers, corruption and restored outstanding obligations. | [HEE3-DONE-store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-store) · [src/store.rs](file:///var/home/herdr-engineering-engine-v3/src/store.rs) |
| roster | Spoofed agent/model/service identity, stale health, capability escalation and provenance. | [HEE3-DONE-roster](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-roster) · [src/roster.rs](file:///var/home/herdr-engineering-engine-v3/src/roster.rs) |
| route | Eligibility/privacy bypass, silent model substitution, stale evaluations and constrained fallback. | [HEE3-DONE-route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-route) · [src/route.rs](file:///var/home/herdr-engineering-engine-v3/src/route.rs) |
| budget | Concurrent overcommit, omitted checker/context usage and ambiguous-effect liabilities. | [HEE3-DONE-budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-budget) · [src/budget.rs](file:///var/home/herdr-engineering-engine-v3/src/budget.rs) |
| worker | Process/workspace custody, hostile output, real model identity, cancellation and residual writable children. | [HEE3-DONE-worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-worker) · [src/worker/mod.rs](file:///var/home/herdr-engineering-engine-v3/src/worker/mod.rs) |
| check | Forged evidence, compromised fixtures, invalid denominator and independent verifier/collector custody. | [HEE3-DONE-check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-check) · [src/check.rs](file:///var/home/herdr-engineering-engine-v3/src/check.rs) |
| recovery | PID reuse, stale epochs, duplicate execution, incomplete cleanup and lost-response readback. | [HEE3-DONE-recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-recovery) · [src/recovery.rs](file:///var/home/herdr-engineering-engine-v3/src/recovery.rs) |
| cohort | Cross-thread write collisions, context/grant contamination, missing/contradictory child evidence and false parent joins. | [HEE3-DONE-cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-cohort) · [src/cohort.rs](file:///var/home/herdr-engineering-engine-v3/src/cohort.rs) |
| context | Prompt injection, unsafe source paths, bounded expansion, data minimization and provenance. | [HEE3-DONE-context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-context) · [src/context.rs](file:///var/home/herdr-engineering-engine-v3/src/context.rs) |
| notify | Visibility leakage, outbox-before-commit, replay/epoch errors and slow subscriber resource abuse. | [HEE3-DONE-notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-notify) · [src/notify.rs](file:///var/home/herdr-engineering-engine-v3/src/notify.rs) |
| service | Unauthorized lifecycle effects, wrong owner/unit, useful-health spoofing and uncertain replay. | [HEE3-DONE-service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-service) · [src/service.rs](file:///var/home/herdr-engineering-engine-v3/src/service.rs) |
| herdr | Client/pane authority confusion, stale UI truth, reconnect/cancel and duplicate submission. | [HEE3-DONE-herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-herdr) · [src/herdr.rs](file:///var/home/herdr-engineering-engine-v3/src/herdr.rs) |
| numerical | Rust/Julia framing, memory/device bounds, immutable dataset identity, units and untrusted artifacts. | [HEE3-DONE-numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-numerical) · [src/numerical.rs](file:///var/home/herdr-engineering-engine-v3/src/numerical.rs) |
| julia | Numerical oracle validity, data leakage, nonfinite/shape/unit input, out-of-domain behavior and reproducibility. | [HEE3-DONE-julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-julia) · [julia/src/HabitatAnalysis.jl](file:///var/home/herdr-engineering-engine-v3/julia/src/HabitatAnalysis.jl) |
| app | Local socket permissions/peer custody, startup races, config validation, dependency composition and shutdown. | [HEE3-DONE-app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-app) · [src/main.rs](file:///var/home/herdr-engineering-engine-v3/src/main.rs) |
| actions | Hidden capability leakage, schema/version drift, ungranted effect and CLI/tool/UDS parity. | [HEE3-DONE-actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-actions) · [src/actions.rs](file:///var/home/herdr-engineering-engine-v3/src/actions.rs) |
| bash | Literal argv, shell metacharacters, pipeline producer status, environment and cancellation. | [HEE3-DONE-bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-bash) · [integrations/bash/README.stub.md](file:///var/home/herdr-engineering-engine-v3/integrations/bash/README.stub.md) |
| pi_extension | Host/tool version, stale callbacks, cross-call identity, reload and permission parity. | [HEE3-DONE-pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-pi_extension) · [integrations/pi/README.stub.md](file:///var/home/herdr-engineering-engine-v3/integrations/pi/README.stub.md) |
| skills | Instruction-as-grant confusion, unsafe references, stale dependencies and bounded source scope. | [HEE3-DONE-skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-skills) · [skills/README.stub.md](file:///var/home/herdr-engineering-engine-v3/skills/README.stub.md) |
| workflows | Injected steps, cycles, retries after uncertain effects, source/wrapper drift and independent final verification. | [HEE3-DONE-workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FCompletion%2FDONE-workflows) · [workflows/README.stub.md](file:///var/home/herdr-engineering-engine-v3/workflows/README.stub.md) |

## Required review record

- requested model alias and effective model/revision when exposed; unknown stays unknown
- Codex version, session/worker identity, reasoning effort, selection observation and review_model override disposition
- exact candidate/criteria/interface/standard/dependency/config/fixture identities
- bounded review scope, allowed effects/resources and threat-model revision
- reproducible finding, location, impact, intended fault and benign control
- repair identity, independent raw results and re-verification disposition
- remaining uncertainty/material findings, reviewer identity and acceptance authority separation
- Structured identity observation: requested_model, selection_surface, selection_observed, effective_model, effective_revision if exposed, effort, review_model_override, observation_source, identity_verdict. A launch-time override or delegated agent invocation is not interactive /model evidence. Unknown fields remain null, never inferred.
- Repair binding: pre-fix finding subject digest, post-fix candidate digest, exact repair diff/range, affected dependency/config/fixture/harness identities, finding-to-test mapping and regression-selection rationale. Rerun affected qualifications and integrated regressions; material subject/profile changes invalidate earlier applicability.
- Security profile revision/hash plus completion/interface standard hashes; retain independent review and exact evidence for an intentional profile revision. The editable semantic baseline is a drift tripwire, not authenticated admission.

Treat model findings as claims to reproduce. Qualify intended-fault and benign controls, protect verifier/fixture custody, preserve actual exit status and raw diagnostics, and retain failed/invalid/unmeasured cells. The 50-case and zero-warning requirements remain mandatory. Model review neither replaces deterministic checks nor grants admission, migration execution, deployment or permission to scan unrelated systems.

## Configuration and failure behavior

[Configuration mapping](file:///var/home/herdr-engineering-engine-v3/config/README.md) connects desired model, agent, route, service and numerical profiles to the existing comment stubs. It supplies no active credentials, provider connection or engine runtime configuration. Missing Daybreak availability leaves the designated security review pending; a replacement requires an explicit reviewed profile decision. Alias/effort/harness changes invalidate affected reviewer applicability and require re-review where material.

[RB05 full procedure](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05) · [Corpus update and separate acceptance](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FUpdate%20Protocol) · [Current integration review](file:///var/home/Louranicas/planning/herdr-engine-vault-20260915/ultra-map-review/daybreak-20260915/REVIEW.md)

## Adopted supplementary requirements

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). These active clauses strengthen the existing owner; none relaxes its baseline or claims implementation.

### F5 · Security and hardening

**Recorded score:** 83/100. **Conditional target:** 92/100; no promotion from documentation adoption.

**Primary owners:** [worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-worker), [check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check), [service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-service), [actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-actions), [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app), [store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store), [context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-context), [bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-bash), [pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-pi_extension), [skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-skills), [workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-workflows).

**Owning task contracts:** [T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26), [T15](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T15), [T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17), [T27](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T27).

**Governing standard/procedure routes:** [Daybreak Profile](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Security%2FDaybreak%20Profile) · [RB05](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FRunbooks%2FRB05). These are generated views of the same adopted clauses; edit the convention once.

- **F5-C01:** Complete a threat model of principals, assets and grants across agent prompts, context sources, code workspaces, credentials, sockets, model/provider transports, evidence, dependency updates and service controls.
- **F5-C02:** Treat build.rs, procedural macros, generated scripts, test binaries, doctests and Julia package hooks as candidate-controlled execution. A trusted wrapper cannot confer trust on these subprocesses.
- **F5-C03:** Specify separate trusted-host development and adversarial qualification profiles. State writable/readable mounts, permitted network destinations, environment/secret allowlist, process/resource limits, cancellation, descendant cleanup and the evidence return channel. Same-UID permissions alone cannot substantiate hostile-code isolation.
- **F5-C04:** Keep the trusted collector, protected fixtures, evidence store, ledger and operator socket outside candidate write authority. Give each worker only its admitted workspace and scoped capability; do not deliver the operator socket as a general tool channel.
- **F5-C05:** Use the existing Daybreak profile/RB05: verify actual effective model identity and review overrides for the scoped review, retain the reviewer evidence and independently reproduce findings. Model choice grants neither execution authority nor acceptance.
- **F5-C06:** Specify exact dependency/source/license inventory, update review and advisory dispositions for the eventual locked package. Record finding impact, reproduction, repair, regression, pre/post-fix identities, residual owner and invalidation trigger.

**Required future proof:**

- A valid packaged slice runs under the declared isolation profile, while hostile build/test hooks fail to alter protected paths, steal unintended secrets, misuse sockets, impersonate the collector, contact forbidden destinations or retain writable descendants.
- Each security fault reaches its intended boundary and produces a meaningful refusal/readback; successful benign cases prove the fixture was viable.
- T27 independently reproduces and closes material findings against the integrated packaged subject; dependency and prompt/context attacks are included, with excluded or unmeasured scope explicit.

**Reassessment:** 90 requires demonstrated hostile controls on a packaged slice. 92 requires integrated finding closure and no unresolved material finding in the claimed profile.

**Complexity guard:** Select controls from actual threats. Do not add a security platform, permanent scanner daemon or permissions merely because a tool can use them.

