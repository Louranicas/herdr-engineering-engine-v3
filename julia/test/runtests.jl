using Test
using Logging
using TOML
using LinearAlgebra
using SHA

# This producer runs as candidate-controlled code. These counts are claims,
# never a protected collector verdict, discovered case inventory, or admission.
mutable struct FoundationLogger <: AbstractLogger
    parent::AbstractLogger
    records::Vector{Dict{String,Any}}
end

Logging.min_enabled_level(logger::FoundationLogger) =
    Logging.min_enabled_level(logger.parent)
Logging.shouldlog(logger::FoundationLogger, level, mod, group, id) =
    level >= Logging.Warn || Logging.shouldlog(logger.parent, level, mod, group, id)
Logging.catch_exceptions(::FoundationLogger) = false

function Logging.handle_message(
    logger::FoundationLogger,
    level,
    message,
    mod,
    group,
    id,
    file,
    line;
    kwargs...,
)
    if level >= Logging.Warn
        push!(
            logger.records,
            Dict{String,Any}(
                "level" => string(level),
                "message" => string(message),
                "module" => string(mod),
                "file" => string(file),
                "line" => line,
            ),
        )
    end
    return Logging.handle_message(
        logger.parent,
        level,
        message,
        mod,
        group,
        id,
        file,
        line;
        kwargs...,
    )
end

function skipped_results(testset::Test.DefaultTestSet)
    total = 0
    for result in testset.results
        if result isa Test.Broken && result.test_type === :skipped
            total += 1
        elseif result isa Test.DefaultTestSet
            total += skipped_results(result)
        end
    end
    return total
end

function run_foundation_tests()
    modes = (
        "baseline",
        "assertion",
        "warning",
        "skip",
        "broken",
        "bounds",
        "deprecation",
        "empty",
    )
    mode = isempty(ARGS) ? "baseline" : only(ARGS)
    mode in modes || error("Unsupported Julia quality control")
    VERSION == v"1.12.7" || error("Julia 1.12.7 is required")
    options = Base.JLOptions()
    options.startupfile == 2 || error("--startup-file=no is required in the test process")
    options.check_bounds == 1 || error("--check-bounds=yes is required in the test process")
    options.depwarn == 2 || error("--depwarn=error is required in the test process")
    Threads.nthreads() == 1 || error("Exactly one Julia worker thread is required")
    Threads.nthreads(:interactive) == 0 ||
        error("Interactive Julia threads are not in this profile")
    LinearAlgebra.BLAS.get_num_threads() == 1 ||
        error("Exactly one BLAS thread is required")
    logger = FoundationLogger(ConsoleLogger(stderr), Dict{String,Any}[])
    root = Test.DefaultTestSet("T25 Julia package foundation"; showtiming = false)
    Test.push_testset(root)
    try
        with_logger(logger) do
            @testset "selected fixture: $mode" begin
                loaded_package = Base.require(Main, :HabitatAnalysis)
                if mode == "baseline"
                    # One package-load smoke assertion; no invented analysis API.
                    @test Base.PkgId(loaded_package).name == "HabitatAnalysis"
                else
                    include(
                        joinpath(
                            @__DIR__,
                            "..",
                            "..",
                            "tools",
                            "julia-quality-fixtures",
                            mode * ".jl",
                        ),
                    )
                end
            end
        end
    finally
        Test.pop_testset()
    end
    try
        Test.finish(root)
    catch exception
        exception isa Test.TestSetException || rethrow()
        # Test already printed each failure/error and its original backtrace.
    end
    counts = Test.get_test_counts(root)
    passes = counts.passes + counts.cumulative_passes
    failures = counts.fails + counts.cumulative_fails
    errors = counts.errors + counts.cumulative_errors
    all_broken = counts.broken + counts.cumulative_broken
    skipped = skipped_results(root)
    broken = all_broken - skipped
    total = passes + failures + errors + all_broken
    warnings = count(record -> record["level"] == "Warn", logger.records)
    error_logs = length(logger.records) - warnings
    qualifying_baseline =
        total > 0 &&
        failures == 0 &&
        errors == 0 &&
        all_broken == 0 &&
        warnings == 0 &&
        error_logs == 0
    test_project_path = Base.active_project()
    test_manifest_path = joinpath(dirname(test_project_path), "Manifest.toml")
    test_project_bytes = read(test_project_path)
    test_manifest_bytes = read(test_manifest_path)
    summary = Dict{String,Any}(
        "scope" => "candidate-controlled Julia package/test foundation; no module or T25 admission",
        "mode" => mode,
        "julia_version" => string(VERSION),
        "selected_results" => total,
        "executed_results" => total - skipped,
        "passed" => passes,
        "failed" => failures,
        "errors" => errors,
        "broken" => broken,
        "skipped" => skipped,
        "warnings" => warnings,
        "error_logs" => error_logs,
        "baseline_checks_passed" => qualifying_baseline,
        "module_case_credits" => 0,
        "formatting_qualified" => false,
        "startup_file_disabled" => options.startupfile == 2,
        "bounds_checks_enabled" => options.check_bounds == 1,
        "deprecations_are_errors" => options.depwarn == 2,
        "julia_threads" => Threads.nthreads(),
        "blas_threads" => LinearAlgebra.BLAS.get_num_threads(),
        "structured_logs" => logger.records,
        "test_project_path" => test_project_path,
        "test_project_sha256" => bytes2hex(sha256(test_project_bytes)),
        "test_manifest_sha256" => bytes2hex(sha256(test_manifest_bytes)),
        "test_project_toml" => String(test_project_bytes),
        "test_manifest_toml" => String(test_manifest_bytes),
        "limitations" => [
            "Only the benign package-load smoke case is baseline behavior; no numerical behavior is implemented.",
            "Counts are observed Test results, not an independent discovery or selected-case inventory.",
            "Raw stdout/stderr and producer exit require independent collection and review.",
            "The logger sees ordinary structured logs; arbitrary output, replaced loggers or forged reports can evade this candidate process.",
            "Formatter/static-analysis tooling, full module cases and hostile isolation remain unqualified.",
        ],
    )
    println("HEE3_JULIA_TEST_SUMMARY_BEGIN")
    TOML.print(stdout, summary; sorted = true)
    println("HEE3_JULIA_TEST_SUMMARY_END")
    qualifying_baseline || error(
        "HEE3_JULIA_BASELINE_REFUSED: failures, errors, warnings, broken/skipped or zero test results",
    )
    return nothing
end

run_foundation_tests()
