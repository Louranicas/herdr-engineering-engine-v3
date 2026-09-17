# Module testing standards and conventions

**Active additive standard: HEE3-TEST-STD-001, revision 1.**

User-directed additive testing convention. The preserved atlas task predicates and this standard apply together; rendered Markdown, vault notes and codebase copies are projections. The user explicitly requires at least 50 meaningful tests per module and zero pedantic warnings/errors.

[Atlas master](file:///var/home/Louranicas/planning/herdr-engine-vision-20260915/MASTER_INDEX_habitat_engine.md) · [Vault master](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=00%20-%20Master%20Index) · [Ultra map](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FIndex) · [Codebase master](file:///var/home/herdr-engineering-engine-v3/README.md)

## Scope, authority and qualification boundary

This user-authorized revision adds a mandatory qualification standard to every one of the 22 planned modules, including Rust, Julia, Bash, Pi extensions, skills and workflows. It applies to implementation tests and their baseline build/lint targets, not merely release binaries. The original PLAN JSON and T01–T29 predicates remain the preserved design baseline. Current qualification requires **both** those predicates and this additive standard. T25 owns the quality recipe; T26/T27 govern hostile execution boundaries and finding closure; T17/T18–T20 retain integration, recovery and release obligations.

Each module must have **at least 50 qualifying, distinct, module-owned behavioral test cases** before its testing qualification can pass. All baseline compilation, test and lint runs must have **zero warnings and zero errors**, including zero pedantic Clippy diagnostics on admitted Rust targets and feature profiles. A minimum of 1,100 primary case credits is therefore required for all 22 modules; this is a requirement, not a claim that 1,100 tests exist.

Tests must demonstrate their specified behavior on the exact candidate. A count, coverage percentage or clean log alone is insufficient. Optional/deferred modules remain unqualified until admitted and checked; they do not receive an exemption or an invented pass. Existing reference-example and corpus-maintenance receipts retain their smaller, separate scopes and do not satisfy a production module's 50-case floor.

This update creates standards and documentation bindings. It does not create or execute engine tests, qualify an engine validator, deploy a service, or advance task acceptance. The observed engine sources remain deployment stubs. A later implementation request must deliver the actual tests and qualified collection path.

## Counting rules and the anti-padding convention

A qualifying case has a stable case ID, one primary module owner, an independently stated contract or invariant, input/equivalence class, expected outcome and oracle, fixture identity, test level, applicable profile set, and its own attributable execution result. Use names such as `TST-route-001` only after a real case is specified; empty numbered slots are not cases. Every credited case must be discovered, executed, and pass in each profile where its declared obligation applies. Required skips, unresolved failures, missing results, stale subjects and flaky rerun-only passes block qualification. Report discovered, selected, executed, passed, failed, skipped, quarantined and not-applicable values separately.

Count each behavior/obligation once. Multiple assertions supporting one scenario remain one case. One property is one case; seeds, generated inputs, fuzz iterations, shrinking steps, schedules and repetitions describe exploration depth. Parameter rows earn separate credits only when the reviewed manifest explains distinct equivalence classes or failure obligations and separate expected outcomes. Identical copied tests, renamed wrappers and the same scenario run in debug/release or several profiles do not multiply credits.

A shared integration execution can support several modules, but a single oracle receives one primary credit. Other modules receive secondary coverage unless the execution records separate module-specific cases with distinct obligations and oracles. Do not turn one assertion into several module credits. Test counts are per candidate and module, not pooled across releases, old commits, unrelated reference examples or incompatible environments.

Lint, formatting, dependency scans, compiler success, mutation operators and mutant executions are separate gates, not behavioral case credits. A compile-fail or detector fixture may count only when it is itself a qualifying module behavior with a precise expected refusal and independent oracle. Executable doctests may count if they assert a real module contract; examples that merely compile do not. Snapshot acceptance must be reviewed against an external specification; regenerating snapshots is not proof.

The floor is not a quota to fill with trivial cases. Every case must explain a failure it can detect, and every material obligation must have a test or an explicit unresolved gap. Fifty passing low-value cases cannot offset an untested authority or recovery boundary.

Every qualifying case must apply to at least one admitted profile. The admitted module profile matrix must itself be nonempty and closed before counting. Empty-profile, not-applicable, unsupported, unselected and unexecuted records earn no credit. Any unresolved material obligation or gap blocks full module qualification regardless of 50 passing cases; recording the gap does not waive it.

## Risk-directed test design and oracles

Each module maintains a reviewed case manifest and coverage matrix across public contracts; valid/invalid and boundary inputs; typed refusal/error behavior; state transitions and identities; replay/idempotency; cancellation/deadlines/resource limits; dependency failures and recovery; concurrency where relevant; security/authority/provenance; and observability/evidence integrity. No fixed per-family quotas are imposed. An inapplicable family needs a concrete reason tied to that module; the 50-case floor remains.

Prefer observable behavior at declared boundaries over private-function shape or implementation snapshots. Use separately derived expected values, small reference models, golden contracts with known provenance, metamorphic relations, round trips with independent checks, and differential comparisons when implementations have genuinely different failure modes. A producer and verifier sharing the same algorithm or model family must disclose their shared assumptions. Cross-family model review can challenge a test design but cannot replace execution evidence.

Preserve property/fuzz counterexamples and their seeds, generator/tool versions and concrete minimized inputs when available. A replayed seed can change meaning when a generator changes, so retain the input or fixture identity needed to reproduce the original failure. Model-based and concurrency exploration must record the explored bounds and unmodeled operations; bounded exploration is not a proof over every execution. [Proptest persistence](https://proptest-rs.github.io/proptest/proptest/failure-persistence.html) and [Loom scope](https://docs.rs/loom/latest/loom/) provide the relevant tool contracts.

Tests must be isolated, order-independent and deterministic where the domain permits. Inject clocks, deadlines and failure sources; avoid wall-clock sleeps as oracles. Use owned scratch and per-run resources, seeded stochastic exploration and preserved failure traces. Measure p95/p99 latency, resource exhaustion and numerical error against predeclared budgets when relevant; define comparison populations, uncertainty and acceptance thresholds before interpreting results. The project has no measured basis for asserting a top-7% quality ranking from a test count.

## Assimilation testing: seams and adopted lessons

Boundary assimilation verifies every declared module dependency and producer/consumer flow in both directions. Map the case to the atlas flow/API/IPC/action IDs and relevant module owners. Exercise compatible request/result exchange, malformed or unsupported versions, identity/correlation and ordering, typed refusal/error preservation, timeout/cancellation, retry/readback, and recovery or shutdown as applicable. Test Rust↔Julia exchanges, provider/worker adapters, Unix socket lifetime/peer/method boundaries, Bash argv and pipeline status, Pi extension versions, skill/tool admission and workflow joins through their real admitted interfaces. A mocked unit test must not masquerade as an installed seam or full-stack test.

Before accepting an upstream component, pin provenance and version, verify adapter semantics against an independent contract, preserve license/source evidence, test failure/upgrade behavior, and verify that imported defaults do not gain authority, silently change task state, consume unbounded resources or erase errors. Assimilate the useful component behind its module boundary; do not import the previous engine's architecture wholesale.

Rule assimilation turns a relevant learning into a maintained detector, invariant or gate. Each adopted rule needs a rule ID, source/lesson, owning module, detector and version, severity, workflow phase, tested workspace/subject scope, intended-fault negative control, and a valid use-pattern/benign-neighbor control. Prove that the detector actually reads the candidate and fires for the intended reason. A scan with zero findings and zero applicable working detectors is not a pass.

The Toolshed assimilation and gate-weight notes show why context-sensitive checks matter: a valid bounded-read pattern should not be rejected by a blanket method ban, and malformed fault formatting must not steal credit from the gate intended to detect a semantic defect. Retain the fault→gate matrix, baseline status, expected diagnostic, actual diagnostic, excluded/unmeasured cells and remaining blind spots. A rule is assimilated only when the required workflow invokes and verifies it, not when it appears in a note.

Assimilation cases can contribute to the 50-case floor only under the same distinct case, module-owner and oracle rules. Whole-stack scenarios remain additional release evidence; parent acceptance is independent of child-module success.

## Mutation testing and mutation of lint/detector gates

Keep a passing, warning-free unmutated baseline before a campaign. Materialize each mutant in an isolated disposable copy, bind its exact diff and subject, and use separate owned target/scratch directories. Confirm the runner reads that copy rather than the authoring checkout. Preserve configured filters, selected mutation surface, tool version, effective environment and the full outcome inventory. Bound concurrency and resources without sharing mutable build targets between unrelated candidates.

Classify outcomes separately as caught, missed, timeout, unviable, reviewed-equivalent, excluded and unmeasured. A build failure, incidental formatting failure or unavailable dependency is not proof that a behavioral test caught the intended fault. Timeouts need diagnosis and cannot automatically become kills. Equivalent mutants require a specific semantic argument and review evidence; do not erase them silently from the denominator. Preserve both raw tool outcomes and any subsequent reviewed disposition. [cargo-mutants outcomes](https://mutants.rs/using-results.html) and [retained outputs](https://mutants.rs/mutants-out.html) define the tool's native records.

Require every hand-seeded consequential fault to be detected for its intended reason, including bypassed authority checks, lost cancellation, duplicate dispatch, stale evidence, incorrect units and forged success where relevant. Unexplained surviving consequential mutants block qualification. Review all remaining survivors and exclusions; use meaningful public-contract regressions to close gaps. Report mutation adequacy with the selected population and disposition ledger rather than a single unqualified percentage. No arbitrary universal mutation-score percentage is introduced here.

Apply qualified generated mutation tooling to suitable Rust decision surfaces. For Julia, Bash, configuration and orchestration boundaries, use reviewed hand-seeded faults or a specifically qualified tool; do not claim cargo-mutants covers those languages. Include lint/detector-gate mutations: disabling a consequential rule, narrowing it so the known defect escapes, reading the wrong root, suppressing producer exit status, or accepting zero executed checks. Every such fixture must be otherwise valid so the intended gate bears the weight. Include benign near-neighbors to expose overbroad rejection.

Mutant runs and lint-gate campaigns remain separate evidence dimensions and do not themselves inflate the 50-case count. Recheck affected campaigns after fixes and run required integration regressions before closure.

Each module needs a nonempty reviewed mutation/fault campaign over its consequential behaviors. Zero generated opportunities, an empty hand-seeded set, or an all-unviable/excluded population remains unmeasured and unqualified. Declarative and integration modules use meaningful schema, argv, version, exit-status, authority or detector mutations instead of claiming the campaign is inapplicable.

## Zero-warning and zero-error baseline convention

The user requires **zero pedantic warnings and errors**. The qualification baseline must therefore emit no project compiler, test, rustdoc, formatter, lint or applicable static-analysis warnings/errors. Capture original producer exit status and complete diagnostics before rendering or filtering. A pipeline's final formatter, grep, tee or summarizer status cannot substitute for the checker. Log truncation, swallowed status, missing tools, zero targets, skipped profiles, stale cache output and successful authoring hooks cannot establish this gate.

For Rust, the pinned recipe must include `cargo fmt --all -- --check`, compiler warnings denied, `cargo clippy --workspace --all-targets --locked -- -D warnings -D clippy::pedantic`, the required unit/integration/target tests, and a **separate** `cargo test --workspace --doc --locked` invocation with rustdoc/compiler warnings denied. Repeat the relevant recipes across the explicit supported feature and build-profile matrix. `--all-targets` does not cover doctests; `--all-features` is not a substitute for mutually exclusive supported combinations. Record exact effective compiler/rustdoc flags without silently dropping existing cfg or safety options. [Clippy usage](https://doc.rust-lang.org/clippy/usage.html) and [Cargo test target selection](https://doc.rust-lang.org/cargo/commands/cargo-test.html) support these distinctions.

Do not reduce the pedantic profile, blanket-allow lints, cap lint levels, hide stderr, remove failing cases or relabel required checks optional to obtain green. Fix the cause. A genuinely incorrect tool diagnostic needs a narrowly scoped, reviewed disposition identifying the exact lint, affected code, reason, alternative control, owner and revalidation trigger. It must not hide unrelated findings or leave emitted baseline warnings. Existing scoped exemptions are evidence to review, not an automatic waiver of the user's gate. Choose restriction lints individually for real hazards; the whole restriction group can contain conflicting rules.

Expected failure diagnostics belong in isolated compile-fail, hostile, mutation or negative-lint fixtures. The harness must assert the intended error/refusal and verify the surrounding baseline remains clean. These deliberately triggered fixture results are successful negative tests only when the precise oracle is met; they are never unexplained baseline errors.

For Julia, use a pinned formatter/check recipe, locked `Pkg.test(allow_reresolve=false)` where supported by the admitted Julia/Pkg version, and explicit `--check-bounds=yes` in the test process. Unexpected warnings, deprecations, exceptions, skipped/broken tests and missing checks block the applicable baseline. Rust's pedantic category does not exist in Julia; qualify named Julia analysis rules against their own positive and negative controls. For Bash, use `bash -n`, pinned ShellCheck and behavior fixtures with correct argv/quoting, exit propagation, cancellation and cleanup. For schema/skill/Pi/workflow artifacts, validate the admitted parser, schema, host versions and rejection semantics. Applicable warnings/errors remain zero across those baselines; tool availability must be qualified before claiming enforcement.

## Numerical, neural-operator and resource conventions

Numerical and Julia modules must test dimensional/shape/dtype/unit contracts, finite values and invalid inputs, boundary conditions, conservation/symmetry relations where the domain warrants them, calibrated tolerances, stable reference solutions, solver/convergence behavior, cache identity and resource limits. Metamorphic relations and differential baselines must have independent mathematical justification. A solver fitting its own training data or an operator matching a shared flawed reference is not domain validation.

Optional PyTorch/neural-operator work must bind model/checkpoint/data/configuration identity, train/validation/test separation, distribution and resolution shift, deterministic configuration where supported, seed state, device/kernel/runtime versions, numerical tolerances and performance measurements. Check gradient behavior when the admitted operation is differentiable. Record unsupported nondeterministic operations and cross-device limitations. [PyTorch reproducibility](https://docs.pytorch.org/docs/2.14/notes/randomness.html) explicitly limits reproducibility across releases/platforms and between CPU/GPU; an identical seed alone is not equivalence proof.

Device-specific integration tests need the real admitted runtime and hardware. A CPU fixture cannot qualify an unavailable accelerator path. Optional paths stay unqualified or unadmitted until their required environment exists. Resource and performance experiments must declare budgets, sample design and measurement error before evaluating the candidate; this standard supplies no invented universal latency, coverage or numerical threshold.

## Evidence, counting ledger and qualification receipt contract

The future qualification collector must bind the candidate source, test source, fixtures, case inventory, dependency locks, toolchain, schema/configuration and effective feature/profile matrix, including declared transitive inputs. Record hashes, tool/checker versions, explicit argv/effective environment, timestamps, producer exit codes, full diagnostics and raw per-case outcomes. Preserve failures and counter-evidence; do not retain only a summary of selected passing cases.

Each case record requires: case ID; primary module; related module task/requirement/flow/API/IPC/action IDs; contract/invariant; distinct input class; independent oracle; expected outcome; test level; fixture hash; profile applicability; source/test location; and execution/result references. Verify task IDs belong to that module's declared contract set. A proposed primary-credit ledger must prevent duplicate IDs, repeated scenario credits and cross-module laundering of one oracle. The 50-case floor is computed from eligible case records, never an aggregate `executed` field.

The qualification receipt must additionally bind: complete subject/input inventory; case-inventory hash; distinct qualifying case IDs and their profile results; discovered/executed/skip/failure counts; assimilation register and gate controls; mutation diffs/outcomes/denominators/dispositions; lint/compiler/rustdoc/formatter profiles and raw diagnostics; scoped exception review; evidence custody and producer trust boundary; owner/reviewer disposition; and unresolved gaps. Collection must follow T25–T27 custody requirements: candidate-controlled tests, build scripts, macros and package hooks must not author the trusted verdict or modify protected fixtures/evidence.

Maintain independent dimensions for scoped testing observations, the 50-case floor, assimilation, mutation sensitivity, lint baseline and full qualification. Any relevant source/test/fixture/toolchain/profile/policy change invalidates the affected qualification until rechecked. Full qualification requires every applicable dimension and original task criteria; a partial observation cannot close a module or parent release.

**Current implementation limit:** `corpus_evidence.py` schema v1 validates scoped record consistency. An aggregate `executed: 50` is not 50 reviewed cases. It does not enforce complete subject coverage, module-specific task relevance, distinct case oracles, full profile coverage, assimilation, mutation sensitivity, zero diagnostics or producer authentication. Its `current_observation` state must never be relabelled as this standard's qualification. The dedicated qualification receipt/collector described here is **proposed and unavailable**. All module qualification bindings remain **unassessed**, independently of existing observation statuses. No fake case inventory or accepted qualification record is created by this standards update.

## Module loop, orchestration and proof of completion

1. The module owner turns its atlas contracts and risk profile into at least 50 meaningful case specifications, reviewed for duplicate credit and missing obligations. Freeze the candidate/profile/policy identity and acceptance thresholds before interpreting results.
2. The implementation thread builds the authorized vertical slice and its tests. A verifier thread checks independent oracles, case credit and the real boundary; the security thread challenges authority, provenance and evidence custody. Separation of roles is not automatically independence of assumptions.
3. Establish the zero-warning/error baseline and execute required behavioral cases in admitted environments. Record failures, skips, unsupported profiles and uncertain effects precisely.
4. Run seam/rule assimilation, mutation and lint-gate discrimination against the passing candidate. Reproduce defects, repair within scope, rerun affected tests and mandatory integration regressions. Keep negative controls and benign mirrors.
5. The trusted collector retains raw results and checks the case ledger, inputs, profiles and every evidence dimension. A reviewer challenges unexplained survivors, counting choices and missing scope. Failed or unavailable required checks keep qualification open.
6. The module owner admits full testing qualification only when the floor, zero-diagnostic baseline and every applicable criterion pass on the bound subject. The parent orchestrator separately integrates, hardens, installs/rolls back and verifies under T17–T20/T27; child success is insufficient.
7. Run `tools/corpus-sync` after authorized source/evidence changes and require a complete publication receipt followed by `--check`. Update the atlas, vault, ultra map and source-comment anchors from their owners. Documentation repair must not redispatch completed engine effects. Standard changes increment the revision and invalidate affected qualification; new modules inherit the same floor and zero-diagnostic policy.

Completion evidence consists of the real case/diagnostic/assimilation/mutation records and integrated acceptance, not a prose statement that testing is done. The current maintenance command verifies documentation bindings only; no qualification collector is activated by this note.

## Module-specific coverage priorities

Every row requires at least **50 distinct qualifying module-owned cases** and a baseline with **zero warnings and errors**. Priorities guide case design; they are not completed cases or fixed category quotas.

| Module | Planned boundary and risk priorities |
| --- | --- |
| [contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-contracts) | schema/version compatibility; identity/correlation integrity; bounded decode and malformed inputs; typed errors and serialization invariants |
| [task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task) | state transitions and terminal invariants; duplicate/replayed commands; cancellation and deadlines; parent/child acceptance separation |
| [store](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-store) | atomic writes and crash recovery; idempotent replay; schema migration/rollback; stale writers and durable identity |
| [roster](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-roster) | agent/model/service schema admission; capability freshness and identity; unsupported versions and expiry; no authority from registry labels |
| [route](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-route) | capability eligibility and refusal; task-specific quality/cost choices; budget/latency constraints; stale evidence and deterministic fallback |
| [budget](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-budget) | reservation/spend accounting; overflow and boundary arithmetic; concurrent reservation ownership; refund/cancel/exhaustion semantics |
| [worker](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-worker) | provider/adapter request-return contracts; partial streams and malformed output; cancel/deadline/cleanup; no worker self-acceptance |
| [check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check) | independent oracle and exact subject; zero/missing/skipped case refusal; forged or stale evidence; producer/collector custody and negative controls |
| [recovery](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-recovery) | uncertain effects and useful readback; retry without duplicate effects; crash points and preserved obligations; shutdown/rollback ordering |
| [cohort](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-cohort) | thread/agent roster cohesion; ownership and claim transfer; join/fanout/cancellation; parent verification and stale messages |
| [context](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-context) | provenance and authority separation; bounded context and truncation disclosure; prompt-injection boundaries; versioned retrieval/return identity |
| [notify](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-notify) | delivery ordering and correlation; deduplication/replay; bounded subscribers and backpressure; disconnect/recovery without false acceptance |
| [service](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-service) | admission and lifecycle ownership; binary/daemon/library/operator boundaries; health/readback and failure isolation; restart/stop and resource cleanup |
| [herdr](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-herdr) | session/socket/generation binding; pane/agent identity drift; presentation versus engine authority; disconnect/reconnect and roundtrip contracts |
| [numerical](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-numerical) | units/shapes/tolerances; independent reference and metamorphic properties; domain/shift/resource bounds; optional operator admission and evidence provenance |
| [julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-julia) | Rust-Julia request/return framing; locked package/bounds-checked execution; numerical invalid/edge inputs; timeout/cancel and deterministic fixtures |
| [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app) | CLI/serve composition and dependency injection; complete command/error propagation; startup/shutdown/configuration drift; packaged full-stack integration and release readback |
| [actions](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-actions) | command/tool schema equivalence; method authority and effect classification; idempotency/readback/correlation; hostile arguments and consistent errors |
| [bash](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-bash) | literal argv and quoting; producer/pipeline exit status; cancellation and cleanup; schema/version/status preservation |
| [pi_extension](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-pi_extension) | host/extension version contract; tool-call/stream adapter behavior; cancellation and refusal fidelity; context and capability admission |
| [skills](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-skills) | skill provenance/version/context boundary; procedure versus execution authority; required input/output schema; unsafe or incompatible tool-call refusal |
| [workflows](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-workflows) | DAG join and dependency completion; partial failure and skipped stage refusal; fanout cancellation and ownership; complete evidence return and no duplicate effects |

## Evidence base and source limits

The 50-case floor and zero-warning/error rule are user requirements. The sources below support specific techniques and known limitations; they do not prove a performance percentile or qualify this engine.

- [Clippy usage](https://doc.rust-lang.org/clippy/usage.html): Warnings-as-errors, pedantic activation, target scope and restriction-lint caveats.
- [Cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html): Target/profile selection and separate doctest coverage.
- [cargo-mutants outcomes](https://mutants.rs/using-results.html): Passing baseline, distinct mutant outcomes and contract-level regression design.
- [cargo-mutants output records](https://mutants.rs/mutants-out.html): Retain mutation inventory, diffs, logs and native outcome data; archive before tool output rotation.
- [Julia Test](https://docs.julialang.org/en/v1/stdlib/Test/): Broken and skipped tests are separate from passing execution; use explicit case accounting.
- [Julia Pkg API](https://pkgdocs.julialang.org/v1/api/): Version-specific locked test execution and test-process configuration.
- [ShellCheck](https://github.com/koalaman/shellcheck): Shell semantic/quoting diagnostics complement syntax and behavioral tests.
- [Proptest persistence](https://proptest-rs.github.io/proptest/proptest/failure-persistence.html): Retain counterexamples and replay inputs; generator changes affect seed interpretation.
- [Loom](https://docs.rs/loom/latest/loom/): Schedule/model exploration is bounded by modeled primitives and declared limits.
- [PyTorch reproducibility](https://docs.pytorch.org/docs/2.14/notes/randomness.html): Seeds do not establish cross-platform/release equivalence; bind runtime/device details.
- [Assimilation - Rules That Fire](obsidian://open?vault=toolshed.vault&file=60%20Workflows%2FAssimilation%20-%20Rules%20That%20Fire): Enforced adopted rules need negative controls and valid-use mirrors; historical demonstrations retain their scope. Captured SHA-256: `e2343124d050bf7086cc978859cff965c94045e923745079630ed97f4ec245c3`.
- [Weight Matrix - Which Gate Bore the Weight](obsidian://open?vault=toolshed.vault&file=60%20Workflows%2FWeight%20Matrix%20-%20Which%20Gate%20Bore%20the%20Weight): Use passing baselines, intended-fault evidence, isolated targets and explicit unmeasured denominators. Captured SHA-256: `483cc76be214e92831838f7384ab860c1da647d220b6f8857f8d320d4d6af247`.
- [Claim-Time Guard](obsidian://open?vault=toolshed.vault&file=60%20Workflows%2FClaim-Time%20Guard): Preserve checker producer status and evidence at the claim boundary. Captured SHA-256: `940f304792b3a1b2d1cfb55dcee55b7237f3707c886a29d073c9921af8be25bd`.

Primary pages were reviewed on 2026-09-15. Pin exact tool versions and supported options when implementation is authorized. Local field findings retain their original scope and timing.

## Adopted supplementary requirements

[Adopted readiness requirements and evidence roadmap](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FReadiness%2FIndex) · [Codebase readiness plan](file:///var/home/herdr-engineering-engine-v3/docs/readiness-plan.md) · [Six resolved design contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FContracts%2FIndex) · [Concrete contract decisions](file:///var/home/herdr-engineering-engine-v3/docs/contract-decisions.md). These active clauses strengthen the existing owner; none relaxes its baseline or claims implementation.

### F4 · Testing, verification and completion evidence

**Recorded score:** 86/100. **Conditional target:** 93/100; no promotion from documentation adoption.

**Primary owners:** [check](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-check), [contracts](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-contracts), [app](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-app), [task](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-task), [julia](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Ultra%20Map%2FModules%2FUM-julia).

**Owning task contracts:** [T01](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T01), [T25](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T25), [T26](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T26), [T06](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T06), [T17](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Delivery%2FTasks%2FTASK-T17).

**Governing standard/procedure routes:** [Module Testing](obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Standards%2FModule%20Testing). These are generated views of the same adopted clauses; edit the convention once.

- **F4-C01:** Specify a small trusted launch/collection boundary owned by check. Pin the criterion-to-oracle map and evidence schema before implementation; candidate test programs and their structured reports remain untrusted claims.
- **F4-C02:** Bind a receipt to source/tree and dirty-state digest, interface/schema, dependency lockfiles, toolchain/profile, fixture/oracle/harness identities, exact invocation/environment, selected and executed case IDs, raw stdout/stderr, actual exit/signal and observation time.
- **F4-C03:** Separate qualifying baseline results from deliberately failing controls. Zero selected tests, skipped mandatory cases, missing diagnostics, stale artifacts, ambiguous producer outcomes or an unavailable validator must refuse qualification.
- **F4-C04:** Keep the minimum of 50 distinct meaningful primary-owned cases for every accepted module. Map cases to obligations and risk classes; parameter repetitions, seeds, assertions, profiles, retries and mutant runs do not create extra case credit. All 22 completed modules imply at least 1,100 distinct primary credits.
- **F4-C05:** Define assimilation controls from actual adopted lessons: reproduce the hazard in a valid fixture, observe the intended detector, and pass its benign counterpart. A formatting failure before the behavioral fault does not establish behavioral detection.
- **F4-C06:** Qualify mutation operators against real invariants. Retain killed, surviving, equivalent, invalid, timed-out and unexecuted dispositions with denominator and review reasons. Require intended fault detection; never choose an unsupported universal mutation percentage to inflate confidence.
- **F4-C07:** Define the complete Rust/Julia feature, target and optimization matrix. Pedantic Clippy and all other required baseline diagnostics must be clean; no blanket suppression, output filtering, ignored failures or automatic baseline acceptance.

**Required future proof:**

- Bootstrap the collector against a small independently reviewed reference fixture: known pass/fail, benign near-neighbor, forged PASS, wrong subject, changed fixture, missing log, verifier crash and post-check mutation. The collector cannot admit its own trust.
- T06 produces a failed candidate, repairs it and re-verifies the exact repaired subject. Independent acceptance rejects worker exit-zero and unrelated evidence.
- The first admitted module has at least 50 qualifying cases, zero baseline diagnostics and nonempty, valid mutation/assimilation evidence; repeat on a second materially different module or integrated seam.

**Reassessment:** 90–91 requires qualified collection plus one admitted module slice. 93 requires repeatability on another meaningful seam. Whole-release qualification still requires every included module and all integration obligations.

**Complexity guard:** Start with a narrow collector/library or CLI and retained files. Do not create a verification service, second ledger or generic workflow framework.

### Rust/Julia check specifications — unavailable until implemented

Use the following as recipe specifications after a real pinned package exists. They are not currently available engine recipes and were not executed for this proposal. Apply each to the supported feature/target profile, retain its exact identity, and place candidate-controlled build/test processes in the declared qualification domain.

A Rust profile should include formatting, compilation, pedantic Clippy, applicable tests and rustdoc/doctests. Cargo’s explicit `--all-targets` selection covers lib/bins/tests/benches/examples; library doctests need their own route in that arrangement. `--locked` requires an existing unchanged lockfile. See the [Cargo test reference](https://doc.rust-lang.org/cargo/commands/cargo-test.html) and [Clippy usage](https://doc.rust-lang.org/clippy/usage.html).

```text
Proposed Rust recipe bodies, after supported package/profile selection:
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings -D clippy::pedantic
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
```

Do not assume --all-features represents every supported configuration: mutually exclusive features and no-default-feature builds need explicit admissible profiles. Retain complete compiler/build/test diagnostics, compile-doc checks and selected/ignored counts. A zero-warning process from which diagnostics were discarded cannot qualify. Fault-control diagnostics remain separately scoped from the clean baseline.

For Julia, pin the package and manifest, disable startup-file interference, enable bounds checks and treat deprecations as errors in the actual test process. The [Pkg.test API](https://pkgdocs.julialang.org/v1/api/#Pkg.test) supports `allow_reresolve=false` and explicit `julia_args`; select a compatible version and bind the real resolved test environment. Use the [Julia CLI options](https://docs.julialang.org/en/v1/manual/command-line-interface/) to record the exact profile.

A Julia collector must treat required `Broken`/skipped results as nonqualifying. It must collect stderr and structured logs: `@test_nowarn` alone does not check `@warn`; the latter needs `@test_logs` handling. These distinctions are documented in [Julia Test](https://docs.julialang.org/en/v1/stdlib/Test/).

Mutation tooling is a way to challenge an oracle, not a quality score by itself. [cargo-mutants](https://mutants.rs/) is a candidate for Rust; a Julia strategy still needs a scoped, validated choice. Do not pretend that one language’s tool covers both runtimes.

### Qualified fault and benign controls

| Boundary | Intended fault | Benign control | Required observation |
| --- | --- | --- | --- |
| Collector | Candidate prints fabricated PASS and exits zero | Valid candidate with matching independent oracle | Forged report cannot admit; real producer/oracle outcomes retained |
| Identity | Candidate changes after verification | Exact immutable verified subject | Stale subject rejected; matched subject eligible for independent review |
| Submission | Same principal/key with different canonical body | Same principal/key with identical body | Conflict versus same durable task readback |
| Cancellation | Old worker submits after committed cancel intent | Current uncancelled generation returns | No false acceptance; permitted current result can advance |
| IPC | Partial/oversized frame or stale socket generation | Bounded complete frame from admitted peer | Intended parser/custody refusal without loss of task truth |
| Context | Prompt/source requests access beyond grant | Same useful content within admitted scope | Untrusted instructions cannot increase authority |
| Isolation | Build/test hook writes protected evidence | Valid build/test writes only owned outputs | Write refused and independently observed; baseline still runs |
| Restore | Backup omits post-backup external effects or a service | Inventory and effects are fully reconciled | Missing obligation detected; no blind external replay |
| Julia | Stale/nonfinite/missing dataset or timed-out result | Valid immutable dataset with expected tolerance | Typed rejection/advisory failure preserves approved route |
| Cohort | One child done but integrated parent fails | All required joins and parent verifier pass | Parent remains unaccepted until its independent proof |

Each fault must reach its intended detector. If an earlier formatter, compiler or fixture error prevents that observation, classify the experiment as invalid for the deeper gate. Do not turn a negative-control failure into a blanket suppression of the clean release profile.

