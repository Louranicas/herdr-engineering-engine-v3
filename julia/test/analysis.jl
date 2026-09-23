using HabitatAnalysis, JSON3, Test, LinearAlgebra, Logging
BLAS.set_num_threads(1)
const NOW = UInt64(1_769_999_995_000)
const RAW = read(joinpath(@__DIR__, "../../tests/fixtures/t21/J01.json"))
fixture() = JSON3.read(String(copy(RAW)), Dict{String,Any})
bytes(q) = Vector{UInt8}(codeunits(JSON3.write(q)))
function rejection(f)
    q=fixture()
    f(q)
    @test_throws AnalysisError analyze(bytes(q), NOW)
end
function reject_at(path, value)
    rejection() do q
        at=q
        for key in path[1:(end-1)]
            at=at[key]
        end
        at[path[end]]=value
    end
end
@testset "T21 descriptive evaluator" begin
    @testset "profile" begin
        @test VERSION == v"1.12.7"
        @test Base.JLOptions().check_bounds == 1
        @test Base.JLOptions().depwarn == 2
        @test Threads.nthreads() == 1
        @test BLAS.get_num_threads() == 1
    end
    @testset "J01 independent fixed oracle" begin
        result=analyze(RAW, NOW)
        r=JSON3.read(result)
        @test r.request_sha256 ==
              "sha256:3aa532ed778498457dd105e0acdf5d01ff4dd2673fa081755f10c7083a2620c7"
        @test r.counts.total == "5"
        for key in (:accepted, :failed, :cancelled, :abandoned, :running)
            @test r.counts[key]=="1"
        end
        @test r.counts.unknown_usage == "3"
        @test r.counts.censored == "1"
        @test r.counts.known_usage_sum == "4"
        @test r.acceptance_fraction == 0.2
        @test r.mean_observed_ms == 30.0
        @test result == analyze(RAW, NOW)
        write(ENV["T21_JULIA_REPORT"], result)
    end
    for (name, path, value) in [
        ("protocol", ["protocol"], "hee3.control"),
        ("version", ["version"], 2),
        ("boolean version", ["version"], true),
        ("float version", ["version"], 1.0),
        ("request ID", ["request_id"], "1"),
        ("task ID", ["subject", "task_id"], "1"),
        ("subject attempt", ["subject", "attempt_id"], "1"),
        ("numeric generation", ["subject", "generation"], 3),
        ("leading generation", ["subject", "generation"], "03"),
        ("overflow generation", ["subject", "generation"], "18446744073709551616"),
        ("bare digest", ["subject", "artifact_sha256"], repeat("2", 64)),
        ("future cutoff", ["cutoff_unix_ms"], "1770000000000"),
        ("expired", ["expires_unix_ms"], "1769999995000"),
        ("numeric cutoff", ["cutoff_unix_ms"], 1769999990000),
        ("recipe", ["recipe", "id"], "regression"),
        ("recipe version", ["recipe", "version"], 2),
        ("elapsed unit", ["units", "elapsed"], "s"),
        ("usage unit", ["units", "usage"], "tokens"),
        ("zero rows", ["shape", "rows"], 0),
        ("excessive rows", ["shape", "rows"], 4097),
        ("shape mismatch", ["shape", "rows"], 4),
        ("wrong fields", ["shape", "fields"], 4),
        ("bool rows", ["shape", "rows"], true),
        ("empty rows", ["observations"], []),
        (
            "duplicate ID",
            ["observations", 2, "attempt_id"],
            "123e4567-e89b-42d3-a456-000000000021",
        ),
        ("unknown outcome", ["observations", 1, "outcome"], "success"),
        ("negative elapsed", ["observations", 1, "elapsed_ms"], -1),
        ("overflow elapsed", ["observations", 1, "elapsed_ms"], 86400001),
        ("bool elapsed", ["observations", 1, "elapsed_ms"], true),
        ("string elapsed", ["observations", 1, "elapsed_ms"], "10"),
        ("numeric usage", ["observations", 1, "usage_tokens"], 4),
        ("negative usage", ["observations", 1, "usage_tokens"], "-1"),
        ("overflow usage", ["observations", 1, "usage_tokens"], "4294967296"),
        ("leading usage", ["observations", 1, "usage_tokens"], "04"),
        ("accepted censored", ["observations", 1, "censored"], true),
        ("running uncensored", ["observations", 5, "censored"], false),
    ]
        @testset "$name" begin
            reject_at(path, value)
        end
    end
    @testset "missing nullable usage" begin
        rejection(q->delete!(q["observations"][1], "usage_tokens"))
    end
    @testset "unknown top field" begin
        rejection(q->q["extra"]=nothing)
    end
    @testset "unknown row field" begin
        rejection(q->q["observations"][1]["extra"]=nothing)
    end
    @testset "missing subject" begin
        rejection(q->delete!(q, "subject"))
    end
    @testset "actual excessive rows" begin
        rejection(q->begin
            q["observations"]=fill(q["observations"][1], 4097)
            q["shape"]["rows"]=4097
        end)
    end
    for key in ("protocol", "generation", "id", "usage", "rows", "elapsed_ms")
        @testset "duplicate $key" begin
            raw=replace(
                String(copy(RAW)),
                "\"$key\":"=>"\"$key\":null,\"$key\":";
                count = 1,
            )
            @test_throws AnalysisError analyze(Vector{UInt8}(codeunits(raw)), NOW)
        end
    end
    @testset "escaped duplicate" begin
        raw=replace(
            String(copy(RAW)),
            "\"protocol\":"=>"\"pr\\u006ftocol\":null,\"protocol\":";
            count = 1,
        )
        @test_throws AnalysisError analyze(Vector{UInt8}(codeunits(raw)), NOW)
    end
    for bad in [
        vcat(RAW, RAW),
        RAW[1:(end-1)],
        UInt8[0x7b, 0xff, 0x7d],
        fill(UInt8(' '), 1_048_577),
        fill(UInt8('['), 33),
    ]
        @testset "framing $(length(bad)) $(bad[1])" begin
            @test_throws AnalysisError analyze(bad, NOW)
        end
    end
    for literal in ("NaN", "Infinity", "1e999")
        @testset "nonfinite $literal" begin
            raw=replace(String(copy(RAW)), "10.0"=>literal; count = 1)
            @test_throws AnalysisError analyze(Vector{UInt8}(codeunits(raw)), NOW)
        end
    end
    @testset "zero becomes unknown" begin
        q=fixture()
        q["observations"][3]["usage_tokens"]=nothing
        r=JSON3.read(analyze(bytes(q), NOW))
        @test r.counts.unknown_usage=="4"
        @test r.counts.known_usage_sum=="4"
    end
    @testset "unknown becomes zero" begin
        q=fixture()
        q["observations"][2]["usage_tokens"]="0"
        r=JSON3.read(analyze(bytes(q), NOW))
        @test r.counts.unknown_usage=="2"
        @test r.counts.known_usage_sum=="4"
    end
    @testset "relabel includes every row" begin
        q=fixture()
        q["observations"][1]["outcome"]="failed"
        r=JSON3.read(analyze(bytes(q), NOW))
        @test r.acceptance_fraction==0.0
        @test r.mean_observed_ms==30.0
    end
    @testset "whitespace remains bound" begin
        r=JSON3.read(analyze(vcat(UInt8[0x20, 0x0a], RAW, UInt8[0x09]), NOW))
        @test r.request_sha256 != JSON3.read(analyze(RAW, NOW)).request_sha256
    end
    @testset "all usage bounded sum" begin
        q=fixture()
        for row in q["observations"]
            row["usage_tokens"]="4294967295"
        end
        r=JSON3.read(analyze(bytes(q), NOW))
        @test r.counts.known_usage_sum=="21474836475"
    end
end

@testset "T21 numerical boundary controls" begin
    @testset "full row limit compensated sum" begin
        q=fixture()
        prototype=q["observations"][1]
        q["observations"]=[
            merge(
                prototype,
                Dict(
                    "attempt_id"=>"123e4567-e89b-42d3-a456-"*string(i; base = 16, pad = 12),
                    "elapsed_ms"=>(i==0 ? 86_400_000.0 : 1e-8),
                ),
            ) for i = 0:4095
        ]
        q["shape"]["rows"]=4096
        r=JSON3.read(analyze(bytes(q), NOW))
        expected=Float64((big"86400000"+4095*big"0.00000001")/4096)
        @test abs(r.mean_observed_ms-expected)<=8*eps(Float64)*expected
        @test r.counts.total=="4096"
        @test r.counts.known_usage_sum=="16384"
    end
    @testset "maximum request whitespace" begin
        raw=vcat(RAW, fill(UInt8(' '), 1_048_576-length(RAW)))
        @test JSON3.read(analyze(raw, NOW)).counts.total=="5"
    end
    @testset "escaped integer key float" begin
        raw=replace(String(copy(RAW)), "\"version\":1"=>"\"ver\\u0073ion\":1.0"; count = 1)
        @test_throws AnalysisError analyze(Vector{UInt8}(codeunits(raw)), NOW)
    end
    for value in ("01", "+1", "1.", "1e", "--1")
        @testset "malformed number $value" begin
            raw=replace(String(copy(RAW)), "10.0"=>value; count = 1)
            @test_throws AnalysisError analyze(Vector{UInt8}(codeunits(raw)), NOW)
        end
    end
end

@testset "T21 deferred string decoding" begin
    for (name, original, replacement) in [
        ("lone high surrogate", "hee3.analysis", "hee3.analysis\\ud800"),
        ("lone low surrogate", "hee3.analysis", "hee3.analysis\\udc00"),
        ("invalid escape", "hee3.analysis", "hee3.analysis\\q"),
        ("nested invalid escape", "descriptive", "descriptive\\ud800"),
        ("row invalid escape", "accepted", "accepted\\ud800"),
    ]
        @testset "$name" begin
            changed=replace(String(copy(RAW)), original=>replacement; count = 1)
            @test_throws AnalysisError analyze(Vector{UInt8}(codeunits(changed)), NOW)
        end
    end
    @testset "valid decoded escape remains accepted and raw-bound" begin
        changed=replace(
            String(copy(RAW)),
            "hee3.analysis"=>"hee3.analysi\\u0073";
            count = 1,
        )
        result=JSON3.read(analyze(Vector{UInt8}(codeunits(changed)), NOW))
        @test result.protocol=="hee3.analysis"
        @test result.request_sha256!=JSON3.read(analyze(RAW, NOW)).request_sha256
    end
end

# ---------------------------------------------------------------------------------------
# T22 cohort cohesion. Appended to the T21 file because `tools/julia-quality.jl` hashes
# `test/analysis.jl` as a recipe subject and a new test file would be outside that set.
# ---------------------------------------------------------------------------------------

const C01 = read(joinpath(@__DIR__, "../../tests/fixtures/t22/C01.json"))
const C02 = read(joinpath(@__DIR__, "../../tests/fixtures/t22/C02.json"))
const C_NOW = UInt64(1_769_999_500_000)
oracle(name) = JSON3.read(
    read(joinpath(@__DIR__, "../../tests/fixtures/t22/$name-known-answers.json"), String),
)

"""Assert one report field-for-field against answers computed outside this language.

`fixtures/make-C02.py` implements the T22 rule in Python and never reads `Cohesion.jl`, so a
disagreement here names a defect on one side or the other -- establish which before editing
either (F94). The two fixtures differ in every computed field, and C02 exists because C01
leaves `overlapping_claims` empty and `errors_delta` at zero, which pin nothing (F129)."""
function against_oracle(raw, want)
    r = JSON3.read(cohesion(raw, C_NOW))
    @test r.request_sha256 == want.request_sha256
    for key in (
        :threads,
        :required,
        :met,
        :unmet,
        :dissent,
        :indeterminate,
        :rework,
        :unknown_cost,
    )
        @test r.counts[key] == want.counts[key]
    end
    for key in (:met, :dissent, :rework, :error)
        @test r.rates[key].value == want.rates[key]
        @test r.rates[key].of == want.rates.of
    end
    @test r.allocation.accounted_tokens == want.allocation_accounted_tokens
    @test r.allocation.conserved === true
    @test r.comparison.cohort.cost_tokens == want.comparison.cohort_cost_tokens
    @test r.comparison.cohort.errors == want.comparison.cohort_errors
    @test r.comparison.cohort.disagreement == want.comparison.cohort_disagreement
    @test r.comparison.cohort.rework == want.comparison.cohort_rework
    @test r.comparison.cost_delta_tokens == want.comparison.cost_delta_tokens
    @test r.comparison.errors_delta == want.comparison.errors_delta
    @test r.comparison.rework_delta == want.comparison.rework_delta
    @test String.(r.overlapping_claims) == String.(want.overlapping_claims)
    @test String.(r.excluded) == String.(want.excluded)
    return r
end

cu(n) = string(lpad(string(n, base = 16), 8, '0'), "-0000-4000-8000-000000000000")
cbase() = JSON3.read(String(copy(C01)), Dict{String,Any})
function crun(mutate)
    q = cbase()
    mutate(q)
    raw = Vector{UInt8}(codeunits(JSON3.write(q) * "\n"))
    try
        cohesion(raw, C_NOW)
        return :NO_REFUSAL
    catch e
        e isa AnalysisError ? e.code : Symbol("WRONG_EXCEPTION_", typeof(e))
    end
end
function crun_raw(edit)
    raw = Vector{UInt8}(edit(JSON3.write(cbase()) * "\n"))
    try
        cohesion(raw, C_NOW)
        return :NO_REFUSAL
    catch e
        e isa AnalysisError ? e.code : Symbol("WRONG_EXCEPTION_", typeof(e))
    end
end

@testset "T22 cohort cohesion" begin
    @testset "C01 independent fixed oracle" begin
        r = against_oracle(C01, oracle("C01"))
        @test r.protocol == "hee3.cohesion"
        # The join is derived from the rows by `cohort::join`'s rule and must equal the
        # request's declaration; the expected reasons are the ones Rust computes over these
        # rows (`tests/t22_cohort.rs::fixture_joins_agree_with_cohort_join`), not a second
        # hand-typed list.
        @test r.join.verdict == "blocked"
        @test String.(r.join.reasons) == ["unmet", "dissent", "stale-brief"]
        @test cohesion(C01, C_NOW) == cohesion(C01, C_NOW + UInt64(1000))
    end
    @testset "C02 independent fixed oracle" begin
        r = against_oracle(C02, oracle("C02"))
        @test r.join.verdict == "blocked"
        @test String.(r.join.reasons) == ["dissent", "stale-brief"]
        @test cohesion(C02, C_NOW) == cohesion(C02, C_NOW + UInt64(1000))
        # The two fixtures must not agree anywhere the rule computes, or one of them is
        # pinning the other's answer rather than the rule.
        @test JSON3.read(cohesion(C01, C_NOW)).counts != r.counts
    end
    @testset "claim overlap agrees with the shared table" begin
        # The Rust half is `tests/t22_cohort.rs::claim_overlap_agrees_with_the_shared_table`.
        # Both read this file; neither owns it. `evaluation/cohorts/make-claim-overlap.py`
        # generates it from a third statement of the rule and `--check` re-derives it.
        table = JSON3.read(
            read(
                joinpath(@__DIR__, "../../evaluation/cohorts/claim-overlap-v1.json"),
                String,
            ),
        )
        @test table.schema == "hee3.evaluation.claim-overlap.v1"
        @test length(table.cases) >= 20
        answers = Set{Bool}()
        for case in table.cases
            push!(answers, case.overlap)
            @test HabitatAnalysis.claims_conflict(case.a, case.b) == case.overlap
            @test HabitatAnalysis.claims_conflict(case.b, case.a) == case.overlap
        end
        @test answers == Set([true, false])
    end
    # The join is derived, so it must be pinned at more than the two fixtures' values: a
    # report that echoed one constant would pass both. Each case edits C01's rows, declares
    # what `cohort::join` yields for them, and requires the report to carry exactly that.
    # C01's rows: 1 met, 2 met, 3 dissent, 5 indeterminate (required); 4 unmet, 6 met
    # (optional); 6 and 7 assigned on briefs 2 and 3 against the cohort's 4.
    @testset "the join is derived from the rows" begin
        current =
            q -> (
                q["threads"][6]["brief_revision"] = "4";
                q["threads"][7]["brief_revision"] = "4"
            )
        settled =
            q -> (
                current(q);
                q["threads"][3]["outcome"] = "met";
                q["threads"][5]["outcome"] = "met"
            )
        for (name, mutate, verdict, reasons) in [
            ("stale rows made current", current, "blocked", ["unmet", "dissent"]),
            (
                "every required row met on the current brief",
                settled,
                "integrable",
                String[],
            ),
            (
                "an optional dissent still blocks",
                q -> (settled(q); q["threads"][4]["outcome"] = "dissent"),
                "blocked",
                ["dissent"],
            ),
            (
                "a required indeterminate blocks as unmet",
                q -> (settled(q); q["threads"][5]["outcome"] = "indeterminate"),
                "blocked",
                ["unmet"],
            ),
            (
                "a stale dissent is stale, not dissent",
                q -> (
                    settled(q);
                    q["threads"][3]["outcome"] = "dissent";
                    q["threads"][3]["brief_revision"] = "3"
                ),
                "blocked",
                ["stale-brief"],
            ),
        ]
            @testset "$name" begin
                q = cbase()
                mutate(q)
                q["join"]["verdict"] = verdict
                q["join"]["reasons"] = reasons
                r = JSON3.read(
                    cohesion(Vector{UInt8}(codeunits(JSON3.write(q) * "\n")), C_NOW),
                )
                @test r.join.verdict == verdict
                @test String.(r.join.reasons) == reasons
                # The same rows under any other well-formed declaration are refused, or the
                # acceptance above would hold for an echo as well as for a derivation.
                other =
                    isempty(reasons) ? ("blocked", ["dissent"]) : ("integrable", String[])
                @test crun(function (p)
                    mutate(p)
                    p["join"]["verdict"] = other[1]
                    p["join"]["reasons"] = other[2]
                end) == :join
            end
        end
    end
    # Every refusal SITE in Cohesion.jl, one case each, asserting its own symbol. The site
    # set was enumerated from the module's own source and each case verified to change its
    # answer when its site alone is neutered -- a case per symbol NAME would have left eight
    # sites covered by a neighbour raising the same symbol (F140).
    @testset "refusals" begin
        for (name, mutate, expected) in [
            ("a well-formed request is quiet", q -> nothing, :NO_REFUSAL),
            ("unknown protocol", q -> q["protocol"] = "hee3.other", :schema),
            ("unknown recipe", q -> q["recipe"]["id"] = "descriptive", :schema),
            ("wrong usage unit", q -> q["units"]["usage"] = "microcent", :schema),
            ("extra top-level key", q -> q["surprise"] = 1, :schema),
            ("malformed cohort id", q -> q["subject"]["cohort_id"] = "nope", :identity),
            (
                "malformed artifact digest",
                q -> q["subject"]["artifact_sha256"] = "nope",
                :identity,
            ),
            ("expired request", q -> q["expires_unix_ms"] = "1769999000001", :stale),
            (
                "request from before its own cutoff",
                q -> q["cutoff_unix_ms"] = "1769999600000",
                :stale,
            ),
            ("unknown outcome name", q -> q["threads"][1]["outcome"] = "approved", :schema),
            (
                "thread brief ahead of the cohort",
                q -> q["threads"][1]["brief_revision"] = "5",
                :domain,
            ),
            (
                "duplicate thread identity",
                q -> (q["threads"][2]["thread_id"] = q["threads"][1]["thread_id"]),
                :identity,
            ),
            (
                "allocation does not conserve",
                q -> q["allocation"]["spent_tokens"] = "9000",
                :conservation,
            ),
            (
                "integrable join carrying a reason",
                q -> (
                    q["join"]["verdict"] = "integrable";
                    q["join"]["reasons"] = ["dissent"]
                ),
                :domain,
            ),
            (
                "blocked join carrying no reason",
                q -> (q["join"]["verdict"] = "blocked"; q["join"]["reasons"] = []),
                :domain,
            ),
            (
                "unknown join verdict",
                q -> (q["join"]["verdict"] = "unknown"; q["join"]["reasons"] = []),
                :schema,
            ),
            (
                "unknown blocked reason",
                q -> q["join"]["reasons"] = ["whatever", "unmet"],
                :schema,
            ),
            (
                "duplicate blocked reason",
                q -> q["join"]["reasons"] = ["dissent", "dissent"],
                :duplicate,
            ),
            ("reasons is not a list", q -> q["join"]["reasons"] = "dissent", :schema),
            (
                "declared integrable over rows that block",
                q -> (q["join"]["verdict"] = "integrable"; q["join"]["reasons"] = []),
                :join,
            ),
            (
                "declared join omits a reason the rows carry",
                q -> q["join"]["reasons"] = ["unmet", "dissent"],
                :join,
            ),
            (
                "declared join names a reason the rows do not carry",
                q ->
                    q["join"]["reasons"] =
                        ["missing-child", "unmet", "dissent", "stale-brief"],
                :join,
            ),
            (
                "declared reasons out of the rule's order",
                q -> q["join"]["reasons"] = ["dissent", "unmet", "stale-brief"],
                :join,
            ),
            (
                "more reasons than the vocabulary",
                q ->
                    q["join"]["reasons"] =
                        ["dissent", "unmet", "missing-child", "stale-brief", "dissent"],
                :bound,
            ),
            ("shape rows disagree with threads", q -> q["shape"]["rows"] = 9, :schema),
            ("shape fields disagree with the row", q -> q["shape"]["fields"] = 5, :schema),
            (
                "threads is not a list",
                q -> (q["threads"] = Dict{String,Any}(); q["shape"]["rows"] = 0),
                :schema,
            ),
            ("empty thread list", q -> (q["threads"] = []; q["shape"]["rows"] = 0), :bound),
            ("non-boolean required", q -> q["threads"][1]["required"] = "yes", :schema),
            ("empty claim path", q -> q["threads"][1]["claims"] = [""], :schema),
            ("claims is not a list", q -> q["threads"][1]["claims"] = "", :schema),
            (
                "too many claims on one thread",
                q -> q["threads"][1]["claims"] = ["p/$i" for i = 1:65],
                :bound,
            ),
            (
                "report exceeds the admitted bound",
                q -> begin
                    q["threads"] = [
                        Dict{String,Any}(
                            "thread_id" => cu(0x100 + i),
                            "outcome" => "met",
                            "brief_revision" => "4",
                            "required" => true,
                            "cost_tokens" => "1",
                            "claims" => ["src/store"],
                        ) for i = 1:64
                    ]
                    q["shape"]["rows"] = 64
                    q["allocation"]["spent_tokens"] = "64"
                    # Sixty-four current met rows join; the declaration must say so, or the
                    # join site refuses before the report is ever sized.
                    q["join"]["verdict"] = "integrable"
                    q["join"]["reasons"] = []
                end,
                :bound,
            ),
        ]
            @testset "$name" begin
                @test crun(mutate) == expected
            end
        end
        # Two sites that a Julia Dict cannot reach: it holds neither invalid UTF-8 nor an
        # unpaired escape, so these are planted in the encoded bytes.
        @testset "invalid UTF-8 in the payload" begin
            @test crun_raw(function (text)
                bytes = Vector{UInt8}(codeunits(text))
                bytes[findfirst(==(UInt8('t')), bytes)] = 0xff
                bytes
            end) == :encoding
        end
        @testset "unpaired surrogate escape" begin
            @test crun_raw(
                text -> replace(text, "hee3.cohesion" => "hee3.cohesio\\ud800"; count = 1),
            ) == :encoding
        end
    end
end
