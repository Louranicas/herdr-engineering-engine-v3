#!/usr/bin/env julia
# T25 candidate recipe; this process cannot authenticate its own reports.
using Pkg
using SHA
using TOML

function main()
    VERSION == v"1.12.7" || error("Julia 1.12.7 is required")
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
    project = realpath(joinpath(@__DIR__, "..", "julia"))
    Base.active_project() == joinpath(project, "Project.toml") ||
        error("Activate this exact julia/ project before the recipe")
    options = Base.JLOptions()
    options.startupfile == 2 || error("--startup-file=no is required")
    options.check_bounds == 1 || error("--check-bounds=yes is required")
    options.depwarn == 2 || error("--depwarn=error is required")
    Threads.nthreads() == 1 || error("--threads=1 is required")
    length(DEPOT_PATH) == 1 ||
        error("Use one explicit isolated JULIA_DEPOT_PATH without inherited depots")
    get(ENV, "JULIA_PKG_OFFLINE", "") == "true" ||
        error("JULIA_PKG_OFFLINE=true is required")
    get(ENV, "JULIA_PKG_SERVER", "unset") == "" || error("JULIA_PKG_SERVER must be empty")
    get(ENV, "JULIA_PKG_PRECOMPILE_AUTO", "") == "0" ||
        error("JULIA_PKG_PRECOMPILE_AUTO=0 is required")
    get(ENV, "OPENBLAS_NUM_THREADS", "") == "1" ||
        error("OPENBLAS_NUM_THREADS=1 is required")
    get(ENV, "JULIA_LOAD_PATH", "") == "@:@stdlib" ||
        error("JULIA_LOAD_PATH=@:@stdlib is required")
    registry = joinpath(only(DEPOT_PATH), "registries", "HEE3Empty", "Registry.toml")
    registry_fixture =
        joinpath(@__DIR__, "julia-quality-fixtures", "registry", "Registry.toml")
    isfile(registry) && read(registry) == read(registry_fixture) || error(
        "Explicit empty local registry is required; do not download a default registry",
    )
    sort(readdir(dirname(dirname(registry)))) == ["HEE3Empty"] ||
        error("Unexpected registry in isolated depot")
    manifest_path = joinpath(project, "Manifest.toml")
    isfile(manifest_path) ||
        error("Existing Manifest.toml is required; no acceptance-time resolve")
    manifest = TOML.parsefile(manifest_path)
    manifest["julia_version"] == "1.12.7" || error("Manifest Julia version differs")
    expected_self = Dict(
        "path" => ".",
        "uuid" => "27bc5c0a-e2f1-4a83-a4c2-9c324f347a1e",
        "version" => "0.1.0",
    )
    get(manifest, "deps", Dict()) == Dict("HabitatAnalysis" => [expected_self]) || error(
        "This foundation admits only its own local package manifest entry, without external runtime dependencies",
    )
    metadata = TOML.parsefile(joinpath(project, "Project.toml"))
    metadata["compat"]["julia"] == "=1.12.7" ||
        error("Exact Julia compatibility is required")
    isempty(get(metadata, "deps", Dict())) || error("Unexpected runtime dependencies")
    expected_extras = Dict(
        "Test" => "8dfed614-e22c-5e08-85e1-65c5234f0b40",
        "Logging" => "56ddb016-857b-54e1-b83d-db4d58db5568",
        "TOML" => "fa267f1f-6049-4f14-aa54-33bafae1ed76",
        "SHA" => "ea8e919c-243c-51af-8825-aaa63cd721ce",
        "LinearAlgebra" => "37e2e46d-f89d-539d-b4ee-838fcccc9c8e",
    )
    metadata["extras"] == expected_extras || error("Unexpected test dependency")
    metadata["targets"]["test"] == ["Test", "Logging", "TOML", "SHA", "LinearAlgebra"] ||
        error("Unexpected test target")
    Pkg.is_manifest_current(project) === true ||
        error("Manifest is stale; resolve only in separately authorized setup")
    subjects = [
        joinpath(project, "Project.toml"),
        manifest_path,
        joinpath(project, "src", "HabitatAnalysis.jl"),
        joinpath(project, "test", "runtests.jl"),
        abspath(@__FILE__),
        registry_fixture,
    ]
    append!(
        subjects,
        [
            joinpath(@__DIR__, "julia-quality-fixtures", name * ".jl") for
            name in modes if name != "baseline"
        ],
    )
    before = Dict(path => bytes2hex(sha256(read(path))) for path in subjects)
    println("HEE3_JULIA_RECIPE_SUBJECTS_BEGIN")
    TOML.print(
        stdout,
        Dict(
            "mode" => mode,
            "subjects_sha256" => before,
            "depot" => only(DEPOT_PATH),
            "scope" => "candidate recipe, not protected collection",
        );
        sorted = true,
    )
    println("HEE3_JULIA_RECIPE_SUBJECTS_END")
    flush(stdout)
    Pkg.offline(true)
    try
        # The default Pkg IO is stderr. Selecting stdout keeps child stdout and
        # stderr on separate captured parent streams (Pkg API, pinned 1.12.7).
        Pkg.test(;
            io = stdout,
            allow_reresolve = false,
            julia_args = [
                "--startup-file=no",
                "--check-bounds=yes",
                "--depwarn=error",
                "--threads=1",
            ],
            test_args = [mode],
        )
    finally
        for path in subjects
            bytes2hex(sha256(read(path))) == before[path] ||
                error("Julia quality subject changed during testing: " * path)
        end
    end
    println(
        "HEE3_JULIA_RECIPE_TEST_SUCCESS: package/test foundation only; formatting and full qualification unavailable",
    )
    return nothing
end

main()
