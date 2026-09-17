# Context brief and agent cohesion

Use the existing task handoff/evidence location. This is a portable brief format, not another task-state ledger or an engine admission receipt.

## Brief fields

| Field | Required content |
| --- | --- |
| Scope and subject | Module ID, original task, active instruction/authority, exact generation and candidate revision/hashes, host/Toolbx root mapping. |
| Intended step | Current implemented facts versus intended behavior, smallest useful change, invariants and complexity boundary. |
| Ownership | Primary owner, supporting modules/files, write/resource claims, original prerequisites and their actual status. |
| Relationship coverage | Affected dependency and consumer; caller input; callee result/error; cancellation/timeout; usage/resource ownership; evidence/admission/notification return. Mark absent or unread directions. |
| Verification design | Exact criterion/RC/gate, supported profile, independent oracle, intended fault and benign control, mutation/assimilation/security obligation and expected raw evidence. |
| Source coverage | Path/root, section/line/symbol or JSON pointer, hash/revision/date, current versus capture, read state, reason selected, limitations and conflicting claims. |
| Tools and budget | Actually observed executable/interface, effect scope, version/profile when required, unavailable or proposed tool, discovery/packet/supplement/re-read costs. |
| Gaps and continuation | Material unknowns, omitted sources/relationships, next exact reads, conclusion scope, next original task and return obligations. |

Do not assign a numerical mastery score from file counts. Context is sufficient only for the named next step when its invariants, changed boundaries, authority and verification oracle are understood and no unresolved gap could materially alter that step. Future performance measurements remain future work.

## Specialist threads when authorized

The orchestrator selects the original task and owns parent integration. Delegate concrete independent questions, not copies of the whole corpus: boundary/schema scout, relevant implementation owner, test/oracle verifier, security reviewer, numerical reviewer or operations/recovery reviewer as needed. Each brief names exact sources and hashes, scope, allowed effects, write ownership, budget, prerequisites and return evidence. A scout may remain read-only even when the wider task permits implementation.

A child returns facts with locators, design implications, counter-evidence, changed paths, checks actually run and unresolved obligations. The parent reconciles differing assumptions and shared schemas, reads critical source itself, verifies consequential results and closes integrated return paths. One child's success cannot close the parent. Parallel work must not create competing writers, process owners, task ledgers or silent model fallbacks.

## Re-scout on change

Reopen affected context after interface/schema/config/fixture/toolchain changes, a failed check, a newly discovered caller or evidence that contradicts the design. Follow both directions and across clusters. Preserve unchanged snapshots for reuse; invalidate only claims whose subjects or assumptions changed. An authorized implementation task resumes after this review; the skill does not create an extra permission requirement. Engine coding still requires Luke's actual instruction, and completion still requires the existing qualified admission process.

[Module workflow](file:///var/home/herdr-engineering-engine-v3/docs/module-context.md) · [Runbooks](file:///var/home/herdr-engineering-engine-v3/runbooks/README.md) · [Context handoff](file:///var/home/herdr-engineering-engine-v3/corpus/CONTEXT_HANDOFF.md)

## Agents with different filesystem access

A coding agent needs access to the relevant source content and a route to request deeper context; it does not inherit unrestricted host or vault access. For a remote or isolated worker, the trusted context owner must provide authorized snapshots/excerpts with source identities, relationship records and omission diagnostics, then satisfy scoped follow-up requests through the selected context/tool boundary. Local file URLs are location hints when that worker cannot open them. Preserve denied/private-source boundaries and disclose missing inputs; never solve access by copying credentials, mounting an entire private vault or granting the operator socket. The planned engine context/worker/action modules own that future transport and its qualification. This documentation skill supplies a usable local workflow today, not a claim that the remote runtime delivery path already exists.
