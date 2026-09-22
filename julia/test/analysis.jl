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
